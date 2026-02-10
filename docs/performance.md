# Performances (benchmarks Criterion)

Mesuré sur macOS ARM64 (Apple Silicon), facture simple 3 lignes, `cargo bench`.

## Parsing (pdp-invoice)

| Format | Temps | Débit |
|--------|-------|-------|
| UBL (roxmltree) | 26 µs | **~38 000 factures/s** |
| CII (roxmltree) | 30 µs | **~33 000 factures/s** |
| Factur-X PDF (lopdf + roxmltree) | 644 µs | **~1 550 factures/s** |
| Détection auto format | < 3 ns | — |

## Validation (pdp-validate)

| Étape | CII | UBL |
|-------|-----|-----|
| XSD (libxml2) | 12 ms (~83/s) | 5 ms (~200/s) |
| Schematron EN16931 (SaxonC natif) | 85 ms | 122 ms |
| Schematron BR-FR (SaxonC natif) | 35 ms | 37 ms |
| Schematron EN16931 + BR-FR (**parallèle**) | 92 ms | 124 ms |
| **Validation complète** (XSD + Schematron //) | **101 ms (~10/s)** | **131 ms (~7.6/s)** |

> SaxonC natif (`transform`) est ~10× plus rapide que SaxonJ (`saxon`) grâce à l'absence de JVM.
> EN16931 et BR-FR s'exécutent en parallèle (`std::thread::scope`), fichier temporaire partagé.

## Validation Factur-X (pdp-validate)

| Étape                                                               | Temps             |
| ---------------------------------------------------------------------| -------------------|
| XSD Factur-X EN16931 (libxml2)                                      | 0.56 ms           |
| Schematron EN16931-CII (SaxonC)                                     | 86 ms             |
| Schematron Factur-X EN16931 (SaxonC)                                | 113 ms            |
| Schematron BR-FR CII (SaxonC)                                       | 35 ms             |
| **Validation complète Factur-X** (XSD + EN16931 + Factur-X + BR-FR) | **200 ms (~5/s)** |

## Transformation (pdp-transform)

| Conversion                             | Temps  | Débit            |
| ----------------------------------------| --------| ------------------|
| UBL → CII (XSLT SaxonC)                | 17 ms  | **~60/s**        |
| CII → UBL (XSLT SaxonC)                | 21 ms  | **~48/s**        |
| Roundtrip UBL → CII → UBL              | 38 ms  | ~26/s            |
| Factur-X → CII (extraction)            | 253 ns | **~4 000 000/s** |
| Factur-X → UBL (extraction + XSLT)     | 24 ms  | **~42/s**        |
| Factur-X → PDF (retourne PDF existant) | 2.2 µs | **~450 000/s**   |
| UBL → PDF (SaxonC FFI + FOP)           | 1.44 s | **~0.69/s**      |
| CII → PDF (SaxonC FFI + FOP)           | 1.53 s | **~0.65/s**      |
| UBL → Factur-X (SaxonC FFI + FOP + lopdf) | 1.48 s | **~0.68/s**   |
| CII → Factur-X (SaxonC FFI + FOP + lopdf) | 1.51 s | **~0.66/s**   |

> XSLT UBL↔CII : SaxonC natif est **~30× plus rapide** que SaxonJ (17 ms vs 490 ms).
> PDF/Factur-X : **SaxonC FFI in-process** (pas de fork/exec) réduit la pipeline de **~25-30%** par rapport au CLI.
> Le goulot restant est Apache FOP (~1 s), pas XSLT.

## Pipeline complet (parse + XSD + Schematron + transform)

| Scénario | Temps estimé |
|----------|-------------|
| Réception CII : parse + validation + transform → UBL | **~122 ms (~8/s)** |
| Réception UBL : parse + validation + transform → CII | **~148 ms (~7/s)** |
| Réception CII → Factur-X (validate + SaxonC FFI + FOP + lopdf) | **~1.6 s** |
| Réception UBL → Factur-X (validate + SaxonC FFI + FOP + lopdf) | **~1.6 s** |
| Réception Factur-X → CII (parse PDF + validate + extract) | ~200 ms |
| Réception Factur-X → UBL (parse PDF + validate + XSLT) | ~224 ms |

## Décompression archives (pdp-core)

| Source | 1 fichier | 10 fichiers | 50 fichiers | 100 fichiers |
|--------|-----------|-------------|-------------|--------------|
| Fichiers XML directs | 28 µs | 169 µs | 783 µs | 1.57 ms |
| Archive tar.gz | 34 µs | 63 µs | 179 µs | 333 µs |
| Archive ZIP | 40 µs | 124 µs | 503 µs | 901 µs |
| Mixte (5 XML + 10 tar.gz + 10 zip = 25) | — | — | — | 248 µs |

> tar.gz est **~3× plus rapide** que les fichiers directs pour 100 factures (333 µs vs 1.57 ms) grâce à une seule lecture disque.
> ZIP est **~1.7× plus rapide** que les fichiers directs pour 100 factures.
> Le coût de décompression est largement compensé par la réduction des I/O disque.

## Lancer les benchmarks

```bash
# Tous les benchmarks
cargo bench --workspace

# Par crate
cargo bench -p pdp-core        # archives (tar.gz, zip, mixte)
cargo bench -p pdp-invoice
cargo bench -p pdp-validate
cargo bench -p pdp-transform
cargo bench -p pdp-cdar
```
