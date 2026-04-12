<?xml version="1.0" encoding="UTF-8"?>
<!--
  CII D22B → Flux 1 Base CII (PPF)
  
  Transformation d'une facture CII complète vers le format allégé Flux 1 Base
  conforme au XSD F1_BASE_CII_D22B des spécifications externes PPF v3.1.
  
  Principales transformations :
  - BT-24 (GuidelineID) → urn.cpro.gouv.fr:1p0:einvoicingextract#Base
  - Suppression de toutes les lignes de facture (IncludedSupplyChainTradeLineItem)
  - Allègement des parties (TradeParty) : GlobalID, Name, LegalOrganization/ID,
    PostalTradeAddress (allégée), TaxRegistration
  - Suppression des remises/majorations, moyens de paiement, pièces jointes
  - MonetarySummation réduit à TaxBasisTotalAmount + TaxTotalAmount
  - TradePaymentTerms réduit à DueDateDateTime
  
  Conforme aux règles BR-FR-MAP-15 et BR-FR-MAP-24.
-->
<xsl:stylesheet version="2.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
  xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
  xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"
  xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100"
  exclude-result-prefixes="">

  <xsl:output method="xml" indent="yes" encoding="UTF-8"/>
  <xsl:strip-space elements="*"/>

  <!-- ============================================================
       Racine : CrossIndustryInvoice
       ============================================================ -->
  <xsl:template match="rsm:CrossIndustryInvoice">
    <rsm:CrossIndustryInvoice
      xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
      xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
      xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"
      xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
      <xsl:apply-templates select="rsm:ExchangedDocumentContext"/>
      <xsl:apply-templates select="rsm:ExchangedDocument"/>
      <xsl:apply-templates select="rsm:SupplyChainTradeTransaction"/>
    </rsm:CrossIndustryInvoice>
  </xsl:template>

  <!-- ============================================================
       ExchangedDocumentContext : change BT-24 → Base
       ============================================================ -->
  <xsl:template match="rsm:ExchangedDocumentContext">
    <rsm:ExchangedDocumentContext>
      <xsl:if test="ram:BusinessProcessSpecifiedDocumentContextParameter">
        <ram:BusinessProcessSpecifiedDocumentContextParameter>
          <ram:ID><xsl:value-of select="ram:BusinessProcessSpecifiedDocumentContextParameter/ram:ID"/></ram:ID>
        </ram:BusinessProcessSpecifiedDocumentContextParameter>
      </xsl:if>
      <ram:GuidelineSpecifiedDocumentContextParameter>
        <ram:ID>urn.cpro.gouv.fr:1p0:einvoicingextract#Base</ram:ID>
      </ram:GuidelineSpecifiedDocumentContextParameter>
    </rsm:ExchangedDocumentContext>
  </xsl:template>

  <!-- ============================================================
       ExchangedDocument : ID, TypeCode, IssueDateTime, IncludedNote
       ============================================================ -->
  <xsl:template match="rsm:ExchangedDocument">
    <rsm:ExchangedDocument>
      <ram:ID><xsl:value-of select="ram:ID"/></ram:ID>
      <xsl:if test="ram:TypeCode">
        <ram:TypeCode><xsl:value-of select="ram:TypeCode"/></ram:TypeCode>
      </xsl:if>
      <xsl:copy-of select="ram:IssueDateTime"/>
      <xsl:for-each select="ram:IncludedNote">
        <ram:IncludedNote>
          <xsl:if test="ram:Content">
            <ram:Content><xsl:value-of select="ram:Content"/></ram:Content>
          </xsl:if>
          <xsl:if test="ram:SubjectCode">
            <ram:SubjectCode><xsl:value-of select="ram:SubjectCode"/></ram:SubjectCode>
          </xsl:if>
        </ram:IncludedNote>
      </xsl:for-each>
    </rsm:ExchangedDocument>
  </xsl:template>

  <!-- ============================================================
       SupplyChainTradeTransaction : SANS lignes (BR-FR-MAP-24)
       ============================================================ -->
  <xsl:template match="rsm:SupplyChainTradeTransaction">
    <rsm:SupplyChainTradeTransaction>
      <!-- Pas de IncludedSupplyChainTradeLineItem dans Flux 1 Base -->
      <xsl:apply-templates select="ram:ApplicableHeaderTradeAgreement"/>
      <xsl:apply-templates select="ram:ApplicableHeaderTradeDelivery"/>
      <xsl:apply-templates select="ram:ApplicableHeaderTradeSettlement"/>
    </rsm:SupplyChainTradeTransaction>
  </xsl:template>

  <!-- ============================================================
       HeaderTradeAgreement : Seller, Buyer, SellerTaxRep seulement
       ============================================================ -->
  <xsl:template match="ram:ApplicableHeaderTradeAgreement">
    <ram:ApplicableHeaderTradeAgreement>
      <xsl:if test="ram:SellerTradeParty">
        <ram:SellerTradeParty>
          <xsl:call-template name="F1BaseTradeParty">
            <xsl:with-param name="party" select="ram:SellerTradeParty"/>
          </xsl:call-template>
        </ram:SellerTradeParty>
      </xsl:if>
      <xsl:if test="ram:BuyerTradeParty">
        <ram:BuyerTradeParty>
          <xsl:call-template name="F1BaseTradeParty">
            <xsl:with-param name="party" select="ram:BuyerTradeParty"/>
          </xsl:call-template>
        </ram:BuyerTradeParty>
      </xsl:if>
      <xsl:if test="ram:SellerTaxRepresentativeTradeParty">
        <ram:SellerTaxRepresentativeTradeParty>
          <xsl:call-template name="F1BaseTradeParty">
            <xsl:with-param name="party" select="ram:SellerTaxRepresentativeTradeParty"/>
          </xsl:call-template>
        </ram:SellerTaxRepresentativeTradeParty>
      </xsl:if>
    </ram:ApplicableHeaderTradeAgreement>
  </xsl:template>

  <!-- ============================================================
       HeaderTradeDelivery : ActualDeliverySupplyChainEvent seulement
       ============================================================ -->
  <xsl:template match="ram:ApplicableHeaderTradeDelivery">
    <ram:ApplicableHeaderTradeDelivery>
      <xsl:if test="ram:ActualDeliverySupplyChainEvent">
        <ram:ActualDeliverySupplyChainEvent>
          <xsl:copy-of select="ram:ActualDeliverySupplyChainEvent/ram:OccurrenceDateTime"/>
        </ram:ActualDeliverySupplyChainEvent>
      </xsl:if>
    </ram:ApplicableHeaderTradeDelivery>
  </xsl:template>

  <!-- ============================================================
       HeaderTradeSettlement : allégé
       ============================================================ -->
  <xsl:template match="ram:ApplicableHeaderTradeSettlement">
    <ram:ApplicableHeaderTradeSettlement>
      <!-- InvoiceCurrencyCode -->
      <xsl:if test="ram:InvoiceCurrencyCode">
        <ram:InvoiceCurrencyCode><xsl:value-of select="ram:InvoiceCurrencyCode"/></ram:InvoiceCurrencyCode>
      </xsl:if>

      <!-- ApplicableTradeTax (ventilation TVA) -->
      <xsl:for-each select="ram:ApplicableTradeTax">
        <ram:ApplicableTradeTax>
          <xsl:if test="ram:CalculatedAmount">
            <ram:CalculatedAmount>
              <xsl:copy-of select="ram:CalculatedAmount/@*"/>
              <xsl:value-of select="ram:CalculatedAmount"/>
            </ram:CalculatedAmount>
          </xsl:if>
          <xsl:if test="ram:TypeCode">
            <ram:TypeCode><xsl:value-of select="ram:TypeCode"/></ram:TypeCode>
          </xsl:if>
          <xsl:if test="ram:ExemptionReason">
            <ram:ExemptionReason><xsl:value-of select="ram:ExemptionReason"/></ram:ExemptionReason>
          </xsl:if>
          <xsl:if test="ram:BasisAmount">
            <ram:BasisAmount>
              <xsl:copy-of select="ram:BasisAmount/@*"/>
              <xsl:value-of select="ram:BasisAmount"/>
            </ram:BasisAmount>
          </xsl:if>
          <xsl:if test="ram:CategoryCode">
            <ram:CategoryCode><xsl:value-of select="ram:CategoryCode"/></ram:CategoryCode>
          </xsl:if>
          <xsl:if test="ram:ExemptionReasonCode">
            <ram:ExemptionReasonCode><xsl:value-of select="ram:ExemptionReasonCode"/></ram:ExemptionReasonCode>
          </xsl:if>
          <xsl:if test="ram:DueDateTypeCode">
            <ram:DueDateTypeCode><xsl:value-of select="ram:DueDateTypeCode"/></ram:DueDateTypeCode>
          </xsl:if>
          <xsl:if test="ram:RateApplicablePercent">
            <ram:RateApplicablePercent><xsl:value-of select="ram:RateApplicablePercent"/></ram:RateApplicablePercent>
          </xsl:if>
        </ram:ApplicableTradeTax>
      </xsl:for-each>

      <!-- BillingSpecifiedPeriod -->
      <xsl:if test="ram:BillingSpecifiedPeriod">
        <ram:BillingSpecifiedPeriod>
          <xsl:copy-of select="ram:BillingSpecifiedPeriod/ram:StartDateTime"/>
          <xsl:copy-of select="ram:BillingSpecifiedPeriod/ram:EndDateTime"/>
        </ram:BillingSpecifiedPeriod>
      </xsl:if>

      <!-- SpecifiedTradePaymentTerms : DueDateDateTime seulement -->
      <xsl:if test="ram:SpecifiedTradePaymentTerms">
        <ram:SpecifiedTradePaymentTerms>
          <xsl:copy-of select="ram:SpecifiedTradePaymentTerms/ram:DueDateDateTime"/>
        </ram:SpecifiedTradePaymentTerms>
      </xsl:if>

      <!-- MonetarySummation : TaxBasisTotalAmount + TaxTotalAmount seulement -->
      <xsl:if test="ram:SpecifiedTradeSettlementHeaderMonetarySummation">
        <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
          <xsl:if test="ram:SpecifiedTradeSettlementHeaderMonetarySummation/ram:TaxBasisTotalAmount">
            <ram:TaxBasisTotalAmount>
              <xsl:copy-of select="ram:SpecifiedTradeSettlementHeaderMonetarySummation/ram:TaxBasisTotalAmount/@*"/>
              <xsl:value-of select="ram:SpecifiedTradeSettlementHeaderMonetarySummation/ram:TaxBasisTotalAmount"/>
            </ram:TaxBasisTotalAmount>
          </xsl:if>
          <xsl:for-each select="ram:SpecifiedTradeSettlementHeaderMonetarySummation/ram:TaxTotalAmount">
            <ram:TaxTotalAmount>
              <xsl:copy-of select="@*"/>
              <xsl:value-of select="."/>
            </ram:TaxTotalAmount>
          </xsl:for-each>
        </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
      </xsl:if>

      <!-- InvoiceReferencedDocument -->
      <xsl:for-each select="ram:InvoiceReferencedDocument">
        <ram:InvoiceReferencedDocument>
          <xsl:if test="ram:IssuerAssignedID">
            <ram:IssuerAssignedID><xsl:value-of select="ram:IssuerAssignedID"/></ram:IssuerAssignedID>
          </xsl:if>
        </ram:InvoiceReferencedDocument>
      </xsl:for-each>
    </ram:ApplicableHeaderTradeSettlement>
  </xsl:template>

  <!-- ============================================================
       Template nommé : TradeParty allégé pour Flux 1 Base
       Garde : GlobalID, Name, LegalOrganization/ID,
               PostalTradeAddress (allégée), TaxRegistration
       ============================================================ -->
  <xsl:template name="F1BaseTradeParty">
    <xsl:param name="party"/>

    <!-- GlobalID -->
    <xsl:if test="$party/ram:GlobalID">
      <ram:GlobalID>
        <xsl:copy-of select="$party/ram:GlobalID/@*"/>
        <xsl:value-of select="$party/ram:GlobalID"/>
      </ram:GlobalID>
    </xsl:if>

    <!-- Name -->
    <xsl:if test="$party/ram:Name">
      <ram:Name><xsl:value-of select="$party/ram:Name"/></ram:Name>
    </xsl:if>

    <!-- SpecifiedLegalOrganization : ID seulement -->
    <xsl:if test="$party/ram:SpecifiedLegalOrganization">
      <ram:SpecifiedLegalOrganization>
        <ram:ID>
          <xsl:copy-of select="$party/ram:SpecifiedLegalOrganization/ram:ID/@*"/>
          <xsl:value-of select="$party/ram:SpecifiedLegalOrganization/ram:ID"/>
        </ram:ID>
      </ram:SpecifiedLegalOrganization>
    </xsl:if>

    <!-- PostalTradeAddress : allégée -->
    <xsl:if test="$party/ram:PostalTradeAddress">
      <ram:PostalTradeAddress>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:PostcodeCode">
          <ram:PostcodeCode><xsl:value-of select="$party/ram:PostalTradeAddress/ram:PostcodeCode"/></ram:PostcodeCode>
        </xsl:if>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:LineOne">
          <ram:LineOne><xsl:value-of select="$party/ram:PostalTradeAddress/ram:LineOne"/></ram:LineOne>
        </xsl:if>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:LineTwo">
          <ram:LineTwo><xsl:value-of select="$party/ram:PostalTradeAddress/ram:LineTwo"/></ram:LineTwo>
        </xsl:if>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:LineThree">
          <ram:LineThree><xsl:value-of select="$party/ram:PostalTradeAddress/ram:LineThree"/></ram:LineThree>
        </xsl:if>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:CityName">
          <ram:CityName><xsl:value-of select="$party/ram:PostalTradeAddress/ram:CityName"/></ram:CityName>
        </xsl:if>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:CountryID">
          <ram:CountryID><xsl:value-of select="$party/ram:PostalTradeAddress/ram:CountryID"/></ram:CountryID>
        </xsl:if>
        <xsl:if test="$party/ram:PostalTradeAddress/ram:CountrySubDivisionName">
          <ram:CountrySubDivisionName><xsl:value-of select="$party/ram:PostalTradeAddress/ram:CountrySubDivisionName"/></ram:CountrySubDivisionName>
        </xsl:if>
      </ram:PostalTradeAddress>
    </xsl:if>

    <!-- SpecifiedTaxRegistration -->
    <xsl:if test="$party/ram:SpecifiedTaxRegistration">
      <ram:SpecifiedTaxRegistration>
        <xsl:if test="$party/ram:SpecifiedTaxRegistration/ram:ID">
          <ram:ID>
            <xsl:copy-of select="$party/ram:SpecifiedTaxRegistration/ram:ID/@*"/>
            <xsl:value-of select="$party/ram:SpecifiedTaxRegistration/ram:ID"/>
          </ram:ID>
        </xsl:if>
      </ram:SpecifiedTaxRegistration>
    </xsl:if>
  </xsl:template>

</xsl:stylesheet>
