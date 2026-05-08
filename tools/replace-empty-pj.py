#!/usr/bin/env python3
"""Remplace les PJ embarquées (PDF/PNG/CSV placeholders) dans les fixtures
UBL/CII par des fichiers RÉELS générés via `pdp tools gen-attachments`.

Pour chaque fixture, on utilise les filenames présents dans le XML pour
décider quel PDF/PNG/CSV générer :
- `bon_commande_*.pdf` → PDF du bon de commande (BdC)
- `bordereau_livraison_*.png` → PNG du bordereau de livraison
- `detail_lignes_*.csv` → CSV des lignes
- autres `*.pdf` → PDF visuel de la facture (`pdp transform --to PDF`)

Les fichiers générés contiennent les VRAIES données de chaque facture
(raison sociale, SIRET, lignes, montants).
"""
from __future__ import annotations
import base64
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
PDP_BIN = os.path.join(REPO, "target", "release", "pdp")
EMBED_RX = re.compile(
    r'(<cbc:EmbeddedDocumentBinaryObject\s+mimeCode="[^"]+"\s+filename="([^"]+)"\s*>)([^<]+)(</cbc:EmbeddedDocumentBinaryObject>)',
    re.DOTALL,
)
# Variante avec retours à la ligne dans l'ouverture (cas réels)
EMBED_RX_FLEX = re.compile(
    r'(<cbc:EmbeddedDocumentBinaryObject[^>]*?filename="([^"]+)"[^>]*>)([^<]+)(</cbc:EmbeddedDocumentBinaryObject>)',
    re.DOTALL,
)


def mime_for(filename: str) -> str:
    f = filename.lower()
    if f.endswith(".pdf"):
        return "application/pdf"
    if f.endswith(".png"):
        return "image/png"
    if f.endswith(".csv"):
        return "text/csv"
    return "application/octet-stream"


def gen_attachments(invoice_xml_path: str) -> dict[str, bytes]:
    """Appelle `pdp tools gen-attachments` et retourne les 3 PJ générées
    indexées par type ('bdc', 'bl', 'csv'). Plus un 'invoice_pdf' pour le
    rendu visuel de la facture (utilisé par les noms génériques)."""
    out = {}
    with tempfile.TemporaryDirectory(prefix="pj-") as tmp:
        try:
            subprocess.run(
                [PDP_BIN, "tools", "gen-attachments", invoice_xml_path, "-o", tmp],
                check=True, capture_output=True, text=True,
            )
            for name in os.listdir(tmp):
                p = os.path.join(tmp, name)
                if name.startswith("bon_commande_"):
                    out["bdc"] = open(p, "rb").read()
                elif name.startswith("bordereau_livraison_"):
                    out["bl"] = open(p, "rb").read()
                elif name.startswith("detail_lignes_"):
                    out["csv"] = open(p, "rb").read()
        except subprocess.CalledProcessError as e:
            raise RuntimeError(f"gen-attachments échoué : {e.stderr.strip()[:200] if e.stderr else e}") from e

    # PDF visuel facture pour les filenames génériques (*.pdf sans préfixe BdC).
    pdf_path = invoice_xml_path + ".visual.pdf"
    try:
        subprocess.run(
            [PDP_BIN, "transform", "--to", "PDF", "-o", pdf_path, invoice_xml_path],
            check=True, capture_output=True, text=True,
        )
        out["invoice_pdf"] = open(pdf_path, "rb").read()
    finally:
        if os.path.exists(pdf_path):
            os.unlink(pdf_path)
    return out


def pick_attachment(filename: str, generated: dict[str, bytes]) -> bytes | None:
    """Sélectionne le bon contenu PJ selon le nom de fichier."""
    fname_lower = filename.lower()
    if fname_lower.startswith("bon_commande") or fname_lower.startswith("bdc"):
        return generated.get("bdc")
    if fname_lower.startswith("bordereau_livraison") or fname_lower.startswith("bl_"):
        return generated.get("bl")
    if fname_lower.endswith(".csv") or "detail_lignes" in fname_lower:
        return generated.get("csv")
    if fname_lower.endswith(".png"):
        return generated.get("bl")  # fallback PNG → bordereau
    if fname_lower.endswith(".pdf"):
        return generated.get("invoice_pdf")
    return None


def needs_patch(b64: str) -> bool:
    """True si la PJ décodée fait < 1 Ko (signature placeholder)."""
    try:
        return len(base64.b64decode(b64)) < 1024
    except Exception:
        return False


def is_named_attachment(filename: str) -> bool:
    """Filename qui désigne explicitement une PJ "métier" connue (BdC, BL,
    détail). On régénère systématiquement ces PJ même si elles ne sont pas
    vides — pour s'assurer que `bon_commande_*.pdf` est bien un BdC, pas un
    rendu de la facture."""
    f = filename.lower()
    return (
        f.startswith("bon_commande")
        or f.startswith("bdc")
        or f.startswith("bordereau_livraison")
        or f.startswith("bl_")
        or "detail_lignes" in f
    )


def patch_fixture(path: str) -> str | None:
    xml = open(path, encoding="utf-8").read()
    matches = list(EMBED_RX_FLEX.finditer(xml))
    if not matches:
        return None
    if not any(needs_patch(m.group(3)) or is_named_attachment(m.group(2)) for m in matches):
        return None

    # Génère les vrais artefacts depuis une copie sans PJ (sinon `pdp transform`
    # essaierait de réembarquer une autre couche).
    no_pj_xml = EMBED_RX_FLEX.sub(r'\1<!--placeholder-->\4', xml)
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".xml", delete=False, encoding="utf-8"
    ) as tmp:
        tmp.write(no_pj_xml)
        tmp_path = tmp.name
    try:
        try:
            generated = gen_attachments(tmp_path)
        except RuntimeError as e:
            return f"  ❌ {os.path.basename(path)} : {e}"
    finally:
        os.unlink(tmp_path)

    replaced = []
    def repl(m):
        opening, filename, b64, closing = m.groups()
        content = pick_attachment(filename, generated)
        if content is None:
            return m.group(0)  # filename inconnu, on laisse
        if not needs_patch(b64) and not is_named_attachment(filename):
            return m.group(0)  # déjà OK et pas de pattern connu à forcer
        replaced.append(filename)
        new_b64 = base64.b64encode(content).decode("ascii")
        return f"{opening}{new_b64}{closing}"

    new_xml = EMBED_RX_FLEX.sub(repl, xml)
    if not replaced:
        return None
    with open(path, "w", encoding="utf-8") as f:
        f.write(new_xml)
    return f"  ✅ {os.path.basename(path)} : {', '.join(replaced)}"


def main() -> int:
    if not os.path.isfile(PDP_BIN):
        print(f"❌ Binaire pdp introuvable : {PDP_BIN}", file=sys.stderr)
        return 1

    fixture_dirs = [
        os.path.join(REPO, "tests", "fixtures", "ubl"),
        os.path.join(REPO, "tests", "fixtures", "cii"),
    ]
    patched = 0
    skipped = 0
    failed = 0
    for d in fixture_dirs:
        if not os.path.isdir(d):
            continue
        for name in sorted(os.listdir(d)):
            if not name.endswith(".xml"):
                continue
            path = os.path.join(d, name)
            res = patch_fixture(path)
            if res is None:
                skipped += 1
            elif res.startswith("  ✅"):
                patched += 1
                print(res)
            else:
                failed += 1
                print(res, file=sys.stderr)

    print()
    print(f"Patché : {patched}")
    print(f"Inchangé : {skipped}")
    print(f"Échec : {failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
