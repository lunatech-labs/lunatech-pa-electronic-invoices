use std::collections::HashMap;

use chrono::Utc;
use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::InvoiceData;

use crate::model::*;

/// Générateur de rapports e-reporting conformes au XSD PPF V1.0
pub struct EReportingGenerator {
    /// SIREN de la PDP émettrice
    pub pdp_siren: String,
    /// Nom de la PDP émettrice
    pub pdp_name: String,
}

impl EReportingGenerator {
    pub fn new(pdp_siren: &str, pdp_name: &str) -> Self {
        Self {
            pdp_siren: pdp_siren.to_string(),
            pdp_name: pdp_name.to_string(),
        }
    }

    /// Crée un rapport de transactions (flux 10.1) à partir d'une liste de factures
    pub fn create_transactions_report(
        &self,
        report_id: &str,
        declarant_siren: &str,
        declarant_name: &str,
        period_start: &str,
        period_end: &str,
        invoices: Vec<TransactionInvoice>,
    ) -> EReport {
        let now = Utc::now();
        let dt = now.format("%Y%m%d%H%M%S").to_string();

        EReport {
            document: ReportDocument {
                id: report_id.to_string(),
                name: Some(format!("E-reporting_10.1_{}", report_id)),
                issue_datetime: dt,
                type_code: ReportTypeCode::TransactionsInitial,
                sender: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: self.pdp_siren.clone(),
                    name: self.pdp_name.clone(),
                    role_code: "WK".to_string(),
                    endpoint_uri: None,
                },
                issuer: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: declarant_siren.to_string(),
                    name: declarant_name.to_string(),
                    role_code: "SE".to_string(),
                    endpoint_uri: None,
                },
            },
            transactions: Some(TransactionsReport {
                period_start: period_start.to_string(),
                period_end: period_end.to_string(),
                invoices,
                aggregated_transactions: Vec::new(),
            }),
            payments: None,
        }
    }

    /// Convertit une InvoiceData en TransactionInvoice pour le e-reporting.
    ///
    /// Implémente les règles BR-FR-MAP pour la construction du flux 10.1.
    pub fn invoice_to_transaction(invoice: &InvoiceData) -> TransactionInvoice {
        let seller_siret = invoice.seller_siret.as_deref().unwrap_or("");
        let seller_siren = if seller_siret.len() >= 9 { &seller_siret[..9] } else { seller_siret };

        let buyer_siret = invoice.buyer_siret.as_deref().unwrap_or("");
        let buyer_siren = if buyer_siret.len() >= 9 { &buyer_siret[..9] } else { buyer_siret };

        // BR-FR-MAP-01 : Dérive le cadre de facturation depuis BT-23 ou la note BAR
        let business_process_id = Self::derive_business_process(invoice);

        // BR-FR-MAP-04 : Mapper les notes (incluant BAR, PMT, PMD, AAB, TXD)
        let notes: Vec<InvoiceNote> = invoice.notes.iter().map(|n| InvoiceNote {
            subject: n.subject_code.clone(),
            content: Some(n.content.clone()),
        }).collect();

        // BR-FR-MAP-06 : Mapper la référence à la facture antérieure
        let referenced_documents: Vec<ReferencedInvoice> = invoice.preceding_invoice_reference
            .as_ref()
            .map(|ref_id| vec![ReferencedInvoice {
                id: ref_id.clone(),
                issue_date: invoice.preceding_invoice_date.clone().map(|d| d.replace('-', "")),
            }])
            .unwrap_or_default();

        // BR-FR-MAP-08 + BR-FR-MAP-16 : Mapper le vendeur avec TVA, pays et schéma d'identifiant
        let seller_country = invoice.seller_country.as_deref().or(Some("FR"));
        let seller_id_scheme = Self::id_scheme_for_country(seller_country);
        let seller = TransactionParty {
            company_id: Some(seller_siren.to_string()),
            company_id_scheme: Some(seller_id_scheme.to_string()),
            tax_registration_id: invoice.seller_vat_id.clone(),
            tax_qualifying_id: invoice.seller_vat_id.as_ref().map(|_| "VA".to_string()),
            country_code: seller_country.map(|s| s.to_string()),
        };

        // BR-FR-MAP-10 + BR-FR-MAP-16 : Mapper l'acheteur avec TVA, pays et schéma d'identifiant
        let buyer_country = invoice.buyer_country.as_deref();
        let buyer_id_scheme = Self::id_scheme_for_country(buyer_country);
        let buyer = if !buyer_siren.is_empty() || invoice.buyer_vat_id.is_some() || invoice.buyer_country.is_some() {
            Some(TransactionParty {
                company_id: if !buyer_siren.is_empty() { Some(buyer_siren.to_string()) } else { None },
                company_id_scheme: if !buyer_siren.is_empty() { Some(buyer_id_scheme.to_string()) } else { None },
                tax_registration_id: invoice.buyer_vat_id.clone(),
                tax_qualifying_id: invoice.buyer_vat_id.as_ref().map(|_| "VA".to_string()),
                country_code: invoice.buyer_country.clone(),
            })
        } else {
            None
        };

        // BR-FR-MAP-12 : Mapper le représentant fiscal du vendeur
        let seller_tax_representative = invoice.tax_representative_vat_id.as_ref().map(|vat_id| {
            TaxRepresentative {
                tax_registration_id: vat_id.clone(),
                scheme_id: "VA".to_string(),
            }
        });

        // BR-FR-MAP-14 : Mapper la livraison
        let deliveries = if invoice.delivery_date.is_some() || invoice.delivery_address.is_some() {
            vec![Delivery {
                date: invoice.delivery_date.clone().map(|d| d.replace('-', "")),
                location: invoice.delivery_address.as_ref().map(|addr| DeliveryLocation {
                    line_one: addr.line1.clone(),
                    line_two: addr.line2.clone(),
                    line_three: addr.line3.clone(),
                    city_name: addr.city.clone(),
                    postal_zone: addr.postal_code.clone(),
                    country_subentity: addr.country_subdivision.clone(),
                    country_code: addr.country_code.clone(),
                }),
            }]
        } else {
            Vec::new()
        };

        // BR-FR-MAP-15 : Mapper la période de facturation
        let invoice_period = if invoice.invoice_period_start.is_some() || invoice.invoice_period_end.is_some() {
            Some(InvoicePeriod {
                start_date: invoice.invoice_period_start.clone().map(|d| d.replace('-', "")),
                end_date: invoice.invoice_period_end.clone().map(|d| d.replace('-', "")),
            })
        } else {
            None
        };

        // BR-FR-MAP-16 : Mapper les remises/charges au niveau document
        let allowance_charges: Vec<AllowanceCharge> = invoice.allowance_charges.iter().map(|ac| {
            AllowanceCharge {
                charge_indicator: ac.charge_indicator,
                amount: ac.amount,
                tax_category_code: ac.tax_category_code.clone(),
                tax_percent: ac.tax_percent,
            }
        }).collect();

        // BR-FR-MAP-17/18 : Mapper la ventilation TVA depuis les vrais BG-23
        let tax_subtotals: Vec<TaxSubTotal> = if !invoice.tax_breakdowns.is_empty() {
            invoice.tax_breakdowns.iter().map(|tb| TaxSubTotal {
                taxable_amount: tb.taxable_amount.unwrap_or(0.0),
                tax_amount: tb.tax_amount.unwrap_or(0.0),
                tax_category_code: tb.category_code.clone(),
                tax_percent: tb.percent.unwrap_or(0.0),
                tax_exemption_reason: tb.exemption_reason.clone(),
                tax_exemption_reason_code: tb.exemption_reason_code.clone(),
            }).collect()
        } else {
            // Fallback : une seule ligne avec les totaux si pas de ventilation
            vec![TaxSubTotal {
                taxable_amount: invoice.total_ht.unwrap_or(0.0),
                tax_amount: invoice.total_tax.unwrap_or(0.0),
                tax_category_code: Some("S".to_string()),
                tax_percent: 20.0,
                tax_exemption_reason: None,
                tax_exemption_reason_code: None,
            }]
        };

        // BR-FR-MAP-19 : Mapper les lignes de facture
        let lines: Vec<crate::model::InvoiceLine> = invoice.lines.iter().map(|line| {
            crate::model::InvoiceLine {
                notes: Vec::new(),
                allowance_charges: Vec::new(),
                line_net_amount: line.line_net_amount,
                invoiced_quantity: line.quantity,
                invoiced_quantity_unit: line.unit_code.clone(),
                price_amount: line.price,
                tax_category_code: line.tax_category_code.clone(),
                tax_percent: line.tax_percent,
            }
        }).collect();

        TransactionInvoice {
            id: invoice.invoice_number.clone(),
            // BR-FR-MAP-23 : Conversion date YYYY-MM-DD → YYYYMMDD
            issue_date: invoice.issue_date.clone().unwrap_or_default().replace('-', ""),
            type_code: invoice.invoice_type_code.clone().unwrap_or_else(|| "380".to_string()),
            currency_code: invoice.currency.clone().unwrap_or_else(|| "EUR".to_string()),
            due_date: invoice.due_date.clone().map(|d| d.replace('-', "")),
            tax_due_date_type_code: None,
            notes,
            business_process: BusinessProcess {
                id: business_process_id,
                type_id: "urn.cpro.gouv.fr:1p0:ereporting".to_string(),
            },
            referenced_documents,
            seller,
            buyer,
            seller_tax_representative,
            deliveries,
            invoice_period,
            allowance_charges,
            monetary_total: MonetaryTotal {
                tax_exclusive_amount: invoice.total_ht,
                tax_amount: invoice.total_tax.unwrap_or(0.0),
                tax_amount_currency: invoice.currency.clone().unwrap_or_else(|| "EUR".to_string()),
            },
            tax_subtotals,
            lines,
        }
    }

    // ================================================================
    // BR-FR-MAP-16 : Schéma d'identifiant selon le pays
    // ================================================================

    /// Détermine le schemeId de l'identifiant d'un acteur selon son pays.
    ///
    /// - `0002` (SIREN) pour la France
    /// - `0223` (identifiant EU) pour un pays de l'UE hors France
    /// - `0227` (identifiant hors UE) pour un pays tiers
    pub fn id_scheme_for_country(country_code: Option<&str>) -> &'static str {
        match country_code {
            Some("FR") | None => "0002",
            Some(cc) if Self::is_eu_country(cc) => "0223",
            Some(_) => "0227",
        }
    }

    /// Liste des codes pays membres de l'UE (hors France, traitée séparément).
    pub fn is_eu_country(code: &str) -> bool {
        matches!(
            code,
            "AT" | "BE" | "BG" | "HR" | "CY" | "CZ" | "DK" | "EE"
            | "FI" | "DE" | "GR" | "HU" | "IE" | "IT" | "LV" | "LT"
            | "LU" | "MT" | "NL" | "PL" | "PT" | "RO" | "SK" | "SI"
            | "ES" | "SE"
        )
    }

    // ================================================================
    // Dérivation de la catégorie de transaction (TT-81)
    // ================================================================

    /// Détermine la catégorie de transaction agrégée à partir des lignes de facture.
    ///
    /// Logique :
    /// - Catégorie TVA `L` ou `M` → **TMA1** (régime de la marge)
    /// - Catégorie TVA `E`, `G`, `K`, `O`, `AE` → **TNT1** (non taxable en France)
    /// - Catégorie TVA `S`, `Z`, `AA` avec taux > 0 → nécessite distinction biens/services
    ///   - Si `item_classification` ou contexte indique des biens → **TLB1**
    ///   - Par défaut → **TPS1** (prestation de services, cas le plus fréquent en B2C)
    ///
    /// Pour les factures mixtes, la catégorie dominante est retenue.
    pub fn derive_transaction_category(invoice: &InvoiceData) -> TransactionCategory {
        let mut has_margin = false;
        let mut has_non_taxable = false;
        let mut has_goods = false;
        let mut has_services = false;

        // Analyser les ventilations TVA (BG-23)
        for tb in &invoice.tax_breakdowns {
            match tb.category_code.as_deref() {
                // Régime de la marge
                Some("L") | Some("M") => has_margin = true,
                // Non taxable en France (export, intra-EU, reverse charge, etc.)
                Some("E") | Some("G") | Some("K") | Some("O") | Some("AE") => {
                    has_non_taxable = true
                }
                // Taxable standard — analyser les lignes pour distinguer biens/services
                _ => {}
            }
        }

        // Si toutes les lignes sont marge, c'est TMA1
        if has_margin && !has_non_taxable {
            return TransactionCategory::TMA1;
        }

        // Si toutes les lignes sont non-taxables, c'est TNT1
        if has_non_taxable && !has_margin {
            return TransactionCategory::TNT1;
        }

        // Analyser les lignes individuelles pour les cas S/Z/AA
        for line in &invoice.lines {
            match line.tax_category_code.as_deref() {
                Some("L") | Some("M") => has_margin = true,
                Some("E") | Some("G") | Some("K") | Some("O") | Some("AE") => {
                    has_non_taxable = true
                }
                _ => {
                    // Heuristique : si la facture a une livraison physique, c'est des biens
                    if invoice.delivery_date.is_some() || invoice.delivery_address.is_some() {
                        has_goods = true;
                    } else {
                        has_services = true;
                    }
                }
            }
        }

        // Heuristique globale : si aucune ligne n'a pu trancher et qu'il y a une
        // livraison physique au niveau facture, c'est des biens (TLB1)
        if !has_goods && !has_services && !has_margin && !has_non_taxable {
            if invoice.delivery_date.is_some() || invoice.delivery_address.is_some() {
                has_goods = true;
            }
        }

        // Priorité : TMA1 > TNT1 > TLB1 > TPS1
        if has_margin {
            TransactionCategory::TMA1
        } else if has_non_taxable {
            TransactionCategory::TNT1
        } else if has_goods && !has_services {
            TransactionCategory::TLB1
        } else {
            // Par défaut, prestation de services (cas le plus fréquent en e-reporting B2C)
            TransactionCategory::TPS1
        }
    }

    // ================================================================
    // Validation EUR pour les montants TVA
    // ================================================================

    /// Valide que le montant de TVA est exprimé en EUR (obligation réglementaire).
    ///
    /// TT-52 (flux 10.1) et TT-83 (flux 10.3) doivent toujours être en EUR.
    /// Si la devise de la facture est différente, le montant de TVA doit être
    /// converti en EUR avant la transmission.
    pub fn validate_tax_currency(invoice: &InvoiceData) -> PdpResult<()> {
        if let Some(ref currency) = invoice.currency {
            if currency != "EUR" {
                // La devise de la facture n'est pas EUR.
                // Le montant de TVA (TT-52/TT-83) DOIT être en EUR.
                // Vérifier si un montant TVA en EUR a été fourni.
                if invoice.tax_amount_eur.is_none() && invoice.total_tax.is_some() {
                    return Err(PdpError::ValidationError(format!(
                        "E-reporting: la facture {} est en {} mais le montant TVA doit être en EUR (TT-52/TT-83). \
                         Fournir tax_amount_eur ou convertir avant transmission.",
                        invoice.invoice_number, currency
                    )));
                }
            }
        }
        Ok(())
    }

    /// Retourne le montant TVA en EUR, en utilisant tax_amount_eur si disponible,
    /// sinon total_tax si la devise est EUR.
    fn tax_amount_in_eur(invoice: &InvoiceData) -> f64 {
        // Priorité 1 : montant TVA explicitement en EUR
        if let Some(eur) = invoice.tax_amount_eur {
            return eur;
        }
        // Priorité 2 : devise déjà en EUR
        if invoice.currency.as_deref() == Some("EUR") || invoice.currency.is_none() {
            return invoice.total_tax.unwrap_or(0.0);
        }
        // Fallback (ne devrait pas arriver si validate_tax_currency est appelé)
        invoice.total_tax.unwrap_or(0.0)
    }

    // ================================================================
    // Détection autoliquidation (reverse charge)
    // ================================================================

    /// Détecte si une facture contient de l'autoliquidation (reverse charge).
    ///
    /// L'autoliquidation est identifiée par la catégorie TVA `AE` dans les
    /// ventilations BG-23. Dans ce cas, la TVA est due par l'acheteur.
    pub fn has_reverse_charge(invoice: &InvoiceData) -> bool {
        invoice.tax_breakdowns.iter().any(|tb| {
            tb.category_code.as_deref() == Some("AE")
        })
    }

    // ================================================================
    // Flux 10.3 : Création de rapport de transactions agrégées
    // ================================================================

    /// Crée un rapport de transactions agrégées (flux 10.3) à partir d'une liste de factures.
    ///
    /// Les factures sont regroupées par :
    /// - Date (TT-77)
    /// - Devise (TT-78)
    /// - Catégorie de transaction (TT-81)
    /// - Option de paiement TVA (TT-80)
    ///
    /// Pour chaque groupe, les montants sont cumulés et le nombre de transactions comptabilisé.
    pub fn create_aggregated_transactions_report(
        &self,
        report_id: &str,
        declarant_siren: &str,
        declarant_name: &str,
        period_start: &str,
        period_end: &str,
        invoices: &[InvoiceData],
    ) -> PdpResult<EReport> {
        let now = Utc::now();
        let dt = now.format("%Y%m%d%H%M%S").to_string();

        // Valider les devises TVA
        for inv in invoices {
            Self::validate_tax_currency(inv)?;
        }

        // Clé d'agrégation : (date, devise, catégorie, option_paiement_tva)
        let mut groups: HashMap<(String, String, TransactionCategory, String), AggregationAccumulator> =
            HashMap::new();

        for inv in invoices {
            let date = inv.issue_date.clone().unwrap_or_default().replace('-', "");
            let currency = inv.currency.clone().unwrap_or_else(|| "EUR".to_string());
            let category = Self::derive_transaction_category(inv);
            let vat_option = if inv.tax_due_on_payment.unwrap_or(false) {
                "debit".to_string()
            } else {
                "non-debit".to_string()
            };

            let key = (date, currency, category, vat_option);
            let acc = groups.entry(key).or_insert_with(AggregationAccumulator::new);

            // Cumuler les montants
            acc.total_ht += inv.total_ht.unwrap_or(0.0);
            acc.total_tax_eur += Self::tax_amount_in_eur(inv);
            acc.count += 1;

            // Cumuler par taux de TVA
            for tb in &inv.tax_breakdowns {
                let rate_key = (
                    tb.category_code.clone().unwrap_or_else(|| "S".to_string()),
                    format!("{:.2}", tb.percent.unwrap_or(0.0)),
                );
                let rate_acc = acc.by_rate.entry(rate_key).or_insert((0.0, 0.0));
                rate_acc.0 += tb.taxable_amount.unwrap_or(0.0);
                rate_acc.1 += tb.tax_amount.unwrap_or(0.0);
            }
        }

        // Construire les transactions agrégées
        let aggregated_transactions: Vec<AggregatedTransaction> = groups
            .into_iter()
            .map(|((date, currency, category, vat_option), acc)| {
                let tax_subtotals: Vec<AggregatedTaxSubTotal> = acc
                    .by_rate
                    .into_iter()
                    .map(|((cat_code, _rate_str), (taxable, tax))| {
                        let rate: f64 = _rate_str.parse().unwrap_or(0.0);
                        AggregatedTaxSubTotal {
                            taxable_amount: taxable,
                            tax_amount: tax,
                            tax_category_code: cat_code,
                            tax_percent: rate,
                        }
                    })
                    .collect();

                AggregatedTransaction {
                    date,
                    currency_code: currency,
                    vat_payment_option: Some(vat_option),
                    category,
                    cumulative_amount_ht: acc.total_ht,
                    cumulative_tax_amount_eur: acc.total_tax_eur,
                    transaction_count: acc.count,
                    tax_subtotals,
                }
            })
            .collect();

        Ok(EReport {
            document: ReportDocument {
                id: report_id.to_string(),
                name: Some(format!("E-reporting_10.3_{}", report_id)),
                issue_datetime: dt,
                type_code: ReportTypeCode::TransactionsAggregated,
                sender: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: self.pdp_siren.clone(),
                    name: self.pdp_name.clone(),
                    role_code: "WK".to_string(),
                    endpoint_uri: None,
                },
                issuer: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: declarant_siren.to_string(),
                    name: declarant_name.to_string(),
                    role_code: "SE".to_string(),
                    endpoint_uri: None,
                },
            },
            transactions: Some(TransactionsReport {
                period_start: period_start.to_string(),
                period_end: period_end.to_string(),
                invoices: Vec::new(),
                aggregated_transactions,
            }),
            payments: None,
        })
    }

    /// BR-FR-MAP-01 : Dérive le cadre de facturation pour le e-reporting.
    ///
    /// Ordre de priorité :
    /// 1. Note BAR (BT-21 = "BAR") → valeur de BT-22
    /// 2. BT-23 (business_process) si présent
    /// 3. Défaut "B2C" (cas e-reporting le plus fréquent)
    fn derive_business_process(invoice: &InvoiceData) -> String {
        // Chercher la note BAR
        for note in &invoice.notes {
            if note.subject_code.as_deref() == Some("BAR") {
                let bar_value = note.content.trim().to_uppercase();
                // Valeurs autorisées : B2B, B2BINT, B2C, OUTOFSCOPE, ARCHIVEONLY
                match bar_value.as_str() {
                    "B2B" | "B2BINT" | "B2C" | "OUTOFSCOPE" | "ARCHIVEONLY" => {
                        return bar_value;
                    }
                    _ => {}
                }
            }
        }

        // Fallback sur BT-23
        if let Some(ref bp) = invoice.business_process {
            let bp_upper = bp.trim().to_uppercase();
            // Mapper les cadres de facturation Sx/Bx/Mx vers e-reporting
            match bp_upper.as_str() {
                "S1" | "B1" | "S2" | "B2" | "S3" | "B3" | "S4" | "B4"
                | "S5" | "B5" | "S6" | "B6" | "S7" | "B7" | "S8" | "B8" | "M8" => {
                    return "B2B".to_string();
                }
                _ => {
                    return bp_upper;
                }
            }
        }

        // Défaut e-reporting
        "B2C".to_string()
    }

    /// Sérialise un rapport e-reporting en XML conforme au XSD PPF
    pub fn to_xml(&self, report: &EReport) -> PdpResult<String> {
        let mut xml = String::with_capacity(8192);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<Report>"#);

        // ReportDocument
        self.write_report_document(&mut xml, &report.document);

        // TransactionsReport
        if let Some(ref txns) = report.transactions {
            self.write_transactions_report(&mut xml, txns);
        }

        // PaymentsReport
        if let Some(ref pays) = report.payments {
            self.write_payments_report(&mut xml, pays);
        }

        xml.push_str("\n</Report>");
        Ok(xml)
    }

    fn write_report_document(&self, xml: &mut String, doc: &ReportDocument) {
        xml.push_str(&format!(r#"
    <ReportDocument>
        <Id>{}</Id>"#, xml_escape(&doc.id)));

        if let Some(ref name) = doc.name {
            xml.push_str(&format!("\n        <Name>{}</Name>", xml_escape(name)));
        }

        xml.push_str(&format!(r#"
        <IssueDateTime>
            <DateTimeString>{}</DateTimeString>
        </IssueDateTime>
        <TypeCode>{}</TypeCode>
        <Sender>
            <Id schemeId="{}">{}</Id>
            <Name>{}</Name>
            <RoleCode>{}</RoleCode>"#,
            &doc.issue_datetime,
            xml_escape(doc.type_code.code()),
            xml_escape(&doc.sender.id_scheme),
            xml_escape(&doc.sender.id),
            xml_escape(&doc.sender.name),
            xml_escape(&doc.sender.role_code),
        ));

        if let Some(ref uri) = doc.sender.endpoint_uri {
            xml.push_str(&format!(r#"
            <URIUniversalCommunication>
                <URIID>{}</URIID>
            </URIUniversalCommunication>"#, xml_escape(uri)));
        }

        xml.push_str(&format!(r#"
        </Sender>
        <Issuer>
            <Id schemeId="{}">{}</Id>
            <Name>{}</Name>
            <RoleCode>{}</RoleCode>"#,
            xml_escape(&doc.issuer.id_scheme),
            xml_escape(&doc.issuer.id),
            xml_escape(&doc.issuer.name),
            xml_escape(&doc.issuer.role_code),
        ));

        if let Some(ref uri) = doc.issuer.endpoint_uri {
            xml.push_str(&format!(r#"
            <URIUniversalCommunication>
                <URIID>{}</URIID>
            </URIUniversalCommunication>"#, xml_escape(uri)));
        }

        xml.push_str(r#"
        </Issuer>
    </ReportDocument>"#);
    }

    fn write_transactions_report(&self, xml: &mut String, txns: &TransactionsReport) {
        xml.push_str(&format!(r#"
    <TransactionsReport>
        <ReportPeriod>
            <StartDate>{}</StartDate>
            <EndDate>{}</EndDate>
        </ReportPeriod>"#,
            xml_escape(&txns.period_start),
            xml_escape(&txns.period_end),
        ));

        // TG-8 : Factures détaillées (flux 10.1)
        for inv in &txns.invoices {
            self.write_transaction_invoice(xml, inv);
        }

        // TG-31 : Transactions agrégées (flux 10.3)
        for agg in &txns.aggregated_transactions {
            self.write_aggregated_transaction(xml, agg);
        }

        xml.push_str("\n    </TransactionsReport>");
    }

    /// Écrit un bloc XML TG-31 (transaction agrégée) pour le flux 10.3
    fn write_aggregated_transaction(&self, xml: &mut String, agg: &AggregatedTransaction) {
        xml.push_str(&format!(r#"
        <AggregatedTransaction>
            <Date>{}</Date>
            <CurrencyCode>{}</CurrencyCode>"#,
            xml_escape(&agg.date),
            xml_escape(&agg.currency_code),
        ));

        if let Some(ref opt) = agg.vat_payment_option {
            xml.push_str(&format!(
                "\n            <VATPaymentOption>{}</VATPaymentOption>",
                xml_escape(opt)
            ));
        }

        xml.push_str(&format!(r#"
            <Category>{}</Category>
            <CumulativeAmountHT>{:.2}</CumulativeAmountHT>
            <CumulativeTaxAmountEUR>{:.2}</CumulativeTaxAmountEUR>
            <TransactionCount>{}</TransactionCount>"#,
            xml_escape(agg.category.code()),
            agg.cumulative_amount_ht,
            agg.cumulative_tax_amount_eur,
            agg.transaction_count,
        ));

        // TG-32 : Ventilation TVA
        for tst in &agg.tax_subtotals {
            xml.push_str(&format!(r#"
            <TaxSubTotal>
                <TaxableAmount>{:.2}</TaxableAmount>
                <TaxAmount>{:.2}</TaxAmount>
                <TaxCategory>
                    <Code>{}</Code>
                    <Percent>{:.2}</Percent>
                </TaxCategory>
            </TaxSubTotal>"#,
                tst.taxable_amount,
                tst.tax_amount,
                xml_escape(&tst.tax_category_code),
                tst.tax_percent,
            ));
        }

        xml.push_str("\n        </AggregatedTransaction>");
    }

    fn write_transaction_invoice(&self, xml: &mut String, inv: &TransactionInvoice) {
        xml.push_str(&format!(r#"
        <Invoice>
            <ID>{}</ID>
            <IssueDate>{}</IssueDate>
            <TypeCode>{}</TypeCode>
            <CurrencyCode>{}</CurrencyCode>"#,
            xml_escape(&inv.id),
            xml_escape(&inv.issue_date),
            xml_escape(&inv.type_code),
            xml_escape(&inv.currency_code),
        ));

        if let Some(ref dd) = inv.due_date {
            xml.push_str(&format!("\n            <DueDate>{}</DueDate>", xml_escape(dd)));
        }

        // BusinessProcess
        xml.push_str(&format!(r#"
            <BusinessProcess>
                <ID>{}</ID>
                <TypeID>{}</TypeID>
            </BusinessProcess>"#,
            xml_escape(&inv.business_process.id),
            xml_escape(&inv.business_process.type_id),
        ));

        // Seller
        xml.push_str("\n            <Seller>");
        if let (Some(ref id), Some(ref scheme)) = (&inv.seller.company_id, &inv.seller.company_id_scheme) {
            xml.push_str(&format!(r#"
                <CompanyId schemeId="{}">{}</CompanyId>"#, xml_escape(scheme), xml_escape(id)));
        }
        if let Some(ref cc) = inv.seller.country_code {
            xml.push_str(&format!(r#"
                <PostalAddress>
                    <CountryId>{}</CountryId>
                </PostalAddress>"#, xml_escape(cc)));
        }
        xml.push_str("\n            </Seller>");

        // Buyer
        if let Some(ref buyer) = inv.buyer {
            xml.push_str("\n            <Buyer>");
            if let (Some(ref id), Some(ref scheme)) = (&buyer.company_id, &buyer.company_id_scheme) {
                xml.push_str(&format!(r#"
                <CompanyId schemeId="{}">{}</CompanyId>"#, xml_escape(scheme), xml_escape(id)));
            }
            if let Some(ref cc) = buyer.country_code {
                xml.push_str(&format!(r#"
                <PostalAddress>
                    <CountryId>{}</CountryId>
                </PostalAddress>"#, xml_escape(cc)));
            }
            xml.push_str("\n            </Buyer>");
        }

        // MonetaryTotal
        xml.push_str("\n            <MonetaryTotal>");
        if let Some(tea) = inv.monetary_total.tax_exclusive_amount {
            xml.push_str(&format!("\n                <TaxExclusiveAmount>{:.2}</TaxExclusiveAmount>", tea));
        }
        xml.push_str(&format!(r#"
                <TaxAmount CurrencyCode="{}">{:.2}</TaxAmount>"#,
            xml_escape(&inv.monetary_total.tax_amount_currency),
            inv.monetary_total.tax_amount,
        ));
        xml.push_str("\n            </MonetaryTotal>");

        // TaxSubTotals
        for tst in &inv.tax_subtotals {
            xml.push_str(&format!(r#"
            <TaxSubTotal>
                <TaxableAmount>{:.2}</TaxableAmount>
                <TaxAmount>{:.2}</TaxAmount>
                <TaxCategory>"#,
                tst.taxable_amount, tst.tax_amount));

            if let Some(ref code) = tst.tax_category_code {
                xml.push_str(&format!("\n                    <Code>{}</Code>", xml_escape(code)));
            }
            xml.push_str(&format!("\n                    <Percent>{:.2}</Percent>", tst.tax_percent));
            if let Some(ref reason) = tst.tax_exemption_reason {
                xml.push_str(&format!("\n                    <TaxExemptionReason>{}</TaxExemptionReason>", xml_escape(reason)));
            }
            if let Some(ref code) = tst.tax_exemption_reason_code {
                xml.push_str(&format!("\n                    <TaxExemptionReasonCode>{}</TaxExemptionReasonCode>", xml_escape(code)));
            }
            xml.push_str(r#"
                </TaxCategory>
            </TaxSubTotal>"#);
        }

        xml.push_str("\n        </Invoice>");
    }

    /// Crée un rapport de paiements par facture (flux 10.2)
    pub fn create_payments_report(
        &self,
        report_id: &str,
        declarant_siren: &str,
        declarant_name: &str,
        period_start: &str,
        period_end: &str,
        invoices: Vec<PaymentInvoice>,
    ) -> EReport {
        let now = Utc::now();
        let dt = now.format("%Y%m%d%H%M%S").to_string();

        EReport {
            document: ReportDocument {
                id: report_id.to_string(),
                name: Some(format!("E-reporting_10.2_{}", report_id)),
                issue_datetime: dt,
                type_code: ReportTypeCode::PaymentsInitial,
                sender: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: self.pdp_siren.clone(),
                    name: self.pdp_name.clone(),
                    role_code: "WK".to_string(),
                    endpoint_uri: None,
                },
                issuer: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: declarant_siren.to_string(),
                    name: declarant_name.to_string(),
                    role_code: "SE".to_string(),
                    endpoint_uri: None,
                },
            },
            transactions: None,
            payments: Some(PaymentsReport {
                period_start: period_start.to_string(),
                period_end: period_end.to_string(),
                invoices,
                transactions: Vec::new(),
            }),
        }
    }

    /// Crée un rapport de paiements agrégés (flux 10.4)
    pub fn create_aggregated_payments_report(
        &self,
        report_id: &str,
        declarant_siren: &str,
        declarant_name: &str,
        period_start: &str,
        period_end: &str,
        transactions: Vec<PaymentTransaction>,
    ) -> EReport {
        let now = Utc::now();
        let dt = now.format("%Y%m%d%H%M%S").to_string();

        EReport {
            document: ReportDocument {
                id: report_id.to_string(),
                name: Some(format!("E-reporting_10.4_{}", report_id)),
                issue_datetime: dt,
                type_code: ReportTypeCode::PaymentsAggregated,
                sender: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: self.pdp_siren.clone(),
                    name: self.pdp_name.clone(),
                    role_code: "WK".to_string(),
                    endpoint_uri: None,
                },
                issuer: ReportParty {
                    id_scheme: "0002".to_string(),
                    id: declarant_siren.to_string(),
                    name: declarant_name.to_string(),
                    role_code: "SE".to_string(),
                    endpoint_uri: None,
                },
            },
            transactions: None,
            payments: Some(PaymentsReport {
                period_start: period_start.to_string(),
                period_end: period_end.to_string(),
                invoices: Vec::new(),
                transactions,
            }),
        }
    }

    fn write_payments_report(&self, xml: &mut String, pays: &PaymentsReport) {
        xml.push_str(&format!(r#"
    <PaymentsReport>
        <ReportPeriod>
            <StartDate>{}</StartDate>
            <EndDate>{}</EndDate>
        </ReportPeriod>"#,
            xml_escape(&pays.period_start),
            xml_escape(&pays.period_end),
        ));

        // TG-34 : Paiements par facture (flux 10.2)
        for inv in &pays.invoices {
            xml.push_str(&format!(r#"
        <Invoice>
            <InvoiceID>{}</InvoiceID>
            <IssueDate>{}</IssueDate>"#,
                xml_escape(&inv.invoice_id),
                xml_escape(&inv.issue_date),
            ));
            self.write_payment_detail(xml, &inv.payment, 12);
            xml.push_str("\n        </Invoice>");
        }

        // TG-37 : Transactions agrégées (flux 10.4)
        for txn in &pays.transactions {
            xml.push_str("\n        <Transactions>");
            self.write_payment_detail(xml, &txn.payment, 12);
            xml.push_str("\n        </Transactions>");
        }

        xml.push_str("\n    </PaymentsReport>");
    }

    fn write_payment_detail(&self, xml: &mut String, payment: &PaymentDetail, indent: usize) {
        let pad = " ".repeat(indent);
        xml.push_str(&format!("\n{}<Payment>", pad));
        xml.push_str(&format!("\n{}    <Date>{}</Date>", pad, xml_escape(&payment.date)));

        for st in &payment.sub_totals {
            xml.push_str(&format!("\n{}    <SubTotals>", pad));
            xml.push_str(&format!("\n{}        <TaxPercent>{:.2}</TaxPercent>", pad, st.tax_percent));
            if let Some(ref cc) = st.currency_code {
                xml.push_str(&format!("\n{}        <CurrencyCode>{}</CurrencyCode>", pad, xml_escape(cc)));
            }
            xml.push_str(&format!("\n{}        <Amount>{:.2}</Amount>", pad, st.amount));
            xml.push_str(&format!("\n{}    </SubTotals>", pad));
        }

        xml.push_str(&format!("\n{}</Payment>", pad));
    }
}

/// Accumulateur interne pour l'agrégation des transactions (Flux 10.3)
struct AggregationAccumulator {
    total_ht: f64,
    total_tax_eur: f64,
    count: u32,
    /// Ventilation par (code_categorie_tva, taux_str) → (base_imposable, montant_tva)
    by_rate: HashMap<(String, String), (f64, f64)>,
}

impl AggregationAccumulator {
    fn new() -> Self {
        Self {
            total_ht: 0.0,
            total_tax_eur: 0.0,
            count: 0,
            by_rate: HashMap::new(),
        }
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
        inv.seller_vat_id = Some("FR12100000009".to_string());
        inv.seller_country = Some("FR".to_string());
        inv.buyer_country = Some("FR".to_string());
        inv.total_ht = Some(1000.00);
        inv.total_tax = Some(200.00);
        inv.total_ttc = Some(1200.00);
        inv.currency = Some("EUR".to_string());
        inv.invoice_type_code = Some("380".to_string());
        inv.tax_breakdowns = vec![
            pdp_core::model::TaxBreakdown {
                taxable_amount: Some(800.00),
                tax_amount: Some(160.00),
                category_code: Some("S".to_string()),
                percent: Some(20.0),
                exemption_reason: None,
                exemption_reason_code: None,
            },
            pdp_core::model::TaxBreakdown {
                taxable_amount: Some(200.00),
                tax_amount: Some(40.00),
                category_code: Some("S".to_string()),
                percent: Some(5.5),  // Taux réduit non hardcodé
                exemption_reason: None,
                exemption_reason_code: None,
            },
        ];
        inv.notes = vec![
            pdp_core::model::InvoiceNote {
                content: "B2BINT".to_string(),
                subject_code: Some("BAR".to_string()),
            },
        ];
        inv
    }

    #[test]
    fn test_create_transactions_report() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let txn = EReportingGenerator::invoice_to_transaction(&invoice);

        let report = gen.create_transactions_report(
            "RPT-2025-001",
            "100000009",
            "VENDEUR",
            "20250701",
            "20250731",
            vec![txn],
        );

        assert_eq!(report.document.type_code, ReportTypeCode::TransactionsInitial);
        assert!(report.transactions.is_some());
        assert!(report.payments.is_none());

        let txns = report.transactions.as_ref().unwrap();
        assert_eq!(txns.invoices.len(), 1);
        assert_eq!(txns.invoices[0].id, "F202500001");
        assert_eq!(txns.invoices[0].monetary_total.tax_amount, 200.00);
    }

    #[test]
    fn test_invoice_to_transaction() {
        let invoice = make_test_invoice();
        let txn = EReportingGenerator::invoice_to_transaction(&invoice);

        assert_eq!(txn.id, "F202500001");
        assert_eq!(txn.issue_date, "20250701");
        assert_eq!(txn.type_code, "380");
        assert_eq!(txn.currency_code, "EUR");
        assert_eq!(txn.seller.company_id, Some("100000009".to_string()));
        assert_eq!(txn.buyer.as_ref().unwrap().company_id, Some("200000008".to_string()));

        // BR-FR-MAP-01 : cadre de facturation dérivé de la note BAR
        assert_eq!(txn.business_process.id, "B2BINT");

        // BR-FR-MAP-08 : TVA vendeur mappée
        assert_eq!(txn.seller.tax_registration_id, Some("FR12100000009".to_string()));
        assert_eq!(txn.seller.country_code, Some("FR".to_string()));

        // BR-FR-MAP-10 : pays acheteur mappé
        assert_eq!(txn.buyer.as_ref().unwrap().country_code, Some("FR".to_string()));

        // BR-FR-MAP-17 : ventilation TVA réelle (2 taux, pas hardcodé 20%)
        assert_eq!(txn.tax_subtotals.len(), 2);
        assert_eq!(txn.tax_subtotals[0].tax_percent, 20.0);
        assert_eq!(txn.tax_subtotals[0].taxable_amount, 800.00);
        assert_eq!(txn.tax_subtotals[1].tax_percent, 5.5);
        assert_eq!(txn.tax_subtotals[1].taxable_amount, 200.00);

        // BR-FR-MAP-04 : notes mappées
        assert_eq!(txn.notes.len(), 1);
        assert_eq!(txn.notes[0].subject, Some("BAR".to_string()));
    }

    #[test]
    fn test_report_to_xml() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");
        let invoice = make_test_invoice();
        let txn = EReportingGenerator::invoice_to_transaction(&invoice);

        let report = gen.create_transactions_report(
            "RPT-2025-001",
            "100000009",
            "VENDEUR",
            "20250701",
            "20250731",
            vec![txn],
        );

        let xml = gen.to_xml(&report).unwrap();

        assert!(xml.contains("<Report>"));
        assert!(xml.contains("<ReportDocument>"));
        assert!(xml.contains("<TypeCode>10.1</TypeCode>"));
        assert!(xml.contains("<ID>F202500001</ID>"));
        assert!(xml.contains("schemeId=\"0002\""));
        assert!(xml.contains("<TaxAmount CurrencyCode=\"EUR\">200.00</TaxAmount>"));
        assert!(xml.contains("<TaxableAmount>800.00</TaxableAmount>"));
        assert!(xml.contains("<TaxableAmount>200.00</TaxableAmount>"));
        assert!(xml.contains("<StartDate>20250701</StartDate>"));
        assert!(xml.contains("<EndDate>20250731</EndDate>"));
        // BR-FR-MAP-01 : cadre de facturation depuis note BAR
        assert!(xml.contains("<ID>B2BINT</ID>"));
    }

    #[test]
    fn test_report_type_codes() {
        assert_eq!(ReportTypeCode::TransactionsInitial.code(), "10.1");
        assert_eq!(ReportTypeCode::PaymentsInitial.code(), "10.2");
        assert_eq!(ReportTypeCode::TransactionsAggregated.code(), "10.3");
        assert_eq!(ReportTypeCode::PaymentsAggregated.code(), "10.4");

        assert_eq!(ReportTypeCode::from_code("10.1"), Some(ReportTypeCode::TransactionsInitial));
        assert_eq!(ReportTypeCode::from_code("invalid"), None);
    }

    #[test]
    fn test_create_payments_report() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");

        let invoices = vec![PaymentInvoice {
            invoice_id: "F202500001".to_string(),
            issue_date: "20250701".to_string(),
            payment: PaymentDetail {
                date: "20250715".to_string(),
                sub_totals: vec![
                    PaymentSubTotal {
                        tax_percent: 20.0,
                        currency_code: Some("EUR".to_string()),
                        amount: 1200.00,
                    },
                ],
            },
        }];

        let report = gen.create_payments_report(
            "PAY-2025-001",
            "100000009",
            "VENDEUR",
            "20250701",
            "20250731",
            invoices,
        );

        assert_eq!(report.document.type_code, ReportTypeCode::PaymentsInitial);
        assert!(report.transactions.is_none());
        assert!(report.payments.is_some());

        let pays = report.payments.as_ref().unwrap();
        assert_eq!(pays.invoices.len(), 1);
        assert_eq!(pays.invoices[0].invoice_id, "F202500001");
        assert_eq!(pays.invoices[0].payment.sub_totals[0].amount, 1200.00);
        assert!(pays.transactions.is_empty());
    }

    #[test]
    fn test_payments_report_to_xml() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");

        let invoices = vec![PaymentInvoice {
            invoice_id: "F202500001".to_string(),
            issue_date: "20250701".to_string(),
            payment: PaymentDetail {
                date: "20250715".to_string(),
                sub_totals: vec![
                    PaymentSubTotal {
                        tax_percent: 20.0,
                        currency_code: Some("EUR".to_string()),
                        amount: 1200.00,
                    },
                    PaymentSubTotal {
                        tax_percent: 5.5,
                        currency_code: Some("EUR".to_string()),
                        amount: 105.50,
                    },
                ],
            },
        }];

        let report = gen.create_payments_report(
            "PAY-2025-001",
            "100000009",
            "VENDEUR",
            "20250701",
            "20250731",
            invoices,
        );

        let xml = gen.to_xml(&report).unwrap();

        assert!(xml.contains("<TypeCode>10.2</TypeCode>"));
        assert!(xml.contains("<PaymentsReport>"));
        assert!(xml.contains("<InvoiceID>F202500001</InvoiceID>"));
        assert!(xml.contains("<IssueDate>20250701</IssueDate>"));
        assert!(xml.contains("<Date>20250715</Date>"));
        assert!(xml.contains("<TaxPercent>20.00</TaxPercent>"));
        assert!(xml.contains("<TaxPercent>5.50</TaxPercent>"));
        assert!(xml.contains("<CurrencyCode>EUR</CurrencyCode>"));
        assert!(xml.contains("<Amount>1200.00</Amount>"));
        assert!(xml.contains("<Amount>105.50</Amount>"));
    }

    // ============================================================
    // Tests pour la dérivation de catégorie de transaction
    // ============================================================

    #[test]
    fn test_derive_category_services_default() {
        // Par défaut (TVA standard, pas de livraison) → TPS1
        let inv = make_test_invoice();
        let cat = EReportingGenerator::derive_transaction_category(&inv);
        assert_eq!(cat, TransactionCategory::TPS1);
    }

    #[test]
    fn test_derive_category_goods_with_delivery() {
        // Facture avec livraison physique → TLB1
        let mut inv = make_test_invoice();
        inv.delivery_date = Some("2025-07-15".to_string());
        // Enlever les notes BAR pour simplifier
        inv.notes.clear();
        let cat = EReportingGenerator::derive_transaction_category(&inv);
        assert_eq!(cat, TransactionCategory::TLB1);
    }

    #[test]
    fn test_derive_category_margin_scheme() {
        // Catégorie TVA L (marge biens d'occasion) → TMA1
        let mut inv = make_test_invoice();
        inv.tax_breakdowns = vec![pdp_core::model::TaxBreakdown {
            taxable_amount: Some(500.0),
            tax_amount: Some(0.0),
            category_code: Some("L".to_string()),
            percent: Some(0.0),
            exemption_reason: Some("Régime de la marge".to_string()),
            exemption_reason_code: None,
        }];
        let cat = EReportingGenerator::derive_transaction_category(&inv);
        assert_eq!(cat, TransactionCategory::TMA1);
    }

    #[test]
    fn test_derive_category_non_taxable() {
        // Catégorie TVA G (export hors UE) → TNT1
        let mut inv = make_test_invoice();
        inv.tax_breakdowns = vec![pdp_core::model::TaxBreakdown {
            taxable_amount: Some(1000.0),
            tax_amount: Some(0.0),
            category_code: Some("G".to_string()),
            percent: Some(0.0),
            exemption_reason: Some("Exportation".to_string()),
            exemption_reason_code: Some("VATEX-EU-G".to_string()),
        }];
        let cat = EReportingGenerator::derive_transaction_category(&inv);
        assert_eq!(cat, TransactionCategory::TNT1);
    }

    #[test]
    fn test_derive_category_reverse_charge() {
        // Catégorie TVA AE (autoliquidation) → TNT1
        let mut inv = make_test_invoice();
        inv.tax_breakdowns = vec![pdp_core::model::TaxBreakdown {
            taxable_amount: Some(1000.0),
            tax_amount: Some(0.0),
            category_code: Some("AE".to_string()),
            percent: Some(0.0),
            exemption_reason: None,
            exemption_reason_code: None,
        }];
        let cat = EReportingGenerator::derive_transaction_category(&inv);
        assert_eq!(cat, TransactionCategory::TNT1);
    }

    // ============================================================
    // Tests BR-FR-MAP-16 (schémas d'identifiants par pays)
    // ============================================================

    #[test]
    fn test_id_scheme_france() {
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("FR")), "0002");
        assert_eq!(EReportingGenerator::id_scheme_for_country(None), "0002");
    }

    #[test]
    fn test_id_scheme_eu() {
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("DE")), "0223");
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("IT")), "0223");
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("ES")), "0223");
    }

    #[test]
    fn test_id_scheme_non_eu() {
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("US")), "0227");
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("CH")), "0227");
        assert_eq!(EReportingGenerator::id_scheme_for_country(Some("GB")), "0227");
    }

    #[test]
    fn test_invoice_to_transaction_eu_buyer() {
        let mut inv = make_test_invoice();
        inv.buyer_country = Some("DE".to_string());
        let txn = EReportingGenerator::invoice_to_transaction(&inv);
        // BR-FR-MAP-16 : acheteur UE → schéma 0223
        assert_eq!(
            txn.buyer.as_ref().unwrap().company_id_scheme,
            Some("0223".to_string())
        );
    }

    // ============================================================
    // Tests détection autoliquidation
    // ============================================================

    #[test]
    fn test_has_reverse_charge() {
        let mut inv = make_test_invoice();
        assert!(!EReportingGenerator::has_reverse_charge(&inv));

        inv.tax_breakdowns.push(pdp_core::model::TaxBreakdown {
            taxable_amount: Some(500.0),
            tax_amount: Some(0.0),
            category_code: Some("AE".to_string()),
            percent: Some(0.0),
            exemption_reason: None,
            exemption_reason_code: None,
        });
        assert!(EReportingGenerator::has_reverse_charge(&inv));
    }

    // ============================================================
    // Tests validation EUR
    // ============================================================

    #[test]
    fn test_validate_tax_currency_eur_ok() {
        let inv = make_test_invoice(); // EUR par défaut
        assert!(EReportingGenerator::validate_tax_currency(&inv).is_ok());
    }

    #[test]
    fn test_validate_tax_currency_foreign_without_eur_amount() {
        let mut inv = make_test_invoice();
        inv.currency = Some("USD".to_string());
        inv.tax_amount_eur = None;
        // Doit échouer : devise USD sans montant TVA en EUR
        assert!(EReportingGenerator::validate_tax_currency(&inv).is_err());
    }

    #[test]
    fn test_validate_tax_currency_foreign_with_eur_amount() {
        let mut inv = make_test_invoice();
        inv.currency = Some("USD".to_string());
        inv.tax_amount_eur = Some(185.50);
        // OK : montant TVA EUR fourni
        assert!(EReportingGenerator::validate_tax_currency(&inv).is_ok());
    }

    // ============================================================
    // Tests Flux 10.3 (transactions agrégées)
    // ============================================================

    #[test]
    fn test_create_aggregated_transactions_report() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");

        let inv1 = make_test_invoice();
        let mut inv2 = make_test_invoice();
        inv2.invoice_number = "F202500002".to_string();
        inv2.total_ht = Some(500.0);
        inv2.total_tax = Some(100.0);
        inv2.tax_breakdowns = vec![pdp_core::model::TaxBreakdown {
            taxable_amount: Some(500.0),
            tax_amount: Some(100.0),
            category_code: Some("S".to_string()),
            percent: Some(20.0),
            exemption_reason: None,
            exemption_reason_code: None,
        }];

        let report = gen
            .create_aggregated_transactions_report(
                "AGG-2025-001",
                "100000009",
                "VENDEUR",
                "20250701",
                "20250731",
                &[inv1, inv2],
            )
            .unwrap();

        assert_eq!(
            report.document.type_code,
            ReportTypeCode::TransactionsAggregated
        );
        assert!(report.transactions.is_some());
        let txns = report.transactions.as_ref().unwrap();
        assert!(txns.invoices.is_empty());
        assert!(!txns.aggregated_transactions.is_empty());

        // Les deux factures ont la même date, devise et catégorie → agrégées ensemble
        let agg = &txns.aggregated_transactions[0];
        assert_eq!(agg.transaction_count, 2);
        assert_eq!(agg.cumulative_amount_ht, 1500.0); // 1000 + 500
        assert_eq!(agg.cumulative_tax_amount_eur, 300.0); // 200 + 100
    }

    #[test]
    fn test_aggregated_transactions_xml() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");
        let inv = make_test_invoice();

        let report = gen
            .create_aggregated_transactions_report(
                "AGG-2025-002",
                "100000009",
                "VENDEUR",
                "20250701",
                "20250731",
                &[inv],
            )
            .unwrap();

        let xml = gen.to_xml(&report).unwrap();

        assert!(xml.contains("<TypeCode>10.3</TypeCode>"));
        assert!(xml.contains("<AggregatedTransaction>"));
        assert!(xml.contains("<Category>TPS1</Category>"));
        assert!(xml.contains("<CumulativeAmountHT>"));
        assert!(xml.contains("<CumulativeTaxAmountEUR>"));
        assert!(xml.contains("<TransactionCount>1</TransactionCount>"));
        assert!(xml.contains("<TaxSubTotal>"));
        assert!(xml.contains("</AggregatedTransaction>"));
    }

    #[test]
    fn test_aggregated_payments_report_to_xml() {
        let gen = EReportingGenerator::new("100000009", "PDP Test");

        let transactions = vec![PaymentTransaction {
            payment: PaymentDetail {
                date: "20250731".to_string(),
                sub_totals: vec![
                    PaymentSubTotal {
                        tax_percent: 20.0,
                        currency_code: Some("EUR".to_string()),
                        amount: 50000.00,
                    },
                ],
            },
        }];

        let report = gen.create_aggregated_payments_report(
            "PAY-AGG-2025-001",
            "100000009",
            "VENDEUR",
            "20250701",
            "20250731",
            transactions,
        );

        let xml = gen.to_xml(&report).unwrap();

        assert!(xml.contains("<TypeCode>10.4</TypeCode>"));
        assert!(xml.contains("<Transactions>"));
        assert!(xml.contains("<Amount>50000.00</Amount>"));
        assert!(!xml.contains("<InvoiceID>"));
    }
}
