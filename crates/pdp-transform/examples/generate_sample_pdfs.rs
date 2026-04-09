//! Génère des exemples de factures PDF à partir de fichiers CII et UBL.
//!
//! Usage: cargo run -p pdp-transform --example generate_sample_pdfs

use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let output_dir = project_root.join("output/sample_pdfs");
    fs::create_dir_all(&output_dir).expect("Impossible de créer output/sample_pdfs");

    println!("=== Génération de factures PDF via Typst ===\n");

    // --- CII ---
    let cii_files = [
        ("tests/fixtures/cii/facture_cii_001.xml", "facture_cii_001"),
        ("tests/fixtures/cii/avoir_cii_381.xml", "avoir_cii_381"),
        ("tests/fixtures/cii/facture_cii_remises_multitva.xml", "facture_cii_remises_multitva"),
    ];

    for (path, name) in &cii_files {
        generate_pdf(project_root, &output_dir, path, name, "CII");
    }

    // --- UBL ---
    let ubl_files = [
        ("tests/fixtures/ubl/facture_ubl_001.xml", "facture_ubl_001"),
        ("tests/fixtures/ubl/facture_ubl_002_avoir.xml", "avoir_ubl_002"),
        ("tests/fixtures/ubl/facture_ubl_remises_multitva.xml", "facture_ubl_remises_multitva"),
    ];

    for (path, name) in &ubl_files {
        generate_pdf(project_root, &output_dir, path, name, "UBL");
    }

    // --- Factur-X avec attachements ---
    generate_facturx_with_attachments(project_root, &output_dir);

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

    // Parser le XML
    let invoice = match format {
        "CII" => pdp_invoice::CiiParser::new().parse(&xml),
        "UBL" => pdp_invoice::UblParser::new().parse(&xml),
        _ => unreachable!(),
    };

    let invoice = match invoice {
        Ok(inv) => inv,
        Err(e) => {
            println!("  ERREUR parsing {} {}: {}", format, name, e);
            return;
        }
    };

    // Générer le PDF via Typst
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

    // Aussi générer en Factur-X
    let facturx = pdp_transform::FacturXGenerator::from_manifest_dir();
    let start = std::time::Instant::now();
    match facturx.generate(&invoice) {
        Ok(result) => {
            let elapsed = start.elapsed();
            let fx_path = output_dir.join(format!("{}_facturx.pdf", name));
            fs::write(&fx_path, &result.pdf).expect("Écriture Factur-X");
            println!(
                "  OK  {} (Factur-X) → {} ({} bytes, {:.0}ms)",
                name, fx_path.display(), result.pdf.len(), elapsed.as_secs_f64() * 1000.0
            );
        }
        Err(e) => {
            println!("  WARN Factur-X {} {}: {}", format, name, e);
        }
    }
}

fn generate_facturx_with_attachments(project_root: &Path, output_dir: &Path) {
    println!("\n--- Factur-X avec pièces jointes ---");

    let xml_file = project_root.join("tests/fixtures/cii/facture_cii_001.xml");
    let xml = fs::read_to_string(&xml_file).expect("Lecture CII 001");
    let mut invoice = pdp_invoice::CiiParser::new().parse(&xml).expect("Parse CII");

    // Ajouter des pièces jointes
    use pdp_core::model::InvoiceAttachment;

    // PJ 1 : un vrai petit PDF (bon de commande) généré via lopdf
    let bon_commande_pdf = {
        use lopdf::{Document, Object, Stream, StringFormat};
        use lopdf::dictionary;

        let mut doc = Document::with_version("1.4");

        // Police Helvetica
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });

        // Contenu de la page
        let content = b"BT /F1 16 Tf 50 750 Td (Bon de commande BC-001) Tj ET\n\
            BT /F1 12 Tf 50 700 Td (Client: Ma Societe SAS) Tj ET\n\
            BT /F1 12 Tf 50 670 Td (Date: 2025-01-15) Tj ET\n\
            BT /F1 12 Tf 50 640 Td (Ref: Prestation de conseil informatique) Tj ET";
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.to_vec()));

        // Resources
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });

        // Page
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Contents" => content_id,
            "Resources" => resources_id,
        });

        // Pages
        let pages_id = doc.add_object(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        });

        // Mettre à jour le Parent de la page
        doc.get_object_mut(page_id).unwrap()
            .as_dict_mut().unwrap()
            .set("Parent", pages_id);

        // Catalog
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });

        doc.trailer.set("Root", catalog_id);

        let mut buf = Vec::new();
        doc.save_to(&mut buf).expect("Génération PDF bon de commande");
        buf
    };
    invoice.attachments.push(InvoiceAttachment {
        id: Some("BC-001".to_string()),
        description: Some("Bon de commande".to_string()),
        external_uri: None,
        embedded_content: Some(bon_commande_pdf),
        mime_code: Some("application/pdf".to_string()),
        filename: Some("bon_commande.pdf".to_string()),
    });

    // PJ 2 : un CSV de détail
    let csv_data = b"Article;Quantite;PU HT\nConseil;10;100.00\nFormation;5;100.00\n";
    invoice.attachments.push(InvoiceAttachment {
        id: Some("DETAIL-001".to_string()),
        description: Some("Détail des prestations".to_string()),
        external_uri: None,
        embedded_content: Some(csv_data.to_vec()),
        mime_code: Some("text/csv".to_string()),
        filename: Some("detail_prestations.csv".to_string()),
    });

    // PJ 3 : référence externe uniquement (pas embarquée)
    invoice.attachments.push(InvoiceAttachment {
        id: Some("DEVIS-001".to_string()),
        description: Some("Devis original".to_string()),
        external_uri: Some("https://example.com/devis/001.pdf".to_string()),
        embedded_content: None,
        mime_code: None,
        filename: None,
    });

    let facturx = pdp_transform::FacturXGenerator::from_manifest_dir();
    let start = std::time::Instant::now();
    match facturx.generate(&invoice) {
        Ok(result) => {
            let elapsed = start.elapsed();
            let fx_path = output_dir.join("facture_cii_001_avec_pj_facturx.pdf");
            fs::write(&fx_path, &result.pdf).expect("Écriture");
            println!(
                "  OK  facturx+PJ → {} ({} bytes, {:.0}ms, {} attachments)",
                fx_path.display(), result.pdf.len(), elapsed.as_secs_f64() * 1000.0,
                invoice.attachments.len()
            );
        }
        Err(e) => {
            println!("  ERREUR Factur-X+PJ: {}", e);
        }
    }
}
