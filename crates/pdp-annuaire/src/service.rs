//! Service annuaire PPF — couche métier au-dessus de AnnuaireStore.
//!
//! Fournit des méthodes de validation pour vérifier l'existence et le statut
//! des parties (vendeur/acheteur) dans l'annuaire local (PostgreSQL).
//!
//! Conforme à la règle AFNOR G1.63 : "Le SIREN du vendeur (BT-30) et de
//! l'acheteur (BT-47) sont obligatoires et doivent exister dans l'annuaire."

use crate::db::{AnnuaireStore, DbError};
use crate::model::{Statut, UniteLegale};

/// Résultat de la validation des parties (vendeur + acheteur)
#[derive(Debug, Clone)]
pub struct PartyValidationResult {
    pub seller_exists: bool,
    pub seller_active: bool,
    pub seller_name: Option<String>,
    pub buyer_exists: bool,
    pub buyer_active: bool,
    pub buyer_name: Option<String>,
}

/// Service de validation annuaire PPF.
///
/// Encapsule les requêtes de vérification d'existence et de statut
/// des entités dans l'annuaire local (PostgreSQL).
#[derive(Clone)]
pub struct AnnuaireService {
    store: AnnuaireStore,
}

impl AnnuaireService {
    pub fn new(store: AnnuaireStore) -> Self {
        Self { store }
    }

    /// Vérifie si un SIREN existe dans l'annuaire
    pub async fn exists_siren(&self, siren: &str) -> Result<bool, DbError> {
        self.store
            .lookup_unite_legale(siren)
            .await
            .map(|opt| opt.is_some())
    }

    /// Vérifie si un SIREN est actif dans l'annuaire.
    /// Retourne `Some(true)` si actif, `Some(false)` si inactif, `None` si introuvable.
    pub async fn is_siren_active(&self, siren: &str) -> Result<Option<bool>, DbError> {
        let ul = self.store.lookup_unite_legale(siren).await?;
        Ok(ul.map(|u| u.statut == Statut::Actif))
    }

    /// Vérifie si un SIRET existe dans l'annuaire
    pub async fn exists_siret(&self, siret: &str) -> Result<bool, DbError> {
        self.store
            .lookup_etablissement_by_siret(siret)
            .await
            .map(|opt| opt.is_some())
    }

    /// Retourne l'unité légale complète pour un SIREN
    pub async fn get_unite_legale(&self, siren: &str) -> Result<Option<UniteLegale>, DbError> {
        self.store.lookup_unite_legale(siren).await
    }

    /// Valide l'existence et le statut du vendeur et de l'acheteur (G1.63).
    ///
    /// - `seller_siren` : SIREN du vendeur (obligatoire)
    /// - `buyer_siren` : SIREN de l'acheteur (optionnel — pas vérifié en réception)
    pub async fn validate_parties(
        &self,
        seller_siren: &str,
        buyer_siren: Option<&str>,
    ) -> Result<PartyValidationResult, DbError> {
        // Lookup vendeur
        let seller_ul = self.store.lookup_unite_legale(seller_siren).await?;
        let (seller_exists, seller_active, seller_name) = match &seller_ul {
            Some(ul) => (true, ul.statut == Statut::Actif, Some(ul.nom.clone())),
            None => (false, false, None),
        };

        // Lookup acheteur (si demandé)
        let (buyer_exists, buyer_active, buyer_name) = if let Some(siren) = buyer_siren {
            let buyer_ul = self.store.lookup_unite_legale(siren).await?;
            match &buyer_ul {
                Some(ul) => (true, ul.statut == Statut::Actif, Some(ul.nom.clone())),
                None => (false, false, None),
            }
        } else {
            (true, true, None) // Pas de vérification demandée → OK
        };

        Ok(PartyValidationResult {
            seller_exists,
            seller_active,
            seller_name,
            buyer_exists,
            buyer_active,
            buyer_name,
        })
    }
}
