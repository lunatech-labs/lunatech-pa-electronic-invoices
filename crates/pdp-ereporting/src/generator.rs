use chrono::Utc;
use pdp_core::error::PdpResult;
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
            }),
            payments: None,
        }
    }

    /// Convertit une InvoiceData en TransactionInvoice pour le e-reporting
    pub fn invoice_to_transaction(invoice: &InvoiceData) -> TransactionInvoice {
        let seller_siret = invoice.seller_siret.as_deref().unwrap_or("");
        let seller_siren = if seller_siret.len() >= 9 { &seller_siret[..9] } else { seller_siret };

        let buyer_siret = invoice.buyer_siret.as_deref().unwrap_or("");
        let buyer_siren = if buyer_siret.len() >= 9 { &buyer_siret[..9] } else { buyer_siret };

        TransactionInvoice {
            id: invoice.invoice_number.clone(),
            issue_date: invoice.issue_date.clone().unwrap_or_default().replace('-', ""),
            type_code: invoice.invoice_type_code.clone().unwrap_or_else(|| "380".to_string()),
            currency_code: invoice.currency.clone().unwrap_or_else(|| "EUR".to_string()),
            due_date: invoice.due_date.clone().map(|d| d.replace('-', "")),
            tax_due_date_type_code: None,
            notes: Vec::new(),
            business_process: BusinessProcess {
                id: "B2C".to_string(),
                type_id: "urn.cpro.gouv.fr:1p0:ereporting".to_string(),
            },
            referenced_documents: Vec::new(),
            seller: TransactionParty {
                company_id: Some(seller_siren.to_string()),
                company_id_scheme: Some("0002".to_string()),
                tax_registration_id: None,
                tax_qualifying_id: None,
                country_code: Some("FR".to_string()),
            },
            buyer: if !buyer_siren.is_empty() {
                Some(TransactionParty {
                    company_id: Some(buyer_siren.to_string()),
                    company_id_scheme: Some("0002".to_string()),
                    tax_registration_id: None,
                    tax_qualifying_id: None,
                    country_code: None,
                })
            } else {
                None
            },
            seller_tax_representative: None,
            deliveries: Vec::new(),
            invoice_period: None,
            allowance_charges: Vec::new(),
            monetary_total: MonetaryTotal {
                tax_exclusive_amount: invoice.total_ht,
                tax_amount: invoice.total_tax.unwrap_or(0.0),
                tax_amount_currency: invoice.currency.clone().unwrap_or_else(|| "EUR".to_string()),
            },
            tax_subtotals: vec![TaxSubTotal {
                taxable_amount: invoice.total_ht.unwrap_or(0.0),
                tax_amount: invoice.total_tax.unwrap_or(0.0),
                tax_category_code: Some("S".to_string()),
                tax_percent: 20.0,
                tax_exemption_reason: None,
                tax_exemption_reason_code: None,
            }],
            lines: Vec::new(),
        }
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

        for inv in &txns.invoices {
            self.write_transaction_invoice(xml, inv);
        }

        xml.push_str("\n    </TransactionsReport>");
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
        inv.total_ht = Some(1000.00);
        inv.total_tax = Some(200.00);
        inv.total_ttc = Some(1200.00);
        inv.currency = Some("EUR".to_string());
        inv.invoice_type_code = Some("380".to_string());
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
        assert!(xml.contains("<TaxableAmount>1000.00</TaxableAmount>"));
        assert!(xml.contains("<StartDate>20250701</StartDate>"));
        assert!(xml.contains("<EndDate>20250731</EndDate>"));
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
