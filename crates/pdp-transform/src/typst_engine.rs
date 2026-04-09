//! Moteur PDF basé sur Typst — remplacement in-process d'Apache FOP.
//!
//! Pipeline : InvoiceData → JSON dict → Typst compilation → PDF bytes
//! Avantages : ~50-100ms vs ~1.4s (FOP Java), pas de JVM, PDF/A-3a natif.

use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::InvoiceData;

/// Moteur de génération PDF via Typst.
pub struct TypstPdfEngine {
    /// Répertoire specs/ contenant les templates et polices
    specs_dir: std::path::PathBuf,
}

impl TypstPdfEngine {
    /// Crée un moteur Typst à partir du répertoire specs/ déduit de CARGO_MANIFEST_DIR.
    pub fn from_manifest_dir() -> Self {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let specs_dir = manifest_dir.parent().unwrap().parent().unwrap().join("specs");
        Self { specs_dir }
    }

    /// Crée un moteur Typst avec un répertoire specs/ explicite.
    pub fn new(specs_dir: std::path::PathBuf) -> Self {
        Self { specs_dir }
    }

    /// Génère un PDF à partir d'une InvoiceData.
    pub fn generate_pdf(&self, invoice: &InvoiceData) -> PdpResult<Vec<u8>> {
        let json_data = self.invoice_to_json(invoice);
        let template = self.load_template()?;
        let fonts = self.load_fonts();

        self.compile_to_pdf(&template, &json_data, &fonts)
    }

    /// Charge le template Typst de facture.
    fn load_template(&self) -> PdpResult<String> {
        let path = self.specs_dir.join("typst/invoice.typ");
        std::fs::read_to_string(&path).map_err(|e| PdpError::TransformError {
            source_format: "InvoiceData".to_string(),
            target_format: "PDF".to_string(),
            message: format!("Template Typst introuvable ({}): {}", path.display(), e),
        })
    }

    /// Charge les polices TTF depuis specs/xslt/mustang/fonts/.
    fn load_fonts(&self) -> Vec<Vec<u8>> {
        let fonts_dir = self.specs_dir.join("xslt/mustang/fonts");
        let font_files = [
            "SourceSansPro-Regular.ttf",
            "SourceSansPro-Bold.ttf",
            "SourceSansPro-It.ttf",
            "SourceSansPro-BoldIt.ttf",
            "SourceSerifPro-Regular.ttf",
            "SourceSerifPro-Bold.ttf",
            "SourceSerifPro-It.ttf",
            "SourceSerifPro-BoldIt.ttf",
        ];

        font_files
            .iter()
            .filter_map(|f| {
                let path = fonts_dir.join(f);
                match std::fs::read(&path) {
                    Ok(bytes) => Some(bytes),
                    Err(e) => {
                        tracing::warn!(font = %f, "Police introuvable: {}", e);
                        None
                    }
                }
            })
            .collect()
    }

    /// Convertit InvoiceData en JSON pour le template Typst.
    fn invoice_to_json(&self, inv: &InvoiceData) -> String {
        let mut map = serde_json::Map::new();

        // Champs scalaires
        Self::set_str(&mut map, "invoice_number", &Some(inv.invoice_number.clone()));
        Self::set_str(&mut map, "issue_date", &inv.issue_date);
        Self::set_str(&mut map, "due_date", &inv.due_date);
        Self::set_str(&mut map, "invoice_type_code", &inv.invoice_type_code);
        Self::set_str(&mut map, "currency", &inv.currency);
        Self::set_str(&mut map, "buyer_reference", &inv.buyer_reference);
        Self::set_str(&mut map, "order_reference", &inv.order_reference);
        Self::set_str(&mut map, "contract_reference", &inv.contract_reference);
        Self::set_str(&mut map, "project_reference", &inv.project_reference);
        Self::set_str(&mut map, "preceding_invoice_reference", &inv.preceding_invoice_reference);

        // Vendeur
        Self::set_str(&mut map, "seller_name", &inv.seller_name);
        Self::set_str(&mut map, "seller_trading_name", &inv.seller_trading_name);
        Self::set_str(&mut map, "seller_siret", &inv.seller_siret);
        Self::set_str(&mut map, "seller_vat_id", &inv.seller_vat_id);
        Self::set_address(&mut map, "seller_address", &inv.seller_address);

        // Acheteur
        Self::set_str(&mut map, "buyer_name", &inv.buyer_name);
        Self::set_str(&mut map, "buyer_trading_name", &inv.buyer_trading_name);
        Self::set_str(&mut map, "buyer_siret", &inv.buyer_siret);
        Self::set_str(&mut map, "buyer_vat_id", &inv.buyer_vat_id);
        Self::set_address(&mut map, "buyer_address", &inv.buyer_address);

        // Livraison
        Self::set_str(&mut map, "delivery_party_name", &inv.delivery_party_name);
        Self::set_str(&mut map, "delivery_date", &inv.delivery_date);
        Self::set_address(&mut map, "delivery_address", &inv.delivery_address);

        // Période
        Self::set_str(&mut map, "invoice_period_start", &inv.invoice_period_start);
        Self::set_str(&mut map, "invoice_period_end", &inv.invoice_period_end);

        // Paiement
        Self::set_str(&mut map, "payment_means_text", &inv.payment_means_text);
        Self::set_str(&mut map, "payment_iban", &inv.payment_iban);
        Self::set_str(&mut map, "payment_bic", &inv.payment_bic);
        Self::set_str(&mut map, "payment_terms", &inv.payment_terms);

        // Totaux
        Self::set_f64(&mut map, "total_ht", inv.total_ht);
        Self::set_f64(&mut map, "total_ttc", inv.total_ttc);
        Self::set_f64(&mut map, "total_tax", inv.total_tax);
        Self::set_f64(&mut map, "allowance_total_amount", inv.allowance_total_amount);
        Self::set_f64(&mut map, "charge_total_amount", inv.charge_total_amount);
        Self::set_f64(&mut map, "prepaid_amount", inv.prepaid_amount);
        Self::set_f64(&mut map, "payable_amount", inv.payable_amount);

        // Lignes
        let lines: Vec<serde_json::Value> = inv.lines.iter().map(|l| {
            let mut lm = serde_json::Map::new();
            Self::set_str(&mut lm, "line_id", &l.line_id);
            Self::set_str(&mut lm, "item_name", &l.item_name);
            Self::set_str(&mut lm, "item_description", &l.item_description);
            Self::set_str(&mut lm, "unit_code", &l.unit_code);
            Self::set_str(&mut lm, "tax_category_code", &l.tax_category_code);
            Self::set_f64(&mut lm, "quantity", l.quantity);
            Self::set_f64(&mut lm, "price", l.price);
            Self::set_f64(&mut lm, "tax_percent", l.tax_percent);
            Self::set_f64(&mut lm, "line_net_amount", l.line_net_amount);
            serde_json::Value::Object(lm)
        }).collect();
        map.insert("lines".to_string(), serde_json::Value::Array(lines));

        // Ventilation TVA
        let tax_bds: Vec<serde_json::Value> = inv.tax_breakdowns.iter().map(|t| {
            let mut tm = serde_json::Map::new();
            Self::set_str(&mut tm, "category_code", &t.category_code);
            Self::set_f64(&mut tm, "taxable_amount", t.taxable_amount);
            Self::set_f64(&mut tm, "tax_amount", t.tax_amount);
            Self::set_f64(&mut tm, "percent", t.percent);
            Self::set_str(&mut tm, "exemption_reason", &t.exemption_reason);
            serde_json::Value::Object(tm)
        }).collect();
        map.insert("tax_breakdowns".to_string(), serde_json::Value::Array(tax_bds));

        // Notes
        let notes: Vec<serde_json::Value> = inv.notes.iter().map(|n| {
            let mut nm = serde_json::Map::new();
            nm.insert("content".to_string(), serde_json::Value::String(n.content.clone()));
            Self::set_str(&mut nm, "subject_code", &n.subject_code);
            serde_json::Value::Object(nm)
        }).collect();
        map.insert("notes".to_string(), serde_json::Value::Array(notes));

        // Remises/charges document
        let acs: Vec<serde_json::Value> = inv.allowance_charges.iter().map(|a| {
            let mut am = serde_json::Map::new();
            am.insert("charge_indicator".to_string(), serde_json::Value::Bool(a.charge_indicator));
            Self::set_f64(&mut am, "amount", a.amount);
            Self::set_str(&mut am, "reason", &a.reason);
            Self::set_str(&mut am, "tax_category_code", &a.tax_category_code);
            Self::set_f64(&mut am, "tax_percent", a.tax_percent);
            serde_json::Value::Object(am)
        }).collect();
        map.insert("allowance_charges".to_string(), serde_json::Value::Array(acs));

        serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
    }

    fn set_str(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, val: &Option<String>) {
        if let Some(v) = val {
            map.insert(key.to_string(), serde_json::Value::String(v.clone()));
        }
    }

    fn set_f64(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, val: Option<f64>) {
        if let Some(v) = val {
            map.insert(key.to_string(), serde_json::json!(v));
        }
    }

    fn set_address(map: &mut serde_json::Map<String, serde_json::Value>, prefix: &str, addr: &Option<pdp_core::model::PostalAddress>) {
        if let Some(a) = addr {
            Self::set_str(map, &format!("{}_line1", prefix), &a.line1);
            Self::set_str(map, &format!("{}_line2", prefix), &a.line2);
            Self::set_str(map, &format!("{}_line3", prefix), &a.line3);
            Self::set_str(map, &format!("{}_city", prefix), &a.city);
            Self::set_str(map, &format!("{}_postal_code", prefix), &a.postal_code);
            Self::set_str(map, &format!("{}_country_code", prefix), &a.country_code);
        }
    }

    /// Compile le template Typst avec les données JSON et produit un PDF.
    fn compile_to_pdf(&self, template: &str, json_data: &str, fonts: &[Vec<u8>]) -> PdpResult<Vec<u8>> {
        use typst::diag::Warned;
        use typst::foundations::Smart;
        use typst_as_lib::TypstEngine;

        // Construire le moteur Typst avec les polices
        let font_refs: Vec<&[u8]> = fonts.iter().map(|f| f.as_slice()).collect();

        let engine = TypstEngine::builder()
            .main_file(template)
            .fonts(font_refs)
            .build();

        // Préparer les inputs : invoice-data = JSON string
        let mut inputs = typst::foundations::Dict::new();
        inputs.insert(
            typst::foundations::Str::from("invoice-data"),
            typst::foundations::Value::Str(typst::foundations::Str::from(json_data)),
        );

        let Warned { output, warnings } =
            engine.compile_with_input::<_, typst::layout::PagedDocument>(inputs);

        // Vérifier les warnings
        for w in &warnings {
            tracing::warn!(warning = %w.message, "Typst warning");
        }

        let document = output.map_err(|e| {
            PdpError::TransformError {
                source_format: "InvoiceData".to_string(),
                target_format: "PDF".to_string(),
                message: format!("Typst compilation échouée: {}", e),
            }
        })?;

        // Générer le PDF
        let pdf_options = typst_pdf::PdfOptions {
            ident: Smart::Auto,
            timestamp: None,
            standards: typst_pdf::PdfStandards::default(),
            page_ranges: None,
            tagged: true,
        };

        let pdf_bytes = typst_pdf::pdf(&document, &pdf_options).map_err(|e| {
            let msgs: Vec<String> = e.iter().map(|d| d.message.to_string()).collect();
            PdpError::TransformError {
                source_format: "InvoiceData".to_string(),
                target_format: "PDF".to_string(),
                message: format!("Typst PDF export échoué: {}", msgs.join("; ")),
            }
        })?;

        if pdf_bytes.len() < 5 || &pdf_bytes[0..5] != b"%PDF-" {
            return Err(PdpError::TransformError {
                source_format: "InvoiceData".to_string(),
                target_format: "PDF".to_string(),
                message: "Le fichier généré par Typst n'est pas un PDF valide".to_string(),
            });
        }

        tracing::debug!(pdf_size = pdf_bytes.len(), "PDF généré via Typst");
        Ok(pdf_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdp_core::model::*;

    fn sample_invoice() -> InvoiceData {
        let mut inv = InvoiceData::new("FA-2025-001".to_string(), InvoiceFormat::CII);
        inv.issue_date = Some("2025-01-15".to_string());
        inv.due_date = Some("2025-02-15".to_string());
        inv.invoice_type_code = Some("380".to_string());
        inv.currency = Some("EUR".to_string());
        inv.seller_name = Some("ACME SAS".to_string());
        inv.seller_siret = Some("12345678901234".to_string());
        inv.seller_vat_id = Some("FR12345678901".to_string());
        inv.seller_address = Some(PostalAddress {
            line1: Some("123 Rue de la Paix".to_string()),
            line2: None,
            line3: None,
            city: Some("Paris".to_string()),
            postal_code: Some("75001".to_string()),
            country_subdivision: None,
            country_code: Some("FR".to_string()),
        });
        inv.buyer_name = Some("Entreprise Client SARL".to_string());
        inv.buyer_siret = Some("98765432109876".to_string());
        inv.buyer_vat_id = Some("FR98765432109".to_string());
        inv.buyer_address = Some(PostalAddress {
            line1: Some("456 Avenue des Champs-Élysées".to_string()),
            line2: None,
            line3: None,
            city: Some("Lyon".to_string()),
            postal_code: Some("69001".to_string()),
            country_subdivision: None,
            country_code: Some("FR".to_string()),
        });

        inv.lines = vec![
            InvoiceLine {
                line_id: Some("1".to_string()),
                note: None,
                object_id: None,
                quantity: Some(10.0),
                unit_code: Some("C62".to_string()),
                line_net_amount: Some(1000.0),
                order_line_reference: None,
                accounting_cost: None,
                price: Some(100.0),
                gross_price: None,
                item_name: Some("Prestation de conseil".to_string()),
                item_description: Some("Accompagnement projet digital".to_string()),
                seller_item_id: None,
                buyer_item_id: None,
                standard_item_id: None,
                standard_item_id_scheme: None,
                tax_category_code: Some("S".to_string()),
                tax_percent: Some(20.0),
                period_start: None,
                period_end: None,
            },
            InvoiceLine {
                line_id: Some("2".to_string()),
                note: None,
                object_id: None,
                quantity: Some(5.0),
                unit_code: Some("HUR".to_string()),
                line_net_amount: Some(500.0),
                order_line_reference: None,
                accounting_cost: None,
                price: Some(100.0),
                gross_price: None,
                item_name: Some("Formation technique".to_string()),
                item_description: None,
                seller_item_id: None,
                buyer_item_id: None,
                standard_item_id: None,
                standard_item_id_scheme: None,
                tax_category_code: Some("S".to_string()),
                tax_percent: Some(20.0),
                period_start: None,
                period_end: None,
            },
        ];

        inv.tax_breakdowns = vec![TaxBreakdown {
            taxable_amount: Some(1500.0),
            tax_amount: Some(300.0),
            category_code: Some("S".to_string()),
            percent: Some(20.0),
            exemption_reason: None,
            exemption_reason_code: None,
        }];

        inv.total_ht = Some(1500.0);
        inv.total_tax = Some(300.0);
        inv.total_ttc = Some(1800.0);
        inv.payable_amount = Some(1800.0);
        inv.payment_iban = Some("FR76 1234 5678 9012 3456 7890 123".to_string());
        inv.payment_bic = Some("BNPAFRPP".to_string());
        inv.payment_terms = Some("Paiement à 30 jours".to_string());

        inv
    }

    #[test]
    fn test_typst_generate_pdf() {
        let engine = TypstPdfEngine::from_manifest_dir();
        let invoice = sample_invoice();
        let pdf = engine.generate_pdf(&invoice).expect("PDF generation failed");

        assert!(pdf.len() > 1000, "PDF trop petit: {} bytes", pdf.len());
        assert_eq!(&pdf[0..5], b"%PDF-", "Header PDF invalide");
    }

    #[test]
    fn test_typst_invoice_to_json() {
        let engine = TypstPdfEngine::from_manifest_dir();
        let invoice = sample_invoice();
        let json = engine.invoice_to_json(&invoice);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON invalide");

        assert_eq!(parsed["invoice_number"], "FA-2025-001");
        assert_eq!(parsed["seller_name"], "ACME SAS");
        assert_eq!(parsed["lines"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["tax_breakdowns"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_typst_avoir() {
        let engine = TypstPdfEngine::from_manifest_dir();
        let mut invoice = sample_invoice();
        invoice.invoice_type_code = Some("381".to_string());
        invoice.invoice_number = "AV-2025-001".to_string();
        let pdf = engine.generate_pdf(&invoice).expect("PDF avoir failed");
        assert!(pdf.len() > 1000);
        assert_eq!(&pdf[0..5], b"%PDF-");
    }

    #[test]
    fn test_typst_minimal_invoice() {
        let engine = TypstPdfEngine::from_manifest_dir();
        let inv = InvoiceData::new("MINIMAL-001".to_string(), InvoiceFormat::CII);
        let pdf = engine.generate_pdf(&inv).expect("Minimal PDF failed");
        assert!(pdf.len() > 500);
        assert_eq!(&pdf[0..5], b"%PDF-");
    }
}
