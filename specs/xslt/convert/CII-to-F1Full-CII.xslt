<?xml version="1.0" encoding="UTF-8"?>
<!--
  CII D22B → Flux 1 Full CII (PPF)
  
  Transformation d'une facture CII complète vers le format Flux 1 Full
  conforme au XSD F1_FULL_CII_D22B des spécifications externes PPF v3.1.
  
  Le profil Full conserve les lignes de facture, remises/majorations,
  livraison et prix, mais filtre les éléments non autorisés par le XSD :
  - Pas de BuyerReference, PaymentMeans, AdditionalReferencedDocument
  - Lignes : pas de LineID, pas de Description produit, pas de ApplicableTradeTax
  - GrossPriceProductTradePrice obligatoire avant NetPriceProductTradePrice
  - TradeParty allégé (même structure que Base)
  
  BT-24 (GuidelineID) → urn.cpro.gouv.fr:1p0:einvoicingextract#Full
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
       Racine
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
       ExchangedDocumentContext : BT-24 → Full
       ============================================================ -->
  <xsl:template match="rsm:ExchangedDocumentContext">
    <rsm:ExchangedDocumentContext>
      <xsl:if test="ram:BusinessProcessSpecifiedDocumentContextParameter">
        <ram:BusinessProcessSpecifiedDocumentContextParameter>
          <ram:ID><xsl:value-of select="ram:BusinessProcessSpecifiedDocumentContextParameter/ram:ID"/></ram:ID>
        </ram:BusinessProcessSpecifiedDocumentContextParameter>
      </xsl:if>
      <ram:GuidelineSpecifiedDocumentContextParameter>
        <ram:ID>urn.cpro.gouv.fr:1p0:einvoicingextract#Full</ram:ID>
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
       SupplyChainTradeTransaction : AVEC lignes (Full)
       ============================================================ -->
  <xsl:template match="rsm:SupplyChainTradeTransaction">
    <rsm:SupplyChainTradeTransaction>
      <xsl:for-each select="ram:IncludedSupplyChainTradeLineItem">
        <xsl:call-template name="F1FullLineItem"/>
      </xsl:for-each>
      <xsl:apply-templates select="ram:ApplicableHeaderTradeAgreement"/>
      <xsl:apply-templates select="ram:ApplicableHeaderTradeDelivery"/>
      <xsl:apply-templates select="ram:ApplicableHeaderTradeSettlement"/>
    </rsm:SupplyChainTradeTransaction>
  </xsl:template>

  <!-- ============================================================
       Ligne de facture (IncludedSupplyChainTradeLineItem)
       XSD: AssociatedDocumentLineDocument?, SpecifiedTradeProduct,
            SpecifiedLineTradeAgreement, SpecifiedLineTradeDelivery,
            SpecifiedLineTradeSettlement?
       ============================================================ -->
  <xsl:template name="F1FullLineItem">
    <ram:IncludedSupplyChainTradeLineItem>
      <!-- AssociatedDocumentLineDocument : seulement IncludedNote (pas de LineID) -->
      <xsl:if test="ram:AssociatedDocumentLineDocument/ram:IncludedNote">
        <ram:AssociatedDocumentLineDocument>
          <xsl:for-each select="ram:AssociatedDocumentLineDocument/ram:IncludedNote">
            <ram:IncludedNote>
              <xsl:if test="ram:Content">
                <ram:Content><xsl:value-of select="ram:Content"/></ram:Content>
              </xsl:if>
              <xsl:if test="ram:SubjectCode">
                <ram:SubjectCode><xsl:value-of select="ram:SubjectCode"/></ram:SubjectCode>
              </xsl:if>
            </ram:IncludedNote>
          </xsl:for-each>
        </ram:AssociatedDocumentLineDocument>
      </xsl:if>

      <!-- SpecifiedTradeProduct : seulement Name -->
      <ram:SpecifiedTradeProduct>
        <ram:Name><xsl:value-of select="ram:SpecifiedTradeProduct/ram:Name"/></ram:Name>
      </ram:SpecifiedTradeProduct>

      <!-- SpecifiedLineTradeAgreement : GrossPrice puis NetPrice -->
      <ram:SpecifiedLineTradeAgreement>
        <!-- GrossPriceProductTradePrice (obligatoire) -->
        <ram:GrossPriceProductTradePrice>
          <xsl:choose>
            <xsl:when test="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice">
              <ram:ChargeAmount>
                <xsl:copy-of select="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice/ram:ChargeAmount/@*"/>
                <xsl:value-of select="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice/ram:ChargeAmount"/>
              </ram:ChargeAmount>
              <xsl:if test="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice/ram:AppliedTradeAllowanceCharge">
                <ram:AppliedTradeAllowanceCharge>
                  <ram:ChargeIndicator>
                    <udt:Indicator><xsl:value-of select="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice/ram:AppliedTradeAllowanceCharge/ram:ChargeIndicator/udt:Indicator"/></udt:Indicator>
                  </ram:ChargeIndicator>
                  <ram:ActualAmount>
                    <xsl:copy-of select="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice/ram:AppliedTradeAllowanceCharge/ram:ActualAmount/@*"/>
                    <xsl:value-of select="ram:SpecifiedLineTradeAgreement/ram:GrossPriceProductTradePrice/ram:AppliedTradeAllowanceCharge/ram:ActualAmount"/>
                  </ram:ActualAmount>
                </ram:AppliedTradeAllowanceCharge>
              </xsl:if>
            </xsl:when>
            <xsl:otherwise>
              <!-- Fallback : utiliser le NetPrice comme GrossPrice -->
              <ram:ChargeAmount>
                <xsl:copy-of select="ram:SpecifiedLineTradeAgreement/ram:NetPriceProductTradePrice/ram:ChargeAmount/@*"/>
                <xsl:value-of select="ram:SpecifiedLineTradeAgreement/ram:NetPriceProductTradePrice/ram:ChargeAmount"/>
              </ram:ChargeAmount>
            </xsl:otherwise>
          </xsl:choose>
        </ram:GrossPriceProductTradePrice>
        <!-- NetPriceProductTradePrice (obligatoire) -->
        <ram:NetPriceProductTradePrice>
          <ram:ChargeAmount>
            <xsl:copy-of select="ram:SpecifiedLineTradeAgreement/ram:NetPriceProductTradePrice/ram:ChargeAmount/@*"/>
            <xsl:value-of select="ram:SpecifiedLineTradeAgreement/ram:NetPriceProductTradePrice/ram:ChargeAmount"/>
          </ram:ChargeAmount>
        </ram:NetPriceProductTradePrice>
      </ram:SpecifiedLineTradeAgreement>

      <!-- SpecifiedLineTradeDelivery : BilledQuantity, ShipToTradeParty?, ActualDelivery? -->
      <ram:SpecifiedLineTradeDelivery>
        <xsl:copy-of select="ram:SpecifiedLineTradeDelivery/ram:BilledQuantity"/>
        <xsl:if test="ram:SpecifiedLineTradeDelivery/ram:ShipToTradeParty">
          <ram:ShipToTradeParty>
            <xsl:call-template name="F1FullTradeParty">
              <xsl:with-param name="party" select="ram:SpecifiedLineTradeDelivery/ram:ShipToTradeParty"/>
            </xsl:call-template>
          </ram:ShipToTradeParty>
        </xsl:if>
        <xsl:if test="ram:SpecifiedLineTradeDelivery/ram:ActualDeliverySupplyChainEvent">
          <ram:ActualDeliverySupplyChainEvent>
            <xsl:copy-of select="ram:SpecifiedLineTradeDelivery/ram:ActualDeliverySupplyChainEvent/ram:OccurrenceDateTime"/>
          </ram:ActualDeliverySupplyChainEvent>
        </xsl:if>
      </ram:SpecifiedLineTradeDelivery>

      <!-- SpecifiedLineTradeSettlement : BillingPeriod, AllowanceCharge, InvoiceRef (pas de ApplicableTradeTax) -->
      <xsl:if test="ram:SpecifiedLineTradeSettlement">
        <ram:SpecifiedLineTradeSettlement>
          <xsl:if test="ram:SpecifiedLineTradeSettlement/ram:BillingSpecifiedPeriod">
            <ram:BillingSpecifiedPeriod>
              <xsl:copy-of select="ram:SpecifiedLineTradeSettlement/ram:BillingSpecifiedPeriod/ram:StartDateTime"/>
              <xsl:copy-of select="ram:SpecifiedLineTradeSettlement/ram:BillingSpecifiedPeriod/ram:EndDateTime"/>
            </ram:BillingSpecifiedPeriod>
          </xsl:if>
          <xsl:for-each select="ram:SpecifiedLineTradeSettlement/ram:SpecifiedTradeAllowanceCharge">
            <ram:SpecifiedTradeAllowanceCharge>
              <ram:ChargeIndicator>
                <udt:Indicator><xsl:value-of select="ram:ChargeIndicator/udt:Indicator"/></udt:Indicator>
              </ram:ChargeIndicator>
              <ram:ActualAmount>
                <xsl:copy-of select="ram:ActualAmount/@*"/>
                <xsl:value-of select="ram:ActualAmount"/>
              </ram:ActualAmount>
              <xsl:if test="ram:CategoryTradeTax">
                <xsl:call-template name="F1FullTradeTax">
                  <xsl:with-param name="tax" select="ram:CategoryTradeTax"/>
                  <xsl:with-param name="wrapper" select="'ram:CategoryTradeTax'"/>
                </xsl:call-template>
              </xsl:if>
            </ram:SpecifiedTradeAllowanceCharge>
          </xsl:for-each>
          <xsl:for-each select="ram:SpecifiedLineTradeSettlement/ram:InvoiceReferencedDocument">
            <ram:InvoiceReferencedDocument>
              <xsl:if test="ram:IssuerAssignedID">
                <ram:IssuerAssignedID><xsl:value-of select="ram:IssuerAssignedID"/></ram:IssuerAssignedID>
              </xsl:if>
              <xsl:copy-of select="ram:FormattedIssueDateTime"/>
            </ram:InvoiceReferencedDocument>
          </xsl:for-each>
        </ram:SpecifiedLineTradeSettlement>
      </xsl:if>
    </ram:IncludedSupplyChainTradeLineItem>
  </xsl:template>

  <!-- ============================================================
       HeaderTradeAgreement : Seller, Buyer, SellerTaxRep (pas de BuyerReference)
       ============================================================ -->
  <xsl:template match="ram:ApplicableHeaderTradeAgreement">
    <ram:ApplicableHeaderTradeAgreement>
      <xsl:if test="ram:SellerTradeParty">
        <ram:SellerTradeParty>
          <xsl:call-template name="F1FullTradeParty">
            <xsl:with-param name="party" select="ram:SellerTradeParty"/>
          </xsl:call-template>
        </ram:SellerTradeParty>
      </xsl:if>
      <xsl:if test="ram:BuyerTradeParty">
        <ram:BuyerTradeParty>
          <xsl:call-template name="F1FullTradeParty">
            <xsl:with-param name="party" select="ram:BuyerTradeParty"/>
          </xsl:call-template>
        </ram:BuyerTradeParty>
      </xsl:if>
      <xsl:if test="ram:SellerTaxRepresentativeTradeParty">
        <ram:SellerTaxRepresentativeTradeParty>
          <xsl:call-template name="F1FullTradeParty">
            <xsl:with-param name="party" select="ram:SellerTaxRepresentativeTradeParty"/>
          </xsl:call-template>
        </ram:SellerTaxRepresentativeTradeParty>
      </xsl:if>
    </ram:ApplicableHeaderTradeAgreement>
  </xsl:template>

  <!-- ============================================================
       HeaderTradeDelivery : ShipToTradeParty + ActualDelivery
       ============================================================ -->
  <xsl:template match="ram:ApplicableHeaderTradeDelivery">
    <ram:ApplicableHeaderTradeDelivery>
      <xsl:if test="ram:ShipToTradeParty">
        <ram:ShipToTradeParty>
          <xsl:call-template name="F1FullTradeParty">
            <xsl:with-param name="party" select="ram:ShipToTradeParty"/>
          </xsl:call-template>
        </ram:ShipToTradeParty>
      </xsl:if>
      <xsl:if test="ram:ActualDeliverySupplyChainEvent">
        <ram:ActualDeliverySupplyChainEvent>
          <xsl:copy-of select="ram:ActualDeliverySupplyChainEvent/ram:OccurrenceDateTime"/>
        </ram:ActualDeliverySupplyChainEvent>
      </xsl:if>
    </ram:ApplicableHeaderTradeDelivery>
  </xsl:template>

  <!-- ============================================================
       HeaderTradeSettlement : allégé (pas de PaymentMeans)
       Ordre XSD : InvoiceCurrencyCode, ApplicableTradeTax,
                   BillingSpecifiedPeriod, SpecifiedTradeAllowanceCharge,
                   SpecifiedTradePaymentTerms, MonetarySummation,
                   InvoiceReferencedDocument
       ============================================================ -->
  <xsl:template match="ram:ApplicableHeaderTradeSettlement">
    <ram:ApplicableHeaderTradeSettlement>
      <xsl:if test="ram:InvoiceCurrencyCode">
        <ram:InvoiceCurrencyCode><xsl:value-of select="ram:InvoiceCurrencyCode"/></ram:InvoiceCurrencyCode>
      </xsl:if>

      <!-- ApplicableTradeTax -->
      <xsl:for-each select="ram:ApplicableTradeTax">
        <xsl:call-template name="F1FullTradeTax">
          <xsl:with-param name="tax" select="."/>
          <xsl:with-param name="wrapper" select="'ram:ApplicableTradeTax'"/>
        </xsl:call-template>
      </xsl:for-each>

      <!-- BillingSpecifiedPeriod -->
      <xsl:if test="ram:BillingSpecifiedPeriod">
        <ram:BillingSpecifiedPeriod>
          <xsl:copy-of select="ram:BillingSpecifiedPeriod/ram:StartDateTime"/>
          <xsl:copy-of select="ram:BillingSpecifiedPeriod/ram:EndDateTime"/>
        </ram:BillingSpecifiedPeriod>
      </xsl:if>

      <!-- SpecifiedTradeAllowanceCharge (document-level) -->
      <xsl:for-each select="ram:SpecifiedTradeAllowanceCharge">
        <ram:SpecifiedTradeAllowanceCharge>
          <ram:ChargeIndicator>
            <udt:Indicator><xsl:value-of select="ram:ChargeIndicator/udt:Indicator"/></udt:Indicator>
          </ram:ChargeIndicator>
          <ram:ActualAmount>
            <xsl:copy-of select="ram:ActualAmount/@*"/>
            <xsl:value-of select="ram:ActualAmount"/>
          </ram:ActualAmount>
          <xsl:if test="ram:CategoryTradeTax">
            <xsl:call-template name="F1FullTradeTax">
              <xsl:with-param name="tax" select="ram:CategoryTradeTax"/>
              <xsl:with-param name="wrapper" select="'ram:CategoryTradeTax'"/>
            </xsl:call-template>
          </xsl:if>
        </ram:SpecifiedTradeAllowanceCharge>
      </xsl:for-each>

      <!-- SpecifiedTradePaymentTerms : DueDateDateTime seulement -->
      <xsl:if test="ram:SpecifiedTradePaymentTerms">
        <ram:SpecifiedTradePaymentTerms>
          <xsl:copy-of select="ram:SpecifiedTradePaymentTerms/ram:DueDateDateTime"/>
        </ram:SpecifiedTradePaymentTerms>
      </xsl:if>

      <!-- MonetarySummation : TaxBasisTotalAmount + TaxTotalAmount -->
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
          <xsl:copy-of select="ram:FormattedIssueDateTime"/>
        </ram:InvoiceReferencedDocument>
      </xsl:for-each>
    </ram:ApplicableHeaderTradeSettlement>
  </xsl:template>

  <!-- ============================================================
       Template nommé : TradeTax (réutilisé pour header et lignes)
       ============================================================ -->
  <xsl:template name="F1FullTradeTax">
    <xsl:param name="tax"/>
    <xsl:param name="wrapper"/>
    <xsl:element name="{$wrapper}">
      <xsl:if test="$tax/ram:CalculatedAmount">
        <ram:CalculatedAmount>
          <xsl:copy-of select="$tax/ram:CalculatedAmount/@*"/>
          <xsl:value-of select="$tax/ram:CalculatedAmount"/>
        </ram:CalculatedAmount>
      </xsl:if>
      <xsl:if test="$tax/ram:TypeCode">
        <ram:TypeCode><xsl:value-of select="$tax/ram:TypeCode"/></ram:TypeCode>
      </xsl:if>
      <xsl:if test="$tax/ram:ExemptionReason">
        <ram:ExemptionReason><xsl:value-of select="$tax/ram:ExemptionReason"/></ram:ExemptionReason>
      </xsl:if>
      <xsl:if test="$tax/ram:BasisAmount">
        <ram:BasisAmount>
          <xsl:copy-of select="$tax/ram:BasisAmount/@*"/>
          <xsl:value-of select="$tax/ram:BasisAmount"/>
        </ram:BasisAmount>
      </xsl:if>
      <xsl:if test="$tax/ram:CategoryCode">
        <ram:CategoryCode><xsl:value-of select="$tax/ram:CategoryCode"/></ram:CategoryCode>
      </xsl:if>
      <xsl:if test="$tax/ram:ExemptionReasonCode">
        <ram:ExemptionReasonCode><xsl:value-of select="$tax/ram:ExemptionReasonCode"/></ram:ExemptionReasonCode>
      </xsl:if>
      <xsl:if test="$tax/ram:DueDateTypeCode">
        <ram:DueDateTypeCode><xsl:value-of select="$tax/ram:DueDateTypeCode"/></ram:DueDateTypeCode>
      </xsl:if>
      <xsl:if test="$tax/ram:RateApplicablePercent">
        <ram:RateApplicablePercent><xsl:value-of select="$tax/ram:RateApplicablePercent"/></ram:RateApplicablePercent>
      </xsl:if>
    </xsl:element>
  </xsl:template>

  <!-- ============================================================
       Template nommé : TradeParty (même structure que Base)
       GlobalID, Name, LegalOrganization/ID, PostalTradeAddress, TaxRegistration
       ============================================================ -->
  <xsl:template name="F1FullTradeParty">
    <xsl:param name="party"/>

    <xsl:if test="$party/ram:GlobalID">
      <ram:GlobalID>
        <xsl:copy-of select="$party/ram:GlobalID/@*"/>
        <xsl:value-of select="$party/ram:GlobalID"/>
      </ram:GlobalID>
    </xsl:if>

    <xsl:if test="$party/ram:Name">
      <ram:Name><xsl:value-of select="$party/ram:Name"/></ram:Name>
    </xsl:if>

    <xsl:if test="$party/ram:SpecifiedLegalOrganization">
      <ram:SpecifiedLegalOrganization>
        <ram:ID>
          <xsl:copy-of select="$party/ram:SpecifiedLegalOrganization/ram:ID/@*"/>
          <xsl:value-of select="$party/ram:SpecifiedLegalOrganization/ram:ID"/>
        </ram:ID>
      </ram:SpecifiedLegalOrganization>
    </xsl:if>

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
