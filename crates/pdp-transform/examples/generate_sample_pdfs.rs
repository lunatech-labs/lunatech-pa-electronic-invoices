//! Génère des exemples de factures PDF à partir de fichiers CII et UBL.
//!
//! Usage: cargo run -p pdp-transform --example generate_sample_pdfs

use std::fs;
use std::path::Path;

use pdp_core::model::InvoiceAttachment;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let output_dir = project_root.join("output/sample_pdfs");
    fs::create_dir_all(&output_dir).expect("Impossible de créer output/sample_pdfs");

    println!("=== Génération de factures PDF via Typst ===\n");

    // =====================================================================
    // CII : factures, avoirs, rectificatives, cas spéciaux
    // =====================================================================
    println!("--- CII ---");
    let cii_files = [
        ("tests/fixtures/cii/facture_cii_001.xml", "facture_cii_001"),
        ("tests/fixtures/cii/avoir_cii_381.xml", "avoir_cii_381"),
        ("tests/fixtures/cii/facture_rectificative_cii_384.xml", "rectificative_cii_384"),
        ("tests/fixtures/cii/facture_cii_remises_multitva.xml", "facture_cii_remises_multitva"),
        ("tests/fixtures/cii/facture_cii_marketplace_a8.xml", "facture_cii_marketplace_a8"),
        ("tests/fixtures/cii/facture_cii_soustraitance_a4.xml", "facture_cii_soustraitance_a4"),
        ("tests/fixtures/cii/autofacture_cii_389.xml", "autofacture_cii_389"),
        ("tests/fixtures/cii/facture_cii_acompte.xml", "facture_cii_acompte"),
    ];

    for (path, name) in &cii_files {
        generate_pdf(project_root, &output_dir, path, name, "CII");
    }

    // =====================================================================
    // UBL : factures, avoirs, rectificatives, cas spéciaux
    // =====================================================================
    println!("\n--- UBL ---");
    let ubl_files = [
        ("tests/fixtures/ubl/facture_ubl_001.xml", "facture_ubl_001"),
        ("tests/fixtures/ubl/facture_ubl_002_avoir.xml", "avoir_ubl_002"),
        ("tests/fixtures/ubl/facture_rectificative_ubl_384.xml", "rectificative_ubl_384"),
        ("tests/fixtures/ubl/facture_ubl_remises_multitva.xml", "facture_ubl_remises_multitva"),
        ("tests/fixtures/ubl/facture_ubl_marketplace_a8.xml", "facture_ubl_marketplace_a8"),
        ("tests/fixtures/ubl/facture_ubl_soustraitance_a4.xml", "facture_ubl_soustraitance_a4"),
        ("tests/fixtures/ubl/autofacture_ubl_389.xml", "autofacture_ubl_389"),
        ("tests/fixtures/ubl/facture_ubl_acompte_386.xml", "facture_ubl_acompte_386"),
    ];

    for (path, name) in &ubl_files {
        generate_pdf(project_root, &output_dir, path, name, "UBL");
    }

    // =====================================================================
    // Factur-X avec pièces jointes — différents types de factures
    // =====================================================================
    println!("\n--- Factur-X avec pièces jointes ---");

    // CII facture simple + PJ (PDF + CSV + URI externe)
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/cii/facture_cii_001.xml", "CII",
        "facture_cii_avec_pj_facturx",
        vec![
            make_pdf_attachment("BC-001", "Bon de commande", "bon_commande.pdf"),
            make_csv_attachment("DETAIL-001", "Détail des prestations", "detail_prestations.csv",
                "Article;Quantite;PU HT\nConseil;10;100.00\nFormation;5;100.00\n"),
            make_uri_attachment("DEVIS-001", "Devis original", "https://example.com/devis/001.pdf"),
        ],
    );

    // CII avoir + PJ (note de crédit avec justificatif retour)
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/cii/avoir_cii_381.xml", "CII",
        "avoir_cii_avec_pj_facturx",
        vec![
            make_pdf_attachment("RETOUR-001", "Procès-verbal de retour", "pv_retour.pdf"),
            make_text_attachment("MOTIF-001", "Motif de l'avoir", "motif_avoir.txt",
                "Retour marchandise défectueuse - lot 2025-03-15"),
        ],
    );

    // CII rectificative + PJ
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/cii/facture_rectificative_cii_384.xml", "CII",
        "rectificative_cii_avec_pj_facturx",
        vec![
            make_pdf_attachment("JUSTIF-001", "Justificatif de correction", "justificatif_correction.pdf"),
        ],
    );

    // UBL facture simple + PJ (bon de commande PDF uniquement)
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/ubl/facture_ubl_001.xml", "UBL",
        "facture_ubl_avec_pj_facturx",
        vec![
            make_pdf_attachment("BC-002", "Bon de commande client", "bon_commande_client.pdf"),
        ],
    );

    // UBL avoir + PJ
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/ubl/facture_ubl_002_avoir.xml", "UBL",
        "avoir_ubl_avec_pj_facturx",
        vec![
            make_pdf_attachment("RETOUR-002", "Bordereau de retour", "bordereau_retour.pdf"),
            make_csv_attachment("STOCK-001", "État du stock retourné", "stock_retour.csv",
                "Reference;Quantite;Etat\nSRV-2025-001;1;Defectueux\n"),
        ],
    );

    // UBL rectificative + PJ
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/ubl/facture_rectificative_ubl_384.xml", "UBL",
        "rectificative_ubl_avec_pj_facturx",
        vec![
            make_text_attachment("EXPL-001", "Explication de la correction", "explication.txt",
                "Correction du taux de TVA appliqué : 20% au lieu de 10%"),
        ],
    );

    // CII facture sans pièce jointe (référence — vérifie que seul factur-x.xml est embarqué)
    generate_facturx_with_attachments(
        project_root, &output_dir,
        "tests/fixtures/cii/facture_cii_001.xml", "CII",
        "facture_cii_sans_pj_facturx",
        vec![],
    );

    // =====================================================================
    // Factur-X avec différents profils (EN16931 vs Extended)
    // =====================================================================
    println!("\n--- Factur-X multi-profils ---");
    generate_facturx_with_profile(
        project_root, &output_dir,
        "tests/fixtures/cii/facture_cii_001.xml", "CII",
        "facture_cii_profil_en16931",
        "urn:cen.eu:en16931:2017",
    );
    generate_facturx_with_profile(
        project_root, &output_dir,
        "tests/fixtures/ubl/facture_ubl_001.xml", "UBL",
        "facture_ubl_profil_en16931",
        "urn:cen.eu:en16931:2017",
    );
    // Extended (profil par défaut des fixtures)
    generate_facturx_with_profile(
        project_root, &output_dir,
        "tests/fixtures/cii/facture_cii_001.xml", "CII",
        "facture_cii_profil_extended",
        "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
    );

    println!("\n=== Tous les PDFs sont dans {} ===", output_dir.display());
}

fn generate_pdf(project_root: &Path, output_dir: &Path, xml_path: &str, name: &str, format: &str) {
    let xml_file = project_root.join(xml_path);
    let xml = match fs::read_to_string(&xml_file) {
        Ok(x) => x,
        Err(e) => {
            println!("  SKIP {} ({}): {}", name, xml_file.display(), e);
            return;
        }
    };

    let invoice = match parse_invoice(&xml, format) {
        Ok(inv) => inv,
        Err(e) => {
            println!("  ERREUR parsing {} {}: {}", format, name, e);
            return;
        }
    };

    // PDF visuel via Typst
    let typst = pdp_transform::TypstPdfEngine::from_manifest_dir();
    let start = std::time::Instant::now();
    match typst.generate_pdf(&invoice) {
        Ok(pdf) => {
            let elapsed = start.elapsed();
            let pdf_path = output_dir.join(format!("{}.pdf", name));
            fs::write(&pdf_path, &pdf).expect("Écriture PDF");
            println!(
                "  OK  {} ({}) → {} ({} bytes, {:.0}ms)",
                name, format, pdf_path.display(), pdf.len(), elapsed.as_secs_f64() * 1000.0
            );
        }
        Err(e) => {
            println!("  ERREUR PDF {} {}: {}", format, name, e);
        }
    }

    // Factur-X
    let facturx = pdp_transform::FacturXGenerator::from_manifest_dir();
    let start = std::time::Instant::now();
    match facturx.generate(&invoice) {
        Ok(result) => {
            let elapsed = start.elapsed();
            let fx_path = output_dir.join(format!("{}_facturx.pdf", name));
            fs::write(&fx_path, &result.pdf).expect("Écriture Factur-X");
            println!(
                "  OK  {} (Factur-X {}) → {} ({} bytes, {:.0}ms)",
                name, result.level, fx_path.display(), result.pdf.len(), elapsed.as_secs_f64() * 1000.0
            );
        }
        Err(e) => {
            println!("  WARN Factur-X {} {}: {}", format, name, e);
        }
    }
}

fn generate_facturx_with_attachments(
    project_root: &Path,
    output_dir: &Path,
    xml_path: &str,
    format: &str,
    output_name: &str,
    attachments: Vec<InvoiceAttachment>,
) {
    let xml_file = project_root.join(xml_path);
    let xml = match fs::read_to_string(&xml_file) {
        Ok(x) => x,
        Err(e) => {
            println!("  SKIP {} ({}): {}", output_name, xml_file.display(), e);
            return;
        }
    };

    let mut invoice = match parse_invoice(&xml, format) {
        Ok(inv) => inv,
        Err(e) => {
            println!("  ERREUR parsing {} {}: {}", format, output_name, e);
            return;
        }
    };

    let n_attachments = attachments.len();
    invoice.attachments.extend(attachments);

    let facturx = pdp_transform::FacturXGenerator::from_manifest_dir();
    let start = std::time::Instant::now();
    match facturx.generate(&invoice) {
        Ok(result) => {
            let elapsed = start.elapsed();
            let fx_path = output_dir.join(format!("{}.pdf", output_name));
            fs::write(&fx_path, &result.pdf).expect("Écriture");
            println!(
                "  OK  {} → {} ({} bytes, {:.0}ms, {} PJ, profil {})",
                output_name, fx_path.display(), result.pdf.len(),
                elapsed.as_secs_f64() * 1000.0, n_attachments, result.level
            );
        }
        Err(e) => {
            println!("  ERREUR {}: {}", output_name, e);
        }
    }
}

fn generate_facturx_with_profile(
    project_root: &Path,
    output_dir: &Path,
    xml_path: &str,
    format: &str,
    output_name: &str,
    profile_uri: &str,
) {
    let xml_file = project_root.join(xml_path);
    let xml = match fs::read_to_string(&xml_file) {
        Ok(x) => x,
        Err(e) => {
            println!("  SKIP {} ({}): {}", output_name, xml_file.display(), e);
            return;
        }
    };

    let mut invoice = match parse_invoice(&xml, format) {
        Ok(inv) => inv,
        Err(e) => {
            println!("  ERREUR parsing {} {}: {}", format, output_name, e);
            return;
        }
    };

    // Remplacer le profile_id dans le raw_xml
    if let Some(ref raw) = invoice.raw_xml {
        let new_xml = raw.replace(
            "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
            profile_uri,
        );
        invoice.raw_xml = Some(new_xml);
    }
    invoice.profile_id = Some(profile_uri.to_string());

    let facturx = pdp_transform::FacturXGenerator::from_manifest_dir();
    let start = std::time::Instant::now();
    match facturx.generate(&invoice) {
        Ok(result) => {
            let elapsed = start.elapsed();
            let fx_path = output_dir.join(format!("{}.pdf", output_name));
            fs::write(&fx_path, &result.pdf).expect("Écriture");
            println!(
                "  OK  {} → {} ({} bytes, {:.0}ms, profil {})",
                output_name, fx_path.display(), result.pdf.len(),
                elapsed.as_secs_f64() * 1000.0, result.level
            );
        }
        Err(e) => {
            println!("  ERREUR {}: {}", output_name, e);
        }
    }
}

fn parse_invoice(xml: &str, format: &str) -> Result<pdp_core::model::InvoiceData, String> {
    match format {
        "CII" => pdp_invoice::CiiParser::new().parse(xml).map_err(|e| e.to_string()),
        "UBL" => pdp_invoice::UblParser::new().parse(xml).map_err(|e| e.to_string()),
        _ => Err(format!("Format inconnu: {}", format)),
    }
}

/// Génère un vrai petit PDF valide via lopdf.
fn make_valid_pdf(title: &str) -> Vec<u8> {
    use lopdf::{Document, Stream};
    use lopdf::dictionary;

    let mut doc = Document::with_version("1.4");
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let content = format!("BT /F1 14 Tf 50 750 Td ({}) Tj ET", title);
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources_id,
    });
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
    });
    doc.get_object_mut(page_id).unwrap()
        .as_dict_mut().unwrap()
        .set("Parent", pages_id);
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    let mut buf = Vec::new();
    doc.save_to(&mut buf).expect("Génération PDF");
    buf
}

fn make_pdf_attachment(id: &str, desc: &str, filename: &str) -> InvoiceAttachment {
    InvoiceAttachment {
        id: Some(id.to_string()),
        description: Some(desc.to_string()),
        external_uri: None,
        embedded_content: Some(make_valid_pdf(desc)),
        mime_code: Some("application/pdf".to_string()),
        filename: Some(filename.to_string()),
    }
}

fn make_csv_attachment(id: &str, desc: &str, filename: &str, content: &str) -> InvoiceAttachment {
    InvoiceAttachment {
        id: Some(id.to_string()),
        description: Some(desc.to_string()),
        external_uri: None,
        embedded_content: Some(content.as_bytes().to_vec()),
        mime_code: Some("text/csv".to_string()),
        filename: Some(filename.to_string()),
    }
}

fn make_text_attachment(id: &str, desc: &str, filename: &str, content: &str) -> InvoiceAttachment {
    InvoiceAttachment {
        id: Some(id.to_string()),
        description: Some(desc.to_string()),
        external_uri: None,
        embedded_content: Some(content.as_bytes().to_vec()),
        mime_code: Some("text/plain".to_string()),
        filename: Some(filename.to_string()),
    }
}

fn make_uri_attachment(id: &str, desc: &str, uri: &str) -> InvoiceAttachment {
    InvoiceAttachment {
        id: Some(id.to_string()),
        description: Some(desc.to_string()),
        external_uri: Some(uri.to_string()),
        embedded_content: None,
        mime_code: None,
        filename: None,
    }
}
