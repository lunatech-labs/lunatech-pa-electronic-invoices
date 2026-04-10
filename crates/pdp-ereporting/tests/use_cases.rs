//! Tests d'intégration couvrant les cas d'usage XP Z12-014 v1.3
//!
//! Chaque test correspond à un cas d'usage ou un scénario de la norme AFNOR.
//! Les numéros de cas (Cas n°X) renvoient à l'Annexe A de XP Z12-014.

use pdp_core::model::{
    DocumentAllowanceCharge, InvoiceData, InvoiceFormat, InvoiceNote, PostalAddress, TaxBreakdown,
};
use pdp_ereporting::generator::EReportingGenerator;
use pdp_ereporting::model::*;

// ============================================================
// Helpers pour la construction de factures de test
// ============================================================

/// Crée une facture B2B standard française (le cas le plus simple)
fn make_standard_b2b_invoice() -> InvoiceData {
    let mut inv = InvoiceData::new("F202500001".to_string(), InvoiceFormat::UBL);
    inv.issue_date = Some("2025-07-01".to_string());
    inv.seller_siret = Some("10000000900015".to_string());
    inv.buyer_siret = Some("20000000800014".to_string());
    inv.seller_name = Some("VENDEUR SA".to_string());
    inv.buyer_name = Some("ACHETEUR SAS".to_string());
    inv.seller_vat_id = Some("FR12100000009".to_string());
    inv.buyer_vat_id = Some("FR34200000008".to_string());
    inv.seller_country = Some("FR".to_string());
    inv.buyer_country = Some("FR".to_string());
    inv.total_ht = Some(1000.00);
    inv.total_tax = Some(200.00);
    inv.total_ttc = Some(1200.00);
    inv.currency = Some("EUR".to_string());
    inv.invoice_type_code = Some("380".to_string());
    inv.business_process = Some("S1".to_string());
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(1000.00),
        tax_amount: Some(200.00),
        category_code: Some("S".to_string()),
        percent: Some(20.0),
        exemption_reason: None,
        exemption_reason_code: None,
    }];
    inv.lines = vec![pdp_core::model::InvoiceLine {
        line_id: Some("1".to_string()),
        note: None,
        object_id: None,
        item_name: Some("Prestation de conseil".to_string()),
        item_description: None,
        seller_item_id: None,
        buyer_item_id: None,
        quantity: Some(10.0),
        unit_code: Some("HUR".to_string()),
        price: Some(100.00),
        gross_price: None,
        line_net_amount: Some(1000.00),
        tax_category_code: Some("S".to_string()),
        tax_percent: Some(20.0),
        standard_item_id: None,
        standard_item_id_scheme: None,
        order_line_reference: None,
        accounting_cost: None,
        period_start: None,
        period_end: None,
        price_discount: None,
        base_quantity: None,
        base_quantity_unit_code: None,
        allowance_charges: Vec::new(),
        line_type: None,
        sub_lines: Vec::new(),
    }];
    inv
}

/// Crée un générateur e-reporting de test
fn make_generator() -> EReportingGenerator {
    EReportingGenerator::new("100000009", "PDP Test")
}

/// Helper : crée une InvoiceLine simple pour les tests
fn make_invoice_line(
    id: &str,
    name: &str,
    qty: f64,
    price: f64,
    tax_code: &str,
    tax_pct: f64,
) -> pdp_core::model::InvoiceLine {
    pdp_core::model::InvoiceLine {
        line_id: Some(id.to_string()),
        note: None,
        object_id: None,
        item_name: Some(name.to_string()),
        item_description: None,
        seller_item_id: None,
        buyer_item_id: None,
        quantity: Some(qty),
        unit_code: Some("C62".to_string()),
        price: Some(price),
        gross_price: None,
        line_net_amount: Some(qty * price),
        tax_category_code: Some(tax_code.to_string()),
        tax_percent: Some(tax_pct),
        standard_item_id: None,
        standard_item_id_scheme: None,
        order_line_reference: None,
        accounting_cost: None,
        period_start: None,
        period_end: None,
        price_discount: None,
        base_quantity: None,
        base_quantity_unit_code: None,
        allowance_charges: Vec::new(),
        line_type: None,
        sub_lines: Vec::new(),
    }
}

// ============================================================
// CHAPITRE 2 : Cas nominal d'échange de factures
// ============================================================

/// §2.1 — Transmission d'une facture standard et cycle de vie
/// Vérifie que la conversion invoice_to_transaction produit un XML valide
/// pour une facture B2B standard (380) avec TVA à 20%.
#[test]
fn test_cas_nominal_facture_standard_380() {
    let inv = make_standard_b2b_invoice();
    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.id, "F202500001");
    assert_eq!(txn.issue_date, "20250701"); // BR-FR-MAP-23 : YYYYMMDD
    assert_eq!(txn.type_code, "380");
    assert_eq!(txn.currency_code, "EUR");

    // BR-FR-MAP-01 : cadre de facturation dérivé de BT-23 "S1" → "B2B"
    assert_eq!(txn.business_process.id, "B2B");
    assert_eq!(
        txn.business_process.type_id,
        "urn.cpro.gouv.fr:1p0:ereporting"
    );

    // BR-FR-MAP-08 : vendeur FR → schéma 0002
    assert_eq!(txn.seller.company_id, Some("100000009".to_string()));
    assert_eq!(txn.seller.company_id_scheme, Some("0002".to_string()));
    assert_eq!(txn.seller.country_code, Some("FR".to_string()));

    // BR-FR-MAP-10 : acheteur FR → schéma 0002
    let buyer = txn.buyer.as_ref().unwrap();
    assert_eq!(buyer.company_id, Some("200000008".to_string()));
    assert_eq!(buyer.company_id_scheme, Some("0002".to_string()));

    // BR-FR-MAP-17/18 : ventilation TVA
    assert_eq!(txn.tax_subtotals.len(), 1);
    assert_eq!(txn.tax_subtotals[0].taxable_amount, 1000.00);
    assert_eq!(txn.tax_subtotals[0].tax_amount, 200.00);
    assert_eq!(txn.tax_subtotals[0].tax_percent, 20.0);
}

/// §2.1 — Vérification XML complète du cas nominal
#[test]
fn test_cas_nominal_xml_complet() {
    let gen = make_generator();
    let inv = make_standard_b2b_invoice();
    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    let report = gen.create_transactions_report(
        "RPT-2025-001",
        "100000009",
        "VENDEUR SA",
        "20250701",
        "20250731",
        vec![txn],
    );

    let xml = gen.to_xml(&report).unwrap();

    // Structure XML de base
    assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("<Report>"));
    assert!(xml.contains("</Report>"));
    assert!(xml.contains("<ReportDocument>"));
    assert!(xml.contains("<TransactionsReport>"));

    // TT-4 : Code type flux 10.1
    assert!(xml.contains("<TypeCode>10.1</TypeCode>"));

    // TG-3 : Émetteur PDP
    assert!(xml.contains("<RoleCode>WK</RoleCode>"));
    // TG-5 : Déclarant
    assert!(xml.contains("<RoleCode>SE</RoleCode>"));

    // Période
    assert!(xml.contains("<StartDate>20250701</StartDate>"));
    assert!(xml.contains("<EndDate>20250731</EndDate>"));

    // Facture
    assert!(xml.contains("<ID>F202500001</ID>"));
    assert!(xml.contains("<IssueDate>20250701</IssueDate>"));
    assert!(xml.contains("<TypeCode>380</TypeCode>"));

    // TVA
    assert!(xml.contains("<TaxableAmount>1000.00</TaxableAmount>"));
    assert!(xml.contains("<TaxAmount CurrencyCode=\"EUR\">200.00</TaxAmount>"));
}

// ============================================================
// §2.6 — Avoir (Credit Note) — Cas de litige suivi d'un avoir
// ============================================================

/// Cas d'usage §2.6 : Gestion d'un litige suivi d'un AVOIR (type 381)
#[test]
fn test_cas_avoir_credit_note_381() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_number = "AV202500001".to_string();
    inv.invoice_type_code = Some("381".to_string()); // Avoir
    inv.total_ht = Some(-500.00);
    inv.total_tax = Some(-100.00);
    inv.total_ttc = Some(-600.00);
    inv.preceding_invoice_reference = Some("F202500001".to_string());
    inv.preceding_invoice_date = Some("2025-07-01".to_string());
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(-500.00),
        tax_amount: Some(-100.00),
        category_code: Some("S".to_string()),
        percent: Some(20.0),
        exemption_reason: None,
        exemption_reason_code: None,
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.type_code, "381");

    // BR-FR-MAP-06 : référence à la facture initiale
    assert_eq!(txn.referenced_documents.len(), 1);
    assert_eq!(txn.referenced_documents[0].id, "F202500001");
    assert_eq!(
        txn.referenced_documents[0].issue_date,
        Some("20250701".to_string())
    );

    // Montants négatifs conservés
    assert_eq!(txn.monetary_total.tax_exclusive_amount, Some(-500.00));
    assert_eq!(txn.monetary_total.tax_amount, -100.00);
}

/// §2.7 : Facture rectificative (type 384)
#[test]
fn test_cas_facture_rectificative_384() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_number = "FR202500001".to_string();
    inv.invoice_type_code = Some("384".to_string());
    inv.preceding_invoice_reference = Some("F202500001".to_string());

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.type_code, "384");
    assert_eq!(txn.referenced_documents.len(), 1);
}

// ============================================================
// §3.2.2 — Cas n°1 : Multi-commande / Multi-livraison
// ============================================================

/// Cas n°1 : Facture avec livraison
#[test]
fn test_cas_1_multi_livraison() {
    let mut inv = make_standard_b2b_invoice();
    inv.delivery_date = Some("2025-07-15".to_string());
    inv.delivery_address = Some(PostalAddress {
        line1: Some("10 rue de la Paix".to_string()),
        line2: None,
        line3: None,
        city: Some("Paris".to_string()),
        postal_code: Some("75001".to_string()),
        country_code: Some("FR".to_string()),
        country_subdivision: None,
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    // BR-FR-MAP-14 : livraison mappée
    assert_eq!(txn.deliveries.len(), 1);
    assert_eq!(txn.deliveries[0].date, Some("20250715".to_string()));
    let loc = txn.deliveries[0].location.as_ref().unwrap();
    assert_eq!(loc.country_code, Some("FR".to_string()));
    assert_eq!(loc.city_name, Some("Paris".to_string()));
}

// ============================================================
// §3.2.3 — Cas n°2 : Facture déjà payée
// ============================================================

/// Cas n°2 : Facture déjà payée — génère un rapport de paiement (flux 10.2)
#[test]
fn test_cas_2_facture_deja_payee() {
    let gen = make_generator();

    let invoices = vec![PaymentInvoice {
        invoice_id: "F202500001".to_string(),
        issue_date: "20250701".to_string(),
        payment: PaymentDetail {
            date: "20250701".to_string(), // Payé le jour même
            sub_totals: vec![PaymentSubTotal {
                tax_percent: 20.0,
                currency_code: Some("EUR".to_string()),
                amount: 1200.00,
            }],
        },
    }];

    let report = gen.create_payments_report(
        "PAY-2025-001",
        "100000009",
        "VENDEUR SA",
        "20250701",
        "20250731",
        invoices,
    );

    assert_eq!(report.document.type_code, ReportTypeCode::PaymentsInitial);
    let pays = report.payments.as_ref().unwrap();
    assert_eq!(pays.invoices.len(), 1);
    assert_eq!(pays.invoices[0].payment.date, "20250701");
}

// ============================================================
// §3.2.4 — Cas n°3 : Facture à payer par un tiers PAYEUR
// ============================================================

/// Cas n°3 : La facture a un tiers payeur — note PMT
#[test]
fn test_cas_3_tiers_payeur() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes.push(InvoiceNote {
        content: "Paiement par SOCIÉTÉ MÈRE SIRET 30000000700013".to_string(),
        subject_code: Some("PMT".to_string()),
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    // BR-FR-MAP-04 : notes mappées, y compris PMT
    assert!(txn
        .notes
        .iter()
        .any(|n| n.subject == Some("PMT".to_string())));
}

// ============================================================
// §3.2.5 — Cas n°4 : Paiement partiel par un tiers
// ============================================================

/// Cas n°4 : Subvention/assurance — paiement partiel par un tiers
#[test]
fn test_cas_4_paiement_partiel_tiers() {
    let gen = make_generator();

    let invoices = vec![PaymentInvoice {
        invoice_id: "F202500010".to_string(),
        issue_date: "20250715".to_string(),
        payment: PaymentDetail {
            date: "20250720".to_string(),
            sub_totals: vec![PaymentSubTotal {
                tax_percent: 20.0,
                currency_code: Some("EUR".to_string()),
                amount: 800.00, // Paiement partiel tiers
            }],
        },
    }];

    let report = gen.create_payments_report(
        "PAY-2025-004",
        "100000009",
        "VENDEUR SA",
        "20250701",
        "20250731",
        invoices,
    );

    let xml = gen.to_xml(&report).unwrap();
    assert!(xml.contains("<Amount>800.00</Amount>"));
}

// ============================================================
// §3.2.8 — Cas n°7 : Carte logée (carte d'achat)
// ============================================================

/// Cas n°7 : Facture payée par carte logée — note AAB
#[test]
fn test_cas_7_carte_logee() {
    let mut inv = make_standard_b2b_invoice();
    inv.payment_means_code = Some("48".to_string()); // Carte de paiement
    inv.notes.push(InvoiceNote {
        content: "Paiement par carte logée entreprise".to_string(),
        subject_code: Some("AAB".to_string()),
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert!(txn
        .notes
        .iter()
        .any(|n| n.subject == Some("AAB".to_string())));
}

// ============================================================
// §3.2.9 — Cas n°8 : Affacturage
// ============================================================

/// Cas n°8 : Facture avec affacturage — note ACC
#[test]
fn test_cas_8_affacturage() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes.push(InvoiceNote {
        content: "Cédée à FACTOR SA SIRET 40000000600012".to_string(),
        subject_code: Some("ACC".to_string()),
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    let acc_note = txn
        .notes
        .iter()
        .find(|n| n.subject == Some("ACC".to_string()));
    assert!(acc_note.is_some());
    assert!(acc_note
        .unwrap()
        .content
        .as_ref()
        .unwrap()
        .contains("FACTOR SA"));
}

// ============================================================
// §3.2.12 — Cas n°13 : Sous-traitance avec autoliquidation
// ============================================================

/// Cas n°13 : Sous-traitance avec autoliquidation (reverse charge, catégorie AE)
#[test]
fn test_cas_13_sous_traitance_autoliquidation() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(5000.00),
        tax_amount: Some(0.00),
        category_code: Some("AE".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Autoliquidation art. 283-2 nonies CGI".to_string()),
        exemption_reason_code: Some("VATEX-EU-AE".to_string()),
    }];

    // Détection autoliquidation
    assert!(EReportingGenerator::has_reverse_charge(&inv));

    // Catégorie AE → TNT1
    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(
        txn.tax_subtotals[0].tax_category_code,
        Some("AE".to_string())
    );
    assert_eq!(
        txn.tax_subtotals[0].tax_exemption_reason_code,
        Some("VATEX-EU-AE".to_string())
    );
}

// ============================================================
// §3.2.17 — Cas n°18 : Notes de débit
// ============================================================

/// Cas n°18 : Note de débit (type 383)
#[test]
fn test_cas_18_note_debit() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_number = "ND202500001".to_string();
    inv.invoice_type_code = Some("383".to_string());

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.type_code, "383");
}

// ============================================================
// §3.2.18 — Cas n°19b : Auto-facturation (self-billing)
// ============================================================

/// Cas n°19b : Auto-facturation (type 389)
#[test]
fn test_cas_19b_autofacturation() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_number = "SF202500001".to_string();
    inv.invoice_type_code = Some("389".to_string());
    inv.business_process = Some("B5".to_string());

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.type_code, "389");
    assert_eq!(txn.business_process.id, "B2B");
}

// ============================================================
// §3.2.19 — Cas n°20/21 : Acompte et facture finale
// ============================================================

/// Cas n°20 : Facture d'acompte (type 386)
#[test]
fn test_cas_20_facture_acompte() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_number = "AC202500001".to_string();
    inv.invoice_type_code = Some("386".to_string());
    inv.total_ht = Some(300.00);
    inv.total_tax = Some(60.00);
    inv.total_ttc = Some(360.00);
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(300.00),
        tax_amount: Some(60.00),
        category_code: Some("S".to_string()),
        percent: Some(20.0),
        exemption_reason: None,
        exemption_reason_code: None,
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.type_code, "386");
    assert_eq!(txn.monetary_total.tax_exclusive_amount, Some(300.00));
    assert_eq!(txn.monetary_total.tax_amount, 60.00);
}

/// Cas n°21 : Facture finale après acompte (380 avec référence)
#[test]
fn test_cas_21_facture_finale_apres_acompte() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_number = "FF202500001".to_string();
    inv.preceding_invoice_reference = Some("AC202500001".to_string());
    inv.preceding_invoice_date = Some("2025-06-15".to_string());
    inv.prepaid_amount = Some(360.00);

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.type_code, "380");
    assert_eq!(txn.referenced_documents.len(), 1);
    assert_eq!(txn.referenced_documents[0].id, "AC202500001");
}

// ============================================================
// §3.2.20/21 — Cas n°22a/22b : Escompte
// ============================================================

/// Cas n°22a : Escompte sur prestations de services (TVA à l'encaissement)
#[test]
fn test_cas_22a_escompte_services() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_due_on_payment = Some(true);
    inv.notes.push(InvoiceNote {
        content: "Escompte 2% pour paiement sous 10 jours".to_string(),
        subject_code: Some("AAB".to_string()),
    });
    inv.allowance_charges = vec![DocumentAllowanceCharge {
        charge_indicator: false,
        amount: Some(20.00),
        reason: Some("Escompte".to_string()),
        tax_category_code: Some("S".to_string()),
        tax_percent: Some(20.0),
        base_amount: None,
        percentage: None,
        reason_code: None,
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.allowance_charges.len(), 1);
    assert!(!txn.allowance_charges[0].charge_indicator);
    assert_eq!(txn.allowance_charges[0].amount, Some(20.00));
}

/// Cas n°22b : Escompte sur livraison de biens
#[test]
fn test_cas_22b_escompte_biens() {
    let mut inv = make_standard_b2b_invoice();
    inv.delivery_date = Some("2025-07-10".to_string());
    inv.tax_due_on_payment = Some(false);
    inv.allowance_charges = vec![DocumentAllowanceCharge {
        charge_indicator: false,
        amount: Some(15.00),
        reason: Some("Escompte 1.5%".to_string()),
        tax_category_code: Some("S".to_string()),
        tax_percent: Some(20.0),
        base_amount: None,
        percentage: None,
        reason_code: None,
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.allowance_charges.len(), 1);
    assert!(!txn.allowance_charges[0].charge_indicator);
}

// ============================================================
// §3.2.22 — Cas n°23 : Auto-facturation B2C
// ============================================================

/// Cas n°23 : Flux auto-facturation entre particulier et professionnel
#[test]
fn test_cas_23_autofacturation_b2c() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_siret = None;
    inv.buyer_vat_id = None;
    inv.buyer_country = None;
    inv.buyer_name = Some("Particulier".to_string());
    inv.notes = vec![InvoiceNote {
        content: "B2C".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.business_process.id, "B2C");
    assert!(txn.buyer.is_none());
}

// ============================================================
// §3.2.25 — Cas n°26 : Réserve de propriété
// ============================================================

/// Cas n°26 : Clause de réserve → note spécifique
#[test]
fn test_cas_26_reserve_propriete() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes.push(InvoiceNote {
        content: "Clause de réserve de propriété applicable conformément à la loi 80-335"
            .to_string(),
        subject_code: Some("AAB".to_string()),
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert!(txn.notes.len() >= 1);
}

// ============================================================
// §3.2.28 — Cas n°29 : Assujetti unique
// ============================================================

/// Cas n°29 : Opérations internes à un assujetti unique
#[test]
fn test_cas_29_assujetti_unique() {
    let mut inv = make_standard_b2b_invoice();
    inv.business_process = Some("S7".to_string());
    inv.notes.push(InvoiceNote {
        content: "Opération interne assujetti unique art. 256 C du CGI".to_string(),
        subject_code: Some("TXD".to_string()),
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.business_process.id, "B2B");
    assert!(txn
        .notes
        .iter()
        .any(|n| n.subject == Some("TXD".to_string())));
}

// ============================================================
// §3.2.29 — Cas n°30 : TVA déjà collectée (B2C → facture a posteriori)
// ============================================================

#[test]
fn test_cas_30_tva_deja_collectee() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes = vec![InvoiceNote {
        content: "B2C".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "B2C");
}

// ============================================================
// §3.2.30 — Cas n°31 : Factures "mixtes" (multi-TVA)
// ============================================================

/// Cas n°31 : Facture avec 3 taux de TVA distincts
#[test]
fn test_cas_31_facture_mixte_multi_tva() {
    let mut inv = make_standard_b2b_invoice();
    inv.total_ht = Some(1500.00);
    inv.total_tax = Some(311.00);
    inv.tax_breakdowns = vec![
        TaxBreakdown {
            taxable_amount: Some(1000.00),
            tax_amount: Some(200.00),
            category_code: Some("S".to_string()),
            percent: Some(20.0),
            exemption_reason: None,
            exemption_reason_code: None,
        },
        TaxBreakdown {
            taxable_amount: Some(200.00),
            tax_amount: Some(11.00),
            category_code: Some("S".to_string()),
            percent: Some(5.5),
            exemption_reason: None,
            exemption_reason_code: None,
        },
        TaxBreakdown {
            taxable_amount: Some(300.00),
            tax_amount: Some(100.00),
            category_code: Some("S".to_string()),
            percent: Some(10.0),
            exemption_reason: None,
            exemption_reason_code: None,
        },
    ];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.tax_subtotals.len(), 3);
    assert_eq!(txn.tax_subtotals[0].tax_percent, 20.0);
    assert_eq!(txn.tax_subtotals[1].tax_percent, 5.5);
    assert_eq!(txn.tax_subtotals[2].tax_percent, 10.0);

    let total_tax: f64 = txn.tax_subtotals.iter().map(|t| t.tax_amount).sum();
    assert!((total_tax - 311.0).abs() < 0.01);
}

// ============================================================
// §3.2.31 — Cas n°32 : Mensualités
// ============================================================

/// Cas n°32 : Facture de mensualité avec période de facturation
#[test]
fn test_cas_32_mensualites() {
    let mut inv = make_standard_b2b_invoice();
    inv.invoice_period_start = Some("2025-07-01".to_string());
    inv.invoice_period_end = Some("2025-07-31".to_string());

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    let period = txn.invoice_period.as_ref().unwrap();
    assert_eq!(period.start_date, Some("20250701".to_string()));
    assert_eq!(period.end_date, Some("20250731".to_string()));
}

// ============================================================
// §3.2.32 — Cas n°33 : Régime de la marge (TMA1)
// ============================================================

/// Cas n°33 : Biens d'occasion — catégorie TVA "L"
#[test]
fn test_cas_33_regime_marge_biens_occasion_l() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(800.00),
        tax_amount: Some(0.00),
        category_code: Some("L".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Régime de la marge - biens d'occasion".to_string()),
        exemption_reason_code: None,
    }];

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TMA1);
}

/// Cas n°33 : Agences de voyage — catégorie TVA "M"
#[test]
fn test_cas_33_regime_marge_agence_voyage_m() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(500.00),
        tax_amount: Some(0.00),
        category_code: Some("M".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Régime de la marge - agences de voyage".to_string()),
        exemption_reason_code: None,
    }];

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TMA1);
}

// ============================================================
// §3.2.33 — Cas n°34 : Encaissements partiels
// ============================================================

/// Cas n°34 : Paiement partiel puis complément
#[test]
fn test_cas_34_paiements_partiels() {
    let gen = make_generator();

    let invoices = vec![PaymentInvoice {
        invoice_id: "F202500020".to_string(),
        issue_date: "20250701".to_string(),
        payment: PaymentDetail {
            date: "20250715".to_string(),
            sub_totals: vec![PaymentSubTotal {
                tax_percent: 20.0,
                currency_code: Some("EUR".to_string()),
                amount: 600.00,
            }],
        },
    }];

    let report = gen.create_payments_report(
        "PAY-2025-034",
        "100000009",
        "VENDEUR SA",
        "20250701",
        "20250731",
        invoices,
    );

    let xml = gen.to_xml(&report).unwrap();
    assert!(xml.contains("<Amount>600.00</Amount>"));
    assert!(xml.contains("<Date>20250715</Date>"));
}

// ============================================================
// §3.2.35 — Cas n°36 : Secret professionnel
// ============================================================

/// Cas n°36 : Note TXD pour données sensibles
#[test]
fn test_cas_36_secret_professionnel() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes.push(InvoiceNote {
        content: "Données soumises au secret professionnel".to_string(),
        subject_code: Some("TXD".to_string()),
    });

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert!(txn
        .notes
        .iter()
        .any(|n| n.subject == Some("TXD".to_string())));
}

// ============================================================
// §3.2.37 — Cas n°38 : Sous-lignes multi-TVA
// ============================================================

/// Cas n°38 : Facture multi-lignes avec différentes TVA
#[test]
fn test_cas_38_sous_lignes_multi_tva() {
    let mut inv = make_standard_b2b_invoice();
    inv.lines = vec![
        make_invoice_line("1", "Produit A", 5.0, 100.0, "S", 20.0),
        make_invoice_line("2", "Produit B taux réduit", 10.0, 50.0, "S", 5.5),
    ];
    inv.tax_breakdowns = vec![
        TaxBreakdown {
            taxable_amount: Some(500.00),
            tax_amount: Some(100.00),
            category_code: Some("S".to_string()),
            percent: Some(20.0),
            exemption_reason: None,
            exemption_reason_code: None,
        },
        TaxBreakdown {
            taxable_amount: Some(500.00),
            tax_amount: Some(27.50),
            category_code: Some("S".to_string()),
            percent: Some(5.5),
            exemption_reason: None,
            exemption_reason_code: None,
        },
    ];
    inv.total_ht = Some(1000.00);
    inv.total_tax = Some(127.50);

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.lines.len(), 2);
    assert_eq!(txn.lines[0].tax_percent, Some(20.0));
    assert_eq!(txn.lines[1].tax_percent, Some(5.5));
    assert_eq!(txn.tax_subtotals.len(), 2);
}

// ============================================================
// §3.2.42 — Cas n°43 : E-reporting B2B international
// ============================================================

/// Cas n°43 : Vente B2B international — acheteur UE (Allemagne)
#[test]
fn test_cas_43_b2b_international_ue() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_siret = None;
    inv.buyer_vat_id = Some("DE123456789".to_string());
    inv.buyer_country = Some("DE".to_string());
    inv.notes = vec![InvoiceNote {
        content: "B2BINT".to_string(),
        subject_code: Some("BAR".to_string()),
    }];
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(1000.00),
        tax_amount: Some(0.00),
        category_code: Some("K".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Livraison intracommunautaire".to_string()),
        exemption_reason_code: Some("VATEX-EU-IC".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.business_process.id, "B2BINT");

    // BR-FR-MAP-16 : acheteur UE → pas de SIREN
    let buyer = txn.buyer.as_ref().unwrap();
    assert_eq!(buyer.company_id, None);
    assert_eq!(buyer.country_code, Some("DE".to_string()));

    // Catégorie K → TNT1
    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);
}

/// Cas n°43 : Vente B2B international — acheteur hors UE (USA)
#[test]
fn test_cas_43_b2b_international_hors_ue() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_siret = None;
    inv.buyer_vat_id = None;
    inv.buyer_country = Some("US".to_string());
    inv.notes = vec![InvoiceNote {
        content: "B2BINT".to_string(),
        subject_code: Some("BAR".to_string()),
    }];
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(2000.00),
        tax_amount: Some(0.00),
        category_code: Some("G".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Export hors UE".to_string()),
        exemption_reason_code: Some("VATEX-EU-G".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.business_process.id, "B2BINT");

    let buyer = txn.buyer.as_ref().unwrap();
    assert_eq!(buyer.country_code, Some("US".to_string()));

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);
}

/// Cas n°43a : Opérations triangulaires
#[test]
fn test_cas_43a_operations_triangulaires() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_siret = None;
    inv.buyer_vat_id = Some("IT12345678901".to_string());
    inv.buyer_country = Some("IT".to_string());
    inv.notes = vec![InvoiceNote {
        content: "B2BINT".to_string(),
        subject_code: Some("BAR".to_string()),
    }];
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(5000.00),
        tax_amount: Some(0.00),
        category_code: Some("K".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Opération triangulaire art. 141 dir. 2006/112/CE".to_string()),
        exemption_reason_code: Some("VATEX-EU-IC".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "B2BINT");

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);
}

/// Cas n°43b : Transfert de stock intra-communautaire
#[test]
fn test_cas_43b_transfert_stock_intra_ue() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_siret = None;
    inv.buyer_vat_id = Some("ES12345678A".to_string());
    inv.buyer_country = Some("ES".to_string());
    inv.notes = vec![InvoiceNote {
        content: "B2BINT".to_string(),
        subject_code: Some("BAR".to_string()),
    }];
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(3000.00),
        tax_amount: Some(0.00),
        category_code: Some("K".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Transfert de stock art. 17 dir. 2006/112/CE".to_string()),
        exemption_reason_code: Some("VATEX-EU-IC".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "B2BINT");

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);
}

// ============================================================
// §3.2.43 — Cas n°44 : DROM / COM / TAAF
// ============================================================

/// Cas n°44 : Transaction DROM avec taux réduit
#[test]
fn test_cas_44_drom_taux_reduit() {
    let mut inv = make_standard_b2b_invoice();
    inv.delivery_address = Some(PostalAddress {
        line1: Some("Zone industrielle Jarry".to_string()),
        line2: None,
        line3: None,
        city: Some("Baie-Mahault".to_string()),
        postal_code: Some("97122".to_string()),
        country_code: Some("GP".to_string()),
        country_subdivision: None,
    });
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(1000.00),
        tax_amount: Some(85.00),
        category_code: Some("S".to_string()),
        percent: Some(8.5),
        exemption_reason: None,
        exemption_reason_code: None,
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.tax_subtotals[0].tax_percent, 8.5);
}

// ============================================================
// Exonérations TVA
// ============================================================

/// Opérations exonérées de TVA (catégorie E)
#[test]
fn test_exoneration_tva_categorie_e() {
    let mut inv = make_standard_b2b_invoice();
    inv.total_tax = Some(0.00);
    inv.total_ht = Some(2000.00);
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(2000.00),
        tax_amount: Some(0.00),
        category_code: Some("E".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Formation professionnelle continue".to_string()),
        exemption_reason_code: Some("VATEX-EU-132".to_string()),
    }];

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.monetary_total.tax_amount, 0.00);
}

/// Opérations hors champ territorial (catégorie O)
#[test]
fn test_hors_champ_territorial_categorie_o() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(1500.00),
        tax_amount: Some(0.00),
        category_code: Some("O".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Non soumis à la TVA".to_string()),
        exemption_reason_code: Some("VATEX-EU-O".to_string()),
    }];

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TNT1);
}

// ============================================================
// BR-FR-MAP-16 : Schéma d'identifiant par pays
// ============================================================

#[test]
fn test_br_fr_map_16_all_eu_countries() {
    let eu_countries = vec![
        "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "DE", "GR", "HU", "IE", "IT",
        "LV", "LT", "LU", "MT", "NL", "PL", "PT", "RO", "SK", "SI", "ES", "SE",
    ];
    for cc in eu_countries {
        assert_eq!(
            EReportingGenerator::id_scheme_for_country(Some(cc)),
            "0223",
            "Pays UE {} devrait avoir le schéma 0223",
            cc
        );
    }
}

#[test]
fn test_br_fr_map_16_non_eu_countries() {
    let non_eu = vec!["US", "GB", "CH", "NO", "JP", "CN", "BR", "IN", "AU", "CA"];
    for cc in non_eu {
        assert_eq!(
            EReportingGenerator::id_scheme_for_country(Some(cc)),
            "0227",
            "Pays hors UE {} devrait avoir le schéma 0227",
            cc
        );
    }
}

#[test]
fn test_br_fr_map_16_france_and_none() {
    assert_eq!(
        EReportingGenerator::id_scheme_for_country(Some("FR")),
        "0002"
    );
    assert_eq!(EReportingGenerator::id_scheme_for_country(None), "0002");
}

#[test]
fn test_invoice_to_transaction_eu_buyer_scheme() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_country = Some("DE".to_string());

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    let buyer = txn.buyer.as_ref().unwrap();
    assert_eq!(buyer.company_id_scheme, Some("0223".to_string()));
}

// ============================================================
// BR-FR-MAP-12 : Représentant fiscal
// ============================================================

/// Vendeur non-FR avec représentant fiscal en France
#[test]
fn test_representant_fiscal() {
    let mut inv = make_standard_b2b_invoice();
    inv.seller_country = Some("DE".to_string());
    inv.tax_representative_vat_id = Some("FR99888777666".to_string());
    inv.tax_representative_name = Some("REPFISC SAS".to_string());

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    let rep = txn.seller_tax_representative.as_ref().unwrap();
    assert_eq!(rep.tax_registration_id, "FR99888777666");
    assert_eq!(rep.scheme_id, "VA");

    // BR-FR-MAP-16 : vendeur DE → schéma 0223
    assert_eq!(txn.seller.company_id_scheme, Some("0223".to_string()));
}

// ============================================================
// BR-FR-MAP-01 : Cadre de facturation — tests exhaustifs
// ============================================================

/// Tous les cadres Sx/Bx/Mx → B2B
#[test]
fn test_br_fr_map_01_cadres_facturation() {
    let cadres = vec![
        "S1", "B1", "S2", "B2", "S3", "B3", "S4", "B4", "S5", "B5", "S6", "B6", "S7", "B7",
        "S8", "B8", "M8",
    ];
    for cadre in cadres {
        let mut inv = make_standard_b2b_invoice();
        inv.notes.clear();
        inv.business_process = Some(cadre.to_string());

        let txn = EReportingGenerator::invoice_to_transaction(&inv);
        assert_eq!(
            txn.business_process.id, "B2B",
            "Cadre {} devrait mapper vers B2B",
            cadre
        );
    }
}

/// Note BAR prioritaire sur BT-23
#[test]
fn test_br_fr_map_01_bar_priorite_sur_bt23() {
    let mut inv = make_standard_b2b_invoice();
    inv.business_process = Some("S1".to_string());
    inv.notes = vec![InvoiceNote {
        content: "B2C".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "B2C");
}

/// Cadres BAR valides
#[test]
fn test_br_fr_map_01_bar_valeurs_valides() {
    let valeurs = vec!["B2B", "B2BINT", "B2C", "OUTOFSCOPE", "ARCHIVEONLY"];
    for val in valeurs {
        let mut inv = make_standard_b2b_invoice();
        inv.business_process = None;
        inv.notes = vec![InvoiceNote {
            content: val.to_string(),
            subject_code: Some("BAR".to_string()),
        }];

        let txn = EReportingGenerator::invoice_to_transaction(&inv);
        assert_eq!(txn.business_process.id, val);
    }
}

/// BAR invalide → fallback défaut
#[test]
fn test_br_fr_map_01_bar_invalide_fallback() {
    let mut inv = make_standard_b2b_invoice();
    inv.business_process = None;
    inv.notes = vec![InvoiceNote {
        content: "INVALID_VALUE".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "B2C");
}

// ============================================================
// Validation devises EUR (TT-52 / TT-83)
// ============================================================

#[test]
fn test_validation_eur_devise_eur() {
    let inv = make_standard_b2b_invoice();
    assert!(EReportingGenerator::validate_tax_currency(&inv).is_ok());
}

#[test]
fn test_validation_eur_devise_etrangere_sans_eur() {
    let mut inv = make_standard_b2b_invoice();
    inv.currency = Some("USD".to_string());
    inv.total_tax = Some(200.0);
    inv.tax_amount_eur = None;
    assert!(EReportingGenerator::validate_tax_currency(&inv).is_err());
}

#[test]
fn test_validation_eur_devise_etrangere_avec_eur() {
    let mut inv = make_standard_b2b_invoice();
    inv.currency = Some("GBP".to_string());
    inv.total_tax = Some(170.0);
    inv.tax_amount_eur = Some(195.50);
    assert!(EReportingGenerator::validate_tax_currency(&inv).is_ok());
}

#[test]
fn test_validation_eur_pas_de_devise() {
    let mut inv = make_standard_b2b_invoice();
    inv.currency = None;
    assert!(EReportingGenerator::validate_tax_currency(&inv).is_ok());
}

#[test]
fn test_validation_eur_devise_etrangere_sans_tva() {
    let mut inv = make_standard_b2b_invoice();
    inv.currency = Some("CHF".to_string());
    inv.total_tax = None;
    inv.tax_amount_eur = None;
    assert!(EReportingGenerator::validate_tax_currency(&inv).is_ok());
}

// ============================================================
// Flux 10.3 : Transactions agrégées
// ============================================================

/// Agrégation de factures homogènes
#[test]
fn test_flux_10_3_agregation_homogene() {
    let gen = make_generator();

    let inv1 = make_standard_b2b_invoice();
    let mut inv2 = make_standard_b2b_invoice();
    inv2.invoice_number = "F202500002".to_string();
    inv2.total_ht = Some(2000.00);
    inv2.total_tax = Some(400.00);
    inv2.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(2000.00),
        tax_amount: Some(400.00),
        category_code: Some("S".to_string()),
        percent: Some(20.0),
        exemption_reason: None,
        exemption_reason_code: None,
    }];

    let report = gen
        .create_aggregated_transactions_report(
            "AGG-001",
            "100000009",
            "VENDEUR SA",
            "20250701",
            "20250731",
            &[inv1, inv2],
        )
        .unwrap();

    let txns = report.transactions.as_ref().unwrap();
    assert!(txns.invoices.is_empty());
    assert!(!txns.aggregated_transactions.is_empty());

    let agg = &txns.aggregated_transactions[0];
    assert_eq!(agg.transaction_count, 2);
    assert_eq!(agg.cumulative_amount_ht, 3000.00);
    assert_eq!(agg.cumulative_tax_amount_eur, 600.00);
    assert_eq!(agg.category, TransactionCategory::TPS1);
}

/// Agrégation hétérogène → groupes séparés
#[test]
fn test_flux_10_3_agregation_heterogene() {
    let gen = make_generator();

    let inv_services = make_standard_b2b_invoice();

    let mut inv_export = make_standard_b2b_invoice();
    inv_export.invoice_number = "EX202500001".to_string();
    inv_export.buyer_country = Some("US".to_string());
    inv_export.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(3000.00),
        tax_amount: Some(0.00),
        category_code: Some("G".to_string()),
        percent: Some(0.0),
        exemption_reason: Some("Export".to_string()),
        exemption_reason_code: None,
    }];
    inv_export.total_ht = Some(3000.00);
    inv_export.total_tax = Some(0.00);

    let report = gen
        .create_aggregated_transactions_report(
            "AGG-002",
            "100000009",
            "VENDEUR SA",
            "20250701",
            "20250731",
            &[inv_services, inv_export],
        )
        .unwrap();

    let txns = report.transactions.as_ref().unwrap();
    assert!(txns.aggregated_transactions.len() >= 2);

    let has_tps1 = txns
        .aggregated_transactions
        .iter()
        .any(|a| a.category == TransactionCategory::TPS1);
    let has_tnt1 = txns
        .aggregated_transactions
        .iter()
        .any(|a| a.category == TransactionCategory::TNT1);
    assert!(has_tps1, "Devrait avoir un groupe TPS1");
    assert!(has_tnt1, "Devrait avoir un groupe TNT1");
}

/// Flux 10.3 XML complet
#[test]
fn test_flux_10_3_xml_complet() {
    let gen = make_generator();
    let inv = make_standard_b2b_invoice();

    let report = gen
        .create_aggregated_transactions_report(
            "AGG-003",
            "100000009",
            "VENDEUR SA",
            "20250701",
            "20250731",
            &[inv],
        )
        .unwrap();

    let xml = gen.to_xml(&report).unwrap();

    assert!(xml.contains("<TypeCode>10.3</TypeCode>"));
    assert!(xml.contains("<AggregatedTransaction>"));
    assert!(xml.contains("<Date>20250701</Date>"));
    assert!(xml.contains("<CurrencyCode>EUR</CurrencyCode>"));
    assert!(xml.contains("<Category>TPS1</Category>"));
    assert!(xml.contains("<CumulativeAmountHT>"));
    assert!(xml.contains("<CumulativeTaxAmountEUR>"));
    assert!(xml.contains("<TransactionCount>1</TransactionCount>"));
    assert!(xml.contains("</AggregatedTransaction>"));
    assert!(!xml.contains("<Invoice>"));
}

/// Flux 10.3 avec devise étrangère → échec validation
#[test]
fn test_flux_10_3_devise_etrangere_echec() {
    let gen = make_generator();
    let mut inv = make_standard_b2b_invoice();
    inv.currency = Some("USD".to_string());
    inv.tax_amount_eur = None;

    let result = gen.create_aggregated_transactions_report(
        "AGG-ERR",
        "100000009",
        "VENDEUR SA",
        "20250701",
        "20250731",
        &[inv],
    );
    assert!(result.is_err());
}

// ============================================================
// Flux 10.4 : Paiements agrégés
// ============================================================

/// Flux 10.4 : Paiements agrégés multi-taux
#[test]
fn test_flux_10_4_paiements_agreges() {
    let gen = make_generator();

    let transactions = vec![
        PaymentTransaction {
            payment: PaymentDetail {
                date: "20250731".to_string(),
                sub_totals: vec![
                    PaymentSubTotal {
                        tax_percent: 20.0,
                        currency_code: Some("EUR".to_string()),
                        amount: 25000.00,
                    },
                    PaymentSubTotal {
                        tax_percent: 5.5,
                        currency_code: Some("EUR".to_string()),
                        amount: 3000.00,
                    },
                ],
            },
        },
        PaymentTransaction {
            payment: PaymentDetail {
                date: "20250731".to_string(),
                sub_totals: vec![PaymentSubTotal {
                    tax_percent: 10.0,
                    currency_code: Some("EUR".to_string()),
                    amount: 8000.00,
                }],
            },
        },
    ];

    let report = gen.create_aggregated_payments_report(
        "PAY-AGG-001",
        "100000009",
        "VENDEUR SA",
        "20250701",
        "20250731",
        transactions,
    );

    assert_eq!(
        report.document.type_code,
        ReportTypeCode::PaymentsAggregated
    );

    let xml = gen.to_xml(&report).unwrap();
    assert!(xml.contains("<TypeCode>10.4</TypeCode>"));
    assert!(xml.contains("<Transactions>"));
    assert!(xml.contains("<TaxPercent>20.00</TaxPercent>"));
    assert!(xml.contains("<TaxPercent>5.50</TaxPercent>"));
    assert!(xml.contains("<Amount>25000.00</Amount>"));
}

// ============================================================
// Dérivation catégorie — cas limites
// ============================================================

/// Facture sans ventilation TVA → ligne S par défaut → TPS1
#[test]
fn test_categorie_sans_ventilation_tva() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = Vec::new();
    inv.lines = vec![make_invoice_line("1", "Service", 1.0, 100.0, "S", 20.0)];

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TPS1);
}

/// Facture avec livraison → TLB1
#[test]
fn test_categorie_livraison_biens() {
    let mut inv = make_standard_b2b_invoice();
    inv.delivery_date = Some("2025-07-15".to_string());
    inv.delivery_address = Some(PostalAddress {
        line1: Some("Entrepôt".to_string()),
        line2: None,
        line3: None,
        city: Some("Lyon".to_string()),
        postal_code: Some("69000".to_string()),
        country_code: Some("FR".to_string()),
        country_subdivision: None,
    });

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TLB1);
}

/// Facture mixte marge + standard → TMA1 prioritaire
#[test]
fn test_categorie_mixte_marge_prioritaire() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = vec![
        TaxBreakdown {
            taxable_amount: Some(500.00),
            tax_amount: Some(0.00),
            category_code: Some("L".to_string()),
            percent: Some(0.0),
            exemption_reason: None,
            exemption_reason_code: None,
        },
        TaxBreakdown {
            taxable_amount: Some(200.00),
            tax_amount: Some(40.00),
            category_code: Some("S".to_string()),
            percent: Some(20.0),
            exemption_reason: None,
            exemption_reason_code: None,
        },
    ];

    let cat = EReportingGenerator::derive_transaction_category(&inv);
    assert_eq!(cat, TransactionCategory::TMA1);
}

// ============================================================
// B2C pur sans acheteur
// ============================================================

#[test]
fn test_b2c_sans_acheteur() {
    let mut inv = make_standard_b2b_invoice();
    inv.buyer_siret = None;
    inv.buyer_vat_id = None;
    inv.buyer_country = None;
    inv.buyer_name = None;
    inv.notes = vec![InvoiceNote {
        content: "B2C".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.business_process.id, "B2C");
    assert!(txn.buyer.is_none());
}

// ============================================================
// Remises et charges au niveau document
// ============================================================

#[test]
fn test_remises_charges_document() {
    let mut inv = make_standard_b2b_invoice();
    inv.allowance_charges = vec![
        DocumentAllowanceCharge {
            charge_indicator: false,
            amount: Some(50.00),
            reason: Some("Remise fidélité".to_string()),
            tax_category_code: Some("S".to_string()),
            tax_percent: Some(20.0),
            base_amount: None,
            percentage: None,
            reason_code: None,
        },
        DocumentAllowanceCharge {
            charge_indicator: true,
            amount: Some(25.00),
            reason: Some("Frais de livraison".to_string()),
            tax_category_code: Some("S".to_string()),
            tax_percent: Some(20.0),
            base_amount: None,
            percentage: None,
            reason_code: None,
        },
    ];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);

    assert_eq!(txn.allowance_charges.len(), 2);
    assert!(!txn.allowance_charges[0].charge_indicator);
    assert!(txn.allowance_charges[1].charge_indicator);
    assert_eq!(txn.allowance_charges[0].amount, Some(50.00));
    assert_eq!(txn.allowance_charges[1].amount, Some(25.00));
}

// ============================================================
// Cadres OUTOFSCOPE et ARCHIVEONLY
// ============================================================

#[test]
fn test_cadre_outofscope() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes = vec![InvoiceNote {
        content: "OUTOFSCOPE".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "OUTOFSCOPE");
}

#[test]
fn test_cadre_archiveonly() {
    let mut inv = make_standard_b2b_invoice();
    inv.notes = vec![InvoiceNote {
        content: "ARCHIVEONLY".to_string(),
        subject_code: Some("BAR".to_string()),
    }];

    let txn = EReportingGenerator::invoice_to_transaction(&inv);
    assert_eq!(txn.business_process.id, "ARCHIVEONLY");
}

// ============================================================
// Détection autoliquidation
// ============================================================

#[test]
fn test_has_reverse_charge_true() {
    let mut inv = make_standard_b2b_invoice();
    inv.tax_breakdowns = vec![TaxBreakdown {
        taxable_amount: Some(1000.0),
        tax_amount: Some(0.0),
        category_code: Some("AE".to_string()),
        percent: Some(0.0),
        exemption_reason: None,
        exemption_reason_code: None,
    }];
    assert!(EReportingGenerator::has_reverse_charge(&inv));
}

#[test]
fn test_has_reverse_charge_false() {
    let inv = make_standard_b2b_invoice();
    assert!(!EReportingGenerator::has_reverse_charge(&inv));
}

// ============================================================
// TransactionCategory et ReportTypeCode : roundtrip
// ============================================================

#[test]
fn test_transaction_category_display() {
    assert_eq!(format!("{}", TransactionCategory::TLB1), "TLB1");
    assert_eq!(format!("{}", TransactionCategory::TPS1), "TPS1");
    assert_eq!(format!("{}", TransactionCategory::TNT1), "TNT1");
    assert_eq!(format!("{}", TransactionCategory::TMA1), "TMA1");
}

#[test]
fn test_transaction_category_roundtrip() {
    let cats = vec![
        TransactionCategory::TLB1,
        TransactionCategory::TPS1,
        TransactionCategory::TNT1,
        TransactionCategory::TMA1,
    ];
    for cat in cats {
        assert_eq!(TransactionCategory::from_code(cat.code()), Some(cat));
    }
    assert_eq!(TransactionCategory::from_code("INVALID"), None);
}

#[test]
fn test_report_type_code_roundtrip() {
    let codes = vec![
        ReportTypeCode::TransactionsInitial,
        ReportTypeCode::PaymentsInitial,
        ReportTypeCode::TransactionsAggregated,
        ReportTypeCode::PaymentsAggregated,
    ];
    for code in codes {
        assert_eq!(ReportTypeCode::from_code(code.code()), Some(code));
    }
    assert_eq!(ReportTypeCode::from_code("99.9"), None);
}
