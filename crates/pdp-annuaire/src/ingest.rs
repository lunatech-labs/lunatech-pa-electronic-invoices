//! Ingestion de l'annuaire PPF : orchestre le parsing streaming F14
//! et l'écriture en batch dans PostgreSQL dans une transaction unique.
//!
//! Architecture streaming : le parser tourne dans un thread bloquant
//! (spawn_blocking) et envoie les éléments via un channel mpsc.
//! Le récepteur async accumule par batch de BATCH_SIZE et flush
//! directement dans la transaction — mémoire bornée à ~5×BATCH_SIZE.

use std::io::BufRead;
use tracing::info;

use crate::db::AnnuaireStore;
use crate::model::*;
use crate::parser::{self, F14Event, ParseError};

/// 65535 (u16::MAX) paramètres bind max en PostgreSQL.
/// Établissements = 15 colonnes → 65535/15 = 4369, on prend 4000 par sécurité.
const BATCH_SIZE: usize = 4000;

/// Index à supprimer avant un import complet (accélère les INSERT)
const DROP_INDEXES_SQL: &[&str] = &[
    "DROP INDEX IF EXISTS idx_lignes_siren",
    "DROP INDEX IF EXISTS idx_lignes_siret",
    "DROP INDEX IF EXISTS idx_lignes_matricule",
    "DROP INDEX IF EXISTS idx_lignes_nature_dates",
    "DROP INDEX IF EXISTS idx_etab_siren",
    "DROP INDEX IF EXISTS idx_ul_nom_trgm",
    "DROP INDEX IF EXISTS idx_etab_nom_trgm",
    "DROP INDEX IF EXISTS idx_etab_adresse_trgm",
    "DROP INDEX IF EXISTS idx_etab_localite_trgm",
    "DROP INDEX IF EXISTS idx_cr_siret",
];

/// Index à recréer après un import complet
const CREATE_INDEXES_SQL: &[&str] = &[
    "CREATE INDEX idx_lignes_siren ON lignes_annuaire(siren)",
    "CREATE INDEX idx_lignes_siret ON lignes_annuaire(siret)",
    "CREATE INDEX idx_lignes_matricule ON lignes_annuaire(matricule)",
    "CREATE INDEX idx_lignes_nature_dates ON lignes_annuaire(nature, date_debut)",
    "CREATE INDEX idx_etab_siren ON etablissements(siren)",
    "CREATE INDEX idx_cr_siret ON codes_routage(siret)",
    "CREATE INDEX idx_ul_nom_trgm ON unites_legales USING gin (UPPER(nom) gin_trgm_ops)",
    "CREATE INDEX idx_etab_nom_trgm ON etablissements USING gin (UPPER(nom) gin_trgm_ops)",
    "CREATE INDEX idx_etab_adresse_trgm ON etablissements USING gin (UPPER(adresse_1) gin_trgm_ops)",
    "CREATE INDEX idx_etab_localite_trgm ON etablissements USING gin (UPPER(localite) gin_trgm_ops)",
];

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
    #[error("Channel fermé : le parser s'est arrêté prématurément")]
    ChannelClosed,
}

use sqlx::PgConnection;

/// Ingère un flux F14 complet ou différentiel dans la base.
///
/// Le parsing tourne dans un thread dédié (spawn_blocking) et envoie
/// les éléments via un channel. Le récepteur async accumule par batch
/// de 5000 et flush dans une transaction PostgreSQL — mémoire bornée.
pub async fn ingest_f14<R: BufRead + Send + 'static>(
    reader: R,
    store: &AnnuaireStore,
    expect_horodate: Option<&str>,
) -> Result<ImportStats, IngestError> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<F14Event>(BATCH_SIZE);

    // Parser dans un thread bloquant
    let parser_handle = tokio::task::spawn_blocking(move || {
        parser::parse_f14(reader, |event| {
            // Bloque si le channel est plein (backpressure)
            tx.blocking_send(event).map_err(|_| {
                ParseError::MissingField("Channel fermé".into())
            })
        })
    });

    // Récepteur async : accumule et flush par batch
    let mut tx_db = store.begin().await?;
    let mut header: Option<F14Header> = None;
    let mut truncated = false;

    let mut buf_ul: Vec<UniteLegale> = Vec::with_capacity(BATCH_SIZE);
    let mut buf_etab: Vec<Etablissement> = Vec::with_capacity(BATCH_SIZE);
    let mut buf_cr: Vec<CodeRoutage> = Vec::with_capacity(BATCH_SIZE);
    let mut buf_pf: Vec<Plateforme> = Vec::with_capacity(BATCH_SIZE);
    let mut buf_la: Vec<LigneAnnuaire> = Vec::with_capacity(BATCH_SIZE);

    let mut total_stats = ImportStats::default();
    let expect_horodate = expect_horodate.map(String::from);

    while let Some(event) = rx.recv().await {
        match event {
            F14Event::Header(h) => {
                info!(
                    "Header F14 : type={:?}, horodate={}",
                    h.type_flux, h.horodate_production
                );

                // Vérification horodatage pour les flux différentiels
                if let (Some(ref expected), Some(ref dernier)) =
                    (&expect_horodate, &h.dernier_horodate_production)
                {
                    if expected != dernier {
                        return Err(IngestError::HorodateMismatch {
                            expected: expected.clone(),
                            received: dernier.clone(),
                        });
                    }
                }

                // Flux complet : dropper les index et vider les tables
                if h.type_flux == TypeFlux::Complet && !truncated {
                    info!("Flux complet : suppression des index pour accélérer l'import...");
                    for idx in DROP_INDEXES_SQL {
                        sqlx::query(idx).execute(&mut *tx_db).await?;
                    }
                    info!("Flux complet : vidage des tables...");
                    sqlx::query("TRUNCATE lignes_annuaire, codes_routage, etablissements, plateformes, unites_legales")
                        .execute(&mut *tx_db)
                        .await?;
                    truncated = true;
                }

                header = Some(h);
            }

            event => {
                // Garde-fou : le Header doit arriver avant tout élément
                if header.is_none() {
                    return Err(IngestError::Parse(ParseError::MissingField(
                        "Header F14 non reçu avant les données — parsing incohérent".into(),
                    )));
                }

                match event {
                    F14Event::Header(_) => unreachable!(),
                    F14Event::UniteLegale(ul) => {
                        buf_ul.push(ul);
                        if buf_ul.len() >= BATCH_SIZE {
                            insert_unites_legales_tx(&mut tx_db, &buf_ul).await?;
                            total_stats.unites_legales += buf_ul.len();
                            buf_ul.clear();
                        }
                    }
                    F14Event::Etablissement(etab) => {
                        buf_etab.push(etab);
                        if buf_etab.len() >= BATCH_SIZE {
                            insert_etablissements_tx(&mut tx_db, &buf_etab).await?;
                            total_stats.etablissements += buf_etab.len();
                            buf_etab.clear();
                        }
                    }
                    F14Event::CodeRoutage(cr) => {
                        buf_cr.push(cr);
                        if buf_cr.len() >= BATCH_SIZE {
                            insert_codes_routage_tx(&mut tx_db, &buf_cr).await?;
                            total_stats.codes_routage += buf_cr.len();
                            buf_cr.clear();
                        }
                    }
                    F14Event::Plateforme(pf) => {
                        buf_pf.push(pf);
                        if buf_pf.len() >= BATCH_SIZE {
                            insert_plateformes_tx(&mut tx_db, &buf_pf).await?;
                            total_stats.plateformes += buf_pf.len();
                            buf_pf.clear();
                        }
                    }
                    F14Event::LigneAnnuaire(la) => {
                        buf_la.push(la);
                        if buf_la.len() >= BATCH_SIZE {
                            insert_lignes_annuaire_tx(&mut tx_db, &buf_la).await?;
                            total_stats.lignes_annuaire += buf_la.len();
                            buf_la.clear();
                        }
                    }
                }
            }
        }
    }

    // Flush les restes
    if !buf_ul.is_empty() {
        insert_unites_legales_tx(&mut tx_db, &buf_ul).await?;
        total_stats.unites_legales += buf_ul.len();
    }
    if !buf_etab.is_empty() {
        insert_etablissements_tx(&mut tx_db, &buf_etab).await?;
        total_stats.etablissements += buf_etab.len();
    }
    if !buf_cr.is_empty() {
        insert_codes_routage_tx(&mut tx_db, &buf_cr).await?;
        total_stats.codes_routage += buf_cr.len();
    }
    if !buf_pf.is_empty() {
        insert_plateformes_tx(&mut tx_db, &buf_pf).await?;
        total_stats.plateformes += buf_pf.len();
    }
    if !buf_la.is_empty() {
        insert_lignes_annuaire_tx(&mut tx_db, &buf_la).await?;
        total_stats.lignes_annuaire += buf_la.len();
    }

    // Vérifier que le parser a terminé sans erreur
    let parse_stats = parser_handle
        .await
        .map_err(|_| IngestError::ChannelClosed)?
        .map_err(IngestError::Parse)?;
    total_stats.errors = parse_stats.errors;

    // Métadonnées de synchro
    if let Some(ref h) = header {
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
        .execute(&mut *tx_db)
        .await?;
    }

    // Recréer les index si on les a droppés
    if truncated {
        info!("Recréation des index...");
        for idx in CREATE_INDEXES_SQL {
            sqlx::query(idx).execute(&mut *tx_db).await?;
        }
        info!("Index recréés");
    }

    // Commit
    info!("Commit de la transaction...");
    tx_db.commit().await?;

    info!("Import F14 terminé : {:?}", total_stats);
    Ok(total_stats)
}

// --- Fonctions d'insert sur transaction ---

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
