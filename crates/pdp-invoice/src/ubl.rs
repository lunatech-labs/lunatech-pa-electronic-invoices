use pdp_core::error::{PdpError, PdpResult};
use pdp_core::model::{
    DocumentAllowanceCharge, InvoiceAttachment, InvoiceData, InvoiceFormat, InvoiceLine,
    InvoiceNote, InvoiceProfile, PostalAddress, TaxBreakdown,
};
use roxmltree::Document;

/// Parser pour les factures UBL (Universal Business Language)
pub struct UblParser;

impl UblParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse une facture UBL depuis du XML brut
    pub fn parse(&self, xml: &str) -> PdpResult<InvoiceData> {
        let doc = Document::parse(xml)
            .map_err(|e| PdpError::ParseError(format!("XML UBL invalide: {}", e)))?;

        let root = doc.root_element();
        let mut invoice = InvoiceData::new(String::new(), InvoiceFormat::UBL);
        invoice.raw_xml = Some(xml.to_string());

        // BT-24 : Profil (CustomizationID)
        if let Some(cust_id) = self.find_cbc_text(&root, "CustomizationID") {
            invoice.profile = if cust_id.contains("#Full") {
                Some(InvoiceProfile::Full)
            } else {
                Some(InvoiceProfile::Base)
            };
            invoice.profile_id = Some(cust_id);
        }

        // BT-23 : Cadre de facturation (ProfileID)
        invoice.business_process = self.find_cbc_text(&root, "ProfileID");

        // BT-1 : Numéro de facture
        invoice.invoice_number = self
            .find_cbc_text(&root, "ID")
            .unwrap_or_else(|| "INCONNU".to_string());

        // BT-3 : Type de document
        invoice.invoice_type_code = self.find_cbc_text(&root, "InvoiceTypeCode")
            .or_else(|| self.find_cbc_text(&root, "CreditNoteTypeCode"));

        // BT-2 : Date d'émission
        invoice.issue_date = self.find_cbc_text(&root, "IssueDate");

        // BT-9 : Date d'échéance
        invoice.due_date = self.find_cbc_text(&root, "DueDate");

        // BT-5 : Devise
        invoice.currency = self.find_cbc_text(&root, "DocumentCurrencyCode");

        // BT-6 : Devise TVA
        invoice.tax_currency = self.find_cbc_text(&root, "TaxCurrencyCode");

        // BT-10 : Référence acheteur
        invoice.buyer_reference = self.find_cbc_text(&root, "BuyerReference");

        // Notes (BG-1)
        for note_node in root.children().filter(|n| n.tag_name().name() == "Note") {
            if let Some(text) = note_node.text() {
                invoice.notes.push(InvoiceNote {
                    content: text.trim().to_string(),
                    subject_code: None,
                });
            }
        }

        // BT-13 : Référence de commande
        if let Some(order_ref) = self.find_element(&root, "OrderReference") {
            invoice.order_reference = self.find_cbc_text(&order_ref, "ID");
        }

        // BT-25/BT-26 : Référence facture précédente
        if let Some(billing_ref) = self.find_element(&root, "BillingReference") {
            if let Some(inv_doc_ref) = self.find_element(&billing_ref, "InvoiceDocumentReference") {
                invoice.preceding_invoice_reference = self.find_cbc_text(&inv_doc_ref, "ID");
                invoice.preceding_invoice_date = self.find_cbc_text(&inv_doc_ref, "IssueDate");
            }
        }

        // BT-12 : Référence contrat
        if let Some(contract_ref) = self.find_element(&root, "ContractDocumentReference") {
            invoice.contract_reference = self.find_cbc_text(&contract_ref, "ID");
        }

        // BT-11 : Référence projet
        if let Some(project_ref) = self.find_element(&root, "ProjectReference") {
            invoice.project_reference = self.find_cbc_text(&project_ref, "ID");
        }

        // BT-19 : Référence comptable acheteur
        invoice.buyer_accounting_reference = self.find_cbc_text(&root, "AccountingCost");

        // Période de facturation (BG-14)
        if let Some(period) = self.find_element(&root, "InvoicePeriod") {
            invoice.invoice_period_start = self.find_cbc_text(&period, "StartDate");
            invoice.invoice_period_end = self.find_cbc_text(&period, "EndDate");
        }

        // Vendeur (BG-4)
        self.parse_seller(&root, &mut invoice);

        // Acheteur (BG-7)
        self.parse_buyer(&root, &mut invoice);

        // BG-10 : Bénéficiaire du paiement (PayeeParty)
        if let Some(payee) = self.find_element(&root, "PayeeParty") {
            if let Some(party_name) = self.find_element(&payee, "PartyName") {
                invoice.payee_name = self.find_cbc_text(&party_name, "Name");
            }
            if let Some(ident) = self.find_element(&payee, "PartyIdentification") {
                if let Some(id_node) = ident.children().find(|n| n.tag_name().name() == "ID") {
                    invoice.payee_id = id_node.text().map(|t| t.trim().to_string());
                    invoice.payee_id_scheme = id_node.attribute("schemeID").map(|s| s.to_string());
                }
            }
            invoice.payee_siret = self.extract_legal_id(&payee);
        }

        // Facturant / Délégation de facturation (EXT-FR-FE-BG-05, rôle II)
        // En UBL, représenté par un élément InvoicerParty ou via AccountingSupplierParty extension
        if let Some(invoicer_party) = self.find_element(&root, "InvoicerParty") {
            self.parse_party_fields(&invoicer_party,
                &mut invoice.invoicer_name,
                &mut invoice.invoicer_id,
                &mut invoice.invoicer_vat_id);
        }

        // Agent de l'acheteur (EXT-FR-FE-BG-01, rôle AB)
        if let Some(buyer_agent) = self.find_element(&root, "BuyerAgentParty") {
            if let Some(party) = self.find_element(&buyer_agent, "Party") {
                if let Some(pn) = self.find_element(&party, "PartyName") {
                    invoice.buyer_agent_name = self.find_cbc_text(&pn, "Name");
                }
                invoice.buyer_agent_id = self.extract_legal_id(&party);
            }
        }

        // Adressé à (EXT-FR-FE-BG-04, rôle IV)
        if let Some(addressed) = self.find_element(&root, "AddressedToParty") {
            if let Some(party) = self.find_element(&addressed, "Party") {
                if let Some(pn) = self.find_element(&party, "PartyName") {
                    invoice.addressed_to_name = self.find_cbc_text(&pn, "Name");
                }
                invoice.addressed_to_id = self.extract_legal_id(&party);
            }
        }

        // BG-11 : Représentant fiscal
        self.parse_tax_representative(&root, &mut invoice);

        // Livraison (BG-13)
        self.parse_delivery(&root, &mut invoice);

        // Moyens de paiement (BG-16)
        self.parse_payment_means(&root, &mut invoice);

        // BT-20 : Conditions de paiement
        if let Some(pt) = self.find_element(&root, "PaymentTerms") {
            invoice.payment_terms = self.find_cbc_text(&pt, "Note");
        }

        // Remises/charges au niveau document (BG-20/BG-21)
        self.parse_allowance_charges(&root, &mut invoice);

        // Totaux et TVA
        self.parse_totals(&root, &mut invoice);

        // Lignes de facture (BG-25)
        self.parse_lines(&root, &mut invoice);

        // Pièces jointes (BG-24)
        self.parse_attachments(&root, &mut invoice);

        tracing::info!(
            invoice_number = %invoice.invoice_number,
            seller = invoice.seller_name.as_deref().unwrap_or("N/A"),
            buyer = invoice.buyer_name.as_deref().unwrap_or("N/A"),
            total_ttc = invoice.total_ttc.unwrap_or(0.0),
            "Facture UBL parsée"
        );

        Ok(invoice)
    }

    /// Cherche un élément par nom local dans les descendants directs ou proches
    fn find_element<'a>(&self, node: &'a roxmltree::Node, name: &str) -> Option<roxmltree::Node<'a, 'a>> {
        node.descendants().find(|n| n.tag_name().name() == name)
    }

    /// Cherche un élément cbc:* direct ou descendant et retourne son texte
    fn find_cbc_text(&self, node: &roxmltree::Node, local_name: &str) -> Option<String> {
        node.descendants()
            .find(|n| n.tag_name().name() == local_name && !n.has_children_elements())
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
    }

    /// Extrait l'adresse postale d'un noeud cac:PostalAddress
    fn extract_address(&self, parent: &roxmltree::Node) -> Option<PostalAddress> {
        let addr = self.find_element(parent, "PostalAddress")?;
        Some(PostalAddress {
            line1: self.find_cbc_text(&addr, "StreetName"),
            line2: self.find_cbc_text(&addr, "AdditionalStreetName"),
            line3: self.find_cbc_text(&addr, "BuildingNumber"),
            city: self.find_cbc_text(&addr, "CityName"),
            postal_code: self.find_cbc_text(&addr, "PostalZone"),
            country_subdivision: self.find_cbc_text(&addr, "CountrySubentity"),
            country_code: self.find_element(&addr, "Country")
                .and_then(|c| self.find_cbc_text(&c, "IdentificationCode")),
        })
    }

    /// Extrait le numéro TVA d'une Party
    fn extract_vat_id(&self, party: &roxmltree::Node) -> Option<String> {
        if let Some(tax_scheme) = self.find_element(party, "PartyTaxScheme") {
            return self.find_cbc_text(&tax_scheme, "CompanyID");
        }
        None
    }

    /// Extrait les champs principaux d'un élément Party (nom, SIRET, TVA)
    fn parse_party_fields(&self, wrapper: &roxmltree::Node,
        name: &mut Option<String>, id: &mut Option<String>, vat_id: &mut Option<String>)
    {
        if let Some(party) = self.find_element(wrapper, "Party") {
            if let Some(pn) = self.find_element(&party, "PartyName") {
                *name = self.find_cbc_text(&pn, "Name");
            }
            *id = self.extract_legal_id(&party);
            if let Some(tax_scheme) = self.find_element(&party, "PartyTaxScheme") {
                *vat_id = self.find_cbc_text(&tax_scheme, "CompanyID");
            }
        }
    }

    /// Extrait le SIREN/SIRET d'une Party via CompanyID dans PartyLegalEntity
    fn extract_legal_id(&self, party: &roxmltree::Node) -> Option<String> {
        // PartyLegalEntity -> CompanyID
        if let Some(legal) = self.find_element(party, "PartyLegalEntity") {
            if let Some(id) = self.find_cbc_text(&legal, "CompanyID") {
                return Some(id.replace(' ', ""));
            }
        }
        // PartyIdentification -> ID
        if let Some(ident) = self.find_element(party, "PartyIdentification") {
            return self.find_cbc_text(&ident, "ID");
        }
        None
    }

    /// Extrait le code pays d'une Party
    fn extract_country(&self, party: &roxmltree::Node) -> Option<String> {
        if let Some(addr) = self.find_element(party, "PostalAddress") {
            if let Some(country) = self.find_element(&addr, "Country") {
                return self.find_cbc_text(&country, "IdentificationCode");
            }
        }
        None
    }

    /// Parse le vendeur (BG-4)
    fn parse_seller(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(supplier_node) = self.find_element(root, "AccountingSupplierParty") {
            if let Some(party) = self.find_element(&supplier_node, "Party") {
                invoice.seller_name = self.find_element(&party, "PartyLegalEntity")
                    .and_then(|le| self.find_cbc_text(&le, "RegistrationName"))
                    .or_else(|| self.find_element(&party, "PartyName")
                        .and_then(|pn| self.find_cbc_text(&pn, "Name")));
                // BT-28 : Trading name (PartyName/Name)
                invoice.seller_trading_name = self.find_element(&party, "PartyName")
                    .and_then(|pn| self.find_cbc_text(&pn, "Name"));
                // BT-29 : Seller identifier (PartyIdentification)
                if let Some(pi) = self.find_element(&party, "PartyIdentification") {
                    invoice.seller_id = self.find_cbc_text(&pi, "ID");
                    invoice.seller_id_scheme = pi.descendants()
                        .find(|n| n.tag_name().name() == "ID")
                        .and_then(|n| n.attribute("schemeID"))
                        .map(|s| s.to_string());
                }
                invoice.seller_siret = self.extract_legal_id(&party);
                invoice.seller_vat_id = self.extract_vat_id(&party);
                invoice.seller_country = self.extract_country(&party);
                invoice.seller_address = self.extract_address(&party);
                // BT-34 : EndpointID
                let seller_ep = party.children().find(|n| n.tag_name().name() == "EndpointID");
                invoice.seller_endpoint_id = seller_ep.and_then(|n| n.text()).map(|t| t.trim().to_string());
                invoice.seller_endpoint_scheme = seller_ep.and_then(|n| n.attribute("schemeID")).map(|s| s.to_string());
            }
        }
    }

    /// Parse l'acheteur (BG-7)
    fn parse_buyer(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(customer_node) = self.find_element(root, "AccountingCustomerParty") {
            if let Some(party) = self.find_element(&customer_node, "Party") {
                invoice.buyer_name = self.find_element(&party, "PartyLegalEntity")
                    .and_then(|le| self.find_cbc_text(&le, "RegistrationName"))
                    .or_else(|| self.find_element(&party, "PartyName")
                        .and_then(|pn| self.find_cbc_text(&pn, "Name")));
                // BT-45 : Trading name
                invoice.buyer_trading_name = self.find_element(&party, "PartyName")
                    .and_then(|pn| self.find_cbc_text(&pn, "Name"));
                // BT-46 : Buyer identifier
                if let Some(pi) = self.find_element(&party, "PartyIdentification") {
                    invoice.buyer_id = self.find_cbc_text(&pi, "ID");
                    invoice.buyer_id_scheme = pi.descendants()
                        .find(|n| n.tag_name().name() == "ID")
                        .and_then(|n| n.attribute("schemeID"))
                        .map(|s| s.to_string());
                }
                invoice.buyer_siret = self.extract_legal_id(&party);
                invoice.buyer_vat_id = self.extract_vat_id(&party);
                invoice.buyer_country = self.extract_country(&party);
                invoice.buyer_address = self.extract_address(&party);
                // BT-49 : EndpointID
                let buyer_ep = party.children().find(|n| n.tag_name().name() == "EndpointID");
                invoice.buyer_endpoint_id = buyer_ep.and_then(|n| n.text()).map(|t| t.trim().to_string());
                invoice.buyer_endpoint_scheme = buyer_ep.and_then(|n| n.attribute("schemeID")).map(|s| s.to_string());
            }
        }
    }

    /// Parse le représentant fiscal (BG-11)
    fn parse_tax_representative(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(tax_rep) = self.find_element(root, "TaxRepresentativeParty") {
            invoice.tax_representative_name = self.find_element(&tax_rep, "PartyName")
                .and_then(|pn| self.find_cbc_text(&pn, "Name"));
            invoice.tax_representative_vat_id = self.extract_vat_id(&tax_rep);
            invoice.tax_representative_address = self.extract_address(&tax_rep);
        }
    }

    /// Parse la livraison (BG-13)
    fn parse_delivery(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(delivery) = self.find_element(root, "Delivery") {
            invoice.delivery_date = self.find_cbc_text(&delivery, "ActualDeliveryDate");
            if let Some(location) = self.find_element(&delivery, "DeliveryLocation") {
                if let Some(addr) = self.find_element(&location, "Address") {
                    invoice.delivery_address = Some(PostalAddress {
                        line1: self.find_cbc_text(&addr, "StreetName"),
                        line2: self.find_cbc_text(&addr, "AdditionalStreetName"),
                        line3: self.find_cbc_text(&addr, "BuildingNumber"),
                        city: self.find_cbc_text(&addr, "CityName"),
                        postal_code: self.find_cbc_text(&addr, "PostalZone"),
                        country_subdivision: self.find_cbc_text(&addr, "CountrySubentity"),
                        country_code: self.find_element(&addr, "Country")
                            .and_then(|c| self.find_cbc_text(&c, "IdentificationCode")),
                    });
                }
            }
            // BT-70 : Nom du destinataire de la livraison
            if let Some(delivery_party) = self.find_element(&delivery, "DeliveryParty") {
                if let Some(party_name) = self.find_element(&delivery_party, "PartyName") {
                    invoice.delivery_party_name = self.find_cbc_text(&party_name, "Name");
                }
            }
        }
    }

    /// Parse les moyens de paiement (BG-16)
    fn parse_payment_means(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        if let Some(pm) = self.find_element(root, "PaymentMeans") {
            invoice.payment_means_code = self.find_cbc_text(&pm, "PaymentMeansCode");
            if let Some(account) = self.find_element(&pm, "PayeeFinancialAccount") {
                invoice.payment_iban = self.find_cbc_text(&account, "ID");
                if let Some(branch) = self.find_element(&account, "FinancialInstitutionBranch") {
                    invoice.payment_bic = self.find_cbc_text(&branch, "ID");
                }
            }
        }
    }

    /// Parse les remises/charges au niveau document (BG-20/BG-21)
    fn parse_allowance_charges(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        for ac_node in root.children().filter(|n| n.tag_name().name() == "AllowanceCharge") {
            let charge = self.find_cbc_text(&ac_node, "ChargeIndicator")
                .map(|v| v == "true")
                .unwrap_or(false);
            invoice.allowance_charges.push(DocumentAllowanceCharge {
                charge_indicator: charge,
                amount: self.find_cbc_text(&ac_node, "Amount").and_then(|v| v.parse().ok()),
                tax_category_code: self.find_element(&ac_node, "TaxCategory")
                    .and_then(|tc| self.find_cbc_text(&tc, "ID")),
                tax_percent: self.find_element(&ac_node, "TaxCategory")
                    .and_then(|tc| self.find_cbc_text(&tc, "Percent"))
                    .and_then(|v| v.parse().ok()),
                reason: self.find_cbc_text(&ac_node, "AllowanceChargeReason"),
            });
        }
    }

    /// Parse les totaux et la ventilation TVA
    fn parse_totals(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        // Totaux (BG-22)
        if let Some(totals) = self.find_element(root, "LegalMonetaryTotal") {
            invoice.total_ht = self
                .find_cbc_text(&totals, "TaxExclusiveAmount")
                .and_then(|v| v.parse::<f64>().ok());
            invoice.total_ttc = self
                .find_cbc_text(&totals, "TaxInclusiveAmount")
                .and_then(|v| v.parse::<f64>().ok());
            invoice.prepaid_amount = self
                .find_cbc_text(&totals, "PrepaidAmount")
                .and_then(|v| v.parse::<f64>().ok());
            invoice.payable_amount = self
                .find_cbc_text(&totals, "PayableAmount")
                .and_then(|v| v.parse::<f64>().ok());
            // BT-107 / BT-108
            invoice.allowance_total_amount = self
                .find_cbc_text(&totals, "AllowanceTotalAmount")
                .and_then(|v| v.parse::<f64>().ok());
            invoice.charge_total_amount = self
                .find_cbc_text(&totals, "ChargeTotalAmount")
                .and_then(|v| v.parse::<f64>().ok());
        }

        // TVA totale + ventilation (BG-23)
        if let Some(tax_total) = self.find_element(root, "TaxTotal") {
            invoice.total_tax = tax_total
                .children()
                .find(|n| n.tag_name().name() == "TaxAmount")
                .and_then(|n| n.text())
                .and_then(|v| v.trim().parse::<f64>().ok());

            // Ventilation par catégorie
            for subtotal in tax_total.children().filter(|n| n.tag_name().name() == "TaxSubtotal") {
                let mut tb = TaxBreakdown {
                    taxable_amount: self.find_cbc_text(&subtotal, "TaxableAmount").and_then(|v| v.parse().ok()),
                    tax_amount: self.find_cbc_text(&subtotal, "TaxAmount").and_then(|v| v.parse().ok()),
                    category_code: None,
                    percent: None,
                    exemption_reason: None,
                    exemption_reason_code: None,
                };
                if let Some(cat) = self.find_element(&subtotal, "TaxCategory") {
                    tb.category_code = self.find_cbc_text(&cat, "ID");
                    tb.percent = self.find_cbc_text(&cat, "Percent").and_then(|v| v.parse().ok());
                    tb.exemption_reason = self.find_cbc_text(&cat, "TaxExemptionReason");
                    tb.exemption_reason_code = self.find_cbc_text(&cat, "TaxExemptionReasonCode");
                }
                invoice.tax_breakdowns.push(tb);
            }
        }
    }

    /// Parse les lignes de facture (BG-25)
    fn parse_lines(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        let line_tag = if root.tag_name().name() == "CreditNote" {
            "CreditNoteLine"
        } else {
            "InvoiceLine"
        };

        for line_node in root.children().filter(|n| n.tag_name().name() == line_tag) {
            let mut line = InvoiceLine {
                line_id: self.find_cbc_text(&line_node, "ID"),
                note: self.find_cbc_text(&line_node, "Note"),
                object_id: None,
                quantity: self.find_cbc_text(&line_node, "InvoicedQuantity")
                    .or_else(|| self.find_cbc_text(&line_node, "CreditedQuantity"))
                    .and_then(|v| v.parse().ok()),
                unit_code: line_node.descendants()
                    .find(|n| n.tag_name().name() == "InvoicedQuantity" || n.tag_name().name() == "CreditedQuantity")
                    .and_then(|n| n.attribute("unitCode"))
                    .map(|s| s.to_string()),
                line_net_amount: self.find_cbc_text(&line_node, "LineExtensionAmount").and_then(|v| v.parse().ok()),
                order_line_reference: None,
                accounting_cost: self.find_cbc_text(&line_node, "AccountingCost"),
                price: None,
                gross_price: None,
                item_name: None,
                item_description: None,
                seller_item_id: None,
                buyer_item_id: None,
                standard_item_id: None,
                standard_item_id_scheme: None,
                tax_category_code: None,
                tax_percent: None,
                period_start: None,
                period_end: None,
            };

            // Prix (BG-29)
            if let Some(price_node) = self.find_element(&line_node, "Price") {
                line.price = self.find_cbc_text(&price_node, "PriceAmount").and_then(|v| v.parse().ok());
            }

            // BT-132 : OrderLineReference
            if let Some(olr) = self.find_element(&line_node, "OrderLineReference") {
                line.order_line_reference = self.find_cbc_text(&olr, "LineID");
            }

            // BT-128 : DocumentReference (object identifier)
            if let Some(doc_ref) = self.find_element(&line_node, "DocumentReference") {
                line.object_id = self.find_cbc_text(&doc_ref, "ID");
            }

            // Article (BG-31)
            if let Some(item) = self.find_element(&line_node, "Item") {
                line.item_name = self.find_cbc_text(&item, "Name");
                line.item_description = self.find_cbc_text(&item, "Description");
                // BT-157 : BuyersItemIdentification
                if let Some(bi) = self.find_element(&item, "BuyersItemIdentification") {
                    line.buyer_item_id = self.find_cbc_text(&bi, "ID");
                }
                // BT-155 : SellersItemIdentification
                if let Some(si) = self.find_element(&item, "SellersItemIdentification") {
                    line.seller_item_id = self.find_cbc_text(&si, "ID");
                }
                // BT-158 : StandardItemIdentification
                if let Some(std_id) = self.find_element(&item, "StandardItemIdentification") {
                    line.standard_item_id = self.find_cbc_text(&std_id, "ID");
                    line.standard_item_id_scheme = std_id.descendants()
                        .find(|n| n.tag_name().name() == "ID")
                        .and_then(|n| n.attribute("schemeID"))
                        .map(|s| s.to_string());
                }
                if let Some(tax_cat) = self.find_element(&item, "ClassifiedTaxCategory") {
                    line.tax_category_code = self.find_cbc_text(&tax_cat, "ID");
                    line.tax_percent = self.find_cbc_text(&tax_cat, "Percent").and_then(|v| v.parse().ok());
                }
            }

            // Période de ligne (BG-26)
            if let Some(period) = self.find_element(&line_node, "InvoicePeriod") {
                line.period_start = self.find_cbc_text(&period, "StartDate");
                line.period_end = self.find_cbc_text(&period, "EndDate");
            }

            invoice.lines.push(line);
        }
    }

    /// Parse les pièces jointes (BG-24 : AdditionalDocumentReference)
    fn parse_attachments(&self, root: &roxmltree::Node, invoice: &mut InvoiceData) {
        for doc_ref in root.children().filter(|n| n.tag_name().name() == "AdditionalDocumentReference") {
            let mut att = InvoiceAttachment {
                id: self.find_cbc_text(&doc_ref, "ID"),
                description: self.find_cbc_text(&doc_ref, "DocumentDescription"),
                external_uri: None,
                embedded_content: None,
                mime_code: None,
                filename: None,
            };

            if let Some(attachment_node) = self.find_element(&doc_ref, "Attachment") {
                // Contenu embarqué (BT-125)
                if let Some(bin_obj) = self.find_element(&attachment_node, "EmbeddedDocumentBinaryObject") {
                    att.mime_code = bin_obj.attribute("mimeCode").map(|s| s.to_string());
                    att.filename = bin_obj.attribute("filename").map(|s| s.to_string());
                    if let Some(b64_text) = bin_obj.text() {
                        use base64::Engine;
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64_text.trim()) {
                            att.embedded_content = Some(bytes);
                        }
                    }
                }
                // URI externe (BT-124)
                if let Some(ext_ref) = self.find_element(&attachment_node, "ExternalReference") {
                    att.external_uri = self.find_cbc_text(&ext_ref, "URI");
                }
            }

            invoice.attachments.push(att);
        }
    }
}

trait HasChildrenElements {
    fn has_children_elements(&self) -> bool;
}

impl HasChildrenElements for roxmltree::Node<'_, '_> {
    fn has_children_elements(&self) -> bool {
        self.children().any(|c| c.is_element())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_ubl_fixture() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_001.xml")
            .expect("Fixture UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing UBL");

        assert_eq!(invoice.invoice_number, "FA-2025-00142");
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-11-15"));
        assert_eq!(invoice.due_date.as_deref(), Some("2025-12-15"));
        assert_eq!(invoice.currency.as_deref(), Some("EUR"));
        assert_eq!(invoice.seller_name.as_deref(), Some("TechConseil SAS"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("IndustrieFrance SA"));
        assert_eq!(invoice.total_ht, Some(12000.00));
        assert_eq!(invoice.total_ttc, Some(14400.00));
        assert_eq!(invoice.total_tax, Some(2400.00));
        assert_eq!(invoice.source_format, InvoiceFormat::UBL);
    }

    #[test]
    fn test_parse_ubl_avoir() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_002_avoir.xml")
            .expect("Fixture avoir UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing avoir UBL");

        assert_eq!(invoice.invoice_number, "AV-2025-00031");
        assert_eq!(invoice.total_ttc, Some(10200.00));
    }

    #[test]
    fn test_parse_ubl_autofacture_389() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/autofacture_ubl_389.xml")
            .expect("Fixture auto-facture UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing auto-facture UBL");

        assert_eq!(invoice.invoice_number, "AF-2025-00015");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("389"));
        assert_eq!(invoice.business_process.as_deref(), Some("A9"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-12-05"));
        assert_eq!(invoice.due_date.as_deref(), Some("2026-01-05"));
        assert_eq!(invoice.seller_name.as_deref(), Some("Ferme Bio du Vercors EARL"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Laiterie des Alpes SAS"));
        assert_eq!(invoice.contract_reference.as_deref(), Some("CONTRAT-AF-2025-UBL"));
        assert_eq!(invoice.total_ht, Some(15600.00));
        assert_eq!(invoice.total_ttc, Some(16458.00));
        assert_eq!(invoice.lines.len(), 2);
        // Line 1 item IDs
        let l1 = &invoice.lines[0];
        assert_eq!(l1.seller_item_id.as_deref(), Some("LAIT-BIO-5L"));
        assert_eq!(l1.standard_item_id.as_deref(), Some("3456789012345"));
        assert_eq!(l1.standard_item_id_scheme.as_deref(), Some("0160"));
        assert_eq!(l1.item_description.as_deref(), Some("Lait biologique entier, bidon de 5 litres, origine France"));
    }

    #[test]
    fn test_parse_ubl_tax_representative() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_tax_representative.xml")
            .expect("Fixture tax representative UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur de parsing tax rep UBL");

        assert_eq!(invoice.invoice_number, "FA-2025-DE-0088");
        assert_eq!(invoice.business_process.as_deref(), Some("A2"));
        // Vendeur allemand
        assert_eq!(invoice.seller_name.as_deref(), Some("Maschinenbau GmbH"));
        assert_eq!(invoice.seller_country.as_deref(), Some("DE"));
        // Acheteur français
        assert_eq!(invoice.buyer_name.as_deref(), Some("IndustrieFrance SA"));
        assert_eq!(invoice.buyer_country.as_deref(), Some("FR"));
        // Représentant fiscal (BG-11)
        assert_eq!(invoice.tax_representative_name.as_deref(), Some("Cabinet Fiscal International SARL"));
        assert_eq!(invoice.tax_representative_vat_id.as_deref(), Some("FR55667788990"));
        assert!(invoice.tax_representative_address.is_some());
        let addr = invoice.tax_representative_address.as_ref().unwrap();
        assert_eq!(addr.city.as_deref(), Some("Paris"));
        assert_eq!(addr.postal_code.as_deref(), Some("75008"));
        // Pièce jointe (BG-24)
        assert_eq!(invoice.attachments.len(), 1);
        assert_eq!(invoice.attachments[0].id.as_deref(), Some("ATT-001"));
        assert_eq!(invoice.attachments[0].external_uri.as_deref(), Some("https://docs.example.com/bl/BL-2025-DE-0088.pdf"));
        // Livraison
        assert_eq!(invoice.delivery_date.as_deref(), Some("2025-11-28"));
        assert!(invoice.delivery_address.is_some());
    }

    // ===== Tests sur exemples officiels AFNOR XP Z12-014 v1.2 =====

    #[test]
    fn test_parse_official_uc1_ubl_standard_invoice() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC1/UC1_F202500003_00-INV_20250701_UBL.xml")
            .expect("Fixture officielle UC1 UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing UC1 UBL");

        assert_eq!(invoice.invoice_number, "F202500003");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        assert_eq!(invoice.business_process.as_deref(), Some("S1"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-01"));
        assert_eq!(invoice.due_date.as_deref(), Some("2025-07-31"));
        assert_eq!(invoice.currency.as_deref(), Some("EUR"));
        // Vendeur
        assert_eq!(invoice.seller_name.as_deref(), Some("LE VENDEUR"));
        assert_eq!(invoice.seller_siret.as_deref(), Some("100000009"));
        assert_eq!(invoice.seller_vat_id.as_deref(), Some("FR88100000009"));
        assert_eq!(invoice.seller_endpoint_id.as_deref(), Some("100000009_STATUTS"));
        // Acheteur
        assert_eq!(invoice.buyer_name.as_deref(), Some("LE CLIENT"));
        assert_eq!(invoice.buyer_siret.as_deref(), Some("200000008"));
        assert_eq!(invoice.buyer_vat_id.as_deref(), Some("FR37200000008"));
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
        // TVA
        assert_eq!(invoice.tax_breakdowns.len(), 1);
        assert_eq!(invoice.tax_breakdowns[0].percent, Some(20.00));
        // Lignes
        assert_eq!(invoice.lines.len(), 2);
        assert_eq!(invoice.lines[0].item_name.as_deref(), Some("SERVICE_FOURNI1"));
        assert_eq!(invoice.lines[0].quantity, Some(200.0));
        assert_eq!(invoice.lines[0].line_net_amount, Some(8000.00));
        // Notes
        assert!(invoice.notes.len() >= 5);
    }

    #[test]
    fn test_parse_official_uc4b_ubl_corrective_384() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC4/UC4b_F202500010_00-INVCORR/UC4b_F202500010_00-INVCORR_20250702_UBL.xml")
            .expect("Fixture officielle UC4b UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing UC4b UBL");

        assert_eq!(invoice.invoice_number, "F202500010");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("384"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-02"));
        // Référence facture antérieure (BT-25/BT-26)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("F20250006"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(10000.00));
        assert_eq!(invoice.total_ttc, Some(12000.00));
    }

    #[test]
    fn test_parse_official_uc5b_ubl_credit_note_381() {
        let xml = fs::read_to_string("../../specs/examples/xp-z12-014/UC5/UC5b_F202500011_00-CN_20250703/UC5b_F202500011_00-CN_20250703_UBL.xml")
            .expect("Fixture officielle UC5b UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing UC5b UBL");

        assert_eq!(invoice.invoice_number, "F202500011");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("381"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-03"));
        // Référence facture antérieure
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("F20250007"));
        // Livraison
        assert_eq!(invoice.delivery_party_name.as_deref(), Some("NOUS AUSSI"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(10000.00));
        assert_eq!(invoice.total_ttc, Some(11000.00));
    }

    // ===== Tests sur fixtures métier =====

    #[test]
    fn test_parse_ubl_soustraitance_a4() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_soustraitance_a4.xml")
            .expect("Fixture sous-traitance UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing sous-traitance UBL");

        assert_eq!(invoice.invoice_number, "ST-2025-UBL-00042");
        assert_eq!(invoice.business_process.as_deref(), Some("A4"));
        assert_eq!(invoice.seller_name.as_deref(), Some("Plomberie Durand SARL"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Construction Générale Martin SA"));
        // PayeeParty (BG-10) absent car vendeur = bénéficiaire (EN16931 BR-17)
        assert_eq!(invoice.payee_name, None);
        // Autoliquidation
        assert_eq!(invoice.tax_breakdowns[0].category_code.as_deref(), Some("AE"));
        assert_eq!(invoice.total_ht, Some(63500.00));
        assert_eq!(invoice.payable_amount, Some(63500.00));
    }

    #[test]
    fn test_parse_ubl_marketplace_a8() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_marketplace_a8.xml")
            .expect("Fixture marketplace UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing marketplace UBL");

        assert_eq!(invoice.invoice_number, "MKP-2025-UBL-00789");
        assert_eq!(invoice.business_process.as_deref(), Some("A8"));
        assert_eq!(invoice.seller_name.as_deref(), Some("AudioTech France SAS"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Électronique Distribution SARL"));
        // Déjà payée
        assert_eq!(invoice.prepaid_amount, Some(8334.90));
        assert_eq!(invoice.payable_amount, Some(0.00));
        assert_eq!(invoice.total_ttc, Some(8334.90));
        assert_eq!(invoice.lines.len(), 2);
    }

    #[test]
    fn test_parse_ubl_rectificative_384() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_rectificative_ubl_384.xml")
            .expect("Fixture rectificative UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing rectificative UBL");

        assert_eq!(invoice.invoice_number, "RECT-2025-00002");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("384"));
        assert_eq!(invoice.business_process.as_deref(), Some("S1"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-07-20"));
        assert_eq!(invoice.due_date.as_deref(), Some("2025-08-20"));
        // Référence facture antérieure (BT-25/BT-26)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("FA-2025-00200"));
        assert_eq!(invoice.preceding_invoice_date.as_deref(), Some("2025-07-05"));
        // Vendeur
        assert_eq!(invoice.seller_name.as_deref(), Some("Services Numériques SARL"));
        assert_eq!(invoice.seller_siret.as_deref(), Some("11122233344455"));
        assert_eq!(invoice.seller_vat_id.as_deref(), Some("FR11111222333"));
        // Acheteur
        assert_eq!(invoice.buyer_name.as_deref(), Some("Distribution Express SA"));
        assert_eq!(invoice.buyer_siret.as_deref(), Some("44455566677788"));
        // Paiement
        assert_eq!(invoice.payment_means_code.as_deref(), Some("30"));
        assert_eq!(invoice.payment_iban.as_deref(), Some("FR7610011000201234567890188"));
        assert_eq!(invoice.payment_bic.as_deref(), Some("PSSTFRPPXXX"));
        // Totaux
        assert_eq!(invoice.total_ht, Some(8000.00));
        assert_eq!(invoice.total_ttc, Some(9600.00));
        assert_eq!(invoice.payable_amount, Some(9600.00));
        assert_eq!(invoice.tax_breakdowns.len(), 1);
        assert_eq!(invoice.tax_breakdowns[0].percent, Some(20.00));
        // Lignes
        assert_eq!(invoice.lines.len(), 1);
        assert_eq!(invoice.lines[0].item_name.as_deref(), Some("Composant électronique REF-A42"));
        assert_eq!(invoice.lines[0].quantity, Some(100.0));
        assert_eq!(invoice.lines[0].price, Some(80.00));
        assert_eq!(invoice.lines[0].line_net_amount, Some(8000.00));
    }

    #[test]
    fn test_parse_ubl_acompte_386() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_acompte_386.xml")
            .expect("Fixture acompte UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing acompte UBL");

        assert_eq!(invoice.invoice_number, "FA-2025-UBL-ACOMPTE-001");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("386"));
        assert_eq!(invoice.business_process.as_deref(), Some("A1"));
        assert_eq!(invoice.issue_date.as_deref(), Some("2025-10-01"));
        assert_eq!(invoice.due_date.as_deref(), Some("2025-10-15"));
        assert_eq!(invoice.seller_name.as_deref(), Some("Rénovation Habitat Plus SARL"));
        assert_eq!(invoice.buyer_name.as_deref(), Some("Cabinet Médical Dr Lefèvre"));
        assert_eq!(invoice.contract_reference.as_deref(), Some("DEV-2025-100"));
        assert_eq!(invoice.total_ht, Some(4500.00));
        assert_eq!(invoice.total_ttc, Some(4950.00));
        assert_eq!(invoice.payable_amount, Some(4950.00));
        assert_eq!(invoice.tax_breakdowns.len(), 1);
        assert_eq!(invoice.tax_breakdowns[0].percent, Some(10.0));
        assert_eq!(invoice.lines.len(), 1);
    }

    #[test]
    fn test_parse_ubl_definitive_apres_acompte() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_definitive_apres_acompte.xml")
            .expect("Fixture définitive UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing définitive UBL");

        assert_eq!(invoice.invoice_number, "FA-2025-UBL-DEF-002");
        assert_eq!(invoice.invoice_type_code.as_deref(), Some("380"));
        assert_eq!(invoice.business_process.as_deref(), Some("A1"));
        // Référence à l'acompte (BT-25/BT-26)
        assert_eq!(invoice.preceding_invoice_reference.as_deref(), Some("FA-2025-UBL-ACOMPTE-001"));
        assert_eq!(invoice.preceding_invoice_date.as_deref(), Some("2025-10-01"));
        // Totaux avec déduction acompte
        assert_eq!(invoice.total_ht, Some(15000.00));
        assert_eq!(invoice.total_ttc, Some(16500.00));
        assert_eq!(invoice.prepaid_amount, Some(4950.00));
        assert_eq!(invoice.payable_amount, Some(11550.00));
        assert_eq!(invoice.lines.len(), 4);
        assert_eq!(invoice.delivery_date.as_deref(), Some("2025-11-25"));
    }

    #[test]
    fn test_parse_ubl_remises_multitva() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_remises_multitva.xml")
            .expect("Fixture multi-TVA UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing multi-TVA UBL");

        assert_eq!(invoice.invoice_number, "FA-2025-UBL-00300");
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
    }

    #[test]
    fn test_parse_ubl_delegation_s8() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_delegation_s8.xml")
            .expect("Fixture délégation UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing délégation UBL");

        assert_eq!(invoice.invoice_number, "DELEG-2025-UBL-00001");
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
    fn test_parse_ubl_multivendeurs_b8() {
        let xml = fs::read_to_string("../../tests/fixtures/ubl/facture_ubl_multivendeurs_b8.xml")
            .expect("Fixture multi-vendeurs UBL introuvable");

        let parser = UblParser::new();
        let invoice = parser.parse(&xml).expect("Erreur parsing multi-vendeurs UBL");

        assert_eq!(invoice.invoice_number, "MV-2025-UBL-00100");
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
        // 3 lignes
        assert_eq!(invoice.lines.len(), 3);
        assert_eq!(invoice.total_ht, Some(2255.00));
        assert_eq!(invoice.total_ttc, Some(2501.00));
    }

    #[test]
    fn test_parse_ubl_invalid_xml() {
        let parser = UblParser::new();
        let result = parser.parse("ceci n'est pas du XML");
        assert!(result.is_err());
    }
}
