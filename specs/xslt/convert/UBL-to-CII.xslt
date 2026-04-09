<?xml version="1.0" encoding="UTF-8"?>
<!--
  UBL Invoice 2.1 → UN/CEFACT CII D22B (CrossIndustryInvoice)
  Mapping conforme EN16931 / Factur-X EXTENDED
  Gère tous les BT/BG de l'annexe A EN16931 + attachements BG-24
-->
<xsl:stylesheet version="2.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:ubl="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
    xmlns:cn="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2"
    xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
    xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2"
    xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
    xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
    xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100"
    xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"
    exclude-result-prefixes="ubl cn cac cbc">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:strip-space elements="*"/>

  <!-- ============================================================ -->
  <!-- Root template : Invoice ou CreditNote                        -->
  <!-- ============================================================ -->
  <xsl:template match="/ubl:Invoice | /cn:CreditNote">
    <rsm:CrossIndustryInvoice>
      <xsl:call-template name="ExchangedDocumentContext"/>
      <xsl:call-template name="ExchangedDocument"/>
      <rsm:SupplyChainTradeTransaction>
        <xsl:apply-templates select="cac:InvoiceLine | cac:CreditNoteLine"/>
        <xsl:call-template name="HeaderTradeAgreement"/>
        <xsl:call-template name="HeaderTradeDelivery"/>
        <xsl:call-template name="HeaderTradeSettlement"/>
      </rsm:SupplyChainTradeTransaction>
    </rsm:CrossIndustryInvoice>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- BG-2 : ExchangedDocumentContext                              -->
  <!-- ============================================================ -->
  <xsl:template name="ExchangedDocumentContext">
    <rsm:ExchangedDocumentContext>
      <!-- BT-23 : Business process -->
      <xsl:if test="cbc:ProfileID">
        <ram:BusinessProcessSpecifiedDocumentContextParameter>
          <ram:ID><xsl:value-of select="cbc:ProfileID"/></ram:ID>
        </ram:BusinessProcessSpecifiedDocumentContextParameter>
      </xsl:if>
      <!-- BT-24 : Specification identifier -->
      <ram:GuidelineSpecifiedDocumentContextParameter>
        <ram:ID>
          <xsl:choose>
            <xsl:when test="cbc:CustomizationID">
              <xsl:value-of select="cbc:CustomizationID"/>
            </xsl:when>
            <xsl:otherwise>urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:extended</xsl:otherwise>
          </xsl:choose>
        </ram:ID>
      </ram:GuidelineSpecifiedDocumentContextParameter>
    </rsm:ExchangedDocumentContext>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- BG-1 : ExchangedDocument                                     -->
  <!-- ============================================================ -->
  <xsl:template name="ExchangedDocument">
    <rsm:ExchangedDocument>
      <!-- BT-1 : Invoice number -->
      <ram:ID><xsl:value-of select="cbc:ID"/></ram:ID>
      <!-- BT-3 : Invoice type code -->
      <ram:TypeCode>
        <xsl:choose>
          <xsl:when test="cbc:InvoiceTypeCode"><xsl:value-of select="cbc:InvoiceTypeCode"/></xsl:when>
          <xsl:when test="cbc:CreditNoteTypeCode"><xsl:value-of select="cbc:CreditNoteTypeCode"/></xsl:when>
          <xsl:otherwise>380</xsl:otherwise>
        </xsl:choose>
      </ram:TypeCode>
      <!-- BT-2 : Issue date -->
      <xsl:if test="cbc:IssueDate">
        <ram:IssueDateTime>
          <udt:DateTimeString format="102">
            <xsl:value-of select="translate(cbc:IssueDate, '-', '')"/>
          </udt:DateTimeString>
        </ram:IssueDateTime>
      </xsl:if>
      <!-- BT-22 : Notes -->
      <xsl:for-each select="cbc:Note">
        <ram:IncludedNote>
          <xsl:choose>
            <!-- UBL encodes subject code as #CODE prefix in Note text -->
            <xsl:when test="starts-with(., '#')">
              <ram:Content><xsl:value-of select="substring-after(substring-after(., '#'), ' ')"/></ram:Content>
              <ram:SubjectCode><xsl:value-of select="substring-before(substring-after(., '#'), ' ')"/></ram:SubjectCode>
            </xsl:when>
            <xsl:otherwise>
              <ram:Content><xsl:value-of select="."/></ram:Content>
            </xsl:otherwise>
          </xsl:choose>
        </ram:IncludedNote>
      </xsl:for-each>
    </rsm:ExchangedDocument>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- BG-25 : Invoice lines                                        -->
  <!-- ============================================================ -->
  <xsl:template match="cac:InvoiceLine | cac:CreditNoteLine">
    <ram:IncludedSupplyChainTradeLineItem>
      <!-- BT-126 : Line identifier -->
      <ram:AssociatedDocumentLineDocument>
        <ram:LineID><xsl:value-of select="cbc:ID"/></ram:LineID>
        <!-- BT-127 : Line note -->
        <xsl:if test="cbc:Note">
          <ram:IncludedNote>
            <ram:Content><xsl:value-of select="cbc:Note"/></ram:Content>
          </ram:IncludedNote>
        </xsl:if>
      </ram:AssociatedDocumentLineDocument>

      <!-- BG-31 : Item information -->
      <ram:SpecifiedTradeProduct>
        <!-- BT-158 : Item standard identifier (GTIN etc.) -->
        <xsl:if test="cac:Item/cac:StandardItemIdentification/cbc:ID">
          <ram:GlobalID>
            <xsl:attribute name="schemeID">
              <xsl:value-of select="cac:Item/cac:StandardItemIdentification/cbc:ID/@schemeID"/>
            </xsl:attribute>
            <xsl:value-of select="cac:Item/cac:StandardItemIdentification/cbc:ID"/>
          </ram:GlobalID>
        </xsl:if>
        <!-- BT-155 : Item Seller's identifier -->
        <xsl:if test="cac:Item/cac:SellersItemIdentification/cbc:ID">
          <ram:SellerAssignedID><xsl:value-of select="cac:Item/cac:SellersItemIdentification/cbc:ID"/></ram:SellerAssignedID>
        </xsl:if>
        <!-- BT-157 : Item Buyer's identifier -->
        <xsl:if test="cac:Item/cac:BuyersItemIdentification/cbc:ID">
          <ram:BuyerAssignedID><xsl:value-of select="cac:Item/cac:BuyersItemIdentification/cbc:ID"/></ram:BuyerAssignedID>
        </xsl:if>
        <!-- BT-153 : Item name -->
        <xsl:if test="cac:Item/cbc:Name">
          <ram:Name><xsl:value-of select="cac:Item/cbc:Name"/></ram:Name>
        </xsl:if>
        <!-- BT-154 : Item description -->
        <xsl:if test="cac:Item/cbc:Description">
          <ram:Description><xsl:value-of select="cac:Item/cbc:Description"/></ram:Description>
        </xsl:if>
      </ram:SpecifiedTradeProduct>

      <!-- BG-29 : Price details -->
      <ram:SpecifiedLineTradeAgreement>
        <!-- BT-132 : Referenced purchase order line reference -->
        <xsl:if test="cac:OrderLineReference/cbc:LineID">
          <ram:BuyerOrderReferencedDocument>
            <ram:LineID><xsl:value-of select="cac:OrderLineReference/cbc:LineID"/></ram:LineID>
          </ram:BuyerOrderReferencedDocument>
        </xsl:if>
        <!-- BT-148 : Item gross price -->
        <xsl:if test="cac:Price/cac:AllowanceCharge/cbc:BaseAmount">
          <ram:GrossPriceProductTradePrice>
            <ram:ChargeAmount><xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:BaseAmount"/></ram:ChargeAmount>
          </ram:GrossPriceProductTradePrice>
        </xsl:if>
        <!-- BT-146 : Item net price -->
        <xsl:if test="cac:Price/cbc:PriceAmount">
          <ram:NetPriceProductTradePrice>
            <ram:ChargeAmount><xsl:value-of select="cac:Price/cbc:PriceAmount"/></ram:ChargeAmount>
          </ram:NetPriceProductTradePrice>
        </xsl:if>
      </ram:SpecifiedLineTradeAgreement>

      <!-- BT-129 : Invoiced quantity -->
      <ram:SpecifiedLineTradeDelivery>
        <xsl:choose>
          <xsl:when test="cbc:InvoicedQuantity">
            <ram:BilledQuantity>
              <xsl:attribute name="unitCode"><xsl:value-of select="cbc:InvoicedQuantity/@unitCode"/></xsl:attribute>
              <xsl:value-of select="cbc:InvoicedQuantity"/>
            </ram:BilledQuantity>
          </xsl:when>
          <xsl:when test="cbc:CreditedQuantity">
            <ram:BilledQuantity>
              <xsl:attribute name="unitCode"><xsl:value-of select="cbc:CreditedQuantity/@unitCode"/></xsl:attribute>
              <xsl:value-of select="cbc:CreditedQuantity"/>
            </ram:BilledQuantity>
          </xsl:when>
        </xsl:choose>
      </ram:SpecifiedLineTradeDelivery>

      <!-- Line settlement -->
      <ram:SpecifiedLineTradeSettlement>
        <!-- BT-151 : Invoiced item VAT category code -->
        <ram:ApplicableTradeTax>
          <ram:TypeCode>VAT</ram:TypeCode>
          <ram:CategoryCode>
            <xsl:value-of select="cac:Item/cac:ClassifiedTaxCategory/cbc:ID"/>
          </ram:CategoryCode>
          <xsl:if test="cac:Item/cac:ClassifiedTaxCategory/cbc:Percent">
            <ram:RateApplicablePercent>
              <xsl:value-of select="cac:Item/cac:ClassifiedTaxCategory/cbc:Percent"/>
            </ram:RateApplicablePercent>
          </xsl:if>
        </ram:ApplicableTradeTax>
        <!-- BT-131 : Invoice line net amount -->
        <xsl:if test="cbc:LineExtensionAmount">
          <ram:SpecifiedTradeSettlementLineMonetarySummation>
            <ram:LineTotalAmount><xsl:value-of select="cbc:LineExtensionAmount"/></ram:LineTotalAmount>
          </ram:SpecifiedTradeSettlementLineMonetarySummation>
        </xsl:if>
        <!-- BG-26 : Invoice line period -->
        <xsl:if test="cac:InvoicePeriod">
          <ram:BillingSpecifiedPeriod>
            <xsl:if test="cac:InvoicePeriod/cbc:StartDate">
              <ram:StartDateTime>
                <udt:DateTimeString format="102">
                  <xsl:value-of select="translate(cac:InvoicePeriod/cbc:StartDate, '-', '')"/>
                </udt:DateTimeString>
              </ram:StartDateTime>
            </xsl:if>
            <xsl:if test="cac:InvoicePeriod/cbc:EndDate">
              <ram:EndDateTime>
                <udt:DateTimeString format="102">
                  <xsl:value-of select="translate(cac:InvoicePeriod/cbc:EndDate, '-', '')"/>
                </udt:DateTimeString>
              </ram:EndDateTime>
            </xsl:if>
          </ram:BillingSpecifiedPeriod>
        </xsl:if>
        <!-- BT-133 : Invoice line Buyer accounting reference -->
        <xsl:if test="cbc:AccountingCost">
          <ram:ReceivableSpecifiedTradeAccountingAccount>
            <ram:ID><xsl:value-of select="cbc:AccountingCost"/></ram:ID>
          </ram:ReceivableSpecifiedTradeAccountingAccount>
        </xsl:if>
      </ram:SpecifiedLineTradeSettlement>
    </ram:IncludedSupplyChainTradeLineItem>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- HeaderTradeAgreement (BG-4 Seller, BG-7 Buyer, refs)         -->
  <!-- ============================================================ -->
  <xsl:template name="HeaderTradeAgreement">
    <ram:ApplicableHeaderTradeAgreement>
      <!-- BT-10 : Buyer reference -->
      <xsl:if test="cbc:BuyerReference">
        <ram:BuyerReference><xsl:value-of select="cbc:BuyerReference"/></ram:BuyerReference>
      </xsl:if>

      <!-- BG-4 : Seller -->
      <ram:SellerTradeParty>
        <xsl:call-template name="TradeParty">
          <xsl:with-param name="party" select="cac:AccountingSupplierParty/cac:Party"/>
        </xsl:call-template>
      </ram:SellerTradeParty>

      <!-- BG-7 : Buyer -->
      <ram:BuyerTradeParty>
        <xsl:call-template name="TradeParty">
          <xsl:with-param name="party" select="cac:AccountingCustomerParty/cac:Party"/>
        </xsl:call-template>
      </ram:BuyerTradeParty>

      <!-- BG-10 : Payee (if different from Seller) -->
      <xsl:if test="cac:PayeeParty">
        <ram:PayeeTradeParty>
          <xsl:if test="cac:PayeeParty/cac:PartyIdentification/cbc:ID">
            <ram:ID>
              <xsl:if test="cac:PayeeParty/cac:PartyIdentification/cbc:ID/@schemeID">
                <xsl:attribute name="schemeID"><xsl:value-of select="cac:PayeeParty/cac:PartyIdentification/cbc:ID/@schemeID"/></xsl:attribute>
              </xsl:if>
              <xsl:value-of select="cac:PayeeParty/cac:PartyIdentification/cbc:ID"/>
            </ram:ID>
          </xsl:if>
          <xsl:if test="cac:PayeeParty/cac:PartyName/cbc:Name">
            <ram:Name><xsl:value-of select="cac:PayeeParty/cac:PartyName/cbc:Name"/></ram:Name>
          </xsl:if>
          <xsl:if test="cac:PayeeParty/cac:PartyLegalEntity/cbc:CompanyID">
            <ram:SpecifiedLegalOrganization>
              <ram:ID>
                <xsl:if test="cac:PayeeParty/cac:PartyLegalEntity/cbc:CompanyID/@schemeID">
                  <xsl:attribute name="schemeID"><xsl:value-of select="cac:PayeeParty/cac:PartyLegalEntity/cbc:CompanyID/@schemeID"/></xsl:attribute>
                </xsl:if>
                <xsl:value-of select="cac:PayeeParty/cac:PartyLegalEntity/cbc:CompanyID"/>
              </ram:ID>
            </ram:SpecifiedLegalOrganization>
          </xsl:if>
        </ram:PayeeTradeParty>
      </xsl:if>

      <!-- BG-11 : Seller tax representative -->
      <xsl:if test="cac:TaxRepresentativeParty">
        <ram:SellerTaxRepresentativeTradeParty>
          <xsl:if test="cac:TaxRepresentativeParty/cac:PartyName/cbc:Name">
            <ram:Name><xsl:value-of select="cac:TaxRepresentativeParty/cac:PartyName/cbc:Name"/></ram:Name>
          </xsl:if>
          <xsl:if test="cac:TaxRepresentativeParty/cac:PostalAddress">
            <xsl:call-template name="PostalAddress">
              <xsl:with-param name="addr" select="cac:TaxRepresentativeParty/cac:PostalAddress"/>
            </xsl:call-template>
          </xsl:if>
          <xsl:if test="cac:TaxRepresentativeParty/cac:PartyTaxScheme/cbc:CompanyID">
            <ram:SpecifiedTaxRegistration>
              <ram:ID schemeID="VA"><xsl:value-of select="cac:TaxRepresentativeParty/cac:PartyTaxScheme/cbc:CompanyID"/></ram:ID>
            </ram:SpecifiedTaxRegistration>
          </xsl:if>
        </ram:SellerTaxRepresentativeTradeParty>
      </xsl:if>

      <!-- BT-13 : Purchase order reference -->
      <xsl:if test="cac:OrderReference/cbc:ID">
        <ram:BuyerOrderReferencedDocument>
          <ram:IssuerAssignedID><xsl:value-of select="cac:OrderReference/cbc:ID"/></ram:IssuerAssignedID>
        </ram:BuyerOrderReferencedDocument>
      </xsl:if>

      <!-- BT-12 : Contract reference -->
      <xsl:if test="cac:ContractDocumentReference/cbc:ID">
        <ram:ContractReferencedDocument>
          <ram:IssuerAssignedID><xsl:value-of select="cac:ContractDocumentReference/cbc:ID"/></ram:IssuerAssignedID>
        </ram:ContractReferencedDocument>
      </xsl:if>

      <!-- BT-11 : Project reference -->
      <xsl:if test="cac:ProjectReference/cbc:ID">
        <ram:AdditionalReferencedDocument>
          <ram:IssuerAssignedID><xsl:value-of select="cac:ProjectReference/cbc:ID"/></ram:IssuerAssignedID>
          <ram:TypeCode>50</ram:TypeCode>
        </ram:AdditionalReferencedDocument>
      </xsl:if>

      <!-- BG-24 : Additional supporting documents / attachments -->
      <xsl:for-each select="cac:AdditionalDocumentReference">
        <ram:AdditionalReferencedDocument>
          <ram:IssuerAssignedID><xsl:value-of select="cbc:ID"/></ram:IssuerAssignedID>
          <!-- BT-124 : External document location (URIID avant TypeCode per XSD xs:sequence) -->
          <xsl:if test="cac:Attachment/cac:ExternalReference/cbc:URI">
            <ram:URIID><xsl:value-of select="cac:Attachment/cac:ExternalReference/cbc:URI"/></ram:URIID>
          </xsl:if>
          <ram:TypeCode>916</ram:TypeCode>
          <xsl:if test="cbc:DocumentDescription">
            <ram:Name><xsl:value-of select="cbc:DocumentDescription"/></ram:Name>
          </xsl:if>
          <!-- BT-125 : Attached document (binary) -->
          <xsl:if test="cac:Attachment/cbc:EmbeddedDocumentBinaryObject">
            <ram:AttachmentBinaryObject>
              <xsl:attribute name="mimeCode">
                <xsl:value-of select="cac:Attachment/cbc:EmbeddedDocumentBinaryObject/@mimeCode"/>
              </xsl:attribute>
              <xsl:attribute name="filename">
                <xsl:value-of select="cac:Attachment/cbc:EmbeddedDocumentBinaryObject/@filename"/>
              </xsl:attribute>
              <xsl:value-of select="cac:Attachment/cbc:EmbeddedDocumentBinaryObject"/>
            </ram:AttachmentBinaryObject>
          </xsl:if>
        </ram:AdditionalReferencedDocument>
      </xsl:for-each>
    </ram:ApplicableHeaderTradeAgreement>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- HeaderTradeDelivery (BG-13, BG-14)                           -->
  <!-- ============================================================ -->
  <xsl:template name="HeaderTradeDelivery">
    <ram:ApplicableHeaderTradeDelivery>
      <!-- BG-15 : Deliver to address -->
      <xsl:if test="cac:Delivery/cac:DeliveryLocation/cac:Address">
        <ram:ShipToTradeParty>
          <xsl:call-template name="PostalAddress">
            <xsl:with-param name="addr" select="cac:Delivery/cac:DeliveryLocation/cac:Address"/>
          </xsl:call-template>
        </ram:ShipToTradeParty>
      </xsl:if>
      <!-- BT-72 : Actual delivery date (fallback to IssueDate if no Delivery) -->
      <ram:ActualDeliverySupplyChainEvent>
        <ram:OccurrenceDateTime>
          <udt:DateTimeString format="102">
            <xsl:choose>
              <xsl:when test="cac:Delivery/cbc:ActualDeliveryDate">
                <xsl:value-of select="translate(cac:Delivery/cbc:ActualDeliveryDate, '-', '')"/>
              </xsl:when>
              <xsl:otherwise>
                <xsl:value-of select="translate(cbc:IssueDate, '-', '')"/>
              </xsl:otherwise>
            </xsl:choose>
          </udt:DateTimeString>
        </ram:OccurrenceDateTime>
      </ram:ActualDeliverySupplyChainEvent>
    </ram:ApplicableHeaderTradeDelivery>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- HeaderTradeSettlement                                         -->
  <!-- ============================================================ -->
  <xsl:template name="HeaderTradeSettlement">
    <ram:ApplicableHeaderTradeSettlement>
      <!-- BT-6 : VAT accounting currency code -->
      <xsl:if test="cbc:TaxCurrencyCode">
        <ram:TaxCurrencyCode><xsl:value-of select="cbc:TaxCurrencyCode"/></ram:TaxCurrencyCode>
      </xsl:if>
      <!-- BT-5 : Invoice currency code -->
      <ram:InvoiceCurrencyCode><xsl:value-of select="cbc:DocumentCurrencyCode"/></ram:InvoiceCurrencyCode>

      <!-- BG-16 : Payment instructions -->
      <xsl:if test="cac:PaymentMeans">
        <ram:SpecifiedTradeSettlementPaymentMeans>
          <!-- BT-81 : Payment means type code -->
          <xsl:if test="cac:PaymentMeans/cbc:PaymentMeansCode">
            <ram:TypeCode><xsl:value-of select="cac:PaymentMeans/cbc:PaymentMeansCode"/></ram:TypeCode>
          </xsl:if>
          <!-- BT-82 : Payment means text -->
          <xsl:if test="cac:PaymentMeans/cbc:PaymentMeansCode/@name">
            <ram:Information><xsl:value-of select="cac:PaymentMeans/cbc:PaymentMeansCode/@name"/></ram:Information>
          </xsl:if>
          <!-- BT-84/85 : Payment account (IBAN) -->
          <xsl:if test="cac:PaymentMeans/cac:PayeeFinancialAccount">
            <ram:PayeePartyCreditorFinancialAccount>
              <xsl:if test="cac:PaymentMeans/cac:PayeeFinancialAccount/cbc:ID">
                <ram:IBANID><xsl:value-of select="cac:PaymentMeans/cac:PayeeFinancialAccount/cbc:ID"/></ram:IBANID>
              </xsl:if>
              <xsl:if test="cac:PaymentMeans/cac:PayeeFinancialAccount/cbc:Name">
                <ram:AccountName><xsl:value-of select="cac:PaymentMeans/cac:PayeeFinancialAccount/cbc:Name"/></ram:AccountName>
              </xsl:if>
            </ram:PayeePartyCreditorFinancialAccount>
            <!-- BT-86 : Payment service provider (BIC) -->
            <xsl:if test="cac:PaymentMeans/cac:PayeeFinancialAccount/cac:FinancialInstitutionBranch/cbc:ID">
              <ram:PayeeSpecifiedCreditorFinancialInstitution>
                <ram:BICID><xsl:value-of select="cac:PaymentMeans/cac:PayeeFinancialAccount/cac:FinancialInstitutionBranch/cbc:ID"/></ram:BICID>
              </ram:PayeeSpecifiedCreditorFinancialInstitution>
            </xsl:if>
          </xsl:if>
          <!-- BT-89 : Payment mandate (direct debit) -->
          <xsl:if test="cac:PaymentMeans/cac:PaymentMandate/cbc:ID">
            <ram:DirectDebitMandateID><xsl:value-of select="cac:PaymentMeans/cac:PaymentMandate/cbc:ID"/></ram:DirectDebitMandateID>
          </xsl:if>
        </ram:SpecifiedTradeSettlementPaymentMeans>
      </xsl:if>

      <!-- BG-23 : VAT breakdown -->
      <xsl:for-each select="cac:TaxTotal[1]/cac:TaxSubtotal">
        <ram:ApplicableTradeTax>
          <ram:CalculatedAmount><xsl:value-of select="cbc:TaxAmount"/></ram:CalculatedAmount>
          <ram:TypeCode>VAT</ram:TypeCode>
          <!-- BT-120 : VAT exemption reason text -->
          <xsl:if test="cac:TaxCategory/cbc:TaxExemptionReason">
            <ram:ExemptionReason><xsl:value-of select="cac:TaxCategory/cbc:TaxExemptionReason"/></ram:ExemptionReason>
          </xsl:if>
          <ram:BasisAmount><xsl:value-of select="cbc:TaxableAmount"/></ram:BasisAmount>
          <ram:CategoryCode><xsl:value-of select="cac:TaxCategory/cbc:ID"/></ram:CategoryCode>
          <!-- BT-121 : VAT exemption reason code -->
          <xsl:if test="cac:TaxCategory/cbc:TaxExemptionReasonCode">
            <ram:ExemptionReasonCode><xsl:value-of select="cac:TaxCategory/cbc:TaxExemptionReasonCode"/></ram:ExemptionReasonCode>
          </xsl:if>
          <xsl:if test="cac:TaxCategory/cbc:Percent">
            <ram:RateApplicablePercent><xsl:value-of select="cac:TaxCategory/cbc:Percent"/></ram:RateApplicablePercent>
          </xsl:if>
        </ram:ApplicableTradeTax>
      </xsl:for-each>

      <!-- BG-14 : Invoicing period (header level) -->
      <xsl:if test="cac:InvoicePeriod">
        <ram:BillingSpecifiedPeriod>
          <xsl:if test="cac:InvoicePeriod/cbc:StartDate">
            <ram:StartDateTime>
              <udt:DateTimeString format="102">
                <xsl:value-of select="translate(cac:InvoicePeriod/cbc:StartDate, '-', '')"/>
              </udt:DateTimeString>
            </ram:StartDateTime>
          </xsl:if>
          <xsl:if test="cac:InvoicePeriod/cbc:EndDate">
            <ram:EndDateTime>
              <udt:DateTimeString format="102">
                <xsl:value-of select="translate(cac:InvoicePeriod/cbc:EndDate, '-', '')"/>
              </udt:DateTimeString>
            </ram:EndDateTime>
          </xsl:if>
        </ram:BillingSpecifiedPeriod>
      </xsl:if>

      <!-- BG-20 : Document level allowances -->
      <xsl:for-each select="cac:AllowanceCharge[cbc:ChargeIndicator='false']">
        <xsl:call-template name="AllowanceCharge"/>
      </xsl:for-each>
      <!-- BG-21 : Document level charges -->
      <xsl:for-each select="cac:AllowanceCharge[cbc:ChargeIndicator='true']">
        <xsl:call-template name="AllowanceCharge"/>
      </xsl:for-each>

      <!-- BT-20 : Payment terms -->
      <xsl:if test="cac:PaymentTerms/cbc:Note or cbc:DueDate">
        <ram:SpecifiedTradePaymentTerms>
          <xsl:if test="cac:PaymentTerms/cbc:Note">
            <ram:Description><xsl:value-of select="cac:PaymentTerms/cbc:Note"/></ram:Description>
          </xsl:if>
          <!-- BT-9 : Payment due date -->
          <xsl:if test="cbc:DueDate">
            <ram:DueDateDateTime>
              <udt:DateTimeString format="102">
                <xsl:value-of select="translate(cbc:DueDate, '-', '')"/>
              </udt:DateTimeString>
            </ram:DueDateDateTime>
          </xsl:if>
        </ram:SpecifiedTradePaymentTerms>
      </xsl:if>

      <!-- BG-22 : Document totals (avant InvoiceReferencedDocument per XSD xs:sequence) -->
      <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
        <!-- BT-106 : Sum of Invoice line net amount -->
        <ram:LineTotalAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:LineExtensionAmount"/></ram:LineTotalAmount>
        <!-- BT-108 : Sum of charges on document level -->
        <xsl:if test="cac:LegalMonetaryTotal/cbc:ChargeTotalAmount">
          <ram:ChargeTotalAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:ChargeTotalAmount"/></ram:ChargeTotalAmount>
        </xsl:if>
        <!-- BT-107 : Sum of allowances on document level -->
        <xsl:if test="cac:LegalMonetaryTotal/cbc:AllowanceTotalAmount">
          <ram:AllowanceTotalAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:AllowanceTotalAmount"/></ram:AllowanceTotalAmount>
        </xsl:if>
        <!-- BT-109 : Invoice total amount without VAT -->
        <ram:TaxBasisTotalAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:TaxExclusiveAmount"/></ram:TaxBasisTotalAmount>
        <!-- BT-110 : Invoice total VAT amount -->
        <xsl:if test="cac:TaxTotal/cbc:TaxAmount">
          <ram:TaxTotalAmount>
            <xsl:attribute name="currencyID"><xsl:value-of select="cbc:DocumentCurrencyCode"/></xsl:attribute>
            <xsl:value-of select="cac:TaxTotal[cbc:TaxAmount/@currencyID = current()/cbc:DocumentCurrencyCode]/cbc:TaxAmount"/>
          </ram:TaxTotalAmount>
        </xsl:if>
        <!-- BT-112 : Invoice total amount with VAT -->
        <ram:GrandTotalAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:TaxInclusiveAmount"/></ram:GrandTotalAmount>
        <!-- BT-113 : Paid amount -->
        <xsl:if test="cac:LegalMonetaryTotal/cbc:PrepaidAmount">
          <ram:TotalPrepaidAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:PrepaidAmount"/></ram:TotalPrepaidAmount>
        </xsl:if>
        <!-- BT-115 : Amount due for payment -->
        <ram:DuePayableAmount><xsl:value-of select="cac:LegalMonetaryTotal/cbc:PayableAmount"/></ram:DuePayableAmount>
      </ram:SpecifiedTradeSettlementHeaderMonetarySummation>

      <!-- BT-25/26 : Preceding Invoice reference (après MonetarySummation per XSD xs:sequence) -->
      <xsl:for-each select="cac:BillingReference/cac:InvoiceDocumentReference">
        <ram:InvoiceReferencedDocument>
          <ram:IssuerAssignedID><xsl:value-of select="cbc:ID"/></ram:IssuerAssignedID>
          <xsl:if test="cbc:IssueDate">
            <ram:FormattedIssueDateTime>
              <qdt:DateTimeString format="102">
                <xsl:value-of select="translate(cbc:IssueDate, '-', '')"/>
              </qdt:DateTimeString>
            </ram:FormattedIssueDateTime>
          </xsl:if>
        </ram:InvoiceReferencedDocument>
      </xsl:for-each>

      <!-- BT-19 : Buyer accounting reference -->
      <xsl:if test="cbc:AccountingCost">
        <ram:ReceivableSpecifiedTradeAccountingAccount>
          <ram:ID><xsl:value-of select="cbc:AccountingCost"/></ram:ID>
        </ram:ReceivableSpecifiedTradeAccountingAccount>
      </xsl:if>
    </ram:ApplicableHeaderTradeSettlement>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- Named template : TradeParty (Seller/Buyer)                   -->
  <!-- ============================================================ -->
  <xsl:template name="TradeParty">
    <xsl:param name="party"/>
    <!-- BT-29/46 : Party identification (GlobalID) -->
    <xsl:if test="$party/cac:PartyIdentification/cbc:ID[@schemeID]">
      <ram:GlobalID>
        <xsl:attribute name="schemeID"><xsl:value-of select="$party/cac:PartyIdentification/cbc:ID/@schemeID"/></xsl:attribute>
        <xsl:value-of select="$party/cac:PartyIdentification/cbc:ID"/>
      </ram:GlobalID>
    </xsl:if>
    <!-- BT-27/44 : Seller/Buyer name -->
    <xsl:choose>
      <xsl:when test="$party/cac:PartyLegalEntity/cbc:RegistrationName">
        <ram:Name><xsl:value-of select="$party/cac:PartyLegalEntity/cbc:RegistrationName"/></ram:Name>
      </xsl:when>
      <xsl:when test="$party/cac:PartyName/cbc:Name">
        <ram:Name><xsl:value-of select="$party/cac:PartyName/cbc:Name"/></ram:Name>
      </xsl:when>
    </xsl:choose>
    <!-- BT-30/47 : Seller/Buyer legal registration -->
    <xsl:if test="$party/cac:PartyLegalEntity/cbc:CompanyID or $party/cac:PartyName/cbc:Name">
      <ram:SpecifiedLegalOrganization>
        <xsl:if test="$party/cac:PartyLegalEntity/cbc:CompanyID">
          <ram:ID>
            <xsl:if test="$party/cac:PartyLegalEntity/cbc:CompanyID/@schemeID">
              <xsl:attribute name="schemeID"><xsl:value-of select="$party/cac:PartyLegalEntity/cbc:CompanyID/@schemeID"/></xsl:attribute>
            </xsl:if>
            <xsl:value-of select="$party/cac:PartyLegalEntity/cbc:CompanyID"/>
          </ram:ID>
        </xsl:if>
        <!-- BT-28/45 : Trading name -->
        <xsl:if test="$party/cac:PartyName/cbc:Name and $party/cac:PartyName/cbc:Name != $party/cac:PartyLegalEntity/cbc:RegistrationName">
          <ram:TradingBusinessName><xsl:value-of select="$party/cac:PartyName/cbc:Name"/></ram:TradingBusinessName>
        </xsl:if>
      </ram:SpecifiedLegalOrganization>
    </xsl:if>
    <!-- BG-5/8 : Postal address -->
    <xsl:if test="$party/cac:PostalAddress">
      <xsl:call-template name="PostalAddress">
        <xsl:with-param name="addr" select="$party/cac:PostalAddress"/>
      </xsl:call-template>
    </xsl:if>
    <!-- BT-34/49 : Electronic address -->
    <xsl:if test="$party/cbc:EndpointID">
      <ram:URIUniversalCommunication>
        <ram:URIID>
          <xsl:if test="$party/cbc:EndpointID/@schemeID">
            <xsl:attribute name="schemeID"><xsl:value-of select="$party/cbc:EndpointID/@schemeID"/></xsl:attribute>
          </xsl:if>
          <xsl:value-of select="$party/cbc:EndpointID"/>
        </ram:URIID>
      </ram:URIUniversalCommunication>
    </xsl:if>
    <!-- BT-31/48 : VAT identifier -->
    <xsl:if test="$party/cac:PartyTaxScheme/cbc:CompanyID">
      <ram:SpecifiedTaxRegistration>
        <ram:ID schemeID="VA"><xsl:value-of select="$party/cac:PartyTaxScheme/cbc:CompanyID"/></ram:ID>
      </ram:SpecifiedTaxRegistration>
    </xsl:if>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- Named template : PostalAddress                               -->
  <!-- ============================================================ -->
  <xsl:template name="PostalAddress">
    <xsl:param name="addr"/>
    <ram:PostalTradeAddress>
      <xsl:if test="$addr/cbc:PostalZone">
        <ram:PostcodeCode><xsl:value-of select="$addr/cbc:PostalZone"/></ram:PostcodeCode>
      </xsl:if>
      <xsl:if test="$addr/cbc:StreetName">
        <ram:LineOne><xsl:value-of select="$addr/cbc:StreetName"/></ram:LineOne>
      </xsl:if>
      <xsl:if test="$addr/cbc:AdditionalStreetName">
        <ram:LineTwo><xsl:value-of select="$addr/cbc:AdditionalStreetName"/></ram:LineTwo>
      </xsl:if>
      <xsl:if test="$addr/cac:AddressLine/cbc:Line">
        <ram:LineThree><xsl:value-of select="$addr/cac:AddressLine/cbc:Line"/></ram:LineThree>
      </xsl:if>
      <xsl:if test="$addr/cbc:CityName">
        <ram:CityName><xsl:value-of select="$addr/cbc:CityName"/></ram:CityName>
      </xsl:if>
      <xsl:if test="$addr/cac:Country/cbc:IdentificationCode">
        <ram:CountryID><xsl:value-of select="$addr/cac:Country/cbc:IdentificationCode"/></ram:CountryID>
      </xsl:if>
      <xsl:if test="$addr/cbc:CountrySubentity">
        <ram:CountrySubDivisionName><xsl:value-of select="$addr/cbc:CountrySubentity"/></ram:CountrySubDivisionName>
      </xsl:if>
    </ram:PostalTradeAddress>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- Named template : AllowanceCharge (BG-20/BG-21)               -->
  <!-- ============================================================ -->
  <xsl:template name="AllowanceCharge">
    <ram:SpecifiedTradeAllowanceCharge>
      <ram:ChargeIndicator>
        <udt:Indicator><xsl:value-of select="cbc:ChargeIndicator"/></udt:Indicator>
      </ram:ChargeIndicator>
      <xsl:if test="cbc:Amount">
        <ram:ActualAmount><xsl:value-of select="cbc:Amount"/></ram:ActualAmount>
      </xsl:if>
      <xsl:if test="cbc:AllowanceChargeReason">
        <ram:Reason><xsl:value-of select="cbc:AllowanceChargeReason"/></ram:Reason>
      </xsl:if>
      <ram:CategoryTradeTax>
        <ram:TypeCode>VAT</ram:TypeCode>
        <ram:CategoryCode><xsl:value-of select="cac:TaxCategory/cbc:ID"/></ram:CategoryCode>
        <xsl:if test="cac:TaxCategory/cbc:Percent">
          <ram:RateApplicablePercent><xsl:value-of select="cac:TaxCategory/cbc:Percent"/></ram:RateApplicablePercent>
        </xsl:if>
      </ram:CategoryTradeTax>
    </ram:SpecifiedTradeAllowanceCharge>
  </xsl:template>

</xsl:stylesheet>
