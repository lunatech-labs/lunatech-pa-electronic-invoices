<?xml version="1.0" encoding="UTF-8"?>
<!--
  UBL 2.1 → Flux 1 Base UBL (PPF)
  
  Transformation d'une facture UBL complète vers le format allégé Flux 1 Base
  conforme au XSD F1_BASE_UBL_2.1 des spécifications externes PPF v3.1.
  
  Principales transformations :
  - BT-24 (CustomizationID) → urn.cpro.gouv.fr:1p0:einvoicingextract#Base
  - Suppression de toutes les lignes de facture (InvoiceLine)
  - Allègement des parties : PartyIdentification, PostalAddress (allégée),
    PartyTaxScheme, PartyLegalEntity (CompanyID seulement)
  - Suppression des moyens de paiement, remises/majorations, pièces jointes
  - LegalMonetaryTotal réduit à TaxExclusiveAmount
  
  Gère Invoice et CreditNote.
  Conforme aux règles BR-FR-MAP-15 et BR-FR-MAP-24.
-->
<xsl:stylesheet version="2.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:inv="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
  xmlns:cn="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2"
  xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
  xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2"
  exclude-result-prefixes="">

  <xsl:output method="xml" indent="yes" encoding="UTF-8"/>
  <xsl:strip-space elements="*"/>

  <!-- ============================================================
       Racine : Invoice
       ============================================================ -->
  <xsl:template match="inv:Invoice">
    <Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
             xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
             xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
      <xsl:call-template name="F1BaseInvoiceContent"/>
    </Invoice>
  </xsl:template>

  <!-- ============================================================
       Racine : CreditNote
       ============================================================ -->
  <xsl:template match="cn:CreditNote">
    <CreditNote xmlns="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2"
                xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
                xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
      <xsl:call-template name="F1BaseInvoiceContent"/>
    </CreditNote>
  </xsl:template>

  <!-- ============================================================
       Contenu commun Invoice/CreditNote → Flux 1 Base
       ============================================================ -->
  <xsl:template name="F1BaseInvoiceContent">
    <!-- BT-24 : CustomizationID → Base -->
    <cbc:CustomizationID>urn.cpro.gouv.fr:1p0:einvoicingextract#Base</cbc:CustomizationID>

    <!-- BT-23 : ProfileID -->
    <xsl:if test="cbc:ProfileID">
      <cbc:ProfileID><xsl:value-of select="cbc:ProfileID"/></cbc:ProfileID>
    </xsl:if>

    <!-- BT-1 : ID -->
    <cbc:ID><xsl:value-of select="cbc:ID"/></cbc:ID>

    <!-- BT-2 : IssueDate -->
    <cbc:IssueDate><xsl:value-of select="cbc:IssueDate"/></cbc:IssueDate>

    <!-- BT-9 : DueDate -->
    <xsl:if test="cbc:DueDate">
      <cbc:DueDate><xsl:value-of select="cbc:DueDate"/></cbc:DueDate>
    </xsl:if>

    <!-- BT-3 : InvoiceTypeCode / CreditNoteTypeCode -->
    <xsl:if test="cbc:InvoiceTypeCode">
      <cbc:InvoiceTypeCode><xsl:value-of select="cbc:InvoiceTypeCode"/></cbc:InvoiceTypeCode>
    </xsl:if>
    <xsl:if test="cbc:CreditNoteTypeCode">
      <cbc:CreditNoteTypeCode xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
        <xsl:value-of select="cbc:CreditNoteTypeCode"/>
      </cbc:CreditNoteTypeCode>
    </xsl:if>

    <!-- BT-22 : Note -->
    <xsl:for-each select="cbc:Note">
      <cbc:Note><xsl:value-of select="."/></cbc:Note>
    </xsl:for-each>

    <!-- BT-5 : DocumentCurrencyCode -->
    <cbc:DocumentCurrencyCode><xsl:value-of select="cbc:DocumentCurrencyCode"/></cbc:DocumentCurrencyCode>

    <!-- BG-14 : InvoicePeriod -->
    <xsl:if test="cac:InvoicePeriod">
      <cac:InvoicePeriod>
        <xsl:if test="cac:InvoicePeriod/cbc:StartDate">
          <cbc:StartDate><xsl:value-of select="cac:InvoicePeriod/cbc:StartDate"/></cbc:StartDate>
        </xsl:if>
        <xsl:if test="cac:InvoicePeriod/cbc:EndDate">
          <cbc:EndDate><xsl:value-of select="cac:InvoicePeriod/cbc:EndDate"/></cbc:EndDate>
        </xsl:if>
        <xsl:if test="cac:InvoicePeriod/cbc:DescriptionCode">
          <cbc:DescriptionCode><xsl:value-of select="cac:InvoicePeriod/cbc:DescriptionCode"/></cbc:DescriptionCode>
        </xsl:if>
      </cac:InvoicePeriod>
    </xsl:if>

    <!-- BG-3 : BillingReference (InvoiceDocumentReference/ID seulement) -->
    <xsl:for-each select="cac:BillingReference">
      <cac:BillingReference>
        <cac:InvoiceDocumentReference>
          <cbc:ID><xsl:value-of select="cac:InvoiceDocumentReference/cbc:ID"/></cbc:ID>
        </cac:InvoiceDocumentReference>
      </cac:BillingReference>
    </xsl:for-each>

    <!-- BG-4 : AccountingSupplierParty -->
    <xsl:if test="cac:AccountingSupplierParty">
      <cac:AccountingSupplierParty>
        <cac:Party>
          <xsl:call-template name="F1BaseParty">
            <xsl:with-param name="party" select="cac:AccountingSupplierParty/cac:Party"/>
          </xsl:call-template>
        </cac:Party>
      </cac:AccountingSupplierParty>
    </xsl:if>

    <!-- BG-7 : AccountingCustomerParty -->
    <xsl:if test="cac:AccountingCustomerParty">
      <cac:AccountingCustomerParty>
        <cac:Party>
          <xsl:call-template name="F1BaseParty">
            <xsl:with-param name="party" select="cac:AccountingCustomerParty/cac:Party"/>
          </xsl:call-template>
        </cac:Party>
      </cac:AccountingCustomerParty>
    </xsl:if>

    <!-- BG-11 : TaxRepresentativeParty -->
    <xsl:if test="cac:TaxRepresentativeParty">
      <cac:TaxRepresentativeParty>
        <xsl:call-template name="F1BaseParty">
          <xsl:with-param name="party" select="cac:TaxRepresentativeParty"/>
        </xsl:call-template>
      </cac:TaxRepresentativeParty>
    </xsl:if>

    <!-- BG-13 : Delivery (ActualDeliveryDate seulement) -->
    <xsl:if test="cac:Delivery">
      <cac:Delivery>
        <xsl:if test="cac:Delivery/cbc:ActualDeliveryDate">
          <cbc:ActualDeliveryDate><xsl:value-of select="cac:Delivery/cbc:ActualDeliveryDate"/></cbc:ActualDeliveryDate>
        </xsl:if>
      </cac:Delivery>
    </xsl:if>

    <!-- BG-22 : TaxTotal -->
    <xsl:for-each select="cac:TaxTotal">
      <cac:TaxTotal>
        <cbc:TaxAmount>
          <xsl:copy-of select="cbc:TaxAmount/@*"/>
          <xsl:value-of select="cbc:TaxAmount"/>
        </cbc:TaxAmount>
        <xsl:for-each select="cac:TaxSubtotal">
          <cac:TaxSubtotal>
            <xsl:if test="cbc:TaxableAmount">
              <cbc:TaxableAmount>
                <xsl:copy-of select="cbc:TaxableAmount/@*"/>
                <xsl:value-of select="cbc:TaxableAmount"/>
              </cbc:TaxableAmount>
            </xsl:if>
            <cbc:TaxAmount>
              <xsl:copy-of select="cbc:TaxAmount/@*"/>
              <xsl:value-of select="cbc:TaxAmount"/>
            </cbc:TaxAmount>
            <xsl:if test="cac:TaxCategory">
              <cac:TaxCategory>
                <cbc:ID><xsl:value-of select="cac:TaxCategory/cbc:ID"/></cbc:ID>
                <xsl:if test="cac:TaxCategory/cbc:Percent">
                  <cbc:Percent><xsl:value-of select="cac:TaxCategory/cbc:Percent"/></cbc:Percent>
                </xsl:if>
                <xsl:if test="cac:TaxCategory/cbc:TaxExemptionReasonCode">
                  <cbc:TaxExemptionReasonCode><xsl:value-of select="cac:TaxCategory/cbc:TaxExemptionReasonCode"/></cbc:TaxExemptionReasonCode>
                </xsl:if>
                <xsl:if test="cac:TaxCategory/cbc:TaxExemptionReason">
                  <cbc:TaxExemptionReason><xsl:value-of select="cac:TaxCategory/cbc:TaxExemptionReason"/></cbc:TaxExemptionReason>
                </xsl:if>
                <cac:TaxScheme>
                  <cbc:ID><xsl:value-of select="cac:TaxCategory/cac:TaxScheme/cbc:ID"/></cbc:ID>
                </cac:TaxScheme>
              </cac:TaxCategory>
            </xsl:if>
          </cac:TaxSubtotal>
        </xsl:for-each>
      </cac:TaxTotal>
    </xsl:for-each>

    <!-- BG-22 : LegalMonetaryTotal (TaxExclusiveAmount seulement) -->
    <xsl:if test="cac:LegalMonetaryTotal">
      <cac:LegalMonetaryTotal>
        <xsl:if test="cac:LegalMonetaryTotal/cbc:TaxExclusiveAmount">
          <cbc:TaxExclusiveAmount>
            <xsl:copy-of select="cac:LegalMonetaryTotal/cbc:TaxExclusiveAmount/@*"/>
            <xsl:value-of select="cac:LegalMonetaryTotal/cbc:TaxExclusiveAmount"/>
          </cbc:TaxExclusiveAmount>
        </xsl:if>
      </cac:LegalMonetaryTotal>
    </xsl:if>

    <!-- Pas de InvoiceLine / CreditNoteLine dans Flux 1 Base (BR-FR-MAP-24) -->
  </xsl:template>

  <!-- ============================================================
       Template nommé : Party allégé pour Flux 1 Base UBL
       Garde : PartyIdentification, PostalAddress (allégée),
               PartyTaxScheme, PartyLegalEntity (CompanyID)
       ============================================================ -->
  <xsl:template name="F1BaseParty">
    <xsl:param name="party"/>

    <!-- PartyIdentification -->
    <xsl:for-each select="$party/cac:PartyIdentification">
      <cac:PartyIdentification>
        <cbc:ID>
          <xsl:copy-of select="cbc:ID/@*"/>
          <xsl:value-of select="cbc:ID"/>
        </cbc:ID>
      </cac:PartyIdentification>
    </xsl:for-each>

    <!-- PostalAddress (allégée) -->
    <xsl:if test="$party/cac:PostalAddress">
      <cac:PostalAddress>
        <xsl:if test="$party/cac:PostalAddress/cbc:StreetName">
          <cbc:StreetName><xsl:value-of select="$party/cac:PostalAddress/cbc:StreetName"/></cbc:StreetName>
        </xsl:if>
        <xsl:if test="$party/cac:PostalAddress/cbc:AdditionalStreetName">
          <cbc:AdditionalStreetName><xsl:value-of select="$party/cac:PostalAddress/cbc:AdditionalStreetName"/></cbc:AdditionalStreetName>
        </xsl:if>
        <xsl:if test="$party/cac:PostalAddress/cbc:CityName">
          <cbc:CityName><xsl:value-of select="$party/cac:PostalAddress/cbc:CityName"/></cbc:CityName>
        </xsl:if>
        <xsl:if test="$party/cac:PostalAddress/cbc:PostalZone">
          <cbc:PostalZone><xsl:value-of select="$party/cac:PostalAddress/cbc:PostalZone"/></cbc:PostalZone>
        </xsl:if>
        <xsl:if test="$party/cac:PostalAddress/cbc:CountrySubentity">
          <cbc:CountrySubentity><xsl:value-of select="$party/cac:PostalAddress/cbc:CountrySubentity"/></cbc:CountrySubentity>
        </xsl:if>
        <xsl:if test="$party/cac:PostalAddress/cac:AddressLine">
          <cac:AddressLine>
            <cbc:Line><xsl:value-of select="$party/cac:PostalAddress/cac:AddressLine/cbc:Line"/></cbc:Line>
          </cac:AddressLine>
        </xsl:if>
        <xsl:if test="$party/cac:PostalAddress/cac:Country">
          <cac:Country>
            <cbc:IdentificationCode><xsl:value-of select="$party/cac:PostalAddress/cac:Country/cbc:IdentificationCode"/></cbc:IdentificationCode>
          </cac:Country>
        </xsl:if>
      </cac:PostalAddress>
    </xsl:if>

    <!-- PartyTaxScheme -->
    <xsl:for-each select="$party/cac:PartyTaxScheme">
      <cac:PartyTaxScheme>
        <xsl:if test="cbc:CompanyID">
          <cbc:CompanyID><xsl:value-of select="cbc:CompanyID"/></cbc:CompanyID>
        </xsl:if>
        <cac:TaxScheme>
          <cbc:ID><xsl:value-of select="cac:TaxScheme/cbc:ID"/></cbc:ID>
        </cac:TaxScheme>
      </cac:PartyTaxScheme>
    </xsl:for-each>

    <!-- PartyLegalEntity (CompanyID seulement) -->
    <xsl:for-each select="$party/cac:PartyLegalEntity">
      <cac:PartyLegalEntity>
        <xsl:if test="cbc:CompanyID">
          <cbc:CompanyID>
            <xsl:copy-of select="cbc:CompanyID/@*"/>
            <xsl:value-of select="cbc:CompanyID"/>
          </cbc:CompanyID>
        </xsl:if>
      </cac:PartyLegalEntity>
    </xsl:for-each>
  </xsl:template>

</xsl:stylesheet>
