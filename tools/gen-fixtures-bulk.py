#!/usr/bin/env python3
"""Génère un volume important de fixtures UBL + CII pour le tenant TechConseil
(SIREN 123456789) afin de pouvoir tester la pagination de l'UI et démontrer
les trois formats EN16931 supportés (UBL 2.1, UN/CEFACT CII D16B, Factur-X).

Distribution (configurable, défauts) :
- 60% UBL  (tests/fixtures/ubl/)
- 30% CII  (tests/fixtures/cii/)
- 10% Factur-X (tests/fixtures/facturx/) — le PDF/A-3 est généré via
  `pdp transform --to FacturX` à partir d'un UBL équivalent.

Quelques pourcents sont émises avec PJ, et ~3 % utilisent un SIREN partenaire
absent de l'annuaire (déclenche EMMET_INC en réception).

Lancement :
    python3 tools/gen-fixtures-bulk.py [--count 240]

Les fixtures Factur-X nécessitent que le binaire `pdp` soit construit
(`cargo build --release -p pdp-app`).
"""
from __future__ import annotations
import argparse
import os
import random
from dataclasses import dataclass

# Tenant principal de la démo
TECHCONSEIL_SIREN = "123456789"
TECHCONSEIL_NAME = "TechConseil SAS"
TECHCONSEIL_VAT = "FR12123456789"

# 24 partenaires PRÉSENTS dans l'annuaire — pour les fixtures valides
KNOWN_PARTNERS: list[tuple[str, str, str]] = [
    ("101631577", "Claire Design EURL", "FR10101631577"),
    ("107803423", "ServicesArchitecture SARL", "FR10107803423"),
    ("109009309", "Charlotte Solutions SCI", "FR10109009309"),
    ("111222333", "Menuiserie Artisanale Dupont EURL", "FR11111222333"),
    ("112848438", "Design Santé SAS", "FR11112848438"),
    ("116087782", "Études Bio SARL", "FR11116087782"),
    ("122140034", "Formation Bâtiment SAS", "FR12122140034"),
    ("125656495", "Développement Textile SA", "FR12125656495"),
    ("129352022", "DéveloppementTechnologie EURL", "FR12129352022"),
    ("136145529", "RechercheCommerce EURL", "FR13136145529"),
    ("139937345", "Industrie Culture SA", "FR13139937345"),
    ("144420364", "Sophie Création SAS", "FR14144420364"),
    ("146962925", "Thomas Solutions SA", "FR14146962925"),
    ("147610740", "SolutionsCommerce SAS", "FR14147610740"),
    ("148981446", "Production Luxe SA", "FR14148981446"),
    ("152031711", "Expertise Textile SAS", "FR15152031711"),
    ("222333444", "Plomberie Martin SARL", "FR22222333444"),
    ("415263748", "Conseil Expert SAS", "FR41415263748"),
    ("493827160", "Services Numériques SARL", "FR49493827160"),
    ("536987210", "Électronique Distribution SARL", "FR53536987210"),
    ("738492012", "Plomberie Durand SARL", "FR73738492012"),
    ("827364519", "AudioTech France SAS", "FR82827364519"),
    ("918273645", "Rénovation Habitat Plus SARL", "FR91918273645"),
    ("987654321", "IndustrieFrance SA", "FR98987654321"),
]

# Partenaires INCONNUS (pour 5% d'erreurs EMMET_INC en réception)
UNKNOWN_PARTNERS: list[tuple[str, str, str]] = [
    ("100200300", "Société Inconnue Alpha SAS", "FR10100200300"),
    ("400500600", "Bêta Corp EURL", "FR40400500600"),
    ("700800900", "Gamma Industries SARL", "FR70700800900"),
]

DESCRIPTIONS = [
    ("Prestation de conseil — sprint", "C62", 5, 950.0),
    ("Maintenance trimestrielle infrastructure", "C62", 1, 1250.0),
    ("Audit sécurité applicative", "C62", 3, 1450.0),
    ("Développement module sur mesure", "HUR", 24, 95.0),
    ("Formation équipe technique", "DAY", 2, 1850.0),
    ("Hébergement cloud — mensuel", "C62", 1, 690.0),
    ("Licences logicielles annuelles", "C62", 8, 320.0),
    ("Intervention support niveau 2", "HUR", 12, 110.0),
    ("Étude de faisabilité projet", "C62", 1, 5800.0),
    ("Fournitures bureau", "C62", 1, 240.0),
    ("Travaux de plomberie urgents", "C62", 1, 580.0),
    ("Aménagement salle de réunion", "C62", 1, 3450.0),
    ("Réparation matériel électronique", "C62", 1, 420.0),
    ("Conseil juridique", "HUR", 4, 250.0),
]

# Les PJ sont générées dynamiquement par scénario via `pdp transform --to PDF`
# (Typst — rendu PDF visuel de la facture avec les vraies données vendeur,
# acheteur, montants, lignes). Voir `generate_pj_pdf_base64()`.


@dataclass
class Scenario:
    invoice_number: str
    invoice_date: str
    due_date: str
    seller_name: str
    seller_siren: str
    seller_vat: str
    buyer_name: str
    buyer_siren: str
    buyer_vat: str
    description: str
    qty: int
    unit_code: str
    unit_price: float
    has_pdf: bool

    @property
    def seller_siret(self) -> str:
        return self.seller_siren + "00001"

    @property
    def buyer_siret(self) -> str:
        return self.buyer_siren + "00002"

    @property
    def ht(self) -> float:
        return round(self.qty * self.unit_price, 2)

    @property
    def tva(self) -> float:
        return round(self.ht * 0.20, 2)

    @property
    def ttc(self) -> float:
        return round(self.ht + self.tva, 2)


def daterange(start_year: int = 2025, start_month: int = 6, count: int = 240) -> list[str]:
    """Génère `count` dates étalées sur 12 mois pour étaler les issue_date
    et permettre de tester le filtre de plage de dates."""
    out = []
    for i in range(count):
        # ~20 par mois, 12 mois → 240
        month = (start_month - 1 + i // 20) % 12 + 1
        year = start_year + (start_month - 1 + i // 20) // 12
        day = (i % 28) + 1
        out.append(f"{year:04d}-{month:02d}-{day:02d}")
    return out


def build_scenarios(count: int) -> list[Scenario]:
    """Mix : moitié émises (TechConseil = vendeur), moitié reçues. ~10% avec PJ.
    ~3% avec un partenaire inconnu (en réception → EMMET_INC)."""
    rng = random.Random(42)  # déterministe pour reproductibilité
    dates = daterange(2025, 6, count)
    scenarios: list[Scenario] = []
    for i in range(count):
        is_emise = i % 2 == 0  # alterne pour équilibrer
        has_pdf = rng.random() < 0.10
        # 3% des reçues utilisent un partenaire absent de l'annuaire
        unknown = (not is_emise) and rng.random() < 0.03

        partner = (
            rng.choice(UNKNOWN_PARTNERS) if unknown else rng.choice(KNOWN_PARTNERS)
        )
        if is_emise:
            seller = (TECHCONSEIL_SIREN, TECHCONSEIL_NAME, TECHCONSEIL_VAT)
            buyer = partner
            prefix = "BULK-EMI"
        else:
            seller = partner
            buyer = (TECHCONSEIL_SIREN, TECHCONSEIL_NAME, TECHCONSEIL_VAT)
            prefix = "BULK-REC"

        desc = rng.choice(DESCRIPTIONS)
        invoice_date = dates[i]
        # due = +30j approx (on garde même mois pour simplicité)
        y, m, d = invoice_date.split("-")
        due_date = f"{y}-{m}-{min(28, int(d) + 28):02d}"

        scenarios.append(
            Scenario(
                invoice_number=f"{prefix}-{i + 1:04d}",
                invoice_date=invoice_date,
                due_date=due_date,
                seller_name=seller[1],
                seller_siren=seller[0],
                seller_vat=seller[2],
                buyer_name=buyer[1],
                buyer_siren=buyer[0],
                buyer_vat=buyer[2],
                description=desc[0],
                unit_code=desc[1],
                qty=desc[2],
                unit_price=desc[3],
                has_pdf=has_pdf,
            )
        )
    return scenarios


UBL_TEMPLATE = '''<?xml version="1.0" encoding="UTF-8"?>
<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
         xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
         xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
    <cbc:CustomizationID>urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended</cbc:CustomizationID>
    <cbc:ProfileID>urn:fdc:peppol.eu:2017:poacc:billing:01:1.0</cbc:ProfileID>
    <cbc:ID>{invoice_number}</cbc:ID>
    <cbc:IssueDate>{invoice_date}</cbc:IssueDate>
    <cbc:DueDate>{due_date}</cbc:DueDate>
    <cbc:InvoiceTypeCode>380</cbc:InvoiceTypeCode>
    <cbc:Note>{description}</cbc:Note>
    <cbc:DocumentCurrencyCode>EUR</cbc:DocumentCurrencyCode>
    <cbc:BuyerReference>BC-{invoice_number}</cbc:BuyerReference>
    <cac:AccountingSupplierParty>
        <cac:Party>
            <cac:EndpointID schemeID="0009">{seller_siret}</cac:EndpointID>
            <cac:PartyName><cbc:Name>{seller_name}</cbc:Name></cac:PartyName>
            <cac:PostalAddress>
                <cbc:StreetName>10 rue de la Paix</cbc:StreetName>
                <cbc:CityName>Paris</cbc:CityName>
                <cbc:PostalZone>75001</cbc:PostalZone>
                <cac:Country><cbc:IdentificationCode>FR</cbc:IdentificationCode></cac:Country>
            </cac:PostalAddress>
            <cac:PartyTaxScheme>
                <cbc:CompanyID>{seller_vat}</cbc:CompanyID>
                <cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme>
            </cac:PartyTaxScheme>
            <cac:PartyLegalEntity>
                <cbc:RegistrationName>{seller_name}</cbc:RegistrationName>
                <cbc:CompanyID schemeID="0002">{seller_siren}</cbc:CompanyID>
            </cac:PartyLegalEntity>
        </cac:Party>
    </cac:AccountingSupplierParty>
    <cac:AccountingCustomerParty>
        <cac:Party>
            <cac:EndpointID schemeID="0009">{buyer_siret}</cac:EndpointID>
            <cac:PartyName><cbc:Name>{buyer_name}</cbc:Name></cac:PartyName>
            <cac:PostalAddress>
                <cbc:StreetName>5 avenue du Commerce</cbc:StreetName>
                <cbc:CityName>Lyon</cbc:CityName>
                <cbc:PostalZone>69001</cbc:PostalZone>
                <cac:Country><cbc:IdentificationCode>FR</cbc:IdentificationCode></cac:Country>
            </cac:PostalAddress>
            <cac:PartyTaxScheme>
                <cbc:CompanyID>{buyer_vat}</cbc:CompanyID>
                <cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme>
            </cac:PartyTaxScheme>
            <cac:PartyLegalEntity>
                <cbc:RegistrationName>{buyer_name}</cbc:RegistrationName>
                <cbc:CompanyID schemeID="0002">{buyer_siren}</cbc:CompanyID>
            </cac:PartyLegalEntity>
        </cac:Party>
    </cac:AccountingCustomerParty>
    <cac:PaymentMeans>
        <cbc:PaymentMeansCode>30</cbc:PaymentMeansCode>
        <cbc:PaymentID>{invoice_number}</cbc:PaymentID>
        <cac:PayeeFinancialAccount>
            <cbc:ID>FR7630001007941234567890185</cbc:ID>
            <cbc:Name>{seller_name}</cbc:Name>
        </cac:PayeeFinancialAccount>
    </cac:PaymentMeans>
{attachment}
    <cac:TaxTotal>
        <cbc:TaxAmount currencyID="EUR">{tva:.2f}</cbc:TaxAmount>
        <cac:TaxSubtotal>
            <cbc:TaxableAmount currencyID="EUR">{ht:.2f}</cbc:TaxableAmount>
            <cbc:TaxAmount currencyID="EUR">{tva:.2f}</cbc:TaxAmount>
            <cac:TaxCategory>
                <cbc:ID>S</cbc:ID>
                <cbc:Percent>20.00</cbc:Percent>
                <cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme>
            </cac:TaxCategory>
        </cac:TaxSubtotal>
    </cac:TaxTotal>
    <cac:LegalMonetaryTotal>
        <cbc:LineExtensionAmount currencyID="EUR">{ht:.2f}</cbc:LineExtensionAmount>
        <cbc:TaxExclusiveAmount currencyID="EUR">{ht:.2f}</cbc:TaxExclusiveAmount>
        <cbc:TaxInclusiveAmount currencyID="EUR">{ttc:.2f}</cbc:TaxInclusiveAmount>
        <cbc:PayableAmount currencyID="EUR">{ttc:.2f}</cbc:PayableAmount>
    </cac:LegalMonetaryTotal>
    <cac:InvoiceLine>
        <cbc:ID>1</cbc:ID>
        <cbc:InvoicedQuantity unitCode="{unit_code}">{qty}</cbc:InvoicedQuantity>
        <cbc:LineExtensionAmount currencyID="EUR">{ht:.2f}</cbc:LineExtensionAmount>
        <cac:Item>
            <cbc:Description>{description}</cbc:Description>
            <cbc:Name>{description}</cbc:Name>
            <cac:ClassifiedTaxCategory>
                <cbc:ID>S</cbc:ID>
                <cbc:Percent>20.00</cbc:Percent>
                <cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme>
            </cac:ClassifiedTaxCategory>
        </cac:Item>
        <cac:Price>
            <cbc:PriceAmount currencyID="EUR">{unit_price:.2f}</cbc:PriceAmount>
        </cac:Price>
    </cac:InvoiceLine>
</Invoice>
'''

ATTACHMENT_BLOCK_TEMPLATE = '''
    <cac:AdditionalDocumentReference>
        <cbc:ID>{att_id}</cbc:ID>
        <cbc:DocumentDescription>Visuel de la facture (PDF)</cbc:DocumentDescription>
        <cac:Attachment>
            <cbc:EmbeddedDocumentBinaryObject mimeCode="application/pdf"
                filename="{filename}">{pdf_base64}</cbc:EmbeddedDocumentBinaryObject>
        </cac:Attachment>
    </cac:AdditionalDocumentReference>
'''


def generate_pj_pdf_base64(invoice_xml: str, pdp_bin: str) -> str:
    """Génère un PDF visuel de la facture (via `pdp transform --to PDF`,
    moteur Typst) et retourne son contenu encodé en base64.

    On utilise le rendu PDF de la PDP elle-même : ainsi la PJ contient les
    vraies données (vendeur, acheteur, lignes, montants TVA/TTC), ce qui rend
    la démo plus parlante qu'avec un PDF placeholder vide.
    """
    import subprocess, tempfile, base64
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".xml", delete=False, encoding="utf-8"
    ) as src:
        src.write(invoice_xml)
        src_path = src.name
    pdf_path = src_path + ".pdf"
    try:
        subprocess.run(
            [pdp_bin, "transform", "--to", "PDF", "-o", pdf_path, src_path],
            check=True, capture_output=True, text=True,
        )
        with open(pdf_path, "rb") as f:
            return base64.b64encode(f.read()).decode("ascii")
    finally:
        for p in (src_path, pdf_path):
            try:
                os.unlink(p)
            except FileNotFoundError:
                pass

CII_TEMPLATE = '''<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice
    xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
    xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
    xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
    <rsm:ExchangedDocumentContext>
        <ram:GuidelineSpecifiedDocumentContextParameter>
            <ram:ID>urn:cen.eu:en16931:2017</ram:ID>
        </ram:GuidelineSpecifiedDocumentContextParameter>
    </rsm:ExchangedDocumentContext>
    <rsm:ExchangedDocument>
        <ram:ID>{invoice_number}</ram:ID>
        <ram:TypeCode>380</ram:TypeCode>
        <ram:IssueDateTime>
            <udt:DateTimeString format="102">{invoice_date_compact}</udt:DateTimeString>
        </ram:IssueDateTime>
{cii_attachments}
    </rsm:ExchangedDocument>
    <rsm:SupplyChainTradeTransaction>
        <ram:IncludedSupplyChainTradeLineItem>
            <ram:AssociatedDocumentLineDocument>
                <ram:LineID>1</ram:LineID>
            </ram:AssociatedDocumentLineDocument>
            <ram:SpecifiedTradeProduct>
                <ram:Name>{description}</ram:Name>
            </ram:SpecifiedTradeProduct>
            <ram:SpecifiedLineTradeAgreement>
                <ram:NetPriceProductTradePrice>
                    <ram:ChargeAmount>{unit_price:.2f}</ram:ChargeAmount>
                </ram:NetPriceProductTradePrice>
            </ram:SpecifiedLineTradeAgreement>
            <ram:SpecifiedLineTradeDelivery>
                <ram:BilledQuantity unitCode="{unit_code}">{qty}</ram:BilledQuantity>
            </ram:SpecifiedLineTradeDelivery>
            <ram:SpecifiedLineTradeSettlement>
                <ram:ApplicableTradeTax>
                    <ram:TypeCode>VAT</ram:TypeCode>
                    <ram:CategoryCode>S</ram:CategoryCode>
                    <ram:RateApplicablePercent>20.00</ram:RateApplicablePercent>
                </ram:ApplicableTradeTax>
                <ram:SpecifiedTradeSettlementLineMonetarySummation>
                    <ram:LineTotalAmount>{ht:.2f}</ram:LineTotalAmount>
                </ram:SpecifiedTradeSettlementLineMonetarySummation>
            </ram:SpecifiedLineTradeSettlement>
        </ram:IncludedSupplyChainTradeLineItem>
        <ram:ApplicableHeaderTradeAgreement>
            <ram:BuyerReference>BC-{invoice_number}</ram:BuyerReference>
            <ram:SellerTradeParty>
                <ram:Name>{seller_name}</ram:Name>
                <ram:SpecifiedLegalOrganization>
                    <ram:ID schemeID="0002">{seller_siren}</ram:ID>
                </ram:SpecifiedLegalOrganization>
                <ram:PostalTradeAddress>
                    <ram:LineOne>10 rue de la Paix</ram:LineOne>
                    <ram:CityName>Paris</ram:CityName>
                    <ram:PostcodeCode>75001</ram:PostcodeCode>
                    <ram:CountryID>FR</ram:CountryID>
                </ram:PostalTradeAddress>
                <ram:URIUniversalCommunication>
                    <ram:URIID schemeID="0009">{seller_siret}</ram:URIID>
                </ram:URIUniversalCommunication>
                <ram:SpecifiedTaxRegistration>
                    <ram:ID schemeID="VA">{seller_vat}</ram:ID>
                </ram:SpecifiedTaxRegistration>
            </ram:SellerTradeParty>
            <ram:BuyerTradeParty>
                <ram:Name>{buyer_name}</ram:Name>
                <ram:SpecifiedLegalOrganization>
                    <ram:ID schemeID="0002">{buyer_siren}</ram:ID>
                </ram:SpecifiedLegalOrganization>
                <ram:PostalTradeAddress>
                    <ram:LineOne>5 avenue du Commerce</ram:LineOne>
                    <ram:CityName>Lyon</ram:CityName>
                    <ram:PostcodeCode>69001</ram:PostcodeCode>
                    <ram:CountryID>FR</ram:CountryID>
                </ram:PostalTradeAddress>
                <ram:URIUniversalCommunication>
                    <ram:URIID schemeID="0009">{buyer_siret}</ram:URIID>
                </ram:URIUniversalCommunication>
                <ram:SpecifiedTaxRegistration>
                    <ram:ID schemeID="VA">{buyer_vat}</ram:ID>
                </ram:SpecifiedTaxRegistration>
            </ram:BuyerTradeParty>
        </ram:ApplicableHeaderTradeAgreement>
        <ram:ApplicableHeaderTradeDelivery/>
        <ram:ApplicableHeaderTradeSettlement>
            <ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
            <ram:SpecifiedTradeSettlementPaymentMeans>
                <ram:TypeCode>30</ram:TypeCode>
                <ram:PayeePartyCreditorFinancialAccount>
                    <ram:IBANID>FR7630001007941234567890185</ram:IBANID>
                </ram:PayeePartyCreditorFinancialAccount>
            </ram:SpecifiedTradeSettlementPaymentMeans>
            <ram:ApplicableTradeTax>
                <ram:CalculatedAmount>{tva:.2f}</ram:CalculatedAmount>
                <ram:TypeCode>VAT</ram:TypeCode>
                <ram:BasisAmount>{ht:.2f}</ram:BasisAmount>
                <ram:CategoryCode>S</ram:CategoryCode>
                <ram:RateApplicablePercent>20.00</ram:RateApplicablePercent>
            </ram:ApplicableTradeTax>
            <ram:SpecifiedTradePaymentTerms>
                <ram:DueDateDateTime>
                    <udt:DateTimeString format="102">{due_date_compact}</udt:DateTimeString>
                </ram:DueDateDateTime>
            </ram:SpecifiedTradePaymentTerms>
            <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
                <ram:LineTotalAmount>{ht:.2f}</ram:LineTotalAmount>
                <ram:TaxBasisTotalAmount>{ht:.2f}</ram:TaxBasisTotalAmount>
                <ram:TaxTotalAmount currencyID="EUR">{tva:.2f}</ram:TaxTotalAmount>
                <ram:GrandTotalAmount>{ttc:.2f}</ram:GrandTotalAmount>
                <ram:DuePayableAmount>{ttc:.2f}</ram:DuePayableAmount>
            </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
        </ram:ApplicableHeaderTradeSettlement>
    </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>
'''


def render_ubl(s: Scenario, attachment_block: str = "") -> str:
    return UBL_TEMPLATE.format(
        invoice_number=s.invoice_number,
        invoice_date=s.invoice_date,
        due_date=s.due_date,
        description=s.description,
        seller_siret=s.seller_siret, seller_siren=s.seller_siren,
        seller_vat=s.seller_vat, seller_name=s.seller_name,
        buyer_siret=s.buyer_siret, buyer_siren=s.buyer_siren,
        buyer_vat=s.buyer_vat, buyer_name=s.buyer_name,
        attachment=attachment_block,
        ht=s.ht, tva=s.tva, ttc=s.ttc,
        qty=s.qty, unit_code=s.unit_code, unit_price=s.unit_price,
    )


def build_attachment_block(s: Scenario, pdp_bin: str) -> str:
    """Génère 3 blocs `cac:AdditionalDocumentReference` portant des VRAIES
    pièces jointes : bon de commande PDF, bordereau de livraison PNG, détail
    des lignes CSV — chacun base64-encodé. Conforme XP Z12-012 §BG-24."""
    import base64, subprocess, tempfile, shutil
    no_pj_xml = render_ubl(s, attachment_block="")
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".xml", delete=False, encoding="utf-8"
    ) as tmp:
        tmp.write(no_pj_xml)
        tmp_path = tmp.name
    out_dir = tempfile.mkdtemp(prefix="ubl-pj-")
    try:
        subprocess.run(
            [pdp_bin, "tools", "gen-attachments", tmp_path, "-o", out_dir],
            check=True, capture_output=True, text=True,
        )
        artefacts = []
        for name in sorted(os.listdir(out_dir)):
            mime = (
                "application/pdf" if name.endswith(".pdf")
                else "image/png" if name.endswith(".png")
                else "text/csv" if name.endswith(".csv")
                else "application/octet-stream"
            )
            artefacts.append((name, open(os.path.join(out_dir, name), "rb").read(), mime))
    finally:
        os.unlink(tmp_path)
        shutil.rmtree(out_dir, ignore_errors=True)

    UBL_ATT_TEMPLATE = '''
    <cac:AdditionalDocumentReference>
        <cbc:ID>{att_id}</cbc:ID>
        <cbc:DocumentDescription>{desc}</cbc:DocumentDescription>
        <cac:Attachment>
            <cbc:EmbeddedDocumentBinaryObject mimeCode="{mime}" filename="{filename}">{b64}</cbc:EmbeddedDocumentBinaryObject>
        </cac:Attachment>
    </cac:AdditionalDocumentReference>'''
    blocks = []
    for filename, content, mime in artefacts:
        kind = (
            "BdC" if filename.startswith("bon_commande_")
            else "BL" if filename.startswith("bordereau_")
            else "DET" if filename.startswith("detail_lignes_")
            else "PJ"
        )
        desc_map = {
            "BdC": "Bon de commande",
            "BL": "Bordereau de livraison",
            "DET": "Détail des lignes",
        }
        blocks.append(UBL_ATT_TEMPLATE.format(
            att_id=f"{kind}-{s.invoice_number}",
            desc=desc_map.get(kind, "Pièce jointe"),
            mime=mime,
            filename=filename,
            b64=base64.b64encode(content).decode("ascii"),
        ))
    return "\n".join(blocks)


def render_cii(s: Scenario, cii_attachments: str = "") -> str:
    return CII_TEMPLATE.format(
        invoice_number=s.invoice_number,
        invoice_date_compact=s.invoice_date.replace("-", ""),
        due_date_compact=s.due_date.replace("-", ""),
        description=s.description,
        seller_siret=s.seller_siret, seller_siren=s.seller_siren,
        seller_vat=s.seller_vat, seller_name=s.seller_name,
        buyer_siret=s.buyer_siret, buyer_siren=s.buyer_siren,
        buyer_vat=s.buyer_vat, buyer_name=s.buyer_name,
        ht=s.ht, tva=s.tva, ttc=s.ttc,
        qty=s.qty, unit_code=s.unit_code, unit_price=s.unit_price,
        cii_attachments=cii_attachments,
    )


def build_cii_attachments_block(s: Scenario, pdp_bin: str) -> str:
    """Génère le bloc XML CII (`ram:AdditionalReferencedDocument`) avec les
    3 PJ "métier" embarquées : BdC PDF, bordereau livraison PNG, détail CSV.
    Conforme XP Z12-012 §BG-24, équivalent CII de cac:AdditionalDocumentReference.
    """
    import base64, subprocess, tempfile
    # Génère les artefacts à partir d'un UBL temporaire (le moteur attend une
    # InvoiceData parsée).
    no_pj_xml = render_ubl(s, attachment_block="")
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".xml", delete=False, encoding="utf-8"
    ) as tmp:
        tmp.write(no_pj_xml)
        tmp_path = tmp.name
    out_dir = tempfile.mkdtemp(prefix="cii-pj-")
    try:
        subprocess.run(
            [pdp_bin, "tools", "gen-attachments", tmp_path, "-o", out_dir],
            check=True, capture_output=True, text=True,
        )
        files = {}
        for name in os.listdir(out_dir):
            if name.startswith("bon_commande_"):
                files["bdc"] = (name, open(os.path.join(out_dir, name), "rb").read(), "application/pdf")
            elif name.startswith("bordereau_livraison_"):
                files["bl"] = (name, open(os.path.join(out_dir, name), "rb").read(), "image/png")
            elif name.startswith("detail_lignes_"):
                files["csv"] = (name, open(os.path.join(out_dir, name), "rb").read(), "text/csv")
    finally:
        import shutil
        os.unlink(tmp_path)
        shutil.rmtree(out_dir, ignore_errors=True)

    blocks = []
    for kind, (filename, content, mime) in files.items():
        b64 = base64.b64encode(content).decode("ascii")
        blocks.append(
            f"""        <ram:AdditionalReferencedDocument>
            <ram:IssuerAssignedID>PJ-{kind.upper()}-{s.invoice_number}</ram:IssuerAssignedID>
            <ram:TypeCode>916</ram:TypeCode>
            <ram:Name>{filename}</ram:Name>
            <ram:AttachmentBinaryObject mimeCode="{mime}" filename="{filename}">{b64}</ram:AttachmentBinaryObject>
        </ram:AdditionalReferencedDocument>"""
        )
    return "\n".join(blocks)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=240)
    parser.add_argument(
        "--ubl-pct", type=int, default=60,
        help="Pourcentage de fixtures UBL (défaut 60)",
    )
    parser.add_argument(
        "--cii-pct", type=int, default=30,
        help="Pourcentage de fixtures CII (défaut 30)",
    )
    parser.add_argument(
        "--facturx-pct", type=int, default=10,
        help="Pourcentage de fixtures Factur-X (défaut 10) — généré via `pdp transform`",
    )
    parser.add_argument(
        "--pdp-bin", default=None,
        help="Chemin vers le binaire pdp (défaut: ./target/release/pdp)",
    )
    args = parser.parse_args()

    if args.ubl_pct + args.cii_pct + args.facturx_pct != 100:
        parser.error("La somme --ubl-pct + --cii-pct + --facturx-pct doit valoir 100")

    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    ubl_dir = os.path.join(repo_root, "tests", "fixtures", "ubl")
    cii_dir = os.path.join(repo_root, "tests", "fixtures", "cii")
    fx_dir = os.path.join(repo_root, "tests", "fixtures", "facturx")
    os.makedirs(ubl_dir, exist_ok=True)
    os.makedirs(cii_dir, exist_ok=True)
    os.makedirs(fx_dir, exist_ok=True)

    pdp_bin = args.pdp_bin or os.path.join(repo_root, "target", "release", "pdp")

    scenarios = build_scenarios(args.count)
    n_emis = sum(1 for s in scenarios if s.seller_siren == TECHCONSEIL_SIREN)
    n_recu = sum(1 for s in scenarios if s.buyer_siren == TECHCONSEIL_SIREN)
    n_pj = sum(1 for s in scenarios if s.has_pdf)
    n_unknown = sum(
        1 for s in scenarios
        if s.seller_siren in {p[0] for p in UNKNOWN_PARTNERS}
        or s.buyer_siren in {p[0] for p in UNKNOWN_PARTNERS}
    )

    # Tirage déterministe du format par scénario (seed: invoice_number).
    rng_fmt = random.Random(99)
    n_ubl = n_cii = n_fx = 0
    fx_failures = []
    for s in scenarios:
        roll = rng_fmt.randint(0, 99)
        safe = s.invoice_number.replace(" ", "_").lower()
        if roll < args.ubl_pct:
            # Si la facture porte une PJ, on génère un VRAI PDF visuel via
            # `pdp transform --to PDF` (Typst, contenu lisible : raison
            # sociale, montants, lignes) et on l'embarque en base64.
            attachment = build_attachment_block(s, pdp_bin) if s.has_pdf else ""
            xml = render_ubl(s, attachment_block=attachment)
            path = os.path.join(ubl_dir, f"facture_ubl_{safe}.xml")
            with open(path, "w", encoding="utf-8") as f:
                f.write(xml)
            n_ubl += 1
        elif roll < args.ubl_pct + args.cii_pct:
            # CII : si la facture porte une PJ, on embarque les 3 PJ "métier"
            # via ram:AdditionalReferencedDocument (équivalent CII du UBL
            # cac:AdditionalDocumentReference).
            cii_pj = build_cii_attachments_block(s, pdp_bin) if s.has_pdf else ""
            xml = render_cii(s, cii_attachments=cii_pj)
            path = os.path.join(cii_dir, f"facture_cii_{safe}.xml")
            with open(path, "w", encoding="utf-8") as f:
                f.write(xml)
            n_cii += 1
        else:
            # Factur-X : on rend un UBL temporaire (avec ses 3 PJ si has_pdf)
            # et on appelle `pdp transform --to FacturX`. Le générateur PDF/A-3
            # extrait les `cac:AdditionalDocumentReference` de l'UBL et les
            # embarque comme EmbeddedFile dans le PDF (cf. facturx_generator.rs).
            import subprocess, tempfile
            ubl_attachment = build_attachment_block(s, pdp_bin) if s.has_pdf else ""
            tmp_ubl = tempfile.NamedTemporaryFile(
                mode="w", suffix=".xml", delete=False, encoding="utf-8",
            )
            tmp_ubl.write(render_ubl(s, attachment_block=ubl_attachment))
            tmp_ubl.close()
            pdf_out = os.path.join(fx_dir, f"facture_facturx_{safe}.pdf")
            try:
                subprocess.run(
                    [pdp_bin, "transform", "--to", "FacturX", "-o", pdf_out, tmp_ubl.name],
                    check=True, capture_output=True, text=True,
                )
                n_fx += 1
            except (FileNotFoundError, subprocess.CalledProcessError) as e:
                fx_failures.append((s.invoice_number, str(e)))
                fb_path = os.path.join(ubl_dir, f"facture_ubl_{safe}.xml")
                with open(fb_path, "w", encoding="utf-8") as f:
                    f.write(render_ubl(s, attachment_block=ubl_attachment))
                n_ubl += 1
            finally:
                os.unlink(tmp_ubl.name)

    print(f"Généré {len(scenarios)} fixtures BULK dans tests/fixtures/")
    print(f"  Émises (TechConseil = vendeur)  : {n_emis}")
    print(f"  Reçues (TechConseil = acheteur) : {n_recu}")
    print(f"  Avec PJ                         : {n_pj}")
    print(f"  Avec partenaire inconnu         : {n_unknown}")
    print()
    print(f"  UBL     : {n_ubl} → {ubl_dir}")
    print(f"  CII     : {n_cii} → {cii_dir}")
    print(f"  Factur-X: {n_fx} → {fx_dir}")
    if fx_failures:
        print()
        print(f"  ⚠️  {len(fx_failures)} échecs de génération Factur-X (fallback UBL) :")
        for inv, err in fx_failures[:3]:
            print(f"     {inv}: {err.splitlines()[-1] if err else 'inconnu'}")
        if len(fx_failures) > 3:
            print(f"     ... et {len(fx_failures) - 3} autres")


if __name__ == "__main__":
    main()
