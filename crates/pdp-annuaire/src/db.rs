//! Stockage PostgreSQL de l'annuaire PPF.
//!
//! Gère la création du schéma, l'ingestion par batch (flux complet et
//! différentiel), et la résolution de routage locale.

use sqlx::postgres::PgPool;
use tracing::info;

use crate::model::*;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Erreur SQL : {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// Store PostgreSQL pour l'annuaire PPF
#[derive(Clone)]
pub struct AnnuaireStore {
    pool: PgPool,
}

impl AnnuaireStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Crée les tables si elles n'existent pas
    pub async fn migrate(&self) -> Result<(), DbError> {
        sqlx::query(SCHEMA_SQL).execute(&self.pool).await?;
        info!("Schéma annuaire initialisé");
        Ok(())
    }

    // --- Ingestion flux complet ---

    /// Vide toutes les tables avant un import complet
    pub async fn truncate_all(&self) -> Result<(), DbError> {
        sqlx::query(
            "TRUNCATE lignes_annuaire, codes_routage, etablissements, plateformes, unites_legales",
        )
        .execute(&self.pool)
        .await?;
        info!("Tables annuaire vidées pour import complet");
        Ok(())
    }

    // --- Insert batch ---

    pub async fn insert_unites_legales(&self, batch: &[UniteLegale]) -> Result<(), DbError> {
        if batch.is_empty() {
            return Ok(());
        }
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
        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_etablissements(&self, batch: &[Etablissement]) -> Result<(), DbError> {
        if batch.is_empty() {
            return Ok(());
        }
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "INSERT INTO etablissements (id_instance, siret, siren, type_etablissement, nom, adresse_1, adresse_2, adresse_3, localite, code_postal, code_pays, engagement_juridique, service, moa, diffusible) ",
        );
        qb.push_values(batch, |mut b, etab| {
            let (ej, svc, moa) = etab
                .donnees_b2g
                .as_ref()
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
        qb.push(" ON CONFLICT (siret) DO UPDATE SET nom = EXCLUDED.nom, type_etablissement = EXCLUDED.type_etablissement, adresse_1 = EXCLUDED.adresse_1, adresse_2 = EXCLUDED.adresse_2, localite = EXCLUDED.localite, code_postal = EXCLUDED.code_postal, statut = EXCLUDED.statut, id_instance = EXCLUDED.id_instance, updated_at = NOW()");
        // Note: statut n'est pas dans l'INSERT ci-dessus, on l'ajoute
        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_codes_routage(&self, batch: &[CodeRoutage]) -> Result<(), DbError> {
        if batch.is_empty() {
            return Ok(());
        }
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
        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_plateformes(&self, batch: &[Plateforme]) -> Result<(), DbError> {
        if batch.is_empty() {
            return Ok(());
        }
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
        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_lignes_annuaire(&self, batch: &[LigneAnnuaire]) -> Result<(), DbError> {
        if batch.is_empty() {
            return Ok(());
        }
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
        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    // --- Suppression (flux différentiel) ---

    pub async fn delete_unite_legale(&self, siren: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM unites_legales WHERE siren = $1")
            .bind(siren)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_etablissement(&self, siret: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM etablissements WHERE siret = $1")
            .bind(siret)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_ligne_annuaire(&self, id_instance: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM lignes_annuaire WHERE id_instance = $1")
            .bind(id_instance)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Résolution de routage ---

    /// Résout la plateforme de réception pour un destinataire.
    /// Cherche la ligne d'annuaire la plus spécifique en vigueur.
    ///
    /// Ordre de priorité (plus spécifique en premier) :
    /// 1. Suffixe
    /// 2. Code routage (SIREN + SIRET + id_routage)
    /// 3. SIRET (SIREN + SIRET)
    /// 4. SIREN seul
    pub async fn resolve_routing(
        &self,
        buyer_siren: &str,
        buyer_siret: Option<&str>,
        code_routage: Option<&str>,
        suffixe: Option<&str>,
        date: &str,
    ) -> Result<Option<RoutingResult>, DbError> {
        // 1. Suffixe
        if let Some(suf) = suffixe {
            if let Some(r) = self.lookup_by_suffixe(buyer_siren, suf, date).await? {
                return Ok(Some(r));
            }
        }

        // 2. Code routage
        if let (Some(siret), Some(cr)) = (buyer_siret, code_routage) {
            if let Some(r) = self
                .lookup_by_code_routage(buyer_siren, siret, cr, date)
                .await?
            {
                return Ok(Some(r));
            }
        }

        // 3. SIRET
        if let Some(siret) = buyer_siret {
            if let Some(r) = self.lookup_by_siret(buyer_siren, siret, date).await? {
                return Ok(Some(r));
            }
        }

        // 4. SIREN
        self.lookup_by_siren(buyer_siren, date).await
    }

    async fn lookup_by_suffixe(
        &self,
        siren: &str,
        suffixe: &str,
        date: &str,
    ) -> Result<Option<RoutingResult>, DbError> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT la.matricule, p.nom, p.type_plateforme
             FROM lignes_annuaire la
             LEFT JOIN plateformes p ON p.matricule = la.matricule
             WHERE la.siren = $1 AND la.suffixe = $2
               AND la.nature = 'D'
               AND la.date_debut <= $3
               AND (la.date_fin_effective IS NULL OR la.date_fin_effective >= $3)
             ORDER BY la.date_debut DESC
             LIMIT 1",
        )
        .bind(siren)
        .bind(suffixe)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(matricule, nom, tp)| RoutingResult {
            matricule_plateforme: matricule,
            nom_plateforme: nom,
            type_plateforme: tp.and_then(|s| TypePlateforme::from_code(&s)),
            maille: RoutingMaille::Suffixe,
        }))
    }

    async fn lookup_by_code_routage(
        &self,
        siren: &str,
        siret: &str,
        code_routage: &str,
        date: &str,
    ) -> Result<Option<RoutingResult>, DbError> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT la.matricule, p.nom, p.type_plateforme
             FROM lignes_annuaire la
             LEFT JOIN plateformes p ON p.matricule = la.matricule
             WHERE la.siren = $1 AND la.siret = $2 AND la.id_routage = $3
               AND la.nature = 'D'
               AND la.date_debut <= $4
               AND (la.date_fin_effective IS NULL OR la.date_fin_effective >= $4)
             ORDER BY la.date_debut DESC
             LIMIT 1",
        )
        .bind(siren)
        .bind(siret)
        .bind(code_routage)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(matricule, nom, tp)| RoutingResult {
            matricule_plateforme: matricule,
            nom_plateforme: nom,
            type_plateforme: tp.and_then(|s| TypePlateforme::from_code(&s)),
            maille: RoutingMaille::CodeRoutage,
        }))
    }

    async fn lookup_by_siret(
        &self,
        siren: &str,
        siret: &str,
        date: &str,
    ) -> Result<Option<RoutingResult>, DbError> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT la.matricule, p.nom, p.type_plateforme
             FROM lignes_annuaire la
             LEFT JOIN plateformes p ON p.matricule = la.matricule
             WHERE la.siren = $1 AND la.siret = $2
               AND la.id_routage IS NULL AND la.suffixe IS NULL
               AND la.nature = 'D'
               AND la.date_debut <= $3
               AND (la.date_fin_effective IS NULL OR la.date_fin_effective >= $3)
             ORDER BY la.date_debut DESC
             LIMIT 1",
        )
        .bind(siren)
        .bind(siret)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(matricule, nom, tp)| RoutingResult {
            matricule_plateforme: matricule,
            nom_plateforme: nom,
            type_plateforme: tp.and_then(|s| TypePlateforme::from_code(&s)),
            maille: RoutingMaille::Siret,
        }))
    }

    async fn lookup_by_siren(
        &self,
        siren: &str,
        date: &str,
    ) -> Result<Option<RoutingResult>, DbError> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT la.matricule, p.nom, p.type_plateforme
             FROM lignes_annuaire la
             LEFT JOIN plateformes p ON p.matricule = la.matricule
             WHERE la.siren = $1
               AND la.siret IS NULL AND la.id_routage IS NULL AND la.suffixe IS NULL
               AND la.nature = 'D'
               AND la.date_debut <= $2
               AND (la.date_fin_effective IS NULL OR la.date_fin_effective >= $2)
             ORDER BY la.date_debut DESC
             LIMIT 1",
        )
        .bind(siren)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(matricule, nom, tp)| RoutingResult {
            matricule_plateforme: matricule,
            nom_plateforme: nom,
            type_plateforme: tp.and_then(|s| TypePlateforme::from_code(&s)),
            maille: RoutingMaille::Siren,
        }))
    }

    // --- Stats ---

    pub async fn count_all(&self) -> Result<ImportStats, DbError> {
        let ul: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM unites_legales")
            .fetch_one(&self.pool)
            .await?;
        let et: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM etablissements")
            .fetch_one(&self.pool)
            .await?;
        let cr: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM codes_routage")
            .fetch_one(&self.pool)
            .await?;
        let pf: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM plateformes")
            .fetch_one(&self.pool)
            .await?;
        let la: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lignes_annuaire")
            .fetch_one(&self.pool)
            .await?;

        Ok(ImportStats {
            unites_legales: ul.0 as usize,
            etablissements: et.0 as usize,
            codes_routage: cr.0 as usize,
            plateformes: pf.0 as usize,
            lignes_annuaire: la.0 as usize,
            errors: 0,
        })
    }

    /// Sauvegarde l'horodatage de la dernière synchro
    pub async fn save_sync_metadata(
        &self,
        horodate: &str,
        type_flux: &str,
        stats: &ImportStats,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO annuaire_sync_metadata (horodate_production, type_flux, unites_legales, etablissements, codes_routage, plateformes, lignes_annuaire)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(horodate)
        .bind(type_flux)
        .bind(stats.unites_legales as i64)
        .bind(stats.etablissements as i64)
        .bind(stats.codes_routage as i64)
        .bind(stats.plateformes as i64)
        .bind(stats.lignes_annuaire as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Récupère le dernier horodatage de synchro
    pub async fn last_sync_horodate(&self) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT horodate_production FROM annuaire_sync_metadata ORDER BY synced_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(h,)| h))
    }
}

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS unites_legales (
    id_instance     BIGINT NOT NULL,
    siren           CHAR(9) NOT NULL,
    nom             TEXT NOT NULL,
    type_entite     CHAR(1) NOT NULL DEFAULT 'A',
    statut          CHAR(1) NOT NULL DEFAULT 'A',
    diffusible      BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (siren)
);

CREATE TABLE IF NOT EXISTS etablissements (
    id_instance         BIGINT NOT NULL,
    siret               CHAR(14) NOT NULL,
    siren               CHAR(9) NOT NULL,
    type_etablissement  CHAR(1) DEFAULT 'S',
    nom                 TEXT NOT NULL,
    adresse_1           TEXT,
    adresse_2           TEXT,
    adresse_3           TEXT,
    localite            TEXT,
    code_postal         TEXT,
    code_pays           CHAR(2) DEFAULT 'FR',
    engagement_juridique BOOLEAN DEFAULT FALSE,
    service             BOOLEAN DEFAULT FALSE,
    moa                 BOOLEAN DEFAULT FALSE,
    diffusible          BOOLEAN NOT NULL DEFAULT TRUE,
    statut              CHAR(1) NOT NULL DEFAULT 'A',
    updated_at          TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (siret)
);

CREATE TABLE IF NOT EXISTS codes_routage (
    id_instance     BIGINT NOT NULL,
    siret           CHAR(14) NOT NULL,
    id_routage      TEXT NOT NULL,
    nom             TEXT NOT NULL,
    statut          CHAR(1) NOT NULL DEFAULT 'A',
    adresse_1       TEXT,
    localite        TEXT,
    code_postal     TEXT,
    code_pays       CHAR(2) DEFAULT 'FR',
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (siret, id_routage)
);

CREATE TABLE IF NOT EXISTS plateformes (
    id_instance             BIGINT NOT NULL,
    matricule               CHAR(4) NOT NULL,
    siren                   CHAR(9),
    nom                     TEXT NOT NULL,
    nom_commercial          TEXT,
    type_plateforme         TEXT NOT NULL DEFAULT 'PDP',
    date_debut_immat        TEXT NOT NULL,
    date_fin_immat          TEXT,
    contact                 TEXT,
    updated_at              TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (matricule)
);

CREATE TABLE IF NOT EXISTS lignes_annuaire (
    id_instance     BIGINT NOT NULL,
    siren           CHAR(9) NOT NULL,
    siret           CHAR(14),
    id_routage      TEXT,
    suffixe         TEXT,
    matricule       CHAR(4) NOT NULL,
    nature          CHAR(1) NOT NULL DEFAULT 'D',
    date_debut      TEXT NOT NULL,
    date_fin        TEXT,
    date_fin_effective TEXT,
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id_instance)
);

CREATE INDEX IF NOT EXISTS idx_lignes_siren ON lignes_annuaire(siren);
CREATE INDEX IF NOT EXISTS idx_lignes_siret ON lignes_annuaire(siret);
CREATE INDEX IF NOT EXISTS idx_lignes_matricule ON lignes_annuaire(matricule);
CREATE INDEX IF NOT EXISTS idx_lignes_nature_dates ON lignes_annuaire(nature, date_debut);
CREATE INDEX IF NOT EXISTS idx_etab_siren ON etablissements(siren);

CREATE TABLE IF NOT EXISTS annuaire_sync_metadata (
    id              BIGSERIAL PRIMARY KEY,
    horodate_production TEXT NOT NULL,
    type_flux       TEXT NOT NULL,
    unites_legales  BIGINT DEFAULT 0,
    etablissements  BIGINT DEFAULT 0,
    codes_routage   BIGINT DEFAULT 0,
    plateformes     BIGINT DEFAULT 0,
    lignes_annuaire BIGINT DEFAULT 0,
    synced_at       TIMESTAMPTZ DEFAULT NOW()
);
"#;
