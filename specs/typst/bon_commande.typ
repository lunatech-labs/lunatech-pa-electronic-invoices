// Bon de commande — template Typst rendu par `pdp tools gen-attachments`.
//
// Utilisé comme pièce jointe (cac:AdditionalDocumentReference) pour des
// scénarios de démonstration : la BdC précède la facture et reprend les
// références (BT-13 PurchaseOrderReference). Document ÉMIS PAR L'ACHETEUR
// (le destinataire de la facture), donc rôles inversés vs la facture.

#let data = json.decode(sys.inputs.at("invoice-data", default: "{}"))
#let get(key, default: none) = data.at(key, default: default)

#set page(paper: "a4", margin: (x: 2cm, y: 2cm))
#set text(font: ("Source Sans Pro", "Helvetica"), size: 10pt)

#let buyer = get("buyer_name", default: "Acheteur")
#let buyer_siret = get("buyer_siret", default: "—")
#let buyer_vat = get("buyer_vat_id", default: "")
#let buyer_line1 = get("buyer_address_line1")
#let buyer_postal = get("buyer_address_postal_code")
#let buyer_city = get("buyer_address_city")
#let seller = get("seller_name", default: "Fournisseur")
#let seller_siret = get("seller_siret", default: "—")
#let seller_vat = get("seller_vat_id", default: "")
#let seller_line1 = get("seller_address_line1")
#let seller_postal = get("seller_address_postal_code")
#let seller_city = get("seller_address_city")
#let invoice_no = get("invoice_number", default: "—")
#let order_ref = get("order_reference", default: "BC-" + invoice_no)
#let issue_date = get("issue_date", default: "—")

// En-tête
#align(center, text(size: 22pt, weight: "bold", fill: rgb("#1a472a"))[BON DE COMMANDE])
#align(center, text(size: 12pt)[N° #order_ref])
#v(8pt)

// Encadré donneur d'ordre / fournisseur
#grid(
  columns: (1fr, 1fr),
  column-gutter: 12pt,
  box(
    width: 100%,
    inset: 10pt,
    radius: 4pt,
    stroke: 0.5pt + luma(180),
    fill: luma(248),
    {
      text(weight: "bold", "DONNEUR D'ORDRE")
      linebreak()
      v(4pt)
      text(weight: "bold", buyer)
      linebreak()
      if buyer_line1 != none [#buyer_line1 \ ]
      if buyer_postal != none and buyer_city != none [
        #buyer_postal #buyer_city \
      ]
      text(size: 9pt, fill: luma(80))[SIRET : #buyer_siret]
      linebreak()
      if buyer_vat != none and buyer_vat != "" { text(size: 9pt, fill: luma(80))[TVA : #buyer_vat] }
    },
  ),
  box(
    width: 100%,
    inset: 10pt,
    radius: 4pt,
    stroke: 0.5pt + luma(180),
    {
      text(weight: "bold", "FOURNISSEUR")
      linebreak()
      v(4pt)
      text(weight: "bold", seller)
      linebreak()
      if seller_line1 != none [#seller_line1 \ ]
      if seller_postal != none and seller_city != none [
        #seller_postal #seller_city \
      ]
      text(size: 9pt, fill: luma(80))[SIRET : #seller_siret]
      linebreak()
      if seller_vat != none and seller_vat != "" { text(size: 9pt, fill: luma(80))[TVA : #seller_vat] }
    },
  ),
)

#v(12pt)

// Métadonnées commande
#box(
  width: 100%,
  inset: 8pt,
  radius: 3pt,
  fill: rgb("#e8f5e9"),
  grid(
    columns: (1fr, 1fr, 1fr),
    align: left,
    [*Date de commande* \ #issue_date],
    [*Référence* \ #order_ref],
    [*Devise* \ #get("currency", default: "EUR")],
  ),
)

#v(12pt)

// Lignes commandées
#text(weight: "bold", size: 11pt, "Articles commandés")
#v(4pt)
#let lines = get("lines", default: ())
#table(
  columns: (auto, 1fr, auto, auto, auto, auto),
  fill: (col, row) => if row == 0 { rgb("#1a472a") } else { none },
  table.header(
    text(fill: white, weight: "bold", "#"),
    text(fill: white, weight: "bold", "Désignation"),
    text(fill: white, weight: "bold", align(right)[Qté]),
    text(fill: white, weight: "bold", align(right)[PU HT]),
    text(fill: white, weight: "bold", align(right)[TVA %]),
    text(fill: white, weight: "bold", align(right)[Total HT]),
  ),
  ..if lines.len() == 0 {
    (
      align(center, "1"),
      "Article principal",
      align(right, "1"),
      align(right, str(get("total_ht", default: 0))),
      align(right, "20.00"),
      align(right, str(get("total_ht", default: 0))),
    )
  } else {
    lines.enumerate().map(((i, l)) => (
      align(center, str(i + 1)),
      l.at("item_name", default: l.at("name", default: "—")),
      align(right, str(l.at("quantity", default: ""))),
      align(right, str(l.at("price", default: ""))),
      align(right, str(l.at("tax_percent", default: 20.0))),
      align(right, str(l.at("line_net_amount", default: ""))),
    )).flatten()
  }
)

#v(8pt)

// Totaux
#align(right,
  box(
    width: 50%,
    inset: 8pt,
    radius: 3pt,
    stroke: 1pt + rgb("#1a472a"),
    grid(
      columns: (1fr, auto),
      row-gutter: 4pt,
      [Total HT], align(right)[#str(get("total_ht", default: 0)) €],
      [TVA (20 %)], align(right)[#str(get("total_tax", default: 0)) €],
      text(weight: "bold")[*Total TTC*], align(right)[*#str(get("total_ttc", default: 0)) €*],
    ),
  ),
)

#v(20pt)

// Conditions
#box(
  width: 100%,
  inset: 8pt,
  radius: 3pt,
  fill: luma(250),
  stroke: 0.5pt + luma(200),
  {
    text(weight: "bold", size: 9pt, "Conditions générales")
    linebreak()
    text(size: 9pt)[
      Cette commande engage l'acheteur dans les conditions définies au contrat-cadre.
      Une facture devra être émise par le fournisseur après livraison/exécution,
      en référençant la présente commande (BT-13).
    ]
  },
)

#v(20pt)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 24pt,
  [
    #text(size: 9pt, "Visa donneur d'ordre")
    #v(40pt)
    #line(length: 100%, stroke: 0.3pt + luma(120))
    #text(size: 8pt, fill: luma(120))[Date et signature]
  ],
  [
    #text(size: 9pt, "Accusé fournisseur")
    #v(40pt)
    #line(length: 100%, stroke: 0.3pt + luma(120))
    #text(size: 8pt, fill: luma(120))[Date et signature]
  ],
)
