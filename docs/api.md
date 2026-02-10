# API de conversion

La librairie `pdp-transform` fournit une API unifiée pour convertir entre tous les formats de facture.

## Matrice de conversion

| Source ↓ / Cible → | CII | UBL | Factur-X | PDF |
|---------------------|-----|-----|----------|-----|
| **UBL**             | ✅ XSLT | — | ✅ XSLT+FOP+lopdf | ✅ FOP |
| **CII**             | — | ✅ XSLT | ✅ FOP+lopdf | ✅ FOP |
| **Factur-X**        | ✅ extraction | ✅ extraction+XSLT | — | ✅ retourne PDF |

## Utilisation rapide

```rust
use pdp_transform::{convert_to, OutputFormat};
use pdp_invoice::ubl::UblParser;

// 1. Parser la facture source
let xml = std::fs::read_to_string("facture.xml").unwrap();
let invoice = UblParser::new().parse(&xml).unwrap();

// 2. Convertir vers le format souhaité
let result = convert_to(&invoice, OutputFormat::CII).unwrap();

// 3. Utiliser le résultat
println!("Format: {}", result.output_format);       // "CII"
println!("Fichier: {}", result.suggested_filename);  // "FA-001_cii.xml"
let xml = result.as_string().unwrap();               // Contenu XML
```

## Formats de sortie (`OutputFormat`)

| Format | Description | Extension |
|--------|-------------|-----------|
| `OutputFormat::UBL` | XML UBL 2.1 | `_ubl.xml` |
| `OutputFormat::CII` | XML CII D22B | `_cii.xml` |
| `OutputFormat::FacturX` | PDF/A-3 avec XML CII embarqué + pièces jointes | `_facturx.pdf` |
| `OutputFormat::PDF` | PDF visuel seul (sans XML embarqué) | `.pdf` |

## Exemples par conversion

### UBL → CII (transformation XSLT)

```rust
let invoice = UblParser::new().parse(&ubl_xml).unwrap();
let result = convert_to(&invoice, OutputFormat::CII).unwrap();
let cii_xml = result.as_string().unwrap();
```

### UBL → Factur-X (PDF/A-3 avec XML embarqué)

```rust
let invoice = UblParser::new().parse(&ubl_xml).unwrap();
let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
std::fs::write("facture.pdf", &result.content).unwrap();
// Le PDF contient factur-x.xml embarqué + métadonnées XMP
```

### UBL → PDF (visuel seul, sans XML embarqué)

```rust
let invoice = UblParser::new().parse(&ubl_xml).unwrap();
let result = convert_to(&invoice, OutputFormat::PDF).unwrap();
std::fs::write("facture_visuel.pdf", &result.content).unwrap();
```

### CII → UBL

```rust
let invoice = CiiParser::new().parse(&cii_xml).unwrap();
let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
let ubl_xml = result.as_string().unwrap();
```

### Factur-X → CII (extraction du XML embarqué)

```rust
let pdf = std::fs::read("facturx.pdf").unwrap();
let invoice = FacturXParser::new().parse(&pdf).unwrap();
let result = convert_to(&invoice, OutputFormat::CII).unwrap();
let cii_xml = result.as_string().unwrap();
```

### Factur-X → UBL (extraction + transformation XSLT)

```rust
let pdf = std::fs::read("facturx.pdf").unwrap();
let invoice = FacturXParser::new().parse(&pdf).unwrap();
let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
let ubl_xml = result.as_string().unwrap();
```

## Pièces jointes (BG-24)

Les pièces jointes sont modélisées par `InvoiceAttachment` et automatiquement :
- **embarquées dans le XML** (CII/UBL) en base64 via `AdditionalReferencedDocument` / `AdditionalDocumentReference`
- **embarquées dans le PDF Factur-X** avec `AFRelationship=Supplement`
- **extraites du PDF Factur-X** lors du parsing (hors `factur-x.xml`)

### Ajouter une pièce jointe à une facture

```rust
use pdp_core::model::InvoiceAttachment;

let mut invoice = UblParser::new().parse(&ubl_xml).unwrap();

invoice.attachments.push(InvoiceAttachment {
    id: Some("ATT-001".to_string()),
    description: Some("Bon de commande".to_string()),
    external_uri: None,
    embedded_content: Some(std::fs::read("bon_commande.pdf").unwrap()),
    mime_code: Some("application/pdf".to_string()),
    filename: Some("bon_commande.pdf".to_string()),
});
```

### Plusieurs pièces jointes de types différents

```rust
use pdp_core::model::InvoiceAttachment;

let mut invoice = CiiParser::new().parse(&cii_xml).unwrap();

// PDF : bon de commande
invoice.attachments.push(InvoiceAttachment {
    id: Some("BC-2025-042".to_string()),
    description: Some("Bon de commande".to_string()),
    external_uri: None,
    embedded_content: Some(std::fs::read("bon_commande.pdf").unwrap()),
    mime_code: Some("application/pdf".to_string()),
    filename: Some("bon_commande.pdf".to_string()),
});

// Image : photo du bordereau de livraison
invoice.attachments.push(InvoiceAttachment {
    id: Some("BL-2025-042".to_string()),
    description: Some("Bordereau de livraison".to_string()),
    external_uri: None,
    embedded_content: Some(std::fs::read("bordereau.png").unwrap()),
    mime_code: Some("image/png".to_string()),
    filename: Some("bordereau_livraison.png".to_string()),
});

// CSV : détail des lignes
invoice.attachments.push(InvoiceAttachment {
    id: Some("DET-001".to_string()),
    description: Some("Détail des lignes".to_string()),
    external_uri: None,
    embedded_content: Some(b"ref;qte;pu\nA001;10;25.00\nA002;5;12.50".to_vec()),
    mime_code: Some("text/csv".to_string()),
    filename: Some("detail_lignes.csv".to_string()),
});

// Référence externe (pas de contenu embarqué)
invoice.attachments.push(InvoiceAttachment {
    id: Some("SPEC-001".to_string()),
    description: Some("Cahier des charges".to_string()),
    external_uri: Some("https://example.com/specs/cahier_charges.pdf".to_string()),
    embedded_content: None,
    mime_code: None,
    filename: None,
});
```

### Conversion avec pièces jointes

```rust
// UBL + PJ → Factur-X (PJ embarquées dans le PDF avec AFRelationship=Supplement)
let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
std::fs::write("facture_avec_pj.pdf", &result.content).unwrap();

// UBL + PJ → CII (PJ encodées en base64 dans le XML)
let result = convert_to(&invoice, OutputFormat::CII).unwrap();
let cii_xml = result.as_string().unwrap();
// Le XML contient <ram:AdditionalReferencedDocument> avec <ram:AttachmentBinaryObject>

// CII + PJ → UBL (PJ encodées en base64 dans le XML)
let result = convert_to(&invoice, OutputFormat::UBL).unwrap();
let ubl_xml = result.as_string().unwrap();
// Le XML contient <cac:AdditionalDocumentReference> avec <cbc:EmbeddedDocumentBinaryObject>
```

### Lire les pièces jointes d'une facture parsée

```rust
// Depuis un UBL ou CII
let invoice = UblParser::new().parse(&xml).unwrap();
for att in &invoice.attachments {
    println!("PJ: {} ({}) - {} octets",
        att.filename.as_deref().unwrap_or("sans nom"),
        att.mime_code.as_deref().unwrap_or("inconnu"),
        att.embedded_content.as_ref().map(|c| c.len()).unwrap_or(0),
    );
}

// Depuis un Factur-X (les PJ sont extraites du PDF)
let pdf = std::fs::read("facturx_avec_pj.pdf").unwrap();
let invoice = FacturXParser::new().parse(&pdf).unwrap();
for att in &invoice.attachments {
    if let Some(ref content) = att.embedded_content {
        let filename = att.filename.as_deref().unwrap_or("piece_jointe");
        std::fs::write(filename, content).unwrap();
    }
}
```

### Archiver des factures avec pièces jointes

```rust
use pdp_core::archive::ZipBuilder;

let mut zip = ZipBuilder::new();

// Ajouter la facture
zip = zip.add("facture_001.xml", cii_xml.as_bytes());

// Ajouter les pièces jointes
for att in &invoice.attachments {
    if let (Some(ref filename), Some(ref content)) = (&att.filename, &att.embedded_content) {
        zip = zip.add(&format!("pj/{}", filename), content);
    }
}

let archive = zip.build().unwrap();
std::fs::write("facture_avec_pj.zip", &archive).unwrap();
```

## Méthodes utilitaires

```rust
use pdp_transform::{supported_output_formats, OutputFormat};
use pdp_core::model::InvoiceFormat;

// Formats de sortie disponibles depuis UBL
let formats = supported_output_formats(&InvoiceFormat::UBL);
// → [CII, FacturX, PDF]

// Vérifier si le résultat est un PDF
let result = convert_to(&invoice, OutputFormat::FacturX).unwrap();
assert!(result.is_pdf());

// Obtenir le contenu XML (None pour PDF/Factur-X)
let xml: Option<String> = result.as_string();
```
