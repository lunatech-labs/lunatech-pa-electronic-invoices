use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::{
    DocumentAllowanceCharge, InvoiceAttachment, InvoiceData, InvoiceFormat, InvoiceLine,
    InvoiceNote, InvoiceProfile, PostalAddress, TaxBreakdown,
};
use base64::Engine as _;
use roxmltree::Document;

/// Parser pour les factures CII (Cross-Industry Invoice) / UN/CEFACT
pub struct CiiParser;

impl CiiParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse une facture CII depuis du XML brut
    pub fn parse(&self, xml: &str) -> PdpResult<InvoiceData> {
        let doc = Document::parse(xml)
            .map_err(|e| PdpError::ParseError(format!("XML CII invalide: {}", e)))?;

        let root = doc.root_element();
        let mut invoice = InvoiceData::new(String::new(), InvoiceFormat::CII);
        invoice.raw_xml = Some(xml.to_string());

        // ExchangedDocumentContext -> BT-23 (BusinessProcess) + BT-24 (Profile)
        if let Some(ctx) = self.find_element(&root, "ExchangedDocumentContext") {
            if let Some(bp) = self.find_element(&ctx, "BusinessProcessSpecifiedDocumentContextParameter") {
                invoice.business_process = self.find_text(&bp, "ID");
            }
            if let Some(gl) = self.find_element(&ctx, "GuidelineSpecifiedDocumentContextParameter") {
                let profile_uri = self.find_text(&gl, "ID");
                if let Some(ref uri) = profile_uri {
                    invoice.profile = if uri.contains("#Full") {
                        Some(InvoiceProfile::Full)
                    } else {
                        Some(InvoiceProfile::Base)
                    };
                }
                invoice.profile_id = profile_uri;
            }
        }

        // ExchangedDocument -> ID, TypeCode, IssueDateTime, IncludedNote
        if let Some(exch_doc) = self.find_element(&root, "ExchangedDocument") {
            invoice.invoice_number = self
                .find_text(&exch_doc, "ID")
                .unwrap_or_else(|| "INCONNU".to_string());

            invoice.invoice_type_code = self.find_text(&exch_doc, "TypeCode");
            invoice.issue_date = self.find_date(&exch_doc, "IssueDateTime");

            // Notes (BG-1)
            for note_node in exch_doc.children().filter(|n| n.tag_name().name() == "IncludedNote") {
                if let Some(content) = self.find_text(&note_node, "Content") {
                    invoice.notes.push(InvoiceNote {
                        content,
                        subject_code: self.find_text(&note_node, "SubjectCode"),
                    });
                }
            }
        }

        // SupplyChainTradeTransaction
        if let Some(transaction) = self.find_element(&root, "SupplyChainTradeTransaction") {
            // Lines (BG-25)
            self.parse_lines(&transaction, &mut invoice);
            self.parse_parties(&transaction, &mut invoice);
            self.parse_delivery(&transaction, &mut invoice);
            self.parse_settlement(&transaction, &mut invoice);
        }

        tracing::info!(
            invoice_number = %invoice.invoice_number,
            seller = invoice.seller_name.as_deref().unwrap_or("N/A"),
            buyer = invoice.buyer_name.as_deref().unwrap_or("N/A"),
            total_ttc = invoice.total_ttc.unwrap_or(0.0),
            "Facture CII parsée"
        );

        Ok(invoice)
    }

    /// Cherche un élément par son nom local dans les descendants
    fn find_element<'a>(&self, node: &'a roxmltree::Node, name: &str) -> Option<roxmltree::Node<'a, 'a>> {
        node.descendants().find(|n| n.tag_name().name() == name)
    }

    /// Cherche le texte d'un élément descendant
    fn find_text(&self, node: &roxmltree::Node, name: &str) -> Option<String> {
        node.descendants()
            .find(|n| n.tag_name().name() == name && n.text().is_some())
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Parse une date CII (format 102 = YYYYMMDD -> YYYY-MM-DD)
    fn find_date(&self, node: &roxmltree::Node, datetime_tag: &str) -> Option<String> {
        let dt_node = node.descendants().find(|n| n.tag_name().name() == datetime_tag)?;
        let date_str = dt_node
            .descendants()
            .find(|n| n.tag_name().name() == "DateTimeString")
            .and_then(|n| n.text())?
            .trim();

        // Convertir YYYYMMDD -> YYYY-MM-DD
        if date_str.len() == 8 && date_str.chars().all(|c| c.is_ascii_digit()) {
            Some(format!(
                "{}-{}-{}",
                &date_str[0..4],
                &date_str[4..6],
                &date_str[6..8]
            ))
        } else {
            Some(date_str.to_string())
        }
    }

    /// Parse les parties (vendeur / acheteur) depuis ApplicableHeaderTradeAgreement
    fn parse_parties(&self, transaction: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(agreement) = self.find_element(transaction, "ApplicableHeaderTradeAgreement") {
            // Référence acheteur (BT-10)
            invoice.buyer_reference = self.find_text(&agreement, "BuyerReference");

            // Vendeur (BG-4)
            if let Some(seller) = self.find_element(&agreement, "SellerTradeParty") {
                invoice.seller_name = self.find_text(&seller, "Name");
                invoice.seller_trading_name = self.extract_trading_name(&seller);
                invoice.seller_id = self.extract_global_id(&seller);
                invoice.seller_id_scheme = self.extract_global_id_scheme(&seller);
                invoice.seller_siret = self.extract_legal_id(&seller);
                invoice.seller_vat_id = self.extract_vat_id(&seller);
                invoice.seller_country = self.extract_country(&seller);
                invoice.seller_address = self.extract_address(&seller);
                invoice.seller_endpoint_id = self.extract_endpoint_id(&seller);
                invoice.seller_endpoint_scheme = self.extract_endpoint_scheme(&seller);
            }

            // Acheteur (BG-7)
            if let Some(buyer) = self.find_element(&agreement, "BuyerTradeParty") {
                invoice.buyer_name = self.find_text(&buyer, "Name");
                invoice.buyer_trading_name = self.extract_trading_name(&buyer);
                invoice.buyer_id = self.extract_global_id(&buyer);
                invoice.buyer_id_scheme = self.extract_global_id_scheme(&buyer);
                invoice.buyer_siret = self.extract_legal_id(&buyer);
                invoice.buyer_vat_id = self.extract_vat_id(&buyer);
                invoice.buyer_country = self.extract_country(&buyer);
                invoice.buyer_address = self.extract_address(&buyer);
                invoice.buyer_endpoint_id = self.extract_endpoint_id(&buyer);
                invoice.buyer_endpoint_scheme = self.extract_endpoint_scheme(&buyer);
            }

            // Représentant fiscal (BG-11)
            if let Some(tax_rep) = self.find_element(&agreement, "SellerTaxRepresentativeTradeParty") {
                invoice.tax_representative_name = self.find_text(&tax_rep, "Name");
                invoice.tax_representative_vat_id = self.extract_vat_id(&tax_rep);
                invoice.tax_representative_address = self.extract_address(&tax_rep);
            }

            // Agent du vendeur / Mandataire de facturation (marketplace A8)
            if let Some(agent) = self.find_element(&agreement, "SalesAgentTradeParty") {
                invoice.billing_mandate_name = self.find_text(&agent, "Name");
                invoice.billing_mandate_id = self.extract_legal_id(&agent);
            }

            // Agent de l'acheteur (EXT-FR-FE-BG-01, rôle AB)
            if let Some(buyer_agent) = self.find_element(&agreement, "BuyerAgentTradeParty") {
                invoice.buyer_agent_name = self.find_text(&buyer_agent, "Name");
                invoice.buyer_agent_id = self.extract_legal_id(&buyer_agent);
            }

            // Références (BT-13, BT-12, BT-11)
            if let Some(order_ref) = self.find_element(&agreement, "BuyerOrderReferencedDocument") {
                invoice.order_reference = self.find_text(&order_ref, "IssuerAssignedID");
            }
            if let Some(contract_ref) = self.find_element(&agreement, "ContractReferencedDocument") {
                invoice.contract_reference = self.find_text(&contract_ref, "IssuerAssignedID");
            }
            if let Some(project_ref) = self.find_element(&agreement, "SpecifiedProcuringProject") {
                invoice.project_reference = self.find_text(&project_ref, "ID");
            }

            // Pièces jointes (BG-24 : AdditionalReferencedDocument avec TypeCode 916)
            self.parse_attachments(&agreement, invoice);
        }
    }

    /// Parse les pièces jointes (BG-24 : AdditionalReferencedDocument)
    fn parse_attachments(&self, agreement: &roxmltree::Node, invoice: &mut InvoiceData) {
        for doc_ref in agreement.children().filter(|n| n.tag_name().name() == "AdditionalReferencedDocument") {
            // Seuls les TypeCode 916 (ou absent) sont des PJ BG-24
            let type_code = self.find_text(&doc_ref, "TypeCode");
            if let Some(ref tc) = type_code {
                if tc != "916" {
                    continue;
                }
            }

            let mut att = InvoiceAttachment {
                id: self.find_text(&doc_ref, "IssuerAssignedID"),
                description: self.find_text(&doc_ref, "Name"),
                external_uri: self.find_text(&doc_ref, "URIID"),
                embedded_content: None,
                mime_code: None,
                filename: None,
            };

            // AttachmentBinaryObject (contenu base64)
            if let Some(bin_obj) = doc_ref.descendants().find(|n| n.tag_name().name() == "AttachmentBinaryObject") {
                att.mime_code = bin_obj.attribute("mimeCode").map(|s| s.to_string());
                att.filename = bin_obj.attribute("filename").map(|s| s.to_string());
                if let Some(b64_text) = bin_obj.text() {
                    let b64_clean: String = b64_text.chars().filter(|c| !c.is_whitespace()).collect();
                    if !b64_clean.is_empty() {
                        if let Ok(decoded) = base64::Engine::decode(
                            &base64::engine::general_purpose::STANDARD,
                            &b64_clean,
                        ) {
                            att.embedded_content = Some(decoded);
                        }
                    }
                }
            }

            invoice.attachments.push(att);
        }
    }

    /// Extrait le nom commercial (BT-28/BT-45) via SpecifiedLegalOrganization/TradingBusinessName
    fn extract_trading_name(&self, party: &roxmltree::Node) -> Option<String> {
        let legal_org = self.find_element(party, "SpecifiedLegalOrganization")?;
        self.find_text(&legal_org, "TradingBusinessName")
    }

    /// Extrait le GlobalID (BT-29/BT-46) d'une TradeParty
    fn extract_global_id(&self, party: &roxmltree::Node) -> Option<String> {
        party.children()
            .find(|n| n.tag_name().name() == "GlobalID")
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
    }

    /// Extrait le schemeID du GlobalID
    fn extract_global_id_scheme(&self, party: &roxmltree::Node) -> Option<String> {
        party.children()
            .find(|n| n.tag_name().name() == "GlobalID")
            .and_then(|n| n.attribute("schemeID"))
            .map(|s| s.to_string())
    }

    /// Extrait l'EndpointID (BT-34/BT-49) via URIUniversalCommunication/URIID
    fn extract_endpoint_id(&self, party: &roxmltree::Node) -> Option<String> {
        let uri_comm = self.find_element(party, "URIUniversalCommunication")?;
        self.find_text(&uri_comm, "URIID")
    }

    /// Extrait le schemeID de l'EndpointID (BT-34-1/BT-49-1) via URIUniversalCommunication/URIID@schemeID
    fn extract_endpoint_scheme(&self, party: &roxmltree::Node) -> Option<String> {
        let uri_comm = self.find_element(party, "URIUniversalCommunication")?;
        let uriid = uri_comm.children().find(|n| n.has_tag_name("URIID") || n.tag_name().name() == "URIID")?;
        uriid.attribute("schemeID").map(|s| s.to_string())
    }

    /// Extrait l'ID légal (SIREN/SIRET) d'une TradeParty via SpecifiedLegalOrganization
    fn extract_legal_id(&self, party: &roxmltree::Node) -> Option<String> {
        if let Some(legal_org) = self.find_element(party, "SpecifiedLegalOrganization") {
            if let Some(id) = self.find_text(&legal_org, "ID") {
                return Some(id.replace(' ', ""));
            }
        }
        party
            .children()
            .find(|n| n.tag_name().name() == "ID")
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
    }

    /// Extrait le numéro TVA d'une TradeParty via SpecifiedTaxRegistration
    fn extract_vat_id(&self, party: &roxmltree::Node) -> Option<String> {
        if let Some(tax_reg) = self.find_element(party, "SpecifiedTaxRegistration") {
            return self.find_text(&tax_reg, "ID");
        }
        None
    }

    /// Extrait le code pays d'une TradeParty via PostalTradeAddress -> CountryID
    fn extract_country(&self, party: &roxmltree::Node) -> Option<String> {
        if let Some(addr) = self.find_element(party, "PostalTradeAddress") {
            return self.find_text(&addr, "CountryID");
        }
        None
    }

    /// Extrait l'adresse postale d'une TradeParty
    fn extract_address(&self, party: &roxmltree::Node) -> Option<PostalAddress> {
        let addr = self.find_element(party, "PostalTradeAddress")?;
        Some(PostalAddress {
            line1: self.find_text(&addr, "LineOne"),
            line2: self.find_text(&addr, "LineTwo"),
            line3: self.find_text(&addr, "LineThree"),
            city: self.find_text(&addr, "CityName"),
            postal_code: self.find_text(&addr, "PostcodeCode"),
            country_subdivision: self.find_text(&addr, "CountrySubDivisionName"),
            country_code: self.find_text(&addr, "CountryID"),
        })
    }

    /// Parse les informations de livraison (BG-13)
    fn parse_delivery(&self, transaction: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(delivery) = self.find_element(transaction, "ApplicableHeaderTradeDelivery") {
            // Date de livraison (BT-72)
            if let Some(event) = self.find_element(&delivery, "ActualDeliverySupplyChainEvent") {
                invoice.delivery_date = self.find_date(&event, "OccurrenceDateTime");
            }
            // Destinataire de la livraison (BT-70)
            if let Some(ship_to) = self.find_element(&delivery, "ShipToTradeParty") {
                invoice.delivery_party_name = self.find_text(&ship_to, "Name");
                invoice.delivery_address = self.extract_address(&ship_to);
            }
        }
    }

    /// Parse les données de règlement (totaux, TVA, devise, échéance, ventilation TVA)
    fn parse_settlement(&self, transaction: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(settlement) = self.find_element(transaction, "ApplicableHeaderTradeSettlement") {
            // Devise (BT-5)
            invoice.currency = self.find_text(&settlement, "InvoiceCurrencyCode");
            // Devise TVA (BT-6)
            invoice.tax_currency = self.find_text(&settlement, "TaxCurrencyCode");

            // Payeur tiers (sous-traitance avec délégation de paiement)
            if let Some(payer) = self.find_element(&settlement, "PayerTradeParty") {
                invoice.payer_name = self.find_text(&payer, "Name");
                invoice.payer_id = self.extract_legal_id(&payer);
            }

            // Facturant / Délégation de facturation (EXT-FR-FE-BG-05, rôle II)
            if let Some(invoicer) = self.find_element(&settlement, "InvoicerTradeParty") {
                invoice.invoicer_name = self.find_text(&invoicer, "Name");
                invoice.invoicer_id = self.extract_legal_id(&invoicer);
                invoice.invoicer_vat_id = self.extract_vat_id(&invoicer);
            }

            // Adressé à (EXT-FR-FE-BG-04, rôle IV)
            if let Some(addressed) = self.find_element(&settlement, "InvoiceeTradeParty") {
                invoice.addressed_to_name = self.find_text(&addressed, "Name");
                invoice.addressed_to_id = self.extract_legal_id(&addressed);
            }

            // Bénéficiaire du paiement (BG-10) — si différent du vendeur
            if let Some(payee) = self.find_element(&settlement, "PayeeTradeParty") {
                invoice.payee_name = self.find_text(&payee, "Name");
                invoice.payee_id = self.extract_global_id(&payee);
                invoice.payee_id_scheme = self.extract_global_id_scheme(&payee);
                invoice.payee_siret = self.extract_legal_id(&payee);
            }

            // Échéance (BT-9) + Conditions de paiement (BT-20)
            if let Some(terms) = self.find_element(&settlement, "SpecifiedTradePaymentTerms") {
                invoice.due_date = self.find_date(&terms, "DueDateDateTime");
                invoice.payment_terms = self.find_text(&terms, "Description");
            }

            // Référence comptable acheteur (BT-19)
            if let Some(acct) = self.find_element(&settlement, "ReceivableSpecifiedTradeAccountingAccount") {
                invoice.buyer_accounting_reference = self.find_text(&acct, "ID");
            }

            // Période de facturation (BG-14)
            if let Some(period) = self.find_element(&settlement, "BillingSpecifiedPeriod") {
                invoice.invoice_period_start = self.find_date(&period, "StartDateTime");
                invoice.invoice_period_end = self.find_date(&period, "EndDateTime");
            }

            // Moyens de paiement (BG-16)
            if let Some(pm) = self.find_element(&settlement, "SpecifiedTradeSettlementPaymentMeans") {
                invoice.payment_means_code = self.find_text(&pm, "TypeCode");
                if let Some(account) = self.find_element(&pm, "PayeePartyCreditorFinancialAccount") {
                    invoice.payment_iban = self.find_text(&account, "IBANID");
                }
                if let Some(inst) = self.find_element(&pm, "PayeeSpecifiedCreditorFinancialInstitution") {
                    invoice.payment_bic = self.find_text(&inst, "BICID");
                }
            }

            // Ventilation TVA (BG-23) — toutes les occurrences
            for tax_node in settlement.children().filter(|n| n.tag_name().name() == "ApplicableTradeTax") {
                invoice.tax_breakdowns.push(TaxBreakdown {
                    taxable_amount: self.find_text(&tax_node, "BasisAmount").and_then(|v| v.parse().ok()),
                    tax_amount: self.find_text(&tax_node, "CalculatedAmount").and_then(|v| v.parse().ok()),
                    category_code: self.find_text(&tax_node, "CategoryCode"),
                    percent: self.find_text(&tax_node, "RateApplicablePercent").and_then(|v| v.parse().ok()),
                    exemption_reason: self.find_text(&tax_node, "ExemptionReason"),
                    exemption_reason_code: self.find_text(&tax_node, "ExemptionReasonCode"),
                });
            }

            // Total TVA = somme des ventilations
            if invoice.total_tax.is_none() && !invoice.tax_breakdowns.is_empty() {
                invoice.total_tax = Some(
                    invoice.tax_breakdowns.iter()
                        .filter_map(|tb| tb.tax_amount)
                        .sum()
                );
            }

            // Remises/charges au niveau document (BG-20/BG-21)
            for ac_node in settlement.children().filter(|n| n.tag_name().name() == "SpecifiedTradeAllowanceCharge") {
                let charge = self.find_text(&ac_node, "ChargeIndicator")
                    .or_else(|| {
                        self.find_element(&ac_node, "ChargeIndicator")
                            .and_then(|ci| self.find_text(&ci, "Indicator"))
                    })
                    .map(|v| v == "true")
                    .unwrap_or(false);
                invoice.allowance_charges.push(DocumentAllowanceCharge {
                    charge_indicator: charge,
                    amount: self.find_text(&ac_node, "ActualAmount").and_then(|v| v.parse().ok()),
                    tax_category_code: self.find_text(&ac_node, "CategoryCode"),
                    tax_percent: self.find_text(&ac_node, "RateApplicablePercent").and_then(|v| v.parse().ok()),
                    reason: self.find_text(&ac_node, "Reason"),
                });
            }

            // Référence facture précédente (BT-25/BT-26) — pour les avoirs et factures définitives
            if let Some(inv_ref) = self.find_element(&settlement, "InvoiceReferencedDocument") {
                invoice.preceding_invoice_reference = self.find_text(&inv_ref, "IssuerAssignedID");
                invoice.preceding_invoice_date = self.find_date(&inv_ref, "FormattedIssueDateTime");
            }

            // Totaux (BG-22)
            if let Some(summary) = self.find_element(&settlement, "SpecifiedTradeSettlementHeaderMonetarySummation") {
                invoice.total_ht = self
                    .find_text(&summary, "TaxBasisTotalAmount")
                    .and_then(|v| v.parse::<f64>().ok());
                invoice.total_ttc = self
                    .find_text(&summary, "GrandTotalAmount")
                    .and_then(|v| v.parse::<f64>().ok());
                invoice.prepaid_amount = self
                    .find_text(&summary, "TotalPrepaidAmount")
                    .and_then(|v| v.parse::<f64>().ok());
                invoice.payable_amount = self
                    .find_text(&summary, "DuePayableAmount")
                    .and_then(|v| v.parse::<f64>().ok());
                invoice.allowance_total_amount = self
                    .find_text(&summary, "AllowanceTotalAmount")
                    .and_then(|v| v.parse::<f64>().ok());
                invoice.charge_total_amount = self
                    .find_text(&summary, "ChargeTotalAmount")
                    .and_then(|v| v.parse::<f64>().ok());
                invoice.rounding_amount = self
                    .find_text(&summary, "RoundingAmount")
                    .and_then(|v| v.parse::<f64>().ok());
            }
        }
    }

    /// Parse les lignes de facture (BG-25)
    fn parse_lines(&self, transaction: &roxmltree::Node, invoice: &mut InvoiceData) {
        for line_node in transaction.children().filter(|n| n.tag_name().name() == "IncludedSupplyChainTradeLineItem") {
            let mut line = InvoiceLine {
                line_id: None, note: None, object_id: None,
                quantity: None, unit_code: None,
                line_net_amount: None, order_line_reference: None, accounting_cost: None,
                price: None, gross_price: None,
                item_name: None, item_description: None,
                seller_item_id: None, buyer_item_id: None,
                standard_item_id: None, standard_item_id_scheme: None,
                tax_category_code: None, tax_percent: None,
                period_start: None, period_end: None,
            };

            // Line document
            if let Some(doc) = self.find_element(&line_node, "AssociatedDocumentLineDocument") {
                line.line_id = self.find_text(&doc, "LineID");
                if let Some(note_node) = self.find_element(&doc, "IncludedNote") {
                    line.note = self.find_text(&note_node, "Content");
                }
            }

            // Product (BG-31)
            if let Some(product) = self.find_element(&line_node, "SpecifiedTradeProduct") {
                line.item_name = self.find_text(&product, "Name");
                line.item_description = self.find_text(&product, "Description");
                line.seller_item_id = self.find_text(&product, "SellerAssignedID");
                line.buyer_item_id = self.find_text(&product, "BuyerAssignedID");
                // BT-158 : GlobalID (standard item ID)
                if let Some(gid_node) = product.children().find(|n| n.tag_name().name() == "GlobalID") {
                    line.standard_item_id = gid_node.text().map(|t| t.trim().to_string());
                    line.standard_item_id_scheme = gid_node.attribute("schemeID").map(|s| s.to_string());
                }
            }

            // Price (BG-29) + BT-132 order line reference
            if let Some(agreement) = self.find_element(&line_node, "SpecifiedLineTradeAgreement") {
                if let Some(net_price) = self.find_element(&agreement, "NetPriceProductTradePrice") {
                    line.price = self.find_text(&net_price, "ChargeAmount").and_then(|v| v.parse().ok());
                }
                if let Some(gross_price) = self.find_element(&agreement, "GrossPriceProductTradePrice") {
                    line.gross_price = self.find_text(&gross_price, "ChargeAmount").and_then(|v| v.parse().ok());
                }
                // BT-132 : Order line reference
                if let Some(order_ref) = self.find_element(&agreement, "BuyerOrderReferencedDocument") {
                    line.order_line_reference = self.find_text(&order_ref, "LineID");
                }
            }

            // Quantity (BT-129/BT-130)
            if let Some(delivery) = self.find_element(&line_node, "SpecifiedLineTradeDelivery") {
                if let Some(qty_node) = delivery.children().find(|n| n.tag_name().name() == "BilledQuantity") {
                    line.quantity = qty_node.text().and_then(|t| t.trim().parse().ok());
                    line.unit_code = qty_node.attribute("unitCode").map(|s| s.to_string());
                }
            }

            // Settlement (line net amount, tax, period, accounting cost)
            if let Some(settle) = self.find_element(&line_node, "SpecifiedLineTradeSettlement") {
                // Tax
                if let Some(tax) = self.find_element(&settle, "ApplicableTradeTax") {
                    line.tax_category_code = self.find_text(&tax, "CategoryCode");
                    line.tax_percent = self.find_text(&tax, "RateApplicablePercent").and_then(|v| v.parse().ok());
                }
                // BT-133 : Accounting cost
                if let Some(acct) = self.find_element(&settle, "ReceivableSpecifiedTradeAccountingAccount") {
                    line.accounting_cost = self.find_text(&acct, "ID");
                }
                // Line net amount
                if let Some(sum) = self.find_element(&settle, "SpecifiedTradeSettlementLineMonetarySummation") {
                    line.line_net_amount = self.find_text(&sum, "LineTotalAmount").and_then(|v| v.parse().ok());
                }
                // Period (BG-26)
                if let Some(period) = self.find_element(&settle, "BillingSpecifiedPeriod") {
                    line.period_start = self.find_date(&period, "StartDateTime");
                    line.period_end = self.find_date(&period, "EndDateTime");
                }
            }

            invoice.lines.push(line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_cii_fixture() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_001.xml")
            .expect("Fixture CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing CII");

        assert_eq!(invoice.invoice_number, "FA-2025-00256");
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-11-20"));
        assert_eq!(invoice.due_date.as_deref(), Some("2026-01-15"));
        assert_eq!(invoice.currency.as_deref(), Some("EUR"));
        assert_eq!(invoice.seller_name.as_deref(), Some("InfoTech Solutions SARL"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Manufacture Lyonnaise SAS"));
        assert_eq!(invoice.total_ht, Some(32000.00));
        assert_eq!(invoice.total_ttc, Some(38400.00));
        assert_eq!(invoice.total_tax, Some(6400.00));
        assert_eq!(invoice.source_format, InvoiceFormat::CII);
    }

    #[test]
    fn test_parse_cii_autofacture_389() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/autofacture_cii_389.xml")
            .expect("Fixture auto-facture CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing auto-facture CII");

        assert_eq!(invoice.invoice_number, "AF-2025-00012");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("389"));
        assert_eq!(invoice.business_process.as_deref(), Some("A9"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-12-01"));
        assert_eq!(invoice.seller_name.as_deref(), Some("Ferme Bio du Vercors EARL"));
        assert_eq!(invoice.seller_trading_name.as_deref(), Some("Ferme Bio du Vercors"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Laiterie des Alpes SAS"));
        assert_eq!(invoice.buyer_trading_name.as_deref(), Some("Laiterie des Alpes"));
        assert_eq!(invoice.contract_reference.as_deref(), Some("CONTRAT-AF-2025-001"));
        assert_eq!(invoice.total_ht, Some(15600.00));
        assert_eq!(invoice.total_ttc, Some(16458.00));
        assert_eq!(invoice.total_tax, Some(858.00));
        assert_eq!(invoice.payment_terms.as_deref(), Some("Paiement à 30 jours"));
        assert_eq!(invoice.lines.len(), 2);
        // Line 1 : item IDs
        let l1 = &invoice.lines[0];
        assert_eq!(l1.item_name.as_deref(), Some("Lait bio 5L"));
        assert_eq!(l1.seller_item_id.as_deref(), Some("LAIT-BIO-5L"));
        assert_eq!(l1.buyer_item_id.as_deref(), Some("MP-LAIT-001"));
        assert_eq!(l1.standard_item_id.as_deref(), Some("3456789012345"));
        assert_eq!(l1.standard_item_id_scheme.as_deref(), Some("0160"));
        assert_eq!(l1.tax_percent, Some(5.50));
    }

    #[test]
    fn test_parse_cii_avoir_381() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/avoir_cii_381.xml")
            .expect("Fixture avoir CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing avoir CII");

        assert_eq!(invoice.invoice_number, "AV-2025-00045");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("381"));
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("FA-2025-00256"));
        assert_eq!(invoice.total_ht, Some(8500.00));
        assert_eq!(invoice.total_ttc, Some(10200.00));
        assert_eq!(invoice.lines.len(), 1);
    }

    #[test]
    fn test_parse_cii_remises_multitva() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_remises_multitva.xml")
            .expect("Fixture multi-TVA CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing multi-TVA CII");

        assert_eq!(invoice.invoice_number, "FA-2025-00300");
        assert_eq!(invoice.business_process.as_deref(), Some("A1"));
        assert_eq!(invoice.lines.len(), 3);
        // 3 ventilations TVA (20%, 5.5%, exonéré)
        assert_eq!(invoice.tax_breakdowns.len(), 3);
        // Remises/charges document
        assert_eq!(invoice.allowance_charges.len(), 2);
        assert!(!invoice.allowance_charges[0].charge_indicator); // remise
        assert!(invoice.allowance_charges[1].charge_indicator);  // charge
        assert_eq!(invoice.allowance_charges[0].amount, Some(515.00));
        assert_eq!(invoice.allowance_charges[1].amount, Some(150.00));
        // Totaux
        assert_eq!(invoice.allowance_total_amount, Some(515.00));
        assert_eq!(invoice.charge_total_amount, Some(150.00));
        assert_eq!(invoice.total_ttc, Some(10711.50));
        // Ligne exonérée
        let l3 = &invoice.lines[2];
        assert_eq!(l3.tax_category_code.as_deref(), Some("E"));
        assert_eq!(l3.tax_percent, Some(0.00));
        // Gross price sur ligne 1
        assert_eq!(invoice.lines[0].gross_price, Some(450.00));
        assert_eq!(invoice.lines[0].price, Some(400.00));
    }

    // ===== Tests sur exemples officiels AFNOR XP Z12-014 v1.2 =====

    #[test]
    fn test_parse_official_uc1_cii_standard_invoice() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_CII.xml")
            .expect("Fixture officielle UC1 CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing UC1 CII");

        assert_eq!(invoice.invoice_number, "F202500003");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        assert_eq!(invoice.business_process.as_deref(), Some("S1"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-01"));
        assert_eq!(invoice.due_date.as_deref(), Some("2025-07-31"));
        assert_eq!(invoice.currency.as_deref(), Some("EUR"));
        // Vendeur
        assert_eq!(invoice.seller_name.as_deref(), Some("LE VENDEUR"));
        assert_eq!(invoice.seller_trading_name.as_deref(), Some("VENDEUR NOM COMMERCIAL"));
        assert_eq!(invoice.seller_id.as_deref(), Some("587451236587"));
        assert_eq!(invoice.seller_id_scheme.as_deref(), Some("0088"));
        assert_eq!(invoice.seller_siret.as_deref(), Some("100000009"));
        assert_eq!(invoice.seller_vat_id.as_deref(), Some("FR88100000009"));
        assert_eq!(invoice.seller_country.as_deref(), Some("FR"));
        assert_eq!(invoice.seller_endpoint_id.as_deref(), Some("100000009_STATUTS"));
        // Acheteur
        assert_eq!(invoice.buyer_name.as_deref(), Some("LE CLIENT"));
        assert_eq!(invoice.buyer_siret.as_deref(), Some("200000008"));
        assert_eq!(invoice.buyer_vat_id.as_deref(), Some("FR37200000008"));
        assert_eq!(invoice.buyer_endpoint_id.as_deref(), Some("200000008"));
        // Références
        assert_eq!(invoice.buyer_reference.as_deref(), Some("BU_2516"));
        assert_eq!(invoice.order_reference.as_deref(), Some("PO202525478"));
        assert_eq!(invoice.buyer_accounting_reference.as_deref(), Some("REF COMPTABLE ACHETEUR"));
        // Livraison (BT-70)
        assert_eq!(invoice.delivery_party_name.as_deref(), Some("NOUS AUSSI"));
        assert!(invoice.delivery_address.is_some());
        let addr = invoice.delivery_address.as_ref().unwrap();
        assert_eq!(addr.city.as_deref(), Some("MA VILLE"));
        assert_eq!(addr.postal_code.as_deref(), Some("06000"));
        // Paiement
        assert_eq!(invoice.payment_means_code.as_deref(), Some("30"));
        assert_eq!(invoice.payment_terms.as_deref(), Some("PAIEMENT 30 JOURS NET"));
        assert_eq!(invoice.payment_iban.as_deref(), Some("FR20 1254 2547 2569 8542 5874 698"));
        assert_eq!(invoice.payment_bic.as_deref(), Some("BIC_MONCOMPTE"));
        // Période
        assert_eq!(invoice.invoice_period_start.as_deref(), Some("2025-06-01"));
        assert_eq!(invoice.invoice_period_end.as_deref(), Some("2025-06-30"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(10000.00));
        assert_eq!(invoice.total_ttc, Some(12000.00));
        assert_eq!(invoice.prepaid_amount, Some(0.00));
        assert_eq!(invoice.payable_amount, Some(12000.00));
        assert_eq!(invoice.allowance_total_amount, Some(0.00));
        assert_eq!(invoice.charge_total_amount, Some(0.00));
        // TVA
        assert_eq!(invoice.tax_breakdowns.len(), 1);
        assert_eq!(invoice.tax_breakdowns[0].category_code.as_deref(), Some("S"));
        assert_eq!(invoice.tax_breakdowns[0].percent, Some(20.00));
        assert_eq!(invoice.tax_breakdowns[0].taxable_amount, Some(10000.00));
        assert_eq!(invoice.tax_breakdowns[0].tax_amount, Some(2000.00));
        // Lignes
        assert_eq!(invoice.lines.len(), 2);
        assert_eq!(invoice.lines[0].item_name.as_deref(), Some("SERVICE_FOURNI1"));
        assert_eq!(invoice.lines[0].price, Some(40.0));
        assert_eq!(invoice.lines[0].quantity, Some(200.0));
        assert_eq!(invoice.lines[0].line_net_amount, Some(8000.00));
        assert_eq!(invoice.lines[1].item_name.as_deref(), Some("SERVICE_FOURNI2"));
        assert_eq!(invoice.lines[1].line_net_amount, Some(2000.00));
        // Notes
        assert!(invoice.notes.len() >= 5);
    }

    #[test]
    fn test_parse_official_uc4b_cii_corrective_384() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC4/UC4b_F202500010_00-INVCORR/UC4b_F202500010_00-INVCORR_20250702_CII.xml")
            .expect("Fixture officielle UC4b CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing UC4b CII");

        assert_eq!(invoice.invoice_number, "F202500010");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("384"));
        assert_eq!(invoice.business_process.as_deref(), Some("S1"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-02"));
        // Référence facture antérieure (BT-25/BT-26)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("F20250006"));
        assert_eq!(invoice.preceding_invoice_date.as_deref(), Some("2025-07-01"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(10000.00));
        assert_eq!(invoice.total_ttc, Some(12000.00));
    }

    #[test]
    fn test_parse_official_uc5b_cii_credit_note_381() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC5/UC5b_F202500011_00-CN_20250703/UC5b_F202500011_00-CN_20250703_CII.xml")
            .expect("Fixture officielle UC5b CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing UC5b CII");

        assert_eq!(invoice.invoice_number, "F202500011");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("381"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-03"));
        // Référence facture antérieure (BT-25/BT-26)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("F20250007"));
        assert_eq!(invoice.preceding_invoice_date.as_deref(), Some("2025-07-02"));
        // Livraison
        assert_eq!(invoice.delivery_party_name.as_deref(), Some("NOUS AUSSI"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(10000.00));
        assert_eq!(invoice.total_ttc, Some(11000.00));
        assert_eq!(invoice.tax_breakdowns[0].percent, Some(10.00));
    }

    // ===== Tests sur fixtures métier (sous-traitance, marketplace, acompte) =====

    #[test]
    fn test_parse_cii_soustraitance_a4() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_soustraitance_a4.xml")
            .expect("Fixture sous-traitance CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing sous-traitance CII");

        assert_eq!(invoice.invoice_number, "ST-2025-00042");
        assert_eq!(invoice.business_process.as_deref(), Some("A4"));
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        // Vendeur = sous-traitant
        assert_eq!(invoice.seller_name.as_deref(), Some("Plomberie Durand SARL"));
        // Acheteur = entrepreneur principal
        assert_eq!(invoice.buyer_name.as_deref(), Some("Construction Générale Martin SA"));
        // Payeur tiers = maître d'ouvrage
        assert_eq!(invoice.payer_name.as_deref(), Some("SCI Résidence Les Érables"));
        assert_eq!(invoice.payer_id.as_deref(), Some("55566677788899"));
        // Contrat
        assert_eq!(invoice.contract_reference.as_deref(), Some("MARCHE-ERABLES-2025"));
        // Autoliquidation TVA
        assert_eq!(invoice.tax_breakdowns[0].category_code.as_deref(), Some("AE"));
        assert_eq!(invoice.tax_breakdowns[0].exemption_reason_code.as_deref(), Some("VATEX-EU-AE"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(63500.00));
        assert_eq!(invoice.total_ttc, Some(63500.00));
        assert_eq!(invoice.payable_amount, Some(63500.00));
        assert_eq!(invoice.lines.len(), 2);
    }

    #[test]
    fn test_parse_cii_marketplace_a8() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_marketplace_a8.xml")
            .expect("Fixture marketplace CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing marketplace CII");

        assert_eq!(invoice.invoice_number, "MKP-2025-00789");
        assert_eq!(invoice.business_process.as_deref(), Some("A8"));
        // Vendeur = fournisseur réel
        assert_eq!(invoice.seller_name.as_deref(), Some("AudioTech France SAS"));
        // Acheteur = client
        assert_eq!(invoice.buyer_name.as_deref(), Some("Électronique Distribution SARL"));
        // Mandataire de facturation (SalesAgentTradeParty)
        assert_eq!(invoice.billing_mandate_name.as_deref(), Some("MarketPlace Pro SAS"));
        assert_eq!(invoice.billing_mandate_id.as_deref(), Some("11122233344455"));
        // Déjà payée
        assert_eq!(invoice.prepaid_amount, Some(8334.90));
        assert_eq!(invoice.payable_amount, Some(0.00));
        assert_eq!(invoice.total_ttc, Some(8334.90));
        assert_eq!(invoice.lines.len(), 2);
    }

    #[test]
    fn test_parse_cii_acompte() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_acompte.xml")
            .expect("Fixture acompte CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing acompte CII");

        assert_eq!(invoice.invoice_number, "FA-2025-ACOMPTE-001");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("386"));
        assert_eq!(invoice.business_process.as_deref(), Some("A1"));
        assert_eq!(invoice.total_ht, Some(4500.00));
        assert_eq!(invoice.total_ttc, Some(4950.00));
        assert_eq!(invoice.payable_amount, Some(4950.00));
        assert_eq!(invoice.contract_reference.as_deref(), Some("DEV-2025-100"));
    }

    #[test]
    fn test_parse_cii_definitive_apres_acompte() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_definitive_apres_acompte.xml")
            .expect("Fixture définitive CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing définitive CII");

        assert_eq!(invoice.invoice_number, "FA-2025-DEF-002");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        // Référence à l'acompte (BT-25)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("FA-2025-ACOMPTE-001"));
        assert_eq!(invoice.preceding_invoice_date.as_deref(), Some("2025-10-01"));
        // Totaux avec déduction acompte
        assert_eq!(invoice.total_ht, Some(15000.00));
        assert_eq!(invoice.total_ttc, Some(16500.00));
        assert_eq!(invoice.prepaid_amount, Some(4950.00));
        assert_eq!(invoice.payable_amount, Some(11550.00));
        assert_eq!(invoice.lines.len(), 4);
    }

    #[test]
    fn test_parse_cii_invalid_xml() {
        let parser = CiiParser::new();
        let result = parser.parse("<broken>xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cii_date_format_102() {
        let parser = CiiParser::new();
        let xml = r#"<?xml version="1.0"?>
        <rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
                                  xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
                                  xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
            <rsm:ExchangedDocument>
                <ram:ID>TEST-001</ram:ID>
                <ram:TypeCode>380</ram:TypeCode>
                <ram:IssueDateTime>
                    <udt:DateTimeString format="102">20251225</udt:DateTimeString>
                </ram:IssueDateTime>
            </rsm:ExchangedDocument>
            <rsm:SupplyChainTradeTransaction>
                <ram:ApplicableHeaderTradeAgreement>
                    <ram:SellerTradeParty><ram:Name>Test Seller</ram:Name></ram:SellerTradeParty>
                    <ram:BuyerTradeParty><ram:Name>Test Buyer</ram:Name></ram:BuyerTradeParty>
                </ram:ApplicableHeaderTradeAgreement>
                <ram:ApplicableHeaderTradeDelivery/>
                <ram:ApplicableHeaderTradeSettlement>
                    <ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
                    <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
                        <ram:GrandTotalAmount>100.00</ram:GrandTotalAmount>
                    </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
                </ram:ApplicableHeaderTradeSettlement>
            </rsm:SupplyChainTradeTransaction>
        </rsm:CrossIndustryInvoice>"#;

        let invoice = parser.parse(xml).expect("Parsing failed");
        assert_eq!(invoice.invoice_number, "TEST-001");
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-12-25"));
    }

    #[test]
    fn test_parse_cii_rectificative_384() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_rectificative_cii_384.xml")
            .expect("Fixture rectificative CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing rectificative CII");

        assert_eq!(invoice.invoice_number, "RECT-2025-00001");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("384"));
        assert_eq!(invoice.business_process.as_deref(), Some("S1"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-15"));
        assert_eq!(invoice.due_date.as_deref(), Some("2025-08-14"));
        // Référence facture antérieure (BT-25/BT-26)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("FA-2025-00100"));
        assert_eq!(invoice.preceding_invoice_date.as_deref(), Some("2025-07-01"));
        // Vendeur
        assert_eq!(invoice.seller_name.as_deref(), Some("Conseil Expert SAS"));
        assert_eq!(invoice.seller_siret.as_deref(), Some("12345678901234"));
        assert_eq!(invoice.seller_vat_id.as_deref(), Some("FR12123456789"));
        assert_eq!(invoice.seller_endpoint_id.as_deref(), Some("123456789_STATUTS"));
        // Acheteur
        assert_eq!(invoice.buyer_name.as_deref(), Some("Industries Modernes SA"));
        assert_eq!(invoice.buyer_siret.as_deref(), Some("98765432109876"));
        assert_eq!(invoice.buyer_endpoint_id.as_deref(), Some("987654321_FACTURES"));
        // Paiement
        assert_eq!(invoice.payment_means_code.as_deref(), Some("30"));
        assert_eq!(invoice.payment_iban.as_deref(), Some("FR7630001007941234567890185"));
        assert_eq!(invoice.payment_bic.as_deref(), Some("BNPAFRPPXXX"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(6000.00));
        assert_eq!(invoice.total_ttc, Some(7200.00));
        assert_eq!(invoice.payable_amount, Some(7200.00));
        assert_eq!(invoice.tax_breakdowns.len(), 1);
        assert_eq!(invoice.tax_breakdowns[0].percent, Some(20.00));
        // Notes
        assert!(invoice.notes.len() >= 5);
        assert!(invoice.notes.iter().any(|n| n.subject_code.as_deref() == Some("ADN")));
        assert!(invoice.notes.iter().any(|n| n.subject_code.as_deref() == Some("BAR")));
        // Lignes
        assert_eq!(invoice.lines.len(), 1);
        assert_eq!(invoice.lines[0].item_name.as_deref(), Some("Prestation de conseil - prix corrigé"));
        assert_eq!(invoice.lines[0].quantity, Some(50.0));
        assert_eq!(invoice.lines[0].price, Some(120.00));
        assert_eq!(invoice.lines[0].line_net_amount, Some(6000.00));
    }

    #[test]
    fn test_parse_cii_delegation_s8() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_delegation_s8.xml")
            .expect("Fixture délégation CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing délégation CII");

        assert_eq!(invoice.invoice_number, "DELEG-2025-00001");
        assert_eq!(invoice.business_process.as_deref(), Some("S8"));
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        // Vendeur = artisan
        assert_eq!(invoice.seller_name.as_deref(), Some("Menuiserie Artisanale Dupont EURL"));
        assert_eq!(invoice.seller_siret.as_deref(), Some("11122233344455"));
        // Acheteur = filiale
        assert_eq!(invoice.buyer_name.as_deref(), Some("Hôtellerie du Sud-Ouest SAS"));
        assert_eq!(invoice.buyer_siret.as_deref(), Some("66677788899900"));
        // Facturant (II) = cabinet comptable
        assert_eq!(invoice.invoicer_name.as_deref(), Some("Cabinet Comptable Gironde SARL"));
        assert_eq!(invoice.invoicer_id.as_deref(), Some("99988877766655"));
        assert_eq!(invoice.invoicer_vat_id.as_deref(), Some("FR99988877766"));
        // Adressé à (IV) = siège du groupe
        assert_eq!(invoice.addressed_to_name.as_deref(), Some("Groupe Hôtelier Atlantique SA"));
        assert_eq!(invoice.addressed_to_id.as_deref(), Some("55544433322211"));
        // Agent de l'acheteur (AB) = service achats
        assert_eq!(invoice.buyer_agent_name.as_deref(), Some("Service Achats Groupe Atlantique"));
        assert_eq!(invoice.buyer_agent_id.as_deref(), Some("55544433322299"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(4460.00));
        assert_eq!(invoice.total_ttc, Some(5352.00));
        assert_eq!(invoice.lines.len(), 2);
    }

    #[test]
    fn test_parse_cii_multivendeurs_b8() {
        let xml = fs::read_to_string("../../tests/fixtures/cii/facture_cii_multivendeurs_b8.xml")
            .expect("Fixture multi-vendeurs CII introuvable");

        let parser = CiiParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing multi-vendeurs CII");

        assert_eq!(invoice.invoice_number, "MV-2025-00100");
        assert_eq!(invoice.business_process.as_deref(), Some("B8"));
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        // Vendeur principal = plombier
        assert_eq!(invoice.seller_name.as_deref(), Some("Plomberie Martin SARL"));
        assert_eq!(invoice.seller_siret.as_deref(), Some("22233344455566"));
        // Acheteur = propriétaire
        assert_eq!(invoice.buyer_name.as_deref(), Some("SCI Les Tilleuls"));
        // Facturant (II) = plateforme
        assert_eq!(invoice.invoicer_name.as_deref(), Some("ArtisanConnect SAS"));
        assert_eq!(invoice.invoicer_id.as_deref(), Some("33344455566677"));
        assert_eq!(invoice.invoicer_vat_id.as_deref(), Some("FR33344455566"));
        // Multi-TVA (10% + 20%)
        assert_eq!(invoice.tax_breakdowns.len(), 2);
        // 3 lignes (plombier + électricien + commission)
        assert_eq!(invoice.lines.len(), 3);
        assert_eq!(invoice.total_ht, Some(2255.00));
        assert_eq!(invoice.total_ttc, Some(2501.00));
    }
}
