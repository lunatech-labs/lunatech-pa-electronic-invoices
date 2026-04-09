use serde::{Deserialize, Serialize};

/// TB-0 : Rapport e-reporting (racine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EReport {
    /// TB-1 : En-tête du rapport
    pub document: ReportDocument,
    /// TB-2 : Transmission des transactions (flux 10.1 / 10.3)
    pub transactions: Option<TransactionsReport>,
    /// TB-3 : Transmission des paiements (flux 10.2 / 10.4)
    pub payments: Option<PaymentsReport>,
}

/// TG-1 : En-tête du document de transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDocument {
    /// TT-1 : Identifiant de la transmission
    pub id: String,
    /// TT-2 : Nom du flux
    pub name: Option<String>,
    /// TT-3 : Date et heure de création (format YYYYMMDDHHmmss)
    pub issue_datetime: String,
    /// TT-4 : Type de transmission
    pub type_code: ReportTypeCode,
    /// TG-3 : Émetteur du document
    pub sender: ReportParty,
    /// TG-5 : Déclarant
    pub issuer: ReportParty,
}

/// Type de transmission — TT-4
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReportTypeCode {
    /// Flux 10.1 : Transactions B2C/B2BInt
    TransactionsInitial,
    /// Flux 10.2 : Paiements
    PaymentsInitial,
    /// Flux 10.3 : Transactions agrégées
    TransactionsAggregated,
    /// Flux 10.4 : Paiements agrégés
    PaymentsAggregated,
}

impl ReportTypeCode {
    pub fn code(&self) -> &str {
        match self {
            Self::TransactionsInitial => "10.1",
            Self::PaymentsInitial => "10.2",
            Self::TransactionsAggregated => "10.3",
            Self::PaymentsAggregated => "10.4",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "10.1" => Some(Self::TransactionsInitial),
            "10.2" => Some(Self::PaymentsInitial),
            "10.3" => Some(Self::TransactionsAggregated),
            "10.4" => Some(Self::PaymentsAggregated),
            _ => None,
        }
    }
}

impl std::fmt::Display for ReportTypeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Acteur dans un rapport e-reporting (émetteur ou déclarant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportParty {
    /// TT-7/TT-12 : Type d'identifiant (schemeId)
    pub id_scheme: String,
    /// TT-8/TT-13 : Identifiant
    pub id: String,
    /// TT-9/TT-14 : Raison sociale
    pub name: String,
    /// TT-10/TT-15 : Code rôle
    pub role_code: String,
    /// TT-11/TT-16 : Adresse électronique (réseau CEF)
    pub endpoint_uri: Option<String>,
}

/// TB-2 : Rapport de transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsReport {
    /// TG-7 : Période de transmission
    pub period_start: String,
    pub period_end: String,
    /// TG-8 : Factures (flux 10.1 — transactions détaillées)
    #[serde(default)]
    pub invoices: Vec<TransactionInvoice>,
    /// TG-31 : Transactions agrégées (flux 10.3)
    #[serde(default)]
    pub aggregated_transactions: Vec<AggregatedTransaction>,
}

/// TG-8 : Facture dans un rapport de transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInvoice {
    /// TT-19 : Identification de la facture
    pub id: String,
    /// TT-20 : Date d'émission
    pub issue_date: String,
    /// TT-21 : Code de type de facture
    pub type_code: String,
    /// TT-22 : Code de devise
    pub currency_code: String,
    /// TT-201 : Date d'échéance
    pub due_date: Option<String>,
    /// TT-24 : Code de date d'exigibilité TVA
    pub tax_due_date_type_code: Option<String>,
    /// TG-9 : Notes de facture
    pub notes: Vec<InvoiceNote>,
    /// TG-10 : Processus métier
    pub business_process: BusinessProcess,
    /// TG-11 : Références à des factures antérieures
    pub referenced_documents: Vec<ReferencedInvoice>,
    /// TG-12 : Vendeur
    pub seller: TransactionParty,
    /// TG-14 : Acheteur
    pub buyer: Option<TransactionParty>,
    /// TG-16 : Représentant fiscal du vendeur
    pub seller_tax_representative: Option<TaxRepresentative>,
    /// TG-17 : Informations de livraison
    pub deliveries: Vec<Delivery>,
    /// TG-18 : Période de facturation
    pub invoice_period: Option<InvoicePeriod>,
    /// TG-20/TG-21 : Remises et charges au niveau document
    pub allowance_charges: Vec<AllowanceCharge>,
    /// TG-22 : Totaux du document
    pub monetary_total: MonetaryTotal,
    /// TG-23 : Ventilation de la TVA
    pub tax_subtotals: Vec<TaxSubTotal>,
    /// TG-24 : Lignes de facture
    pub lines: Vec<InvoiceLine>,
}

/// TG-9 : Note de facture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceNote {
    /// TT-26 : Code du sujet
    pub subject: Option<String>,
    /// TT-27 : Contenu
    pub content: Option<String>,
}

/// TG-10 : Processus métier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessProcess {
    /// TT-28 : Type de processus métier (cadre de facturation)
    pub id: String,
    /// TT-29 : Type de profil
    pub type_id: String,
}

/// TG-11 : Référence à une facture antérieure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencedInvoice {
    /// TT-30 : Référence
    pub id: String,
    /// TT-31 : Date d'émission
    pub issue_date: Option<String>,
}

/// Acteur dans une transaction (vendeur ou acheteur)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionParty {
    /// TT-33/TT-36 : Identifiant
    pub company_id: Option<String>,
    /// TT-33-1/TT-37 : schemeId de l'identifiant
    pub company_id_scheme: Option<String>,
    /// TT-34/TT-38 : Identifiant TVA
    pub tax_registration_id: Option<String>,
    /// TT-34-0/TT-38-0 : Qualifiant TVA
    pub tax_qualifying_id: Option<String>,
    /// TT-35/TT-39 : Code pays
    pub country_code: Option<String>,
}

/// TG-16 : Représentant fiscal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRepresentative {
    /// TT-122 : Identifiant TVA
    pub tax_registration_id: String,
    /// TT-40 : schemeId
    pub scheme_id: String,
}

/// TG-17 : Informations de livraison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delivery {
    /// TT-41 : Date effective de livraison
    pub date: Option<String>,
    /// TG-19 : Adresse de livraison
    pub location: Option<DeliveryLocation>,
}

/// TG-19 : Adresse de livraison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryLocation {
    pub line_one: Option<String>,
    pub line_two: Option<String>,
    pub line_three: Option<String>,
    pub city_name: Option<String>,
    pub postal_zone: Option<String>,
    pub country_subentity: Option<String>,
    /// TT-44 : Code pays
    pub country_code: Option<String>,
}

/// TG-18 : Période de facturation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoicePeriod {
    /// TT-42 : Date de début
    pub start_date: Option<String>,
    /// TT-43 : Date de fin
    pub end_date: Option<String>,
}

/// TG-20/TG-21 : Remise ou charge au niveau document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowanceCharge {
    /// Indicateur : true = charge, false = remise
    pub charge_indicator: bool,
    /// TT-45/TT-48 : Montant
    pub amount: Option<f64>,
    /// TT-46/TT-49 : Code de type de TVA
    pub tax_category_code: Option<String>,
    /// TT-47/TT-50 : Taux de TVA
    pub tax_percent: Option<f64>,
}

/// TG-22 : Totaux du document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryTotal {
    /// TT-51 : Montant total HT
    pub tax_exclusive_amount: Option<f64>,
    /// TT-52 : Montant total de TVA
    pub tax_amount: f64,
    /// TT-202 : Devise du montant de TVA
    pub tax_amount_currency: String,
}

/// TG-23 : Ventilation de la TVA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSubTotal {
    /// TT-54 : Base d'imposition
    pub taxable_amount: f64,
    /// TT-55 : Montant de TVA
    pub tax_amount: f64,
    /// TT-56 : Code de type de TVA
    pub tax_category_code: Option<String>,
    /// TT-57 : Taux de TVA
    pub tax_percent: f64,
    /// TT-58 : Motif d'exonération
    pub tax_exemption_reason: Option<String>,
    /// TT-59 : Code motif d'exonération
    pub tax_exemption_reason_code: Option<String>,
}

/// TG-24 : Ligne de facture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    /// Notes de ligne
    pub notes: Vec<LineNote>,
    /// Remises/charges au niveau ligne
    pub allowance_charges: Vec<AllowanceCharge>,
    /// TT-62 : Montant net de la ligne
    pub line_net_amount: Option<f64>,
    /// TT-63 : Quantité facturée
    pub invoiced_quantity: Option<f64>,
    /// TT-64 : Unité de mesure
    pub invoiced_quantity_unit: Option<String>,
    /// TT-65 : Prix unitaire net
    pub price_amount: Option<f64>,
    /// TT-66 : Code de type de TVA de la ligne
    pub tax_category_code: Option<String>,
    /// TT-67 : Taux de TVA de la ligne
    pub tax_percent: Option<f64>,
}

/// Note de ligne de facture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineNote {
    pub code: Option<String>,
    pub comment: Option<String>,
}

// ============================================================
// Flux 10.3 : Transactions agregees (TG-31)
// ============================================================

/// Categorie de transaction pour l'e-reporting agrege
/// Determine a partir de l'analyse des lignes de facture
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransactionCategory {
    /// TLB1 : Livraison de biens
    TLB1,
    /// TPS1 : Prestation de services
    TPS1,
    /// TNT1 : Opération non taxable en France
    TNT1,
    /// TMA1 : Opération sous le régime de la marge
    TMA1,
}

impl TransactionCategory {
    pub fn code(&self) -> &str {
        match self {
            Self::TLB1 => "TLB1",
            Self::TPS1 => "TPS1",
            Self::TNT1 => "TNT1",
            Self::TMA1 => "TMA1",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "TLB1" => Some(Self::TLB1),
            "TPS1" => Some(Self::TPS1),
            "TNT1" => Some(Self::TNT1),
            "TMA1" => Some(Self::TMA1),
            _ => None,
        }
    }
}

impl std::fmt::Display for TransactionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// TG-31 : Transaction agrégée dans un rapport de transactions (flux 10.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedTransaction {
    /// TT-77 : Date de la transaction (YYYYMMDD)
    pub date: String,
    /// TT-78 : Code devise
    pub currency_code: String,
    /// TT-80 : Option de paiement TVA (debit / non-debit)
    pub vat_payment_option: Option<String>,
    /// TT-81 : Categorie de transaction
    pub category: TransactionCategory,
    /// TT-82 : Montant cumule HT
    pub cumulative_amount_ht: f64,
    /// TT-83 : Montant cumule TVA (toujours en EUR)
    pub cumulative_tax_amount_eur: f64,
    /// TT-85 : Nombre de transactions
    pub transaction_count: u32,
    /// TG-32 : Ventilation par taux de TVA
    pub tax_subtotals: Vec<AggregatedTaxSubTotal>,
}

/// TG-32 : Ventilation TVA d'une transaction agrégée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedTaxSubTotal {
    /// TT-86 : Base d'imposition cumulée
    pub taxable_amount: f64,
    /// TT-87 : Montant TVA cumulé
    pub tax_amount: f64,
    /// TT-88 : Code catégorie TVA
    pub tax_category_code: String,
    /// TT-89 : Taux de TVA
    pub tax_percent: f64,
}

/// TB-3 : Rapport de paiements (flux 10.2 / 10.4) — conforme payment.xsd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentsReport {
    /// TG-33 : Période de transmission
    pub period_start: String,
    pub period_end: String,
    /// TG-34 : Paiements par facture (flux 10.2)
    #[serde(default)]
    pub invoices: Vec<PaymentInvoice>,
    /// TG-37 : Paiements agrégés par transaction (flux 10.4)
    #[serde(default)]
    pub transactions: Vec<PaymentTransaction>,
}

/// TG-34 : Facture dans un rapport de paiements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInvoice {
    /// TT-91 : Numéro de facture
    pub invoice_id: String,
    /// TT-102 : Date de la facture (format YYYYMMDD)
    pub issue_date: String,
    /// TG-35 : Paiement associé
    pub payment: PaymentDetail,
}

/// TG-37 : Transaction agrégée dans un rapport de paiements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    /// TG-38 : Paiement associé
    pub payment: PaymentDetail,
}

/// TG-35/TG-38 : Détail d'un paiement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDetail {
    /// TT-92/TT-96 : Date du paiement (format YYYYMMDD)
    pub date: String,
    /// TG-36/TG-39 : Répartition par taux de TVA
    pub sub_totals: Vec<PaymentSubTotal>,
}

/// TG-36/TG-39 : Répartition par taux de TVA d'un paiement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSubTotal {
    /// TT-93/TT-97 : Taux de TVA
    pub tax_percent: f64,
    /// TT-94/TT-98 : Code devise du paiement
    pub currency_code: Option<String>,
    /// TT-95/TT-99 : Montant encaissé
    pub amount: f64,
}
