#!/usr/bin/env python3
"""Génère un volume important de fixtures UBL pour le tenant TechConseil
(SIREN 123456789) afin de pouvoir tester la pagination de l'UI.

Volume par défaut : 240 factures (120 émises + 120 reçues), réparties sur
plusieurs partenaires de l'annuaire F14_demo.xml. Quelques pourcents sont
émises avec PJ, et ~5% utilisent un SIREN partenaire absent de l'annuaire
(déclenche EMMET_INC en réception).

Lancement :
    python3 tools/gen-fixtures-bulk.py [--count 240]

Les XML sont écrits dans tests/fixtures/ubl/.
Les invoice_number sont préfixés par BULK- pour les distinguer.
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

# PJ minimale (PDF 1 page vide)
PDF_BASE64 = (
    "JVBERi0xLjQKJeLjz9MKMSAwIG9iago8PC9UeXBlL0NhdGFsb2cvUGFnZXMgMiAwIFI+Pg"
    "plbmRvYmoKMiAwIG9iago8PC9UeXBlL1BhZ2VzL0tpZHNbMyAwIFJdL0NvdW50IDE+Pgpl"
    "bmRvYmoKMyAwIG9iago8PC9UeXBlL1BhZ2UvUGFyZW50IDIgMCBSL01lZGlhQm94WzAg"
    "MCAxMDAgMTAwXT4+CmVuZG9iagp4cmVmCjAgNAowMDAwMDAwMDAwIDY1NTM1IGYNCjAw"
    "MDAwMDAwMDkgMDAwMDAgbg0KMDAwMDAwMDA1NyAwMDAwMCBuDQowMDAwMDAwMTEzIDAw"
    "MDAwIG4NCnRyYWlsZXIKPDwvU2l6ZSA0L1Jvb3QgMSAwIFI+PgpzdGFydHhyZWYKMTcy"
    "CiUlRU9G"
)


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

ATTACHMENT_BLOCK = f'''
    <cac:AdditionalDocumentReference>
        <cbc:ID>BULK-PJ-001</cbc:ID>
        <cbc:DocumentDescription>Bon de commande</cbc:DocumentDescription>
        <cac:Attachment>
            <cbc:EmbeddedDocumentBinaryObject mimeCode="application/pdf"
                filename="bdc.pdf">{PDF_BASE64}</cbc:EmbeddedDocumentBinaryObject>
        </cac:Attachment>
    </cac:AdditionalDocumentReference>
'''


def render(s: Scenario) -> str:
    return UBL_TEMPLATE.format(
        invoice_number=s.invoice_number,
        invoice_date=s.invoice_date,
        due_date=s.due_date,
        description=s.description,
        seller_siret=s.seller_siret, seller_siren=s.seller_siren,
        seller_vat=s.seller_vat, seller_name=s.seller_name,
        buyer_siret=s.buyer_siret, buyer_siren=s.buyer_siren,
        buyer_vat=s.buyer_vat, buyer_name=s.buyer_name,
        attachment=ATTACHMENT_BLOCK if s.has_pdf else "",
        ht=s.ht, tva=s.tva, ttc=s.ttc,
        qty=s.qty, unit_code=s.unit_code, unit_price=s.unit_price,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=240)
    args = parser.parse_args()

    out_dir = os.path.join(os.path.dirname(__file__), "..", "tests", "fixtures", "ubl")
    out_dir = os.path.abspath(out_dir)
    os.makedirs(out_dir, exist_ok=True)

    scenarios = build_scenarios(args.count)
    n_emis = sum(1 for s in scenarios if s.seller_siren == TECHCONSEIL_SIREN)
    n_recu = sum(1 for s in scenarios if s.buyer_siren == TECHCONSEIL_SIREN)
    n_pj = sum(1 for s in scenarios if s.has_pdf)
    n_unknown = sum(
        1 for s in scenarios
        if s.seller_siren in {p[0] for p in UNKNOWN_PARTNERS}
        or s.buyer_siren in {p[0] for p in UNKNOWN_PARTNERS}
    )

    for s in scenarios:
        xml = render(s)
        safe = s.invoice_number.replace(" ", "_").lower()
        path = os.path.join(out_dir, f"facture_ubl_{safe}.xml")
        with open(path, "w", encoding="utf-8") as f:
            f.write(xml)

    print(f"Généré {len(scenarios)} fixtures BULK dans {out_dir}")
    print(f"  Émises (TechConseil = vendeur)  : {n_emis}")
    print(f"  Reçues (TechConseil = acheteur) : {n_recu}")
    print(f"  Avec PJ                         : {n_pj}")
    print(f"  Avec partenaire inconnu         : {n_unknown}")


if __name__ == "__main__":
    main()
