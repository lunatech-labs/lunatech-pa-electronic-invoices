use serde::{Deserialize, Serialize};

/// Type de CDV (Compte-rendu De Vie) — MDT-77
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CdvTypeCode {
    /// Phase Traitement (acheteur → vendeur)
    Traitement,
    /// Phase Transmission (PDP → PDP)
    Transmission,
}

impl CdvTypeCode {
    pub fn code(&self) -> &str {
        match self {
            CdvTypeCode::Traitement => "23",
            CdvTypeCode::Transmission => "305",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "23" => Some(CdvTypeCode::Traitement),
            "305" => Some(CdvTypeCode::Transmission),
            _ => None,
        }
    }
}

impl std::fmt::Display for CdvTypeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Code statut de traitement de la facture — MDT-105
/// Conforme BR-FR-CDV-CL-06
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum InvoiceStatusCode {
    Deposee = 200,
    Emise = 201,
    Recue = 202,
    MiseADisposition = 203,
    PriseEnCharge = 204,
    Approuvee = 205,
    ApprouveePartiellement = 206,
    EnLitige = 207,
    Suspendue = 208,
    Completee = 209,
    Refusee = 210,
    PaiementTransmis = 211,
    Encaissee = 212,
    Rejetee = 213,
    Visee = 214,
    Annulee = 220,
    ErreurRoutage = 221,
    DemandePaiementDirect = 224,
    Affacturee = 225,
    AffactureeConfidentiel = 226,
    ChangementCompteAPayer = 227,
    NonAffacturee = 228,
    Irrecevable = 501,
}

impl InvoiceStatusCode {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Deposee => "Déposée",
            Self::Emise => "Émise",
            Self::Recue => "Reçue",
            Self::MiseADisposition => "Mise_à_disposition",
            Self::PriseEnCharge => "Prise_en_charge",
            Self::Approuvee => "Approuvée",
            Self::ApprouveePartiellement => "Approuvée_partiellement",
            Self::EnLitige => "En_litige",
            Self::Suspendue => "Suspendue",
            Self::Completee => "Complétée",
            Self::Refusee => "Refusée",
            Self::PaiementTransmis => "Paiement_transmis",
            Self::Encaissee => "Encaissée",
            Self::Rejetee => "Rejetée",
            Self::Visee => "Visée",
            Self::Annulee => "Annulée",
            Self::ErreurRoutage => "ERREUR_ROUTAGE",
            Self::DemandePaiementDirect => "Demande_de_Paiement_Direct",
            Self::Affacturee => "Affacturée",
            Self::AffactureeConfidentiel => "Affacturée_Confidentiel",
            Self::ChangementCompteAPayer => "Changement_de_Compte_à_Payer",
            Self::NonAffacturee => "Non_Affacturée",
            Self::Irrecevable => "Irrecevable",
        }
    }

    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            200 => Some(Self::Deposee),
            201 => Some(Self::Emise),
            202 => Some(Self::Recue),
            203 => Some(Self::MiseADisposition),
            204 => Some(Self::PriseEnCharge),
            205 => Some(Self::Approuvee),
            206 => Some(Self::ApprouveePartiellement),
            207 => Some(Self::EnLitige),
            208 => Some(Self::Suspendue),
            209 => Some(Self::Completee),
            210 => Some(Self::Refusee),
            211 => Some(Self::PaiementTransmis),
            212 => Some(Self::Encaissee),
            213 => Some(Self::Rejetee),
            214 => Some(Self::Visee),
            220 => Some(Self::Annulee),
            221 => Some(Self::ErreurRoutage),
            224 => Some(Self::DemandePaiementDirect),
            225 => Some(Self::Affacturee),
            226 => Some(Self::AffactureeConfidentiel),
            227 => Some(Self::ChangementCompteAPayer),
            228 => Some(Self::NonAffacturee),
            501 => Some(Self::Irrecevable),
            _ => None,
        }
    }
}

impl std::fmt::Display for InvoiceStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Code statut de transmission — MDT-88
/// Conforme BR-FR-CDV-CL-05
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TransmissionStatusCode {
    /// Phase Transmission (MDT-77=305)
    Received = 10,
    Forwarded = 51,
    Delivered = 43,
    Rejected = 8,
    Pending = 48,
    /// Phase Traitement (MDT-77=23)
    /// 45 (In Process) = Prise en charge
    InProcess = 45,
    /// 39 (on hold) = Suspendue
    OnHold = 39,
    /// 37 (Complete) = Complétée
    Complete = 37,
    /// 50 (Rejected / Refused) = Refusée (by C4) — BR-FR-CDV-CL-05
    Refused = 50,
    /// 49 (Conditionally accepted) = Approuvée Partiellement
    ConditionallyAccepted = 49,
    /// 47 (Paid) = Paiement Transmis ET Encaissée
    Paid = 47,
    /// 46 (Under Query) = En litige
    UnderQuery = 46,
    /// 1 (accepted) = Approuvée
    Accepted = 1,
}

impl TransmissionStatusCode {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            10 => Some(Self::Received),
            51 => Some(Self::Forwarded),
            43 => Some(Self::Delivered),
            8 => Some(Self::Rejected),
            48 => Some(Self::Pending),
            45 => Some(Self::InProcess),
            39 => Some(Self::OnHold),
            37 => Some(Self::Complete),
            50 => Some(Self::Refused),
            49 => Some(Self::ConditionallyAccepted),
            47 => Some(Self::Paid),
            46 => Some(Self::UnderQuery),
            1 => Some(Self::Accepted),
            _ => None,
        }
    }
}

/// Code rôle d'un acteur — MDT-21, MDT-40, MDT-59
/// Conforme BR-FR-CDV-CL-02/03/04
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RoleCode {
    /// Acheteur
    BY,
    /// Agent de l'acheteur
    AB,
    /// Livré à
    DL,
    /// Vendeur
    SE,
    /// Représentant du vendeur
    SR,
    /// Plateforme (PDP/PPF)
    WK,
    /// Payeur
    PE,
    /// Facturant
    PR,
    /// Émetteur
    II,
    /// Facturé
    IV,
    /// PPF (Direction des Finances Publiques)
    DFH,
}

impl RoleCode {
    pub fn code(&self) -> &str {
        match self {
            Self::BY => "BY",
            Self::AB => "AB",
            Self::DL => "DL",
            Self::SE => "SE",
            Self::SR => "SR",
            Self::WK => "WK",
            Self::PE => "PE",
            Self::PR => "PR",
            Self::II => "II",
            Self::IV => "IV",
            Self::DFH => "DFH",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "BY" => Some(Self::BY),
            "AB" => Some(Self::AB),
            "DL" => Some(Self::DL),
            "SE" => Some(Self::SE),
            "SR" => Some(Self::SR),
            "WK" => Some(Self::WK),
            "PE" => Some(Self::PE),
            "PR" => Some(Self::PR),
            "II" => Some(Self::II),
            "IV" => Some(Self::IV),
            "DFH" => Some(Self::DFH),
            _ => None,
        }
    }
}

impl std::fmt::Display for RoleCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Code motif de statut — MDT-113
/// Conforme BR-FR-CDV-CL-09
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StatusReasonCode {
    NonTransmise,
    JustifAbs,
    RoutageErr,
    Autre,
    CoordBancErr,
    TxTvaErr,
    MontantTotalErr,
    CalculErr,
    NonConforme,
    Doublon,
    DestInc,
    DestErr,
    TransacInc,
    EmmetInc,
    ContratTerm,
    DoubleFact,
    CmdErr,
    AdrErr,
    SiretErr,
    CodeRoutageErr,
    RefCtAbsent,
    RefErr,
    PuErr,
    RemErr,
    QteErr,
    ArtErr,
    ModpaiErr,
    QualiteErr,
    LivrIncomp,
    RejSeman,
    RejUni,
    RejCoh,
    RejAdr,
    RejContB2g,
    RejRefPj,
    RejAssPj,
    IrrVideF,
    IrrTypeF,
    IrrSyntax,
    IrrTaillePj,
    IrrNomPj,
    IrrVidPj,
    IrrExtDoc,
    IrrTailleF,
    IrrAntivirus,
}

impl StatusReasonCode {
    pub fn code(&self) -> &str {
        match self {
            Self::NonTransmise => "NON_TRANSMISE",
            Self::JustifAbs => "JUSTIF_ABS",
            Self::RoutageErr => "ROUTAGE_ERR",
            Self::Autre => "AUTRE",
            Self::CoordBancErr => "COORD_BANC_ERR",
            Self::TxTvaErr => "TX_TVA_ERR",
            Self::MontantTotalErr => "MONTANTTOTAL_ERR",
            Self::CalculErr => "CALCUL_ERR",
            Self::NonConforme => "NON_CONFORME",
            Self::Doublon => "DOUBLON",
            Self::DestInc => "DEST_INC",
            Self::DestErr => "DEST_ERR",
            Self::TransacInc => "TRANSAC_INC",
            Self::EmmetInc => "EMMET_INC",
            Self::ContratTerm => "CONTRAT_TERM",
            Self::DoubleFact => "DOUBLE_FACT",
            Self::CmdErr => "CMD_ERR",
            Self::AdrErr => "ADR_ERR",
            Self::SiretErr => "SIRET_ERR",
            Self::CodeRoutageErr => "CODE_ROUTAGE_ERR",
            Self::RefCtAbsent => "REF_CT_ABSENT",
            Self::RefErr => "REF_ERR",
            Self::PuErr => "PU_ERR",
            Self::RemErr => "REM_ERR",
            Self::QteErr => "QTE_ERR",
            Self::ArtErr => "ART_ERR",
            Self::ModpaiErr => "MODPAI_ERR",
            Self::QualiteErr => "QUALITE_ERR",
            Self::LivrIncomp => "LIVR_INCOMP",
            Self::RejSeman => "REJ_SEMAN",
            Self::RejUni => "REJ_UNI",
            Self::RejCoh => "REJ_COH",
            Self::RejAdr => "REJ_ADR",
            Self::RejContB2g => "REJ_CONT_B2G",
            Self::RejRefPj => "REJ_REF_PJ",
            Self::RejAssPj => "REJ_ASS_PJ",
            Self::IrrVideF => "IRR_VIDE_F",
            Self::IrrTypeF => "IRR_TYPE_F",
            Self::IrrSyntax => "IRR_SYNTAX",
            Self::IrrTaillePj => "IRR_TAILLE_PJ",
            Self::IrrNomPj => "IRR_NOM_PJ",
            Self::IrrVidPj => "IRR_VID_PJ",
            Self::IrrExtDoc => "IRR_EXT_DOC",
            Self::IrrTailleF => "IRR_TAILLE_F",
            Self::IrrAntivirus => "IRR_ANTIVIRUS",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "NON_TRANSMISE" => Some(Self::NonTransmise),
            "JUSTIF_ABS" => Some(Self::JustifAbs),
            "ROUTAGE_ERR" => Some(Self::RoutageErr),
            "AUTRE" => Some(Self::Autre),
            "COORD_BANC_ERR" => Some(Self::CoordBancErr),
            "TX_TVA_ERR" => Some(Self::TxTvaErr),
            "MONTANTTOTAL_ERR" => Some(Self::MontantTotalErr),
            "CALCUL_ERR" => Some(Self::CalculErr),
            "NON_CONFORME" => Some(Self::NonConforme),
            "DOUBLON" => Some(Self::Doublon),
            "DEST_INC" => Some(Self::DestInc),
            "DEST_ERR" => Some(Self::DestErr),
            "TRANSAC_INC" => Some(Self::TransacInc),
            "EMMET_INC" => Some(Self::EmmetInc),
            "CONTRAT_TERM" => Some(Self::ContratTerm),
            "DOUBLE_FACT" => Some(Self::DoubleFact),
            "CMD_ERR" => Some(Self::CmdErr),
            "ADR_ERR" => Some(Self::AdrErr),
            "SIRET_ERR" => Some(Self::SiretErr),
            "CODE_ROUTAGE_ERR" => Some(Self::CodeRoutageErr),
            "REF_CT_ABSENT" => Some(Self::RefCtAbsent),
            "REF_ERR" => Some(Self::RefErr),
            "PU_ERR" => Some(Self::PuErr),
            "REM_ERR" => Some(Self::RemErr),
            "QTE_ERR" => Some(Self::QteErr),
            "ART_ERR" => Some(Self::ArtErr),
            "MODPAI_ERR" => Some(Self::ModpaiErr),
            "QUALITE_ERR" => Some(Self::QualiteErr),
            "LIVR_INCOMP" => Some(Self::LivrIncomp),
            "REJ_SEMAN" => Some(Self::RejSeman),
            "REJ_UNI" => Some(Self::RejUni),
            "REJ_COH" => Some(Self::RejCoh),
            "REJ_ADR" => Some(Self::RejAdr),
            "REJ_CONT_B2G" => Some(Self::RejContB2g),
            "REJ_REF_PJ" => Some(Self::RejRefPj),
            "REJ_ASS_PJ" => Some(Self::RejAssPj),
            "IRR_VIDE_F" => Some(Self::IrrVideF),
            "IRR_TYPE_F" => Some(Self::IrrTypeF),
            "IRR_SYNTAX" => Some(Self::IrrSyntax),
            "IRR_TAILLE_PJ" => Some(Self::IrrTaillePj),
            "IRR_NOM_PJ" => Some(Self::IrrNomPj),
            "IRR_VID_PJ" => Some(Self::IrrVidPj),
            "IRR_EXT_DOC" => Some(Self::IrrExtDoc),
            "IRR_TAILLE_F" => Some(Self::IrrTailleF),
            "IRR_ANTIVIRUS" => Some(Self::IrrAntivirus),
            _ => None,
        }
    }
}

impl std::fmt::Display for StatusReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Code action attendue — MDT-121
/// Conforme BR-FR-CDV-CL-10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionCode {
    /// Aucune action requise
    NOA,
    /// Créer une facture rectificative (prix)
    PIN,
    /// Créer une facture rectificative (autre)
    NIN,
    /// Confirmer
    CNF,
    /// Confirmer partiellement
    CNP,
    /// Annuler
    CNA,
    /// Autre
    OTH,
}

impl ActionCode {
    pub fn code(&self) -> &str {
        match self {
            Self::NOA => "NOA",
            Self::PIN => "PIN",
            Self::NIN => "NIN",
            Self::CNF => "CNF",
            Self::CNP => "CNP",
            Self::CNA => "CNA",
            Self::OTH => "OTH",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "NOA" => Some(Self::NOA),
            "PIN" => Some(Self::PIN),
            "NIN" => Some(Self::NIN),
            "CNF" => Some(Self::CNF),
            "CNP" => Some(Self::CNP),
            "CNA" => Some(Self::CNA),
            "OTH" => Some(Self::OTH),
            _ => None,
        }
    }
}

impl std::fmt::Display for ActionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Code type de caractéristique — MDT-207
/// Conforme BR-FR-CDV-CL-11
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CharacteristicTypeCode {
    MEN, MPA, RAP, ESC, RAB, REM, MAP, MAPTTC, MNA, MNATTC, CBB, DIV, DVA, MAJ,
}

impl CharacteristicTypeCode {
    pub fn code(&self) -> &str {
        match self {
            Self::MEN => "MEN", Self::MPA => "MPA", Self::RAP => "RAP",
            Self::ESC => "ESC", Self::RAB => "RAB", Self::REM => "REM",
            Self::MAP => "MAP", Self::MAPTTC => "MAPTTC", Self::MNA => "MNA",
            Self::MNATTC => "MNATTC", Self::CBB => "CBB", Self::DIV => "DIV",
            Self::DVA => "DVA", Self::MAJ => "MAJ",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "MEN" => Some(Self::MEN), "MPA" => Some(Self::MPA), "RAP" => Some(Self::RAP),
            "ESC" => Some(Self::ESC), "RAB" => Some(Self::RAB), "REM" => Some(Self::REM),
            "MAP" => Some(Self::MAP), "MAPTTC" => Some(Self::MAPTTC), "MNA" => Some(Self::MNA),
            "MNATTC" => Some(Self::MNATTC), "CBB" => Some(Self::CBB), "DIV" => Some(Self::DIV),
            "DVA" => Some(Self::DVA), "MAJ" => Some(Self::MAJ),
            _ => None,
        }
    }
}

/// Cadre de facturation — MDT-2
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BusinessProcessCode {
    Regulated,
    NonRegulated,
    B2C,
    B2BInt,
    OutOfScope,
}

impl BusinessProcessCode {
    pub fn code(&self) -> &str {
        match self {
            Self::Regulated => "REGULATED",
            Self::NonRegulated => "NON_REGULATED",
            Self::B2C => "B2C",
            Self::B2BInt => "B2BINT",
            Self::OutOfScope => "OUTOFSCOPE",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "REGULATED" => Some(Self::Regulated),
            "NON_REGULATED" => Some(Self::NonRegulated),
            "B2C" => Some(Self::B2C),
            "B2BINT" => Some(Self::B2BInt),
            "OUTOFSCOPE" => Some(Self::OutOfScope),
            _ => None,
        }
    }
}

impl std::fmt::Display for BusinessProcessCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Acteur commercial (SenderTradeParty, IssuerTradeParty, RecipientTradeParty)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeParty {
    /// MDT-57/MDT-38 : Identifiant global (SIREN, etc.)
    pub global_id: Option<String>,
    /// Attribut schemeID de l'identifiant global (0002=SIREN, 0238=PPF)
    pub global_id_scheme: Option<String>,
    /// MDT-58/MDT-39 : Raison sociale
    pub name: Option<String>,
    /// MDT-21/MDT-40/MDT-59 : Code rôle
    pub role_code: RoleCode,
    /// MDT-73 : Adresse électronique (réseau CEF)
    pub endpoint_id: Option<String>,
    /// Attribut schemeID de l'adresse électronique
    pub endpoint_scheme: Option<String>,
}

/// Caractéristique d'un document (SpecifiedDocumentCharacteristic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCharacteristic {
    /// MDT-206 : ID de la donnée du document
    pub id: Option<String>,
    /// MDT-207 : Code du type de donnée
    pub type_code: String,
    /// MDT-209 : Indicateur de changement
    pub value_changed: Option<bool>,
    /// MDT-211 : Nom de la donnée
    pub name: Option<String>,
    /// MDT-213 : Localisation XPath dans l'XML
    pub location: Option<String>,
    /// MDT-224 : Pourcentage
    pub value_percent: Option<String>,
    /// MDT-208 : Montant
    pub value_amount: Option<String>,
}

/// Statut de document avec motif et action (SpecifiedDocumentStatus)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStatus {
    /// MDT-115 : Code statut (optionnel, sous-statut)
    pub status_code: Option<u16>,
    /// MDT-113 : Code motif
    pub reason_code: Option<String>,
    /// MDT-114 : Libellé motif
    pub reason: Option<String>,
    /// MDT-121 : Code action attendue
    pub action_code: Option<String>,
    /// MDT-122 : Libellé action attendue
    pub action: Option<String>,
    /// MDT-124-2 : Numéro incrémental
    pub sequence: Option<u32>,
    /// Caractéristiques associées
    pub characteristics: Vec<DocumentCharacteristic>,
}

/// Document référencé (ReferenceReferencedDocument) — la facture concernée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencedDocument {
    /// MDT-87 : ID objet (BT-1 de la facture)
    pub invoice_id: String,
    /// MDT-88 : Code statut de transmission
    pub status_code: Option<u16>,
    /// MDT-91 : Code type de l'objet (BT-3 de la facture)
    pub type_code: Option<String>,
    /// MDT-95 : Date-heure de réception
    pub receipt_datetime: Option<String>,
    /// MDT-100 : Date de facture (BT-2), format 102 (YYYYMMDD)
    pub issue_date: Option<String>,
    /// MDT-105 : Code statut traitement
    pub process_condition_code: u16,
    /// MDT-106 : Libellé statut traitement
    pub process_condition: Option<String>,
    /// MDT-129 : Émetteur du document référencé
    pub issuer: Option<TradeParty>,
    /// MDT-158 : Destinataire du document référencé
    pub recipient: Option<TradeParty>,
    /// Statuts détaillés avec motifs
    pub statuses: Vec<DocumentStatus>,
}

/// CDV (Compte-rendu De Vie) — CrossDomainAcknowledgementAndResponse
/// Conforme au schéma XSD D22B et aux Schematrons BR-FR-CDV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdvResponse {
    // --- ExchangedDocumentContext ---
    /// MDT-2 : Cadre de facturation
    pub business_process: String,
    /// MDT-3 : Type de profil
    pub guideline_id: String,

    // --- ExchangedDocument ---
    /// MDT-4 : Identifiant du document CDV
    pub document_id: String,
    /// MDT-5 : Nom du document
    pub document_name: Option<String>,
    /// MDT-8 : Date-heure de création du CDV (format 204: YYYYMMDDHHmmss)
    pub issue_datetime: String,
    /// MDT-21 : Émetteur (SenderTradeParty)
    pub sender: TradeParty,
    /// MDT-40 : Auteur (IssuerTradeParty)
    pub issuer: Option<TradeParty>,
    /// Destinataires (RecipientTradeParty) — peut être multiple
    pub recipients: Vec<TradeParty>,

    // --- AcknowledgementDocument ---
    /// MDT-74 : Indicateur mono/multi document
    pub multiple_references: bool,
    /// MDT-77 : Code type document (23=Traitement, 305=Transmission)
    pub type_code: CdvTypeCode,
    /// MDT-78 : Date-heure de dépôt du statut CDV
    pub status_datetime: String,
    /// Documents référencés (factures concernées)
    pub referenced_documents: Vec<ReferencedDocument>,
}

impl CdvResponse {
    pub fn is_success(&self) -> bool {
        self.referenced_documents.iter().all(|doc| {
            let code = doc.process_condition_code;
            code >= 200 && code <= 228 && code != 210 && code != 213 && code != 220 && code != 221
        })
    }

    pub fn is_rejected(&self) -> bool {
        self.referenced_documents.iter().any(|doc| {
            doc.process_condition_code == 213
        })
    }

    pub fn is_annulee(&self) -> bool {
        self.referenced_documents.iter().any(|doc| {
            doc.process_condition_code == 220
        })
    }

    pub fn is_irrecevable(&self) -> bool {
        self.referenced_documents.iter().any(|doc| {
            doc.process_condition_code == 501
        })
    }

    pub fn has_reason_codes(&self) -> bool {
        self.referenced_documents.iter().any(|doc| {
            doc.statuses.iter().any(|s| s.reason_code.is_some())
        })
    }

    pub fn status_code(&self) -> Option<u16> {
        self.referenced_documents.first().map(|d| d.process_condition_code)
    }

    pub fn status_label(&self) -> Option<&'static str> {
        self.referenced_documents.first()
            .and_then(|d| InvoiceStatusCode::from_code(d.process_condition_code))
            .map(|s| s.label())
    }
}

// --- Aliases de compatibilité pour le processor ---

/// Alias pour CdvResponse (ancien nom)
pub type CdarResponse = CdvResponse;

/// Erreur de validation pour le processor (simplifié)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdarValidationError {
    pub rule_id: String,
    pub severity: String,
    pub location: Option<String>,
    pub message: String,
    pub reason_code: Option<StatusReasonCode>,
}
