//! Trait `TraceBackend` : surface de lecture utilisée par l'UI et l'API HTTP.
//!
//! Permet aux handlers de dépendre d'une abstraction plutôt que du
//! [`TraceStore`] concret (qui parle Elasticsearch). Les tests d'intégration
//! UI fournissent une implémentation 100 % en mémoire ([`InMemoryTraceBackend`]).
//!
//! Le pipeline d'ingestion (`TraceProcessor`, `DuplicateCheckProcessor`)
//! continue de dépendre de `TraceStore` concret car il a besoin des écritures
//! et des recherches avancées (full-text XML, agrégations) — non incluses ici.

use async_trait::async_trait;

use pdp_core::error::PdpResult;

use crate::store::{DailyBreakdown, ExchangeDocument, ExchangeSummary, TraceStats};

/// Méthodes de lecture du store de traçabilité utilisées par les handlers
/// HTTP et UI. Tout est `async` (le store ES répond via HTTP).
#[async_trait]
pub trait TraceBackend: Send + Sync {
    /// Statistiques globales (tous tenants confondus).
    async fn get_stats(&self) -> PdpResult<TraceStats>;

    /// Statistiques pour un tenant donné (par SIREN).
    async fn get_stats_for_siren(&self, siren: &str) -> PdpResult<TraceStats>;

    /// Raison sociale lue depuis le premier document du tenant — pour
    /// afficher "TechConseil SAS — SIREN 123456789" dans l'UI.
    async fn get_tenant_name(&self, siren: &str) -> Option<String>;

    /// Liste paginée des exchanges d'un tenant avec filtres optionnels.
    ///
    /// `direction` :
    /// - `Some("emises")` filtre les factures dont le tenant est vendeur,
    /// - `Some("recues")` celles dont il est acheteur,
    /// - `None` retourne les deux (legacy / écran "tout").
    async fn list_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
        direction: Option<&str>,
    ) -> PdpResult<Vec<ExchangeSummary>>;

    /// Compte le nombre total d'exchanges pour un tenant (avec les mêmes
    /// filtres que `list_exchanges`, mais sans pagination ni limite). Utilisé
    /// par l'UI pour afficher le nombre total de factures et le nombre de pages.
    async fn count_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
    ) -> PdpResult<i64>;

    /// Variante de [`list_exchanges`] avec déduplication par `invoice_number`.
    /// Quand `dedup_by_invoice=true`, retourne un seul exchange par numéro
    /// de facture (le plus récent). Implémentation par défaut : délègue à
    /// `list_exchanges` (sans dedup).
    async fn list_exchanges_with_dedup(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
        direction: Option<&str>,
        _dedup_by_invoice: bool,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        self.list_exchanges(siren, status, from_date, to_date, page, page_size, direction)
            .await
    }

    /// Variante de [`count_exchanges`] avec comptage par factures uniques.
    /// Quand `dedup_by_invoice=true`, retourne la cardinalité distincte
    /// d'`invoice_number` au lieu du nombre brut d'exchanges. Implémentation
    /// par défaut : délègue à `count_exchanges` (pas de dedup).
    async fn count_exchanges_with_dedup(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
        _dedup_by_invoice: bool,
    ) -> PdpResult<i64> {
        self.count_exchanges(siren, status, from_date, to_date, direction)
            .await
    }

    /// Document complet (avec XML/PDF/événements) par ID. Si `siren` est `None`,
    /// le document est cherché à travers tous les tenants.
    async fn get_exchange(
        &self,
        exchange_id: &str,
        siren: Option<&str>,
    ) -> PdpResult<Option<ExchangeDocument>>;

    /// Variante par `flow_id` qui privilégie l'index `pdp-{siren}` — utile
    /// pour les flows intra-PDP où le même flow_id existe dans 2 docs
    /// (seller-side et buyer-side). Sans ce ciblage, l'UI tombe sur le
    /// mauvais doc et affiche les CDVs de l'autre côté.
    ///
    /// Impl par défaut : passe par `list_exchanges` puis `get_exchange` —
    /// suffisant pour `InMemoryTraceBackend` (un seul doc par flow_id dans
    /// les tests), surchargé par `TraceStore` pour le ciblage d'index ES.
    async fn get_exchange_by_flow_id(
        &self,
        flow_id: &str,
        siren: &str,
    ) -> PdpResult<Option<ExchangeDocument>> {
        let summaries = self
            .list_exchanges(siren, None, None, None, 0, 200, None)
            .await?;
        let summary = match summaries
            .iter()
            .find(|s| s.flow_id == flow_id || s.exchange_id == flow_id)
        {
            Some(s) => s,
            None => return Ok(None),
        };
        self.get_exchange(&summary.exchange_id, Some(siren)).await
    }

    /// Tous les flux en erreur, tous tenants confondus (utilisé par l'API
    /// `/v1/flows` quand on demande `?status=error`).
    async fn get_error_flows(&self) -> PdpResult<Vec<ExchangeSummary>>;

    /// Compteur quotidien des flux d'un tenant sur les `days` derniers jours
    /// (seller OR buyer). Utilisé par les sparklines du dashboard.
    /// Retour aligné à droite : index `days - 1` = aujourd'hui.
    /// Implémentation par défaut : renvoie un Vec de zéros (sparkline vide).
    /// Les backends qui supportent l'agrégation surchargent.
    async fn daily_counts_for_siren(&self, _siren: &str, days: u32) -> PdpResult<Vec<i64>> {
        Ok(vec![0; days as usize])
    }

    /// Breakdown quotidien : total, distribués, pending, erreurs sur `days` jours.
    /// Implémentation par défaut : zéros (les backends ES surchargent).
    async fn daily_breakdown_for_siren(&self, _siren: &str, days: u32) -> PdpResult<DailyBreakdown> {
        Ok(DailyBreakdown::zeros(days as usize))
    }
}

#[async_trait]
impl TraceBackend for crate::store::TraceStore {
    async fn get_stats(&self) -> PdpResult<TraceStats> {
        crate::store::TraceStore::get_stats(self).await
    }

    async fn get_stats_for_siren(&self, siren: &str) -> PdpResult<TraceStats> {
        crate::store::TraceStore::get_stats_for_siren(self, siren).await
    }

    async fn get_tenant_name(&self, siren: &str) -> Option<String> {
        crate::store::TraceStore::get_tenant_name(self, siren).await
    }

    async fn list_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
        direction: Option<&str>,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        crate::store::TraceStore::list_exchanges(
            self, siren, status, from_date, to_date, page, page_size, direction,
        )
        .await
    }

    async fn count_exchanges(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
    ) -> PdpResult<i64> {
        crate::store::TraceStore::count_exchanges(
            self, siren, status, from_date, to_date, direction,
        )
        .await
    }

    async fn list_exchanges_with_dedup(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        page: usize,
        page_size: usize,
        direction: Option<&str>,
        dedup_by_invoice: bool,
    ) -> PdpResult<Vec<ExchangeSummary>> {
        crate::store::TraceStore::list_exchanges_with_dedup(
            self, siren, status, from_date, to_date, page, page_size, direction, dedup_by_invoice,
        )
        .await
    }

    async fn count_exchanges_with_dedup(
        &self,
        siren: &str,
        status: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        direction: Option<&str>,
        dedup_by_invoice: bool,
    ) -> PdpResult<i64> {
        crate::store::TraceStore::count_exchanges_with_dedup(
            self, siren, status, from_date, to_date, direction, dedup_by_invoice,
        )
        .await
    }

    async fn get_exchange(
        &self,
        exchange_id: &str,
        siren: Option<&str>,
    ) -> PdpResult<Option<ExchangeDocument>> {
        crate::store::TraceStore::get_exchange(self, exchange_id, siren).await
    }

    async fn get_exchange_by_flow_id(
        &self,
        flow_id: &str,
        siren: &str,
    ) -> PdpResult<Option<ExchangeDocument>> {
        crate::store::TraceStore::get_exchange_by_flow_id(self, flow_id, siren).await
    }

    async fn get_error_flows(&self) -> PdpResult<Vec<ExchangeSummary>> {
        crate::store::TraceStore::get_error_flows(self).await
    }

    async fn daily_counts_for_siren(&self, siren: &str, days: u32) -> PdpResult<Vec<i64>> {
        crate::store::TraceStore::daily_counts_for_siren(self, siren, days).await
    }

    async fn daily_breakdown_for_siren(&self, siren: &str, days: u32) -> PdpResult<DailyBreakdown> {
        crate::store::TraceStore::daily_breakdown_for_siren(self, siren, days).await
    }
}
