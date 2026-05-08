// Bordereau de livraison — template Typst rendu en PNG par
// `pdp tools gen-attachments`. Usage typique : pièce jointe d'une facture
// pour matérialiser la livraison effective de la marchandise/prestation.

#let data = json.decode(sys.inputs.at("invoice-data", default: "{}"))
#let get(key, default: none) = data.at(key, default: default)

#set page(paper: "a5", margin: (x: 1.2cm, y: 1.2cm))
#set text(font: ("Source Sans Pro", "Helvetica"), size: 9pt)

#let seller = get("seller_name", default: "Fournisseur")
#let buyer = get("buyer_name", default: "Acheteur")
#let buyer_line1 = get("buyer_address_line1")
#let buyer_postal = get("buyer_address_postal_code")
#let buyer_city = get("buyer_address_city")
#let invoice_no = get("invoice_number", default: "—")
#let issue_date = get("issue_date", default: "—")
#let bl_no = "BL-" + invoice_no
#let lines = get("lines", default: ())

#align(center, text(size: 16pt, weight: "bold", fill: rgb("#0d47a1"))[BORDEREAU DE LIVRAISON])
#align(center, text(size: 10pt)[N° #bl_no])
#v(8pt)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 8pt,
  box(
    width: 100%,
    inset: 6pt,
    radius: 3pt,
    stroke: 0.4pt + luma(180),
    {
      text(weight: "bold", size: 8pt, "Expéditeur")
      linebreak()
      text(size: 9pt, weight: "bold", seller)
      linebreak()
      text(size: 8pt)[SIRET : #get("seller_siret", default: "—")]
    },
  ),
  box(
    width: 100%,
    inset: 6pt,
    radius: 3pt,
    stroke: 0.4pt + luma(180),
    fill: rgb("#e3f2fd"),
    {
      text(weight: "bold", size: 8pt, "Destinataire")
      linebreak()
      text(size: 9pt, weight: "bold", buyer)
      linebreak()
      if buyer_line1 != none [
        #text(size: 8pt)[#buyer_line1] \
      ]
      if buyer_postal != none [
        #text(size: 8pt)[#buyer_postal #buyer_city] \
      ]
      text(size: 8pt)[SIRET : #get("buyer_siret", default: "—")]
    },
  ),
)

#v(8pt)

#box(
  width: 100%,
  inset: 5pt,
  fill: luma(245),
  grid(
    columns: (1fr, 1fr, 1fr),
    text(size: 8pt)[*Date livraison* \ #get("delivery_date", default: issue_date)],
    text(size: 8pt)[*Réf. facture* \ #invoice_no],
    text(size: 8pt)[*Mode* \ Livraison directe],
  ),
)

#v(8pt)

#text(weight: "bold", size: 10pt, "Articles livrés")
#v(2pt)
#table(
  columns: (auto, 1fr, auto, auto),
  fill: (col, row) => if row == 0 { rgb("#0d47a1") } else { none },
  table.header(
    text(fill: white, size: 8pt, weight: "bold", "#"),
    text(fill: white, size: 8pt, weight: "bold", "Désignation"),
    text(fill: white, size: 8pt, weight: "bold", align(right)[Qté]),
    text(fill: white, size: 8pt, weight: "bold", align(center)[État]),
  ),
  ..if lines.len() == 0 {
    (
      text(size: 8pt, align(center, "1")),
      text(size: 8pt, "Article principal"),
      text(size: 8pt, align(right, "1")),
      text(size: 8pt, align(center, "✓ Conforme")),
    )
  } else {
    lines.enumerate().map(((i, l)) => (
      text(size: 8pt, align(center, str(i + 1))),
      text(size: 8pt, l.at("item_name", default: l.at("name", default: "—"))),
      text(size: 8pt, align(right, str(l.at("quantity", default: "")))),
      text(size: 8pt, align(center, "✓ Conforme")),
    )).flatten()
  }
)

#v(12pt)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 16pt,
  [
    #text(size: 8pt, "Signature expéditeur")
    #v(20pt)
    #line(length: 100%, stroke: 0.3pt + luma(120))
  ],
  [
    #text(size: 8pt, "Signature destinataire")
    #v(20pt)
    #line(length: 100%, stroke: 0.3pt + luma(120))
  ],
)

#v(4pt)
#align(center, text(size: 7pt, fill: luma(120))[
  Bordereau émis le #issue_date — pièce jointe à la facture #invoice_no
])
