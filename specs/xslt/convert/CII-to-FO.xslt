<?xml version="1.0" encoding="UTF-8"?>
<!--
  CII CrossIndustryInvoice → XSL-FO pour génération PDF/A-3 via Apache FOP
  Produit un rendu lisible de la facture avec les données structurées.
  Le XML CII est embarqué séparément comme pièce jointe par FOP.
-->
<xsl:stylesheet version="2.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:fo="http://www.w3.org/1999/XSL/Format"
    xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
    xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
    xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100"
    xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"
    exclude-result-prefixes="rsm ram udt qdt">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:strip-space elements="*"/>

  <!-- Helper: CII date (YYYYMMDD) → display date (DD/MM/YYYY) -->
  <xsl:template name="display-date">
    <xsl:param name="date"/>
    <xsl:if test="string-length($date) = 8">
      <xsl:value-of select="concat(substring($date,7,2),'/',substring($date,5,2),'/',substring($date,1,4))"/>
    </xsl:if>
  </xsl:template>

  <!-- ============================================================ -->
  <!-- Root                                                         -->
  <!-- ============================================================ -->
  <xsl:template match="/rsm:CrossIndustryInvoice">
    <xsl:variable name="agreement" select="rsm:SupplyChainTradeTransaction/ram:ApplicableHeaderTradeAgreement"/>
    <xsl:variable name="settlement" select="rsm:SupplyChainTradeTransaction/ram:ApplicableHeaderTradeSettlement"/>
    <xsl:variable name="summation" select="$settlement/ram:SpecifiedTradeSettlementHeaderMonetarySummation"/>

    <fo:root>
      <fo:layout-master-set>
        <fo:simple-page-master master-name="A4"
            page-height="297mm" page-width="210mm"
            margin-top="15mm" margin-bottom="15mm"
            margin-left="20mm" margin-right="20mm">
          <fo:region-body margin-top="10mm" margin-bottom="15mm"/>
          <fo:region-after extent="12mm"/>
        </fo:simple-page-master>
      </fo:layout-master-set>

      <fo:page-sequence master-reference="A4" font-family="Helvetica, Arial, sans-serif" font-size="9pt">
        <!-- Footer -->
        <fo:static-content flow-name="xsl-region-after">
          <fo:block text-align="center" font-size="7pt" color="#888888" border-top="0.5pt solid #cccccc" padding-top="3mm">
            <xsl:text>Factur-X EN16931 — Document généré automatiquement par pdp-facture</xsl:text>
          </fo:block>
        </fo:static-content>

        <fo:flow flow-name="xsl-region-body">
          <!-- ===== Header: Invoice type + number ===== -->
          <fo:block font-size="16pt" font-weight="bold" color="#1a1a1a" space-after="3mm">
            <xsl:choose>
              <xsl:when test="rsm:ExchangedDocument/ram:TypeCode = '381'">AVOIR</xsl:when>
              <xsl:when test="rsm:ExchangedDocument/ram:TypeCode = '384'">FACTURE RECTIFICATIVE</xsl:when>
              <xsl:when test="rsm:ExchangedDocument/ram:TypeCode = '389'">AUTOFACTURE</xsl:when>
              <xsl:otherwise>FACTURE</xsl:otherwise>
            </xsl:choose>
          </fo:block>

          <fo:table width="100%" space-after="6mm">
            <fo:table-column column-width="50%"/>
            <fo:table-column column-width="50%"/>
            <fo:table-body>
              <fo:table-row>
                <fo:table-cell>
                  <fo:block font-size="11pt" font-weight="bold">
                    <xsl:text>N° </xsl:text>
                    <xsl:value-of select="rsm:ExchangedDocument/ram:ID"/>
                  </fo:block>
                </fo:table-cell>
                <fo:table-cell text-align="right">
                  <fo:block>
                    <xsl:text>Date : </xsl:text>
                    <xsl:call-template name="display-date">
                      <xsl:with-param name="date" select="rsm:ExchangedDocument/ram:IssueDateTime/udt:DateTimeString"/>
                    </xsl:call-template>
                  </fo:block>
                </fo:table-cell>
              </fo:table-row>
            </fo:table-body>
          </fo:table>

          <!-- ===== Seller / Buyer ===== -->
          <fo:table width="100%" space-after="6mm">
            <fo:table-column column-width="48%"/>
            <fo:table-column column-width="4%"/>
            <fo:table-column column-width="48%"/>
            <fo:table-body>
              <fo:table-row>
                <fo:table-cell border="0.5pt solid #cccccc" padding="3mm" background-color="#f8f8f8">
                  <fo:block font-weight="bold" space-after="2mm" color="#333333">VENDEUR</fo:block>
                  <fo:block font-weight="bold"><xsl:value-of select="$agreement/ram:SellerTradeParty/ram:Name"/></fo:block>
                  <xsl:if test="$agreement/ram:SellerTradeParty/ram:PostalTradeAddress">
                    <fo:block><xsl:value-of select="$agreement/ram:SellerTradeParty/ram:PostalTradeAddress/ram:LineOne"/></fo:block>
                    <fo:block>
                      <xsl:value-of select="$agreement/ram:SellerTradeParty/ram:PostalTradeAddress/ram:PostcodeCode"/>
                      <xsl:text> </xsl:text>
                      <xsl:value-of select="$agreement/ram:SellerTradeParty/ram:PostalTradeAddress/ram:CityName"/>
                    </fo:block>
                  </xsl:if>
                  <xsl:if test="$agreement/ram:SellerTradeParty/ram:SpecifiedTaxRegistration/ram:ID">
                    <fo:block space-before="1mm" font-size="8pt" color="#666666">
                      <xsl:text>TVA : </xsl:text>
                      <xsl:value-of select="$agreement/ram:SellerTradeParty/ram:SpecifiedTaxRegistration/ram:ID"/>
                    </fo:block>
                  </xsl:if>
                  <xsl:if test="$agreement/ram:SellerTradeParty/ram:SpecifiedLegalOrganization/ram:ID">
                    <fo:block font-size="8pt" color="#666666">
                      <xsl:text>SIRET : </xsl:text>
                      <xsl:value-of select="$agreement/ram:SellerTradeParty/ram:SpecifiedLegalOrganization/ram:ID"/>
                    </fo:block>
                  </xsl:if>
                </fo:table-cell>
                <fo:table-cell><fo:block/></fo:table-cell>
                <fo:table-cell border="0.5pt solid #cccccc" padding="3mm">
                  <fo:block font-weight="bold" space-after="2mm" color="#333333">ACHETEUR</fo:block>
                  <fo:block font-weight="bold"><xsl:value-of select="$agreement/ram:BuyerTradeParty/ram:Name"/></fo:block>
                  <xsl:if test="$agreement/ram:BuyerTradeParty/ram:PostalTradeAddress">
                    <fo:block><xsl:value-of select="$agreement/ram:BuyerTradeParty/ram:PostalTradeAddress/ram:LineOne"/></fo:block>
                    <fo:block>
                      <xsl:value-of select="$agreement/ram:BuyerTradeParty/ram:PostalTradeAddress/ram:PostcodeCode"/>
                      <xsl:text> </xsl:text>
                      <xsl:value-of select="$agreement/ram:BuyerTradeParty/ram:PostalTradeAddress/ram:CityName"/>
                    </fo:block>
                  </xsl:if>
                  <xsl:if test="$agreement/ram:BuyerTradeParty/ram:SpecifiedTaxRegistration/ram:ID">
                    <fo:block space-before="1mm" font-size="8pt" color="#666666">
                      <xsl:text>TVA : </xsl:text>
                      <xsl:value-of select="$agreement/ram:BuyerTradeParty/ram:SpecifiedTaxRegistration/ram:ID"/>
                    </fo:block>
                  </xsl:if>
                </fo:table-cell>
              </fo:table-row>
            </fo:table-body>
          </fo:table>

          <!-- ===== References ===== -->
          <xsl:if test="$agreement/ram:BuyerReference or $agreement/ram:BuyerOrderReferencedDocument">
            <fo:block space-after="4mm" font-size="8pt" color="#555555">
              <xsl:if test="$agreement/ram:BuyerReference">
                <xsl:text>Réf. acheteur : </xsl:text>
                <xsl:value-of select="$agreement/ram:BuyerReference"/>
                <xsl:text>  </xsl:text>
              </xsl:if>
              <xsl:if test="$agreement/ram:BuyerOrderReferencedDocument/ram:IssuerAssignedID">
                <xsl:text>Bon de commande : </xsl:text>
                <xsl:value-of select="$agreement/ram:BuyerOrderReferencedDocument/ram:IssuerAssignedID"/>
              </xsl:if>
            </fo:block>
          </xsl:if>

          <!-- ===== Line items table ===== -->
          <fo:table width="100%" border-collapse="collapse" space-after="4mm">
            <fo:table-column column-width="6%"/>
            <fo:table-column column-width="34%"/>
            <fo:table-column column-width="12%"/>
            <fo:table-column column-width="12%"/>
            <fo:table-column column-width="12%"/>
            <fo:table-column column-width="12%"/>
            <fo:table-column column-width="12%"/>
            <fo:table-header>
              <fo:table-row background-color="#2c3e50" color="white" font-weight="bold">
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50"><fo:block>#</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50"><fo:block>Désignation</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50" text-align="right"><fo:block>Qté</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50" text-align="right"><fo:block>Unité</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50" text-align="right"><fo:block>PU HT</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50" text-align="right"><fo:block>TVA</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" border="0.5pt solid #2c3e50" text-align="right"><fo:block>Total HT</fo:block></fo:table-cell>
              </fo:table-row>
            </fo:table-header>
            <fo:table-body>
              <xsl:for-each select="rsm:SupplyChainTradeTransaction/ram:IncludedSupplyChainTradeLineItem">
                <fo:table-row>
                  <xsl:if test="position() mod 2 = 0">
                    <xsl:attribute name="background-color">#f5f5f5</xsl:attribute>
                  </xsl:if>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd">
                    <fo:block><xsl:value-of select="ram:AssociatedDocumentLineDocument/ram:LineID"/></fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd">
                    <fo:block font-weight="bold"><xsl:value-of select="ram:SpecifiedTradeProduct/ram:Name"/></fo:block>
                    <xsl:if test="ram:SpecifiedTradeProduct/ram:Description">
                      <fo:block font-size="7pt" color="#777777"><xsl:value-of select="ram:SpecifiedTradeProduct/ram:Description"/></fo:block>
                    </xsl:if>
                  </fo:table-cell>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block><xsl:value-of select="ram:SpecifiedLineTradeDelivery/ram:BilledQuantity"/></fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block><xsl:value-of select="ram:SpecifiedLineTradeDelivery/ram:BilledQuantity/@unitCode"/></fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block>
                      <xsl:value-of select="format-number(ram:SpecifiedLineTradeAgreement/ram:NetPriceProductTradePrice/ram:ChargeAmount, '#,##0.00')"/>
                      <xsl:text> €</xsl:text>
                    </fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block>
                      <xsl:value-of select="ram:SpecifiedLineTradeSettlement/ram:ApplicableTradeTax/ram:RateApplicablePercent"/>
                      <xsl:text> %</xsl:text>
                    </fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="2mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block font-weight="bold">
                      <xsl:value-of select="format-number(ram:SpecifiedLineTradeSettlement/ram:SpecifiedTradeSettlementLineMonetarySummation/ram:LineTotalAmount, '#,##0.00')"/>
                      <xsl:text> €</xsl:text>
                    </fo:block>
                  </fo:table-cell>
                </fo:table-row>
              </xsl:for-each>
            </fo:table-body>
          </fo:table>

          <!-- ===== VAT breakdown ===== -->
          <fo:table width="60%" margin-left="40%" space-after="4mm" border-collapse="collapse">
            <fo:table-column column-width="25%"/>
            <fo:table-column column-width="25%"/>
            <fo:table-column column-width="25%"/>
            <fo:table-column column-width="25%"/>
            <fo:table-header>
              <fo:table-row background-color="#ecf0f1" font-weight="bold" font-size="8pt">
                <fo:table-cell padding="1.5mm" border="0.5pt solid #cccccc"><fo:block>Code TVA</fo:block></fo:table-cell>
                <fo:table-cell padding="1.5mm" border="0.5pt solid #cccccc" text-align="right"><fo:block>Base HT</fo:block></fo:table-cell>
                <fo:table-cell padding="1.5mm" border="0.5pt solid #cccccc" text-align="right"><fo:block>Taux</fo:block></fo:table-cell>
                <fo:table-cell padding="1.5mm" border="0.5pt solid #cccccc" text-align="right"><fo:block>Montant TVA</fo:block></fo:table-cell>
              </fo:table-row>
            </fo:table-header>
            <fo:table-body>
              <xsl:for-each select="$settlement/ram:ApplicableTradeTax">
                <fo:table-row font-size="8pt">
                  <fo:table-cell padding="1.5mm" border="0.5pt solid #dddddd"><fo:block><xsl:value-of select="ram:CategoryCode"/></fo:block></fo:table-cell>
                  <fo:table-cell padding="1.5mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block><xsl:value-of select="format-number(ram:BasisAmount, '#,##0.00')"/> €</fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="1.5mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block><xsl:value-of select="ram:RateApplicablePercent"/> %</fo:block>
                  </fo:table-cell>
                  <fo:table-cell padding="1.5mm" border="0.5pt solid #dddddd" text-align="right">
                    <fo:block><xsl:value-of select="format-number(ram:CalculatedAmount, '#,##0.00')"/> €</fo:block>
                  </fo:table-cell>
                </fo:table-row>
              </xsl:for-each>
            </fo:table-body>
          </fo:table>

          <!-- ===== Totals ===== -->
          <fo:table width="40%" margin-left="60%" space-after="6mm">
            <fo:table-column column-width="50%"/>
            <fo:table-column column-width="50%"/>
            <fo:table-body>
              <fo:table-row>
                <fo:table-cell padding="1.5mm"><fo:block>Total HT</fo:block></fo:table-cell>
                <fo:table-cell padding="1.5mm" text-align="right">
                  <fo:block><xsl:value-of select="format-number($summation/ram:TaxBasisTotalAmount, '#,##0.00')"/> €</fo:block>
                </fo:table-cell>
              </fo:table-row>
              <fo:table-row>
                <fo:table-cell padding="1.5mm"><fo:block>Total TVA</fo:block></fo:table-cell>
                <fo:table-cell padding="1.5mm" text-align="right">
                  <fo:block><xsl:value-of select="format-number($summation/ram:TaxTotalAmount, '#,##0.00')"/> €</fo:block>
                </fo:table-cell>
              </fo:table-row>
              <fo:table-row background-color="#2c3e50" color="white" font-weight="bold" font-size="11pt">
                <fo:table-cell padding="2mm"><fo:block>Total TTC</fo:block></fo:table-cell>
                <fo:table-cell padding="2mm" text-align="right">
                  <fo:block><xsl:value-of select="format-number($summation/ram:GrandTotalAmount, '#,##0.00')"/> €</fo:block>
                </fo:table-cell>
              </fo:table-row>
              <xsl:if test="$summation/ram:TotalPrepaidAmount">
                <fo:table-row>
                  <fo:table-cell padding="1.5mm"><fo:block>Déjà payé</fo:block></fo:table-cell>
                  <fo:table-cell padding="1.5mm" text-align="right">
                    <fo:block><xsl:value-of select="format-number($summation/ram:TotalPrepaidAmount, '#,##0.00')"/> €</fo:block>
                  </fo:table-cell>
                </fo:table-row>
              </xsl:if>
              <xsl:if test="$summation/ram:DuePayableAmount != $summation/ram:GrandTotalAmount">
                <fo:table-row font-weight="bold">
                  <fo:table-cell padding="1.5mm"><fo:block>Net à payer</fo:block></fo:table-cell>
                  <fo:table-cell padding="1.5mm" text-align="right">
                    <fo:block><xsl:value-of select="format-number($summation/ram:DuePayableAmount, '#,##0.00')"/> €</fo:block>
                  </fo:table-cell>
                </fo:table-row>
              </xsl:if>
            </fo:table-body>
          </fo:table>

          <!-- ===== Payment info ===== -->
          <xsl:if test="$settlement/ram:SpecifiedTradeSettlementPaymentMeans or $settlement/ram:SpecifiedTradePaymentTerms">
            <fo:block border="0.5pt solid #cccccc" padding="3mm" space-after="4mm" background-color="#fafafa">
              <fo:block font-weight="bold" space-after="2mm" color="#333333">INFORMATIONS DE PAIEMENT</fo:block>
              <xsl:if test="$settlement/ram:SpecifiedTradePaymentTerms/ram:Description">
                <fo:block><xsl:value-of select="$settlement/ram:SpecifiedTradePaymentTerms/ram:Description"/></fo:block>
              </xsl:if>
              <xsl:if test="$settlement/ram:SpecifiedTradePaymentTerms/ram:DueDateDateTime/udt:DateTimeString">
                <fo:block>
                  <xsl:text>Échéance : </xsl:text>
                  <xsl:call-template name="display-date">
                    <xsl:with-param name="date" select="$settlement/ram:SpecifiedTradePaymentTerms/ram:DueDateDateTime/udt:DateTimeString"/>
                  </xsl:call-template>
                </fo:block>
              </xsl:if>
              <xsl:if test="$settlement/ram:SpecifiedTradeSettlementPaymentMeans/ram:PayeePartyCreditorFinancialAccount/ram:IBANID">
                <fo:block>
                  <xsl:text>IBAN : </xsl:text>
                  <xsl:value-of select="$settlement/ram:SpecifiedTradeSettlementPaymentMeans/ram:PayeePartyCreditorFinancialAccount/ram:IBANID"/>
                </fo:block>
              </xsl:if>
              <xsl:if test="$settlement/ram:SpecifiedTradeSettlementPaymentMeans/ram:PayeeSpecifiedCreditorFinancialInstitution/ram:BICID">
                <fo:block>
                  <xsl:text>BIC : </xsl:text>
                  <xsl:value-of select="$settlement/ram:SpecifiedTradeSettlementPaymentMeans/ram:PayeeSpecifiedCreditorFinancialInstitution/ram:BICID"/>
                </fo:block>
              </xsl:if>
            </fo:block>
          </xsl:if>

          <!-- ===== Notes ===== -->
          <xsl:if test="rsm:ExchangedDocument/ram:IncludedNote">
            <fo:block font-size="8pt" color="#666666" space-after="3mm">
              <xsl:for-each select="rsm:ExchangedDocument/ram:IncludedNote">
                <fo:block><xsl:value-of select="ram:Content"/></fo:block>
              </xsl:for-each>
            </fo:block>
          </xsl:if>

          <!-- ===== Preceding invoice reference ===== -->
          <xsl:if test="$settlement/ram:InvoiceReferencedDocument">
            <fo:block font-size="8pt" color="#666666">
              <xsl:text>Réf. facture d'origine : </xsl:text>
              <xsl:value-of select="$settlement/ram:InvoiceReferencedDocument/ram:IssuerAssignedID"/>
              <xsl:if test="$settlement/ram:InvoiceReferencedDocument/ram:FormattedIssueDateTime/qdt:DateTimeString">
                <xsl:text> du </xsl:text>
                <xsl:call-template name="display-date">
                  <xsl:with-param name="date" select="$settlement/ram:InvoiceReferencedDocument/ram:FormattedIssueDateTime/qdt:DateTimeString"/>
                </xsl:call-template>
              </xsl:if>
            </fo:block>
          </xsl:if>

        </fo:flow>
      </fo:page-sequence>
    </fo:root>
  </xsl:template>

</xsl:stylesheet>
