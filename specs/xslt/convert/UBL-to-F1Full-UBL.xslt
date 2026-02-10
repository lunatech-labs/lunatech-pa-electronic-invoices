<?xml version="1.0" encoding="UTF-8"?>
<!--
  UBL 2.1 → Flux 1 Full UBL (PPF)
  
  Transformation d'une facture UBL complète vers le format Flux 1 Full
  conforme au XSD F1_FULL_UBL_2.1 des spécifications externes PPF v3.1.
  
  Le profil Full conserve les lignes, remises/majorations, livraison et prix,
  mais filtre les éléments non autorisés par le XSD :
  - Pas de UBLVersionID, BuyerReference, OrderReference, ContractDocumentReference
  - Pas de AdditionalDocumentReference, ProjectReference, PaymentMeans, PaymentTerms
  - Parties allégées (même structure que Base)
  
  BT-24 (CustomizationID) → urn.cpro.gouv.fr:1p0:einvoicingextract#Full
  
  Gère Invoice et CreditNote.
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
      <xsl:call-template name="F1FullHeader"/>
      <!-- InvoiceLine -->
      <xsl:for-each select="cac:InvoiceLine">
        <xsl:call-template name="F1FullInvoiceLine"/>
      </xsl:for-each>
    </Invoice>
  </xsl:template>

  <!-- ============================================================
       Racine : CreditNote
       ============================================================ -->
  <xsl:template match="cn:CreditNote">
    <CreditNote xmlns="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2"
                xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
                xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
      <xsl:call-template name="F1FullHeader"/>
      <!-- CreditNoteLine -->
      <xsl:for-each select="cac:CreditNoteLine">
        <xsl:call-template name="F1FullCreditNoteLine"/>
      </xsl:for-each>
    </CreditNote>
  </xsl:template>

  <!-- ============================================================
       Contenu commun Invoice/CreditNote → Flux 1 Full (header)
       XSD order: CustomizationID, ProfileID, ID, IssueDate, DueDate,
                  InvoiceTypeCode/CreditNoteTypeCode, Note*, DocumentCurrencyCode,
                  InvoicePeriod?, BillingReference*,
                  AccountingSupplierParty, AccountingCustomerParty,
                  TaxRepresentativeParty?, Delivery?,
                  AllowanceCharge*, TaxTotal{1,2}, LegalMonetaryTotal
       ============================================================ -->
  <xsl:template name="F1FullHeader">
    <!-- BT-24 : CustomizationID → Full -->
    <cbc:CustomizationID>urn.cpro.gouv.fr:1p0:einvoicingextract#Full</cbc:CustomizationID>

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

    <!-- BG-3 : BillingReference -->
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
          <xsl:call-template name="F1FullParty">
            <xsl:with-param name="party" select="cac:AccountingSupplierParty/cac:Party"/>
          </xsl:call-template>
        </cac:Party>
      </cac:AccountingSupplierParty>
    </xsl:if>

    <!-- BG-7 : AccountingCustomerParty -->
    <xsl:if test="cac:AccountingCustomerParty">
      <cac:AccountingCustomerParty>
        <cac:Party>
          <xsl:call-template name="F1FullParty">
            <xsl:with-param name="party" select="cac:AccountingCustomerParty/cac:Party"/>
          </xsl:call-template>
        </cac:Party>
      </cac:AccountingCustomerParty>
    </xsl:if>

    <!-- BG-11 : TaxRepresentativeParty -->
    <xsl:if test="cac:TaxRepresentativeParty">
      <cac:TaxRepresentativeParty>
        <xsl:call-template name="F1FullParty">
          <xsl:with-param name="party" select="cac:TaxRepresentativeParty"/>
        </xsl:call-template>
      </cac:TaxRepresentativeParty>
    </xsl:if>

    <!-- BG-13 : Delivery -->
    <xsl:if test="cac:Delivery">
      <cac:Delivery>
        <xsl:if test="cac:Delivery/cbc:ActualDeliveryDate">
          <cbc:ActualDeliveryDate><xsl:value-of select="cac:Delivery/cbc:ActualDeliveryDate"/></cbc:ActualDeliveryDate>
        </xsl:if>
        <xsl:if test="cac:Delivery/cac:DeliveryLocation">
          <cac:DeliveryLocation>
            <xsl:if test="cac:Delivery/cac:DeliveryLocation/cac:Address">
              <cac:Address>
                <xsl:call-template name="F1FullAddress">
                  <xsl:with-param name="addr" select="cac:Delivery/cac:DeliveryLocation/cac:Address"/>
                </xsl:call-template>
              </cac:Address>
            </xsl:if>
          </cac:DeliveryLocation>
        </xsl:if>
      </cac:Delivery>
    </xsl:if>

    <!-- BG-20/21 : AllowanceCharge (document-level) -->
    <xsl:for-each select="cac:AllowanceCharge">
      <cac:AllowanceCharge>
        <cbc:ChargeIndicator><xsl:value-of select="cbc:ChargeIndicator"/></cbc:ChargeIndicator>
        <cbc:Amount>
          <xsl:copy-of select="cbc:Amount/@*"/>
          <xsl:value-of select="cbc:Amount"/>
        </cbc:Amount>
        <xsl:if test="cbc:BaseAmount">
          <cbc:BaseAmount>
            <xsl:copy-of select="cbc:BaseAmount/@*"/>
            <xsl:value-of select="cbc:BaseAmount"/>
          </cbc:BaseAmount>
        </xsl:if>
        <xsl:if test="cac:TaxCategory">
          <cac:TaxCategory>
            <cbc:ID><xsl:value-of select="cac:TaxCategory/cbc:ID"/></cbc:ID>
            <xsl:if test="cac:TaxCategory/cbc:Percent">
              <cbc:Percent><xsl:value-of select="cac:TaxCategory/cbc:Percent"/></cbc:Percent>
            </xsl:if>
            <cac:TaxScheme>
              <cbc:ID><xsl:value-of select="cac:TaxCategory/cac:TaxScheme/cbc:ID"/></cbc:ID>
            </cac:TaxScheme>
          </cac:TaxCategory>
        </xsl:if>
      </cac:AllowanceCharge>
    </xsl:for-each>

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

    <!-- LegalMonetaryTotal : TaxExclusiveAmount seulement -->
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
  </xsl:template>

  <!-- ============================================================
       InvoiceLine
       ============================================================ -->
  <xsl:template name="F1FullInvoiceLine">
    <cac:InvoiceLine>
      <xsl:if test="cbc:Note">
        <cbc:Note><xsl:value-of select="cbc:Note"/></cbc:Note>
      </xsl:if>
      <cbc:InvoicedQuantity>
        <xsl:copy-of select="cbc:InvoicedQuantity/@*"/>
        <xsl:value-of select="cbc:InvoicedQuantity"/>
      </cbc:InvoicedQuantity>
      <!-- InvoicePeriod (line-level) -->
      <xsl:if test="cac:InvoicePeriod">
        <cac:InvoicePeriod>
          <xsl:if test="cac:InvoicePeriod/cbc:StartDate">
            <cbc:StartDate><xsl:value-of select="cac:InvoicePeriod/cbc:StartDate"/></cbc:StartDate>
          </xsl:if>
          <xsl:if test="cac:InvoicePeriod/cbc:EndDate">
            <cbc:EndDate><xsl:value-of select="cac:InvoicePeriod/cbc:EndDate"/></cbc:EndDate>
          </xsl:if>
        </cac:InvoicePeriod>
      </xsl:if>
      <!-- Line AllowanceCharge -->
      <xsl:for-each select="cac:AllowanceCharge">
        <cac:AllowanceCharge>
          <cbc:ChargeIndicator><xsl:value-of select="cbc:ChargeIndicator"/></cbc:ChargeIndicator>
          <cbc:Amount>
            <xsl:copy-of select="cbc:Amount/@*"/>
            <xsl:value-of select="cbc:Amount"/>
          </cbc:Amount>
          <xsl:if test="cbc:BaseAmount">
            <cbc:BaseAmount>
              <xsl:copy-of select="cbc:BaseAmount/@*"/>
              <xsl:value-of select="cbc:BaseAmount"/>
            </cbc:BaseAmount>
          </xsl:if>
        </cac:AllowanceCharge>
      </xsl:for-each>
      <!-- Item (Name seulement) -->
      <cac:Item>
        <cbc:Name><xsl:value-of select="cac:Item/cbc:Name"/></cbc:Name>
      </cac:Item>
      <!-- Price -->
      <cac:Price>
        <cbc:PriceAmount>
          <xsl:copy-of select="cac:Price/cbc:PriceAmount/@*"/>
          <xsl:value-of select="cac:Price/cbc:PriceAmount"/>
        </cbc:PriceAmount>
        <xsl:if test="cac:Price/cac:AllowanceCharge">
          <cac:AllowanceCharge>
            <cbc:ChargeIndicator><xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:ChargeIndicator"/></cbc:ChargeIndicator>
            <cbc:Amount>
              <xsl:copy-of select="cac:Price/cac:AllowanceCharge/cbc:Amount/@*"/>
              <xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:Amount"/>
            </cbc:Amount>
            <xsl:if test="cac:Price/cac:AllowanceCharge/cbc:BaseAmount">
              <cbc:BaseAmount>
                <xsl:copy-of select="cac:Price/cac:AllowanceCharge/cbc:BaseAmount/@*"/>
                <xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:BaseAmount"/>
              </cbc:BaseAmount>
            </xsl:if>
          </cac:AllowanceCharge>
        </xsl:if>
      </cac:Price>
    </cac:InvoiceLine>
  </xsl:template>

  <!-- ============================================================
       CreditNoteLine
       ============================================================ -->
  <xsl:template name="F1FullCreditNoteLine">
    <cac:CreditNoteLine xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2">
      <xsl:if test="cbc:Note">
        <cbc:Note><xsl:value-of select="cbc:Note"/></cbc:Note>
      </xsl:if>
      <cbc:CreditedQuantity>
        <xsl:copy-of select="cbc:CreditedQuantity/@*"/>
        <xsl:value-of select="cbc:CreditedQuantity"/>
      </cbc:CreditedQuantity>
      <!-- InvoicePeriod (line-level) -->
      <xsl:if test="cac:InvoicePeriod">
        <cac:InvoicePeriod>
          <xsl:if test="cac:InvoicePeriod/cbc:StartDate">
            <cbc:StartDate><xsl:value-of select="cac:InvoicePeriod/cbc:StartDate"/></cbc:StartDate>
          </xsl:if>
          <xsl:if test="cac:InvoicePeriod/cbc:EndDate">
            <cbc:EndDate><xsl:value-of select="cac:InvoicePeriod/cbc:EndDate"/></cbc:EndDate>
          </xsl:if>
        </cac:InvoicePeriod>
      </xsl:if>
      <!-- Line AllowanceCharge -->
      <xsl:for-each select="cac:AllowanceCharge">
        <cac:AllowanceCharge>
          <cbc:ChargeIndicator><xsl:value-of select="cbc:ChargeIndicator"/></cbc:ChargeIndicator>
          <cbc:Amount>
            <xsl:copy-of select="cbc:Amount/@*"/>
            <xsl:value-of select="cbc:Amount"/>
          </cbc:Amount>
          <xsl:if test="cbc:BaseAmount">
            <cbc:BaseAmount>
              <xsl:copy-of select="cbc:BaseAmount/@*"/>
              <xsl:value-of select="cbc:BaseAmount"/>
            </cbc:BaseAmount>
          </xsl:if>
        </cac:AllowanceCharge>
      </xsl:for-each>
      <!-- Item (Name seulement) -->
      <cac:Item>
        <cbc:Name><xsl:value-of select="cac:Item/cbc:Name"/></cbc:Name>
      </cac:Item>
      <!-- Price -->
      <cac:Price>
        <cbc:PriceAmount>
          <xsl:copy-of select="cac:Price/cbc:PriceAmount/@*"/>
          <xsl:value-of select="cac:Price/cbc:PriceAmount"/>
        </cbc:PriceAmount>
        <xsl:if test="cac:Price/cac:AllowanceCharge">
          <cac:AllowanceCharge>
            <cbc:ChargeIndicator><xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:ChargeIndicator"/></cbc:ChargeIndicator>
            <cbc:Amount>
              <xsl:copy-of select="cac:Price/cac:AllowanceCharge/cbc:Amount/@*"/>
              <xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:Amount"/>
            </cbc:Amount>
            <xsl:if test="cac:Price/cac:AllowanceCharge/cbc:BaseAmount">
              <cbc:BaseAmount>
                <xsl:copy-of select="cac:Price/cac:AllowanceCharge/cbc:BaseAmount/@*"/>
                <xsl:value-of select="cac:Price/cac:AllowanceCharge/cbc:BaseAmount"/>
              </cbc:BaseAmount>
            </xsl:if>
          </cac:AllowanceCharge>
        </xsl:if>
      </cac:Price>
    </cac:CreditNoteLine>
  </xsl:template>

  <!-- ============================================================
       Template nommé : Party allégé pour Flux 1 Full UBL
       (même structure que Base)
       ============================================================ -->
  <xsl:template name="F1FullParty">
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

    <!-- PostalAddress -->
    <xsl:if test="$party/cac:PostalAddress">
      <cac:PostalAddress>
        <xsl:call-template name="F1FullAddress">
          <xsl:with-param name="addr" select="$party/cac:PostalAddress"/>
        </xsl:call-template>
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

    <!-- PartyLegalEntity -->
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

  <!-- ============================================================
       Template nommé : Address allégée
       ============================================================ -->
  <xsl:template name="F1FullAddress">
    <xsl:param name="addr"/>
    <xsl:if test="$addr/cbc:StreetName">
      <cbc:StreetName><xsl:value-of select="$addr/cbc:StreetName"/></cbc:StreetName>
    </xsl:if>
    <xsl:if test="$addr/cbc:AdditionalStreetName">
      <cbc:AdditionalStreetName><xsl:value-of select="$addr/cbc:AdditionalStreetName"/></cbc:AdditionalStreetName>
    </xsl:if>
    <xsl:if test="$addr/cbc:CityName">
      <cbc:CityName><xsl:value-of select="$addr/cbc:CityName"/></cbc:CityName>
    </xsl:if>
    <xsl:if test="$addr/cbc:PostalZone">
      <cbc:PostalZone><xsl:value-of select="$addr/cbc:PostalZone"/></cbc:PostalZone>
    </xsl:if>
    <xsl:if test="$addr/cbc:CountrySubentity">
      <cbc:CountrySubentity><xsl:value-of select="$addr/cbc:CountrySubentity"/></cbc:CountrySubentity>
    </xsl:if>
    <xsl:if test="$addr/cac:AddressLine">
      <cac:AddressLine>
        <cbc:Line><xsl:value-of select="$addr/cac:AddressLine/cbc:Line"/></cbc:Line>
      </cac:AddressLine>
    </xsl:if>
    <xsl:if test="$addr/cac:Country">
      <cac:Country>
        <cbc:IdentificationCode><xsl:value-of select="$addr/cac:Country/cbc:IdentificationCode"/></cbc:IdentificationCode>
      </cac:Country>
    </xsl:if>
  </xsl:template>

</xsl:stylesheet>
