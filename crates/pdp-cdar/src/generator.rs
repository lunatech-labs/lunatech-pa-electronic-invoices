use chrono::Utc;
use pdp_core::error::PdpResult;
use pdp_core::model::InvoiceData;

use crate::model::*;

/// Identifiant standard du PPF (Portail Public de Facturation)
const PPF_GLOBAL_ID: &str = "9998";
/// Scheme ID de l'identifiant PPF (CEF eDelivery)
const PPF_SCHEME_ID: &str = "0238";

/// Générateur de CDV (Compte-rendu De Vie) conformes au format
/// CrossDomainAcknowledgementAndResponse D22B
pub struct CdarGenerator {
    /// SIREN de la PDP émettrice
    pub pdp_siren: String,
    /// Nom de la PDP émettrice
    pub pdp_name: String,
}

impl CdarGenerator {
    pub fn new(pdp_siren: &str, pdp_name: &str) -> Self {
        Self {
            pdp_siren: pdp_siren.to_string(),
            pdp_name: pdp_name.to_string(),
        }
    }

    /// Construit le TradeParty destinataire PPF (rôle DFH)
    fn ppf_recipient() -> TradeParty {
        TradeParty {
            global_id: Some(PPF_GLOBAL_ID.to_string()),
            global_id_scheme: Some(PPF_SCHEME_ID.to_string()),
            name: Some("PPF".to_string()),
            role_code: RoleCode::DFH,
            endpoint_id: None,
            endpoint_scheme: None,
        }
    }

    /// Formate un DateTime en format 204 (YYYYMMDDHHmmss)
    fn format_datetime_204(dt: &chrono::DateTime<Utc>) -> String {
        dt.format("%Y%m%d%H%M%S").to_string()
    }

    /// Formate une date ISO (YYYY-MM-DD) en format 102 (YYYYMMDD)
    fn format_date_102(iso_date: &str) -> String {
        iso_date.replace('-', "")
    }

    /// Génère un CDV de dépôt (statut 200 — Déposée, phase Transmission)
    pub fn generate_deposee(
        &self,
        invoice: &InvoiceData,
        invoice_type_code: &str,
    ) -> CdvResponse {
        let now = Utc::now();
        let dt = Self::format_datetime_204(&now);
        let issue_date_102 = invoice.issue_date.as_deref()
            .map(|d| Self::format_date_102(d))
            .unwrap_or_default();

        let seller_siren = invoice.seller_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");

        CdvResponse {
            business_process: "REGULATED".to_string(),
            guideline_id: "urn.cpro.gouv.fr:1p0:CDV:invoice".to_string(),
            document_id: format!("{}_{}_{}#{}_{}",
                invoice.invoice_number, 200, &dt, invoice_type_code, &issue_date_102),
            document_name: Some(format!("CDV-200_Deposee_{}", invoice.invoice_number)),
            issue_datetime: dt.clone(),
            sender: TradeParty {
                global_id: None,
                global_id_scheme: None,
                name: None,
                role_code: RoleCode::WK,
                endpoint_id: None,
                endpoint_scheme: None,
            },
            issuer: Some(TradeParty {
                global_id: None,
                global_id_scheme: None,
                name: None,
                role_code: RoleCode::WK,
                endpoint_id: None,
                endpoint_scheme: None,
            }),
            recipients: vec![
                TradeParty {
                    global_id: Some(seller_siren.to_string()),
                    global_id_scheme: Some("0002".to_string()),
                    name: invoice.seller_name.clone(),
                    role_code: RoleCode::SE,
                    endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                    endpoint_scheme: Some("0225".to_string()),
                },
                Self::ppf_recipient(),
            ],
            multiple_references: false,
            type_code: CdvTypeCode::Transmission,
            status_datetime: dt.clone(),
            referenced_documents: vec![
                ReferencedDocument {
                    invoice_id: invoice.invoice_number.clone(),
                    status_code: Some(10),
                    type_code: Some(invoice_type_code.to_string()),
                    receipt_datetime: Some(dt.clone()),
                    issue_date: Some(issue_date_102),
                    process_condition_code: 200,
                    process_condition: Some("Déposée".to_string()),
                    issuer: Some(TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: None,
                        role_code: RoleCode::SE,
                        endpoint_id: None,
                        endpoint_scheme: None,
                    }),
                    recipient: None,
                    statuses: Vec::new(),
                },
            ],
        }
    }

    /// Génère un CDV de réception (statut 202 — Reçue, phase Transmission)
    /// Utilisé par la PDP réceptrice quand elle reçoit une facture d'une autre PDP.
    ///
    /// Conforme à la fixture AFNOR UC1_F202500003_02-CDV-202_Recue.xml :
    /// - Recipients : Vendeur (SE) + Acheteur (BY), PAS de PPF
    /// - Referenced Doc Issuer : Vendeur (SE) — c'est lui qui a émis la facture
    pub fn generate_recue(
        &self,
        invoice: &InvoiceData,
        invoice_type_code: &str,
    ) -> CdvResponse {
        let now = Utc::now();
        let dt = Self::format_datetime_204(&now);
        let issue_date_102 = invoice.issue_date.as_deref()
            .map(|d| Self::format_date_102(d))
            .unwrap_or_default();

        let seller_siren = invoice.seller_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");

        let buyer_siren = invoice.buyer_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");

        CdvResponse {
            business_process: "REGULATED".to_string(),
            guideline_id: "urn.cpro.gouv.fr:1p0:CDV:invoice".to_string(),
            document_id: format!("{}_{}_{}#{}_{}",
                invoice.invoice_number, 202, &dt, invoice_type_code, &issue_date_102),
            document_name: Some(format!("CDV-202_Recue_{}", invoice.invoice_number)),
            issue_datetime: dt.clone(),
            sender: TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: RoleCode::WK, endpoint_id: None, endpoint_scheme: None,
            },
            issuer: Some(TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: RoleCode::WK, endpoint_id: None, endpoint_scheme: None,
            }),
            // Destinataires conformes AFNOR : Vendeur (SE) + Acheteur (BY)
            // PAS de PPF — contrairement au CDV 200 (émission)
            recipients: vec![
                TradeParty {
                    global_id: Some(seller_siren.to_string()),
                    global_id_scheme: Some("0002".to_string()),
                    name: invoice.seller_name.clone(),
                    role_code: RoleCode::SE,
                    endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                    endpoint_scheme: Some("0225".to_string()),
                },
                TradeParty {
                    global_id: Some(buyer_siren.to_string()),
                    global_id_scheme: Some("0002".to_string()),
                    name: invoice.buyer_name.clone(),
                    role_code: RoleCode::BY,
                    endpoint_id: Some(buyer_siren.to_string()),
                    endpoint_scheme: None,
                },
            ],
            multiple_references: false,
            type_code: CdvTypeCode::Transmission,
            status_datetime: dt.clone(),
            referenced_documents: vec![
                ReferencedDocument {
                    invoice_id: invoice.invoice_number.clone(),
                    status_code: Some(43), // Transferred to the next party
                    type_code: Some(invoice_type_code.to_string()),
                    receipt_datetime: Some(dt.clone()),
                    issue_date: Some(issue_date_102),
                    process_condition_code: 202,
                    process_condition: Some("Reçue".to_string()),
                    // Issuer = le VENDEUR (SE) — c'est lui qui a émis la facture
                    issuer: Some(TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: None,
                        role_code: RoleCode::SE,
                        endpoint_id: None,
                        endpoint_scheme: None,
                    }),
                    recipient: None,
                    statuses: Vec::new(),
                },
            ],
        }
    }

    /// Génère un CDV de rejet (statut 213 — Rejetée, phase Transmission)
    pub fn generate_rejetee(
        &self,
        invoice: &InvoiceData,
        invoice_type_code: &str,
        errors: Vec<CdarValidationError>,
    ) -> CdvResponse {
        let now = Utc::now();
        let dt = Self::format_datetime_204(&now);
        let issue_date_102 = invoice.issue_date.as_deref()
            .map(|d| Self::format_date_102(d))
            .unwrap_or_default();

        let seller_siren = invoice.seller_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");
        let buyer_siren = invoice.buyer_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s });

        let statuses: Vec<DocumentStatus> = errors.iter().enumerate().map(|(i, err)| {
            let reason_code = err.reason_code.as_ref()
                .map(|r| r.code().to_string())
                .unwrap_or_else(|| "REJ_SEMAN".to_string());

            DocumentStatus {
                status_code: None,
                reason_code: Some(reason_code),
                reason: Some(err.message.clone()),
                action_code: Some("NIN".to_string()),
                action: Some("Corriger et redéposer".to_string()),
                sequence: Some((i + 1) as u32),
                characteristics: Vec::new(),
            }
        }).collect();

        CdvResponse {
            business_process: "REGULATED".to_string(),
            guideline_id: "urn.cpro.gouv.fr:1p0:CDV:invoice".to_string(),
            document_id: format!("{}_{}_{}#{}_{}",
                invoice.invoice_number, 213, &dt, invoice_type_code, &issue_date_102),
            document_name: Some(format!("CDV-213_Rejetee_{}", invoice.invoice_number)),
            issue_datetime: dt.clone(),
            sender: TradeParty {
                global_id: None,
                global_id_scheme: None,
                name: None,
                role_code: RoleCode::WK,
                endpoint_id: None,
                endpoint_scheme: None,
            },
            issuer: Some(TradeParty {
                global_id: None,
                global_id_scheme: None,
                name: None,
                role_code: RoleCode::WK,
                endpoint_id: None,
                endpoint_scheme: None,
            }),
            recipients: {
                let mut r = vec![
                    TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.seller_name.clone(),
                        role_code: RoleCode::SE,
                        endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                        endpoint_scheme: Some("0225".to_string()),
                    },
                ];
                if let Some(bs) = buyer_siren {
                    r.push(TradeParty {
                        global_id: Some(bs.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.buyer_name.clone(),
                        role_code: RoleCode::BY,
                        endpoint_id: Some(bs.to_string()),
                        endpoint_scheme: None,
                    });
                }
                r.push(Self::ppf_recipient());
                r
            },
            multiple_references: false,
            type_code: CdvTypeCode::Transmission,
            status_datetime: dt.clone(),
            referenced_documents: vec![
                ReferencedDocument {
                    invoice_id: invoice.invoice_number.clone(),
                    status_code: Some(8),
                    type_code: Some(invoice_type_code.to_string()),
                    receipt_datetime: Some(dt.clone()),
                    issue_date: Some(issue_date_102),
                    process_condition_code: 213,
                    process_condition: Some("Rejetée".to_string()),
                    issuer: Some(TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: None,
                        role_code: RoleCode::SE,
                        endpoint_id: None,
                        endpoint_scheme: None,
                    }),
                    recipient: None,
                    statuses,
                },
            ],
        }
    }

    /// Génère un CDV d'irrecevabilité (statut 501, phase Transmission)
    pub fn generate_irrecevable(
        &self,
        invoice: &InvoiceData,
        reason: StatusReasonCode,
        message: &str,
    ) -> CdvResponse {
        let now = Utc::now();
        let dt = Self::format_datetime_204(&now);

        let seller_siren = invoice.seller_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");

        CdvResponse {
            business_process: "REGULATED".to_string(),
            guideline_id: "urn.cpro.gouv.fr:1p0:CDV:invoice".to_string(),
            document_id: format!("{}_501_{}", invoice.invoice_number, &dt),
            document_name: Some(format!("CDV-501_Irrecevable_{}", invoice.invoice_number)),
            issue_datetime: dt.clone(),
            sender: TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: RoleCode::WK, endpoint_id: None, endpoint_scheme: None,
            },
            issuer: Some(TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: RoleCode::WK, endpoint_id: None, endpoint_scheme: None,
            }),
            recipients: vec![
                TradeParty {
                    global_id: Some(seller_siren.to_string()),
                    global_id_scheme: Some("0002".to_string()),
                    name: invoice.seller_name.clone(),
                    role_code: RoleCode::SE,
                    endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                    endpoint_scheme: Some("0225".to_string()),
                },
                Self::ppf_recipient(),
            ],
            multiple_references: false,
            type_code: CdvTypeCode::Transmission,
            status_datetime: dt.clone(),
            referenced_documents: vec![
                ReferencedDocument {
                    invoice_id: invoice.invoice_number.clone(),
                    status_code: Some(8),
                    type_code: None,
                    receipt_datetime: Some(dt.clone()),
                    issue_date: None,
                    process_condition_code: 501,
                    process_condition: Some("Irrecevable".to_string()),
                    issuer: None,
                    recipient: None,
                    statuses: vec![DocumentStatus {
                        status_code: None,
                        reason_code: Some(reason.code().to_string()),
                        reason: Some(message.to_string()),
                        action_code: None,
                        action: None,
                        sequence: Some(1),
                        characteristics: Vec::new(),
                    }],
                },
            ],
        }
    }

    /// Génère un CDV générique pour n'importe quel statut.
    /// `status` : code statut (InvoiceStatusCode)
    /// `type_code` : Transmission (305) ou Traitement (23)
    /// `sender_role` : rôle de l'émetteur du CDV
    /// `invoice` : données de la facture concernée
    /// `invoice_type_code` : code type de la facture (380, 381, etc.)
    /// `statuses` : motifs de statut optionnels (pour rejets, litiges, etc.)
    /// `characteristics` : caractéristiques optionnelles (pour encaissée MEN, etc.)
    pub fn generate_status(
        &self,
        status: InvoiceStatusCode,
        type_code: CdvTypeCode,
        sender_role: RoleCode,
        invoice: &InvoiceData,
        invoice_type_code: &str,
        statuses: Vec<DocumentStatus>,
    ) -> CdvResponse {
        let now = Utc::now();
        let dt = Self::format_datetime_204(&now);
        let issue_date_102 = invoice.issue_date.as_deref()
            .map(|d| Self::format_date_102(d))
            .unwrap_or_default();

        let seller_siren = invoice.seller_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");
        let buyer_siren = invoice.buyer_siret.as_deref()
            .map(|s| if s.len() >= 9 { &s[..9] } else { s })
            .unwrap_or("000000000");

        let status_code_num = status.code();
        let label = status.label();

        // Déterminer le StatusCode (MDT-88) selon BR-FR-CDV-CL-05
        // Valeurs conformes aux exemples XP Z12-012 Annexe B V1.3
        let mdt88 = match status {
            InvoiceStatusCode::Deposee => Some(10),             // Received
            InvoiceStatusCode::Emise => Some(10),               // Received
            InvoiceStatusCode::Recue => Some(43),               // Transferred to the next party
            InvoiceStatusCode::MiseADisposition => Some(48),    // Available
            InvoiceStatusCode::PriseEnCharge => Some(45),       // Under investigation
            InvoiceStatusCode::Approuvee => Some(1),            // Accepted
            InvoiceStatusCode::ApprouveePartiellement => Some(1), // Accepted
            InvoiceStatusCode::EnLitige => Some(46),            // Under query
            InvoiceStatusCode::Suspendue => Some(46),           // Under query
            InvoiceStatusCode::Completee => Some(45),           // Under investigation
            InvoiceStatusCode::Refusee => Some(8),              // Rejected
            InvoiceStatusCode::PaiementTransmis => Some(47),    // Pending
            InvoiceStatusCode::Encaissee => Some(47),           // Pending
            InvoiceStatusCode::Rejetee => Some(8),              // Rejected
            InvoiceStatusCode::Visee => Some(1),                // Accepted
            InvoiceStatusCode::Annulee => Some(8),              // Rejected
            InvoiceStatusCode::ErreurRoutage => Some(8),        // Rejected
            InvoiceStatusCode::DemandePaiementDirect => Some(47), // Pending
            InvoiceStatusCode::Affacturee => Some(47),          // Pending
            InvoiceStatusCode::AffactureeConfidentiel => Some(47), // Pending
            InvoiceStatusCode::ChangementCompteAPayer => Some(47), // Pending
            InvoiceStatusCode::NonAffacturee => Some(47),       // Pending
            InvoiceStatusCode::Irrecevable => Some(8),          // Rejected
        };

        // Destinataires selon le statut et la phase
        // Conforme aux exemples XP Z12-012 Annexe B V1.3 :
        // - Transmission 200/201 : SE + DFH
        // - Transmission 202/203 : SE + BY (PA-R notifie les deux parties)
        // - Traitement 204-211 : SE seul (l'initiateur sait déjà)
        // - Traitement 212 (Encaissée) : BY + DFH (vendeur est Issuer, pas destinataire)
        // - Transmission 213/221/501 : utiliser generate_rejetee/irrecevable
        let recipients = match (type_code, status) {
            // Transmission : Reçue / MAD → vendeur + acheteur
            (CdvTypeCode::Transmission, InvoiceStatusCode::Recue)
            | (CdvTypeCode::Transmission, InvoiceStatusCode::MiseADisposition) => {
                vec![
                    TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.seller_name.clone(),
                        role_code: RoleCode::SE,
                        endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                        endpoint_scheme: Some("0225".to_string()),
                    },
                    TradeParty {
                        global_id: Some(buyer_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.buyer_name.clone(),
                        role_code: RoleCode::BY,
                        endpoint_id: Some(buyer_siren.to_string()),
                        endpoint_scheme: None,
                    },
                ]
            }
            // Traitement : Encaissée → acheteur + PPF (vendeur est Issuer)
            (CdvTypeCode::Traitement, InvoiceStatusCode::Encaissee) => {
                vec![
                    TradeParty {
                        global_id: Some(buyer_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.buyer_name.clone(),
                        role_code: RoleCode::BY,
                        endpoint_id: Some(buyer_siren.to_string()),
                        endpoint_scheme: Some("0225".to_string()),
                    },
                    Self::ppf_recipient(),
                ]
            }
            // Transmission par défaut (200, 201, etc.) : vendeur + PPF
            (CdvTypeCode::Transmission, _) => {
                vec![
                    TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.seller_name.clone(),
                        role_code: RoleCode::SE,
                        endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                        endpoint_scheme: Some("0225".to_string()),
                    },
                    Self::ppf_recipient(),
                ]
            }
            // Traitement par défaut (204-211 sauf 212) : vendeur seul
            (CdvTypeCode::Traitement, _) => {
                vec![
                    TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: invoice.seller_name.clone(),
                        role_code: RoleCode::SE,
                        endpoint_id: Some(format!("{}_STATUTS", seller_siren)),
                        endpoint_scheme: Some("0225".to_string()),
                    },
                ]
            }
        };

        CdvResponse {
            business_process: "REGULATED".to_string(),
            guideline_id: "urn.cpro.gouv.fr:1p0:CDV:invoice".to_string(),
            document_id: format!("{}_{}_{}#{}_{}",
                invoice.invoice_number, status_code_num, &dt, invoice_type_code, &issue_date_102),
            document_name: Some(format!("CDV-{}_{}_{}",
                status_code_num, label, invoice.invoice_number)),
            issue_datetime: dt.clone(),
            sender: TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: sender_role,
                endpoint_id: None, endpoint_scheme: None,
            },
            issuer: Some(TradeParty {
                global_id: None, global_id_scheme: None, name: None,
                role_code: sender_role,
                endpoint_id: None, endpoint_scheme: None,
            }),
            recipients,
            multiple_references: false,
            type_code,
            status_datetime: dt.clone(),
            referenced_documents: vec![
                ReferencedDocument {
                    invoice_id: invoice.invoice_number.clone(),
                    status_code: mdt88,
                    type_code: Some(invoice_type_code.to_string()),
                    receipt_datetime: Some(dt.clone()),
                    issue_date: Some(issue_date_102),
                    process_condition_code: status_code_num,
                    process_condition: Some(label.to_string()),
                    issuer: Some(TradeParty {
                        global_id: Some(seller_siren.to_string()),
                        global_id_scheme: Some("0002".to_string()),
                        name: None,
                        role_code: RoleCode::SE,
                        endpoint_id: None,
                        endpoint_scheme: None,
                    }),
                    recipient: None,
                    statuses,
                },
            ],
        }
    }

    // --- Méthodes de commodité pour les statuts de phase Transmission ---

    /// Génère un CDV Émise (201)
    pub fn generate_emise(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::Emise, CdvTypeCode::Transmission,
            RoleCode::WK, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Mise à disposition (203)
    pub fn generate_mise_a_disposition(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::MiseADisposition, CdvTypeCode::Transmission,
            RoleCode::WK, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Erreur routage (221)
    pub fn generate_erreur_routage(&self, invoice: &InvoiceData, message: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::ErreurRoutage, CdvTypeCode::Transmission,
            RoleCode::WK, invoice, "380", vec![DocumentStatus {
                status_code: None,
                reason_code: Some(StatusReasonCode::RoutageErr.code().to_string()),
                reason: Some(message.to_string()),
                action_code: None, action: None, sequence: Some(1),
                characteristics: Vec::new(),
            }])
    }

    // --- Méthodes de commodité pour les statuts de phase Traitement ---

    /// Génère un CDV Prise en charge (204)
    pub fn generate_prise_en_charge(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::PriseEnCharge, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Approuvée partiellement (206)
    pub fn generate_approuvee_partiellement(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::ApprouveePartiellement, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Suspendue (208)
    pub fn generate_suspendue(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::Suspendue, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Complétée (209)
    pub fn generate_completee(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::Completee, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Refusée (210) avec motifs
    pub fn generate_refusee(&self, invoice: &InvoiceData, invoice_type_code: &str,
        reason_code: StatusReasonCode, message: &str) -> CdvResponse
    {
        self.generate_status(InvoiceStatusCode::Refusee, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, vec![DocumentStatus {
                status_code: None,
                reason_code: Some(reason_code.code().to_string()),
                reason: Some(message.to_string()),
                action_code: Some("NIN".to_string()),
                action: Some("Corriger et redéposer".to_string()),
                sequence: Some(1),
                characteristics: Vec::new(),
            }])
    }

    /// Génère un CDV Paiement transmis (211)
    pub fn generate_paiement_transmis(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::PaiementTransmis, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Encaissée (212) avec montant encaissé (MEN) — BR-FR-CDV-14
    pub fn generate_encaissee(&self, invoice: &InvoiceData, invoice_type_code: &str,
        montant_encaisse: f64) -> CdvResponse
    {
        self.generate_status(InvoiceStatusCode::Encaissee, CdvTypeCode::Traitement,
            RoleCode::SE, invoice, invoice_type_code, vec![DocumentStatus {
                status_code: None,
                reason_code: None,
                reason: None,
                action_code: None,
                action: None,
                sequence: Some(1),
                characteristics: vec![DocumentCharacteristic {
                    id: None,
                    type_code: "MEN".to_string(),
                    value_changed: None,
                    name: Some("Montant encaissé".to_string()),
                    location: None,
                    value_percent: None,
                    value_amount: Some(format!("{:.2}", montant_encaisse)),
                }],
            }])
    }

    /// Génère un CDV Visée (214)
    pub fn generate_visee(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::Visee, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Annulée (220)
    pub fn generate_annulee(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::Annulee, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Demande de Paiement Direct (224)
    pub fn generate_demande_paiement_direct(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::DemandePaiementDirect, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Affacturée (225)
    pub fn generate_affacturee(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::Affacturee, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Affacturée Confidentiel (226)
    pub fn generate_affacturee_confidentiel(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::AffactureeConfidentiel, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Changement de Compte à Payer (227)
    pub fn generate_changement_compte_a_payer(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::ChangementCompteAPayer, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Génère un CDV Non Affacturée (228)
    pub fn generate_non_affacturee(&self, invoice: &InvoiceData, invoice_type_code: &str) -> CdvResponse {
        self.generate_status(InvoiceStatusCode::NonAffacturee, CdvTypeCode::Traitement,
            RoleCode::BY, invoice, invoice_type_code, Vec::new())
    }

    /// Sérialise un CDV en XML conforme CrossDomainAcknowledgementAndResponse D22B
    pub fn to_xml(&self, cdv: &CdvResponse) -> PdpResult<String> {
        let mut xml = String::with_capacity(4096);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossDomainAcknowledgementAndResponse
 xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"
 xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100"
 xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
 xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossDomainAcknowledgementAndResponse:100"
 xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#);

        // ExchangedDocumentContext
        xml.push_str(&format!(r#"
     <rsm:ExchangedDocumentContext>
          <ram:BusinessProcessSpecifiedDocumentContextParameter>
               <ram:ID>{}</ram:ID>
          </ram:BusinessProcessSpecifiedDocumentContextParameter>
          <ram:GuidelineSpecifiedDocumentContextParameter>
               <ram:ID>{}</ram:ID>
          </ram:GuidelineSpecifiedDocumentContextParameter>
     </rsm:ExchangedDocumentContext>"#,
            xml_escape(&cdv.business_process),
            xml_escape(&cdv.guideline_id),
        ));

        // ExchangedDocument
        xml.push_str(&format!(r#"
     <rsm:ExchangedDocument>
          <ram:ID>{}</ram:ID>"#, xml_escape(&cdv.document_id)));

        if let Some(name) = &cdv.document_name {
            xml.push_str(&format!(r#"
          <ram:Name>{}</ram:Name>"#, xml_escape(name)));
        }

        xml.push_str(&format!(r#"
          <ram:IssueDateTime>
               <udt:DateTimeString format="204">{}</udt:DateTimeString>
          </ram:IssueDateTime>"#, &cdv.issue_datetime));

        // SenderTradeParty
        self.write_trade_party(&mut xml, "SenderTradeParty", &cdv.sender);

        // IssuerTradeParty
        if let Some(issuer) = &cdv.issuer {
            self.write_trade_party(&mut xml, "IssuerTradeParty", issuer);
        }

        // RecipientTradeParty(s)
        for recipient in &cdv.recipients {
            self.write_trade_party(&mut xml, "RecipientTradeParty", recipient);
        }

        xml.push_str(r#"
     </rsm:ExchangedDocument>"#);

        // AcknowledgementDocument
        xml.push_str(&format!(r#"
     <rsm:AcknowledgementDocument>
          <ram:MultipleReferencesIndicator>
               <udt:Indicator>{}</udt:Indicator>
          </ram:MultipleReferencesIndicator>
          <ram:TypeCode>{}</ram:TypeCode>
          <ram:IssueDateTime>
               <udt:DateTimeString format="204">{}</udt:DateTimeString>
          </ram:IssueDateTime>"#,
            if cdv.multiple_references { "true" } else { "false" },
            cdv.type_code.code(),
            &cdv.status_datetime,
        ));

        // ReferenceReferencedDocument(s)
        for doc in &cdv.referenced_documents {
            self.write_referenced_document(&mut xml, doc);
        }

        xml.push_str(r#"
     </rsm:AcknowledgementDocument>"#);

        xml.push_str(r#"
</rsm:CrossDomainAcknowledgementAndResponse>"#);

        Ok(xml)
    }

    fn write_trade_party(&self, xml: &mut String, tag: &str, party: &TradeParty) {
        xml.push_str(&format!(r#"
          <ram:{}>"#, tag));

        if let (Some(id), Some(scheme)) = (&party.global_id, &party.global_id_scheme) {
            xml.push_str(&format!(r#"
               <ram:GlobalID schemeID="{}">{}</ram:GlobalID>"#,
                xml_escape(scheme), xml_escape(id)));
        }

        if let Some(name) = &party.name {
            xml.push_str(&format!(r#"
               <ram:Name>{}</ram:Name>"#, xml_escape(name)));
        }

        xml.push_str(&format!(r#"
               <ram:RoleCode>{}</ram:RoleCode>"#, party.role_code.code()));

        if let Some(endpoint) = &party.endpoint_id {
            xml.push_str(r#"
               <ram:URIUniversalCommunication>"#);
            if let Some(scheme) = &party.endpoint_scheme {
                xml.push_str(&format!(r#"
                    <ram:URIID schemeID="{}">{}</ram:URIID>"#,
                    xml_escape(scheme), xml_escape(endpoint)));
            } else {
                xml.push_str(&format!(r#"
                    <ram:URIID>{}</ram:URIID>"#, xml_escape(endpoint)));
            }
            xml.push_str(r#"
               </ram:URIUniversalCommunication>"#);
        }

        xml.push_str(&format!(r#"
          </ram:{}>"#, tag));
    }

    fn write_referenced_document(&self, xml: &mut String, doc: &ReferencedDocument) {
        xml.push_str(r#"
          <ram:ReferenceReferencedDocument>"#);

        xml.push_str(&format!(r#"
               <ram:IssuerAssignedID>{}</ram:IssuerAssignedID>"#,
            xml_escape(&doc.invoice_id)));

        if let Some(sc) = doc.status_code {
            xml.push_str(&format!(r#"
               <ram:StatusCode>{}</ram:StatusCode>"#, sc));
        }

        if let Some(tc) = &doc.type_code {
            xml.push_str(&format!(r#"
               <ram:TypeCode>{}</ram:TypeCode>"#, xml_escape(tc)));
        }

        if let Some(rdt) = &doc.receipt_datetime {
            xml.push_str(&format!(r#"
               <ram:ReceiptDateTime>
                    <udt:DateTimeString format="204">{}</udt:DateTimeString>
               </ram:ReceiptDateTime>"#, rdt));
        }

        if let Some(idt) = &doc.issue_date {
            xml.push_str(&format!(r#"
               <ram:FormattedIssueDateTime>
                    <qdt:DateTimeString format="102">{}</qdt:DateTimeString>
               </ram:FormattedIssueDateTime>"#, idt));
        }

        xml.push_str(&format!(r#"
               <ram:ProcessConditionCode>{}</ram:ProcessConditionCode>"#,
            doc.process_condition_code));

        if let Some(pc) = &doc.process_condition {
            xml.push_str(&format!(r#"
               <ram:ProcessCondition>{}</ram:ProcessCondition>"#, xml_escape(pc)));
        }

        // IssuerTradeParty du document référencé
        if let Some(issuer) = &doc.issuer {
            xml.push_str(r#"
               <ram:IssuerTradeParty>"#);
            if let (Some(id), Some(scheme)) = (&issuer.global_id, &issuer.global_id_scheme) {
                xml.push_str(&format!(r#"
                    <ram:GlobalID schemeID="{}">{}</ram:GlobalID>"#,
                    xml_escape(scheme), xml_escape(id)));
            }
            if let Some(name) = &issuer.name {
                xml.push_str(&format!(r#"
                    <ram:Name>{}</ram:Name>"#, xml_escape(name)));
            }
            xml.push_str(&format!(r#"
                    <ram:RoleCode>{}</ram:RoleCode>"#, issuer.role_code.code()));
            xml.push_str(r#"
               </ram:IssuerTradeParty>"#);
        }

        // SpecifiedDocumentStatus(es)
        for status in &doc.statuses {
            xml.push_str(r#"
               <ram:SpecifiedDocumentStatus>"#);

            if let Some(rc) = &status.reason_code {
                xml.push_str(&format!(r#"
                    <ram:ReasonCode>{}</ram:ReasonCode>"#, xml_escape(rc)));
            }
            if let Some(r) = &status.reason {
                xml.push_str(&format!(r#"
                    <ram:Reason>{}</ram:Reason>"#, xml_escape(r)));
            }
            if let Some(ac) = &status.action_code {
                xml.push_str(&format!(r#"
                    <ram:RequestedActionCode>{}</ram:RequestedActionCode>"#, xml_escape(ac)));
            }
            if let Some(a) = &status.action {
                xml.push_str(&format!(r#"
                    <ram:RequestedAction>{}</ram:RequestedAction>"#, xml_escape(a)));
            }
            if let Some(seq) = status.sequence {
                xml.push_str(&format!(r#"
                    <ram:SequenceNumeric>{}</ram:SequenceNumeric>"#, seq));
            }

            for ch in &status.characteristics {
                xml.push_str(r#"
                    <ram:SpecifiedDocumentCharacteristic>"#);
                if let Some(id) = &ch.id {
                    xml.push_str(&format!(r#"
                         <ram:ID>{}</ram:ID>"#, xml_escape(id)));
                }
                xml.push_str(&format!(r#"
                         <ram:TypeCode>{}</ram:TypeCode>"#, xml_escape(&ch.type_code)));
                if let Some(vc) = ch.value_changed {
                    xml.push_str(&format!(r#"
                         <ram:ValueChangedIndicator>
                              <udt:IndicatorString>{}</udt:IndicatorString>
                         </ram:ValueChangedIndicator>"#,
                        if vc { "true" } else { "false" }));
                }
                if let Some(name) = &ch.name {
                    xml.push_str(&format!(r#"
                         <ram:Name>{}</ram:Name>"#, xml_escape(name)));
                }
                if let Some(loc) = &ch.location {
                    xml.push_str(&format!(r#"
                         <ram:Location>{}</ram:Location>"#, xml_escape(loc)));
                }
                if let Some(vp) = &ch.value_percent {
                    xml.push_str(&format!(r#"
                         <ram:ValuePercent>{}</ram:ValuePercent>"#, vp));
                }
                if let Some(va) = &ch.value_amount {
                    xml.push_str(&format!(r#"
                         <ram:ValueAmount>{}</ram:ValueAmount>"#, va));
                }
                xml.push_str(r#"
                    </ram:SpecifiedDocumentCharacteristic>"#);
            }

            xml.push_str(r#"
               </ram:SpecifiedDocumentStatus>"#);
        }

        xml.push_str(r#"
          </ram:ReferenceReferencedDocument>"#);
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::model::InvoiceFormat;

    fn make_test_invoice() -> InvoiceData {
        let mut inv = InvoiceData::new("F202500001".to_string(), InvoiceFormat::UBL);
        inv.issue_date = Some("2025-07-01".to_string());
        inv.seller_siret = Some("10000000900015".to_string());
        inv.buyer_siret = Some("20000000800014".to_string());
        inv.seller_name = Some("VENDEUR".to_string());
        inv.buyer_name = Some("ACHETEUR".to_string());
        inv
    }

    #[test]
    fn test_generate_deposee() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let cdv = gen.generate_deposee(&invoice, "380");

        assert!(cdv.is_success());
        assert!(!cdv.is_rejected());
        assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 200);
        assert_eq!(cdv.referenced_documents[0].invoice_id, "F202500001");
        assert_eq!(cdv.referenced_documents[0].status_code, Some(10));
    }

    #[test]
    fn test_generate_rejetee() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let errors = vec![
            CdarValidationError {
                rule_id: "BR-CO-15".to_string(),
                severity: "ERROR".to_string(),
                location: Some("/Invoice/ID".to_string()),
                message: "Erreur de validation sémantique".to_string(),
                reason_code: Some(StatusReasonCode::RejSeman),
            },
        ];
        let cdv = gen.generate_rejetee(&invoice, "380", errors);

        assert!(!cdv.is_success());
        assert!(cdv.is_rejected());
        assert!(cdv.has_reason_codes());
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 213);
        assert_eq!(cdv.referenced_documents[0].statuses[0].reason_code, Some("REJ_SEMAN".to_string()));
    }

    #[test]
    fn test_generate_irrecevable() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let cdv = gen.generate_irrecevable(
            &invoice,
            StatusReasonCode::IrrSyntax,
            "Erreur de syntaxe XML",
        );

        assert!(cdv.is_irrecevable());
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 501);
        assert_eq!(cdv.referenced_documents[0].statuses[0].reason_code, Some("IRR_SYNTAX".to_string()));
    }

    #[test]
    fn test_cdv_to_xml_deposee() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let cdv = gen.generate_deposee(&invoice, "380");
        let xml = gen.to_xml(&cdv).unwrap();

        assert!(xml.contains("CrossDomainAcknowledgementAndResponse"));
        assert!(xml.contains("F202500001"));
        assert!(xml.contains("<ram:TypeCode>305</ram:TypeCode>"));
        assert!(xml.contains("<ram:ProcessConditionCode>200</ram:ProcessConditionCode>"));
        assert!(xml.contains("<ram:RoleCode>WK</ram:RoleCode>"));
        assert!(xml.contains("<ram:RoleCode>SE</ram:RoleCode>"));
        assert!(xml.contains("schemeID=\"0002\""));
        assert!(xml.contains("urn.cpro.gouv.fr:1p0:CDV:invoice"));
    }

    #[test]
    fn test_cdv_to_xml_rejetee() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let errors = vec![
            CdarValidationError {
                rule_id: "BR-01".to_string(),
                severity: "FATAL".to_string(),
                location: None,
                message: "Erreur sémantique".to_string(),
                reason_code: Some(StatusReasonCode::RejSeman),
            },
        ];
        let cdv = gen.generate_rejetee(&invoice, "380", errors);
        let xml = gen.to_xml(&cdv).unwrap();

        assert!(xml.contains("<ram:ProcessConditionCode>213</ram:ProcessConditionCode>"));
        assert!(xml.contains("<ram:ReasonCode>REJ_SEMAN</ram:ReasonCode>"));
        assert!(xml.contains("<ram:StatusCode>8</ram:StatusCode>"));
    }

    // ===== Tests generate_status pour tous les statuts =====

    fn roundtrip(cdv: &CdvResponse, gen: &CdarGenerator, expected_code: u16) {
        let xml = gen.to_xml(cdv).unwrap();
        let parser = crate::parser::CdarParser::new();
        let parsed = parser.parse(&xml).expect("Roundtrip parse failed");
        assert_eq!(parsed.referenced_documents[0].process_condition_code, expected_code);
        assert_eq!(parsed.referenced_documents[0].invoice_id, "F202500001");
    }

    #[test]
    fn test_generate_emise_201() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_emise(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 201);
        assert_eq!(cdv.type_code, CdvTypeCode::Transmission);
        roundtrip(&cdv, &gen, 201);
    }

    #[test]
    fn test_generate_recue_202() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_status(InvoiceStatusCode::Recue, CdvTypeCode::Transmission,
            RoleCode::WK, &inv, "380", Vec::new());
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 202);
        roundtrip(&cdv, &gen, 202);
    }

    #[test]
    fn test_generate_mise_a_disposition_203() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_mise_a_disposition(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 203);
        roundtrip(&cdv, &gen, 203);
    }

    #[test]
    fn test_generate_prise_en_charge_204() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_prise_en_charge(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 204);
        assert_eq!(cdv.type_code, CdvTypeCode::Traitement);
        roundtrip(&cdv, &gen, 204);
    }

    #[test]
    fn test_generate_approuvee_205() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_status(InvoiceStatusCode::Approuvee, CdvTypeCode::Traitement,
            RoleCode::BY, &inv, "380", Vec::new());
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 205);
        roundtrip(&cdv, &gen, 205);
    }

    #[test]
    fn test_generate_approuvee_partiellement_206() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_approuvee_partiellement(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 206);
        roundtrip(&cdv, &gen, 206);
    }

    #[test]
    fn test_generate_en_litige_207() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_status(InvoiceStatusCode::EnLitige, CdvTypeCode::Traitement,
            RoleCode::BY, &inv, "380", vec![DocumentStatus {
                status_code: None,
                reason_code: Some("MONTANTTOTAL_ERR".to_string()),
                reason: Some("Montant contesté".to_string()),
                action_code: None, action: None, sequence: Some(1),
                characteristics: Vec::new(),
            }]);
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 207);
        roundtrip(&cdv, &gen, 207);
    }

    #[test]
    fn test_generate_suspendue_208() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_suspendue(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 208);
        roundtrip(&cdv, &gen, 208);
    }

    #[test]
    fn test_generate_completee_209() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_completee(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 209);
        roundtrip(&cdv, &gen, 209);
    }

    #[test]
    fn test_generate_refusee_210() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_refusee(&inv, "380",
            StatusReasonCode::MontantTotalErr, "Montant incorrect");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 210);
        assert!(cdv.referenced_documents[0].statuses[0].reason_code.as_deref() == Some("MONTANTTOTAL_ERR"));
        roundtrip(&cdv, &gen, 210);
    }

    #[test]
    fn test_generate_paiement_transmis_211() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_paiement_transmis(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 211);
        roundtrip(&cdv, &gen, 211);
    }

    #[test]
    fn test_generate_encaissee_212() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_encaissee(&inv, "380", 1200.00);
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 212);
        let ch = &cdv.referenced_documents[0].statuses[0].characteristics[0];
        assert_eq!(ch.type_code, "MEN");
        assert_eq!(ch.value_amount.as_deref(), Some("1200.00"));
        roundtrip(&cdv, &gen, 212);
    }

    #[test]
    fn test_generate_visee_214() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_visee(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 214);
        roundtrip(&cdv, &gen, 214);
    }

    #[test]
    fn test_generate_annulee_220() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_annulee(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 220);
        roundtrip(&cdv, &gen, 220);
    }

    #[test]
    fn test_generate_erreur_routage_221() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_erreur_routage(&inv, "Destinataire introuvable");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 221);
        roundtrip(&cdv, &gen, 221);
    }

    #[test]
    fn test_generate_demande_paiement_direct_224() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_demande_paiement_direct(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 224);
        roundtrip(&cdv, &gen, 224);
    }

    #[test]
    fn test_generate_affacturee_225() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_affacturee(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 225);
        roundtrip(&cdv, &gen, 225);
    }

    #[test]
    fn test_generate_affacturee_confidentiel_226() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_affacturee_confidentiel(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 226);
        roundtrip(&cdv, &gen, 226);
    }

    #[test]
    fn test_generate_changement_compte_a_payer_227() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_changement_compte_a_payer(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 227);
        roundtrip(&cdv, &gen, 227);
    }

    #[test]
    fn test_generate_non_affacturee_228() {
        let gen = CdarGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();
        let cdv = gen.generate_non_affacturee(&inv, "380");
        assert_eq!(cdv.referenced_documents[0].process_condition_code, 228);
        roundtrip(&cdv, &gen, 228);
    }

    // ===== Test exhaustif : tous les codes statut spec sont mappés =====

    #[test]
    fn test_all_status_codes_have_labels() {
        let all_codes: Vec<u16> = vec![
            200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214,
            220, 221, 224, 225, 226, 227, 228,
            501,
        ];
        for code in all_codes {
            let status = InvoiceStatusCode::from_code(code);
            assert!(status.is_some(), "Code {} non reconnu dans InvoiceStatusCode", code);
            let label = status.unwrap().label();
            assert!(!label.is_empty(), "Code {} a un label vide", code);
        }
    }

}
