// Template de facture française conforme EN16931
// Les données sont passées via sys.inputs sous forme de paires clé=valeur JSON

// --- Paramètres d'entrée ---
#let data = json.decode(sys.inputs.at("invoice-data", default: "{}"))

// --- Fonctions utilitaires ---
#let fmt-amount(val) = {
  if val == none { "—" }
  else {
    let n = if type(val) == str { float(val) } else { val }
    // Formatage français : 2 décimales, virgule
    let abs = calc.abs(n)
    let int-part = calc.floor(abs)
    let dec-part = calc.round((abs - int-part) * 100)
    let dec-str = if dec-part < 10 { "0" + str(int(dec-part)) } else { str(int(dec-part)) }
    // Séparateur de milliers
    let int-str = str(int(int-part))
    let formatted = ""
    let len = int-str.len()
    for (i, c) in int-str.codepoints().enumerate() {
      if i > 0 and calc.rem(len - i, 3) == 0 { formatted += "\u{202F}" }
      formatted += c
    }
    let sign = if n < 0 { "−" } else { "" }
    sign + formatted + "," + dec-str
  }
}

#let fmt-percent(val) = {
  if val == none { "—" }
  else {
    let n = if type(val) == str { float(val) } else { val }
    let abs = calc.abs(n)
    let int-part = calc.floor(abs)
    let dec-part = calc.round((abs - int-part) * 100)
    if dec-part == 0 { str(int(int-part)) + " %" }
    else {
      let dec-str = if dec-part < 10 { "0" + str(int(dec-part)) } else { str(int(dec-part)) }
      str(int(int-part)) + "," + dec-str + " %"
    }
  }
}

#let fmt-qty(val) = {
  if val == none { "—" }
  else {
    let n = if type(val) == str { float(val) } else { val }
    if n == calc.floor(n) { str(int(n)) }
    else {
      let abs = calc.abs(n)
      let int-part = calc.floor(abs)
      let dec-part = calc.round((abs - int-part) * 1000)
      // Supprimer les zéros finaux
      let dec-str = str(int(dec-part))
      while dec-str.ends-with("0") and dec-str.len() > 1 { dec-str = dec-str.slice(0, -1) }
      str(int(int-part)) + "," + dec-str
    }
  }
}

#let get(key, default: none) = data.at(key, default: default)
#let currency = get("currency", default: "EUR")
#let cur = if currency == "EUR" { "€" } else { currency }

// --- Adresse formatée ---
#let fmt-address(prefix) = {
  let parts = ()
  let l1 = get(prefix + "_line1")
  let l2 = get(prefix + "_line2")
  let l3 = get(prefix + "_line3")
  let pc = get(prefix + "_postal_code")
  let city = get(prefix + "_city")
  let country = get(prefix + "_country_code")
  if l1 != none { parts.push(l1) }
  if l2 != none { parts.push(l2) }
  if l3 != none { parts.push(l3) }
  let city-line = ""
  if pc != none { city-line += pc }
  if city != none { city-line += " " + city }
  if city-line.trim() != "" { parts.push(city-line.trim()) }
  if country != none and country != "FR" { parts.push(country) }
  parts.join("\n")
}

// --- Mise en page ---
#set page(
  paper: "a4",
  margin: (top: 25mm, bottom: 25mm, left: 20mm, right: 20mm),
  footer: context {
    let page-num = counter(page).get().first()
    let total = counter(page).final().first()
    set text(size: 8pt, fill: luma(120))
    grid(
      columns: (1fr, 1fr),
      align(left)[#get("seller_name", default: "") — SIRET #get("seller_siret", default: "") — TVA #get("seller_vat_id", default: "")],
      align(right)[Page #page-num / #total],
    )
  }
)

#set text(font: "Source Sans Pro", size: 10pt)
#set par(leading: 0.6em)

// ========== EN-TÊTE ==========
#grid(
  columns: (1fr, 1fr),
  gutter: 20pt,
  // Vendeur
  {
    set text(size: 9pt)
    text(weight: "bold", size: 12pt, get("seller_name", default: ""))
    linebreak()
    if get("seller_trading_name") != none {
      text(style: "italic", get("seller_trading_name"))
      linebreak()
    }
    let addr = fmt-address("seller_address")
    if addr != "" { text(addr); linebreak() }
    if get("seller_siret") != none { text("SIRET : " + get("seller_siret")); linebreak() }
    if get("seller_vat_id") != none { text("TVA : " + get("seller_vat_id")); linebreak() }
  },
  // Titre facture
  {
    set align(right)
    let type-code = get("invoice_type_code", default: "380")
    let title = if type-code == "381" or type-code == "261" or type-code == "262" or type-code == "396" or type-code == "502" or type-code == "503" {
      "AVOIR"
    } else if type-code == "384" or type-code == "471" or type-code == "472" or type-code == "473" {
      "FACTURE RECTIFICATIVE"
    } else if type-code == "386" or type-code == "500" {
      "FACTURE D'ACOMPTE"
    } else {
      "FACTURE"
    }
    text(weight: "bold", size: 18pt, fill: rgb("#2c3e50"), title)
    linebreak()
    v(4pt)
    text(size: 12pt, weight: "bold", "N° " + get("invoice_number", default: "—"))
    linebreak()
    v(2pt)
    if get("issue_date") != none { text("Date : " + get("issue_date")); linebreak() }
    if get("due_date") != none { text("Échéance : " + get("due_date")); linebreak() }
    if get("order_reference") != none { text("Réf. commande : " + get("order_reference")); linebreak() }
    if get("contract_reference") != none { text("Réf. contrat : " + get("contract_reference")); linebreak() }
    if get("preceding_invoice_reference") != none { text("Réf. facture précédente : " + get("preceding_invoice_reference")); linebreak() }
  }
)

v(10pt)

// ========== ACHETEUR ==========
#box(
  width: 100%,
  inset: 10pt,
  radius: 4pt,
  stroke: 0.5pt + luma(180),
  {
    grid(
      columns: (1fr, 1fr),
      gutter: 20pt,
      // Acheteur
      {
        text(weight: "bold", size: 10pt, fill: rgb("#2c3e50"), "ACHETEUR")
        linebreak()
        v(4pt)
        text(weight: "bold", get("buyer_name", default: ""))
        linebreak()
        if get("buyer_trading_name") != none {
          text(style: "italic", get("buyer_trading_name"))
          linebreak()
        }
        let addr = fmt-address("buyer_address")
        if addr != "" { text(size: 9pt, addr); linebreak() }
        if get("buyer_siret") != none { text(size: 9pt, "SIRET : " + get("buyer_siret")); linebreak() }
        if get("buyer_vat_id") != none { text(size: 9pt, "TVA : " + get("buyer_vat_id")) }
      },
      // Livraison (si différent)
      if get("delivery_party_name") != none or get("delivery_address_line1") != none {
        {
          text(weight: "bold", size: 10pt, fill: rgb("#2c3e50"), "LIVRAISON")
          linebreak()
          v(4pt)
          if get("delivery_party_name") != none { text(weight: "bold", get("delivery_party_name")); linebreak() }
          let addr = fmt-address("delivery_address")
          if addr != "" { text(size: 9pt, addr); linebreak() }
          if get("delivery_date") != none { text(size: 9pt, "Date : " + get("delivery_date")) }
        }
      }
    )
  }
)

// ========== PÉRIODE ==========
#if get("invoice_period_start") != none or get("invoice_period_end") != none {
  v(6pt)
  text(size: 9pt, style: "italic",
    "Période de facturation : " +
    get("invoice_period_start", default: "—") + " au " +
    get("invoice_period_end", default: "—")
  )
}

v(12pt)

// ========== LIGNES DE FACTURE ==========
#let lines = get("lines", default: ())

#let header-fill = rgb("#2c3e50")

#table(
  columns: (auto, 1fr, auto, auto, auto, auto, auto),
  align: (center, left, right, center, right, right, right),
  stroke: none,
  inset: (x: 6pt, y: 5pt),

  // En-tête
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "N°")),
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "Désignation")),
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "Qté")),
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "Unité")),
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "P.U. HT")),
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "TVA")),
  table.cell(fill: header-fill, text(fill: white, weight: "bold", size: 9pt, "Total HT")),

  // Lignes
  ..for (i, line) in lines.enumerate() {
    let bg = if calc.rem(i, 2) == 0 { luma(245) } else { white }
    let line-id = line.at("line_id", default: str(i + 1))
    let name = line.at("item_name", default: "—")
    let desc = line.at("item_description", default: none)
    let qty = line.at("quantity", default: none)
    let unit = line.at("unit_code", default: "")
    let price = line.at("price", default: none)
    let tax-pct = line.at("tax_percent", default: none)
    let net = line.at("line_net_amount", default: none)

    (
      table.cell(fill: bg, text(size: 9pt, line-id)),
      table.cell(fill: bg, {
        text(size: 9pt, name)
        if desc != none { linebreak(); text(size: 8pt, fill: luma(100), desc) }
      }),
      table.cell(fill: bg, text(size: 9pt, fmt-qty(qty))),
      table.cell(fill: bg, text(size: 9pt, unit)),
      table.cell(fill: bg, text(size: 9pt, fmt-amount(price))),
      table.cell(fill: bg, text(size: 9pt, fmt-percent(tax-pct))),
      table.cell(fill: bg, text(size: 9pt, fmt-amount(net))),
    )
  }
)

v(8pt)

// ========== REMISES / CHARGES DOCUMENT ==========
#let ac = get("allowance_charges", default: ())
#if ac.len() > 0 {
  for item in ac {
    let is-charge = item.at("charge_indicator", default: false)
    let label = if is-charge { "Charge" } else { "Remise" }
    let reason = item.at("reason", default: "")
    let amount = item.at("amount", default: none)
    text(size: 9pt, label + if reason != "" { " — " + reason } else { "" } + " : " + fmt-amount(amount) + " " + cur)
    linebreak()
  }
  v(6pt)
}

// ========== TOTAUX + TVA ==========
#grid(
  columns: (1fr, auto),
  gutter: 20pt,
  // Ventilation TVA
  {
    let tax-bd = get("tax_breakdowns", default: ())
    if tax-bd.len() > 0 {
      text(weight: "bold", size: 10pt, fill: rgb("#2c3e50"), "Ventilation TVA")
      v(4pt)
      table(
        columns: (auto, auto, auto, auto),
        align: (left, right, right, right),
        stroke: 0.5pt + luma(200),
        inset: (x: 8pt, y: 4pt),
        table.cell(fill: luma(240), text(weight: "bold", size: 9pt, "Catégorie")),
        table.cell(fill: luma(240), text(weight: "bold", size: 9pt, "Base HT")),
        table.cell(fill: luma(240), text(weight: "bold", size: 9pt, "Taux")),
        table.cell(fill: luma(240), text(weight: "bold", size: 9pt, "TVA")),
        ..for bd in tax-bd {
          let cat = bd.at("category_code", default: "S")
          let base = bd.at("taxable_amount", default: none)
          let pct = bd.at("percent", default: none)
          let tax = bd.at("tax_amount", default: none)
          (
            text(size: 9pt, cat),
            text(size: 9pt, fmt-amount(base) + " " + cur),
            text(size: 9pt, fmt-percent(pct)),
            text(size: 9pt, fmt-amount(tax) + " " + cur),
          )
        }
      )
    }
  },
  // Bloc totaux
  {
    set text(size: 10pt)
    let w = 200pt
    box(width: w, inset: 8pt, radius: 4pt, stroke: 0.5pt + luma(180), {
      grid(
        columns: (1fr, auto),
        row-gutter: 6pt,
        text("Total HT"), align(right, text(fmt-amount(get("total_ht")) + " " + cur)),
        if get("allowance_total_amount") != none {
          text(size: 9pt, "Remises")
        },
        if get("allowance_total_amount") != none {
          align(right, text(size: 9pt, "−" + fmt-amount(get("allowance_total_amount")) + " " + cur))
        },
        if get("charge_total_amount") != none {
          text(size: 9pt, "Charges")
        },
        if get("charge_total_amount") != none {
          align(right, text(size: 9pt, "+" + fmt-amount(get("charge_total_amount")) + " " + cur))
        },
        text("Total TVA"), align(right, text(fmt-amount(get("total_tax")) + " " + cur)),
        grid.cell(colspan: 2, line(length: 100%, stroke: 1pt + rgb("#2c3e50"))),
        text(weight: "bold", size: 12pt, "Total TTC"), align(right, text(weight: "bold", size: 12pt, fmt-amount(get("total_ttc")) + " " + cur)),
        if get("prepaid_amount") != none {
          text(size: 9pt, "Acomptes versés")
        },
        if get("prepaid_amount") != none {
          align(right, text(size: 9pt, "−" + fmt-amount(get("prepaid_amount")) + " " + cur))
        },
        if get("payable_amount") != none and get("prepaid_amount") != none {
          text(weight: "bold", "Net à payer")
        },
        if get("payable_amount") != none and get("prepaid_amount") != none {
          align(right, text(weight: "bold", fmt-amount(get("payable_amount")) + " " + cur))
        },
      )
    })
  }
)

v(12pt)

// ========== PAIEMENT ==========
#if get("payment_iban") != none or get("payment_means_text") != none or get("payment_terms") != none {
  box(width: 100%, inset: 10pt, radius: 4pt, fill: luma(248), {
    text(weight: "bold", size: 10pt, fill: rgb("#2c3e50"), "Modalités de paiement")
    linebreak()
    v(4pt)
    if get("payment_means_text") != none { text(size: 9pt, get("payment_means_text")); linebreak() }
    if get("payment_iban") != none {
      text(size: 9pt, "IBAN : " + get("payment_iban"))
      if get("payment_bic") != none { text(size: 9pt, " — BIC : " + get("payment_bic")) }
      linebreak()
    }
    if get("payment_terms") != none { text(size: 9pt, style: "italic", get("payment_terms")) }
  })
  v(8pt)
}

// ========== NOTES ==========
#let notes = get("notes", default: ())
#if notes.len() > 0 {
  text(weight: "bold", size: 10pt, fill: rgb("#2c3e50"), "Notes")
  v(4pt)
  for note in notes {
    let content = note.at("content", default: "")
    let code = note.at("subject_code", default: none)
    if code != none { text(size: 9pt, weight: "bold", "[" + code + "] ") }
    text(size: 9pt, content)
    linebreak()
  }
}

// ========== MENTIONS LÉGALES ==========
#v(1fr)
#line(length: 100%, stroke: 0.5pt + luma(200))
#set text(size: 7pt, fill: luma(120))
#if get("invoice_type_code", default: "380") != "381" {
  text("En cas de retard de paiement, une pénalité de 3 fois le taux d'intérêt légal sera appliquée, ainsi qu'une indemnité forfaitaire de 40 € pour frais de recouvrement (art. L.441-10 du Code de commerce).")
}
