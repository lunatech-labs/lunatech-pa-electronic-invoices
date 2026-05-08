//! Génération de pièces jointes "métier" pour la démo : bon de commande PDF,
//! bordereau de livraison PNG, détail des lignes CSV.
//!
//! Les PJ générées contiennent les VRAIES données de la facture (raison
//! sociale, SIRET, lignes, montants) — utile pour produire des fixtures de
//! démonstration crédibles plutôt que des fichiers vides.
//!
//! Référence : XP Z12-012 BG-24 (cac:AdditionalDocumentReference).

use pdp_core::error::PdpResult;
use pdp_core::model::InvoiceData;

use crate::typst_engine::TypstPdfEngine;

/// Génère un PDF de bon de commande à partir des données de la facture.
/// Le BdC précède la facture dans le cycle commercial : il référence la
/// même transaction (BT-13 PurchaseOrderReference) avec rôles inversés
/// (acheteur = donneur d'ordre, vendeur = fournisseur).
pub fn generate_bon_commande_pdf(invoice: &InvoiceData) -> PdpResult<Vec<u8>> {
    let engine = TypstPdfEngine::from_manifest_dir();
    engine.generate_pdf_with_template(invoice, "bon_commande.typ")
}

/// Génère un PNG de bordereau de livraison (BL) à partir des données de
/// la facture. Document attestant la livraison effective associée à la
/// facture, format A5 paysage avec signature expéditeur/destinataire.
pub fn generate_bordereau_livraison_png(invoice: &InvoiceData) -> PdpResult<Vec<u8>> {
    let engine = TypstPdfEngine::from_manifest_dir();
    engine.generate_png_with_template(invoice, "bordereau_livraison.typ")
}

/// Génère un CSV listant les lignes de facture (BG-25) avec leurs montants.
/// Utile en pièce jointe pour les acheteurs qui réintègrent les lignes
/// dans leur SI sans re-parser l'XML.
pub fn generate_detail_lignes_csv(invoice: &InvoiceData) -> Vec<u8> {
    let mut out = String::new();
    out.push_str("Ligne;Description;Quantite;UniteCode;PU_HT;Total_HT;TVA_pct\n");
    if invoice.lines.is_empty() {
        // Fallback : une ligne synthétique depuis les totaux globaux.
        out.push_str(&format!(
            "1;{};1;C62;{:.2};{:.2};20.00\n",
            csv_escape(&format!("Facture {}", invoice.invoice_number)),
            invoice.total_ht.unwrap_or(0.0),
            invoice.total_ht.unwrap_or(0.0),
        ));
    } else {
        for (i, line) in invoice.lines.iter().enumerate() {
            out.push_str(&format!(
                "{};{};{};{};{:.2};{:.2};{:.2}\n",
                i + 1,
                csv_escape(line.item_name.as_deref().unwrap_or("")),
                line.quantity.unwrap_or(0.0),
                line.unit_code.as_deref().unwrap_or(""),
                line.price.unwrap_or(0.0),
                line.line_net_amount.unwrap_or(0.0),
                line.tax_percent.unwrap_or(0.0),
            ));
        }
    }
    // Total
    out.push_str(&format!(
        ";;;;Total HT;{:.2};\n",
        invoice.total_ht.unwrap_or(0.0)
    ));
    out.push_str(&format!(
        ";;;;Total TVA;{:.2};\n",
        invoice.total_tax.unwrap_or(0.0)
    ));
    out.push_str(&format!(
        ";;;;Total TTC;{:.2};\n",
        invoice.total_ttc.unwrap_or(0.0)
    ));
    out.into_bytes()
}

/// Échappe une chaîne pour CSV : si elle contient `;`, `"` ou newline, on la
/// quote et on double les guillemets internes.
fn csv_escape(s: &str) -> String {
    if s.contains(';') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
