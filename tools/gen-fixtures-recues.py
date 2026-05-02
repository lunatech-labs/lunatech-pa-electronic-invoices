#!/usr/bin/env python3
"""Génère des fixtures UBL pour le tenant TechConseil (SIREN 123456789),
en tant qu'**acheteur** (factures reçues), avec une variété :

- Plusieurs vendeurs distincts présents dans l'annuaire F14_demo.xml
- Quelques fixtures avec PJ embarquée (PDF + CSV)
- Une fixture émise vers un acheteur inconnu de l'annuaire (déclenche DEST_INC)
- Une fixture reçue d'un vendeur inconnu de l'annuaire (déclenche EMMET_INC)

Lancement :
    python3 tools/gen-fixtures-recues.py

Les XML sont écrits dans tests/fixtures/ubl/.
"""
from __future__ import annotations
import os
from dataclasses import dataclass, field

# Tenant principal de la démo (acheteur = TechConseil SAS)
TECHCONSEIL_SIREN = "123456789"
TECHCONSEIL_SIRET = "12345678901234"
TECHCONSEIL_NAME = "TechConseil SAS"
TECHCONSEIL_VAT = "FR12123456789"

# PJ minimales (PDF 1 page vide, CSV 2 lignes) — base64
PDF_BASE64 = (
    "JVBERi0xLjQKJeLjz9MKMSAwIG9iago8PC9UeXBlL0NhdGFsb2cvUGFnZXMgMiAwIFI+Pg"
    "plbmRvYmoKMiAwIG9iago8PC9UeXBlL1BhZ2VzL0tpZHNbMyAwIFJdL0NvdW50IDE+Pgpl"
    "bmRvYmoKMyAwIG9iago8PC9UeXBlL1BhZ2UvUGFyZW50IDIgMCBSL01lZGlhQm94WzAg"
    "MCAxMDAgMTAwXT4+CmVuZG9iagp4cmVmCjAgNAowMDAwMDAwMDAwIDY1NTM1IGYNCjAw"
    "MDAwMDAwMDkgMDAwMDAgbg0KMDAwMDAwMDA1NyAwMDAwMCBuDQowMDAwMDAwMTEzIDAw"
    "MDAwIG4NCnRyYWlsZXIKPDwvU2l6ZSA0L1Jvb3QgMSAwIFI+PgpzdGFydHhyZWYKMTcy"
    "CiUlRU9G"
)
CSV_BASE64 = "cmVmO3F0ZTtwdQpBMDAxOzEwOzI1LjAwCkEwMDI7NTsxMi41MAo="


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
    unit_price: float
    has_pdf: bool = False
    has_csv: bool = False
    note: str | None = None  # surcharge la note du <cbc:Note> (sinon = description)

    @property
    def seller_siret(self) -> str:
        # Convention démo : SIREN + suffixe
        return self.seller_siren + "00001"

    @property
    def buyer_siret(self) -> str:
        return self.buyer_siren + "00002"

    @property
    def ht(self) -> float:
        return self.qty * self.unit_price

    @property
    def tva(self) -> float:
        return round(self.ht * 0.20, 2)

    @property
    def ttc(self) -> float:
        return round(self.ht + self.tva, 2)


# --- Vendeurs présents dans l'annuaire F14_demo.xml ---
PLOMBERIE_DURAND = ("738492012", "Plomberie Durand SARL", "FR73738492012")
CONSEIL_EXPERT = ("415263748", "Conseil Expert SAS", "FR41415263748")
RENO_HABITAT = ("918273645", "Rénovation Habitat Plus SARL", "FR91918273645")
PLOMBERIE_MARTIN = ("222333444", "Plomberie Martin SARL", "FR22222333444")
SERVICES_NUM = ("493827160", "Services Numériques SARL", "FR49493827160")
AUDIOTECH = ("827364519", "AudioTech France SAS", "FR82827364519")
ELECTRO_DIST = ("536987210", "Électronique Distribution SARL", "FR53536987210")

# SIRENs ABSENTS de l'annuaire (déclenchent EMMET_INC / DEST_INC)
SIREN_INCONNU_VENDEUR = ("888777666", "Fournisseur Inconnu SARL", "FR88888777666")
SIREN_INCONNU_ACHETEUR = ("999000111", "Acheteur Mystère SAS", "FR99999000111")
SIREN_INCONNU_VENDEUR_2 = ("777666555", "Société Fantôme SAS", "FR77777666555")
SIREN_INCONNU_VENDEUR_3 = ("666555444", "Entreprise X EURL", "FR66666555444")
SIREN_INCONNU_ACHETEUR_2 = ("555444333", "Client Inexistant SARL", "FR55555444333")


SCENARIOS: list[Scenario] = [
    # ---- Reçues SANS PJ : 4 fournisseurs variés ----
    Scenario(
        invoice_number="REC-PLO-2026-001",
        invoice_date="2026-04-05",
        due_date="2026-05-05",
        seller_name=PLOMBERIE_DURAND[1],
        seller_siren=PLOMBERIE_DURAND[0],
        seller_vat=PLOMBERIE_DURAND[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Réparation fuite circuit eau froide locaux Paris 8e",
        qty=1,
        unit_price=420.00,
    ),
    Scenario(
        invoice_number="REC-CE-2026-014",
        invoice_date="2026-04-08",
        due_date="2026-05-08",
        seller_name=CONSEIL_EXPERT[1],
        seller_siren=CONSEIL_EXPERT[0],
        seller_vat=CONSEIL_EXPERT[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Mission audit sécurité applicative — 5 jours",
        qty=5,
        unit_price=1100.00,
    ),
    Scenario(
        invoice_number="REC-PM-2026-022",
        invoice_date="2026-04-12",
        due_date="2026-05-12",
        seller_name=PLOMBERIE_MARTIN[1],
        seller_siren=PLOMBERIE_MARTIN[0],
        seller_vat=PLOMBERIE_MARTIN[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Maintenance trimestrielle plomberie immeuble",
        qty=1,
        unit_price=680.00,
    ),
    Scenario(
        invoice_number="REC-SN-2026-031",
        invoice_date="2026-04-18",
        due_date="2026-05-18",
        seller_name=SERVICES_NUM[1],
        seller_siren=SERVICES_NUM[0],
        seller_vat=SERVICES_NUM[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Hébergement infrastructure Q2 2026",
        qty=3,
        unit_price=850.00,
    ),
    # ---- Reçue AVEC PJ (PDF bon de commande) ----
    Scenario(
        invoice_number="REC-RH-2026-077",
        invoice_date="2026-04-15",
        due_date="2026-05-15",
        seller_name=RENO_HABITAT[1],
        seller_siren=RENO_HABITAT[0],
        seller_vat=RENO_HABITAT[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Rénovation salle réunion — peinture et faux-plafond",
        qty=1,
        unit_price=4850.00,
        has_pdf=True,
        note="Pièce jointe : devis signé.",
    ),
    # ---- Reçue AVEC PJ (PDF + CSV détail lignes) ----
    Scenario(
        invoice_number="REC-AT-2026-088",
        invoice_date="2026-04-22",
        due_date="2026-05-22",
        seller_name=AUDIOTECH[1],
        seller_siren=AUDIOTECH[0],
        seller_vat=AUDIOTECH[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Achat équipements visioconférence salles 1 à 4",
        qty=4,
        unit_price=1290.00,
        has_pdf=True,
        has_csv=True,
        note="Pièces jointes : bon de commande + détail des lignes en CSV.",
    ),
    # ---- Reçue d'un fournisseur INCONNU de l'annuaire → EMMET_INC ----
    Scenario(
        invoice_number="REC-INCONNU-2026-001",
        invoice_date="2026-04-03",
        due_date="2026-05-03",
        seller_name=SIREN_INCONNU_VENDEUR[1],
        seller_siren=SIREN_INCONNU_VENDEUR[0],
        seller_vat=SIREN_INCONNU_VENDEUR[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Facture fournisseur inconnu (test EMMET_INC)",
        qty=1,
        unit_price=200.00,
    ),
    # ---- Émise par TechConseil vers un acheteur INCONNU → DEST_INC ----
    Scenario(
        invoice_number="EMI-DEST-INCONNU-2026-001",
        invoice_date="2026-04-07",
        due_date="2026-05-07",
        seller_name=TECHCONSEIL_NAME,
        seller_siren=TECHCONSEIL_SIREN,
        seller_vat=TECHCONSEIL_VAT,
        buyer_name=SIREN_INCONNU_ACHETEUR[1],
        buyer_siren=SIREN_INCONNU_ACHETEUR[0],
        buyer_vat=SIREN_INCONNU_ACHETEUR[2],
        description="Prestation pour client non-référencé (test DEST_INC)",
        qty=1,
        unit_price=750.00,
    ),
    # ---- Reçue d'un autre fournisseur INCONNU (avec PJ) → EMMET_INC ----
    Scenario(
        invoice_number="REC-FANTOME-2026-001",
        invoice_date="2026-04-11",
        due_date="2026-05-11",
        seller_name=SIREN_INCONNU_VENDEUR_2[1],
        seller_siren=SIREN_INCONNU_VENDEUR_2[0],
        seller_vat=SIREN_INCONNU_VENDEUR_2[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Prestations diverses - vendeur fantôme",
        qty=2,
        unit_price=550.00,
        has_pdf=True,
    ),
    # ---- Reçue d'un 3e fournisseur INCONNU → EMMET_INC ----
    Scenario(
        invoice_number="REC-INCONNU-X-2026-002",
        invoice_date="2026-04-14",
        due_date="2026-05-14",
        seller_name=SIREN_INCONNU_VENDEUR_3[1],
        seller_siren=SIREN_INCONNU_VENDEUR_3[0],
        seller_vat=SIREN_INCONNU_VENDEUR_3[2],
        buyer_name=TECHCONSEIL_NAME,
        buyer_siren=TECHCONSEIL_SIREN,
        buyer_vat=TECHCONSEIL_VAT,
        description="Achats divers - SIREN absent annuaire",
        qty=1,
        unit_price=320.00,
    ),
    # ---- Émise par TechConseil vers un 2e acheteur INCONNU (avec PJ) → DEST_INC ----
    Scenario(
        invoice_number="EMI-CLIENT-INCONNU-2026-002",
        invoice_date="2026-04-21",
        due_date="2026-05-21",
        seller_name=TECHCONSEIL_NAME,
        seller_siren=TECHCONSEIL_SIREN,
        seller_vat=TECHCONSEIL_VAT,
        buyer_name=SIREN_INCONNU_ACHETEUR_2[1],
        buyer_siren=SIREN_INCONNU_ACHETEUR_2[0],
        buyer_vat=SIREN_INCONNU_ACHETEUR_2[2],
        description="Mission conseil pour client non référencé",
        qty=4,
        unit_price=850.00,
        has_pdf=True,
    ),
    # ---- Émise par TechConseil vers un acheteur connu, AVEC PJ ----
    Scenario(
        invoice_number="EMI-PJ-2026-002",
        invoice_date="2026-04-19",
        due_date="2026-05-19",
        seller_name=TECHCONSEIL_NAME,
        seller_siren=TECHCONSEIL_SIREN,
        seller_vat=TECHCONSEIL_VAT,
        buyer_name=ELECTRO_DIST[1],
        buyer_siren=ELECTRO_DIST[0],
        buyer_vat=ELECTRO_DIST[2],
        description="Prestation conseil technique + livrables annexés",
        qty=10,
        unit_price=900.00,
        has_pdf=True,
        note="Pièce jointe : rapport de mission signé.",
    ),
]


UBL_HEADER = '''<?xml version="1.0" encoding="UTF-8"?>
<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
         xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
         xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
    <cbc:CustomizationID>urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended</cbc:CustomizationID>
    <cbc:ProfileID>urn:fdc:peppol.eu:2017:poacc:billing:01:1.0</cbc:ProfileID>
    <cbc:ID>{invoice_number}</cbc:ID>
    <cbc:IssueDate>{invoice_date}</cbc:IssueDate>
    <cbc:DueDate>{due_date}</cbc:DueDate>
    <cbc:InvoiceTypeCode>380</cbc:InvoiceTypeCode>
    <cbc:Note>{note}</cbc:Note>
    <cbc:DocumentCurrencyCode>EUR</cbc:DocumentCurrencyCode>
    <cbc:BuyerReference>BC-{invoice_number}</cbc:BuyerReference>
'''

UBL_PARTY = '''
    <cac:{party_role}>
        <cac:Party>
            <cac:EndpointID schemeID="0009">{siret}</cac:EndpointID>
            <cac:PartyIdentification>
                <cbc:ID schemeID="0009">{siret}</cbc:ID>
            </cac:PartyIdentification>
            <cac:PartyName>
                <cbc:Name>{name}</cbc:Name>
            </cac:PartyName>
            <cac:PostalAddress>
                <cbc:StreetName>{street}</cbc:StreetName>
                <cbc:CityName>{city}</cbc:CityName>
                <cbc:PostalZone>{zip}</cbc:PostalZone>
                <cac:Country>
                    <cbc:IdentificationCode>FR</cbc:IdentificationCode>
                </cac:Country>
            </cac:PostalAddress>
            <cac:PartyTaxScheme>
                <cbc:CompanyID>{vat}</cbc:CompanyID>
                <cac:TaxScheme>
                    <cbc:ID>VAT</cbc:ID>
                </cac:TaxScheme>
            </cac:PartyTaxScheme>
            <cac:PartyLegalEntity>
                <cbc:RegistrationName>{name}</cbc:RegistrationName>
                <cbc:CompanyID schemeID="0002">{siren}</cbc:CompanyID>
            </cac:PartyLegalEntity>
        </cac:Party>
    </cac:{party_role}>
'''

UBL_ATTACHMENT_PDF = '''
    <cac:AdditionalDocumentReference>
        <cbc:ID>BC-PJ-{idx}</cbc:ID>
        <cbc:DocumentDescription>{desc}</cbc:DocumentDescription>
        <cac:Attachment>
            <cbc:EmbeddedDocumentBinaryObject mimeCode="application/pdf"
                filename="{filename}">{base64}</cbc:EmbeddedDocumentBinaryObject>
        </cac:Attachment>
    </cac:AdditionalDocumentReference>
'''

UBL_ATTACHMENT_CSV = '''
    <cac:AdditionalDocumentReference>
        <cbc:ID>DET-PJ-{idx}</cbc:ID>
        <cbc:DocumentDescription>Détail des lignes (CSV)</cbc:DocumentDescription>
        <cac:Attachment>
            <cbc:EmbeddedDocumentBinaryObject mimeCode="text/csv"
                filename="{filename}">{base64}</cbc:EmbeddedDocumentBinaryObject>
        </cac:Attachment>
    </cac:AdditionalDocumentReference>
'''

UBL_TOTALS_AND_LINE = '''
    <cac:PaymentMeans>
        <cbc:PaymentMeansCode>30</cbc:PaymentMeansCode>
        <cbc:PaymentID>{invoice_number}</cbc:PaymentID>
        <cac:PayeeFinancialAccount>
            <cbc:ID>FR7630001007941234567890185</cbc:ID>
            <cbc:Name>{seller_name}</cbc:Name>
        </cac:PayeeFinancialAccount>
    </cac:PaymentMeans>

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
        <cbc:InvoicedQuantity unitCode="C62">{qty}</cbc:InvoicedQuantity>
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


def render(s: Scenario) -> str:
    note = s.note or s.description
    parts = [UBL_HEADER.format(
        invoice_number=s.invoice_number,
        invoice_date=s.invoice_date,
        due_date=s.due_date,
        note=note,
    )]
    parts.append(UBL_PARTY.format(
        party_role="AccountingSupplierParty",
        siret=s.seller_siret, siren=s.seller_siren, vat=s.seller_vat, name=s.seller_name,
        street="10 rue de la Paix", city="Paris", zip="75001",
    ))
    parts.append(UBL_PARTY.format(
        party_role="AccountingCustomerParty",
        siret=s.buyer_siret, siren=s.buyer_siren, vat=s.buyer_vat, name=s.buyer_name,
        street="5 avenue du Commerce", city="Lyon", zip="69001",
    ))
    if s.has_pdf:
        parts.append(UBL_ATTACHMENT_PDF.format(
            idx="001",
            desc="Bon de commande / devis signé",
            filename=f"bdc_{s.invoice_number.lower()}.pdf",
            base64=PDF_BASE64,
        ))
    if s.has_csv:
        parts.append(UBL_ATTACHMENT_CSV.format(
            idx="002",
            filename=f"detail_{s.invoice_number.lower()}.csv",
            base64=CSV_BASE64,
        ))
    parts.append(UBL_TOTALS_AND_LINE.format(
        invoice_number=s.invoice_number,
        seller_name=s.seller_name,
        ht=s.ht, tva=s.tva, ttc=s.ttc,
        qty=s.qty, unit_price=s.unit_price,
        description=s.description,
    ))
    return "".join(parts)


def main() -> None:
    out_dir = os.path.join(os.path.dirname(__file__), "..", "tests", "fixtures", "ubl")
    out_dir = os.path.abspath(out_dir)
    os.makedirs(out_dir, exist_ok=True)

    for s in SCENARIOS:
        xml = render(s)
        safe = s.invoice_number.replace(" ", "_").lower()
        path = os.path.join(out_dir, f"facture_ubl_{safe}.xml")
        with open(path, "w", encoding="utf-8") as f:
            f.write(xml)
        flag_pj = "+PJ" if (s.has_pdf or s.has_csv) else "  "
        print(f"  ✓ {os.path.basename(path):60s} {flag_pj}  {s.seller_name[:25]:25s} → {s.buyer_name[:25]:25s} ({s.ttc:>9.2f} €)")


if __name__ == "__main__":
    main()
