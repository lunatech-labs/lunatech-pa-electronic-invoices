//! Ingestion de l'annuaire PPF : orchestre le parsing streaming F14
//! et l'écriture en batch dans PostgreSQL dans une transaction unique.
//!
//! Aucune accumulation en mémoire : chaque batch de 5000 éléments est
//! flushed directement dans la transaction en cours de parsing.

use std::io::BufRead;
use tracing::info;

use crate::db::AnnuaireStore;
use crate::model::*;
use crate::parser::{self, F14Event, ParseError};

const BATCH_SIZE: usize = 5000;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Erreur parsing : {0}")]
    Parse(#[from] ParseError),
    #[error("Erreur DB : {0}")]
    Db(#[from] crate::db::DbError),
    #[error("Erreur SQL : {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Incohérence horodatage : attendu {expected}, reçu {received}")]
    HorodateMismatch { expected: String, received: String },
}

/// Ingère un flux F14 complet ou différentiel dans la base.
///
/// Le parsing et l'insertion se font en streaming : les éléments sont
/// accumulés par batch de 5000 puis flushés dans une transaction PostgreSQL.
/// Soit tout l'import réussit, soit la base reste inchangée.
pub async fn ingest_f14<R: BufRead>(
    reader: R,
    store: &AnnuaireStore,
    expect_horodate: Option<&str>,
) -> Result<ImportStats, IngestError> {
    // Phase 1 : parsing streaming — accumule par batch et flush
    let mut batches = BatchCollector::new();

    let stats = parser::parse_f14(reader, |event| {
        match event {
            F14Event::Header(h) => batches.header = Some(h),
            F14Event::UniteLegale(ul) => batches.ul.push(ul),
            F14Event::Etablissement(etab) => batches.etab.push(etab),
            F14Event::CodeRoutage(cr) => batches.cr.push(cr),
            F14Event::Plateforme(pf) => batches.pf.push(pf),
            F14Event::LigneAnnuaire(la) => batches.la.push(la),
        }
        Ok(())
    })?;

    // Vérifier l'en-tête
    let h = batches.header.as_ref().ok_or_else(|| {
        ParseError::MissingField("Header F14 manquant".into())
    })?;

    info!(
        "F14 parsé : type={:?}, horodate={}, {} UL, {} étab, {} CR, {} PF, {} LA",
        h.type_flux, h.horodate_production,
        stats.unites_legales, stats.etablissements, stats.codes_routage,
        stats.plateformes, stats.lignes_annuaire,
    );

    // Vérification horodatage pour les flux différentiels
    if let (Some(expected), Some(dernier)) =
        (expect_horodate, h.dernier_horodate_production.as_deref())
    {
        if expected != dernier {
            return Err(IngestError::HorodateMismatch {
                expected: expected.to_string(),
                received: dernier.to_string(),
            });
        }
    }

    // Phase 2 : insertion dans une transaction unique
    info!("Démarrage de la transaction PostgreSQL...");
    let mut tx = store.begin().await?;

    // Flux complet : vider les tables
    if h.type_flux == TypeFlux::Complet {
        info!("Flux complet : vidage des tables...");
        sqlx::query("TRUNCATE lignes_annuaire, codes_routage, etablissements, plateformes, unites_legales")
            .execute(&mut *tx)
            .await?;
    }

    let mut total_stats = ImportStats::default();

    // Plateformes
    info!("Insertion des plateformes ({})...", batches.pf.len());
    for chunk in batches.pf.chunks(BATCH_SIZE) {
        insert_plateformes_tx(&mut tx, chunk).await?;
    }
    total_stats.plateformes = batches.pf.len();

    // Unités légales
    info!("Insertion des unités légales ({})...", batches.ul.len());
    for chunk in batches.ul.chunks(BATCH_SIZE) {
        insert_unites_legales_tx(&mut tx, chunk).await?;
    }
    total_stats.unites_legales = batches.ul.len();

    // Établissements
    info!("Insertion des établissements ({})...", batches.etab.len());
    for chunk in batches.etab.chunks(BATCH_SIZE) {
        insert_etablissements_tx(&mut tx, chunk).await?;
    }
    total_stats.etablissements = batches.etab.len();

    // Codes routage
    info!("Insertion des codes routage ({})...", batches.cr.len());
    for chunk in batches.cr.chunks(BATCH_SIZE) {
        insert_codes_routage_tx(&mut tx, chunk).await?;
    }
    total_stats.codes_routage = batches.cr.len();

    // Lignes d'annuaire
    info!("Insertion des lignes d'annuaire ({})...", batches.la.len());
    for chunk in batches.la.chunks(BATCH_SIZE) {
        insert_lignes_annuaire_tx(&mut tx, chunk).await?;
    }
    total_stats.lignes_annuaire = batches.la.len();

    // Métadonnées de synchro
    let type_flux_str = match h.type_flux {
        TypeFlux::Complet => "C",
        TypeFlux::Differentiel => "D",
    };
    sqlx::query(
        "INSERT INTO annuaire_sync_metadata (horodate_production, type_flux, unites_legales, etablissements, codes_routage, plateformes, lignes_annuaire)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&h.horodate_production)
    .bind(type_flux_str)
    .bind(total_stats.unites_legales as i64)
    .bind(total_stats.etablissements as i64)
    .bind(total_stats.codes_routage as i64)
    .bind(total_stats.plateformes as i64)
    .bind(total_stats.lignes_annuaire as i64)
    .execute(&mut *tx)
    .await?;

    // Commit
    info!("Commit de la transaction...");
    tx.commit().await?;

    total_stats.errors = stats.errors;
    info!("Import F14 terminé : {:?}", total_stats);
    Ok(total_stats)
}

/// Collecteur de batch pendant le parsing
struct BatchCollector {
    header: Option<F14Header>,
    ul: Vec<UniteLegale>,
    etab: Vec<Etablissement>,
    cr: Vec<CodeRoutage>,
    pf: Vec<Plateforme>,
    la: Vec<LigneAnnuaire>,
}

impl BatchCollector {
    fn new() -> Self {
        Self {
            header: None,
            ul: Vec::new(),
            etab: Vec::new(),
            cr: Vec::new(),
            pf: Vec::new(),
            la: Vec::new(),
        }
    }
}

// --- Fonctions d'insert sur transaction ---

use sqlx::PgConnection;

async fn insert_unites_legales_tx(tx: &mut PgConnection, batch: &[UniteLegale]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO unites_legales (id_instance, siren, nom, type_entite, statut, diffusible) ",
    );
    qb.push_values(batch, |mut b, ul| {
        b.push_bind(ul.id_instance)
            .push_bind(&ul.siren)
            .push_bind(&ul.nom)
            .push_bind(ul.type_entite.as_code())
            .push_bind(ul.statut.as_code())
            .push_bind(matches!(ul.diffusible, Diffusible::Oui));
    });
    qb.push(" ON CONFLICT (siren) DO UPDATE SET nom = EXCLUDED.nom, type_entite = EXCLUDED.type_entite, statut = EXCLUDED.statut, diffusible = EXCLUDED.diffusible, id_instance = EXCLUDED.id_instance, updated_at = NOW()");
    qb.build().execute(&mut *tx).await?;
    Ok(())
}

async fn insert_etablissements_tx(tx: &mut PgConnection, batch: &[Etablissement]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO etablissements (id_instance, siret, siren, type_etablissement, nom, adresse_1, adresse_2, adresse_3, localite, code_postal, code_pays, engagement_juridique, service, moa, diffusible) ",
    );
    qb.push_values(batch, |mut b, etab| {
        let (ej, svc, moa) = etab.donnees_b2g.as_ref()
            .map(|d| (d.engagement_juridique, d.service, d.moa))
            .unwrap_or((false, false, false));
        b.push_bind(etab.id_instance)
            .push_bind(&etab.siret)
            .push_bind(etab.siren())
            .push_bind(etab.type_etablissement.as_code())
            .push_bind(&etab.nom)
            .push_bind(&etab.adresse_1)
            .push_bind(&etab.adresse_2)
            .push_bind(&etab.adresse_3)
            .push_bind(&etab.localite)
            .push_bind(&etab.code_postal)
            .push_bind(etab.code_pays.as_deref().unwrap_or("FR"))
            .push_bind(ej)
            .push_bind(svc)
            .push_bind(moa)
            .push_bind(matches!(etab.diffusible, Diffusible::Oui));
    });
    qb.push(" ON CONFLICT (siret) DO UPDATE SET nom = EXCLUDED.nom, type_etablissement = EXCLUDED.type_etablissement, adresse_1 = EXCLUDED.adresse_1, adresse_2 = EXCLUDED.adresse_2, localite = EXCLUDED.localite, code_postal = EXCLUDED.code_postal, id_instance = EXCLUDED.id_instance, updated_at = NOW()");
    qb.build().execute(&mut *tx).await?;
    Ok(())
}

async fn insert_codes_routage_tx(tx: &mut PgConnection, batch: &[CodeRoutage]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO codes_routage (id_instance, siret, id_routage, nom, statut, adresse_1, localite, code_postal, code_pays) ",
    );
    qb.push_values(batch, |mut b, cr| {
        b.push_bind(cr.id_instance)
            .push_bind(&cr.siret)
            .push_bind(&cr.id_routage)
            .push_bind(&cr.nom)
            .push_bind(cr.statut.as_code())
            .push_bind(&cr.adresse_1)
            .push_bind(&cr.localite)
            .push_bind(&cr.code_postal)
            .push_bind(cr.code_pays.as_deref().unwrap_or("FR"));
    });
    qb.push(" ON CONFLICT (siret, id_routage) DO UPDATE SET nom = EXCLUDED.nom, statut = EXCLUDED.statut, adresse_1 = EXCLUDED.adresse_1, localite = EXCLUDED.localite, code_postal = EXCLUDED.code_postal, id_instance = EXCLUDED.id_instance, updated_at = NOW()");
    qb.build().execute(&mut *tx).await?;
    Ok(())
}

async fn insert_plateformes_tx(tx: &mut PgConnection, batch: &[Plateforme]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO plateformes (id_instance, matricule, siren, nom, nom_commercial, type_plateforme, date_debut_immat, date_fin_immat, contact) ",
    );
    qb.push_values(batch, |mut b, pf| {
        b.push_bind(pf.id_instance)
            .push_bind(&pf.matricule)
            .push_bind(&pf.siren)
            .push_bind(&pf.nom)
            .push_bind(&pf.nom_commercial)
            .push_bind(match pf.type_plateforme {
                TypePlateforme::Pdp => "PDP",
                TypePlateforme::Ppf => "PPF",
                TypePlateforme::AccessPoint => "AP",
                TypePlateforme::NonApplicable => "NA",
            })
            .push_bind(&pf.date_debut_immatriculation)
            .push_bind(&pf.date_fin_immatriculation)
            .push_bind(&pf.contact);
    });
    qb.push(" ON CONFLICT (matricule) DO UPDATE SET nom = EXCLUDED.nom, nom_commercial = EXCLUDED.nom_commercial, type_plateforme = EXCLUDED.type_plateforme, siren = EXCLUDED.siren, date_fin_immat = EXCLUDED.date_fin_immat, contact = EXCLUDED.contact, id_instance = EXCLUDED.id_instance, updated_at = NOW()");
    qb.build().execute(&mut *tx).await?;
    Ok(())
}

async fn insert_lignes_annuaire_tx(tx: &mut PgConnection, batch: &[LigneAnnuaire]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO lignes_annuaire (id_instance, siren, siret, id_routage, suffixe, matricule, nature, date_debut, date_fin, date_fin_effective) ",
    );
    qb.push_values(batch, |mut b, la| {
        b.push_bind(la.id_instance)
            .push_bind(&la.siren)
            .push_bind(&la.siret)
            .push_bind(&la.id_routage)
            .push_bind(&la.suffixe)
            .push_bind(&la.id_plateforme)
            .push_bind(la.nature.as_code())
            .push_bind(&la.date_debut)
            .push_bind(&la.date_fin)
            .push_bind(&la.date_fin_effective);
    });
    qb.push(" ON CONFLICT (id_instance) DO UPDATE SET siren = EXCLUDED.siren, siret = EXCLUDED.siret, id_routage = EXCLUDED.id_routage, suffixe = EXCLUDED.suffixe, matricule = EXCLUDED.matricule, nature = EXCLUDED.nature, date_debut = EXCLUDED.date_debut, date_fin = EXCLUDED.date_fin, date_fin_effective = EXCLUDED.date_fin_effective, updated_at = NOW()");
    qb.build().execute(&mut *tx).await?;
    Ok(())
}
