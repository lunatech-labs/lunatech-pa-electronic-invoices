use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Format de facture supporté
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceFormat {
    UBL,
    CII,
    FacturX,
}

impl std::fmt::Display for InvoiceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceFormat::UBL => write!(f, "UBL"),
            InvoiceFormat::CII => write!(f, "CII"),
            InvoiceFormat::FacturX => write!(f, "Factur-X"),
        }
    }
}

/// Statut d'un flux de facture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlowStatus {
    Received,
    Parsing,
    Parsed,
    Validating,
    Validated,
    Transforming,
    Transformed,
    Distributing,
    Distributed,
    WaitingAck,
    Acknowledged,
    Rejected,
    Cancelled,
    Error,
}

impl std::fmt::Display for FlowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FlowStatus::Received => "REÇU",
            FlowStatus::Parsing => "PARSING",
            FlowStatus::Parsed => "PARSÉ",
            FlowStatus::Validating => "VALIDATION",
            FlowStatus::Validated => "VALIDÉ",
            FlowStatus::Transforming => "TRANSFORMATION",
            FlowStatus::Transformed => "TRANSFORMÉ",
            FlowStatus::Distributing => "DISTRIBUTION",
            FlowStatus::Distributed => "DISTRIBUÉ",
            FlowStatus::WaitingAck => "ATTENTE_ACK",
            FlowStatus::Acknowledged => "ACQUITTÉ",
            FlowStatus::Rejected => "REJETÉ",
            FlowStatus::Cancelled => "ANNULÉ",
            FlowStatus::Error => "ERREUR",
        };
        write!(f, "{}", s)
    }
}

/// Type de document entrant dans le pipeline.
///
/// Permet de distinguer les factures des CDAR (statuts de cycle de vie)
/// et des déclarations e-reporting, pour router vers le bon traitement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentType {
    /// Facture (UBL Invoice/CreditNote, CII CrossIndustryInvoice, Factur-X)
    Invoice,
    /// Compte-rendu De Vie (CDAR D22B)
    Cdar,
    /// Déclaration e-reporting
    EReporting,
    /// Type non déterminé
    Unknown,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Invoice => write!(f, "Invoice"),
            DocumentType::Cdar => write!(f, "CDAR"),
            DocumentType::EReporting => write!(f, "EReporting"),
            DocumentType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Type de destinataire
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecipientType {
    Buyer,
    PPF,
    OtherPDP,
}

/// Profil de facturation (BT-24)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceProfile {
    /// Profil de base (champs minimaux)
    Base,
    /// Profil complet (tous les champs)
    Full,
}

impl std::fmt::Display for InvoiceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base => write!(f, "Base"),
            Self::Full => write!(f, "Full"),
        }
    }
}

/// Clef primaire métier d'une facture.
/// Conforme aux spécifications françaises : une facture est identifiée de manière unique
/// par le triplet (SIREN de l'émetteur, numéro de facture, année d'émission).
/// BR-FR-01 garantit l'unicité du numéro de facture au sein d'un SIREN et d'une année.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvoiceKey {
    /// SIREN de l'émetteur (9 chiffres, extrait des 9 premiers caractères du SIRET vendeur)
    pub seller_siren: String,
    /// BT-1 : Numéro de facture
    pub invoice_number: String,
    /// Année d'émission (extraite de BT-2 issue_date)
    pub issue_year: u16,
}

impl InvoiceKey {
    pub fn new(seller_siren: &str, invoice_number: &str, issue_year: u16) -> Self {
        Self {
            seller_siren: seller_siren.to_string(),
            invoice_number: invoice_number.to_string(),
            issue_year,
        }
    }

    /// Représentation sous forme de chaîne : "SIREN/NUMERO/ANNEE"
    pub fn to_string_key(&self) -> String {
        format!("{}/{}/{}", self.seller_siren, self.invoice_number, self.issue_year)
    }

    /// Parse une clef depuis sa représentation chaîne "SIREN/NUMERO/ANNEE"
    /// Le SIREN est avant le premier '/', l'année est après le dernier '/',
    /// le numéro de facture est entre les deux (peut contenir des '/').
    pub fn from_string_key(key: &str) -> Option<Self> {
        let first_slash = key.find('/')?;
        let last_slash = key.rfind('/')?;
        if first_slash == last_slash {
            return None; // Il faut au moins 2 '/'
        }
        let siren = &key[..first_slash];
        let number = &key[first_slash + 1..last_slash];
        let year_str = &key[last_slash + 1..];
        if number.is_empty() {
            return None;
        }
        let year = year_str.parse::<u16>().ok()?;
        Some(Self {
            seller_siren: siren.to_string(),
            invoice_number: number.to_string(),
            issue_year: year,
        })
    }
}

impl std::fmt::Display for InvoiceKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_key())
    }
}

/// Données d'une facture parsée — conforme EN16931 / specs v3.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceData {
    /// Identifiant technique interne (UUID)
    pub id: Uuid,
    /// BT-1 : Numéro de facture
    pub invoice_number: String,
    /// BT-2 : Date d'émission (YYYY-MM-DD)
    pub issue_date: Option<String>,
    /// BT-9 : Date d'échéance (YYYY-MM-DD)
    pub due_date: Option<String>,
    /// BT-3 : Code type de document
    /// Factures simples: 380, 389(auto), 393(affacturée), 501(auto+affacturée)
    /// Acomptes: 386, 500(auto)
    /// Rectificatives: 384, 471(auto), 472(affacturée), 473(auto+affacturée)
    /// Avoirs: 261(auto), 262(remise globale), 381, 396(affacturé), 502(auto+affacturé), 503(acompte)
    pub invoice_type_code: Option<String>,
    /// BT-5 : Code devise
    pub currency: Option<String>,
    /// BT-6 : Code devise de la TVA (si différent)
    pub tax_currency: Option<String>,
    /// BT-23 : Cadre de facturation (S1, B1, etc.)
    pub business_process: Option<String>,
    /// BT-24 : Profil (Base ou Full)
    pub profile: Option<InvoiceProfile>,
    /// BT-24 raw : URI du profil (urn.cpro.gouv.fr:1p0:einvoicingextract#Base/Full)
    pub profile_id: Option<String>,

    // --- Vendeur (BG-4) ---
    /// BT-27 : Nom du vendeur
    pub seller_name: Option<String>,
    /// BT-28 : Nom commercial du vendeur
    pub seller_trading_name: Option<String>,
    /// BT-29 : Identifiant du vendeur (GlobalID, schemeID ex: 0088=GLN)
    pub seller_id: Option<String>,
    /// BT-29 schemeID
    pub seller_id_scheme: Option<String>,
    /// BT-30 : SIREN/SIRET du vendeur (schemeID 0002)
    pub seller_siret: Option<String>,
    /// BT-31 : Numéro TVA du vendeur
    pub seller_vat_id: Option<String>,
    /// BT-40 : Code pays du vendeur
    pub seller_country: Option<String>,
    /// BG-5 : Adresse postale du vendeur
    pub seller_address: Option<PostalAddress>,
    /// BT-34 : Identifiant électronique du vendeur
    pub seller_endpoint_id: Option<String>,
    /// BT-34-1 : Scheme ID de l'adresse électronique du vendeur (ex: 0225)
    pub seller_endpoint_scheme: Option<String>,

    // --- Acheteur (BG-7) ---
    /// BT-44 : Nom de l'acheteur
    pub buyer_name: Option<String>,
    /// BT-45 : Nom commercial de l'acheteur
    pub buyer_trading_name: Option<String>,
    /// BT-46 : Identifiant de l'acheteur (GlobalID)
    pub buyer_id: Option<String>,
    /// BT-46 schemeID
    pub buyer_id_scheme: Option<String>,
    /// BT-47 : SIREN/SIRET de l'acheteur (schemeID 0002)
    pub buyer_siret: Option<String>,
    /// BT-48 : Numéro TVA de l'acheteur
    pub buyer_vat_id: Option<String>,
    /// BT-55 : Code pays de l'acheteur
    pub buyer_country: Option<String>,
    /// BG-8 : Adresse postale de l'acheteur
    pub buyer_address: Option<PostalAddress>,
    /// BT-49 : Identifiant électronique de l'acheteur
    pub buyer_endpoint_id: Option<String>,
    /// BT-49-1 : Scheme ID de l'adresse électronique de l'acheteur (ex: 0225)
    pub buyer_endpoint_scheme: Option<String>,
    /// BT-19 : Référence comptable de l'acheteur
    pub buyer_accounting_reference: Option<String>,

    // --- Bénéficiaire du paiement (BG-10) ---
    /// BT-59 : Nom du bénéficiaire (si différent du vendeur)
    pub payee_name: Option<String>,
    /// BT-60 : Identifiant du bénéficiaire
    pub payee_id: Option<String>,
    /// BT-60 schemeID
    pub payee_id_scheme: Option<String>,
    /// BT-61 : SIRET du bénéficiaire
    pub payee_siret: Option<String>,

    // --- Payeur tiers (sous-traitance avec délégation de paiement) ---
    /// Nom du payeur tiers (quand différent de l'acheteur)
    pub payer_name: Option<String>,
    /// Identifiant du payeur tiers (SIREN/SIRET)
    pub payer_id: Option<String>,

    // --- Mandataire de facturation (auto-facturation / marketplace) ---
    /// Nom du mandataire de facturation (tiers qui émet la facture)
    pub billing_mandate_name: Option<String>,
    /// Identifiant du mandataire (SIREN/SIRET)
    pub billing_mandate_id: Option<String>,
    /// Référence du mandat de facturation
    pub billing_mandate_reference: Option<String>,

    // --- Facturant / Délégation de facturation (EXT-FR-FE-BG-05, rôle II) ---
    /// Nom du facturant (tiers qui crée la facture pour le compte du vendeur)
    pub invoicer_name: Option<String>,
    /// Identifiant du facturant (SIREN/SIRET)
    pub invoicer_id: Option<String>,
    /// Numéro TVA du facturant
    pub invoicer_vat_id: Option<String>,

    // --- Adressé à (EXT-FR-FE-BG-04, rôle IV) ---
    /// Nom du destinataire de la facture (si différent de l'acheteur)
    pub addressed_to_name: Option<String>,
    /// Identifiant du destinataire (SIREN/SIRET)
    pub addressed_to_id: Option<String>,

    // --- Agent de l'acheteur (EXT-FR-FE-BG-01, rôle AB) ---
    /// Nom de l'agent de l'acheteur
    pub buyer_agent_name: Option<String>,
    /// Identifiant de l'agent de l'acheteur (SIREN/SIRET)
    pub buyer_agent_id: Option<String>,

    // --- Représentant fiscal du vendeur (BG-11) ---
    /// BT-62 : Nom du représentant fiscal
    pub tax_representative_name: Option<String>,
    /// BT-63 : Numéro TVA du représentant fiscal
    pub tax_representative_vat_id: Option<String>,
    /// BG-12 : Adresse du représentant fiscal
    pub tax_representative_address: Option<PostalAddress>,

    // --- Totaux (BG-22) ---
    /// BT-106 : Total des montants nets de lignes
    pub total_ht: Option<f64>,
    /// BT-112 : Total TTC
    pub total_ttc: Option<f64>,
    /// BT-110 : Total TVA
    pub total_tax: Option<f64>,
    /// Montant TVA converti en EUR (pour les factures en devise étrangère, e-reporting TT-52/TT-83)
    pub tax_amount_eur: Option<f64>,
    /// BT-107 : Total des remises au niveau document
    pub allowance_total_amount: Option<f64>,
    /// BT-108 : Total des charges au niveau document
    pub charge_total_amount: Option<f64>,
    /// BT-109 : Total HT après remises et charges (= BT-106 - BT-107 + BT-108)
    pub total_without_vat: Option<f64>,
    /// BT-113 : Montant payé (acomptes)
    pub prepaid_amount: Option<f64>,
    /// BT-114 : Montant d'arrondi
    pub rounding_amount: Option<f64>,
    /// BT-115 : Montant dû
    pub payable_amount: Option<f64>,

    // --- Ventilation TVA (BG-23) ---
    pub tax_breakdowns: Vec<TaxBreakdown>,

    // --- Notes (BG-1) ---
    pub notes: Vec<InvoiceNote>,

    // --- Références ---
    /// BT-10 : Référence acheteur (numéro de commande)
    pub buyer_reference: Option<String>,
    /// BT-13 : Référence de commande
    pub order_reference: Option<String>,
    /// BT-25 : Référence de facture précédente (pour les avoirs)
    pub preceding_invoice_reference: Option<String>,
    /// BT-26 : Date de la facture précédente
    pub preceding_invoice_date: Option<String>,
    /// BT-11 : Référence projet
    pub project_reference: Option<String>,
    /// BT-12 : Référence contrat
    pub contract_reference: Option<String>,

    // --- Livraison (BG-13) ---
    /// BT-70 : Nom du destinataire de la livraison
    pub delivery_party_name: Option<String>,
    /// BT-72 : Date de livraison effective
    pub delivery_date: Option<String>,
    /// BG-15 : Adresse de livraison
    pub delivery_address: Option<PostalAddress>,

    // --- Période de facturation (BG-14) ---
    /// BT-73 : Date de début
    pub invoice_period_start: Option<String>,
    /// BT-74 : Date de fin
    pub invoice_period_end: Option<String>,

    // --- Moyens de paiement (BG-16) ---
    /// BT-81 : Code moyen de paiement
    pub payment_means_code: Option<String>,
    /// BT-82 : Texte moyen de paiement
    pub payment_means_text: Option<String>,
    /// BT-83 : Identifiant du mandat de prélèvement
    pub payment_mandate_id: Option<String>,
    /// BT-84 : IBAN
    pub payment_iban: Option<String>,
    /// BT-86 : BIC
    pub payment_bic: Option<String>,
    /// BT-20 : Conditions de paiement (texte libre)
    pub payment_terms: Option<String>,
    /// Indicateur TVA sur les débits (exigibilité à l'encaissement vs livraison)
    /// Utilisé pour le e-reporting TT-80 (option de paiement TVA)
    pub tax_due_on_payment: Option<bool>,

    // --- Lignes de facture (BG-25) ---
    pub lines: Vec<InvoiceLine>,

    // --- Remises/charges au niveau document (BG-20/BG-21) ---
    pub allowance_charges: Vec<DocumentAllowanceCharge>,

    // --- Pièces jointes (BG-24) ---
    pub attachments: Vec<InvoiceAttachment>,

    // --- Format source ---
    pub source_format: InvoiceFormat,
    pub raw_xml: Option<String>,
    pub raw_pdf: Option<Vec<u8>>,
}

impl InvoiceData {
    pub fn new(invoice_number: String, source_format: InvoiceFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            invoice_number,
            issue_date: None,
            due_date: None,
            invoice_type_code: None,
            currency: None,
            tax_currency: None,
            business_process: None,
            profile: None,
            profile_id: None,
            seller_name: None,
            seller_trading_name: None,
            seller_id: None,
            seller_id_scheme: None,
            seller_siret: None,
            seller_vat_id: None,
            seller_country: None,
            seller_address: None,
            seller_endpoint_id: None,
            seller_endpoint_scheme: None,
            buyer_name: None,
            buyer_trading_name: None,
            buyer_id: None,
            buyer_id_scheme: None,
            buyer_siret: None,
            buyer_vat_id: None,
            buyer_country: None,
            buyer_address: None,
            buyer_endpoint_id: None,
            buyer_endpoint_scheme: None,
            buyer_accounting_reference: None,
            payee_name: None,
            payee_id: None,
            payee_id_scheme: None,
            payee_siret: None,
            payer_name: None,
            payer_id: None,
            billing_mandate_name: None,
            billing_mandate_id: None,
            billing_mandate_reference: None,
            invoicer_name: None,
            invoicer_id: None,
            invoicer_vat_id: None,
            addressed_to_name: None,
            addressed_to_id: None,
            buyer_agent_name: None,
            buyer_agent_id: None,
            tax_representative_name: None,
            tax_representative_vat_id: None,
            tax_representative_address: None,
            total_ht: None,
            total_ttc: None,
            total_tax: None,
            tax_amount_eur: None,
            allowance_total_amount: None,
            charge_total_amount: None,
            total_without_vat: None,
            prepaid_amount: None,
            rounding_amount: None,
            payable_amount: None,
            tax_breakdowns: Vec::new(),
            notes: Vec::new(),
            buyer_reference: None,
            order_reference: None,
            preceding_invoice_reference: None,
            preceding_invoice_date: None,
            project_reference: None,
            contract_reference: None,
            delivery_party_name: None,
            delivery_date: None,
            delivery_address: None,
            invoice_period_start: None,
            invoice_period_end: None,
            payment_means_code: None,
            payment_means_text: None,
            payment_mandate_id: None,
            payment_iban: None,
            payment_bic: None,
            payment_terms: None,
            tax_due_on_payment: None,
            lines: Vec::new(),
            allowance_charges: Vec::new(),
            attachments: Vec::new(),
            source_format,
            raw_xml: None,
            raw_pdf: None,
        }
    }

    /// Retourne le SIREN du vendeur (9 premiers caractères du SIRET)
    pub fn seller_siren(&self) -> Option<String> {
        self.seller_siret.as_ref().map(|s| s.chars().take(9).collect())
    }

    /// Retourne le SIREN de l'acheteur
    pub fn buyer_siren(&self) -> Option<String> {
        self.buyer_siret.as_ref().map(|s| s.chars().take(9).collect())
    }

    /// Extrait l'année d'émission depuis issue_date (format YYYY-MM-DD ou YYYYMMDD)
    pub fn issue_year(&self) -> Option<u16> {
        self.issue_date.as_deref().and_then(|d| {
            // Supporte YYYY-MM-DD et YYYYMMDD
            if d.len() >= 4 {
                d[..4].parse::<u16>().ok()
            } else {
                None
            }
        })
    }

    /// Construit la clef primaire métier (SIREN émetteur / numéro facture / année émission).
    /// Retourne None si le SIREN vendeur ou l'année d'émission ne sont pas disponibles.
    pub fn key(&self) -> Option<InvoiceKey> {
        let siren = self.seller_siren()?;
        let year = self.issue_year()?;
        Some(InvoiceKey::new(&siren, &self.invoice_number, year))
    }

    /// Retourne la clef primaire sous forme de chaîne "SIREN/NUMERO/ANNEE",
    /// ou un fallback basé sur l'UUID si les données métier sont incomplètes.
    pub fn key_string(&self) -> String {
        self.key()
            .map(|k| k.to_string_key())
            .unwrap_or_else(|| format!("_/{}/{}", self.invoice_number, self.id))
    }
}

/// Adresse postale (BG-5, BG-8, BG-15)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostalAddress {
    /// BT-35/BT-50/BT-75 : Ligne 1
    pub line1: Option<String>,
    /// BT-36/BT-51/BT-76 : Ligne 2
    pub line2: Option<String>,
    /// BT-162/BT-163/BT-165 : Ligne 3
    pub line3: Option<String>,
    /// BT-37/BT-52/BT-77 : Ville
    pub city: Option<String>,
    /// BT-38/BT-53/BT-78 : Code postal
    pub postal_code: Option<String>,
    /// BT-39/BT-54/BT-79 : Subdivision
    pub country_subdivision: Option<String>,
    /// BT-40/BT-55/BT-80 : Code pays (ISO 3166-1 alpha-2)
    pub country_code: Option<String>,
}

/// Ventilation TVA (BG-23)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBreakdown {
    /// BT-116 : Base d'imposition
    pub taxable_amount: Option<f64>,
    /// BT-117 : Montant de TVA
    pub tax_amount: Option<f64>,
    /// BT-118 : Code catégorie TVA (S, Z, E, AE, K, G, O, L, M)
    pub category_code: Option<String>,
    /// BT-119 : Taux de TVA
    pub percent: Option<f64>,
    /// BT-120 : Motif d'exonération
    pub exemption_reason: Option<String>,
    /// BT-121 : Code motif d'exonération (VATEX-*)
    pub exemption_reason_code: Option<String>,
}

/// Note de facture (BG-1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceNote {
    /// BT-21 : Contenu de la note
    pub content: String,
    /// BT-22 : Code sujet (REG, PMD, PMT, AAB, ABL, AAI, etc.)
    pub subject_code: Option<String>,
}

/// Type de sous-ligne (EXT-FR-FE-162) — XP Z12-012 §3.3.5
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubLineType {
    /// Ligne de détail (sous-ligne contribuant au montant)
    Detail,
    /// Ligne d'information (sous-ligne informative, sans impact montant)
    Information,
    /// Ligne de regroupement (multi-vendeurs B8/S8/M8)
    Group,
}

impl std::fmt::Display for SubLineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Detail => write!(f, "DETAIL"),
            Self::Information => write!(f, "INFORMATION"),
            Self::Group => write!(f, "GROUP"),
        }
    }
}

impl SubLineType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DETAIL" => Some(Self::Detail),
            "INFORMATION" => Some(Self::Information),
            "GROUP" => Some(Self::Group),
            _ => None,
        }
    }
}

/// Remise ou charge au niveau ligne (BG-27 / BG-28) — XP Z12-012 §3.3.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAllowanceCharge {
    /// true = charge (BG-28), false = remise (BG-27)
    pub charge_indicator: bool,
    /// BT-136/BT-141 : Montant de la remise/charge
    pub amount: Option<f64>,
    /// BT-137/BT-142 : Montant de base pour le calcul
    pub base_amount: Option<f64>,
    /// BT-138/BT-143 : Pourcentage de la remise/charge
    pub percentage: Option<f64>,
    /// BT-139/BT-144 : Motif de la remise/charge
    pub reason: Option<String>,
    /// BT-140/BT-145 : Code motif de la remise/charge
    pub reason_code: Option<String>,
}

/// Ligne de facture (BG-25)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    /// BT-126 : Identifiant de la ligne
    pub line_id: Option<String>,
    /// BT-127 : Note de ligne
    pub note: Option<String>,
    /// BT-128 : Identifiant objet de la ligne (référence comptable)
    pub object_id: Option<String>,
    /// BT-129 : Quantité facturée
    pub quantity: Option<f64>,
    /// BT-130 : Unité de mesure
    pub unit_code: Option<String>,
    /// BT-131 : Montant net de la ligne
    pub line_net_amount: Option<f64>,
    /// BT-132 : Référence de commande de la ligne
    pub order_line_reference: Option<String>,
    /// BT-133 : Référence comptable de la ligne
    pub accounting_cost: Option<String>,
    /// BT-146 : Prix unitaire net
    pub price: Option<f64>,
    /// BT-147 : Rabais sur prix (remise unitaire, prix brut - prix net)
    pub price_discount: Option<f64>,
    /// BT-148 : Prix brut
    pub gross_price: Option<f64>,
    /// BT-149 : Quantité de base du prix unitaire (pour calcul BT-131)
    /// Formule : BT-131 = (BT-146 / BT-149) × BT-129 - remises + charges
    pub base_quantity: Option<f64>,
    /// BT-150 : Unité de mesure de la quantité de base
    pub base_quantity_unit_code: Option<String>,
    /// BT-153 : Nom de l'article
    pub item_name: Option<String>,
    /// BT-154 : Description de l'article
    pub item_description: Option<String>,
    /// BT-155 : Identifiant article vendeur
    pub seller_item_id: Option<String>,
    /// BT-157 : Identifiant article acheteur
    pub buyer_item_id: Option<String>,
    /// BT-158 : Identifiant article standard (ex: GTIN/EAN)
    pub standard_item_id: Option<String>,
    /// BT-158 schemeID (ex: 0160 pour GTIN)
    pub standard_item_id_scheme: Option<String>,
    /// BT-151 : Code catégorie TVA de la ligne
    pub tax_category_code: Option<String>,
    /// BT-152 : Taux de TVA de la ligne
    pub tax_percent: Option<f64>,
    /// BG-26 : Période de facturation de la ligne
    pub period_start: Option<String>,
    pub period_end: Option<String>,

    // --- Remises/charges au niveau ligne (BG-27/BG-28) ---
    /// Remises et charges applicables à cette ligne
    pub allowance_charges: Vec<LineAllowanceCharge>,

    // --- Sous-lignes (EXT-FR-FE-162/163) — XP Z12-012 §3.3.5 ---
    /// Type de sous-ligne (DETAIL, INFORMATION, GROUP)
    pub line_type: Option<SubLineType>,
    /// Sous-lignes rattachées à cette ligne (récursif)
    pub sub_lines: Vec<InvoiceLine>,
}

/// Remise ou charge au niveau document (BG-20 / BG-21)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAllowanceCharge {
    /// true = charge (BG-21), false = remise (BG-20)
    pub charge_indicator: bool,
    /// BT-92/BT-99 : Montant de la remise/charge
    pub amount: Option<f64>,
    /// BT-93/BT-100 : Montant de base pour le calcul
    pub base_amount: Option<f64>,
    /// BT-94/BT-101 : Pourcentage de la remise/charge
    pub percentage: Option<f64>,
    /// BT-95/BT-102 : Code catégorie TVA
    pub tax_category_code: Option<String>,
    /// BT-96/BT-103 : Taux de TVA
    pub tax_percent: Option<f64>,
    /// BT-97/BT-104 : Motif
    pub reason: Option<String>,
    /// BT-98/BT-105 : Code motif (UNTDID 5189 pour remises, UNTDID 7161 pour charges)
    pub reason_code: Option<String>,
}

/// Pièce jointe (BG-24)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceAttachment {
    /// BT-122 : Référence du document
    pub id: Option<String>,
    /// BT-123 : Description
    pub description: Option<String>,
    /// BT-124 : URI externe
    pub external_uri: Option<String>,
    /// BT-125 : Contenu embarqué (base64)
    pub embedded_content: Option<Vec<u8>>,
    /// Code MIME
    pub mime_code: Option<String>,
    /// Nom du fichier
    pub filename: Option<String>,
}

/// Événement de flux pour la traçabilité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    pub id: Uuid,
    pub flow_id: Uuid,
    /// Clef primaire métier de la facture (format "SIREN/NUMERO/ANNEE")
    pub invoice_key: Option<String>,
    pub route_id: String,
    pub status: FlowStatus,
    pub message: String,
    pub error_detail: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl FlowEvent {
    pub fn new(flow_id: Uuid, route_id: &str, status: FlowStatus, message: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            flow_id,
            invoice_key: None,
            route_id: route_id.to_string(),
            status,
            message: message.to_string(),
            error_detail: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error_detail = Some(error.to_string());
        self.status = FlowStatus::Error;
        self
    }

    pub fn with_invoice_key(mut self, key: &str) -> Self {
        self.invoice_key = Some(key.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ===== InvoiceKey =====

    #[test]
    fn test_invoice_key_new() {
        let key = InvoiceKey::new("123456789", "FA-2025-001", 2025);
        assert_eq!(key.seller_siren, "123456789");
        assert_eq!(key.invoice_number, "FA-2025-001");
        assert_eq!(key.issue_year, 2025);
    }

    #[test]
    fn test_invoice_key_to_string_key() {
        let key = InvoiceKey::new("123456789", "FA-2025-001", 2025);
        assert_eq!(key.to_string_key(), "123456789/FA-2025-001/2025");
    }

    #[test]
    fn test_invoice_key_display() {
        let key = InvoiceKey::new("123456789", "FA-2025-001", 2025);
        assert_eq!(format!("{}", key), "123456789/FA-2025-001/2025");
    }

    #[test]
    fn test_invoice_key_from_string_key() {
        let key = InvoiceKey::from_string_key("123456789/FA-2025-001/2025").unwrap();
        assert_eq!(key.seller_siren, "123456789");
        assert_eq!(key.invoice_number, "FA-2025-001");
        assert_eq!(key.issue_year, 2025);
    }

    #[test]
    fn test_invoice_key_from_string_key_with_slashes_in_number() {
        let key = InvoiceKey::from_string_key("123456789/FA/2025/001/2025").unwrap();
        assert_eq!(key.seller_siren, "123456789");
        assert_eq!(key.invoice_number, "FA/2025/001");
        assert_eq!(key.issue_year, 2025);
    }

    #[test]
    fn test_invoice_key_from_string_key_invalid() {
        assert!(InvoiceKey::from_string_key("").is_none());
        assert!(InvoiceKey::from_string_key("123456789").is_none());
        assert!(InvoiceKey::from_string_key("123456789/FA-001").is_none());
        assert!(InvoiceKey::from_string_key("123456789/FA-001/XXXX").is_none());
    }

    #[test]
    fn test_invoice_key_roundtrip() {
        let original = InvoiceKey::new("987654321", "RECT-2024-042", 2024);
        let serialized = original.to_string_key();
        let parsed = InvoiceKey::from_string_key(&serialized).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_invoice_key_hash_eq() {
        let k1 = InvoiceKey::new("123456789", "FA-001", 2025);
        let k2 = InvoiceKey::new("123456789", "FA-001", 2025);
        let k3 = InvoiceKey::new("123456789", "FA-001", 2024);
        let k4 = InvoiceKey::new("999999999", "FA-001", 2025);

        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        assert_ne!(k1, k4);

        let mut set = HashSet::new();
        set.insert(k1.clone());
        assert!(set.contains(&k2));
        assert!(!set.contains(&k3));
    }

    // ===== InvoiceData.key() =====

    #[test]
    fn test_invoice_data_key_complete() {
        let mut inv = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::CII);
        inv.seller_siret = Some("12345678900015".to_string());
        inv.issue_date = Some("2025-07-01".to_string());

        let key = inv.key().unwrap();
        assert_eq!(key.seller_siren, "123456789");
        assert_eq!(key.invoice_number, "FA-2025-001");
        assert_eq!(key.issue_year, 2025);
    }

    #[test]
    fn test_invoice_data_key_string_complete() {
        let mut inv = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::UBL);
        inv.seller_siret = Some("12345678900015".to_string());
        inv.issue_date = Some("2025-07-01".to_string());

        assert_eq!(inv.key_string(), "123456789/FA-2025-001/2025");
    }

    #[test]
    fn test_invoice_data_key_missing_siret() {
        let mut inv = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::CII);
        inv.issue_date = Some("2025-07-01".to_string());

        assert!(inv.key().is_none());
        assert!(inv.key_string().starts_with("_/FA-2025-001/"));
    }

    #[test]
    fn test_invoice_data_key_missing_date() {
        let mut inv = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::CII);
        inv.seller_siret = Some("12345678900015".to_string());

        assert!(inv.key().is_none());
        assert!(inv.key_string().starts_with("_/FA-2025-001/"));
    }

    #[test]
    fn test_invoice_data_issue_year_iso() {
        let mut inv = InvoiceData::new("X".to_string(), InvoiceFormat::CII);
        inv.issue_date = Some("2024-12-31".to_string());
        assert_eq!(inv.issue_year(), Some(2024));
    }

    #[test]
    fn test_invoice_data_issue_year_compact() {
        let mut inv = InvoiceData::new("X".to_string(), InvoiceFormat::CII);
        inv.issue_date = Some("20251231".to_string());
        assert_eq!(inv.issue_year(), Some(2025));
    }

    #[test]
    fn test_invoice_data_issue_year_none() {
        let inv = InvoiceData::new("X".to_string(), InvoiceFormat::CII);
        assert_eq!(inv.issue_year(), None);
    }

    #[test]
    fn test_invoice_data_seller_siren_from_siret() {
        let mut inv = InvoiceData::new("X".to_string(), InvoiceFormat::CII);
        inv.seller_siret = Some("12345678900015".to_string());
        assert_eq!(inv.seller_siren(), Some("123456789".to_string()));
    }

    #[test]
    fn test_invoice_data_seller_siren_short() {
        let mut inv = InvoiceData::new("X".to_string(), InvoiceFormat::CII);
        inv.seller_siret = Some("123456789".to_string());
        assert_eq!(inv.seller_siren(), Some("123456789".to_string()));
    }

    // ===== InvoiceKey dans le pipeline (via ParseProcessor) =====

    #[test]
    fn test_key_consistency_same_invoice() {
        let mut inv1 = InvoiceData::new("FA-001".to_string(), InvoiceFormat::CII);
        inv1.seller_siret = Some("11111111100001".to_string());
        inv1.issue_date = Some("2025-01-15".to_string());

        let mut inv2 = InvoiceData::new("FA-001".to_string(), InvoiceFormat::UBL);
        inv2.seller_siret = Some("11111111100001".to_string());
        inv2.issue_date = Some("2025-06-30".to_string());

        assert_eq!(inv1.key(), inv2.key(), "Même SIREN+numéro+année = même clef");
    }

    #[test]
    fn test_key_differs_different_year() {
        let mut inv1 = InvoiceData::new("FA-001".to_string(), InvoiceFormat::CII);
        inv1.seller_siret = Some("11111111100001".to_string());
        inv1.issue_date = Some("2024-12-31".to_string());

        let mut inv2 = InvoiceData::new("FA-001".to_string(), InvoiceFormat::CII);
        inv2.seller_siret = Some("11111111100001".to_string());
        inv2.issue_date = Some("2025-01-01".to_string());

        assert_ne!(inv1.key(), inv2.key(), "Année différente = clef différente");
    }

    #[test]
    fn test_key_differs_different_siren() {
        let mut inv1 = InvoiceData::new("FA-001".to_string(), InvoiceFormat::CII);
        inv1.seller_siret = Some("11111111100001".to_string());
        inv1.issue_date = Some("2025-01-15".to_string());

        let mut inv2 = InvoiceData::new("FA-001".to_string(), InvoiceFormat::CII);
        inv2.seller_siret = Some("22222222200002".to_string());
        inv2.issue_date = Some("2025-01-15".to_string());

        assert_ne!(inv1.key(), inv2.key(), "SIREN différent = clef différente");
    }
}
