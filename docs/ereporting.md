# E-reporting (Flux 10.1, 10.2, 10.3, 10.4)

Ferrite génère les rapports e-reporting conformes au XSD PPF V1.0 selon
**XP Z12-012 §1.5** et les **règles BR-FR-MAP** (Spécifications Externes
DSE AIFE).

## Flux supportés

| Flux | Description | Méthode | Code interface PPF |
|------|-------------|---------|---|
| **10.1** | Transactions ventes détaillées | `EReportingGenerator::create_transactions_report` | `FFE1025A` |
| **10.2** | Paiements ventes par facture | `EReportingGenerator::create_payments_report` | `FFE1025A` |
| **10.3** | Transactions agrégées (catégories TLB1/TPS1/TNT1/TMA1) | `EReportingGenerator::create_aggregated_transactions_report` | `FFE1025A` |
| **10.4** | Paiements agrégés | `EReportingGenerator::create_aggregated_payments_report` | `FFE1025A` |

## Règles BR-FR-MAP appliquées

| Règle | Description | Implémentation |
|-------|-------------|----------------|
| **BR-FR-MAP-01** | Cadre de facturation depuis BT-23 ou note BAR | `derive_business_process()` |
| **BR-FR-MAP-04** | Mapper notes (BAR, PMT, PMD, AAB, TXD) | `invoice_to_transaction` |
| **BR-FR-MAP-06** | Référence à facture antérieure | `referenced_documents` |
| **BR-FR-MAP-08** | Vendeur avec TVA, pays, schemeId | `seller` |
| **BR-FR-MAP-10** | Acheteur idem | `buyer` |
| **BR-FR-MAP-12** | Représentant fiscal vendeur | `seller_tax_representative` |
| **BR-FR-MAP-14** | Livraison | `deliveries` |
| **BR-FR-MAP-15** | Période de facturation | `invoice_period` |
| **BR-FR-MAP-16** | Remises/charges niveau document | `allowance_charges` |
| **BR-FR-MAP-17/18/19** | Lignes, totaux, ventilation TVA | `lines`, `monetary_total`, `tax_subtotals` |
| **BR-FR-MAP-23** | Date au format `YYYYMMDD` | `normalize_date_yyyymmdd()` partout |

## CLI

### Générer un rapport 10.1 (transactions ventes détaillées)

**Source 1 — répertoire local** :

```bash
pdp ereporting generate101 \
  --invoices-dir ./factures-novembre \
  --siren 123456789 \
  --name "ACME SAS" \
  --from 2025-11-01 \
  --to 2025-11-30 \
  --output rapport-10.1-novembre.xml
```

Le répertoire est scanné pour les fichiers `.xml` (UBL/CII) et `.pdf`
(Factur-X). Format détecté automatiquement.

**Source 2 — Elasticsearch (`pdp-{siren}`)** : omettre `--invoices-dir` pour
que Ferrite aille chercher les factures **directement dans son propre index ES**
(toutes les factures avec `status = DISTRIBUÉ` sur la période). Pas besoin de
préparer un répertoire :

```bash
pdp --config config.yaml ereporting generate101 \
  --siren 123456789 \
  --name "ACME SAS" \
  --from 2025-11-01 \
  --to 2025-11-30 \
  --output rapport-10.1-novembre.xml

# 📊 1247 factures trouvées dans pdp-123456789 sur la période 2025-11-01..2025-11-30
# ✅ Rapport écrit dans rapport-10.1-novembre.xml
```

Les factures sont reconstruites depuis `raw_xml` (UBL/CII parsers
automatiques selon `source_format`). Idéal pour automatiser la déclaration
mensuelle via cron.

### Générer un rapport 10.3 (transactions agrégées)

```bash
# Depuis un répertoire local
pdp ereporting generate103 \
  --invoices-dir ./factures-novembre \
  --siren 123456789 \
  --name "ACME SAS" \
  --from 2025-11-01 \
  --to 2025-11-30 \
  --output rapport-10.3-novembre.xml

# Ou depuis Elasticsearch (omettre --invoices-dir)
pdp --config config.yaml ereporting generate103 \
  --siren 123456789 --name "ACME SAS" \
  --from 2025-11-01 --to 2025-11-30 \
  --output rapport-10.3-novembre.xml
```

L'agrégation se fait par `(date, catégorie, taux TVA)` :
- **TLB1** (Livraison de biens) ou **TPS1** (Prestations) selon les lignes
- **TNT1** si non taxable en France
- **TMA1** si régime de la marge

### Sortie stdout

Si `--output` est omis, le XML est affiché sur stdout (pratique pour piper
vers `xmllint --format -` ou `xmlstarlet`).

## Utilisation programmatique

### Flux 10.1 (transactions ventes)

```rust
use pdp_ereporting::EReportingGenerator;
use pdp_invoice::UblParser;

let invoice = UblParser::new().parse(&xml)?;

let gen = EReportingGenerator::new("123456789", "ACME SAS");
let transactions = vec![EReportingGenerator::invoice_to_transaction(&invoice)];

let report = gen.create_transactions_report(
    "RPT-2025-11",
    "123456789",
    "ACME SAS",
    "2025-11-01",   // accepte aussi "20251101"
    "2025-11-30",
    transactions,
);

let xml = gen.to_xml(&report)?;
```

### Flux 10.2 (paiements)

```rust
use pdp_ereporting::{EReportingGenerator, PaymentSubTotal};

let gen = EReportingGenerator::new("123456789", "ACME SAS");

let payment = EReportingGenerator::payment_invoice(
    "F-2025-001",
    "2025-11-15",   // BR-FR-MAP-23 : normalisé en 20251115
    "2025-12-10",   // date paiement
    vec![PaymentSubTotal {
        tax_percent: 20.0,
        currency_code: Some("EUR".into()),
        amount: 12000.0,
    }],
);

let report = gen.create_payments_report(
    "RPT-PAY-2025-11",
    "123456789",
    "ACME SAS",
    "2025-11-01",
    "2025-11-30",
    vec![payment],
);
```

### Flux 10.3 (agrégation)

```rust
let invoices: Vec<InvoiceData> = /* ... factures émises ... */;
let report = gen.create_aggregated_transactions_report(
    "RPT-AGG-2025-11",
    "123456789",
    "ACME SAS",
    "2025-11-01",
    "2025-11-30",
    &invoices,
)?;
```

### Flux 10.4 (paiements agrégés)

```rust
let txn = EReportingGenerator::payment_transaction(
    "2025-12-10",
    vec![PaymentSubTotal { tax_percent: 20.0, currency_code: Some("EUR".into()), amount: 50000.0 }],
);
let report = gen.create_aggregated_payments_report(
    "RPT-AGG-PAY-2025-11",
    "123456789",
    "ACME SAS",
    "2025-11-01",
    "2025-11-30",
    vec![txn],
);
```

## Conformité dates (BR-FR-MAP-23)

Toutes les dates en entrée acceptent **deux formats** indifféremment :

```rust
EReportingGenerator::normalize_date_yyyymmdd("2025-11-15")          // → "20251115"
EReportingGenerator::normalize_date_yyyymmdd("20251115")            // → "20251115"
EReportingGenerator::normalize_date_yyyymmdd("2025-11-15T10:30:00") // → "20251115" (heure tronquée)
```

Appliqué automatiquement dans :
- `invoice_to_transaction` : `issue_date`, `due_date`
- `create_transactions_report` : `period_start`, `period_end`
- `create_aggregated_transactions_report` : idem + `aggregated.date`
- `create_payments_report` : `period_start`, `period_end`
- `create_aggregated_payments_report` : idem
- `payment_invoice` / `payment_transaction` : `issue_date`, `payment.date`

## Envoi au PPF

Le code interface PPF pour les flux 10.x est `FFE1025A` (déjà défini dans
`pdp-client/ppf.rs`). Le rapport XML peut être empaqueté en tar.gz via
`PpfSftpProducer` (voir [docs/ppf-afnor.md](ppf-afnor.md)).

```rust
use pdp_client::ppf::CodeInterface;

let xml = gen.to_xml(&report)?;
let archive = PpfSftpProducer::build_archive(
    CodeInterface::F10TransactionPaiement,
    "AAA001",       // séquence PPF
    "rapport.xml",
    xml.as_bytes(),
)?;
// Dépôt SFTP sur SAS PPF
```

## Voir aussi

- [docs/ppf-afnor.md](ppf-afnor.md) — codes interface PPF, SFTP
- [docs/specifications.md](specifications.md) — vue d'ensemble des specs
- `crates/pdp-ereporting/src/generator.rs` — code source
