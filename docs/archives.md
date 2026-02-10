# Utilitaires d'archivage (ZIP / tar.gz)

Le module `pdp_core::archive` fournit des builders pour créer et lire des archives en mémoire.

## Décompression automatique à l'entrée

Le `FileEndpoint` (répertoire d'entrée) détecte et décompresse automatiquement les archives déposées :

| Extension | Format | Traitement |
|-----------|--------|------------|
| `.tar.gz` | tar gzip | Extraction en mémoire, un exchange par fichier |
| `.tgz` | tar gzip | Idem |
| `.zip` | ZIP | Extraction en mémoire, un exchange par fichier |
| autre | fichier brut | Lu directement comme un seul exchange |

Chaque fichier extrait d'une archive devient un `Exchange` individuel avec :
- `source_filename` = nom du fichier dans l'archive (sans le chemin)
- propriété `source_archive` = nom de l'archive d'origine
- Les répertoires dans les archives sont ignorés
- Les fichiers vides sont ignorés

```
data/in/
├── facture_directe.xml          → 1 exchange (facture_directe.xml)
├── lot_janvier.tar.gz           → N exchanges (un par fichier dans l'archive)
│   ├── facture_001.xml
│   ├── facture_002.xml
│   └── facture_003.xml
└── lot_fevrier.zip              → M exchanges (un par fichier dans l'archive)
    ├── facture_010.xml
    └── facture_011.xml
```

### Exemple de configuration

```yaml
routes:
  - id: route-reception
    source:
      type: file
      path: ./data/in     # Accepte XML, tar.gz, tgz, zip
    destination:
      type: file
      path: ./data/out
    validate: true
```

### Traçabilité

L'archive d'origine est tracée via la propriété `source_archive` de l'exchange :

```rust
if let Some(archive) = exchange.get_property("source_archive") {
    println!("Extrait de l'archive : {}", archive);
}
```

## Créer un tar.gz

```rust
use pdp_core::archive::TarGzBuilder;

let tgz = TarGzBuilder::new()
    .add("facture_001.xml", &xml_bytes)
    .add("facture_002.xml", &xml_bytes_2)
    .compression_level(9) // 0-9, défaut: 6
    .build()
    .unwrap();

std::fs::write("flux.tar.gz", &tgz).unwrap();
```

## Créer un ZIP

```rust
use pdp_core::archive::ZipBuilder;

let zip = ZipBuilder::new()
    .add("facture_001.xml", &xml_bytes)
    .add("pieces_jointes/bon_commande.pdf", &pdf_bytes)
    .build()
    .unwrap();

std::fs::write("archive.zip", &zip).unwrap();
```

## Depuis des fichiers sur disque

```rust
use pdp_core::archive::{TarGzBuilder, ZipBuilder};

// tar.gz depuis fichiers
let tgz = TarGzBuilder::new()
    .add_file("data/facture.xml")?
    .add_file_as("Base_fa_001.xml", "data/facture.xml")?
    .build()?;

// ZIP depuis fichiers
let zip = ZipBuilder::new()
    .add_file("data/facture.xml")?
    .no_compression() // ou .deflate() (défaut)
    .build()?;
```

## Lire une archive

```rust
use pdp_core::archive::{read_tar_gz, read_zip};

// Lire un tar.gz
let entries = read_tar_gz(&tgz_bytes).unwrap();
for entry in &entries {
    println!("{}: {} octets", entry.filename, entry.content.len());
}

// Lire un ZIP
let entries = read_zip(&zip_bytes).unwrap();
```
