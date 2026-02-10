## Modifications apportées sur les XSD entre les versions publiées dans les spécifications externes v3.0 du PPF (18/12/2024) et les versions publiées en v3.1 des spécifications externes (30/10/2025)

### XSD E-invoicing

F1BASE_UBL-CommonAggregateComponents-2.1.xsd :

* mise en commentaire de l'élément /cac:BillingReference/cac:InvoiceDocumentReference/cbc:IssueDate (BT-26) qui appartient au profil FULL (trajectoire cible)
* activation des éléments /cac:Delivery/cac:DeliveryLocation/cac:Address (BG-15) et /cac:Delivery/cac:DeliveryLocation/cac:Address/cac:Country/cbc:IdentificationCode (BT-80)

F1BASE_CrossIndustryInvoice_ReusableAggregateBusinessInformationEntity_100pD22B.xsd :

* activation des éléments /rsm:CrossIndustryInvoice/rsm:SupplyChainTradeTransaction/ram:ApplicableHeaderTradeDelivery/ram:ShipToTradeParty/ram:PostalTradeAddress (BG-15) et /rsm:CrossIndustryInvoice/rsm:SupplyChainTradeTransaction/ram:ApplicableHeaderTradeDelivery/ram:ShipToTradeParty/ram:PostalTradeAddress/ram:CountryID (BT-80)


### XSD E-Reporting

transaction.xsd :

* Modification de la cardinalité de TransactionsCount (TT-85) 1..1 --> 0..1

report.xsd :

* Suppression des éléments /ReportDocument/References (TG-2), /ReportDocument/References/ReportId (TT-5) et /ReportDocument/References/ReportId/@schemeId (TT-6)


### XSD Annuaire

Annuaire_Commun.xsd :

* Modification de la cardinalité DateFinEffective (DT-7-3-3) 1..1 --> 0..1
* Modification du type de la donnée attendue dans tous les blocs IdInstance string --> integer
* Ajout de la balise Diffusible dans les blocs UniteLegale (DT-3-6) et Etablissement (DT-4-14)
* Modification de la cardinalité DateFinImmatriculation (DT-6-10) 1..1 --> 0..1