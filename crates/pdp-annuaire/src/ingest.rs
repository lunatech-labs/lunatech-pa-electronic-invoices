//! Ingestion de l'annuaire PPF : orchestre le parsing streaming F14
//! et l'écriture en batch dans PostgreSQL.

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
    #[error("Incohérence horodatage : attendu {expected}, reçu {received}")]
    HorodateMismatch { expected: String, received: String },
}

/// Ingère un flux F14 complet ou différentiel dans la base.
pub async fn ingest_f14<R: BufRead>(
    reader: R,
    store: &AnnuaireStore,
    expect_horodate: Option<&str>,
) -> Result<ImportStats, IngestError> {
    // Buffers de batch
    let mut ul_batch: Vec<UniteLegale> = Vec::with_capacity(BATCH_SIZE);
    let mut etab_batch: Vec<Etablissement> = Vec::with_capacity(BATCH_SIZE);
    let mut cr_batch: Vec<CodeRoutage> = Vec::with_capacity(BATCH_SIZE);
    let mut pf_batch: Vec<Plateforme> = Vec::with_capacity(BATCH_SIZE);
    let mut la_batch: Vec<LigneAnnuaire> = Vec::with_capacity(BATCH_SIZE);

    let mut header: Option<F14Header> = None;
    let mut total_stats = ImportStats::default();

    // Le parser est synchrone, on collecte les événements
    // puis on flush les batches de manière asynchrone
    let stats = parser::parse_f14(reader, |event| {
        match event {
            F14Event::Header(h) => {
                header = Some(h);
            }
            F14Event::UniteLegale(ul) => ul_batch.push(ul),
            F14Event::Etablissement(etab) => etab_batch.push(etab),
            F14Event::CodeRoutage(cr) => cr_batch.push(cr),
            F14Event::Plateforme(pf) => pf_batch.push(pf),
            F14Event::LigneAnnuaire(la) => la_batch.push(la),
        }
        Ok(())
    })?;

    // Vérifier l'en-tête
    let h = header.as_ref().ok_or_else(|| {
        ParseError::MissingField("Header F14 manquant".into())
    })?;

    info!(
        "F14 parsé : type={:?}, horodate={}, {} UL, {} étab, {} CR, {} PF, {} LA",
        h.type_flux,
        h.horodate_production,
        stats.unites_legales,
        stats.etablissements,
        stats.codes_routage,
        stats.plateformes,
        stats.lignes_annuaire,
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

    // Flux complet : vider les tables
    if h.type_flux == TypeFlux::Complet {
        info!("Flux complet : vidage des tables...");
        store.truncate_all().await?;
    }

    // Insérer par batch
    info!("Insertion des plateformes ({})...", pf_batch.len());
    for chunk in pf_batch.chunks(BATCH_SIZE) {
        store.insert_plateformes(chunk).await?;
    }
    total_stats.plateformes = pf_batch.len();

    info!("Insertion des unités légales ({})...", ul_batch.len());
    for chunk in ul_batch.chunks(BATCH_SIZE) {
        store.insert_unites_legales(chunk).await?;
    }
    total_stats.unites_legales = ul_batch.len();

    info!("Insertion des établissements ({})...", etab_batch.len());
    for chunk in etab_batch.chunks(BATCH_SIZE) {
        store.insert_etablissements(chunk).await?;
    }
    total_stats.etablissements = etab_batch.len();

    info!("Insertion des codes routage ({})...", cr_batch.len());
    for chunk in cr_batch.chunks(BATCH_SIZE) {
        store.insert_codes_routage(chunk).await?;
    }
    total_stats.codes_routage = cr_batch.len();

    info!("Insertion des lignes d'annuaire ({})...", la_batch.len());
    for chunk in la_batch.chunks(BATCH_SIZE) {
        store.insert_lignes_annuaire(chunk).await?;
    }
    total_stats.lignes_annuaire = la_batch.len();

    // Sauvegarder les métadonnées de synchro
    let type_flux_str = match h.type_flux {
        TypeFlux::Complet => "C",
        TypeFlux::Differentiel => "D",
    };
    store
        .save_sync_metadata(&h.horodate_production, type_flux_str, &total_stats)
        .await?;

    info!("Import F14 terminé : {:?}", total_stats);
    Ok(total_stats)
}
