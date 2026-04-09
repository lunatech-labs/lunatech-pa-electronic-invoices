XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases Page1 /149
Standardization Committee 
AFNOR Electronic Invoicing
XP Z12-014
Annex A
Description of the main specific use cases
Version 1.3 of February 26th, 2026
Author: FNFE-MPE, based on the initial work of the DPFE of the DGFIP and the AIFE and in accordance with the work of the 
AFNOR Electronic Invoicing Standardization Commission, all of its members, and the working meetings of the sub-groups.
VERSION MANAGEMENT
Version No. Version Date Description of changes
V1.0 June 13, 2025 Initial version
V1.1 July 31, 2025
Following comments and corrections, editorial changes, and additions and/or 
modifications below:
Chapter 1.4: More detailed explanation of electronic invoicing addresses and their 
uses.
Chapter 1.5: Application of Standard Z12-014, Addition
Chapter 2.1: More detailed explanation of controls. Addition of the fact that the 
"Submitted" status no longer has to be transmitted to the PA-R (but still to the PPF).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page2 / 149
Version No. Version Date Description of changes
Chapter 2.4: clarification on cases of REFUSAL (second point).
Chapter 3.2.1: More detailed explanation of the management of Third Parties 
around the SELLER's and BUYER's Pivotal Platforms
Chapter 3.2.2 (Case No. 1): clarifications at the end of the chapter
Chapter 3.2.4: second NOTE added (the Third-Party PAYER can also connect to the 
SELLER's Accredited Platforms («Plateformes Agréées») ).
Chapter 3.2.5: Notes added after diagram (Figure 12).
Chapter 3.2.9 (use cases 8 to 10): addition of a chapter 3.2.9.1 focused on cash 
pooling, with the rest of the chapter focused on factoring.
Chapter 3.2.9.3 (case no. 8): Addition of a NOTE and editorial clarifications on the 
additional identifier of the factoring company. Correction of the Subject code to 
ACC (and not AAC).
Section 3.2.9.5 (case no. 10): Addition of codes for dedicated factoring statuses.
Chapter 3.2.10 (case no. 11): confirmation of the refocusing on the use of an 
electronic invoicing address for the BUYER entrusted to a PA-TR to which the thirdparty manager has access in order to manage invoices on behalf of the BUYER.
Chapter 3.2.12 (Cases 13 and 14): Addition of a chapter title 3.2.12.1 for the 
introduction. 
Section 3.2.12.2: addition of an example with a subcontractor invoice subject to 
reverse charge VAT and a Contractor invoice with VAT. 
Chapter 3.2.12.3: Some editorial corrections in the table describing the steps 
(SELLER replaced by SUBCONTRACTOR).
Chapter 3.2.12.5: editorial corrections in the paragraphs under Figure 21 (COCONTRACTOR and not SUBCONTRACTOR), THE AGENT is positioned as the SELLER'S 
AGENT (and not the BUYER'S AGENT, in particular to allow an EMO to play its role 
as the BUYER'S AGENT).
Chapter 3.2.13 (case no. 15): Addition of a paragraph specifying that the electronic 
invoicing address is entrusted to a PA-TR chosen by the BUYER with its Media 
Agency, and able to allow the Media Agency to process the invoice on behalf of the 
BUYER. Some editorial corrections.
Chapter 3.2.28: case no. 29, Single Taxable Entity: more detailed explanation at the 
end of the chapter.
Chapter 3.2.29 (case no. 30): clarifications in the NOTE.
V1.2 2025 10 31
Vocabulary correction Accredited Platform /PDP, Compatible Solution
Chapter 1.3: addition of factored invoices and multi-vendor invoices to the list
Chapter 1.4: addition of paragraphs 1.4.3 and 1.4.4
Chapter 2, insertion of 2.3 on invoices not transmitted due to absence of PA-R
Chapter 3.2.18, 19a, and 19b: invoicing mandate and self-billing, taking into 
account the seller's VAT regime
Chapter 3.2.31 case no. 32: addition of a paragraph following the ruling of June 5, 
2025 (BOI-RES-RVA- 000209-20250605

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page3 / 149
Version No. Version Date Description of changes
Chapter 3.2.32, case 33: example 2: estimated profit margin (with average margin 
rate)
Addition of Case No. 37: Joint ventures
Addition of case no. 38: Invoice with sub-lines
Addition of case no. 39: Transparent intermediary combining the sales of several 
sellers - Multi-Vendor Invoice
Addition of case no. 40: Grouped payments, netting, or compensation in the event 
of cross-purchases/sales
Addition of case no. 41: Barter companies (inter-company bartering)
Addition of case no. 42: Tax exemption management
V1.3 2026 02 26
Addition of the definition of flow 11, supplement to that of flow 10.
Chapter 1.4.2: some corrections.
Addition of Chapter 1.4.5 on the choice of electronic addresses and the 
consequences in the event of a change of PA.
Additions to Chapter 3.1 on third-party management.
Additions to case no. 2 (chapter 3.2.3).
Some corrections to case no. 13 and addition of examples of “Payment Sent” status 
and “Payment Received” status (chapter 3.2.12.3).
Some clarifications on case no. 37 Joint Ventures (chapter 3.2.26)
Additions to case no. 38 (invoices with sub-lines), chapter 3.2.27.
Additional information on case no. 41 (Barter Companies), chapter 3.2.40.
Addition of chapter 3.2.42, case no. 43: e-reporting for B2Binternational.
Addition of case 43a (chapter 3.2.42.6.1): triangular transactions.
Addition of case 43b (chapter 3.2.42.6.2): Stock transfers treated as intraCommunity deliveries.
Addition of case no. 44 (chapter 3.2.43): Transactions with entities established in 
the DROM / COM / TAAF.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page4 / 149
Table of contents
1 General overview......................................................................................................................... 8
1.1 Presentation of the invoicing process and the parties involved .............................................................. 8
1.2 Presentation of the annexes used in the use cases described in th....................................................... 10
1.3 Presentation of invoices taken into account in use cases...................................................................... 11
1.4 Presentation of electronic invoicing addresses and their link to the Directory ..................................... 11
1.4.1 What is an electronic address for the exchange of invoices and life cycle statuses? .............................. 11
1.4.2 The PPF Directory («Annuaire») and the use of electronic addresses..................................................... 11
1.4.3 What happens to a taxable entity not listed in the PPF Directory (“Annuaire”)?.................................... 13
1.4.4 How to send an invoice to a taxable entity listed in the PPF “Annuaire” but without any choice of 
an Accredited Platform ............................................................................................................................ 13
1.4.5 How to choose these email addresses and what are the consequences if you change your 
Approved Platform................................................................................................................................... 13
1.4.5.1 Electronic addresses for receiving invoices ................................................................................................. 13
1.4.5.2 Electronic addresses for receiving the life cycle statuses of issued invoices................................................ 14
1.4.5.3 Case of self-billing ....................................................................................................................................... 15
1.5 Application of Standard XP Z12-014....................................................................................................... 15
2 Descrip8on of the nominal invoice exchange use case................................................................ 15
2.1 Transmission of an invoice and life cycle ............................................................................................... 15
2.2 Rejection upon issuance......................................................................................................................... 19
2.3 Compliant invoice but not transmitted due to the recipient not choosing an Accredited Platform...... 21
2.4 Rejection upon receipt........................................................................................................................... 22
2.5 Rejection of an invoice by the BUYER (the recipient)............................................................................. 24
2.6 Management of a "Dispute" followed by a CREDIT NOTE...................................................................... 26
2.7 Management of a "Dispute" followed by a Corrected Invoice............................................................... 29
3 Descrip8on of the main use cases.............................................................................................. 31
3.1 Summary table of use cases................................................................................................................... 31
3.2 Handling of the main cases .................................................................................................................... 33
3.2.1 General..................................................................................................................................................... 33
3.2.1.1 Third-party management............................................................................................................................ 33
3.2.1.2 Access by third parties to invoices and their lifecycles................................................................................ 35
3.2.1.3 Additional life cycle statuses....................................................................................................................... 36
3.2.2 Case No. 1: Multi-order/Multi-delivery.................................................................................................... 36
3.2.3 Case 2: Invoice already paid by the BUYER or a third-party PAYER at the time the invoice is issued
................................................................................................................................................................. 37
3.2.4 Case 3: Invoice payable by a third-party PAYER known at the time of invoicing ..................................... 39
3.2.5 Case 4: Invoice payable by the buyer and partially covered by a third party known at the time of 
invoicing (subsidy, insurance, etc.) .......................................................................................................... 41
3.2.6 Case No. 5: Expenses paid by employees with invoices in the company's name .................................... 44
3.2.7 Case No. 6: Expenses paid by employees without an invoice addressed to the company (simple 
receipt or invoice made out to the employee's name and address)........................................................ 45
3.2.8 Case No. 7: Invoice following a purchase paid for with a corporate card (purchasing card)................... 45
3.2.9 Cases 8 to 10: Invoices payable to a third party (including factoring, cash pooling) ............................... 48
3.2.9.1 Cash pooling................................................................................................................................................ 48
3.2.9.2 Focus on factoring management ................................................................................................................ 48
3.2.9.3 Case No. 8: Invoice payable to a third party determined at the time of invoicing (factoring, cash 
pooling) ....................................................................................................................................................... 49
3.2.9.4 Case No. 9: Invoice payable to a third party known at the time of invoicing, who also manages the 
order/receipt, or even invoicing (Distributor / Depositary)......................................................................... 53
3.2.9.5 Case No. 10: Invoice payable to a third-party payee unknown at the time the invoice was created, 
in particular a factoring company (case of subrogation)............................................................................ 53

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page5 / 149
3.2.10 Case No. 11: Invoice to be received and processed by a third party on behalf of the BUYER ................. 57
3.2.11 Case No. 12: Transparent intermediary managing invoices for its principal BUYER................................ 60
3.2.12 Cases 13 and 14: Subcontracting and co-contracting (B2B, particularly for private works 
contracts) ................................................................................................................................................. 64
3.2.12.1 Special features of private works contracts, particularly those covered by the Public Procurement 
Code ............................................................................................................................................................ 64
3.2.12.2 Overview of subcontracting processing ...................................................................................................... 64
3.2.12.3 Case No. 13: Invoice payable by a third party: subcontracting with direct payment or payment 
delegation ................................................................................................................................................... 65
3.2.12.4 Subcontracting with direct payment (only in B2G ), for information .......................................................... 72
3.2.12.5 Case No. 14: Invoice payable by a third party: joint contracting case B2B ................................................. 72
3.2.12.6 Case of co-contracting in B2G. .................................................................................................................... 76
3.2.13 Case No. 15: Sales invoice following an order (and possible payment) by a third party on behalf 
of the BUYER ( ) ........................................................................................................................................ 76
3.2.14 Case No. 16: Expense invoice for reimbursement of the sales invoice paid by the third party............... 80
3.2.15 Case No. 17a: Invoice payable to a third party, payment intermediary (e.g., on Marketplace) .............. 80
3.2.16 Case No. 17b: Invoice payable to a third party, payment intermediary, and third-party invoicing 
under an invoicing mandate..................................................................................................................... 83
3.2.17 Case No. 18: Management of debit notes................................................................................................ 86
3.2.18 Cases 19a and 19b: Invoice issued by a third party on behalf of the SELLER under an Invoicing 
Mandate................................................................................................................................................... 86
3.2.18.1 Case No. 19a: Invoice issued by a third-party invoicing with an invoicing mandate................................... 87
3.2.18.1.1 Option 1: the SELLER and the Agent share the same e-invoicing platform for issuing invoices
............................................................................................................................................... 88
3.2.18.1.2 Option 2: The SELLER does not have access to the PA-E of the Invoicer................................ 90
3.2.18.2 Case No. 19b: Self-billing............................................................................................................................. 91
3.2.19 Cases 20 and 21: Pre-payment invoice and final invoice after advance payment................................... 94
3.2.20 Case No. 22a: Invoice paid with early payment discount for services for which VAT is payable 
upon receipt of payment........................................................................................................................ 100
3.2.21 Case No. 22b: Invoice paid with allowance for deliveries of goods (or provision of services with 
VAT option on debits) ............................................................................................................................ 102
3.2.22 Case No. 23: Self-billing flow between an individual and a professional............................................... 104
3.2.23 Case No. 24: Management of Deposit (“Arrhes”) .................................................................................. 105
3.2.24 Case No. 25: Management of vouchers and gift cards........................................................................... 105
3.2.24.1 Principle of the single-use voucher (BUU) ................................................................................................. 105
3.2.24.2 Principles of the multi-use voucher (BUM)................................................................................................ 106
3.2.25 Case No. 26: Invoices with contractual reservation clauses .................................................................. 107
3.2.26 Case No. 27: Management of toll tickets sold to a taxable entity.......................................................... 108
3.2.27 Case No. 28: Management of restaurant bills issued by a SELLER subject to tax established in 
France..................................................................................................................................................... 109
3.2.28 Case No. 29: Single Taxable Entity within the meaning of Article 256 C of the CGI............................... 111
3.2.29 Case No. 30: VAT already collected - Transactions initially processed in B2C e-reporting, subject 
to a retrospective invoice ....................................................................................................................... 111
3.2.30 Case No. 31: "Mixed" invoices mentioning a main transaction and an ancillary transaction................ 113
3.2.31 Case No. 32: Monthly payments............................................................................................................ 116
3.2.32 Case No. 33: Transactions subject to the margin scheme -profit .......................................................... 120
3.2.33 Case No. 34: Partial payment receipt and cancellation of payment receipt.......................................... 123
3.2.34 Case No. 35: Author's notes................................................................................................................... 123
3.2.35 Case No. 36: Transactions subject to professional secrecy and exchanges of sensitive data................ 123
3.2.36 Case No. 37: SEP (Sociétés en Participation).......................................................................................... 124
3.2.37 Case No. 38: Invoices with sub-lines and line groupings........................................................................ 126
3.2.38 Case No. 39: Transparent intermediary consolidating sales from multiple Sellers for the same 
buyer – Multi-Vendor Invoice ................................................................................................................ 129
3.2.38.1 Individual unit invoices and "global" invoice from the third party transparent to the BUYER.................. 130
3.2.38.2 Multi-Vendor Invoices............................................................................................................................... 131
3.2.39 Case No. 40: Grouped payments, netting, or compensation in the event of cross-purchases/sales
............................................................................................................................................................... 136
3.2.40 Case No. 41 Barter Companies............................................................................................................... 137
3.2.41 Case No. 42: Tax exemption management ............................................................................................ 138
3.2.41.1 Case 1: Initial domestic B2C sale............................................................................................................... 139

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page6 / 149
3.2.41.2 Case 2: B2C sale with VAT not applicable in France.................................................................................. 139
3.2.41.3 Case 3: Tax exemption carried out entirely by a tax exemption operator................................................. 140
3.2.42 Case No. 43: E-reporting for international B2B...................................................................................... 141
3.2.42.1 What are the specific features of an international B2B invoice compared to a domestic B2B invoice?
.................................................................................................................................................................. 141
3.2.42.2 International B2B sales ............................................................................................................................. 142
3.2.42.3 International B2B acquisitions .................................................................................................................. 144
3.2.42.4 Payments received on international B2B sales.......................................................................................... 145
3.2.42.5 Obligations of Accredited Platforms (“Plateformes Agréées”) and VAT Registered entities..................... 146
3.2.42.5.1 Case No. 43a: Triangular transactions................................................................................ 146
3.2.42.5.2 Case No. 43b: Stock transfers treated as intra-Community supply ..................................... 147
3.2.43 Case No. 44: Transactions with entities established in the DROMs/COMs/TAAFs................................ 148

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page7 / 149
Table of figures
Figure1: Illustration of the life cycle and different statuses................................................................................................... 16
Figure2: Nominal case of invoice exchange ........................................................................................................................... 19
Figure3: Rejection upon issuance of an e-invoice.................................................................................................................. 20
Figure4: Invoice filed as "NOT_TRANSMITTED" due to lack of PA-R...................................................................................... 22
Figure5: Rejection of an invoice upon receipt ....................................................................................................................... 23
Figure6: Rejection of an invoice by the BUYER (recipient of the invoice).............................................................................. 25
Figure7: Invoice in dispute, followed by a partial or total CREDIT NOTE ............................................................................... 28
Figure8: Invoice in dispute, followed by a Corrected Invoice ................................................................................................ 30
Figure9: Organization of third parties around the BUYER's and SELLER's Accredited Platforms («Plateformes 
Agréées») ........................................................................................................................................................... 34
Figure10: Transparent third-party invoicing and third-party authorized agent..................................................................... 35
Figure11: Invoice already paid by the BUYER or a third-party PAYER.................................................................................... 38
Figure12: Invoice to be paid by a third party designated for invoicing.................................................................................. 40
Figure13: Invoice payable by the buyer and partially covered by a third party known at the time of invoicing (subsidy, 
insurance, etc.)................................................................................................................................................... 43
Figure14: Expenses paid by an employee, invoice in the name of the company................................................................... 44
Figure15: Expenses paid by an employee, invoice in the employee's name ......................................................................... 45
Figure16: Invoice following a purchase with a corporate card .............................................................................................. 47
Figure17: Invoice payable to a third party determined at the time of invoicing ................................................................... 52
Figure18: Invoice payable to a third party unknown at the time of invoicing ....................................................................... 55
Figure19: Invoice to be processed by a third-party manager other than the BUYER ............................................................ 59
Figure20: Transparent intermediary for the main BUYER with the BUYER's dedicated electronic invoicing address 
entrusted to a PA-TR of the transparent intermediary ...................................................................................... 63
Figure 21 : Subcontractor invoice (F1) to be paid by a third party and main invoice F2 from the CONTRACTOR 
(TITULAIRE) to the end BUYER (case of subcontracting with payment delegation)........................................... 71
Figure22: Invoice payable by a third party (case of co-contracting in B2B)........................................................................... 75
Figure23: Sales invoice following an order (and possibly payment) by a third party on behalf of the BUYER, example 
with invoices for the purchase of advertising space. ......................................................................................... 79
Figure24: Invoice payable to a third party, payment intermediary ....................................................................................... 82
Figure25: Invoice payable to a third party, payment intermediary, and third-party invoicing under an invoicing 
mandate. ............................................................................................................................................................ 85
Figure26: Invoice issued with invoicing mandate (option 1).................................................................................................. 89
Figure27: Invoice issued with invoicing mandate (option 2).................................................................................................. 90
Figure28: Self-billing............................................................................................................................................................... 93
Figure29: prepayment invoice after advance payment already paid and final invoice ......................................................... 98
Figure30: prepayment invoice and final invoice .................................................................................................................... 99
Figure31: Invoice paid with allowance (case of service provision, VAT due upon receipt) ................................................. 101
Figure32: Invoice paid with allowance (case of delivery of goods or provision of services with VAT on debits) ................ 104
Figure33: Management of single-use vouchers for the provision of services to an individual by a third-party service 
provider............................................................................................................................................................ 106
Figure34: Management of a Multi-Purpose Voucher for the provision of services to an individual by a third-party 
supplier............................................................................................................................................................. 107
Figure35: Management of restaurant bills for a non-taxable customer.............................................................................. 109
Figure36: Transaction data declaration for bills under €150 ............................................................................................... 110
Figure37: Issuing/transmitting an electronic invoice for bills exceeding €150 .................................................................... 110
Figure38: Electronic invoice management following a sale that has been subject to e-reporting of the transaction 
(B2C)................................................................................................................................................................. 113

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page8 / 149
Figure39: Mixed invoice with main and ancillary transactions categorized as a sale (case of an alteration on sale).......... 114
Figure40: E-reporting sale of transaction with INDEPENDENT Provision of Services AND Delivery of Goods..................... 115
Figure41: Monthly payments and additional amount payable in a B2C context, no direct debit option (option 1a) ......... 117
Figure42: Monthly payments in the context of a B2C sale, option for debits and additional amount to be paid (case 
1b) .................................................................................................................................................................... 118
Figure43: Monthly payments and final overpayment in a B2C transaction, no option for debits (option 2a) .................... 119
Figure44: Monthly payments and final overpayment in a B2C transaction, option for debits (case 2b) ............................ 120
Figure45: Transactions subject to the margin scheme, B2B invoice subject to e-invoicing................................................. 121
Figure46: Management of sales and purchase invoices on behalf of an SEP (joint venture) .............................................. 125
Figure47: Invoice for a kit with details of its composition using INFORMATION sub-lines.................................................. 127
Figure48: Invoice for a toy book where the price is determined on the sub-items............................................................. 128
Figure49: Invoice with a main line and supplements attached to it.................................................................................... 128
Figure50: Invoice for a composite item with 2 levels........................................................................................................... 129
Figure51: Transparent third party acting as BUYER'S AGENT .............................................................................................. 131
Figure52: Example of a multi-vendor invoice....................................................................................................................... 133
Figure53: Illustration of an invoicing cycle through a barter company................................................................................ 137
Preliminary note 
As part of the reform, the term "Partner Dematerialization Platform (PDP)" has been replaced by "Accredited Platform 
("Plateforme Agréée") (PA)" and the term "Dematerialization Operator (OD)" has been replaced by "Compatible Solution 
(SC)".
1 General overview
1.1 Presentation of the invoicing process and the parties involved
Following the decision to refocus the functions of the PPF (Public Invoicing Portal) on managing the Directory and the Data 
Concentrator, B2B invoice exchanges between taxable entities are now carried out exclusively through Accredited Platforms 
(«Plateformes Agréées»). This corresponds to Circuit C as described in the document on use cases for external specifications, 
version 2.4.
As a reminder, B2G exchanges are carried out via the CHORUSPRO Platform, which will remain the platform for public 
entities subject to invoice receipt obligations in the public sector. However, private sector invoice issuers will also be able 
to use their Accredited Platforms («Plateformes Agréées») to issue invoices to the public sector. In addition, certain complex 
use cases involve exchanges between private actors associated with a public actor (co-contracting, subcontracting). This 
document will therefore also describe these use cases.
This document is intended to provide guidance on various use cases that correspond to business practices, generally on a 
sector-by sector basis. The objective is to propose best practices for implementation, whether in terms of how to implement 
invoicing data in the minimum base formats, which are the subject of a separate publication, how to use life cycle statuses 
in the dynamics of exchanges and processing, but also to define a minimum functional base that Accredited Platforms 
(«Plateformes Agréées») are required to implement.
This document therefore distinguishes between the obligations of Accredited Platforms («Plateformes Agréées») and the 
optional best practice features for ecosystem players, namely Accredited Platforms («Plateformes Agréées»), management 
solution publishers involved in invoice management, taxable entities, third parties, and more generally everything included 
in the term OD (Dematerialization Operator)/SC (Compatible Solution).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page9 / 149
This document is based on the use of the minimum base formats and profiles as described in standard XP Z12-012, in 
addition to the external specifications that describe the specifications of the interfaces between the PPF (Directory and Data 
Concentrator) and the Accredited Platforms («Plateformes Agréées»), including the description of flows 1, 10, as well as the 
use of flow 6 of the Life Cycle applied only to exchanges between Accredited Platforms («Plateformes Agréées») and the 
PPF.
As a reminder, standard XP Z12-012 describes the use of semantic standard EN16931 in the context of the reform through 
two main profiles (EN16931 and EXTENDED-CTC-FR) and a temporary profile implemented in the Factur-x hybrid format 
(BASIC WL). Additional management rules have been defined to comply with the requirements of the reform, as well as 
rules for extracting and transforming structured invoice data to constitute the data required by the tax authorities (flow 1 
and flow 10.1). These profiles are then implemented in three formats, namely:
• UBL 2.1.
• UN/CEFACT CII D22B.
• Factur-X, which is an hybrid format consisting of a PDF/A-3 representation (ISO 19005-3), to which a file containing 
the essential invoice data in UN/CEFACT CII D22B format is attached.
Finally, the standard also describes how the CDAR (Cross Domain Acknowledgment and Response) life cycle status message 
should be used by end users (entities subject to VAT in France) and their Accredited Platforms («Plateformes Agréées»), as 
well as between Accredited Platforms («Plateformes Agréées») themselves.
The use cases covered in this document describe how invoice data fits into the XP Z12-012 standard data model, whose life 
cycle statuses are exchanged, whether in the invoice transmission or processing phase, as well as the dynamics of exchanges 
between the various parties involved (SELLER, BUYERS, their Accredited Platforms («Plateformes Agréées») and information 
systems, including OD/SC (Compatible Solution), as well as various third parties and their Accredited Platforms 
(«Plateformes Agréées») /OD/SC (Compatible Solution)).
These use cases can be divided into three main categories, bearing in mind that some fall into several categories:
• Use cases requiring instructions for implementing invoice data, where applicable requiring additional data, and/or 
a relaxation of certain management rules, i.e., the use of the EXTENDED-CTC-FR profile.
• Use cases involving third parties, which means addressing the issues of invoice sharing and life cycle with these 
third parties.
• Use cases impacting the life cycle, generally due to the involvement of third parties, but also primarily the 
management of rejections, refusals, disputes, etc., and anything that requires multiple invoices, corrected invoices, 
or credit notes to process the invoicing of the underlying transaction. 
In these different use cases, several parties are likely to be involved (in capital letters in the text):
• The SELLER is the party that provides the goods or services covered by an invoice. The seller records the invoice in 
their accounts (as revenue or advance payments). When the invoice includes VAT, the SELLER is responsible for 
collecting it and therefore declaring it as VAT collected (unless they use a TAX REPRESENTATIVE of the SELLER). The 
SELLER is also often referred to as the Supplier.
• The BUYER is the person who purchased the product or service. They are therefore always the ones who record the 
invoice in their accounts as the buyer (as an expense, inventory, fixed asset, or advance payment). When the invoice 
includes VAT, it is the BUYER who can deduct it, and it is therefore the BUYER who declares the deductible VAT. In 
most cases, it is the BUYER who pays the invoice. The BUYER may also be referred to as the Customer.
• THE SELLER'S TAX REPRESENTATIVE, who represents the SELLER vis-à-vis the tax authorities if the SELLER does not 
represent themselves, primarily in relation to VAT returns.
• THE PAYEE, who is the recipient of the invoice payment when it is not the SELLER. An invoice may have several 
PAYEES (only in the EXTENDED-CTC-FR profile). For example, a factoring company is a PAYEE.
• Other Parties have been deemed useful for addressing certain use cases. They are not present in the EN16931
semantic model for essential invoice data, but have been added to the EXTENDED-CTC-FR profile, which is part of 
the Minimum Base that taxable entities and Accredited Platforms («Plateformes Agréées») MUST be able to 
process:
ü The INVOICER is a third party who creates and issues the invoice on behalf of the SELLER. They must do so under 
the SELLER's invoicing mandate, in accordance with the regulations in force (see BOI-TVA-DECLA-30-20-10, 
articles 340 to 500).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page10 / 149
ü The SELLER'S AGENT, who acts on behalf of the SELLER, for example when they are in charge of sales and 
sometimes invoicing (in which case they are also the INVOICER), or even payment receipt.
ü The ADDRESSEE, more accurately referred to in the standards as the "Billed to," is the Party to whom the invoice 
is sent because they are responsible for processing it on behalf of the BUYER. However, the use of multiple 
electronic invoicing addresses for receiving invoices makes it possible to avoid using this ability to send invoices 
to a third party, but simply to allow that third party to process invoices addressed to the BUYER at an electronic 
invoicing address (an invoice reception mailbox) whose management is entrusted to that third party by the 
BUYER. However, when the ADDRESSEE is named on the invoice, this allows the PA-R (receiving party) to 
manage delegation rights in a more targeted manner to allow it to access the invoice and the processing actions 
for which it has been delegated. This third party may also enable compliance with the requirements of Article 
441-9 of the French Commercial Code, which requires that the postal address of the entity that receives and 
processes the invoice on behalf of the BUYER be provided (invoicing address if different from the customer's 
(BUYER's) address, which should be interpreted as the postal invoicing address).
ü The BUYER'S AGENT, who acts on behalf of the BUYER, often because they were in charge of the ordering phase 
and can therefore set the processing status (approval, dispute, etc.).
ü The PAYER, who is the payer of the invoice when it is not the BUYER. For example, when a company within a 
group is responsible for payments for all of the group's subsidiaries, or in the case of a subcontracting invoice 
with direct payment from the end customer. There may also be several PAYERS (for example, an invoice that is 
paid in part by an insurer and in part by the BUYER).
• Accredited Platforms («Plateformes Agréées») are the only ones authorized (and registered for this purpose) to 
exchange invoices falling within the scope of "e-invoicing" scope (domestic B2B invoices between VAT-registered 
entities in France or B2G/G2B between a VAT-registered entity and a public body), and to transmit flows 1, flows 
10 (e-reporting) and so-called "mandatory" statuses to the PPF Data Concentrator.
• OD/SC (Compatible Solutions) (Dematerialization Operators) are service providers or management solutions 
upstream and downstream of invoice exchange and life cycles, and can act as an interface between the various 
Parties described above and the Accredited Platforms («Plateformes Agréées»).
• The PPF (Public Invoicing Portal) is responsible for the Directory of Taxable Entities and their electronic invoicing 
addresses and the Data Concentrator (CdD PPF), i.e., for receiving Flow 1, Flow 10, and Flow 6 for Mandatory 
statuses (Submitted, Rejected, Refused, Received).
1.2 Presentation of the annexes used in the use cases described in th 
The use cases described in this document are based on the documents describing invoice formats and life cycle messages 
published by AFNOR (XP Z12-012). The semantic format of the invoice is described according to two profiles:
• An EN16931 profile compliant with the EN16931 standard, to which a few additional management rules have been 
added to take into account the management rules defined in the annexesto the external specifications version 3.1.
• An EXTENDED-CTC-FR profile, which is an extension as permitted by the EN16931 standard, containing additional 
data referenced using naming conventions such as "EXT-FR-FE-BG-XX" for data blocks and "EXT-FR-FE-XX" for data. 
In addition, certain management rules in the EN16931 standard have been removed or replaced by more flexible 
rules for this profile.
Reference may also be made to the annexes of the external specifications in force published on the impots.gouv.fr website: 
• Annex 1 (Excel format): this annex defines the semantic format of Data Flow 1 required by the tax authorities.
• Annex 2 (Excel format): this annex defines the semantic format of the life cycle of flows and business objects 
(Directory, e-reporting, e-invoicing) and its use in exchanges between Accredited Platforms («Plateformes 
Agréées») and the PPF.
• Annex 3 (Excel format): this annex defines the semantic format of the directory used to create or modify entries in 
the directory of VAT-registered entities, i.e. the electronic invoicing addresses of invoice recipients.
• Annex 6 (Excel format): this annex defines the semantic format for e-reporting of transaction and payment data.
• Annex 7 (Excel format): this annex defines all the management rules for all business objects exchanged between 
the Accredited Platforms («Plateformes Agréées») and the PPF described in the above annexes.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page11 / 149
1.3 Presentation of invoices taken into account in use cases
In the context of the dematerialized exchange of domestic B2B invoices, several types of invoices are taken into account 
(see management rule BR-FR-04 in the description of invoice formats):
• Simple invoices;
• pre-payment invoices;
• Factored invoices;
• Self-billed invoices (when the BUYER creates the invoice on behalf of the SELLER and sends it to them);
• Corrected invoices, which are invoices that cancel and replace a previous invoice. They are therefore both a credit 
note on the previous referenced invoice and a new invoice;
• Credit notes.
• Multi-vendor invoices, only with the EXTENDED-CTC-FR profile (or EXTENDED from Factur-X), which is an invoice 
created by a transparent intermediary on behalf of several SELLERS and sent to a single BUYER, with payment 
centralized to a single payee (usually the transparent intermediary, who can invoice their own services on the same 
invoice).
1.4 Presentation of electronic invoicing addresses and their link to the Directory
All Parties involved in an invoice have an electronic address, which is to the electronic invoice what the postal address is to 
the paper invoice, i.e., data that allows an electronic message (an invoice, a life cycle status) to be sent to the party 
referencing its electronic address.
1.4.1 What is an electronic address for the exchange of invoices and life cycle statuses?
These electronic addresses are composed of an addressing identifier belonging to a given repository, qualified through an 
identification scheme (SchemeID), to be chosen from a list: the EAS (Electronic Address Scheme) list, available in Standard 
EN16931 and referenced in Annex A of Standard XP Z12-012. Some electronic addresses are linked to different exchange 
protocols, such as the "EM" schemeID for an SMTP address (an email: xxx@nomdedomaine.xx ), or "AQ" for an X400 
electronic address. Most other types can be used in point-to-point exchanges between platforms and, more generally, in 
interoperable network exchanges such as PEPPOL.
These addresses are therefore often written in the form "schemeID:Address_Identifier," for example:
• 0088:GLN: address using a GLN-type addressing identifier.
• 0060:DUNS: address using a DUNS-type addressing identifier.
• 9957:TVAINTRAFR: address using an addressing identifier corresponding to a French intra-community VAT number.
• 0199:GLEIF: address using a Global Legal Entity Identifier address identifier.
• …
• And 0225:SIREN or 0225:SIREN_XXX: address using a SIREN (legal identifier in France) or SIREN_XXX addressing 
identifier, where XXX is either free (and is called SUFFIX) or in SIRET or SIRET_CODE_ROUTAGE form, provided that 
the SIRET and CODE_ROUTAGE are present in the PPF Directory («Annuaire»).
1.4.2 The PPF Directory («Annuaire») and the use of electronic addresses
The PPF Directory («Annuaire») contains ALL electronic addresses under schemeID 0225 intended to receive invoices for 
entities SUBJECT to VAT or public entities. Thus, if an electronic address is present in the PPF Directory («Annuaire»), it 
means that the recipient is subject to VAT or is a public entity. There are therefore entities with a SIREN number (such as 
associations) that may not be subject to VAT and will not be listed in the PPF Directory («Annuaire»), nor will any of their 
electronic addresses of the type 0225:SIREN_XXX.
It is therefore also possible that there are electronic addresses under schemeID 0225, i.e., beginning with a SIREN number, 
that are not in the PPF Directory («Annuaire»). Beyond the above example of a non-VAT-registered association with a SIREN 
number, a VAT-registered company that wishes to channel the life cycle status of its issued invoices can enter a 
0225:SIREN_STATUS in its invoice for itself (BT-34 for the SELLER, for example), without having referenced it in the PPF 
Directory («Annuaire»). It can then be used in the PEPPOL network so that the life cycle return statuses are sent to it at this 
address, but it will not be able to receive invoices falling within the scope of e-invoicing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page12 / 149
Thus, in a "traditional" invoice exchange (i.e., not self-billing): 
• The BUYER's electronic address in BT-49 is the electronic address to which the invoice must be sent.
• The SELLER's electronic address in BT-34 is the electronic address at which the SELLER wishes to receive its statuses, 
particularly when it uses interoperability between Approved Platforms (“Plateformes Agréées”) in a network for 
which electronic addresses (also known as “EndPoints”) are the keys enabling automated exchanges between 
platforms (known as “dynamic discovery”). There is no obligation for these life cycle status return addresses to be 
in the PPF Directory (“Annuaire”), or even under schemeID 0225, as only the invoice receipt addresses for VATregistered entities in France must be (and it is not necessarily desirable to receive invoices at the same address as 
these status returns for issued invoices).
• The electronic addresses of the other parties (Payee, Seller's Agent, Buyer's Agent, Payer, etc.) may be used if it is 
necessary to reply to them with statuses or invoices in order to integrate them into the invoice life cycle. They are 
not necessarily in the form 0225:SIREN_XXX (particularly for international players not established in France).
Section 1.4.5 details how to choose these addresses and the impact of changing Authorized Platforms, both for sending and 
receiving.
That said, all VAT-registered entities with a SIREN number are listed in the PPF Directory («Annuaire»). In order to enable 
the transmission of invoices through the Accredited Platforms («Plateformes Agréées») chosen by each taxable entity, 
invoice recipients (generally BUYERS, but also SELLERS in the case of self-billing) MUST define one or more electronic 
invoicing addresses under SchemeId 0225 and have them registered in the PPF Directory («Annuaire») by the PA-R they 
have designated to receive their invoices at this electronic invoicing address on their behalf. Those who practice self-billing 
may choose the same electronic invoicing address for receiving invoices as the one they use to receive their purchase 
invoices, and it is up to them and their PA-R to then distinguish between the flows. They may also dedicate an electronic 
invoicing address to self-billed invoices, for example 0225:SIREN_AUTOFACTURE.
These electronic invoicing addresses are the identifier for the address line described in Annex 3 of the external specifications 
(data DT-7-5-1), and must take one of the following forms:
• "SIREN" of the legal entity. This is the most common address used by companies (especially when they only have 
one).
• "SIREN_SIRET": if the company has chosen to create electronic invoicing addresses for one of its establishments, 
which must itself be active and listed in the PPF Directory («Annuaire»). This address structure is primarily intended 
for public sector entities that are referenced by a SIRET number.
• "SIREN_SIRET_CODEROUTAGE": if the company has chosen to create electronic addresses at a more granular level 
than its establishment. This ROUTING_CODE must be configured in advance in the PPF Directory («Annuaire») and 
must be active and linked to the SIRET number of the address. This address structure is primarily intended for public 
sector entities that may need to have "SERVICE EXECUTANT" CODES in order to distribute invoices to the correct 
departments.
• "SIREN_SUFFIX": if the company wishes to have several electronic invoicing addresses, for example to differentiate 
between purchasing channels, because it has chosen several Accredited Platforms («Plateformes Agréées»)
(bearing in mind that an electronic invoicing address can only be associated with a single Accredited Platform on 
reception side), without having to link these addresses to an internal organization (ROUTING_CODE) or the location 
of its teams (SIRET). SUFFIXES are freely chosen by the company. They do not need to be created in advance in the 
PPF Directory («Annuaire»). They are simply part of the electronic invoicing address. It is therefore this type of 
address that will be recommended and described in this document for certain use cases.
Electronic invoicing addresses for receiving invoices are listed in the PPF Directory («Annuaire»), which can be viewed 
publicly via the Accredited Platforms («Plateformes Agréées») or the PPF Portal, available for public consultation at 
https://portail.chorus-pro.gouv.fr. This means that once you know your customers' SIREN numbers, you can find all their 
electronic invoicing addresses. You then simply need to enter them on your invoices to ensure they reach your customers 
reliably and can be tracked using life cycle statuses. The electronic invoicing address is therefore essential customer data 
that needs to be managed within your information system. Even though it often takes the form of the company's SIREN 
number, it is important to distinguish between the customer's SIREN number, which is a legal identifier, and the customer's 
electronic invoicing address, even if it happens to be the same as the SIREN number.
Invoices must therefore contain the recipient's electronic invoicing address (BUYER or SELLER for self-billed invoices), as well 
as that of the issuer (SELLER or BUYER for self-billed invoices), in order to manage life cycle status returns. Thanks to this 
electronic invoicing address and the PPF Directory («Annuaire»), Accredited Platforms («Plateformes Agréées») can 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page13 / 149
interoperate automatically, in particular through the PEPPOL network under the governance of the PEPPOL France 
Authority.
All invoices that fall within the scope of "e-invoicing" MUST contain an electronic invoicing address under schemeID 0225 
for the invoice recipient, i.e., for the BUYER (BT-49) for traditional invoices, and for the SELLER (BT-34) for self-billed invoices.
For those who use the PEPPOL network, their electronic invoicing addresses are also available to receive invoices from 
entities not subject to VAT in France, starting with international entities using PEPPOL for international B2B invoices.
1.4.3 What happens to a taxable entity not listed in the PPF Directory (“Annuaire”)?
Some VAT taxable entities do not appear (or do not yet appear) in the PPF Directory (“Annuaire”). In this case, they may be 
considered “not subject to VAT” by the issuers of invoices addressed to them, which means that these issuers are no longer 
subject to the obligation to send electronic invoices to recipients and can switch to the B2C e-reporting component of the 
reform for these transactions (e-reporting of B2C transactions of daily turnover totals by transaction category).
This tolerance also applies to the penalty system: when an entity is not yet included in the recipient directory due to the 
administration's own validation procedures or technical difficulties attributable to the administration, it will not be penalized 
if it does not choose an accredited platform.
The suppliers subject to these entities will also not be penalized if they do not issue electronic invoices to entities not yet 
included in the directory. Suppliers will have to carry out e-reporting as if they were invoicing an entity not subject to tax
(flow 10.3 and, where applicable, 10.4 if VAT is due on receipt).
1.4.4 How to send an invoice to a taxable entity listed in the PPF “Annuaire” but without any choice of an Accredited 
Platform 
Some VAT taxable entities appear in the directory but without an active invoicing electronic address, i.e., one connected to 
an Accredited Platform. In this case, the issuing taxable entity remains subject to the electronic invoicing obligation using 
the electronic invoicing address listed in the directory (the one corresponding to the SIREN number). Their Accredited 
Platform can then carry out all the regulatory checks and send a "Submitted" status to the PPF, with the reason 
"NON_TRANSMISE" (NOT_SENT) in order to alert the tax authorities that the recipient has not chosen an Accredited 
Platform.
The issuer can then contact their customer to send them a duplicate invoice, for example, and get paid.
The recipient cannot demand a paper or even PDF invoice, as regulations require them to accept electronic invoices and 
choose an accredited platform. They therefore find themselves in the same situation as if they had not received a paper 
invoice, which they may have lost. They then rely on a duplicate. If necessary, if they act quickly, the Approved Issuing 
Platform may be able to resend the transmission so that their invoices reach them.
1.4.5 How to choose these email addresses and what are the consequences if you change your Approved Platform
The choice of electronic addresses depends on the company's situation, its size, whether it faces certain specific use cases 
that may require gathering related invoices, or its invoice processing organization.
1.4.5.1 Electronic addresses for receiving invoices
Firstly, for electronic addresses used to receive invoices, it is recommended that you choose one or more addresses that are 
as permanent as possible, as each change requires you to notify all the suppliers concerned, which causes a certain amount 
of disruption. As a result, for example, the SIRET network (and therefore also SIRET_CODE_ROUTAGE) does not comply with 
this principle, since a simple change of address results in a change of SIRET and should therefore be avoided.
It is therefore preferable to choose a limited number of SIREN_SUFFIXE electronic addresses, i.e., electronic addresses 
corresponding to specific processing requirements. For example, a company that has to manage significant employee 
expenses may want all resulting invoices to be sent to a dedicated electronic address, because the suppliers are not known 
in advance and because those invoices are often already paid. Similarly, a company that uses several applications to manage 
its purchase invoices (e.g., several ERPs, specialized solutions for certain purchases, etc.) may wish to use several invoice 
receipt electronic addresses as gateways for its various invoicing channels.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page14 / 149
A small business that manages few invoices can make do with a single electronic address (and therefore SIREN). A retailer 
who has several stores and wants to differentiate invoices by store can create dedicated electronic addresses with a SUFFIX 
to make them more clearly identifiable to suppliers.
It is very bad practice to multiply electronic addresses by recipient within the company or by validator or purchaser 
department, firstly because this multiplies them, but also because organizations change regularly, as do people, and 
updating an organizational repository is often complex and time-consuming.
A good illustration of these principles is to consider that electronic addresses for receiving invoices are different entry points 
into the company, and that internal circulation within departments must be based on the references present in the invoices 
(order number, contract number, delivery note, buyer references, or even, where applicable, a SIRET number as a reference 
only, which does not require changing the electronic address for a simple physical move, etc.).
In the event of a change of Accredited Platform (« Plateforme Agréée ») for the electronic address for receiving invoices, 
all invoices issued after the date of change will arrive on the new Accredited Platform (“Plateforme Agréée”), and all 
processing life cycle statuses issued by the issuer ("completed “ and ”Paiement Received") for invoices issued before the 
change of Accredited Platform (“Plateforme Agréée”) will arrive on the new Accredited Platform (“Plateforme Agréée”),
which will be responsible for forwarding them to the recipient, who will then be able to respond to them on their old 
Accredited Platform (“Plateforme Agréée”, thanks to the minimum service being maintained for 12 months after the change 
of AP, as described in the regulations).
1.4.5.2 Electronic addresses for receiving the life cycle statuses of issued invoices
When it comes to issuers' electronic addresses, which are in fact electronic addresses for receiving the life cycle statuses of 
issued invoices, it is also important not to multiply them unnecessarily, but to organize them according to customer invoicing 
processes.
It may make sense to use one or more electronic addresses dedicated to receiving the life cycle statuses of issued invoices, 
which are different from electronic addresses used to receive invoices. However, a small business which processes an 
average of fewer than 400 invoices per year, including just over a hundred issued invoices, can make do with a single 
electronic address (e.g., its SIREN number) for issuing invoices (and therefore receiving the life cycle statuses of its issued 
invoices) and for receiving its electronic invoices.
In the event of a change in the Accredited Platform (‘Plateforme Agréée”) for issuing electronic invoices, the life cycle 
statuses transmitted by the recipients (i.e., all those relating to the recipient's processing of invoices) will arrive at the 
Accredited Platform (‘Plateforme Agréée”) responsible for the issuer's electronic address (the Seller in the case of non-selfbilled invoices) entered in the invoices issued.
There are therefore three possible solutions:
• Keep the same electronic address for receiving the life cycle statuses of invoices issued after the change of 
Accredited Platform (‘Plateforme Agréée”) and entrust it to the new Accredited Platform: the statuses of invoices 
issued with the old PA-E will arrive at the new PA-E, which will be able to transmit them to the issuer of the invoices 
without reconciling them with the invoices since it will not have issued them. It may also offer a service to redirect 
these statuses to the old PA-E.
• Change the electronic address for receiving the life cycle statuses of invoices issued for new invoices and entrust it 
to the new PA-E, while keeping the old address with the old PA-E: The statuses of invoices sent with the old PA-E 
arrive at the old PA-E, and those corresponding to invoices issued with the new PA-E (and the new electronic
address for issuance) arrive at the new PA-E.
• For e-invoicing platforms which use PEPPOL, the electronic addresses for receiving return statuses are entered in 
an envelope (SBDH) in which the invoice is inserted. This e-invoicing platform may propose to replace the electronic
address of the invoice issuer in the SBDH envelope with an electronic address which it controls to ensure that it 
always receives the statuses of the invoices it processes for issuance, while ensuring that the life cycle statuses are 
reassigned to each user's invoices.
For a company which processes few invoices and has a single electronic address for sending and receiving, the first approach 
is the simplest and least disruptive in the event of a change, given the small number of invoices involved.
For a company which decides to dedicate electronic addresses for receiving, the second option is the most effective.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page15 / 149
1.4.5.3 Case of self-billing
Self-billing invoices are invoices issued by the BUYER to the SELLER, under an invoicing mandate given by the SELLER to the 
BUYER. This is often implemented in order to simplify invoice processing from the BUYER's point of view, both for the BUYER, 
who is often in agreement with himself, and for the SELLER, who does not have to create his invoices, but just to record 
them.
These invoices are clearly identified by the invoice type code (BT-3). They must then be processed by the Approved Issuing 
Platforms (PA-E) on the BUYER's side to the Approved Receiving Platforms (PA-R) of the SELLERS.
In terms of electronic invoice receipt addresses, the one in the SELLER block (in BT-34) is therefore used. The SELLER may 
decide to use the same electronic address as for receiving their purchase invoices, but it is their responsibility to clearly 
distinguish their purchase invoices from their self-billed sales invoices.
They may also decide to create a dedicated electronic address for receiving invoices (e.g., 0225:SIREN_AUTOFACTURATION), 
which they entrust to the Accredited Platform (“Plateforme Agréée”) of their choice. For a VAT Registered subject to the 
obligation to receive electronic invoices, this electronic address must be under schemeID 0225 and be listed in the PPF 
Directory (“Annuaire”). As a reminder (see use case 19b described in Annex A), it is up to the PA-E on the BUYER side to 
create flow 1 and transmit it to the PPF.
The same applies to the electronic address for receiving the life cycle statuses of issued invoices, i.e., use the same address 
as for all issued sales invoices, or dedicate an electronic address for the return statuses of issued self-billed invoices (to be 
entered in invoices in BT-49).
PLEASE NOTE: in this case, all life cycle statuses are reversed between BUYER and SELLER, except for payment statuses. The 
BUYER remains the one who transmits the “Payment Sent” status and the SELLER remains the one who transmits the 
“Payment Received” status.
1.5 Application of Standard XP Z12-014
The XP Z12-014 standard and its annexes are intended to describe the use cases identified, how they can be implemented 
within the framework of the Reform, and the respective obligations of Accredited Platforms («Plateformes Agréées»), OD/SC 
(Compatible Solutions) and companies for each use case, as well as certain additional optional features that could be 
implemented to provide added value.
The nominal case and the management of life cycle statuses and settlements with Credit Notes or Corrected Invoices are 
part of the obligations of all Accredited Platforms («Plateformes Agréées»).
However, unless specifically required by regulation, the implementation of the use cases described in Chapter 3 of Annex
A is a matter for the commercial policy of the economic actors (Accredited Platforms («Plateformes Agréées»), OD/SC ( -
Compatible Solution)), which are therefore under no obligation to be able to handle all use cases. It is also likely that some 
Accredited Platforms («Plateformes Agréées») will specialize in managing certain of these use cases. 
Nevertheless, this list of use cases constitutes a functional reference framework that can help establish comparison grids 
for the offerings of different Accredited Platforms («Plateformes Agréées»), Compatible Solutions, or other ODs.
2 Descrip2on of the nominal invoice exchange use case
2.1 Transmission of an invoice and life cycle
Before describing specific use cases, it is important to start with the nominal case of a SELLER who invoices their customer,
the BUYER, through the PA-E of the invoice issuer (in this case, the SELLER) and the PA-R of the BUYER.
This invoice exchange is accompanied by life cycle statuses, first in a transmission phase (statuses created by the Accredited 
Platforms («Plateformes Agréées») for end users to track end-to-end transmission), then in a processing phase where end 
users can create statuses to be transmitted to their counterparties through the Accredited Platforms («Plateformes 
Agréées»).
The transmission statuses (Submitted/Rejected on issue, Issued, Received/Rejected on receipt, Made available) allow the 
progress of the invoice to be tracked in this order. Processing statuses (Rejected, Disputed, Suspended, Completed, 
Approved, Partially Approved, Payment Transmitted, Cashed) can be set independently. Analysis of use cases may lead to 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page16 / 149
the addition of useful processing statuses for some of them. The first of these is a "Cancelled" status to be assigned to an 
invoice which has been corrected, either by the SELLER when issuing the Corrected Invoice, or by the BUYER when processing 
the Corrected Invoice, to indicate that this initial invoice has been replaced and to close its life cycle.
As a reminder, four of these statuses are "mandatory," meaning they must be transmitted to the CdD PPF for the tax 
authorities, namely "Submitted" (“Déposée”), "Rejected (“Rejetée”)," "Refused" (zRefusée”) and "Payment Received"
(“Encaissée”) which is necessary to prepare the VAT pre-fill.
Finally, the "Payment Received" status must be sent to the tax authorities only if VAT is due upon receipt of payment
(service invoices for which the SELLER has not opted for debits). This also applies to advance payment invoices, whether 
for goods or services, for which VAT is payable upon payment of the advance, even if the taxable entity has opted for 
debits.
It is also possible for the SELLER to transmit the "Received" status even when VAT is not due upon receipt, but in this case,
it is only transmitted to the BUYER (and not to the tax authorities via the CdD PPF).
Figure1: Illustration of the life cycle and different statuses
Analysis of use cases shows that it will be necessary to provide additional complementary statuses to support processing, 
such as "Invoiced," "Confidential Invoiced," "Not Invoiced," "Direct Payment Request," etc.
The processing of invoices issued by the PA-E and received by the PA-R involves several steps:
• Technical checks: antivirus, empty files, envelope checks, etc. In the event of an error, an UNACCEPTABLE status 
(IRRECEVABLE”, Code 501) is sent to the issuer, with the reason (see list in Annex A of standard XP Z12-012). The 
issuer can then correct the invoice and resend it.
• "Application" checks: format structure. For XML formats, this corresponds to the xsd check. At this stage, the file 
is recognized as an invoice. In the event of an error, an UNACCEPTABLE status (“IRRECEVABLE”, Code 501) is sent 
to the issuer, with the reason (see list in Annex A of standard XP Z12-012). The issuer can then correct the error 
and resend the file. If the checks do not detect any errors, the document is recognized and can be processed.
• Functional checks: management rules, corresponding first to the profile (EN16931, EXTENDED-CTC-FR, BASIC WL, 
etc.), then to the management rules specific to the reform, as described in Standard XP Z12-012. Then checks for 
duplicates, addresses, and the "reachability" of the electronic invoicing address for receiving invoices. At the end 
of this last phase, the statuses are as follows:
ü In the case of processing upon issuance (PA-E), the status is either "Rejected" in the event of an error, with the 
associated reason, or "Submitted." They are transmitted to the PPF and made available to the issuer (the SELLER 
and the BUYER for self-billed invoices).
SELLER PA-E
PA-R
BUYER
Rejected
(sending side) Submitted
Rejected
Refused
Approved
Received by PA
Made Available
Partially 
Approved
Dispute
Suspended
Payment sent
Completed
Payment 
received
Sent to PA
Mandatory statuses, to be 
sent to Tax Administration
Transmission phase statuses, 
issued by PAs
Legend
Invoice
In Hand
Cancelled (by corrective invoice)
Treatment phase statuses, 
issued by End Users
Transmission 
Phase
Treatment 
Phase

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page17 / 149
ü In the case of processing on receipt, the status is either "Rejected" in the event of an error, with the associated 
reason, or "Received." The "Rejected" status is sent to the PPF and the PA-E and made available to the recipient. 
The "Received" status is sent to the Issuer (via the PA-E) and made available to the Recipient. The transmission 
phase ends with the status "Made Available," which means that the Recipient has received the invoice or a 
notification of receipt of the invoice. This status is transmitted to the Issuer (via the PA-E) and made available 
to the Recipient.
When invoices are exchanged in batches, i.e., in packages, usually in an envelope that provides some description of the 
contents, an initial flow control step is necessary, beginning with a technical check, followed by a breakdown by "business 
object" file (in this case, invoices) and a unit application check. If one of the "business object" files in the batch contains an 
error (does not comply with the file structure), the most common practice (and that used in exchanges with the PPF, for 
example) is to declare the entire batch "UNACCEPTABLE" (“IRRECEVABLE”) so that the sender can reconstruct it and resend 
it. The other practice is to identify what is in error (or what has been received correctly) and let the issuer correct and resend 
what is in error. If the check is positive, the batch can be given an "ACCEPTABLE" status (Code 500). The "ACCEPTABLE" 
(Code 500) or "UNACCEPTABLE" (Code 501) batch statuses can be transmitted via a life cycle message.
For the sake of completeness, it may happen that the PA-E transmits an invoice to a PA-R that is not responsible for the 
recipient's electronic invoicing address, either due to an error in consulting the directory on the PA-E side or, in the event of 
a recent change of PA-R, a problem with updating the directory. In this case, to avoid setting a "Rejected" status, which 
would require canceling the invoice and creating a new one, the PA-R must issue an "ERROR_ROUTING" status (Code 221) 
to the PA-E, which can then retry the transmission after updating the directory or consulting/synchronizing it.
The steps in the nominal case are as follows, organized into seven steps, the numbering of which will be retained throughout 
the description of the various use cases:
Step Step name Responsible 
actor Description
1 Creation of the invoice for 
the BUYER SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It can entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and related 
statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses in its PA-R ("Rejected," "Accepted," "In dispute," 
"Approved," "Partially approved," "Suspended," etc.) for 
transmission to the SELLER via its PA-E.
4c Receipt of invoice statuses SELLER
The SELLER receives the invoice statuses following processing of the 
invoice by the BUYER in accordance with the terms of the life cycle. If 
a "Suspended" status is received, the SELLER may send a 
"Completed" status containing the necessary additional information.
5a
5b
Payment of the invoice
Creation and transmission 
of the "Payment 
transmitted" status
BUYER 
/ PA-R
The BUYER pays the invoice to the SELLER. They can send a "Payment 
Sent" status to the SELLER via PA-R (recommended).
5c Receipt of "Payment 
Transmitted" status
SELLER 
/ PA-E
The SELLER receives the "Payment Transmitted" status from their PAE.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page18 / 149
Step Step name Responsible 
actor Description
6a Invoice payment receipt SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status
SELLER 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER creates 
the "Payment Received" status and transmits it to the CdD PPF via its 
PA-E. The PA-E also transmits the "Payment Received" status to the 
PA-R for the attention of the BUYER.
6c Receipt of "Paid" status by 
the BUYER
BUYER
PA-R The BUYER receives the "Received" status.
7 Receipt of "Paid" status by 
the PPF CdD
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Payment 
Received" status.
The "Payment Transmitted" status indicates the amount paid using block MDG-43 of the life cycle message, if applicable 
with details of the applicable VAT rate (optional, requiring several iterations if multiple VAT rates apply), as follows:
• MDT-207 (Data Type Code): MPA (meaning "Amount Paid").
• MDT-215 (Amount): the amount paid with VAT.
• MDT-224: applicable VAT rate (optional, only if the BUYER wishes to inform the SELLER of the applicable VAT details 
in their payment).
The "Received" status MUST indicate the amount(s) received by applicable VAT rate, as follows: 
• MDT-207 (Data type code): MEN (meaning "Amount received (with VAT)").
• MDT-215 (Amount): the Payment Amount Received with VAT.
• MDT-224: the VAT rate applicable to the amount received with VAT.
Example: an invoice for €1,160 with VAT, of which €600 without VAT at 20% VAT (€120) and €400 without VAT at 10% (€40):
• MDG-43 (1stoccurrence):
ü MDT-207 (Data type code): MEN (meaning "Amount received (with VAT)").
ü MDT-215 (Amount): 720
ü MDT-224: 20
• MDG-43 (2nd occurrence):
ü MDT-207 (Data type code): MEN (meaning "Amount received (with VAT)").
ü MDT-215 (Amount): 440
ü MDT-224: 10
The diagram below illustrates the mechanics of invoice transmission and its life cycle. The white blocks represent the 
exchange of electronic invoices, life cycle statuses, and invoicing data (flow 1) that PA-E or PA-R issue or receive on behalf 
of their respective customers.
The gray blocks correspond to actions or steps that are outside the scope of PA-E and PA-R.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page19 / 149
Figure2: Nominal case of invoice exchange
With regard to the life cycle, each Accredited Platform is required to: 
Ø Create the transmission phase statuses and make them available to their customers (“Rejected upon issuance” 
/ “Submitted”, “Sent”, for the PA-E; “Rejected on receipt” / “Received”, “Made available” for the PA-R).
Ø To transmit the statuses of the transmission phase to the Accredited Platform of the Counterparty, EXCEPT for 
the statuses "Sent" / "Submitted" and "Rejected" upon issuance, when set by the PA-E in charge of processing 
the issuance.
Ø To transmit to the Accredited Platform of the Counterparty the statuses it receives from its users or that its users 
create on the Accredited Platform.
Ø To make available to its users the life cycle statuses it has received from the Accredited Platform of the 
Counterparty or which it has set itself during the transmission phase.
Ø To transmit to the PPF Data Concentrator (CdD), for invoices subject to e-invoicing, the mandatory statuses 
(“Submitted”, “Rejected” (on issue or on receipt), “Refused”, “Payment Received” when VAT is due on payment 
receipt), as well as flow 1 of the data required by the tax authorities for the PA-E in charge of processing on issue.
With regard to the life cycle, businesses subject to the reform are required to:
Ø Choose a reason for the "Refused" status from the list of usable reasons described in the description of the 
minimum base formats and profiles for invoices and life cycle statuses.
Ø Create, or have created on their behalf, a "Payment Received" status for each partial or total payment receipt of 
an invoice within the scope of the reform following payment of that invoice by the BUYER or by a third party on 
their behalf, when VAT is due upon receipt.
2.2 Rejection upon issuance
If any of the regulatory checks results in an error, the PA-E MUST reject the invoice. It creates the status "Rejected" (code 
213), for which a reason must be given to explain the reason for the REJECTION. This status MUST be transmitted to the PPF 
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice
3
Receipt of invoicing data (Flow 1) 
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status 7
"Payment received" status
6b
Treatment statuses
4b
Receipt of invoice statuses
4c
"Payment Sent" status
5b
Receipt of "Payment Sent" status
5c
Receipt of "Payment received" status 
6c
Order/Delivery
Creation of the invoice
1
Processing the invoice
4a
Invoice payment
5a
Payment Received and reconciliation
6a
Transmission of Flow 1, the invoice 
and corresponding status
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page20 / 149
Data Concentrator. It MUST NOT be transmitted to the invoice recipient. It MUST be made available to the SELLER (the 
issuer).
Step Step name Responsible 
actor
Description
1 Creation of the invoice for 
the BUYER
SELLER Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2 Rejection of the invoice and 
transmission of the 
"Rejected" status to the CdD 
PPF
PA-E The PA-E has detected an error in the regulatory checks it is required 
to perform, including checking for duplicates with invoices it has 
already processed on behalf of the SELLER.
It creates a "Rejected" status, which it sends to the CdD PPF and 
makes available to the SELLER.
1b The SELLER cancels the 
rejected invoice in its 
accounts.
SELLER
PA-E
The SELLER cancels the rejected invoice in its accounts, for example by 
creating an INTERNAL CREDIT NOTE (i.e., not sent to the recipient who 
did not receive the rejected invoice). It does not send it to its PA-E, 
except for archiving purposes only, if applicable.
If the PA-E receives a CREDIT NOTE from the SELLER referring to a 
Rejected invoice, it MUST NOT forward it to the BUYER, nor send a 1 
flow to the CdD PPF.
Figure3: Rejection upon issuance of an e-invoice
Obligations of the PA-E in the event of rejection upon issuance:
Ø Set the status to "Rejected," with a reason for rejection from the list of applicable reasons, send it to the CdD 
PPF, and make it available to the SELLER.
Optional PA-E features in the event of rejection upon issuance:
Ø If the SELLER sends its PA-E a CREDIT NOTE referring to an initial Rejected invoice referenced in BG-3 (reference 
to a previous invoice), the PA-E does not send the CREDIT NOTE to the BUYER and does not send a flow 1 of this 
credit note to the CdD PPF. 
SELLER obligations in the event of Rejection on issue:
Ø The SELLER MUST cancel the posting of an invoice that has been given a "Rejected" status by the PA-E, for 
example by creating an INTERNAL CREDIT NOTE that is not sent to the PA-E, based on the "Rejected" life cycle
status, or sent to its PA-E for archiving purposes only and not for transmission.
SELLER
PA-E
BUYER
PA-R
Order/Delivery
Invoice creation
1
Rejection of the invoice on issue
2
Accounting cancellation
1b
CdD
PPF
Receipt of invoicing data (Flow 1) 
and statuses "Submitted", "Rejected"
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page21 / 149
2.3 Compliant invoice but not transmitted due to the recipient not choosing an Accredited Platform 
It may happen that the recipient subject to VAT, listed in the directory, has not chosen an Accredited Platform. This can be 
verified by consulting the PPF Directory («Annuaire») in advance. The recipient has at least one inactive SIREN address. In 
this case, the SELLER may use this inactive electronic address listed in the directory (since no other address is active) and
send it to their PA-E.
The PA-E will carry out the various checks and may create a "Submitted" status and transmit flow 1 and this "Submitted" 
status to the PPF's CdD. In this case, the "Submitted" status must contain a REASON "NOT_TRANSMITTED" to indicate to the 
PPF (and the tax authorities) that the BUYER does not have the necessary equipment.
As a reminder, the draft finance bill for 2026 provides for a quarterly penalty if it is found each quarter that a taxable entity
is not equipped with at least one electronic invoicing address for receiving invoices, subject to its adoption.
The situation is then strictly the same as that of a paper invoice (or even a PDF invoice) not received by the BUYER. The 
SELLER should contact their customer and, if necessary, send them a duplicate of the invoice (e.g., a legible representation 
of it).
The BUYER cannot then demand a paper or PDF invoice, as it is the BUYER who is at fault for not receiving it.
Once the BUYER is equipped, the PA-E may, if it wishes to offer this service, for a period of time that it determines, resend 
the "NOT_SENT" invoices so that they reach the BUYER normally.
In any case, the SELLER must, if necessary, file a "Payment Received" status if VAT is due upon payment receipt. However, 
the BUYER cannot file a "Rejected" status since it is not connected to an Accredited Platform. This does not prevent the 
invoice from being processed, as was the case before the reform was implemented. 
Step Step name Responsible 
party
Description
1 Creation of the invoice for 
the BUYER
SELLER Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It can entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2 Transmission of flow 1 and 
"Submitted" status PA-E
Having carried out the regulatory compliance checks, including duplicate 
checks, the PA-E finds that there is no active electronic invoicing address 
(connected to a PA). It MUST create the status "Submitted" with the 
reason "NOT_TRANSMITTED" and send it to the CdD of the PPF, along 
with the data required by the Administration (flow 1).
It MUST also make the "Submitted" status with the reason 
"NOT_TRANSMITTED" available to the SELLER. 
3 The SELLER contacts the 
BUYER to obtain payment
SELLER The SELLER contacts the BUYER as in the case of an unpaid invoice, 
except that they know immediately that the BUYER has not received the 
invoice through no fault of their own.
They may send a duplicate invoice by any means. They are not required 
to issue another invoice by any means other than sending an electronic 
invoice through their PA-E. Nor are they required to cancel their invoice 
(in this case by issuing a credit note treated in the same way as their 
invoice) and create a new one once the BUYER is equipped.
4 and 5 Processing of the invoice BUYER The BUYER and the SELLER process the invoice, which is paid.
6a and 
6b
Payment receipt and 
“Payment received” status
SELLER 
/ PA-E
If the VAT on the invoice is payable upon payment receipt, the SELLER 
creates the "Payment Received" status and transmits it to the PPF CdD 
via its PA-E.
7
Receipt of "Payment 
Received" status by the PPF 
CdD
PPF Data 
Concentrator The PPF Data Concentrator (CdD PPF) receives the "Received" status.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page22 / 149
Figure4: Invoice filed as "NOT_TRANSMITTED" due to lack of PA-R.
Obligations of the PA-E in the absence of an active BUYER address:
Ø If the invoice is deemed compliant, set the status to "Submitted" with the reason "NOT_TRANSMITTED," 
transmit it to the PPF Data Concentrator, and make it available to the SELLER. Transmit flow 1.
Optional PA-E features in the absence of an active BUYER address:
Ø The PA-E may offer to resend the invoice for several days, then, at the request of its customer, the SELLER, to 
resume the normal invoice life cycle, this option remaining available for a reasonable period (a few months, for 
example).
SELLER obligations if the BUYER does not have an active address:
Ø The SELLER MUST still issue the invoice via its e-invoicing platform. It may then (or beforehand) contact its 
customer to ensure that the latter is equipped to receive it. It MAY offer to send its customer a duplicate of the 
invoice (e.g., a LEGIBLE version of the invoice).
Ø The SELLER MUST file a Paid life cycle status after payment has been received, if VAT is due upon payment
receipt.
2.4 Rejection upon receipt
When an invoice is received by a PA-R, the latter must carry out the regulatory checks. In the event of an error, the PA-R 
MUST reject the invoice. It creates the status "Rejected" (code 213), for which a reason must be given to explain the reason 
for the REJECTION. This status MUST be transmitted to the PPF Data Concentrator. It MUST be transmitted to the issuer of 
the invoice (the SELLER) through its PA-E. It MUST be made available to the BUYER for information purposes.
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoicing data (Flow 1) 
and status "Submitted", 
2
Receipt of "Payment received" status 7
"Payment received" status
6b
Order/Delivery
Creation of the invoice 1
Invoice processing
4
Payment of the invoice
5
Collection and reconciliation
6a
No PA-R for the BUYER
Transmission of flow 1, of the invoice
et status “Submitted” with Reason 
“NOT_TRANSMIITED »
2
Invoice management (via Duplicata)
3

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page23 / 149
Step Step name Responsible 
party Description
1 Creation of the invoice for 
the BUYER SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It can entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address, it MUST transmit the data required by 
the Administration (flow 1) to the PPF Data Concentrator (CdD PPF). 
It MUST transmit the invoice (stream 2 or stream 3) to the BUYER's 
PA-R. It must transmit the "Submitted" status to the CdD PPF.
3a
3b
Rejection of the invoice 
upon receipt PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, and finds an error. It must create a 
"Rejected" status (code 213) with the reason for rejection and send it 
to the CdD PPF and the PA-E for the attention of the SELLER. It makes 
the status available to the BUYER but does not deliver the invoice to 
them unless requested to do so.
1b The SELLER cancels the 
rejected invoice in the 
accounts.
SELLER The SELLER cancels the rejected invoice in its accounts, for example 
by creating an INTERNAL CREDIT NOTE (i.e., not sent to the recipient 
who did not receive the invoice rejected by its PA-R). It does not send 
it to its PA-E, except for archiving purposes only, if applicable.
If the E-PA receives a CREDIT NOTE from the SELLER referring to a 
Rejected invoice, it MUST NOT send it to the BUYER, nor send a 1 
flow to the CdD PPF.
Figure5: Rejection of an invoice upon receipt
Obligations of the PA-R in the event of rejection upon receipt:
Ø Set the status to "Rejected," with a reason for rejection from the list of applicable reasons, send it to the CdD 
PPF and the PA-E, and make it available to the BUYER.
Obligations of the PA-E in the event of rejection upon receipt:
Ø Make the status "Rejected" available to the SELLER.
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Rejection of the invoice on receipt
3a
Receipt of invoicing data (Flow 1)
and statuses "Submitted", "Rejected"
2
Order/Delivery
Creation of the invoice
1
Transmission of Flow 1, invoice 
and corresponding status
2
Accounting cancellation
1b
Receipt of rejection status
3b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page24 / 149
SELLER obligations in the event of rejection upon receipt:
Ø Cancel the posting of an invoice that has been given a "Rejected" status by PA-R, for example by creating an 
INTERNAL CREDIT NOTE that is not sent to PA-E (except for archiving purposes only), based on the "Rejected" 
life cycle status (supporting accounting document).
Optional PA-E features in the event of rejection upon receipt:
Ø If the SELLER sends its PA-E a CREDIT NOTE referring to an initial Rejected invoice referenced in BG-3 (reference 
to a previous invoice), the PA-E does not send the CREDIT NOTE to the BUYER and does not send a flow 1 of this 
credit note to the CdD PPF.
2.5 Rejection of an invoice by the BUYER (the recipient)
The BUYER may reject the invoice. As a result, the flow 1 sent in parallel with the issuance of the invoice to its recipient is 
ignored by the tax authorities for VAT pre-filling. In theory, this requires the SELLER to cancel the "Rejected" invoice in its 
accounts, for example by creating an INTERNAL CREDIT NOTE that is not sent to the BUYER and without a flow 1 sent to the 
CdD PPF, and by creating a new invoice if necessary. On the BUYER's side, setting a "Rejected" status leads either to the 
invoice not being recorded or to its cancellation, justified by the "Rejected" status (to be kept as an accounting document). 
Consequently, the "Rejected" status MUST NOT be used in the event of a dispute or disagreement over the content of an 
invoice, and MUST ALWAYS be accompanied by a reason for "Rejection" from the list of applicable reasons for Rejection, 
which correspond to:
• Either a regulatory non-compliance not tested by the PA-R (e.g., absence of a Purchase Order number provided 
prior to invoicing).
• Either a bill for an unknown transaction, i.e., for which the BUYER does not know the SELLER or has not purchased 
anything from them that would justify receiving a bill.
• Or, where applicable, a breach of contractual conditions preventing the recipient from processing the invoice, 
which must be limited to the provision of additional references from among those available in the semantic model 
of essential invoice data EN16931 and known to the SELLER at the time of invoicing.
The list of reasons for "Rejection" is described in Annex A of Standard XP Z12-012.
For the CdD PPF, any "Rejected" or "Refused" status assigned to an invoice cannot be followed by another status for that 
invoice. Therefore, once a "Rejected" or "Refused" status has been assigned, no other "mandatory" status can be assigned 
by the Accredited Platforms («Plateformes Agréées») or end users on behalf of the SELLER or BUYER (as a reminder, the 
"Mandatory" statuses are: "Submitted," "Rejected," "Refused," and "Payment Received").
Step Step name Responsible 
party Description
1 Creation of the invoice for 
the BUYER SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address, it MUST send the data required by the 
Administration (flow 1) to the PPF Data Concentrator (CdD PPF). It 
MUST transmit the invoice (stream 2 or stream 3) to the BUYER's PAR. It must transmit the "Submitted" status to the PPF Data 
Concentrator.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page25 / 149
Step Step name Responsible 
party Description
4a
4b Rejection of the invoice BUYER
PA-R
The BUYER processes the invoice and decides to reject it by selecting 
the reason for rejection from a strict list (see standard XP Z12-012).
The PA-R transmits the status "Rejected," along with the reason, to 
the CdD PPF and the PA-E, for the attention of the SELLER.
4c Receipt of "Rejected" 
status
SELLER
PA-E
The PA-E receives the "Rejected" status. It makes it available to the 
SELLER.
1a The SELLER cancels the 
"Rejected" invoice in the 
accounts.
SELLER The SELLER cancels the "Rejected" invoice in its accounts, for 
example by creating an INTERNAL CREDIT NOTE. It does not send it 
to its PA-E, except for archiving purposes only, if applicable.
If the PA-E receives a CREDIT NOTE referring to a "Rejected" invoice 
from the SELLER, it must not send it to the BUYER, nor send a flow 1 
to the CdD PPF.
1b If necessary, the BUYER 
cancels the registration of 
the invoice it has rejected.
BUYER If the BUYER has already recorded the invoice in its accounts prior to 
its rejection, it must cancel it, as the "Rejected" status constitutes the 
supporting accounting document.
Figure6: Rejection of an invoice by the BUYER (recipient of the invoice)
NOTE: in the event of misuse of the "Rejected" status, or if this status is applied in error, the SELLER may decide to maintain 
its invoice and not cancel it in its accounts. They shall inform the BUYER of this outside of the life cycle exchanges. This may 
therefore lead to a dispute between the SELLER and the BUYER, or an agreement to maintain the invoice and not have to 
cancel it in the accounts on either side and to reissue an identical new invoice with a different issue date and potentially an 
impact on payment terms. However, the consequence remains that the pre-filled VAT will not take into account this 
retention and therefore the VAT on this "Rejected" invoice, both for the VAT collected on the SELLER's side and for the 
deductible VAT on the BUYER's side. This issue is likely to be clarified in a future version.
PA-R obligations in the event of refusal by the BUYER:
Ø Transmit the "Rejected" status to the CdD PPF and the PA-E.
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice
3a
Receipt of invoicing data (Flow 1)
and statuses "Submitted", "Rejected", "Refused"
2
Order/Delivery
Creation of the invoice
1
Transmission of Flow 1, the invoice 
and corresponding status
2
Accounting cancellation
1a
Receipt of refusal status
4c
Invoice processing
4a
Rejection of the invoice and "Refused" status
4b
If recorded before refusal, accounting cancellation
1b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page26 / 149
Obligations of the PA-E in the event of refusal by the BUYER:
Ø Provide the SELLER with the status "Refused" (with the reason).
Optional PA-E features in the event of rejection by the BUYER:
Ø If the SELLER sends its PA-E a CREDIT NOTE referring to an initial Rejected invoice referenced in BG-3 (reference 
to a previous invoice), the PA-E does not send the CREDIT NOTE to the BUYER and does not send flow 1 to the 
CdD PPF.
Actions taken by the SELLER in the event of refusal by the BUYER:
Ø Based on the "Rejected" life cycle status, the SELLER cancels the posting of the invoice on which the "Rejected" 
status has been placed by the BUYER, for example by creating an INTERNAL CREDIT NOTE that is not sent to the 
PA-E (except for archiving purposes only). This ensures that the VAT return will be aligned with the VAT pre-fill 
proposed by the tax authorities.
Ø However, the SELLER may contest the REFUSAL. In this case, they will initiate a dispute with the BUYER and will 
not create a CREDIT NOTE. This must be done outside of the life cycle status exchange. A future version of this 
Document will examine whether it is necessary to provide for a life cycle status exchange dedicated to this 
normally rare case. In this case, the SELLER's VAT return will retain the VAT on this invoice. If the BUYER finally 
accepts the invoice, then the SELLER and the BUYER will have a discrepancy between their VAT return and the 
pre-filled VAT return, equal to the VAT on the invoice, since the tax authorities will have canceled the VAT on 
the rejected invoice.
Actions taken by the BUYER in the event of refusal by the BUYER:
Ø Based on the "Rejected" life cycle status, the BUYER does not record the "Rejected" invoice or cancels its 
recording. The "Rejected" status serves as supporting documentation for, for example, posting a "Miscellaneous 
Transaction" entry to cancel a "Rejected" invoice. No CREDIT NOTE is therefore expected by the BUYER.
Ø However, the BUYER may contest the "Rejected" status. They may then finally accept the invoice and record it, 
then process it, which is currently organized outside of life cycle status exchanges. On the other hand, the 
BUYER's and SELLER's pre-filled VAT will show a discrepancy with the VAT return equal to the VAT on this 
"Rejected" invoice.
2.6 Management of a "Dispute" followed by a CREDIT NOTE
This case mainly describes how to manage a CREDIT NOTE that partially or totally cancels a disputed invoice.
Step Step name Responsible 
party Description
1 Creation of invoice F1 for 
the BUYER SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address, it MUST send the data required by the 
Administration (flow 1) to the PPF Data Concentrator (CdD PPF). It 
MUST transmit the invoice (stream 2 or stream 3) to the BUYER's PAR. It must transmit the "Submitted" status to the PPF Data 
Concentrator.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the F1 invoice, performs the 
regulatory checks, creates the transmission statuses (here 
"Received," then "Made Available") and makes the invoice available 
to the BUYER for processing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page27 / 149
Step Step name Responsible 
party Description
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
PA-R
The BUYER processes the invoice and notices a discrepancy with 
what is expected. They therefore set the status to "In Dispute," with 
a mandatory reason and, if applicable, an expected action (e.g., 
request for a CREDIT NOTE), to be sent to the SELLER via the PA-R 
and PA-E.
4c Receipt of invoice statuses SELLER The SELLER receives the "In dispute" status of the invoice.
Step Step name Responsible 
party Description
1 (F2) Creation of a CREDIT SELLER The SELLER creates a CREDIT NOTE, either total or partial.
2 Transmission of the CREDIT 
NOTE PA-E
The PA-E processes the CREDIT F2, transmits flow 1 and the 
"Submitted" status of F2 to the CdD PPF, and the CREDIT F2 to the 
PA-R.
3 Receipt of CREDIT PA-R The PA-R receives the F2 CREDIT, processes it, and makes it available 
to the BUYER.
4a
4b
Processing of the F2 CREDIT 
NOTE and the F1 INVOICE BUYER
The BUYER approves the F1 Invoice that was in "Dispute" and the F2 
CREDIT NOTE by setting the status to "Approved" on each document, 
potentially in a Life cycle status message common to F1 and F2.
4c Receipt of F2 statuses SELLER The SELLER receives the "Approved" statuses for F1 and F2.
5a
5b
In the event of a partial 
CREDIT, payment and 
"Payment Sent" status.
BUYER
In the event of a partial credit, the BUYER pays the balance F1 – F2 
and sends the status "Payment Sent" on F1 for the amount of the 
balance, or on F1 as positive and on F2 as negative.
5c Receipt of payment 
statuses
SELLER The seller receives the payment statuses from F1, or even F2.
6a
6b
Reconciliation of receipt (if 
partial credit note)
SELLER
/PA-E
Reconciliation of cash receipts. If VAT is charged on receipt:
• (Highly Recommended) create a "Received" status for F1 
(total) and for F2 (negative amount), which allows the 
"Received" status to be considered final in this case (F1 and 
F2 definitively processed).
• Or create a partial "Received" status on F1 (if partial 
payment), and nothing else. In this case, everyone must 
understand that F1 and F2 are definitively processed.
Transmission of the "Received" status to PA-R and CdD PPF.
6c Receipt of "Paid" statuses BUYER The BUYER receives the "Paid" statuses.
7 Receipt of "Paid" status by 
CdD PPF CdD PPF Receipt of "Paid" status by CdD PPF.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page28 / 149
Figure7: Invoice in dispute, followed by a partial or total CREDIT NOTE
PA-R obligations in the event of "Disputed" status followed by a partial or total CREDIT NOTE:
Ø Transmit the "In Dispute" status from F1 to the PA-E, then process the F2 CREDIT NOTE as a normal invoice, with 
its statuses set by the BUYER.
Obligations of the PA-E in the event of "Disputed" status followed by a partial or total CREDIT NOTE:
Ø Provide the SELLER with the "In Dispute" status of F1, with its reason and required action, then process the 
CREDIT NOTE F2 as a normal invoice, including for the exchange of statuses set by the BUYER or the SELLER.
SELLER obligations in the event of "Disputed" status followed by a partial or total CREDIT NOTE:
Ø If the SELLER accepts the resolution of the dispute, they create a CREDIT NOTE F2 and then treat it as an invoice.
Obligations of the BUYER in the event of a "Disputed" status followed by a partial or total CREDIT NOTE:
Ø If they receive an F2 CREDIT NOTE, they treat it as an invoice. A good practice is to approve the F1 invoice and 
the CREDIT NOTE, then pay the balance as the result of the payment of the F1 invoice from which the amount of 
the F2 CREDIT NOTE is deducted.
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice F1
3
Receipt of invoicing data (Flow 1)
and statuses "Submitted« , "Rejected", "Refused"
2
Receipt of "Payment received" status
7
"Dispute" status
4b
Receipt of invoice statuses
4c
Order/Delivery
Creation of invoice F1
1
Processing of invoice F1
Transmission of Flow 1, invoice F1 4a
and corresponding status
2
Commercial processing of the dispute
Creation of a credit note F2 to cancel F1
1
Receipt of credit note F2
3
Processing of credit note F2
4a
"Payment received" status on F1 and F2
6b
"Approved" status on F1 and F2
4b
Receipt of "Approved" status on F1 and F2
4c
Receipt of "Payment received" status on F1 and F2 
6c
Transmission of Flow 1, credit note F2, and 
corresponding status
2
If applicable, payment of balance F1 – F2
5a
"Payment sent" status
5b
Receipt of "Payment sent" status
5c
(Potential) Payment receipt and reconciliation
6a
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page29 / 149
2.7 Management of a "Dispute" followed by a Corrected Invoice
This case mainly describes how to manage a corrected invoice created to cancel and replace a disputed invoice.
Step Step name Responsible 
party Description
1 Creation of invoice F1 for 
the BUYER SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It can entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address, it MUST transmit the data required by 
the Administration (flow 1) to the CdD PPF. It MUST transmit the 
invoice (flow 2 or flow 3) to the BUYER's PA-R. It must send the 
"Submitted" status to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the F1 invoice, performs the 
regulatory checks, creates the transmission statuses (here 
"Received," then "Made Available") and makes the invoice available 
to the BUYER for processing.
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
PA-R
The BUYER processes the invoice and notices a discrepancy with what 
is expected. They therefore assign a "Disputed" status, with a 
mandatory reason and, if applicable, an expected action (e.g., 
request for a corrected invoice), to be sent to the SELLER via PA-R 
and PA-E.
4c Receipt of invoice statuses SELLER The SELLER receives the "In dispute" status of the invoice and the 
request for a Corrected Invoice.
Step Step name Responsible 
party Description
1 (F2) Creation of a Corrected
Invoice F2 SELLER
The SELLER creates a Corrected Invoice. It must reference the invoice 
it is canceling and replacing in BG-3 (previous invoice). From an 
accounting perspective, this means that the referenced invoice must 
be canceled and replaced by the Corrected Invoice.
2 Transmission of the F2 
Corrected Invoice PA-E
The PA-E processes the F2 invoice, transmits flow 1 and the 
"Submitted" status of F2 to the CdD PPF, and the Corrected Invoice 
F2 to the PA-R.
3 Receipt of the F2 Corrected
Invoice PA-R
The PA-R receives the F2 Corrected Invoice, processes it, and makes 
it available to the BUYER.
4a
4b
Processing of the F2 
Corrected Invoice BUYER
The BUYER processes the F2 Corrected Invoice, which begins with the 
cancellation of F1 (justified by the existence of F2), followed by the 
processing of F2 as a new invoice.
To ensure that Invoice F1 does not remain indefinitely with the status 
"In Dispute," it may be useful to assign it the status "Canceled" and 
send it to the SELLER.
4c
Receipt of F2 statuses, or 
even "Canceled" status for 
F1
SELLER The SELLER receives the status of F2 (presumably "Approved") and 
potentially the "Canceled" status of F1.
5a
5b Payment of F2 invoice BUYER 
/ PA-R
The BUYER pays the F2 invoice to the SELLER. They can send a 
"Payment Sent" status to the SELLER via the PA-R (recommended).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page30 / 149
Step Step name Responsible 
party Description
Creation and transmission 
of the "Payment 
transmitted" status.
5c Receipt of "Payment 
Transmitted" status
SELLER 
/ PA-E
The SELLER receives the "Payment Transmitted" status for F2 from 
their PA-E.
6a
6b
Invoice payment receipt SELLER The SELLER receives payment for invoice F2 (outside the circuit).
6c Issuance of "Payment 
Received" status
SELLER 
/ PA-E
If the VAT on the F2 invoice is payable upon payment receipt, the 
SELLER creates the "Payment Received" status and transmits it to the 
CdD PPF via its PA-E. The PA-E also transmits the "Payment Received" 
status to the PA-R for the attention of the BUYER.
7 Receipt of "Paid" status by 
the BUYER
BUYER
PA-R The BUYER receives the "Received" status from F2.
Figure8: Invoice in dispute, followed by a Corrected Invoice
PA-R obligations in the event of "Disputed" status followed by a Corrected Invoice:
Ø Transmit the "In Dispute" status of invoice F1 to the PA-E, then process the Corrected Invoice F2 as a normal 
invoice, with its statuses set by the BUYER.
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice F1
3
Receipt of invoicing data (Flow 1)
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
"Dispute" status
4b
Receipt of invoice statuses
4c
Order/Delivery
Creation of invoice F1
1
Processing of invoice F1
Transmission of Flow 1, invoice F1 4a
and corresponding status
2
Commercial processing of the dispute
Creation of a corrected invoice F2
1
Receipt of the F2 corrective invoice
3
Processing of the F2 corrective invoice
4a
"Payment received" status on F2
6a
"Approved" status on F2
... and "Canceled" for F1
4b Receipt of "approved" status on F2
... and "Cancelled" for F1
4c
Receipt of "Payment received" status on F2 
6b
Transmission of Flow 1, corrective invoice F2, and 
corresponding status
2
Payment of F2
5a
"Payment sent" status
5b
Payment Receipt and reconciliation
6a
Receipt of "Payment Sent" status
5c
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page31 / 149
Obligations of the PA-E in the event of "Disputed" status followed by a Corrected Invoice:
Ø Provide the SELLER with the "In Dispute" status of invoice F1, with its reason and required action, then process 
the Corrected Invoice F2 as a normal invoice, including for the exchange of statuses set by the BUYER or SELLER.
SELLER's obligations in the event of "Disputed" status followed by a Corrected Invoice:
Ø If the SELLER accepts the resolution of the dispute by means of a Corrected Invoice, they shall create a Corrected
Invoice F2, which cancels invoice F1 (cancellation entry to be organized), and then treat invoice F2 as an invoice.
They can set the status of Invoice F1 to “Cancelled” to signify the end of the lifecycle and share this with the 
SELLER via their respective Pas.
Obligations of the BUYER in the event of "Disputed" status followed by a Corrected Invoice:
Ø If the BUYER receives a Corrected Invoice F2, they MUST cancel the posting of Invoice F1 in their accounts, then 
treat Invoice F2 as a standard invoice. To avoid leaving Invoice F1 in "Disputed" status indefinitely, they can set 
it to "Cancelled" status and share this with the SELLER via their respective Accredited Platforms («Plateformes 
Agréées»).
3 Descrip2on of the main use cases
3.1 Summary table of use cases 
Category ID Use case
Multi-order/Multi-delivery 1 Case No. 1: Multi-order/Multi-delivery
Invoice already paid by a third 
party or the buyer 2 Case 2: Invoice already paid by the BUYER or a third-party PAYER at the time the invoice is issued
Invoice to be paid by a third 
party
3 Case 3: Invoice payable by a third-party PAYER known at the time of invoicing
4 Case 4: Invoice payable by the buyer and partially covered by a third party 
known at the time of invoicing (subsidy, insurance, etc.)
Expenses paid by third parties 
with invoice 5 Case No. 5: Expenses paid by employees with invoices in the company's name
Expenses paid by third parties 
without invoice 6
Case No. 6: Expenses paid by employees without an invoice addressed to the 
company (simple receipt or invoice made out to the employee's name and 
address)
Invoice paid by a third party 7 Case No. 7: Invoice following a purchase paid for with a corporate card 
(purchasing card)
Invoice payable to a third party
8 Case No. 8: Invoice payable to a third party determined at the time of 
invoicing (factoring, cash pooling)
9
Case No. 9: Invoice payable to a third party known at the time of invoicing, 
who also manages the order/receipt, or even invoicing (Distributor /
Depositary)
10 Case No. 10: Invoice payable to a third-party payee unknown at the time the 
invoice was created, in particular a factoring company (case of subrogation)
Invoice with "addressed to" 
different from the buyer 11 Case No. 11: Invoice to be received and processed by a third party on behalf of the BUYER
Transparent intermediary 12 Case No. 12: Transparent intermediary managing invoices for its principal 
BUYER

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page32 / 149
Category ID Use case
Subcontracting invoice for 
direct payment 13 Case No. 13: Invoice payable by a third party: subcontracting with direct payment or payment delegation
Co-contracting invoice 14 Case No. 14: Invoice payable by a third party: joint contracting case B2B
Invoice following an 
order/payment by a third party 
on behalf of the buyer
15 Case No. 15: Sales invoice following an order (and possible payment) by a 
third party on behalf of the BUYER ( )
16 Case No. 16: Expense invoice for reimbursement of the sales invoice paid by 
the third party
Invoice issued by a third party, 
payment intermediary 17a Case No. 17a: Invoice payable to a third party, payment intermediary (e.g., on Marketplace)
Invoice issued by a third party, 
payment intermediary and 
invoicing mandate
17b Case No. 17b: Invoice payable to a third party, payment intermediary, and 
third-party invoicing under an invoicing mandate
Debit notes 18 Case No. 18: Management of debit notes
Invoices issued under thirdparty mandate 19a Case No. 19a: Invoice issued by a third-party invoicing with an invoicing 
mandate
Self-billing 19b Case No. 19b: Self-billing
Pre-payment invoice
20 Cases 20 and 21: Pre-payment invoice and final invoice after advance 
payment 21
Invoice with allowance
22 Case No. 22a: Invoice paid with early payment discount for services for which 
VAT is payable upon receipt of payment 
22b Case No. 22b: Invoice paid with allowance for deliveries of goods (or 
provision of services with VAT option on debits)
Self-billing between an 
individual and a professional 23 Case No. 23: Self-billing flow between an individual and a professional
Deposit (“Arrhes”) 24 Case No. 24: Management of 
Gift vouchers and cards 25 Case No. 25: Management of vouchers and gift cards
Invoices with contractual 
reservation clauses 26 Case No. 26: Invoices with contractual reservation clauses
Toll tickets 27 Case No. 27: Management of toll tickets sold to a taxable entity
Restaurant receipts 28 Case No. 28: Management of restaurant bills issued by a SELLER subject to 
tax established in France
Single Taxable Entity and 
members of the Single Taxable 
Entity
29 Case No. 29: Single Taxable Entity within the meaning of Article 256 C of the 
CGI
E-reporting transaction subject 
to an invoice or "VAT already 
collected"
30 Case No. 30: VAT already collected - Transactions initially processed in B2C ereporting, subject to a retrospective invoice
Mixed invoices 31 Case No. 31: "Mixed" invoices mentioning a main transaction and an ancillary 
transaction
Management of monthly 
payments
32 Case No. 32: Monthly payments
VAT regime on the margin 33 Case No. 33: Transactions subject to the margin scheme -profit

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page33 / 149
Category ID Use case
Partial payment receipt and 
cancellation of payment 
receipt
34 Case No. 34: Partial payment receipt and cancellation of payment receipt
Author's notes 35 Case No. 35: Author's notes 
Professional secrecy 36 Case No. 36: Transactions subject to professional secrecy and exchanges of 
sensitive data
Joint ventures 37 Case No. 37: 
Invoice sub-lines 38 Case No. 38: Invoices with sub-lines and line groupings
Multi-Vendors 39 Case No. 39: Transparent intermediary consolidating sales from multiple 
Sellers for the same buyer – Multi-Vendor Invoice
Offsetting between cross-flows 40 Case No. 40: Grouped payments, netting, or compensation in the event of 
cross-purchases/sales
Barter practices 41 Case No. 41 Barter Companies
Tax exemption 42 Case No. 42: Tax exemption management
International B2B operations 43
Case No. 43: E-reporting for international B2B
Case No. 43a: Triangular transactions
Case No. 43b: Stock transfers treated as intra-Community supply
Operations with French DROM 
/ COM / TAAF 44 Case No. 44: Transactions with entities established in the DROMs/COMs/TAAFs
3.2 Handling of the main cases
3.2.1 General
First, the instruction of these use cases were first examined in accordance with the following two principles, insofar as 
possible:
• "Same problems, same solutions": the issues raised by companies through use cases are sometimes very similar 
from one sector to another.
• "Do not export complexity to counterparties": there are sector-specific issues, but it is important not to resolve 
them by imposing specific constraints on the outside world, or at least as little as possible.
The use cases discussed below can lead to two types of resolution:
• Indicate which data from the EN16931 model can be used to codify certain invoices and, if necessary, use the 
EXTENDED-CTC-FR profile to obtain additional data.
• Indicate how third parties can interact with the management process.
3.2.1.1 Third-party management
For the second type of use case, several third parties are provided for in the EXTENDED-CTC-FR profile. However, these third 
parties generally act either on behalf of the SELLER or on behalf of the BUYER. It is therefore primarily up to the PA-E and 
PA-R to propose solutions to allow these third parties to access invoices and life cycle statuses in order to act on behalf of 
the SELLER or the BUYER.
The diagram below illustrates the organization of third parties around the SELLER's and BUYER's central Accredited Platforms 
(“Plateformes Agréées”). Information sharing or third-party interactions are thus organized around these central Accredited 
Platforms (“Plateformes Agréées”), either because third parties have access to one of the two Accredited Platforms 
(“Plateformes Agréées”), for example through the standard API covered by Standard XP Z12-013, or because they interact 
by exchanging status and invoice files, for example through the PEPPOL network. In both cases, this means that they must 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page34 / 149
be listed on the Accredited Platform where they operate and that they have delegation rights over invoices issued by or on 
behalf of the SELLER or those received by the BUYER.
The management of these rights can be facilitated by seeing the Third Parties named in the invoices. This is why different 
roles have been provided for in the EXTENDED-CTC-FR profile:
• The PAYER third party (EXT-FR-FE-BG-02)
• The BUYER'S AGENT third party (EXT-FR-FE-BG-01)
• The SELLER'S AGENT third party (EXT-FR-FE-BG-03)
• The INVOICER third party (EXT-FR-FE-BG-05), the SELLER's AGENT for creating and transmitting the SELLER's 
invoices on its behalf.
• The third party "ADDRESSEE" or “INVOICEE” (EXT-FR-FE-BG-04), responsible for processing invoices on behalf of 
the BUYER.
Figure9: Organization of third parties around the BUYER's and SELLER's Accredited Platforms («Plateformes Agréées»)
The presence of these third parties in invoices can help organize access rights or actions on invoices. However, these rights
can also be organized independently by the Accredited Platform, which may wish to associate them with a management 
process. 
This means that the SELLER is not required to use the EXTENDED-CTC-FR profile, but above all means that SELLERS do not 
have to manage a list of third parties acting on behalf of the BUYER to be included in their invoices, particularly when these 
third parties do not need to be contacted directly by the SELLER, but are only involved due to the specific requirements of 
the BUYER.
For example, the BUYER's Accredited Platform may provide for roles associated with certain rights and organize a thirdparty repository, such as, for example, a Chartered Accountant, who may have rights to access invoices, monitor statuses, 
and sometimes file certain invoices, a function quite similar to a "To" field on the BUYER side, but without requiring suppliers 
to name them in invoices.
In the diagram above, we can also see that the two Accredited Platforms («Plateformes Agréées») are linked to OD/SC 
(Compatible Solution). This reflects the reality that Accredited Platforms («Plateformes Agréées») are primarily platforms 
responsible for checking invoices and their transmission (issuance and receipt), as well as the related life cycle statuses, and 
that most of the functions upstream and downstream of this exchange of invoices and statuses, namely the creation of 
invoices and statuses, reconciliation, validation, accounting, payment, archiving, etc., can be offered by OD/SC (Compatible 
Solutions). When Accredited Platforms («Plateformes Agréées») themselves have OD/SC-type solutions (Compatible 
Solution), interaction with users and third parties can then be organized using these solutions, provided that they are subject 
to the same security and hosting requirements as Accredited Platforms («Plateformes Agréées») (ISO 27001, hosting in the 
EU, etc.).
A number of use cases involve third parties that are "transparent" in the sense that they act on behalf of either the SELLER 
to issue the invoice on their behalf (in the position of INVOICER), or the BUYER to process the invoice on their behalf 
SELLER
BUYER
SELLER’S 
AGENT
INVOICER
PAYEE
PAYER
BUYER’S AGENT
INVOICEE
PA-SELLER PA-BUYER
E-invoice
Lifecycle statuses
BUYER's electronic invoicing 
address : SIRENBUYER_ _XXX
PA
SC, OD, IS, ERP, …

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page35 / 149
(MANAGER in the position of ADDRESSEE). In order to maintain the principle of an exchange of invoices between a SELLER 
and a BUYER, it is therefore necessary for the issuing PA-E and receiving PA-R to take into account the SELLER and BUYER 
respectively, who are therefore users without necessarily being customers. 
Figure10: Transparent third-party invoicing and third-party authorized agent
For example, the THIRD PARTY AGENT, who has a mandate from the BUYER to receive and process certain purchase invoices 
from the BUYER, may open an account in the name of this BUYER, for a dedicated SIRENacheteur_GESTIONTIERS electronic
address, and take charge of the contractual and commercial aspects with the PA-R in charge of this electronic address for 
receiving invoices, which happens to be that of the BUYER AND the AGENT. The AGENT can then integrate all of this into its 
own service provision to its BUYER client.
Similarly, the PA-E may be chosen by the third-party INVOICER, with the agreement of the SELLER within the framework of 
an invoicing mandate. In practice, it is common to both the SELLER and the INVOICER, with the INVOICER taking charge of 
the contractual and commercial aspects with the PA-E (PA-SELLER and INVOICER). The INVOICER may decide to integrate 
the use of this PA-E into its overall service provision.
3.2.1.2 Access by third parties to invoices and their lifecycles
Third-party access to invoices and lifecycles can be organized around the BUYER's or SELLER's hub platform in two ways:
• By making third parties users of the pivot Accredited Platform, thus giving them their own access, i.e. as a thirdparty entity with its own access and not just the same access as the SELLER or BUYER on whose behalf the third 
party is acting, then implementing a rights delegation feature allowing the third party to view or even act on the 
life cycle of an invoice on behalf of the SELLER or BUYER. The third party can then use the Standard API of the 
XP Z12-013 Standard to connect its information system to the Approved Hub Platform and interact with all the 
accounts for which it has delegation.
• Through message exchanges between the SELLER's or BUYER's pivot platform and the third party's platform, for 
example using the PEPPOL network exchange protocol. To do this, the third party must first be listed on the pivot
platform of the VAT Registered entity for whom they are the third party, with the electronic address at which the 
third party can be reached via the PEPPOL network. Their scope of action must then also be defined, i.e., which 
invoices or statuses must be sent to them, and which statuses they can send in return. Once this is in place, the 
pivot platform can organize the sharing of invoices and statuses by exchanging life cycle status messages (CDAR) 
with the third party's platform, which allow the context to be indicated and, if necessary, an invoice to be sent as 
an attachment.
Regarding this second option, by way of illustration, if an invoice is to be shared with a validating third party:
• The BUYER's Accredited Platform can send a status message with the invoice attached to the third-party validator, 
with a status including, for example, “Request for validation” as the expected action.
• The validator third party can respond with the appropriate status message (“Reviewed”, “Approved”, “In dispute”,
etc.).
SELLER
BUYER
SELLER’S 
AGENT
INVOICER
PAYEE
PAYER
BUYER’S AGENT
THIRD-PARTY 
AGENT
PA-SELLER AND 
INVOICER
PA-BUYER AND 
Third-party Agent
E-invoice
Lifecycle statuses
BUYER's electronic invoicing address : 
SIRENacheteur_GESTIONTIERS
PA
SC, OD, IS, ERP, …

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page36 / 149
• If the invoice must then be paid by a third party, the VAT Registered entity may send a “Payment Request” status 
via its Approved Pivot Platform to the PAYER third party, who may respond with a “Payment Sent” status.
The same mechanism can be implemented in the case of factoring after transmission of invoices (see use case No. 10). In 
this case, the SELLER's Accredited Platform is the pivot platform:
• The pivot platform transmits a “Factored” status, attaching the invoice. This status contains the history of the life
cycle statuses.
• Then, for each status received by the pivot platform, the equivalent status is transmitted to the third-party factoring 
platform.
The use of PEPPOL as a messaging system associated with the life cycle message (CDAR as described in Standard XP Z12-
012) allows these exchanges to be organized.
To do this, additional status values (to be positioned in MDT-105 of the life cycle message), as well as expected actions (in 
MDT-121) can be defined to standardize these exchanges.
In practical terms, PEPPOL exchanges operate on the basis of electronic addresses to be entered in an envelope (SBDH), 
identifying on the one hand the sender (known as C1), whose electronic address (“EndPoint”) corresponds to the address 
to which replies must be sent, and on the other hand the recipient (known as C4), to whom the life cycle status message is 
addressed. The PEPPOL network protocol then uses these electronic addresses ( “EndPoints”) to determine which PEPPOL 
Access Point (i.e., the recipient's platform) is responsible for the recipient's address and thus transmits the status message 
securely.
Again, by way of illustration, if a BUYER wishes to send a payment request to a third-party PAYER:
• The envelope (SBDH) will contain, for example, 0225:SIRENAcheteur_CDVTIERS as the C1 address, then 
0225:SIRENPayeur_CDVTIERSPAYEUR (or any other “EndPoint” electronic address accepted in the PEPPOL network) 
as the third-party payer address (C4).
• The Life Cycle message will identify:
ü The Buyer as the issuer of the message (MDG-16)
ü The Third Party PAYER as the Recipient (MDG-23),
ü The invoice as the “Subject of the response” (MDG-32), in which:
§ The identification details of the invoice to be paid will be provided (Invoice Number, Invoice Date, and SELLER 
ID).
§ The invoice will be attached (MDT-96).
§ A status code meaning “Payment Request” will be provided in MDT-105.
§ And if necessary, the status detail block can be used with the MDG-43 block to provide more details, such 
as the amount to be paid and the bank details of the payment beneficiary.
With these principles presented, this chapter will be completed in a future version of the status sets and required actions, 
as well as how to provide all the necessary context elements through the Life Cycle message.
3.2.1.3 Additional life cycle statuses
Analysis of use cases, and in particular what is described in the section on sharing with third parties above, reveals the need 
to add additional life cycle statuses alongside the processing statuses, to which the status "Cancelled" (“Annulée”) shall also 
be added, namely:
• For factoring: "Factored," "Factored Confidential," "Not Factored," and "Change of Account Payable."
• For co-contracting with direct payment: "Direct Payment Request"
• And others to come in a later version of this document.
3.2.2 Case No. 1: Multi-order/Multi-delivery
The EN16931 standard does not allow the transmission of multi-order/multi-delivery invoices.
In order to be able to transmit these invoices, extensions have been added to manage multiple orders/deliveries on the 
invoice. It is therefore possible to refer to an order or a shipping notice, or to specify a delivery address at the invoice line 
level (block BG-25). 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page37 / 149
The following information is concerned (see description of formats for the EXTENDED-CTC-FR profile):
• Order number on the line (EXT-FR-FE-135 – Order ID relating to the invoice line)
• Shipping notice number on the line (EXT-US-FE-140) and shipping notice line (EXT-US-FE-141).
• Delivery information in the event of different delivery addresses: 
ü Delivery address details per line (multi-delivery management) (block EXT-FR-FE-BG-10);
ü Global location identifier (EXT-FR-FE-146) and its scheme identifier (EXT-FR-FE-148), for example 0088 for a GLN;
ü Place name (EXT-FR-FE-149) - Name of the delivery location;
ü Address lines (EXT-FR-FE-150) - Postal delivery address block per line (3 lines, postal code, city, country 
subdivision, country code).
This makes it possible to create multi-order invoices by entering the orders on the line, or multi-delivery invoices by entering 
the shipping notices and delivery addresses on the line (if there are multiple), or even multi-order AND multi-delivery 
invoices.
It is permitted to create periodic invoices, i.e., to invoice a set of services or deliveries of goods. This can be done with a 
single order and a single delivery address, without the need to provide shipping notices. In this case, it is not necessary to 
use the ability to provide the items listed above.
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles.
Obligations of the SELLER and BUYER:
Ø Know how to process the minimum base formats and profiles, in particular the EXTENDED-CTC-FR profile.
3.2.3 Case 2: Invoice already paid by the BUYER or a third-party PAYER at the time the invoice is issued
This could be, for example, an invoice sent after the delivery of an order paid for at the time of ordering (within a short 
period of time so that the payment is not considered an advance payment, which should have been the subject of a prepayment invoice, followed by a final invoice; see use cases 20 and 21). It may also be an invoice for which the Seller has a 
debt to the Buyer (for example, for an overpayment on a previous invoice) equal to at least the total amount with VAT of 
the invoice and wishes to settle this debt for the total amount with VAT of the invoice.
The specific characteristics of the data and associated management rules are as follows:
• "Cadre de Facturation" (Business process type: BT-23) "Submission of an invoice already paid" (B2 / S2 / M2) or 
"Cadre de Facturation" B1 / S1 / M1;
• Amount paid (BT-113) equal to the total amount with VAT of the invoice (BT-112);
• Therefore, the Amount payable (BT-115) is equal to 0;
• The Due date (BT-9), if present, is equal to the invoice payment date (prior to or equal to the invoice date);
• If VAT is payable upon payment receipt (service invoice ("Cadre de Facturation" S2 or S1) for which the SELLER has 
not opted for debits, i.e. with BT-8 absent, or present and meaning "upon payment receipt" (72 in UN/CEFACT CII 
and 432 in UBL)), the SELLER MUST send a "Payment Received" life cycle status (via a Life cycle 6 flow) to its PA-E 
so that it can be sent to the PPF's CdD and the BUYER's PA-R. This status MAY be sent at the same time as the 
invoice is issued and MUST be sent during the e-reporting period for payment data (and therefore no later than the 
end date of the e-reporting payment period, for example the 10th of the following month in the case of the standard 
VAT regime);
• If the invoice has been paid by a Third-Party PAYER, this MAY be indicated in the invoice in the PAYER block (EXTFR-FE-BG-02). 
NOTE: pre-payment invoices after an advance payment are also invoices which have already been paid, which have the 
particularity of always being subject to VAT upon receipt, even if they are pre-payment invoices for the sale of goods (see 
use cases 20 and 21) and must therefore always be subject to a “Payment Received” status.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page38 / 149
Figure11: Invoice already paid by the BUYER or a third-party PAYER 
In the event that the Buyer requests a CREDIT NOTE on an invoice that has already been paid, there are two possible 
scenarios:
• The BUYER wants a credit note and a new invoice corrected (for example, in the event of an error in certain data). 
In this case, it is preferable that the CREDIT NOTE also be a CREDIT NOTE that has already been paid (i.e., with BT113 = BT-112 and BT-115 = 0), completely cancelling the invoice that has already been paid (including the lettering 
with the payment), followed by a new invoice corrected, which has also already been paid.
• The BUYER wants the invoice cancelled and the payment refunded:
ü If the refund is not made: the CREDIT NOTE cancels the invoice and leaves a Net Amount Payable (BT-115) equal 
to the total with VAT amount (BT-112), which means that the SELLER must pay the Net Amount to be Paid (BT115) to the BUYER (and, if applicable, keeps it on account for the payment of another invoice).
ü If the CREDIT NOTE is made after the refund, then it is itself already paid (and therefore with BT-113 = BT-112 
and BT-115 = 0).
Obligations of PA-E and PD-R:
Ø Know how to handle minimum base formats and profiles.
SELLER obligations:
Ø Create the "Payment Received" status in parallel with the invoice, if VAT is due upon receipt of payment, and 
transmit it via its PA-E.
Optional PA-E feature:
Ø Propose the creation of the "Payment Received" life cycle message on behalf of the SELLER in the case of an 
invoice that has already been paid, based on the invoice data (BT-8 value, "Cadre de Facturation" S2/B2/M2, BT9 for the date, VAT footer, etc.).
BUYER
PA-R
SELLER
PA-E
Creation of the invoice already paid
1
Receipt of invoice
3
CdD
PPF
Receipt of "Payment received" status Receipt of invoicing data (Flow 1) 7
and "Submitted", "Rejected", "Refused" statuses
2
Update of "Payment received" status
6b
Receipt of "Payment received" status 6c
Payment of the transaction
5
Transmission of Flow 1, the invoice 
and corresponding status
2
Payment receipt
6a

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page39 / 149
3.2.4 Case 3: Invoice payable by a third-party PAYER known at the time of invoicing
The invoice is sent by the SELLER to the BUYER, who is responsible for forwarding it to the third-party PAYER after settlement 
or validation. 
The steps in case no. 3, illustrated in the diagram below, are as follows:
Step Name of step Responsible 
actor Description
1 Creation of the invoice for 
the buyer SELLER
The SELLER creates the invoice (flow 2) via its information system. It 
may entrust the creation of the invoice to an OD/SC (Compatible 
Solution) or to its PA-E. It sends it to its PA-E for processing.
2
Issuance of the invoice and 
transmission of flow 1 data 
to the PPF
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the "Submitted" status to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
4a
4b
Processing of the invoice 
and updating of statuses BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses in its PA-R ("Rejected," "Accepted," "In dispute," 
"Approved," "Partially approved," "Suspended," etc.) for transmission 
to the SELLER via its PA-E.
4c Receipt of invoice statuses SELLER
The SELLER receives the invoice statuses following the processing of 
the invoice by the BUYER in accordance with the terms of the life 
cycle.
4d
Notification of the ThirdParty PAYER of the 
Approval of the invoice for 
payment
BUYER
The BUYER may provide the PAYER with information on the correct 
processing of the invoice and on the account to be paid (the one 
indicated on the invoice or modified during the life cycle, for example 
in the case of factoring), thereby triggering its payment (PA-R 
interfaceó Third-party PAYER, directly or via its OD/SC (Compatible 
Solution) or its Accredited Platform, to be implemented). 
5a
5b
Payment of the invoice
Information for the BUYER
Third-party 
PAYER
The PAYER pays the invoice and informs the BUYER of this payment 
and its terms: payment method, account paid, payment reference (if 
applicable) (PA-R interfaceó OD/SC (Compatible Solution)/Approved 
PAYER Platform to be implemented or managed outside the circuit).
5c Issuance of the "Payment 
Transmitted" status by the 
BUYER to the SELLER 
BUYER The BUYER sends a "Payment Transmitted" status to the SELLER.
5d Receipt of "Payment 
Transmitted" status SELLER / PA-E The SELLER receives the "Payment Sent" status from their PA-E.
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status SELLER
If the VAT on the invoice is payable upon Payment receipt, the SELLER 
creates the " Payment Received" status and transmits it to the PPF 
CdD via its PA-E. The PA-E also transmits the " Payment Received" 
status to the PA-R for the attention of the BUYER.
6c
6d
Receipt of the Paid status 
by the BUYER and sharing 
with the PAYER
BUYER The BUYER receives the " Payment Received" status and informs the 
third-party PAYER.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page40 / 149
Step Name of step Responsible 
actor Description
7
Receipt of "Payment 
Received" status by the 
CdD-PPF
CdD PPF The PPF Data Concentrator (CdD PPF) receives the "Paid" status.
Figure12: Invoice to be paid by a third party designated for invoicing
Using the EXTENDED-CTC-FR profile, it is possible to name the PAYER third party in the invoice:
• Block EXT-FR-FE-BG-02, and in particular:
ü Its company name (EXT-FR-FE-43)
ü Its legal identifier (EXT-FR-FE-48): SIREN number, qualified with 0002 (EXT-FR-FE-49), optional
ü One or more identifiers (EXT-FR-FE-46) with its identification scheme qualifier (EXT-FR-FE-47), which allows you 
to indicate a GLN, DUNS, SIRET, etc. See the list of ISO6523 codes for base formats.
ü An electronic address (EXT-FR-FE-52), which can be very useful for contacting the PAYING third party directly if 
necessary (see use case no. 13 for Subcontracting with direct payment from the end Buyer).
Exchanges between the BUYER and the third-party PAYER can be organized as long as the third-party PAYER is listed on the 
PA-R as a third-party PAYER and has direct access or access via its own platform (OD/SC (Compatible Solution) or PA) to the 
invoice and its life cycle. Where applicable, the BUYER may delegate the right to set the status to "Payment Transmitted" to 
the third-party PAYER on its behalf.
If the SELLER is informed of the existence of a third-party PAYER, referencing it in the invoice may allow the BUYER to 
organize its processing procedure to give the third-party PAYER access to the invoice and its life cycle once it is named in the 
invoice.
However, in this use case where the third-party PAYER is linked solely to the BUYER's organization, best practice is to let the 
BUYER manage its relationship with the third-party PAYER in order to make this specificity transparent to the SELLER, and 
not to ask the SELLER to manage this third-party PAYER in its invoices and therefore in its information system.
NOTE: Only one Third-Party Payer can be entered in the invoice in the EXT-FR-FE-BG-02 block, which is not repeatable.
NOTE: in some cases, the THIRD-PARTY PAYER can also connect directly to the PA-E, if they are referenced and have rights 
granted by the Issuer on the PA-E. In particular, they must have access to the life cycle statuses to ensure that the invoice is 
approved.
NOTE: if the third-party PAYER is an individual (e.g., an employee), care must be taken to comply with GDPR regulations. 
See use cases 5, 6, and 7.
Third-party PAYER
SC/OD (or PA) 
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice
3
Payment of invoice
5a
Collection of the invoice and reconciliation 
6a
Information on payment completion
5b
Information on invoice collection
6d
Update of "Payment received" status
6b
Receipt of "Payment received" status
6c
Receipt of invoicing data (Flow 1)
and "Submitted", "Rejected", "Refused" statuses
2
Receipt of "Payment received" status
7
Information on the successful processing of the invoice
4d
Processing of the invoice
4a
Receipt of invoice statuses
4c
Transmission of Flow 1, invoice F1 
and corresponding status
2
Creation of the invoice
1
Update of statuses
4b
"Payment Sent" status
5c Receipt of "Payment Sent" status
5d

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page41 / 149
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles.
Obligations of the BUYER:
Ø Organize interaction with their third-party PAYER, either by listing them on their PA-R as having delegation to 
access invoices that the BUYER has decided to have paid by this third-party PAYER, or by organizing within their 
OD/SC (Compatible Solution) or information system the necessary interconnections for the payment instruction 
of invoices and the return of execution of these payments for the creation of the "Payment transmitted" status.
Optional PA-R feature:
Ø Offer a service enabling third parties to be referenced for its "BUYER" customers, giving them access via various 
channels (Portal, standard API, EDI), associated with a delegation to open access to invoices to be paid by a thirdparty PAYER, and to their life cycle, and a right to create a "Payment transmitted" status.
3.2.5 Case 4: Invoice payable by the buyer and partially covered by a third party known at the time of invoicing 
(subsidy, insurance, etc.)
This management case covers invoices partially paid by a third party directly to the SELLER (for example, a repair invoice 
where the deductible, and VAT if deductible, are paid by the BUYER, with the balance being paid in parallel by the insurer).
Under the current provisions of standard EN16931 and the CII and UBL formats, it is not possible to indicate multiple thirdparty PAYERS, which would allow the various payments and parties involved to be identified.
However, it is possible to indicate a third-party PAYER (only one) in the PAYER block (Block EXT-FR-FE-BG-02) of the 
EXTENDED-CTC-FR profile, with the BUYER also retaining the option to transmit a payment. 
The specific features of the data and the associated management rules are as follows: 
• The SELLER block (BG-4) is used to provide information about the SELLER;
• The BUYER block (BG-7) is used to provide information about the BUYER who is to pay the invoice (e.g., the company 
that has to pay a deductible);
• The INVOICE PAYER block (EXT-FR-FE-BG-02) is used if you wish to mention the third-party PAYER (e.g., the insurer) 
on the invoice. 
• The AMOUNT PAID field (BT-113) MUST then be used to enter the amount of the invoice that has already been paid 
or that will be paid by a third-party PAYER requested directly by the SELLER (e.g., the amount of the invoice covered 
by the insurer).
• This will ensure that the "Net amount payable" (BT-115) is correct, equal to the total amount with VAT of the invoice 
(BT-112), minus the amount already paid (BT-113, also used to indicate an amount to be paid by a third party).
• The INVOICE NOTE block (BG-1) can be used to indicate that part of the invoice has already been paid or will be 
paid by one or more third parties. More specifically, in the BILL NOTE SUBJECT CODE field (BT-21), the SELLER must 
enter the code "PAI," which allows payment information to be indicated.
• The payment data expected by the administration corresponds to the date of receipt and the amount received. 
Therefore, the SELLER may declare a single payment if they receive the entire amount of the invoice on the same 
day. If they receive two payments on different days, they must declare two payments, one from the BUYER and the 
other from the third-party PAYER known at the time of invoicing.
• However, care must be taken with the VAT details provided, as the payment statuses must contain the VAT details 
for each payment. Thus, in the specific case of a partial payment by an insurer received at the same time:
ü If the insurer reimbursed VAT that was otherwise deductible, this would amount to a gain for the company 
equal to the VAT paid by the insurer. Consequently, in this case, the insurer pays the amount without VAT minus 
the deductible, and the BUYER must pay the deductible and all VAT on the invoice.
ü The e-reporting of the amount paid by the BUYER MUST therefore indicate at least two blocks of expected 
payment information (MDG-43 of the CDAR message, with block type code MDT-207 equal to "MEN" meaning 
"amount with VAT collected"):

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page42 / 149
§ A positive payment received of the amount with VAT (MDT-215), with the applicable VAT rate (MDT-224). 
In the event of multiple VAT rates, there must be as many records of amounts received as there are 
applicable VAT rates.
§ A negative payment received of the amount to be paid by the insurer and to which VAT does not apply (MDT215 = amount to be paid/paid by the insurer, MDT-224 = 0)
ü E-reporting of the amount paid by the insurer that is not subject to VAT: MDT-215 = amount paid by the insurer 
and MDT-224 (applicable VAT rate) = 0.
The steps in case no. 4, illustrated in the diagram below, are as follows: 
Step Name of step Responsible 
actor Description
1 Creation of the invoice 
for the buyer SELLER
The SELLER creates the invoice (flow 2) via its information system. It 
may entrust the creation of the invoice to an OD/SC (Compatible 
Solution) or to its PA-E. It sends it to its PA-E for processing.
The SELLER specifies in their invoice the amount paid by the third party 
(completed blocks AMOUNT ALREADY PAID (BT-113) and THIRD-PARTY 
PAYER (EXT-FR-FE-BG-02)). 
2 Transmission of flow 1 
data PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active electronic 
invoicing address for the recipient, it MUST transmit the data required 
by the Administration (flow 1) to the CdD PPF. It MUST also transmit the 
invoice (flow 2 or flow 3) to the PA-R of the BUYER. It must send the 
status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses (here 
"Received," then "Made Available") and makes the invoice available to 
the BUYER for processing.
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding processing 
statuses in its PA-R ("Rejected," "Accepted," "In dispute," "Approved," 
"Partially approved," "Suspended," etc.) for transmission to the SELLER 
via its PA-E.
4c Receipt of invoice 
statuses SELLER The SELLER receives the invoice statuses following the processing of the invoice by the BUYER in accordance with the terms of the life cycle.
4d
Notification of the thirdparty PAYER for payment 
of their share
SELLER
The SELLER informs the third-party PAYER that the invoice has been 
sent to the BUYER, including details of the life cycle status where 
applicable, and requests payment of the portion covered by the thirdparty PAYER.
5a
Payment of the balance 
of the invoice by the 
BUYER and transmission 
of the "Payment 
transmitted" status.
BUYER 
/ PA-R
The BUYER pays the balance of the invoice to the SELLER. They may 
send a "Payment Sent" status to the SELLER via the PA-R 
(recommended).
5b
Payment of the portion 
of the invoice covered by 
the third-party PAYER
Information for the 
SELLER
Third-party 
PAYER
The third-party PAYER pays the invoice and informs the SELLER, for 
example by sending a "Payment Transmitted" status (recommended).
6a
Payment Receipt of the 
invoice amount paid by 
the buyer
SELLER The SELLER receives the invoice amount paid by the BUYER.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page43 / 149
Step Name of step Responsible 
actor Description
6b
Payment Receipt of the 
invoice amount paid by 
the third party
SELLER The SELLER receives the invoice amount paid by the third-party PAYER.
6c
Issuance of the "Payment 
Received" status for the 
receipt of the portion 
paid by the BUYER
SALES 
ASSISTANT 
/ PA-E
If the VAT on the invoice is payable upon receipt of payment, the 
SELLER creates the "Received" status corresponding to the amount paid 
by the BUYER, with the correct VAT details, and transmits it to the CdD 
PPF via its PA-E. The PA-E also transmits the "Received" status to the 
PA-R for the attention of the BUYER.
6d
Issuance of the "Payment 
Received" status for the 
receipt of the portion 
paid by the third-party 
PAYER
SELLER 
/ PA-E 
If the VAT on the invoice is payable upon Payment receipt, the SELLER 
creates the " Payment Received" status corresponding to the amount 
paid by the third-party PAYER, with the correct VAT details, and 
transmits it to the CdD PPF via its PA-E. The PA-E also transmits the "
Payment Received" status to the PA-R for the attention of the BUYER.
6 Receipt of the "Received" 
status by the BUYER
BUYER
PA-R The BUYER receives the " Payment Received" status(es).
7 Receipt of "Paid" 
status(es) by the CdD PPF
PPF Data 
Concentrator The PPF Data Concentrator (CdD PPF) receives the "Paid" status(es).
Figure13: Invoice payable by the buyer and partially covered by a third party known at the time of invoicing (subsidy, insurance, etc.)
NOTE: Only one THIRD-PARTY PAYER can be entered in the invoice in block EXT-FR-FE-BG-02, which is not repeatable.
NOTE: in some cases, the THIRD-PARTY PAYER can also log in directly to the PA-E if they are referenced and have rights 
granted by the Issuer on the PA-E. In particular, they must have access to the life cycle statuses to ensure that the invoice is 
approved. This may be the case for an insurer who pays the SELLER directly for the portion that concerns them (the total 
amount with VAT minus VAT and any deductible).
Obligations of PA-E and PD-R:
Ø Know how to process the minimum base formats and profiles.
Third-party PAYER
SC/OD / PA
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Creation of the invoice for the buyer
1
Receipt of invoice 3
Update of "Payment 
received" status
Payment of the invoice amount by the third party
5b
Payment receipt of the 
invoice amount Payment 
sent by the third party
Payment receipt of the 
invoice amount Payment 
sent by the buyer
Invoice information
4
b
4d
Status update 4b
6b 6a
6d
E-reporting of the amount paid by the third party
E-reporting of the amount paid by the buyer
Receipt of invoicing data (Flow 1) 
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Receipt of "Payment received" status
6
Transmission of Flow 1, invoice 
and corresponding status
2
Processing of the invoice
4
b
4a
Payment of the invoice amount by the buyer
5a
Update of "Payment 
received" status
6c
Receipt of statuses
4c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page44 / 149
Obligations of the SELLER:
Ø Organize interaction with the third-party PAYER, either by referencing them on their PA-E as having delegation 
to access invoices for which the SELLER wishes to request payment by the third-party PAYER, or by organizing 
within their OD/SC (Compatible Solution) or its information system the necessary interconnections for payment 
requests and payment information returns in order to be able to create the " Payment Received" status where 
applicable, or by processing this Payment receipt with the third-party PAYER by any other means, integrating its 
obligation to issue the "Payment Received" status.
Optional PA-R functionality:
Ø Offer a service that allows third parties to be referenced for its "SELLER" customers, giving them access through 
various channels (portal, standard API, EDI), combined with a delegation to open access to invoices that must be 
paid by a third-party PAYER at the request of the SELLER, and to their life cycle.
3.2.6 Case No. 5: Expenses paid by employees with invoices in the company's name
This management case covers expenses advanced by an employee in the course of their professional activity and for which 
an invoice in the company's name has been issued. In this case, the employee has advanced the expenses and the company 
reimburses them.
This case is only valid if the invoice paid by the employee is made out in the company's name and is therefore subject to 
electronic invoicing. 
In practice, the employee provides the electronic invoicing address where their company wishes to receive these business 
expense invoices, as well as their company's SIREN number (normally the beginning of the address in the form SIREN_XXX 
or just SIREN). A dedicated address for this type of invoice, for example SIREN_EMPLOYEEEXPENSES, may be an option, 
especially if the company has many employee expenses to manage. The employee also provides the SELLER with information 
to be included in the invoice that will enable their company to identify them or link them to their receipt (receipt number, 
employee ID number, last 6 digits of the credit card used for payment, etc.).
As the invoice has already been paid, the Amount already paid (BT-113) is equal to the Total amount with VAT (BT-112) and 
the Net amount payable (BT-115) is equal to 0.
The employee is then considered a third-party payer. It is possible to enter the employee's name, or preferably an ID or 
employee number to avoid naming individuals on invoices (GDPR):
• When using the EXTENDED-CTC-FR profile, in the EXT-FR-FE-43 field (company name of the third-party PAYER) 
located in the "INVOICE PAYER" block (EXT-FR-FE-BG-02).
• When using the EN16931 profile, it is possible to use the BT-18 data (invoiced item) with the Item code (BT-18-1) 
equal to AHK (payer reference).
The SELLER may provide the employee with a legible copy of the invoice to enable them to justify their reimbursement.
Figure14: Expenses paid by an employee, invoice in the name of the company
SELLER
PA-E PA-R
BUYER (Company)
Transmission of Flow 1, invoice and corresponding 
status
2 Receipt of the invoice 3
Third party 
(Collaborator)
Transaction payment
5
Payment receipt
6a
Update of "Payment received" status
6b
Receipt of "Payment received" status 6c
Creation of the invoice (“Cadre de Facturation” n°7)
1
PPF Receipt of "Payment received" status Transmission of invoicing data 7
and "Submitted", "Rejected", "Refused" statuses
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page45 / 149
Obligations of PA-E and PD-R:
Ø Know how to process the minimum base formats and profiles.
3.2.7 Case No. 6: Expenses paid by employees without an invoice addressed to the company (simple receipt or invoice 
made out to the employee's name and address)
The other management case corresponds to the case where the SELLER provides either an invoice or a receipt to the 
Employee who pays, and in their name. 
This sale is then made between the SELLER, who is subject to tax, and the employee, who is a private individual not subject 
to tax. It is not subject to the electronic invoicing obligation within the meaning of Article 289 bis of the CGI, but to the ereporting obligation for the SELLER.
The SELLER must therefore declare this sale in its B2C e-reporting, and therefore in its daily total for the day in flow 10.3, 
then, if VAT is due upon receipt of payment, in flow 10.4.
The employee may send a copy of the invoice to their employer to request a refund if the purchase is justified. The 
deductibility of VAT for the BUYER must be specified, as the BUYER does not have an invoice addressed directly to them, 
which is normally mandatory for a sale between VAT-registered parties in France.
The Employee or BUYER may nevertheless request a B2B electronic invoice at a later date, which the SELLER can provide 
with a Business Process Type “Cadre de Facturation” (BT-23) type 7 (S7, B7), which means for the tax authorities "VAT 
already collected as part of e-reporting, but which may be deductible for the BUYER"; see use case 30.
Figure15: Expenses paid by an employee, invoice in the employee's name
Obligations of the PA-E:
Ø Transmit the e-reporting flow based on the information provided by the SELLER.
3.2.8 Case No. 7: Invoice following a purchase paid for with a corporate card (purchasing card) 
This case corresponds to purchases made by employees of the BUYER using a corporate card, generally used for all travel 
expenses, hotel rooms, train tickets, etc. 
The SELLER is paid through the use of the corporate card. They send an invoice that has already been paid to the BUYER. 
The BUYER receives a monthly statement of payments for all purchases made by their employee from various SELLERS, 
which they pay to the corporate card operator. However, they may refuse certain purchases. In this case, the SELLER enters 
into a dispute or refusal management process.
With regard to invoices, the operation of corporate cards is the same as in case no. 2: invoice already paid by the BUYER, 
except that there is no third-party PAYER in this case (the corporate card being a means of payment for the BUYER).
The specific characteristics of the data and associated management rules are therefore as follows:
• Billing framework (“Cadre de Facturation”) "Submission of an invoice already paid" (B2/S2/M2) or standard "Cadre 
de Facturation" B1/S1/M1;
SELLER
PA-E
PPF Receipt of payment data (daily total of receipts)
Addition of the sale to the transmission of Flows 10.3 
and 10.4
2
Third party 
(Employee)
Payment of the transaction
5
Payment receipt
6a
Receipt of invoice 3a
7
Receipt of invoice
3b
Receipt of e-reporting data (daily sales total)
2
BUYER (Company)
SC/OD /PA
Creation of the invoice (for the employee) 1

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page46 / 149
• As payment is made by credit card, the payment transaction information must be provided: BT-87: last 4 to 6 digits 
of the card, and potentially (optional) the cardholder's name in BT-88.
• Amount paid (BT-113) equal to the total amount with VAT of the invoice (BT-112);
• Therefore, the Amount to be paid (BT-115) is equal to 0;
• The Due date (BT-9), if present, is equal to the invoice payment date (prior to or equal to the invoice date).
• If VAT is payable on payment receipt (service invoice ("Cadre de Facturation" S2 or S1) for which the SELLER has 
not opted for debits, i.e. with BT-8 absent or present and meaning "on payment receipt" (72 in UN/CEFACT CII and 
432 in UBL)). The SELLER must send a "Payment Received" life cycle status (via a Life cycle 6 flow) to its PA-E so that 
it can be sent to the PPF and the BUYER's PA-R. This status MAY be sent at the same time as the invoice is issued 
and MUST be sent during the e-reporting period for payment data.
The steps in case no. 7, illustrated in the diagram below, are as follows:
Step Name of step Responsible 
party Description
0 Purchase BUYER An employee of the BUYER purchases a product/trip from the SELLER 
using a purchase card or corporate card.
2b Payment for the 
purchase SELLER The SELLER transmits the payment information to the Card Manager to execute the payment.
5a Payment for the 
purchase
CARD
CARD The CARD MANAGER pays the SELLER on behalf of the BUYER.
6a Receipt of payment SELLER The SELLER receives the purchase amount.
1 Creation of the invoice SELLER
The SELLER creates the invoice (flow 2) via their information system. 
They can entrust its creation to an OD/SC (Compatible Solution) or to 
their PA-E. They send it to their PA-E for processing. This is an invoice 
that has already been paid.
2a
Transmission of flow 1, 
the invoice (flow 2), and 
the related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and for the existence of an active 
electronic invoicing address for the recipient, it MUST send the data 
required by the Administration (flow 1) to the CdD PPF. It MUST also 
send the invoice (flow 2 or flow 3) to the PA-R of the BUYER. It must 
send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Payment Received" then "Made Available") and makes the 
invoice available to the BUYER for processing.
6b Issuance of "Payment 
Received" status
SELLER 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER creates 
the "Received" status and transmits it to the CdD PPF via its PA-E. The 
PA-E also transmits the "Received" status to the PA-R for the 
attention of the BUYER.
7
Receipt of "Payment 
Received" status by the 
CdD-PPF
CdD PPF The PPF Data Concentrator (CdD PPF) receives the "Payment 
Received" status.
4a Transmission of the 
transaction statement
CARD 
MANAGER
CARD
The CARD MANAGER transmits the transaction statement to the 
BUYER (excluding exchanges between PAs). 
4b
Reconciliation between 
the statement and the 
invoice
BUYER The BUYER reconciles the invoices received from SELLERS with the 
transaction statement (excluding PA scope).
4c Settlement of the 
transaction statement BUYER The BUYER settles the transaction statement, partially, if necessary, in the event of a dispute over certain invoices.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page47 / 149
4d
Processing of the invoice 
and updating of its status 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses in its PA-R ("Rejected," "Accepted," "In dispute," 
"Approved," "Partially approved," "Suspended," etc.) for transmission 
to the SELLER via its PA-E.
4e Receipt of invoice 
statuses
SELLER
The SELLER receives the invoice statuses following the processing of 
the invoice by the BUYER in accordance with the terms of the life 
cycle.
Figure16: Invoice following a purchase with a corporate card
In the event of a refusal, dispute, or partial approval of the invoice, the resolution is as described in Chapter 2. Once the
payment is canceled, the SELLER MUST set a negative "Payment Received" status to cancel the initial status.
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles.
Obligations of the SELLER:
Ø Transmit the "Payment Received" status as soon as it receives payment from the CARD MANAGER, within its ereporting period (before the 10th of the following month for companies under the normal regime).
Ø In the event of a payment cancellation, transmit a "Payment Received" cancellation status with negative 
amounts.
Optional PA-E feature:
Ø Propose the creation of a " Payment Received " life cycle message on behalf of the SELLER in the event of an 
invoice that has already been paid, based on the invoice data (BT-8 value, "Cadre de Facturation" S2/B2/M2, BT9 for the date, VAT footer, etc.).
BUYER
PA-R
Third party: Card manager
SC/OD
SELLER
PA-E
CdD
PPF
Receipt of invoicing data (Flow 1)
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Information on payment transaction data 2b Payment of the transaction 5
Receipt of invoice
3
Lodge card
Reconciliation between statement and invoice
Payment of statement amount
4b
Update of "Payment received" status
6b
Creation of the invoice
1
Payment receipt of the transaction
6a
Transmission of Flow 1, the invoice 
and corresponding status
2a
Treatment statuses
4d
Receipt of invoice statuses
4e
Purchase of a product/trip to be paid 
for with the Corporate card
0
Transmission of information statement
4a
Payment receipt of the statement amount
4c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page48 / 149
3.2.9 Cases 8 to 10: Invoices payable to a third party (including factoring, cash pooling)
3.2.9.1 Cash pooling
If a SELLER is part of a group that has implemented cash pooling, they may designate a Third Party Payee for the payment 
of their sales invoices (the entity in charge of cash pooling) and provide their bank details as follows:
• Identification of the PAYEE in BG-10:
ü BT-59: company name of the third-party PAYEE
ü BT-60: Additional identifier of the third-party PAYEE Y, optional
ü BT-61: Legal identifier (SIREN) of the third-party PAYEE, optional, 
ü And in the EXTENDED-CTC-FR profile, it is possible to add, optionally:
§ EXT-FR-FE-26: the role code (mainly intended to identify a third-party PAYEE of the factoring company type)
§ EXT-FR-FE-27: VAT ID of the factoring company
§ EXT-FR-FE-28 and EXT-FR-FE-29: electronic address and its identification scheme, which may be useful for 
replicating the invoice and statuses through the network exchange system (PEPPOL interoperability, for 
example).
§ EXT-FR-FE-31, which is a data group: postal address of the PAYEE.
§ EXT-FR-FE-39, which is a data group: contact details of the PAYEE.
• Bank details to be paid in BG-17 (Transfer Information):
ü BT-84: IBAN to be paid (i.e., that of the third-party PAYEE)
ü BT-85: name of the payment account holder
ü BT-86: BIC
3.2.9.2 Focus on factoring management
Factoring is a credit transaction within the meaning of Article L. 313-1 of the Monetary and Financial Code. This regulated 
financial service, provided by specialized credit institutions or finance companies, is based on the purchase of commercial 
receivables. The legal basis for the transfer of receivables from the supplier to the Factor is contractual subrogation as 
provided for in the Civil Code, the assignment of professional receivables known as "Dailly assignment" as provided for in 
the Monetary and Financial Code, or the assignment of receivables as provided for in the Civil Code. Regardless of the basis, 
the Factor becomes the owner of the assigned receivable. 
When an invoice is subrogated, it is assigned to a factoring company, hereinafter referred to as the "Factor," which may 
result in the BUYER's payment being made directly to the Factor (in general cases), or to a specific bank account held by the 
SELLER with the Factor (in cases of confidential factoring, for example).
Depending on the offer, this assignment may be made for a set of invoices (e.g., for a given customer) or "on demand" per 
invoice.
The general principle is then to allow a third-party Factor access to each subrogated or assigned invoice and its life cycle
only for invoices on which the Factoris identified as a third-party Factor, either directly on the invoice (case no. 8), or through 
life cycle messages if the subrogation takes place after the invoice has been issued (case no. 10).
Several cases may arise:
• "Traditional" factoring: the invoice is a "factored invoice" (codes 393, 501 in BT-3), or "factored credit note" (codes 
396, 502 in BT-3) or "factored corrected invoice" (codes 472, 473), bearing in mind that the second codes (501, 502, 
473) correspond to cases of self-billing in addition to the "factored" nature. The Factor is identified as such in the 
invoice (in a "PAYEE" block, BG-10: Payment payee). 
• Factoring after the invoice has been sent: in this case, the Factor is not identified in the invoice, which can no longer 
be modified (otherwise, a credit note and a new invoice or a corrected invoice would have to be issued). The BUYER 
must therefore be informed of the change of ownership of the invoice and the change of payment destination.
• Confidential factoring: whether done before or after the invoice is issued, this type of offer means that the BUYER 
is not informed of the subrogation or assignment of the invoice. In general, the SELLER designates a bank account 
opened with its factoring company to which the BUYER makes the payment. This account is usually pre-configured 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page49 / 149
by the BUYER, like any SELLER's bank account. The SELLER may indicate to its BUYER during processing that it wishes 
to be paid into another bank account.
• Reverse factoring: this is an offer developed for large buyers and proposed to their SELLER suppliers, with proximity 
to the BUYER allowing access to invoice validation statuses, which makes the receivable more "certain."
• Change of Factoring Company: it may happen that the Factoring Company changes during the life of the invoice. In 
this case, a plan should be put in place whereby the invoice is transferred back to the SELLER and then transferred 
again to a new Factoring Company.
That being said, it is necessary to describe how the necessary exchanges between the SELLER and BUYER of invoices and the 
life cycle should be organized to meet the above requirements.
Furthermore, as the Factoring Companies are the owners of the invoices, they wish to have access to the life cycle statuses, 
particularly those relating to processing ("Dispute", "Approved", "Payment Sent", "Payment Received"), but also, in 
particular, those that result in the cancellation of the pre-filled VAT, which almost always results in the cancellation of the 
invoice through an INTERNAL CREDIT NOTE.
This is primarily a matter for the contractual relationship between the factoring company and the SELLER. 
However, Accredited Platforms («Plateformes Agréées») may offer additional optional features for sharing invoices and life 
cycle information with the factoring company, once it has been identified as the new owner of the invoice. This requires, at 
a minimum, that the factoring company be listed with the PA-E as being able to act on a SELLER's invoices, by delegation of 
rights subject to the transfer of the invoice to the factoring company.
The factoring company can then connect to the PA-E directly, via an OD/SC (Compatible Solution), or through its own 
Accredited Platforms («Plateformes Agréées»). In this case, however, the PA-E must also provide for the replication of 
invoices and statuses for the attention of the Factor, i.e., the transmission of invoices to the Factor's platform as soon as
they are factored, as well as the various life cycle messages exchanged with the BUYER through the SELLER's PA-E.
On the other hand, when invoices are actually paid to the Factor by the BUYER, the Factor MUST transmit the information 
to the SELLER, either by sending it to them or giving them access to it by any means, or through their e-invoicing platform, 
so that the latter can set the status to "Paid" when necessary.
3.2.9.3 Case No. 8: Invoice payable to a third party determined at the time of invoicing (factoring, cash pooling)
In case no. 8, the invoice must be paid to a third party specified at the time of invoicing. This may be an entity designated
by the SELLER, responsible for cash pooling. In this case, it is sufficient to designate this entity in the invoice in the Payee 
block (BG-10). 
It may also be a third-party factoring company, and the rest of this case study will focus on this scenario. It is identified on 
the invoice in the "PAYEE" block (BG-10). Using the EXTENDED-CTC-FR profile, it is possible to qualify the type of Payee using 
the "role code": EXT-FR-FE-26 with the value "DL" (role code from the UNTDID 3035 list for a "Factor").
If the SELLER receives refinancing for their invoice, this does not constitute a payment receipt because the BUYER has not 
paid this invoice. It is therefore only when the BUYER pays the invoice to the Factor that the invoice is deemed "Payment 
Received". The Factor must therefore inform the SELLER so that the latter can set the status to "Payment Received" if VAT 
is due upon payment receipt.
It may also be possible for the Factor to directly transmit the "Payment Received" status to the PPF's CdD on behalf of the 
SELLER. However, it must already use an Accredited Platform to do so, and also inform the SELLER (for its lettering) and 
potentially the BUYER. If the factoring company is prepared to prepare the " Payment Received" status on behalf of its client, 
the SELLER, best practice is for the factoring company to transmit it to the SELLER, either directly or to its PA-E on its behalf 
(and therefore with the delegation rights that allow it). The SELLER can then manage it and send it as they would any other 
invoice. Otherwise, the Factoring Company will continue to send or make the payment receipt information available to the 
SELLER outside the system. 
NOTE: In general, factoring companies consider that they should not take the place of their SELLER clients in their tax 
obligations. Furthermore, in order to remain within a simple regulatory framework controlled by the SELLER, it is preferable 
for the SELLER to remain in control of its exchanges of information with the tax authorities through its PA-E. Consequently, 
allowing a third party to transmit " Payment Received" statuses to the CdD PPF independently of the SELLER is not considered 
good practice.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page50 / 149
In this case no. 8, the following data must be included in the invoice (except in the case of confidential factoring, where the 
invoice remains a standard commercial invoice, without a factoring code and without the name of the Factor):
• Typecode 393 (or 501 in the case of self-billing) in BT-3.
• The identification of the Factor in BG-10 (PAYEE):
ü BT-59: company name of the Factor
ü BT-60: Additional identifier of the Factor, optional, but may be required by a Factor to identify it with certainty 
(e.g., with an LEI)
ü BT-61: Legal identifier (SIREN) of the Factor, optional, but may be required by a Factorto identify it with certainty
ü And in the EXTENDED-CTC-FR profile, it is possible to add, optionally:
§ EXT-FR-FE-26: the role code, with the value "DL" for a Factor
§ EXT-FR-FE-27: VAT identifier of the factoring company
§ EXT-FR-FE-28 and EXT-FR-FE-29: electronic address and its identification scheme, which may be useful for 
replicating the invoice and statuses through the network exchange system (PEPPOL interoperability, for 
example).
§ EXT-FR-FE-31, which is a data group: postal address of the PAYEE.
§ EXT-FR-FE-39, which is a data group: contact details of the PAYEE.
• Bank details for payment in BG-17 (Transfer Information):
ü BT-84: IBAN to be paid (i.e., that of the factoring company)
ü BT-85: name of the payment account holder
ü BT-86: BIC
• A reference to subrogation/assignment, in BG-1 (Note):
ü BT-21 (Subject code): ACC.
ü BT-22 (Content): text of the subrogation.
The description below complies with the principle of not exporting complexity to counterparties. The BUYER treats the 
invoice and statuses like any other invoice. It is the SELLER, and where applicable its PA-E, that acts as the hub for sharing 
or replication with the factoring company. 
The replication of invoices and statuses between the SELLER and the FACTOR is a matter for their commercial relationship. 
In particular, the PA-E may offer invoice and status sharing/replication services to a third-party FACTOR.
The steps in case no. 8, illustrated in the diagram below, are as follows:
Step Step name Responsible party Description
1 Creation of the invoice 
with mention of factoring SELLER
The SELLER creates the invoice (flow 2) via its information 
system. It may entrust the creation of the invoice to an OD/SC 
(Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
As the factoring company is known when the invoice is created, 
it is identified as the "PAYEE" in block BG-10. The payment 
information entered in block BG-17 "TRANSFER" corresponds to 
its bank details.
The invoice type in BT-3 must be set to 393 (factored invoice) or 
501 (self-factored invoice). 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page51 / 149
Step Step name Responsible party Description
2a Transmission of flow 1 
data PA-E
Once the PA-E has carried out the regulatory compliance 
checks, including checks for duplicates and the existence of an 
active electronic invoicing address for the recipient, it MUST 
transmit the data required by the Administration (flow 1) to the 
CdD PPF. It MUST also transmit the invoice (flow 2 or flow 3) to 
the PA-R of the BUYER. It must send the status "Submitted" to 
the CdD PPF.
2b
Transmission/sharing of 
the invoice with the 
factoring company
SELLER / PA-E
OD/SC (Compatible 
Solution)/Approved 
Factoring Company 
Platform
The SELLER shares/transmits the invoice to the factoring 
company.
Optional additional PA-E feature: organize sharing/replication.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 
3), performs regulatory checks, creates transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses in its PA-R ("Rejected," "Accepted," "In 
dispute," "Approved," "Partially approved," "Suspended," etc.) 
for transmission to the SELLER via its PA-E.
4c Transmission/sharing of 
life cycle statuses
SELLER / PA-E
OD/SC (Compatible 
Solution)/Approved 
Factoring Platform
The SELLER shares/transmits the life cycle statuses to the 
Factoring Company.
Optional additional PA-E feature: organize sharing/replication.
5a
5b
Payment of the invoice to 
the factoring company 
and transmission of the 
"Payment Transmitted" 
status to the SELLER.
BUYER 
/ PA-R
The BUYER pays the invoice to the Factor (PAYEE). They can 
send a "Payment Transmitted" status to the SELLER via the PA-R 
(recommended).
5c Receipt of "Payment 
Transmitted" status
SELLER 
/ PA-E
The SELLER receives the "Payment Transmitted" status from 
their PA-E.
5d
Transmission/sharing of 
"Payment Transmitted" 
status
SELLER / PA-E
OD/SC (Compatible 
Solution)/Approved 
Factoring Platform
The SELLER shares/transmits the "Payment Transmitted" status 
to the Factoring Company.
Optional additional PA-E feature: organize sharing/replication.
6a
Invoice payment receipt
and information from the 
SELLER
FACTORING 
COMPANY
The factoring company receives payment for the invoice 
(outside the circuit). 
It informs the SELLER of this payment receipt, either by sending 
it to them or by giving them access to it by any means.
Optional additional feature of PA-E: organize the ability to 
receive a " Payment Received" status prepared and sent by the 
Factoring Company to the SELLER.
6b Receipt of payment 
receipt information SELLER
The SELLER receives or has access to information about the 
payment receipt for their invoice by the Payee (the Factoring 
Company).
6c Issuance of " Payment
Received" status SELLER
If the VAT on the invoice is payable upon payment receipt, the 
SELLER creates the " Payment Received" status and transmits it 
to the CdD PPF via its PA-E. The PA-E also transmits the "
Payment Received" status to the PA-R for the attention of the 
BUYER.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page52 / 149
Step Step name Responsible party Description
6d Receipt of "Paid" status 
by the BUYER
BUYER
PA-R The BUYER receives the "Received" status.
7 Receipt of "Paid" status 
by the PPF CdD
PPF Data 
Concentrator The PPF Data Concentrator (CdD PPF) receives the "Paid" status.
Figure17: Invoice payable to a third party determined at the time of invoicing
Obligations of PA-E and PA-R:
Ø Know how to process minimum base formats and profiles.
SELLER obligations:
Ø Create the invoice with fields identifying the factoring company, its bank details, and the subrogation clause.
Ø Replicate/share the invoice and life cycle statuses with the factoring company.
Obligations of the factoring company:
Ø Allow the SELLER to access payment receipt information enabling them to create "Payment Received" statuses 
on their factored invoices.
Additional optional PA-E features:
Ø Offer an additional feature for managing third-party factoring companies identified on the PA-E and for which 
invoice and status viewing rights are granted.
Ø Offer an additional feature for initiating subrogation/transfer of invoices with a referenced factoring company.
Ø Implement an additional feature for replicating invoices and statuses to the factoring company, for example 
using the PEPPOL network, with the factoring company using a dedicated address for receiving replicated 
invoices and statuses for its business. The factoring company's access point MUST be based in the EU (and this is 
the case if it is a PA). Transmission can also be done through the Standard Data Exchange API.
Ø For Factoring Companies that wish to offer to prepare the "Payment Received" status on behalf of the SELLER 
and transmit it to its PA-E, set up an additional feature for receiving invoice “Payment Received” statuses 
SELLER
PA-E
BUYER
SC/OD / PA PA-R
Third Party PAYEE
CdD
PPF Receipt of "Payment received" status
7 Receipt of invoicing data (Flow 1)
and "Submitted", "Rejected", "Refused" statuses
2
Transmission/sharing of the invoice 
2b
Receipt of invoice
3
Receiving invoice statuses
4b
Receipt of "Payment received" status
6d
Payment receipt of the invoice 
and notification of the SELLER
6a
Processing of the invoice and status update
4a
Update of "Payment received" status
6c
Contract between the supplier and the factoring company 
(third-party beneficiary)
Creation of the invoice with mention of the factoring 
company
1
Creation of the invoice and transmission of the Flow 1,
of the invoice and corresponding status
2a
Payment of the invoice to the factoring company
5a Transmission/sharing of life cycle statuses
4c
"Payment sent" status
5b
Receipt of "Payment sent" status
5c
Transmission/sharing of life cycle statuses
5d
Invoice payment information
6b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page53 / 149
transmitted directly by the Factoring Company on behalf of the SELLER, and prepare the "Payment Received" 
status from this status so that the SELLER can send it to the CdD PPF and the BUYER.
3.2.9.4 Case No. 9: Invoice payable to a third party known at the time of invoicing, who also manages the order/receipt, 
or even invoicing (Distributor / Depositary)
This case is handled in the same way as case no. 8. The different roles (order, receipt, invoicing) can be added to the invoice 
flow (flow 2) as needed. They have no impact on the data to be sent to the administration (flow 1). 
However, this is not factoring, so there is no need to mention subrogation in the invoice.
Furthermore, it is not necessarily necessary to replicate invoices and statuses to the third-party payee (steps 2b and 4c). The 
only information that needs to be retained is the information provided by the payee to the seller regarding the actual 
payment receipt for an invoice.
You can refer to the diagram in case no. 8, "Invoice payable to a third party designated at the time of invoicing (factoring,
cash pooling)." 
3.2.9.5 Case No. 10: Invoice payable to a third-party payee unknown at the time the invoice was created, in particular 
a factoring company (case of subrogation)
The invoice is payable to a third party who is unknown at the time the invoice is created (for example, a factoring company 
for a subrogation made after the invoice has been sent). A contract must be drawn up between the SELLER and the thirdparty Payee before the latter can be declared to the BUYER as a potential payee. The payment of the invoice is received by 
the third-party payee or the SELLER in the case of confidential factoring. The SELLER remains responsible for updating the 
“Payment Received” status and transmitting it to the PPF's CdD.
The steps in case no. 10, illustrated in the diagram below, are as follows:
Step Name of step Responsible 
party Description
1 Creation of the invoice for 
the BUYER SELLER
The SELLER creates the invoice (flow 2) via its information system. It 
may entrust the creation of the invoice to an OD/SC (Compatible 
Solution) or to its PA-E. It sends it to its PA-E for processing. 
2a Transmission of flow 1 data PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses in its PA-R (statuses "Rejected," "Accepted," "In 
dispute," "Approved," "Partially approved," "Suspended," etc.), for 
transmission to the SELLER via its PA-E.
4c
The SELLER designates a 
new Payee to the BUYER. If 
applicable, it sets the status 
to "Factored."
SELLER
The SELLER decides to change the Payee. It informs the BUYER of this 
via a life cycle status, which, depending on the case, may also signify 
the subrogation/assignment of the invoice.
In the case of traditional factoring, this message indicates a 
"Factored" status, which is a status of the invoice and does not 
change the current transmission or processing status.
4d Receipt by the BUYER of 
the change of Payee status BUYER The BUYER receives the change of payee status and, where applicable, the "Factored" status.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page54 / 149
Step Name of step Responsible 
party Description
4e
Transmission/sharing of the 
invoice and life cycle
history
SELLER / PA-E
OD/ SC 
(Compatible 
Solution) / 
Approved 
Factoring 
Platform
The SELLER shares/transmits the invoice and life cycle statuses to the 
Factoring Company.
All life cycle statuses will also be shared/transmitted to the factoring 
company.
Optional additional PA-E feature: organize sharing/replication.
5a
5b
Payment of the invoice to 
the Factoring Company and 
transmission of the 
"Payment Transmitted" 
status to the SELLER.
BUYER 
/ PA-R
The BUYER pays the invoice to the PAYEE (Factoring Company) or to 
the SELLER in the case of Confidential Factoring (in practice, to the 
account indicated on the invoice or modified through a "Change of 
Payable Account" life cycle status). The BUYER may transmit a 
"Payment Transmitted" status to the SELLER via the PA-R 
(recommended).
5c Receipt of "Payment 
Transmitted" status
SELLER 
/ PA-E
The SELLER receives the "Payment Transmitted" status from their PAE.
5d
Transmission/sharing of 
"Payment Transmitted" 
status
SELLER / PA-E
OD/SC 
(Compatible 
Solution) / 
Approved 
Factoring 
Platform
The SELLER shares/transmits the "Payment Transmitted" status to the 
Factoring Company.
Optional additional PA-E feature: organize sharing/replication.
6a
Invoice payment receipt
and information for the 
SELLER
FACTOR
The Factoring Company receives payment for the invoice (off-circuit). 
It informs the SELLER of this payment receipt, either by sending it to 
them or by giving them access to it by any means.
Optional additional feature of PA-E: organize the ability to receive a 
"Received" status prepared and transmitted by the Factoring 
Company to the SELLER.
6b Receipt of payment receipt
information SELLER The SELLER receives or has access to information about the payment receipt for their invoice by the Payee (the Factoring Company).
6c Issuance of "Payment 
Received" status SELLER
If the VAT on the invoice is payable upon rpayment eceipt, the SELLER 
creates the " Payment Received" status and transmits it to the CdD 
PPF via its PA-E. The PA-E also transmits the " Payment Received" 
status to the PA-R for the attention of the BUYER.
6d
Receipt of " Payment 
Received " status by the 
BUYER
BUYER
PA-R The BUYER receives the "Received" status.
7
Receipt of " Payment 
Received " status by the 
PPF CdD
PPF Data 
Concentrator The PPF Data Concentrator (CdD PPF) receives the "Paid" status.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page55 / 149
Figure18: Invoice payable to a third party unknown at the time of invoicing
Management of different factoring cases through the status message between SELLER and BUYER:
• Traditional factoring: the life cycle status sent to the BUYER (step 4c) uses the "Factored" status (status code 225) 
and indicates the new bank details:
ü The new Payee is designated in the "Recipient" block of the CDAR status message (MDG-41):
§ MDT-155: identifier of the new Payee: SIREN number, and/or other identifier (MDT-156 gives the 
identification scheme code).
§ MDT-158: role code: "DL" for a factoring company.
§ MDT-178: e-mail address of the Factor.
ü The new bank details are provided in block MDG-43 (Data to be reported) of the status detail block (MDG-37), 
with one occurrence per piece of information transmitted (BT-84, BT-85, BT-86):
§ MDT-207 = CBB (Payee Bank Details to be modified)
§ MDT-206: indicates the identifier in the EN16931 semantic model of the invoice data to be taken into 
account, i.e.: "BT-84" for the IBAN; "BT-85" for the account holder's name; "BT-86" for the BIC)
§ MDT-214: new data to be taken into account (IBAN, account holder name, BIC respectively).
ü "Invoiced" status:
§ MDT-77: 23 (processing phase)
§ MDT-88: 5 (means "Information only")
§ MDT-105 (Processing status code): code 225, meaning "Factored"
§ MDT-106 (Processing status description): "Factored"
ü Subrogation/assignment notice: in the MDG-39 status note:
§ MDT-126 (Content): Subrogation/assignment notice
§ MDT-127 (Subject code): ACC
• Confidential factoring: in this case, it may be necessary to share a status between the Factor and the SELLER only, 
without informing the BUYER, to indicate this change in the status of the invoice through a life cycle status of 
"Factored Confidential" (code 226) that is NOT TO BE SENT to the BUYER (and therefore not to the PA-R). Next, it 
may be necessary to request that the invoice be paid to an account other than the one initially specified on the 
invoice. To do this, a life cycle status message is sent to the BUYER, indicating only a change in IBAN since the Payee 
remains the same (the SELLER):
ü The new bank details are provided in block MDG-43 (Data to be reported) of the status detail block (MDG-37), 
with one occurrence per piece of information:
Third party FACTOR
SC/OD / PA
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoicing data (Flow 1) 
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Creation of the invoice
1
Receipt of invoice
3
Processing the invoice and updating the status
4a
Receipt of processing status
4b
Transmission of Flow 1 and issuance of the invoice 
2
Subrogation agreement
Status sharing/replication
Life cycle history, then subsequent statutes
4
Receipt of subrogation
Payment of the invoice to the factor
5
Payment receipt of the invoice
6a
Notification of payment receipt
6b
Update of "Payment received" status
6c
Receipt of "Payment received" status
6d
"Factored" status
4c
Receipt of "Factored" status
4d
"Payment Sent" status
5b
Receipt of "Payment Sent" status
5c Transmission/sharing of life cycle Statuses
5d

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page56 / 149
§ MDT-207 = CBB (Payee Bank Details to be changed)
§ MDT-206: indicates the invoice data to be taken into account (BT-84: IBAN; BT-85: Account Name, BT-86: 
BIC)
§ MDT-214: new data to be taken into account (IBAN, Name, BIC, respectively).
ü Status: a new status must also be used here:
§ MDT-77: 23 (processing phase)
§ MDT-88: 5 (means "Information only")
§ MDT-105 (Processing status code): code 227 meaning "Change of account payable"
§ MDT-106 (Processing status description): "Change of Payable Account"
• Change of factoring company: this is the same message as the one announcing that the invoice has been 
"factored," but with the new factoring company.
• Factoring Cancellation: same message, but with the status "Not Factored" (code 228) and the bank account 
changed to that of the SELLER. If the previous status is "Factored Confidential," the "Not Factored" status MUST 
NOT be sent to the BUYER's PA-R (and therefore not to the BUYER).
Management of status messages between the SELLER and the FACTOR (optional): in order to inform the FACTOR of a new 
invoice submitted for factoring, the CDR life cycle status message can be used as follows:
• Issuer: MDG-16: the SELLER
• Sender: MDG-9: only role code WK (if issued by the PA-E)
• Recipient: MDG-23: the FACTOR
• Status type: MDT-77: 23 (processing phase)
• Standard status code: MDT-88: 5 (Information only)
• Invoice identification: 
ü MDT-87: Invoice number
ü MDT-91: invoice type code (e.g., 380)
ü MDG-40: SELLER
• Attachment: MDT-96: invoice encoded in base64
• Code and text Reform status: MDT-105 / MDT-106: 
ü 225 ("Invoiced")
ü 226 ("Confidential invoiced")
ü 228 ("Not invoiced"), for cases where invoicing has ended.
• Status history: use the status detail block (MDG-37), repeatable, with the information relating to the status code 
and date entered below: 
ü MDT-110: status date and time
ü MDT-111: Standard status code (MDT-88 in the status message exchanged and replicated here)
ü MDT-115: reform status code (2xx, from 200 to 228)
ü MDT-116: status description
ü And all the information contained in the status details exchanged and replicated here.
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
SELLER obligations:
Ø Create a life cycle status message for the BUYER announcing a change of payee or account payable (if necessary,
in the case of confidential factoring) and, where applicable, a subrogation/assignment of the invoice.
Ø Inform the FACTOR of the subrogation/assignment of the invoice by sending them the invoice and the life cycle
history.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page57 / 149
Ø Replicate/share the invoice and life cycle statuses with the FACTOR.
OBLIGATIONS OF THE BUYER:
Ø Receive the payee change message and process it.
Obligations of the factoring company:
Ø Allow the SELLER to access the payment receipt information for their invoices, enabling them to create "Payment 
Received" statuses for their factored invoices.
Additional optional features of the PA-E:
Ø Offer an additional feature for managing third-party factoring companies identified on the PA-E and for which 
invoice and status viewing rights are granted.
Ø Offer an additional feature for initiating subrogation/transfer of invoices with a referenced FACTORING 
COMPANY.
Ø Set up a service for replicating invoices and statuses to the FACTORING COMPANY, for example using the PEPPOL 
network, with the factoring company using a dedicated address for receiving replicated invoices and statuses. 
The FACTORING COMPANY's access point MUST be based in the EU (and this is the case if it is a PA). Transmission 
can also be done through the Standard Data Exchange API, which can be used by the Factoring Company once 
its rights and delegation have been configured on the PA-E.
Ø For factoring companies that wish to offer to prepare " Payment Received" statuses on behalf of the SELLER and 
transmit them to its PA-E, set up an additional feature for receiving invoice payment receiptstatuses transmitted 
by the FACTORING COMPANY on behalf of the SELLER, and use this status to prepare the " Payment Received" 
status so that the SELLER can send it to the PPF's CdD and to the BUYER.
3.2.10 Case No. 11: Invoice to be received and processed by a third party on behalf of the BUYER
This management case covers the business case of a third party (e.g., a property management company or a shared services 
center within a group) that manages a company's invoices. The invoice is made out to the taxable buyer (whose transactions 
are subject to electronic invoicing), but the third-party manager wishes to receive them for processing.
The most natural way to manage this situation is through the choice of invoicing electronic addresses. All the BUYER needs 
to do is create an address in the form SIRENacheteur_MONCSP or SIRENacheteur_ADMINBIENS, then choose the same 
Accredited Platform as its third-party manager so that the latter can access the BUYER's invoices for processing on its behalf.
These addresses can be designated directly by the Third-Party Manager, provided that they undertake to receive a mandate 
from their BUYER clients to also manage the creation of the invoicing electronic address that will be dedicated to their thirdparty management activities.
A template for the mandate to designate electronic addresses for the receipt of invoices is provided by the FNFE-MPE and 
is available on its website (www.fnfe-mpe.org).
NOTE: The other possible option, initially planned, was to allow the third party to be named in invoices with the EXTENDEDCTC-FR profile in the "ADDRESSEE" block, and, if the third party is itself subject to VAT in France (and therefore listed in the 
PPF Directory («Annuaire»)), the PA-E would have to send the invoice to the "ADDRESSEE" instead of the BUYER, who would 
no longer receive their invoices. This option goes against the simple principle that is now widely in place (particularly in the 
PEPPOL network), which states that the electronic invoice is sent to the electronic address of the BUYER as entered in BT49 (or the SELLER in BT-34 for self-billed invoices). It would also require the PA-R chosen by the Third-Party Manager to 
clearly distinguish between invoices received on behalf of the Manager and those received on behalf of each of the 
Manager's clients managed by the latter. It contravened the principle that all electronic invoicing addresses for receiving 
invoices from a taxable entity are listed in the directory and can be consulted by everyone. Finally, it required all SELLERS of 
the Manager's clients to use the EXTENDED-CTC-FR profile (since "Addressee" is not available in the EN16931 profile).
This is why this second option is excluded. However, the "Addressee" field can be used to name a third party responsible 
for processing the invoice, facilitating the management of delegations and rights on the PA-R. 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page58 / 149
Ultimately, in both cases, invoices arrive at the PA-R chosen by the Third-Party Manager, in which the BUYER is listed as 
BUYER and has access to their invoices, either directly through the PA-R (with rights of action limited to what is not entrusted 
to the Third-Party Manager), or because the Manager transmits the invoices and the life cycle to the BUYER by its own 
means.
The Third Party may also be authorized to pay on behalf of the BUYER. In this case, the PA-R can organize delegation rights 
to allow the third party to set the status to "Payment Transmitted" on behalf of the BUYER. The PA-R can also make this 
possible if the third-party Manager is also listed on the invoice as a Third-Party PAYER (block EXT-FR-FE-BG-02), as described 
in case no. 3.
Finally, in semantic terms, the THIRD PARTY manager has a role much more similar to that of a BUYER'S AGENT.
The steps in case no. 11, illustrated in the diagram below, are as follows:
Step Step name Responsible actor Description
1 Creation of the invoice for 
the BUYER SELLER
Following a commercial transaction (order/delivery, service 
contract, spot purchase, etc.), the SELLER creates the invoice 
(flow 2) via its information system. It may entrust the creation 
of the invoice to an OD/SC (Compatible Solution) or to its PAE. It sends it to its PA-E for processing.
They indicate the electronic invoicing address that their 
customer or manager has asked them to use, and use the 
appropriate profile.
2
Transmission of flow 1, 
the invoice (flow 2), and 
the related statuses
PA-E
Once the PA-E has carried out the regulatory compliance 
checks, including checks for duplicates and the existence of an 
active electronic invoicing address for the recipient, it MUST 
transmit the data required by the Administration (stream 1) 
to the CdD PPF. It MUST also transmit the invoice (stream 2 or 
stream 3) to the PA-R of the BUYER. It must send the status 
"Submitted" to the CdD PPF.
3 Receipt of the invoice
PA-R
/ Third party
/ BUYER
The Third Party and BUYER platform (PA-R) receives the 
invoice (flow 2 or flow 3), performs regulatory checks, creates 
transmission statuses (here "Received," then "Made 
Available") and makes the invoice available to the BUYER and 
Third Party for processing.
4a
4b
Processing of the invoice 
and updating of statuses 
prior to payment
Third party on behalf 
of the BUYER
Depending on the rights assigned to it, the THIRD PARTY 
processes the invoice on behalf of the BUYER and sets the 
corresponding processing statuses with the PA-R ("Rejected," 
"Accepted," "In dispute," "Approved," "Partially approved," 
"Suspended," etc.) for transmission to the SELLER through its 
PA-E.
4c Receipt of invoice statuses SELLER
The SELLER receives the invoice statuses following the 
processing of the invoice by the BUYER in accordance with the 
terms of the life cycle.
4d
Sharing/transmission of 
the invoice and life cycle 
with the BUYER
THIRD PARTY
The THIRD PARTY shares the invoice and life cycle with the 
BUYER, either through PA-R or directly (for example, if it 
records the invoice directly in the BUYER's information 
system).
5a
5b Payment of the invoice
BUYER or 
Third party on behalf 
of the BUYER
The invoice is paid either by the BUYER or by the Third Party 
on behalf of the BUYER.
If the BUYER pays, they inform the Third Party so that the 
latter can create the "Payment Transmitted" status.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page59 / 149
Step Step name Responsible actor Description
5c Creation of the "Payment 
Transmitted" status
The Third Party, on 
behalf of the BUYER
/PA-R
The Third Party creates the "Payment Transmitted" status and 
transmits it through its PA-R.
5d Receipt of "Payment 
Transmitted" status
SELLER 
/ PA-E
The SELLER receives the "Payment Transmitted" status from 
its PA-E.
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Paid" status SELLER 
/ PA-E
If the VAT on the invoice is payable upon payment receipt, the 
SELLER creates the " Payment Received" status and transmits 
it to the CdD PPF via its PA-E. The PA-E also transmits the "
Payment Received" status to the PA-R for the attention of the 
BUYER.
6c Receipt of "Paid" status by 
the THIRD PARTY
Third party on behalf 
of the BUYER
PA-R
The THIRD PARTY receives the " Payment Received" status on 
behalf of the BUYER.
6d Information to the BUYER 
about payment receipt THIRD PARTY
The Third Party shares the " Payment Received" status with 
the BUYER, either through PA-R or directly (for example, if it 
performs the lettering directly in the BUYER's Information 
System).
7
Receipt of " Payment
Received" status by the 
CdD PPF
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the " Payment
Received" status.
Figure19: Invoice to be processed by a third-party manager other than the BUYER
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
PA-R (the buyer chooses the same PA as the third party)
SELLER BUYER
PA-E
THIRD PARTY MANAGER
PPF
Transmission of Flow 1 and the invoice
2
Receipt of the invoice
3
Processing of the invoice and updating of the 
corresponding statuses
4b
Receipt of invoice processing statuses
4c
Sharing/transmitting the invoice 
and invoice Processing statuses 
4d
Payment of the invoice 5a
Payment receipt of the invoice
6a
Information on payment completion 
5b
Update of "Payment received" status
6b
Receipt of "Payment received" status
6c Information about payment receipt 6d
Transmission of invoicing data
and "Submitted", "Rejected", "Refused" statuses
2
Receipt of "Payment received" status
7
Creation of the invoice
1
Processing of the invoice
4a
"Payment Sent" status
5c
Receipt of "Payment Sent" status
5d

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page60 / 149
SELLER obligations:
Ø If applicable, use the EXTENDED-CTC-EN profile and fill in "ADDRESSEE" (EXT-EN-FE-BG-04).
OBLIGATIONS OF THE BUYER:
Ø Ensure that the management of invoices is entrusted within a controlled delegation framework, for example 
through a management mandate or by ensuring shared access to invoices on a shared PA-R with the third party 
or parties.
Obligations of the THIRD PARTY:
Ø Send invoices and life cycle statuses to the BUYER, either by acting directly on their information system (in the 
case of a Shared Services Center, for example), or by organizing access to the PA-R or the replication of invoices 
and statuses to the BUYER (for example, using standard APIs).
Optional PA-R features:
Ø Offer a management service for third-party managers acting on behalf of BUYERS, allowing them to set up their 
BUYER clients as PA-R users with third-party delegation on the BUYER accounts for which they have a mandate. 
If necessary, certain actions may be open to the third party only if they are named in the invoice (for example, a 
third-party PAYER can only have access to the invoice and its life cycle if they are named in the invoice as a thirdparty PAYER). 
Ø Set up a service for replicating/sharing invoices and life cycles with BUYERS whose invoices are in practice 
processed by a third-party manager.
3.2.11 Case No. 12: Transparent intermediary managing invoices for its principal BUYER
The transparent intermediary, within the meaning of VAT, acts as an intermediary between two parties in contract 
negotiations and the solicitation of suppliers or customers. 
For VAT purposes, the transparent intermediary is deemed to be acting on behalf of and in the name of another person. 
They appear as the principal's representative in their dealings with third-party contractors (BOI-TVA-CHAMP-10-10-40-40, § 
20). 
In practice, the principal may be the SELLER or the BUYER:
• If the transparent intermediary acts on behalf of the SELLER (acting as an intermediary "in the sale"), use case 19a 
(invoicing mandate) applies to the main invoice, i.e., the issuance of an invoice by a third party invoicing under an 
invoicing mandate. Case 17b (payment intermediary under an invoicing mandate, applicable in particular to 
purchases on marketplaces) also falls within the same context of transparent intermediary for the SELLER principal.
• If the transparent intermediary acts on behalf of the BUYER (intermediary "at the time of purchase"), use case No. 
11 applies to the main invoice, which is detailed in case No. 12.
In fact, the invoicing scheme provided for in principle for transparent intermediaries involves at least two invoices:
• An invoice issued by the intermediary for its intermediation services (PS). Intermediary transactions for this type of 
intermediary are considered to be services independent of the service that is the subject of the intermediation 
itself (BOI-TVA-CHAMP-10-10-40-40, §40). Therefore, intermediary services are subject to their own VAT regime 
and must be invoiced. 
• An invoice issued by the SELLER or on its behalf, made out to the BUYER. 
When the main invoice and the intermediary invoice are issued between two taxable entities established in France, they fall 
within the scope of electronic invoicing and are transmitted between Accredited Platforms («Plateformes Agréées»).
In the diagram below, where the transparent intermediary acts on behalf of the BUYER, the intermediary invoice is a 
standard invoice issued by the transparent intermediary to the BUYER.
Invoices are sent to the transparent intermediary using the addressing mechanism, which requires the BUYER to dedicate 
an electronic address for receiving invoices for this type of purchase (e.g., SIRENACH_ACHATTR), which is managed by the 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page61 / 149
PA-TR (Accredited Platform for Receipt by the Transparent Intermediary), generally chosen by the transparent intermediary 
and to which the transparent intermediary has access and delegated rights allowing it to access the invoices and life cycle
received at the BUYER's dedicated electronic address.
The transparent intermediary may create the BUYER's electronic address for receiving invoices dedicated to this type of 
purchase and entrust it to the PA-TR of its choice, PROVIDED THAT IT HAS A MANDATE FROM THE BUYER TO DO SO, given 
that it has at least a management mandate to process the BUYER's purchase invoices on its behalf.
Then, the following two situations occur:
• The BUYER has access to the PA-TR, either directly from its information system or its OD/SC (Compatible Solution), 
or through its own Accredited Platform used for its other types of purchases. In this case, the PA-TR can organize 
the respective rights of the BUYER and the transparent intermediary to view invoices and their life cycles, to set 
statuses, and to create and share the necessary summary documents. This case is detailed in the diagram below.
• The BUYER does not have access to the PA-TR, and it is up to the transparent intermediary to deliver the necessary 
invoices and summary documents to them (as was the case before the reform was implemented).
The steps in case no. 12, illustrated in the diagram below, are as follows:
Step Name of step Responsible actor Description
1 Creation of the invoice 
for the BUYER SELLER
Following a commercial transaction (order/delivery, service 
contract, spot purchase, etc.), the SELLER creates the invoice (flow 
2) via its information system. It may entrust the creation of the 
invoice to an OD/SC (Compatible Solution) or to its PA-E. It sends it 
to its PA-E for processing.
They indicate the electronic invoicing address that their customer or 
manager has asked them to use in accordance with paragraph 1.4 of 
this document, and use the appropriate profile.
2
Transmission of flow 1, 
the invoice (flow 2) and 
the related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (stream 1) to the CdD PPF. It 
MUST also transmit the invoice (stream 2 or stream 3) to the PA-TR 
of the TRANSPARENT INTERMEDIARY. It must transmit the status 
"Submitted" to the CdD PPF.
3 Receipt of the invoice
PA-TR
/ TRANSPARENT 
INTERMEDIARY
/ BUYER
The TRANSPARENT INTERMEDIARY and BUYER (PA-TR) platform 
receives the invoice (flow 2 or flow 3), performs regulatory checks, 
creates transmission statuses (here "Received," then "Made 
Available") and makes the invoice available to the BUYER and Third 
Party for processing.
4a
Processing of the 
invoice and updating of 
statuses prior to 
payment
The TRANSPARENT 
INTERMEDIARY on 
behalf of the 
BUYER
Depending on the rights assigned to it, the THIRD PARTY processes 
the invoice on behalf of the BUYER and sets the corresponding 
processing statuses with the PA-TR ("Rejected," "Accepted," "In 
dispute," "Approved," "Partially approved," "Suspended," etc.), for 
transmission to the SELLER through its PA-E.
4b Receipt of invoice 
statuses
SELLER
The SELLER receives the invoice statuses following the processing of 
the invoice by the BUYER in accordance with the terms of the life 
cycle.
4c
Sharing/transmission of 
the invoice and life 
cycle with the BUYER
THIRD PARTY
The TRANSPARENT INTERMEDIARY shares the invoice and life cycle
with the BUYER, either through PA-TR or directly (for example, if it 
records the invoice directly in the BUYER's information system).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page62 / 149
Step Name of step Responsible actor Description
5a
5b
5c
5d
Payment of the invoice
BUYER or 
the TRANSPARENT 
INTERMEDIARY on 
behalf of the 
BUYER
The invoice is paid either by the BUYER or by the TRANSPARENT 
INTERMEDIARY on behalf of the BUYER.
Where applicable, the TRANSPARENT INTERMEDIARY prepares a 
summary document for the BUYER, listing several invoices 
processed on their behalf, so that the latter can proceed with 
payment.
If the BUYER pays, they inform the TRANSPARENT INTERMEDIARY 
so that the latter can create the "Payment Transmitted" status.
5
Creation of the 
"Payment Transmitted" 
status
The TRANSPARENT 
INTERMEDIARY, on 
behalf of the 
BUYER
/PA-TR
The third party creates the "Payment Transmitted" status and 
transmits it through its PA-TR.
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status
SELLER 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER creates 
the "Payment Received" status and transmits it to the CdD PPF via 
its PA-E. The PA-E also transmits the "Payment Received" status to 
the PA-TR for the attention of the TRANSPARENT INTERMEDIARY on 
behalf of the BUYER.
6c
Receipt of the 
"Received" status by 
the THIRD PARTY
The TRANSPARENT 
INTERMEDIARY on 
behalf of the 
BUYER
PA-TR
The TRANSPARENT INTERMEDIARY receives the status "Payment 
Received" on behalf of the BUYER.
6d
Information for the 
BUYER regarding 
payment receipt
THIRD PARTY
The TRANSPARENT INTERMEDIARY shares the "Payment Received" 
status with the BUYER, either through PA-TR or directly (for 
example, if it performs the lettering directly in the BUYER's 
Information System).
7
Receipt of "Payment 
Received" status by the 
CdD PPF
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Payment 
Received" status.
The TRANSPARENT INTERMEDIARY then creates an intermediary invoice for the BUYER, via its invoice issuance PA-TE, 
addressed to the BUYER's PA-R.
The processing of this intermediary invoice is completely standard.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page63 / 149
Figure20: Transparent intermediary for the main BUYER with the BUYER's dedicated electronic invoicing address entrusted to a PA-TR of 
the transparent intermediary
Obligations of the PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the SELLER:
Ø Where applicable, use the EXTENDED-CTC-FR profile and fill in the "BUYER'S AGENT" (EXT-FR-FE-BG-01) or even 
the "ADDRESSEE" (EXT-FR-FE-BG-04).
BUYER'S obligations:
Ø Ensure that the management of invoices is entrusted within a controlled delegation framework, for example 
through a management mandate or by ensuring shared access to invoices on a common PA-R with the third party 
or parties.
Obligations of the TRANSPARENT INTERMEDIARY:
Ø Transmit invoices and life cycle statuses to the BUYER (who must record them), for example by organizing access 
to the PA-R or by replicating invoices and statuses to the BUYER (e.g., using standard APIs).
Optional PA-R features:
Ø Set up a service for replicating/sharing invoices and life cycle information with BUYERS whose invoices are 
processed by the Transparent Intermediary, including, where applicable, management of the Summary 
Document.
PA-TR
TRANSPARENT INTERMEDIARY
Third party BUYER'S AGENT
SELLER
PA-E
BUYER
SC/OD / PA
CdD
PPF
Transmission of Flows 1, invoices F1, F2, Fn, and 
corresponding statuses
2
Receipt of F1, F2, and Fn invoices
3
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Creation of invoice F1 / F2 ... Fn made out to the 
BUYER and sent to SIRENACH_ACHATTR
1
Processing of invoices and status updates
4
Receipt of invoice processing statuses
4
Access to summary document 
5b
Payment of summary document 5c Collection of payment for the summary document 5d
Creation of summary document
5a
Payment and update of invoice statuses (F1 ..: Fn)
5e
Payment receipt of F1 F2 Fn
6a
Update of "Payment received" status of F1 F2 Fn
6b
Receipt of “Payment received” statuses for F1 F2 Fn
6c
Creation and transmission of Flow 1, the intermediary 
invoice, and corresponding status
1
2 Receipt of the intermediary invoice
3
Processing of the invoice and updating of the 
corresponding statuses
4a
Receipt of invoice processing statuses 4b
Payment of the intermediary invoice
5
Payment receipt of the intermediary invoice 6a
Update of the "Payment received" status of the 
intermediary invoice
6b
Access to the F1 ..Fn life cycle, i.e., collection statuses
6d
Receipt of the “Payment received” status of the 
intermediary invoice
6c
AP-R
Mandate between the Transparent Intermediary and the BUYER, authorizing the 
Intermediary to access invoices received on SIRENACH_ACHATTR
AP-TE
Access to Invoices and Life Cycles for F1 … Fn
4c
7
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page64 / 149
3.2.12 Cases 13 and 14: Subcontracting and co-contracting (B2B, particularly for private works contracts)
3.2.12.1 Special features of private works contracts, particularly those covered by the Public Procurement Code
This section deals only with cases of subcontracting and co-contracting that do not fall within the public sector, therefore, 
end buyers are not subject to CHORUSPRO.
At this stage, private works contracts (B2B) must be invoiced electronically via platforms (PA). To be considered invoices, 
draft statements must contain all mandatory information and comply with standard EN16931. In this case, they will 
therefore have a standard code (BT-3) indicating that they are invoices (380, for example). 
In the event that it is necessary to have a prior exchange of draft statements that are not yet invoices between the SELLER 
and the BUYER before creating the so-called "final" invoice, the type code (BT-3) 325 meaning "Proforma invoice" may be 
used, which, as a reminder, is not an invoice. In this case, this document is not subject to reform. It can be exchanged 
through Accredited Platforms («Plateformes Agréées»), but must not be extracted from flow 1 or transmitted to the PPF. 
Subsequently, reference can be made to the first exchange of draft statements “projet de décompte” (first “proforma 
invoice” exchanged) in the “final” invoice exchanged, using block BG-3 (previous invoice: BT-25 (invoice no.), BT-26 (previous 
invoice date, EXT-FR-FE-02 (previous invoice type code, in EXTENDED-CTC-FR profile only)). This initial draft statement can 
also be attached to the invoice (Block BG-24). The terms for calculating payment deadlines apply in accordance with the 
rules in force based on the information added to the invoice.
3.2.12.2 Overview of subcontracting processing
In the case of B2B subcontracting, two separate invoices must be processed as part of electronic invoicing:
• The subcontractor sends an invoice (F1) to the main company ("TITULAIRE"). If the transaction is eligible for VAT 
reverse charge, the subcontractor must use the VATEX code "VATEX-FR-AE" in BT-121 (reason for VAT exemption 
in block BG-23 of VAT details), as well as the mention VAT REVERSE CHARGE in BT-120 (reason for VAT exemption 
in text).
• The main company (the CONTRACTOR) sends an invoice (F2) to the buyer for the total amount of the services 
and/or goods. The F2 invoice includes all services provided by the CONTRACTOR AND by the subcontractor (and 
potentially subcontractors). Thus, the total without VAT corresponds to the sum of all services provided by the 
CONTRACTOR and their subcontractor(s). VAT is therefore also calculated on this basis.
The example below illustrates the case of a reverse-charged F1 invoice, followed by an F2 invoice with VAT:
• Invoice F1: for earthworks, reverse-charged:
ü Total amount without VAT: 10,000
ü Total VAT (BT-110): 0
ü Total with VAT (BT-112): 10,000
ü VATEX code (BT-121): VATEX-FR-AE
ü Reason for exemption in text (BT-120): SELF-ASSESSMENT
• Invoice F2, including construction of the building, for €30,000, of which €10,000 is for the subcontractor and 
€20,000 is for the CONTRACTOR VAT 10%:
ü Total amount without VAT: 30,000
ü Total VAT (BT-110): 3,000
ü Total with VAT (BT-112): €33,000
ü Amount already paid (BT-113): 10,000
ü Net amount payable (BT-115): 23,000
In this case, VAT is first self-assessed by the CONTRACTOR (main company) for invoice F1, then the entire VAT amount of 
€3,000 is declared for invoice F2.
No specific procedures are required to process these invoices; they will be treated as standard invoices. In the event of 
direct payment by the buyer to the subcontractor, the procedures for recording this payment differ depending on whether 
it is B2B or B2G (see document "1- External Specifications File FE - Chorus Pro_v1.0" in the external specifications file) in 
terms of how the payment request is made.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page65 / 149
This document describes how B2B works. How B2G works is described in the document "1- External Specifications File FE -
Chorus Pro_v1.0" in the external specifications file.
3.2.12.3 Case No. 13: Invoice payable by a third party: subcontracting with direct payment or payment delegation
Business case no. 13 covers the business case of a SUBCONTRACTOR who, within the framework of a contract, sends an 
invoice to the SELLER/CONTRACTOR (TITULAIRE) but with direct payment by the end BUYER within the meaning of the public 
procurement code (B2G) or by delegation within the framework of private contracts (B2B).
In the context of private contracts, the BUYER may pay the SUBCONTRACTOR directly by delegation of payment from the 
SELLER contractor (Article 14 of Law No. 75-1334 of December 31, 1975 (Title II)). As validation by the TITULAIRE is not 
regulated by law, it must be provided for in the contract. It is also possible to provide for tacit validation in the specific 
contract documents. 
In the event of payment delegation (as opposed to direct payment in B2G with the sending of a payment request), the 
SUBCONTRACTOR will issue an invoice (F1) to the main company/Contractor (TITULAIRE). The "Cadre de Facturation" to be 
used may be the "standard" S1 (Submission of an invoice for services rendered) or the S5 “Cadre de Facturation” (Submission 
of an invoice for services rendered by a subcontractor, with direct payment by end buyer). The block "EXT-FR-FE-BG-02 -
PAYER OF THE INVOICE" (see case no. 3) may be used and completed in the F1 invoice (between the SUBCONTRACTOR and 
the CONTRACTOR/TITULAIRE) to indicate that direct payment is expected from the end BUYER - project owner (recipient of 
the F2 invoice from the main CONTRACTOR company).
As indicated above, two invoices are issued: 
• A first invoice (F1) from the SUBCONTRACTOR to the CONTRACTOR for its services;
• A second invoice (F2) from the CONTRACTOR (TITULAIRE) to the BUYER for the overall service (including services 
provided by the subcontractor).
Information concerning the BUYER's payment to the SUBCONTRACTOR is mentioned in Invoice F1 by the "Cadre de 
Facturation" S1 (Submission of a service provision invoice) or S5 (Submission by a subcontractor of a service provision invoice 
with direct payment by the end buyer (with tacit validation)) and by the PAYER block of the SUBCONTRACTOR's invoice, 
which references the third-party PAYER who is the BUYER of invoice F2.
Invoice F1: 
The specific features of the data and associated management rules for invoice F1 are as follows:
• BG-4 (SELLER): SUBCONTRACTOR;
• BG-7 (BUYER): main company/Contractor (TITULAIRE);
• EXT-FR-FE-BG-02 (PAYER OF THE INVOICE): project owner/end buyer third-party payer;
• Total invoice amount (BT-112): amount of the service provided by the SUBCONTRACTOR;
• BT-23: S1 (Submission of a service invoice) or S5 (Submission by a subcontractor of a service invoice with direct 
payment by the end buyer (with tacit approval));
• BT-11 (Project reference), BT-17 (Tender reference, lot reference), or BT-18 (Invoiced item, with qualifier in BT-18-
1 and a long UNTDID 1153 list): allows a Site reference or other reference to be entered in order to organize a 
payment receipt for various SUBCONTRACTOR invoices for the End BUYER (or its Project Manager in charge of 
validation).
The direct payment request may (MUST when the B2B transaction falls within the scope of the public procurement code) 
be sent via a life cycle status message (CDAR), with a dedicated status "Direct Payment Request", code 224.
This message references the invoice, which is attached to the message (MDT-96: Attachment), and is sent by the 
SUBCONTRACTOR's PA-E to the end BUYER's PA-R, who is thus informed. The CONTRACTOR is also designated as the 
recipient of this life cycle status message and receives it like any other status message. 
Invoice F2: 
The specific data and associated management rules for invoice F2 are:
• The END BUYER may send (highly recommended) the CONTRATOR a “Payment sent” status for Invoice F2 for the 
total amount with VAT of F2 minus the total amount with VAT of Invoice F1 (i.e., the net amount payable BT-115 
of Invoice F2).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page66 / 149
• The END BUYER may then send (highly recommended) to the CONTRACTOR a second status of “Payment sent” 
(211), which references:
ü on the one hand Invoice F2 (MDG-32)to indicate the second partial payment of invoice F2 in the amount payable 
for invoice F1, with the SUBCONTRACTOR appearing as the beneficiary of the payment (the payee):
§ MDT-87: invoice number F2 (BT-1 of the invoice)
§ MDT-91: invoice type code F2 (BT-3)
§ MDT-100 (in MDG-35): invoice date F2 (BT-2 of the invoice)
§ MDG-40 (Issuer of the invoice): the CONTRACTOR (legal identifier in MDT-129)
§ MDG-41 (Recipient of the invoice): the SUBCONTRACTOR, with role code MDT-158 equal to PE (meaning 
Payee), i.e. the same rule as for indicating a change of payment destination to a factoring company.
§ MDG-43: Payment details:
o MDT-207: RPA (meaning “amount paid”)
o MDT-215: the amount paid
o MDT-216: the currency of the amount paid
o MDT-219: payment date (with MDT-220 = 102 for a date)
ü On the other hand, Invoice F1 (in a second MDG-32) to indicate that a payment was made to the 
SUBCONTRACTOR on behalf of the CONTRACTOR (necessary in particular to distinguish the third-party payment 
of one SUBCONTRACTOR from that of another SUBCONTRACTOR):
§ MDT-87: F1 invoice number (BT-1 of the invoice)
§ MDT-91: F1 invoice type code (BT-3)
§ MDT-100 (in MDG-35): F1 invoice date (BT-2 of the invoice)
§ MDG-40 (Issuer of the invoice): the SUBCONTRACTOR (legal identifier in MDT-129)
§ MDG-43: Payment details:
o MDT-207: RPA (means “amount paid”)
o MDT-215: the amount paid
o MDT-216: the currency of the amount paid
o MDT-219: date of payment (with MDT-220 = 102 for a date)
• The CONTRACTOR may transmit (highly recommended) a Payment life cycle status to the SUBCONTRACTOR, 
corresponding to thatin the second MDG-32 record with the status “Payment Sent” that the CONTRACTOR received 
from the END BUYER for the payment of (see above, the one referencing the F1 invoice).
• The SUBCONTRACTOR may transmit its “Payment Received” life cycle status for invoice F1 (to the PPF and the 
HOLDER).
• The CONTRACTOR can then create its “Payment Received” life cycle status messages (in principle 2, one for each 
payment: total amount with VAT of invoice F1, and total amount to be paid for invoice F2, paid directly to the 
CONTRACTOR = total amount with VAT of invoice F2 – total amount including tax of invoice F1).
Example of payment status management in the case of a reverse charge invoice F1 for €10,000 without VAT and invoice F2 
for €30,000 without VAT and €33,000 with VAT (VAT 10%), of which €23,000 is paid to the CONTRACTOR and €10,000 is paid 
directly to the SUBCONTRACTOR:
• “Payment Received” life cycle status from the SUBCONTRACTOR to the CONTRACTOR for the payment receipt of 
invoice F1 paid directly by the END BUYER: as this is a reverse charge invoice (i.e., with VAT payable by the recipient),
this status is excluded from the obligation to transmit to the PPF (and therefore must not be transmitted to it), 
but it may be transmitted to the CONTRACTOR (because in practice it is a partial payment of its F2 invoice up to the 
amount with VAT of F1, which here is equal to the amount without VAT of F1: €10,000):
ü MDT-87: F1 invoice number (BT-1 of the invoice)
ü MDT-91: F1 invoice type code (BT-3)
ü MDT-100 (in MDG-35): F1 invoice date (BT-2 of the invoice)
ü MDG-40 (Issuer of the invoice): the SUBCONTRACTOR (legal identifier in MDT-129)
ü MDG-43: Payment details:
§ MDT-207: MEN (means “amount with VAT received”)

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page67 / 149
§ MDT-215: amount received: €10,000
§ MDT-216: currency of the amount received: EUR
§ MDT-219: payment date (with MDT-220 = 102 for a date)
§ MDT-224: applicable VAT rate: 0% (because of reverse charge)
§ It may be useful to indicate the reason for the 0% VAT rate, for example by entering the code VATEX-FR-AE 
in MDT-221
• Life cycle status “Payment Received” from the CONTRACTOR to the End BUYER for the payment of €10,000, which, 
from the END BUYER's point of view, is a partial payment of Invoice F2, sent to the END BUYER and the PPF. As 
invoice F2 has a VAT rate of 10%, this will result in a payment receipt of €10,000, including €909.09 in VAT and 
€9,090.91 excluding VAT:
ü MDT-87: invoice number F2 (BT-1 of the invoice)
ü MDT-91: F2 invoice type code (BT-3)
ü MDT-100 (in MDG-35): F2 invoice date (BT-2 of the invoice)
ü MDG-40 (Issuer of the invoice): the CONTRACTOR (legal identifier in MDT-129)
ü MDG-43: Payment details:
§ MDT-207: MEN (meaning “amount with VAT received”)
§ MDT-215: amount received: €10,000
§ MDT-216: currency of the amount received: EUR
§ MDT-219: payment date (with MDT-220 = 102 for a date)
MDT-224: applicable VAT rate: 10%
• Life cycle status “Payment received” from the CONTRACTOR to the END BUYER for the payment of €23,000, sent 
to the END BUYER and the PPF. As invoice F2 has a VAT rate of 10%, this will result in a payment receipt of €23,000, 
including €2,090.91 in VAT and €20,909.09 excluding VAT:
ü MDT-87: invoice number F2 (BT-1 of the invoice)
ü MDT-91: F2 invoice type code (BT-3)
ü MDT-100 (in MDG-35): F2 invoice date (BT-2 of the invoice)
ü MDG-40 (Issuer of the invoice): the CONTRACTORE(legal identifier in MDT-129)
ü MDG-43: Payment details:
§ MDT-207: MEN (meaning “amount with VAT received”)
§ MDT-215: amount received: €23,000
§ MDT-216: currency of the amount received: EUR
§ MDT-219: payment date (with MDT-220 = 102 for a date)
§ MDT-224: applicable VAT rate: 10%
The steps in management case no. 13, illustrated in the diagram below, are as follows:
• First, for Invoice F1 before payment:
Step Step name Responsible 
party Description
1
Creation of the invoice in 
the name of the 
CONTRACTOR (TITULAIRE), 
identifying the end BUYER 
as a third-party PAYER on 
the invoice
SUBCONTRAC
TOR
The SUBCONTRACTOR creates the F1 invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing. The BUYER (BG-7) on the invoice is the CONTRACTOR 
(TITULAIRE). 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page68 / 149
Step Step name Responsible 
party Description
2
Transmission of flow 1, 
the invoice (flow 2) and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST send the data 
required by the Administration (flow 1) to the CdD PPF. It MUST also 
send the invoice (flow 2 or flow 3) to the PA-TR of the CONTRACTOR 
(TITULAIRE) (BUYER of the F1 Invoice). It must send the status 
"Submitted" to the CdD PPF.
3 Receipt of the invoice PA-TR
The CONTRACTOR's purchase invoice reception platform (PA-R) 
receives the invoice (flow 2 or flow 3), performs the regulatory 
checks, creates the transmission statuses (here "Received," then 
"Made Available") and makes the F1 invoice available to the 
CONTRACTOR (TITULAIRE) for processing.
4
Processing of the invoice 
and updating of statuses 
prior to payment
The 
CONTRACTOR 
(TITULAIRE)
The CONTRACTOR (TITULAIRE) processes the invoice and sets the 
corresponding processing statuses with their PA-TR ("Rejected," 
"Accepted," "In dispute," "Approved," "Partially approved," 
"Suspended," etc.) for transmission to the SUBCONTRACTOR via 
their PA-E.
4b Receipt of invoice statuses SUBCONTRAC
TOR
The SUBCONTRACTOR receives the F1 invoice statuses following the 
processing of the invoice by the CONTRACTOR (TITULAIRE) in 
accordance with the terms of the life cycle.
5a
The SUBCONTRACTOR 
sends a direct payment 
request to the end buyer.
SUBCONTRAC
TOR 
/ PA-E
The SUBCONTRACTOR sends a direct payment request to the end 
BUYER. It uses a Life Cycle message (CDAR), with a status of "Direct 
Payment Request," code 2xx, to which it attaches the F1 Invoice.
5b Receipt of the direct 
payment request
PA-R
/ BUYER
PA-TR / 
CONTRACTOR 
(TITULAIRE)
The BUYER and the CONTRACTOR (TITULAIRE) receive the Direct 
Payment Request, intended primarily for the BUYER.
• Next, regarding the F2 invoice:
Step Name of step Responsible 
party Description
1 Creation of invoice F2 for 
the BUYER CONTRACTOR
The CONTRACTOR (TITULAIRE) creates the invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-TE. It sends it to its PA-TE 
for processing.
2
Transmission of flow 1, 
invoice F2 (flow 2) and 
related statuses
PA-TE
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and for the existence of an active 
electronic invoicing address for the recipient, it MUST send the data 
required by the Administration (flow 1) to the CdD PPF. It MUST also 
send the invoice (flow 2 or flow 3) to the PA-R of the BUYER. It must 
send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the F2 invoice (flow 2 or flow 
3), performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page69 / 149
Step Name of step Responsible 
party Description
4a
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses in its PA-R ("Rejected," "Accepted," "In dispute," 
"Approved," "Partially approved," "Suspended," etc.) for transmission 
to the CONTRACTOR (TITULAIRE) via its PA-TE.
4b Receipt of F2 invoice 
statuses
CONTRACTOR 
(TITULAIRE)
The CONTRACTOR receives the F2 invoice statuses following 
processing of the invoice by the BUYER (or its MOE on its behalf) in 
accordance with the terms of the life cycle.
5a
5b
Payment of the 
CONTRACTOR and the 
SUBCONTRACTOR
BUYER
Based on the information attached to the F2 invoice by the 
CONTRACTOR (TITULAIRE), and the Direct Payment Request, the 
BUYER pays the SUBCONTRACTOR for the F1 invoice, on behalf of the 
CONTRACTOR, and therefore also settles the CONTRACTOR 
(TITULAIRE)'s F2 invoice for the amount with VAT paid for F1.
The BUYER also pays the CONTRACTOR (TITULAIRE) the balance (F2 
amount with VAT – F1 amount with VAT), i.e. the Net Amount 
Payable (BT-115) of invoice F2.
5c
Creation and transmission 
of "Payment transmitted" 
statuses
BUYER
PA-R
The BUYER may transmit the "Payment Transmitted" status to the 
CONTRACTOR (TITULAIRE) via PA-R (recommended, one life cycle
message per payment).
5d Receipt of "Payment 
Transmitted" status
CONTRACTOR 
(TITULAIRE)
PA-TE
The CONTRACTOR (TITULAIRE) receives the "Payment Transmitted" 
status from its PA-TE.
It transmits a "Payment Transmitted" status to the SUBCONTRACTOR 
for their F1 invoice. 
6a Payment receipt for
invoice F2
CONTRACTOR 
(TITULAIRE)
The CONTRACTOR receives payment for invoice F2 (outside the 
circuit). This is the balance between the total with VAT for F2 and the 
total with VAT for F1.
6b Issuance of "Payment 
Received" status
CONTRACTOR 
(TITULAIRE) 
/ PA-E
If the VAT on the invoice is payable upon receipt, the CONTRACTOR 
(TITULAIRE) creates the status "Received" and transmits it to the CdD 
PPF via its PA-TE. The PA-TE also transmits the status "Received" to 
the PA-R for the attention of the BUYER.
6c
Receipt of the "Payment 
Received" status by the 
BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Payment 
Received" status by the 
PPF CdD for F2
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Paid" status for 
invoice F2 for the net amount payable for F2 (F2 amount with VAT –
F1 amount with VAT).
• To continue processing F1:
Step Step name Responsible party Description
5d
Creation and 
transmission of the 
"Payment 
transmitted" status
CONTRACTOR 
(TITULAIRE)
Based on the "Payment transmitted" status received from the 
BUYER for the direct payment of F1. The CONTRACTOR (TITULAIRE) 
transmits a "Payment transmitted" status to the SUBCONTRACTOR 
for their F1 invoice.
5 Receipt of "Payment 
Transmitted" status
SUBCONTRACTOR 
/ PA-E
The SUBCONTRACTOR receives the "Payment Transmitted" status 
from its PA-E.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page70 / 149
Step Step name Responsible party Description
6a Payment receipt for
the invoice SUBCONTRACTOR The SUBCONTRACTOR receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status
SUBCONTRACTOR 
/ PA-E
If the VAT on the invoice is payable upon receipt, the 
SUBCONTRACTOR creates the " Payment Received" status and 
transmits it to the CdD PPF via its PA-E. The PA-E also transmits the "
Payment Received" status to the PA-TR for the attention of the 
CONTRACTOR (TITULAIRE).
6c
Receipt of the "
Payment Received" 
status by the 
CONTRACTOR
CONTRACTOR 
(TITULAIRE)
PA-TR
The CONTRACTOR receives the " Payment Received" status for 
invoice F1.
7
Receipt of "Payment 
Received" status by 
the CdD PPF for F1
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Paid" status 
relating to the Payment of F1 to the SUBCONTRACTOR.
• And finally, on the F2 payment receipt phase:
Step Name of step Responsible party Description
6d Issuance of "Cashed" 
status
CONTRACTOR 
(TITULAIRE) 
/ PA-TE
Upon receipt of the "Paid" status from the SUBCONTRACTOR for 
Invoice F1, if the VAT on the invoice is payable upon receipt, the 
CONTRACTOR (TITULAIRE) creates the "Paid" status for Invoice F2 for 
the total amount with VAT of F1, and transmits it to the CdD-PPF via 
its PA-TE. The PA-TE also transmits the "Received" status to the PA-R 
for the attention of the BUYER.
6
Receipt of the 
"Payment Received" 
status by the BUYER
BUYER
PA-R
The BUYER receives the "Paid" status for invoice F2, for the amount 
with VAT of F1.
7
Receipt of "Paid" 
status by the PPF CdD 
for F2
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Paid" status for 
Payment F2 for the amount with VAT of F1 to the CONTRACTOR, 
made through Direct Payment of F1 to the SUBCONTRACTOR.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page71 / 149
Figure 21 : Subcontractor invoice (F1) to be paid by a third party and main invoice F2 from the CONTRACTOR (TITULAIRE) to the end 
BUYER (case of subcontracting with payment delegation).
Obligations of the SUBCONTRACTOR:
Ø Where applicable, use the EXTENDED-CTC-FR profile and enter the end BUYER in the "Third-party PAYER" block 
(EXT-FR-FE-BG-02) of the invoice.
Obligations of the SUBCONTRACTOR's PA-E:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Know how to transmit a Direct Payment Request Life Cycle status to the End BUYER AND the CONTRACTOR 
(TITULAIRE).
Optional features of the SUBCONTRACTOR's PA-E:
Ø Create the Direct Payment Request status on behalf of the SUBCONTRACTOR.
Obligations of the End BUYER's PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Know how to receive a "Direct Payment Request" Life Cycle status issued by a third party, without having the 
invoice to which it relates.
Optional features of the END BUYER's PA-R:
Ø Know how to process an F2 invoice, linking attachments with Direct Payment Request life cycle status messages 
received elsewhere.
Ø Know how to process/create a "Payment Transmitted" life cycle status message for an F2 invoice through F1 
direct payment to the third-party SUBCONTRACTOR.
Obligations of the End BUYER:
Ø Knowing how to process F2 invoices associated with Direct Payment requests from SUBCONTRACTORS.
CONTRACTOR (TITULAIRE)
PA-TR
SUBCONTRACTOR
PA-E
END BUYER
PA-R
CdD
PPF
Transmission of Flow 1, invoice F1, and corresponding 
statuses
2
Receipt of invoice F1
3
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Creation of invoice F1 in the name of the CONTRACTOR 
(TITULAIRE) (as BUYER), with the END BUYER as PAYER
1
Processing of the invoice 
and status update
4a
Receipt of invoice processing statuses
4b
Payment receipt of F1
6a
Update of F1's "Payment received" status
6b Receipt of F1 "Payment 
received" status
6c
Receipt of invoice F2, with attachments and statuses
3
Processing of the invoice and updating of the 
corresponding statuses
4a
Receipt of invoice processing statuses 4b
Partial payment of F2 up to F2 – F1
5b Partial payment receipt of invoice F2 
(up to F2 – F1)
6a
Update of "Payment received" status 
of F2 for the amount F2-F1
6b Receipt of the Payment received status F2, for the 
amount F2-F1
6c
PA-TE
Receipt of a Direct Payment Request on F1 (life cycle 
message)
5b
Direct payment request to the PAYER 
(END BUYER)
5a
Transmission of Flow 1, invoice F2 and 
corresponding attachments and 
statuses
2
Creation of the F2 invoice from the 
CONTRACTOR (TITULAIRE) to the END 
BUYER, with F1 and attachments + 
Direct payment certificate
1
Payment of F1 to the SUBCONTRACTOR, which also 
constitutes payment of F2 up to the amount of F1
5c
5a
Update of the "Payment received" 
status 
of F2 for the amount F1
6d
"Payment Sent" statuses
5c Receipt of "Payment Sent" status 
from F2
5d
Receipt of "Payment sent" status
5e
"Payment sent" status F1
5d
Receipt of status "Payment received" F2, for amount F1
6
7
2
Receipt of a Direct Payment 
Request (copy)
5b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page72 / 149
PA-TR obligations regarding receipt of invoices from the CONTRACTOR (TITULAIRE):
Ø Know how to process minimum base formats and profiles (invoice and CDAR).
Obligations of the PA-TE to issue invoices from the CONTRACTOR (TITULAIRE):
Ø Know how to process minimum base formats and profiles (invoice and CDAR).
Obligations of the CONTRACTOR (TITULAIRE):
Ø Create an F2 invoice, including the information required for direct payment.
Ø Manage the F2 payment statuses received from the BUYER, then create the F1 invoice status.
Optional features of PA-TR / PA-TE, if identical:
Ø Assist the CONTRACTOR (TITULAIRE) in managing F1 invoices, then create the F2 invoice, with direct payment 
certificates and payment statuses.
3.2.12.4 Subcontracting with direct payment (only in B2G ), for information
The operation of subcontracting with direct payment in B2G requires a few specific features, particularly regarding how to 
express the direct payment request through a second invoice message to be submitted directly to CHORUSPRO, with a 
"Cadre de Facturation" S3 and by modifying the Buyer (BG-7) and Seller Agent (EXT-FR-FE-BG-03) blocks.
These specific requirements are described in the document "1- External Specifications File FE - Chorus Pro_v1.0" in the 
external specifications file.
3.2.12.5 Case No. 14: Invoice payable by a third party: joint contracting case B2B
In the case of co-contracting, one or more co-contractors send F1 invoices to an end BUYER. One of the co-contractors is 
the lead AGENT (MANDATAIRE), whose role is to "approve" the F1 invoices of the other co-contractors, thereby preparing 
the processing of the co-contractors' F1 invoices and the Agent's F2 invoice.
In the description of the case, the various parties involved are as follows:
• The Co-contractor sends an F1 invoice to the BUYER
• The PA-E is the Accredited Platform for issuing Co-Contractor invoices
• The AGENT is the lead contractor, on whom the BUYER relies to compile an "invoicing file" consisting of the F1 
invoice (or F1 invoices in the case of multiple co-contractors) and the Agent's F2 invoice, as the lead co-contractor. 
As such, the Agent is both the SELLER of its F2 invoice and a third party responsible for pre-validating the Cocontractors' F1 invoices with the BUYER.
• The PA-ME is the Accredited Platform for issuing the Agent's F2 invoice.
• The BUYER is the BUYER (BG-7) of the F1 and F2 invoices.
• The PA-R is the BUYER's Accredited Platform, which has granted access rights to the Agent so that it can "approve" 
the Co-contractors' invoices.
In order to distinguish co-contracting invoices from others, it is recommended that the BUYER create a dedicated invoice 
receipt address, for example "SIRENacheteur_COTRAITANCE," and, if necessary, entrust it to an Accredited Platform that 
offers to handle the specificities of co-contracting with an Agent.
The specific features of the data and associated management rules are as follows for invoice F1:
• BG-4 (SELLER): CO-CONTRACTOR;
• BG-7 (BUYER): BUYER;
• EXT-FR-FE-BG-03 (SELLER'S AGENT): AGENT (TITULAIRE);
• BT-23: S6 (Submission of a service invoice by a co-contractor).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page73 / 149
• BT-11 (Project reference), BT-17 (Tender reference, lot reference), or BT-18 (Invoiced item, with qualifier in BT-18-
1 and a long UNTDID 1153 list): allows a Site reference or other reference to be entered in order to organize a 
payment receipt for various CO-CONTRACTOR invoices for the End BUYER (or its Project Manager in charge of 
validation), and so that the AGENT can approve them and produce a summary document.
Once the F1 invoices have been sent, the AGENT can assign "Approved" status to the BUYER's PA-R for each F1 invoice 
concerned by the invoicing sequence. 
It can then create its F2 invoice, attaching a summary document listing all the invoices to be processed and paid (F1 and F2).
The specific features of the data and management rules for the F2 invoice are the same as those for the F1 invoice, with, if 
required by the BUYER or deemed necessary by the AGENT, an attachment (BG-24) corresponding to the summary 
document intended for the BUYER. The qualifier for this attachment (BT-123) is "SUMMARY_CO-CONTRACTING."
The BUYER has access to invoices F1 and F2, which are all "Approved" by the AGENT. The AGENT's invoice F2 contains a 
summary document. The common reference (e.g., a Site number associated with an invoicing period identifier) allows 
invoices F1 and F2 to be grouped together for overall processing.
If the BUYER wishes to entrust the management of its invoices to a project manager (PM), it simply needs to designate the 
PM as a third party on its PA-R, then delegate access and action rights to the PM. Where applicable, its delegation rights 
may depend on the presence of the MOE in the invoices as the BUYER'S AGENT (EXT-FR-FE-BG-01), bearing in mind that this 
requires the CO-CONTRACTOR and the AGENT to be able to enter this information in the invoices.
That being said, the steps in management case no. 14 (B2B), illustrated in the diagram below, are as follows (for invoices F1
and F2):
Step Step name Responsible 
party Description
1
(F1)
Creation of invoice 
F1 for the BUYER
COCONTRACTOR
The CO-CONTRACTOR creates the F1 invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing. The BUYER (BG-7) in the invoice is the BUYER.
The AGENT is identified as SELLER AGENT (EXT-FR-FE-BG-03) to 
organize its preparation role on behalf of the BUYER (status 
"Approved").
2
(F1)
Transmission of 
flow 1, the invoice 
(flow 2) and related 
statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and for the existence of an active 
electronic invoicing address for the recipient, it MUST send the data 
required by the Administration (flow 1) to the CdD PPF. It MUST also 
send the invoice (flow 2 or flow 3) to the PA-R of the BUYER. It must 
send the status "Submitted" to the CdD PPF.
3a
(F1)
Receipt of invoice 
F1 PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
1
(F2)
Creation of the F2 
invoice for the 
BUYER
AGENT 
(MANDATAIRE)
After retrieving the F1 invoice from the CO-CONTRACTOR (either in 
parallel or through its access to the PA-R, if it offers this functionality), 
the AGENT creates the F2 invoice (flow 2) via its information system. It 
can entrust the creation of the invoice to an OD/SC (Compatible 
Solution) or to its PA-E. 
They are also identified as a SELLER AGENT (EXT-FR-FE-BG-03) to 
organize their preparation role on behalf of the BUYER (status 
"Approved," code 214).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page74 / 149
Step Step name Responsible 
party Description
It can attach a summary document of all the invoices to be processed 
together (those of the co-contractors and its own). This document is 
sometimes referred to as the draft summary statement, and details in 
particular the amounts to be paid to each service provider (COCONTRACTOR, AGENT), as well as their bank details and the references 
of the invoices to be paid.
They send the F2 invoice to their PA-ME for processing.
2
(F2)
Transmission of 
flow 1, the invoice 
(flow 2) and the 
related statuses
PA-ME
Once the PA-ME has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the PPF Data 
Concentrator (CdD PPF). It MUST also send the invoice (flow 2 or flow 
3) to the BUYER's PA-R. It must send the "Submitted" status to the PPF 
Data Concentrator.
3b
(F1 and 
F2)
Pre-processing by 
the AGENT.
"Approved" status 
on F1
AGENT 
(MANDATAIRE)
PA-R
If the PA-R proposes it, the AGENT has access to invoices F1 and F2 
(and several in the event of multiple co-contractors) on the BUYER's 
PA-R. Delegation rights may be exercised due to the presence of the 
AGENT as the SELLER's AGENT in invoices F1 and F2.
The AGENT may assign the status "Approved" (code 214) to invoices F1 
and F2, which allows the BUYER (or its project manager) to process 
invoices F1 and F2 together, with the help of the summary document 
attached to invoice F2.
4a
(F1 and 
F2)
Processing the 
invoice and 
updating statuses 
before payment
BUYER
The BUYER processes invoices F1 and F2 and sets the corresponding 
processing statuses with its PA-R ("Rejected," "Accepted," "In dispute," 
"Approved," "Partially approved," "Pending," etc.) for transmission to 
the CO-CONTRACTOR via its PA-E for invoice F1 and to the AGENT via 
its PA-ME for invoice F2.
If the AGENT has access to the PA-R and viewing rights (optional 
feature), they can also track the life cycle statuses assigned to invoice 
F1.
4b
(F1 and 
F2)
Receipt of invoice 
statuses
COCONTRACTOR
and AGENT
The CO-CONTRACTOR and the AGENT receive the F1 and F2 invoice 
statuses respectively, following the processing of the invoice by the 
BUYER in accordance with the terms of the life cycle.
5a
5b
(F1 and 
F2)
Payment of the 
invoice
Creation and 
transmission of the 
"Payment Sent" 
status
BUYER 
/ PA-R
The BUYER pays invoice F1 to the CO-CONTRACTOR and invoice F2 to 
the AGENT. They can send a "Payment Sent" status to the COCONTRACTOR and the AGENT via the PA-R (recommended).
5c
(F1 and 
F2)
Receipt of 
"Payment Sent" 
status
COCONTRACTOR 
/ PA-E
AGENT
PA-ME
The CO-CONTRACTOR receives the status "Payment Sent" for the F1 
invoice of its PA-E
The AGENT receives the status "Payment Sent" for invoice F2 from its 
PA-ME.
6a
(F1 and 
F2)
Payment receipt for
the invoice
COCONTRACTOR 
AGENT
The CO-CONTRACTOR receives payment for invoice F1 (outside the 
circuit).
The AGENT receives payment for invoice F2 (off-circuit).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page75 / 149
Step Step name Responsible 
party Description
6b
(F1 and 
F2)
Issuance of 
"Payment Received" 
status
COCONTRACTOR 
/ PA-E
AGENT
PA-ME
If the VAT on the invoice is payable upon receipt, the CO-CONTRACTOR 
for invoice F1 and the AGENT for invoice F2 create the "Received" 
status for their invoices (F1 and F2) and send it to the CdD-PPF via 
their PA-E / PA-ME. The PA-E / PA-ME also transmits the "Received" 
status to the PA-R for the attention of the BUYER.
6c
(F1 and 
F2)
Receipt of the 
"Payment Received" 
status by the BUYER
BUYER
PA-R The BUYER receives the "Payment Received" statuses for F1 and F2
7
(F1 and 
F2)
Receipt of 
"Payment Received" 
status by the PPF 
CdD
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Paid" status for F1 
and F2.
Figure22: Invoice payable by a third party (case of co-contracting in B2B)
Obligations of the CO-CONTRACTOR:
Ø If applicable, use the EXTENDED-CTC-FR profile and enter the AGENT in the "SELLER'S AGENT" block (EXT-FR-FEBG-03) of the invoice.
Obligation of the CO-CONTRACTOR's PA-E:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
OBLIGATIONS OF THE AGENT:
Ø Where applicable, use the EXTENDED-CTC-FR profile and enter the AGENT in the "SELLER'S AGENT" block (EXTFR-FE-BG-03) of the invoice.
Ø Where applicable (if required by the BUYER), attach a summary document (also known as a draft summary 
statement for works contracts) to the F2 invoice, detailing all F1 and F2 invoices, the amounts to be paid, and 
the bank details to be used.
AGENT (MANDATAIRE)
PA-AE
CO-CONTRACTOR
PA-E
BUYER
PA-R
CdD
PPF
Transmission of Flow 1, invoice F1, and corresponding 
statuses
2
Receipt of invoice F1
3a
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Creation of invoice F1 to the BUYER, the AGENT being the 
BUYER's AGENT (see SELLER's AGENT)
1
Processing of the invoice 
and status update
4a
Receipt of invoice processing statuses
4b
Payment receipt of F1
6a
Update of F1's "Payment received" status
6b
Receipt of F2 invoice, with attachments
3a
Processing of invoice F2 and update of corresponding 
statuses
4a
Receipt of invoice processing statuses
4b
Payment of F2 to the AGENT
5a
Payment receipt of invoice F2
6a
Receipt of F1 Payment received status
6c
Transmission of Flow 1, invoice F2 and 
attachments, and corresponding statuses
2
Creation of the AGENT’s F2 invoice to the 
BUYER, if applicable with a summary list of F1 
invoices (approved)
1
Payment of F1 to the CO-CONTRACTOR
5a
Update of the "Payment received" status
of F2
6b
"Payment sent" statuses on F1
5b
Receipt of "Payment Sent" status from F2
5c
Receipt of "Payment Sent" status
5c
Receipt of F2 Payment received status
6c
7
2
PA-R
The AGENT (MANDATAIRE) 
may assign the status 
"Approved" to F1 invoices.
3b
"Payment Sent" statuses on F2
5b
The AGENT (MANDATAIRE) 
can set the status to 
"Approved" on F2 and 
group invoices F1 and F2 
for the BUYER.
3b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page76 / 149
Ø Where applicable (if required by the BUYER and if permitted by the PA-R), affix the "Approved" status to invoices 
F1 and F2.
Obligations of the PA-ME of the CO-CONTRACTOR:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the End BUYER:
Ø Know how to process the EXTENDED-CTC-FR profile and F2 invoices with the attached summary document.
Ø If an MOE is involved, allow it access to the PA-R, even if this means dedicating an electronic invoicing address 
for receiving invoices for processing these invoices with the MOE and choosing a PA-R that is best suited to 
managing this use case involving different third parties.
Obligations of the BUYER's PA-R:
Ø Know how to process minimum base formats and profiles (invoice and CDAR).
Optional features of the BUYER's PA-R:
Ø Provide the AGENT with access to CO-CONTRACTORS' invoices in order to prepare the Summary Document to be 
attached to the F2 invoice. The right of delegation may depend on the AGENT being listed as the SELLER'S AGENT 
on the F1 and F2 invoices.
Ø Support the grouping of CO-CONTRACTOR invoices, either by initially identifying the members of the group, or 
by using a common reference to be included in the invoices (site ID and invoicing period: for example, 
SITEXX_JANUARY25, ...), and by granting the AGENT rights to organize the pre-processing of grouping and 
"Approval" on the PA-R, then presenting the complete file to the BUYER, if necessary going as far as organizing 
payment.
3.2.12.6 Case of co-contracting in B2G.
This document is not intended to describe how B2G invoices work, which is described in the document "1- External 
specifications file FE - Chorus Pro_v1.0" in the external specifications file.
However, for information purposes, B2G co-contracting, excluding works contracts, is in practice similar to what is described 
for B2B co-contracting, except that the AGENT MUST use the Chorus Pro Platform to "approve" or "reject" the F1 invoice 
for legitimate reasons before the BUYER continues processing. 
At this stage, the SUBCONTRACTOR is also required to be registered on the Chorus Pro Platform to enable the invoice to be 
processed.
3.2.13 Case No. 15: Sales invoice following an order (and possible payment) by a third party on behalf of the BUYER ( )
Management case No. 15 covers the business case of an order placed by a third party on behalf of the BUYER, with the third 
party responsible for validating the invoice and sometimes also for paying it on behalf of the BUYER.
A first concrete example is that of media purchasing, where the Advertiser is the Buyer, the Media Agency is the third party 
placing the order on behalf of the BUYER and may also pay the invoices on its behalf, and the Advertising Agency is the 
SELLER.
This chapter will therefore describe the case of advertising purchases, but this can be transposed to any other case where a 
BUYER'S AGENT organizes purchases and validation, or even payment of invoices on behalf of the BUYER.
The players are therefore as follows:
• The SELLER (the Advertising Agency), with its e-invoicing system
• The BUYER'S AGENT, who uses an e-invoicing system to which the BUYER has dedicated an electronic invoicing 
address for receiving invoices that they have chosen, if applicable with their Media Agency (for example, 
"SIRENacheteur_ACHATPUB"). 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page77 / 149
• The BUYER (the Advertiser), who may have their own PA-R for their other purchases, or a management solution 
(such as OD/SC (Compatible Solution)) that interacts with the PA-TR for receiving invoices on behalf of the BUYER.
As in other use cases involving third parties, the intervention of a third party requires, at a minimum, that they be identified, 
either on the issuing platform or on the receiving platform, as a third party, and with delegation rights that allow them to 
act on behalf of the issuer or recipient (usually the SELLER or BUYER).
The PA-TR may be:
• either the one proposed by the BUYER's AGENT and ultimately chosen by the BUYER, who becomes a simple user. 
In this case, the BUYER can access their invoices on the PA-TR through their main Accredited Platform or their 
OD/SC (Compatible Solution) (via API, replication of invoices and statuses; in practice, the BUYER's Accredited 
Platform acts as an OD/SC (Compatible Solution) for these invoices since it is not responsible for the address where 
these invoices are received.
• This is the address chosen directly by the BUYER and to which the BUYER's AGENT (Media Agency) has access and 
delegation. However, this PA-TR must offer the specific features required for this use case (intervention of the 
BUYER's third-party AGENT, management of statuses and who can set them, management of fund calls, etc.).
Once the BUYER has dedicated an address for receiving its invoices and has chosen a PA-TR to which the BUYER's AGENT 
(the Media Agency) has access and delegation, the specifics of case no. 15 are as follows:
• BG-4: the SELLER (advertising agency);
• BG-7: BUYER (Advertiser), with BT-49 as the electronic invoicing address entrusted to the PA-TR to which the 
BUYER's AGENT (Media Agency) has access and delegation
• EXT-FR-FE-BG-01 (BUYER'S AGENT): the third party who places the order on behalf of the Buyer (the Media Agency) 
and is responsible for approving the invoice;
• EXT-EN-FE-BG-02 (PAYER): the third party paying the invoice on behalf of the BUYER, if applicable. However, as the 
BUYER's AGENT is already identified on the PA-TR with their rights, their presence as a third-party PAYER on the 
invoice is not necessarily required.
• BG-24 (Additional supporting documents): if necessary, an attached document may supplement the invoicing data 
to facilitate invoice validation. For example, in the media purchasing sector, there is a document and format (PFP) 
already used to support the invoice approval and processing process, due to the specific characteristics of 
advertising space purchasing services, which can therefore be attached to the invoice.
The steps in management case no. 15, illustrated in the diagram below, are as follows:
Step Step name Responsible 
actor Description
1 Creation of invoices (F1 ... 
Fn) for the BUYER
SELLER
(Management)
The SELLER creates its invoices F1… Fn (flow 2) via its information 
system. It may entrust their creation to an OD/SC (Compatible 
Solution) or to its PA-E. It sends them to its PA-E for processing.
These invoices identify the BUYER'S AGENT as EXT-FR-FE-BG-01.
2
Transmission of flow 1, 
invoices F1, ... Fn (flow 2) 
and related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (stream 2 or stream 3) to the PA-TR of the 
BUYER and the BUYER'S AGENT. It must transmit the status 
"Submitted" to the CdD PPF).
3 Receipt of invoices F1 ... Fn PA-TR
The PA-TR platform (of the BUYER's AGENT and the BUYER for the 
electronic invoicing address dedicated to these invoices) receives 
invoices F1 ... Fn (flow 2 or flow 3), performs regulatory checks, 
creates transmission statuses (here "Received," then "Made 
Available") and makes the invoice available to the BUYER for 
processing, which allows the BUYER's AGENT to access it by 
delegation of rights.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page78 / 149
Step Step name Responsible 
actor Description
4a
Processing the invoice and 
updating statuses before 
payment
BUYER'S AGENT 
(on behalf of 
the BUYER)
The BUYER'S AGENT processes F1 invoices... Fn invoices on behalf of 
the BUYER and sets the corresponding processing statuses in the PATR ("Rejected," "Accepted," "In dispute," "Approved," "Partially 
approved," "Suspended," etc.) for transmission to the SELLER via its 
PA-E.
4b Receipt of invoice statuses
SELLER
(Régie)
The SELLER (Régie) receives the invoice statuses following the 
processing of the invoice by the BUYER's AGENT in accordance with 
the terms of the life cycle.
4c
Access to invoices F1 ... Fn 
and life cycles for the 
BUYER
BUYER The BUYER has access to the PA-TR to view their invoices and life 
cycle statuses.
5a
5b
5c
5d
Call for funds for payment 
by the BUYER'S AGENT (as 
third-party PAYER)
BUYER'S AGENT 
and BUYER
Call for funds from the BUYER'S AGENT (Media Agency), and receipt 
of funds to PAY invoices on behalf of the BUYER (off-circuit).
5e
Payment of the invoice
Creation and transmission 
of the "Payment 
transmitted" status
BUYER'S AGENT 
(Third-party 
PAYER)
/ PA-TR
The BUYER'S AGENT (third-party PAYER) pays the invoice to the 
SELLER on behalf of the BUYER. They can send a "Payment Sent" 
status to the SELLER via the PA-TR (recommended).
5c Receipt of "Payment 
Transmitted" status
SELLER
(Regie) 
/ PA-E
The SELLER receives the "Payment Sent" status from their PA-E
6a Payment receipt for
invoices F1, … Fn
SELLER
(Regie)
The SELLER receives payment for invoices F1 ... Fn (outside the 
circuit).
6b Issuance of "Payment 
Received" status
SELLER
(Regie) 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER (Régie) 
creates the status "Payment Received" and transmits it to the CdD 
PPF via its PA-E. The PA-E also transmits the "Payment Received" 
status to the PA-TR for the attention of the BUYER (and to which the 
BUYER's AGENT has access by delegation of rights).
6c Receipt of "Payment 
Received" statuses
BUYER and 
BUYER'S AGENT
PA-TR
The BUYER receives the "Payment Received" status on the PA-TR, to 
which the BUYER'S AGENT has access by delegation of rights.
6d
Access to "Payment 
Received" statuses for the 
BUYER
BUYER
PA-TR
The BUYER has access to the PA-TR to view their invoices and "Paid" 
life cycle statuses.
7 Receipt of "Paid" status by 
the CdD PPF
PPF Data 
Concentrator The PPF Data Concentrator (CdD PPF) receives the "Paid" status.
The BUYER's AGENT then sends a service invoice to the BUYER, which is a standard invoice exchanged between a SELLER 
(the BUYER's AGENT) and a BUYER.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page79 / 149
Figure23: Sales invoice following an order (and possibly payment) by a third party on behalf of the BUYER, example with invoices for the 
purchase of advertising space.
SELLER obligations:
Ø Where applicable, use the EXTENDED-CTC-FR profile and enter the third party responsible for ordering and 
managing invoices in the "BUYER'S AGENT" block (EXT-FR-FE-BG-01) of the Invoice.
Obligations of the PA-E SELLER (Advertising Agency):
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the third party (Media Agency):
Ø Organize the receipt of invoices in order to access them, either by asking the BUYER to open access on their PAR, or by dedicating an invoice receipt address (SIRENacheteur_ACHATPUB for example) and choosing a PA-TR 
shared with the Third Party (Media Agency), offering additional services to better manage the use case.
Ø Process invoices F1 ... Fn on behalf of the BUYER, and in particular set the necessary processing statuses. 
Ø Forward or give the BUYER access to F1 ... Fn invoices and their life cycle for accounting purposes.
Ø If the third party acts as a third-party PAYER, organize exchanges for fund calls, then pay the invoices on behalf 
of the BUYER, and transmit the corresponding life cycle statuses.
Obligations of the Third Party PA-TR (Media Agency) and the BUYER:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Know how to give the Third Party (Media Agency) access to invoices received at the BUYER's F1 ... Fn invoice 
receipt invoicing address. 
Optional features of the Third Party (Media Agency) and BUYER (Advertiser) PA-TR:
Ø Organize access to the BUYER from its OD/SC (Compatible Solution) or its PA-R if different from the PA-TR, so 
that it can view and integrate F1 ... Fn invoices and their life cycles.
PA-TR
Third-party BUYER'S AGENT SELLER (i.e. advertising agency)
PA-E
BUYER (i.e., Advertiser)
SC/OD / PA-R
CdD
PPF
Transmission of Flows 1, invoices F1, F2, Fn, and 
corresponding statuses
2
Receipt of F1, F2, and Fn invoices
3
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Creation of invoice F1 / F2 ... Fn made out to the 
PURCHASER and sent to SIRENACH_ACHATPUB
1
Processing of invoices and status updates
4a
Receipt of invoice processing statuses
4
Receipt of CALLS FOR FUNDS
5b
Transfer of funds 5c Receipt of funds for payment of invoices 5d
Creation of a Fund Request
5a
Payment and update of invoice statuses (F1 ..: Fn)
5e
Payment receipt of F1 F2 Fn
6a
Update of "Payment received" status of F1 F2 Fn
6b
Receipt of Payment received statuses for F1 F2 Fn
6c
Creation and transmission of Flow 1, the AGENT's 
service invoice and corresponding status
1
2 Receipt of the service invoice
3
Processing of the invoice and updating of the 
corresponding statuses
4a
Receipt of invoice processing statuses 4b
Payment of the AGENT's service invoice
5
Payment receipt of the service invoice 6a
Update of the "Payment received" status of the service 
invoice
6b
Access to the F1 ..Fn life cycle, i.e., collection statuses
6d
Receipt of the Payment sent status of the service 
invoice
6c
PA-R
Mandate between the BUYER'S AGENT and the BUYER, authorizing the BUYER'S AGENT to 
access invoices received on SIRENACH_ACHATPUB and to process and pay them...
PA-TE
Access to Invoices and Life Cycles for F1 ... Fn
4c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page80 / 149
Ø Propose a solution for managing calls for funds between the Third Party (Media Agency) and the BUYER, and 
their use through the payment of invoices by the Third Party on behalf of the BUYER.
Obligations of the BUYER (Advertiser):
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Organize access for the Third Party (Media Agency) to F1 ... Fn invoices intended for it, for example by dedicating 
an electronic invoicing address for receiving these invoices and entrusting this task to a PA-TR capable of offering 
shared processing with the third party identified as the BUYER's AGENT.
Obligation of the OD/SC (Compatible Solution) / PA-R of the BUYER (Advertiser):
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Optional features of the BUYER's OD/SC (Compatible Solution) / PA-R (Advertiser):
Ø Interoperate with the Third Party's PA-TR to view and integrate F1 ... Fn invoices and their life cycles.
Ø Interoperate with the Third Party's PA-TR to manage fund calls and their use in order to obtain the "Payment 
Sent" or "Payment Received" life cycle statuses, enabling the preparation of the deductible VAT return (if VAT 
on F1 ... Fn invoices is due upon payment receipt).
3.2.14 Case No. 16: Expense invoice for reimbursement of the sales invoice paid by the third party
These invoices are outside the scope of the electronic invoicing reform. They are therefore not subject to the electronic 
invoicing requirement, but they can be exchanged between platforms (Accredited Platforms («Plateformes Agréées») or 
OD/SC (compatible solutions) or directly between companies). They have VAT categories (in the lines (BT-151: tax category), 
in expenses (BT-102: expense VAT type code at document level) and allowances (BT-95: allowance VAT type code at 
document level) at document level and in VAT detail (BT-118: VAT type code)) equal to "O" (meaning "outside VAT scope").
The EN16931 standard does not allow VAT categories "O" to be mixed with others in an invoice (currently being modified in 
the update to the standard). 
However, the EXTENDED-CTC-FR profile already allows this. This makes it possible to have expense lines in a standard invoice 
with VAT-liable invoicing lines. These invoices, which include both lines subject to VAT and expense lines outside the scope 
of VAT (Category Code "O"), MUST be processed in accordance with e-invoicing obligations. The extracted flow 1 therefore 
also contains the expense lines and the corresponding VAT breakdown details.
NOTE: It should also be noted that there is a VATEX exemption code VATEX-EU-79-C, which is used with VAT Category Code 
"E" and can be used for expense reimbursements, which are similar to expense lines.
3.2.15 Case No. 17a: Invoice payable to a third party, payment intermediary (e.g., on Marketplace)
In this case, a BUYER orders and pays an intermediary at the time of ordering, then receives delivery and an invoice from 
the SELLER. This is therefore also a prepaid invoice.
Invoice F1 is the invoice corresponding to the SALE made by the SELLER to the BUYER, paid to the Payment Intermediary.
Invoice F2 is the invoice for services rendered by the payment intermediary to the SELLER.
The steps in management case No. 17a, illustrated in the diagram below, are as follows:
Step Step name Responsible 
party Description
5a Online ordering and 
payment
BUYER
The BUYER places an order with a third party (e.g., online on a 
marketplace) and makes the payment to that payment intermediary 
(outside the tool).
6a Receipt of the order 
amount
Payment 
intermediary
The intermediary receives payment for the entire order placed by 
the BUYER (excluding tools).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page81 / 149
1 Creation of invoice F1 for 
the BUYER SELLER
Following the commercial transaction already paid for, the SELLER 
creates invoice F1 (flow 2) via its information system. It can entrust 
the creation of this invoice to an OD/SC (Compatible Solution) or to 
its PA-E. It sends it to its PA-E for processing.
As the F1 invoice has already been paid, it has an "Amount Already 
Paid" (BT-113) equal to the Total Amount with VAT (BT-112), and 
therefore a Net Amount Payable (BT-115) equal to 0.
2
Transmission of flow 1, 
invoice F1 (flow 2) and 
related statuses
PA-E
Having carried out the regulatory compliance checks, including 
checks for duplicates and the existence of an active electronic 
invoicing address for the recipient, the PA-E MUST transmit the data 
required by the Administration (flow 1) to the CdD PPF. It MUST also 
transmit invoice F1 (flow 2 or flow 3) to the PA-R of the BUYER. It 
must send the status "Submitted" to the CdD PPF.
3 Receipt of the F1 invoice PA-R
The BUYER's platform (PA-R) receives the F1 invoice (flow 2 or flow 
3), performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
6b Issuance of "Paid" status SELLER 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER creates 
the "Received" status and transmits it to the CdD-PPF via its PA-E. 
The PA-E also transmits the "Received" status to the PA-R for the 
attention of the BUYER.
6c Receipt of "Paid" status by 
the BUYER
BUYER
PA-R The BUYER receives the "Received" status.
7 Receipt of "Paid" status for 
Invoice F1 by the CdD-PPF CdD PPF The PPF Data Concentrator (CdD-PPF) receives the "Paid" status for Invoice F1.
5b (F1)
5 (F2)
Payment to the SELLER of 
the proceeds of the sale, 
less intermediary fees
Payment 
intermediary
The intermediary platform transfers the proceeds of the Sale (with 
VAT from Invoice F1) minus the platform fees (with VAT from Invoice 
F2 to be created) to the SELLER. 
As a result, the Intermediary receives payment of its invoice F2 for 
intermediary fees.
6d Payment receipt of the net 
amount of F1 fees SELLER The SELLER receives the net proceeds of their Sale (with VAT from invoice F1 minus the with VAT from invoice F2 to be created).
1 Creation of invoice F2 for 
the SELLER
Payment 
intermediary
The payment intermediary creates invoice F2 (flow 2) via its 
information system. It may entrust its creation to an OD/SC 
(Compatible Solution) or to its PA-T. It sends it to its PA-T for 
processing.
As Invoice F2 has already been paid, it has an "Amount Already Paid" 
(BT-113) equal to the Total Amount with VAT (BT-112), and 
therefore a Net Amount Payable (BT-115) equal to 0.
2
Transmission of flow 1, 
invoice F2 (flow 2) and 
related statuses
PA-T
Having carried out the regulatory compliance checks, including 
checks for duplicates and the existence of an active electronic 
invoicing address for the recipient, the PA-T MUST transmit the data 
required by the Administration (flow 1) to the PPF Data 
Concentrator (CdD PPF). It MUST also transmit invoice F2 (stream 2 
or stream 3) to the SELLER's PA-RV. It must transmit the status 
"Submitted" to the PPF Data Concentrator (CdD PPF).
3 Receipt of the F2 invoice PA-RV
The SELLER's platform (PA-RV) receives the F2 invoice (flow 2 or flow 
3), performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the SELLER for processing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page82 / 149
6a Issuance of "Paid" status
Payment 
intermediary
PA-T
If the VAT on the invoice is payable upon payment receipt, the 
payment intermediary creates the "Payment Received" status and 
transmits it to the CdD PPF via its PA-T. The PA-T also transmits the 
"Payment Received" status to the PA-RV for the attention of the 
SELLER.
6b
Receipt of the "Paid" 
status of the F2 invoice by 
the SELLER
SELLER
PA-RV The SELLER receives the "Payment Received" status for invoice F2.
7 Receipt of "Paid" status for 
invoice F2 by the CdD PPF
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Paid" status for 
the F2 invoice.
Figure24: Invoice payable to a third party, payment intermediary
The specific features of the data and associated management rules for F1 and F2 invoices are:
• Billing framework (Business Process Type - “Cadre de Facturation” : BT-23) meaning "Submission of an invoice 
already paid" (B2/S2/M2) or "Cadre de Facturation" B1/S1/M1 (see management rule BR-FR-08 of Standard XP Z12-
012). As the F2 invoice is for services, it will be S2 or S1;
• Amount paid (BT-113) equal to the total amount with VAT of the invoice (BT-112);
• Amount payable (BT-115) equal to 0;
• For invoice F1, the payment method (BT-81) is not readily known to the SELLER (since they did not receive the 
payment). As the invoice has already been paid, payment method "57" (Standing Agreement) can be used. The 
Payee (BG-10) can be entered as the payment intermediary.
• For Invoice F2, the payment method can be "97" (Clearing between partners).
• The payment date is indicated by transmitting the "Payment Received" status for each F1 and F2 invoice, if VAT is 
payable upon payment receipt.
Obligations of the BUYER and its PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
PA-RV
Payment intermediary
PA-T
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Online ordering and payment
5a Collection of the order amount on behalf of the Seller 
(dedicated account)
6a
Creation of the already paid invoice F1 for the order 
amount
1
Transmission of Flow 1, invoice F1, and corresponding 
status
2
Receipt of invoice F1
3
Update of the "Payment received" status of F1 for its 
total amount
6b
Receipt of invoice F2
3
Creation of an invoice for fees/commissions already 
paid F2 and transmission of Flow 1
1
Update of F2 status to "Received" (no payment by 
supplier, but netting, see 5b)
6a
Transmission of Flow 1, invoice F2, and corresponding 
status
2
Payment receipt of the net amount of F1 fees
6d
Receipt of the Payment received status for F1
6c
Receipt of Payment received status for F2
6b
Payment of the SELLER's sales amount, minus 
commissions (invoice F2)
5b
5
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
2 Receipt of "Payment received" status
7 7

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page83 / 149
Obligations of the Seller and its PA-E and PA-RV:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the payment intermediary:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Inform the SELLER of the payment receipt for the F1 Invoice.
Obligations of the payment intermediary's PA-T:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
3.2.16 Case No. 17b: Invoice payable to a third party, payment intermediary, and third-party invoicing under an 
invoicing mandate
This use case is the same as case 17a, except that the third-party intermediary also takes care of creating and transmitting 
the F1 invoice on behalf of the SELLER. To do so, they must have an invoicing mandate from the SELLER, and all the 
characteristics of issuing an invoice by a third-party invoicer then apply (see case no. 19).
The steps in management case no. 17b, illustrated in the diagram below, are as follows:
Step Step name Responsible actor Description
0 Conclusion of an
invoicing mandate
SELLER
Payment intermediary 
and invoicing party
The SELLER and the payment intermediary/Invoicing Party 
enter into an invoicing mandate so that the payment 
intermediary/Invoicing Party can issue (and possibly file) 
invoices on behalf of the SELLER.
5a Online ordering and 
payment BUYER
The BUYER places an order with a third party (e.g., online on a 
marketplace) and makes the payment to that payment 
intermediary (outside the tool).
6a Receipt of the order 
amount Payment intermediary The intermediary receives the full amount of the order placed by the BUYER (excluding the tool).
1 Creation of invoice F1 
for the BUYER
Payment 
intermediary/Invoicing 
on behalf of the 
SELLER
Following the commercial transaction already paid for, the 
payment intermediary/invoicing agent creates the F1 invoice 
(flow 2) via its information system. It may entrust the creation 
of the invoice to an OD/SC (Compatible Solution) or to its PA-T. 
It sends it to its PA-T for processing.
As the F1 invoice has already been paid, it has an "Amount 
already paid" (BT-113) equal to the Total amount with VAT (BT112), and therefore a Net amount payable (BT-115) equal to 0.
2
Transmission of flow 1, 
invoice F1 (flow 2) and 
related statuses
PA-T
Third-party invoicing 
entity
The PA-T, having carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, MUST transmit 
the data required by the Administration (flow 1) to the CdD 
PPF. It MUST also transmit invoice F1 (flow 2 or flow 3) to the 
PA-R of the BUYER. It must send the status "Submitted" to the 
CdD PPF.
The third-party invoicing entity (with its PA-T, if applicable) 
makes the F1 invoice available to the SELLER, allowing the 
SELLER to potentially reject it before it is transmitted (see case 
19).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page84 / 149
3 Receipt of the F1 invoice PA-R
The BUYER's platform (PA-R) receives the F1 invoice (flow 2 or 
flow 3), performs the regulatory checks, creates the 
transmission statuses (here "Received," then "Made 
Available") and makes the invoice available to the BUYER for 
processing.
6b Issuance of "Payment 
Received" status
Payment intermediary 
/ Invoicing on behalf 
of the SELLER 
PA-T
If the VAT on the invoice is payable upon receipt, the payment 
intermediary/invoicing party creates the status "Received" and 
transmits it to the CdD-PPF via its PA-T. The PA-T also transmits 
the status "Received" to the PA-R for the attention of the 
BUYER.
6c
Receipt of the "Payment 
Received" status by the 
BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Paid" status 
for Invoice F1 by the 
CdD-PPF
CdD PPF The PPF Data Concentrator (CdD-PPF) receives the "Paid" 
status for Invoice F1.
5b 
(F1)
5 (F2)
Payment to the SELLER 
of the proceeds of the 
sale, less intermediary 
fees
Payment intermediary
The intermediary's platform transfers the proceeds of the sale 
(with VAT from Invoice F1) minus the platform fees (with VAT
from Invoice F2 to be created) to the SELLER. 
As a result, the Intermediary receives payment of its invoice F2 
for intermediary fees.
6d Receipt of the net 
amount of F1 fees SELLER The SELLER receives the net proceeds of their Sale (with VATfrom invoice F1 minus with VAT from invoice F2 to be created).
1 Creation of invoice F2 
for the SELLER Payment intermediary
The payment intermediary creates invoice F2 (flow 2) via its 
information system. It may entrust its creation to an OD/SC 
(Compatible Solution) or to its PA-T. It sends it to its PA-T for 
processing.
As Invoice F2 has already been paid, it has an "Amount Already 
Paid" (BT-113) equal to the Total Amount with VAT (BT-112), 
and therefore a Net Amount Payable (BT-115) equal to 0.
2
Transmission of flow 1, 
invoice F2 (flow 2) and 
related statuses
PA-T
Having carried out the regulatory compliance checks, including 
checks for duplicates and the existence of an active electronic 
invoicing address for the recipient, the PA-T MUST transmit the 
data required by the Administration (flow 1) to the PPF Data 
Concentrator (CdD PPF). It MUST also transmit invoice F2 
(stream 2 or stream 3) to the SELLER's PA-RV. It must transmit 
the status "Submitted" to the PPF Data Concentrator (CdD 
PPF).
3 Receipt of the F2 invoice PA-RV
The SELLER's platform (PA-TV) receives the F2 invoice (flow 2 
or flow 3), performs the regulatory checks, creates the 
transmission statuses (here "Received," then "Made 
Available") and makes the invoice available to the SELLER for 
processing.
6a Issuance of "Paid" status Payment intermediary
PA-T
If the VAT on the invoice is payable upon receipt, the payment 
intermediary creates the status "Received" and transmits it to 
the CdD PPF via its PA-T. The PA-T also transmits the status 
"Received" to the PA-RV for the attention of the SELLER.
6b
Receipt of the "Payment 
Received" status of the 
F2 invoice by the SELLER
SELLER
PA-RV
The SELLER receives the "Payment Received" status for invoice 
F2.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page85 / 149
7
Receipt of "Payment 
Received" status for F2 
invoice by CdD PPF
PPF Data Concentrator The PPF Data Concentrator (CdD PPF) receives the "Paid" 
status for invoice F2.
Figure25: Invoice payable to a third party, payment intermediary, and third-party invoicing under an invoicing mandate.
The specific features of the data and associated management rules are:
• For the F1 invoice: 
ü "Cadre de Facturation" B2/S2/M2 ("Submission of an invoice already paid") or "Cadre de Facturation" B1/S1/M1 
(see management rule BR-FR-08 of Standard XP Z12-012).
ü Amount paid (BT-113) equal to the total amount with VAT of the invoice (BT-112).
ü Amount to be paid (BT-115) equal to 0.
ü If applicable, the third-party invoicing entity (payment intermediary) may be present in EXT-FR-FE-BG-05 of the 
EXTENDED-CTC-FR profile.
ü As the third-party invoicing entity is also the payment intermediary, it can provide information on the payment 
method used.
ü To receive the statuses, the F1 invoice must contain an electronic address for the SELLER (BT-34) to which the 
invoicing party has access in order to receive the statuses (and therefore entrusted to the PA-T in the event of 
interoperability with dynamic discovery, as on the PEPPOL network).
• For the F2 invoice
ü "Cadre de Facturation" S2 ("Submission of an invoice already paid") or S1 (see management rule BR-FR-08 of 
Standard XP Z12-012);
ü Amount paid (BT-113) equal to the total amount with VAT of the invoice (BT-112);
ü Amount payable (BT-115) equal to 0;
ü The payment method may be equal to "97" (Clearing between partners).
• For F1 and F2 invoices, the payment date is indicated by the transmission of the "Received" status for each F1 and 
F2 invoice, if VAT is payable upon receipt.
Payment intermediary
INVOICER
PA-T
SELLER
SC/OD / PA-E
CdD
PPF
BUYER
PA-R
Payment receipt of the order amount
6a
Online order and payment
5a
Creation of the already paid invoice F1 for the order 
amount in the name and on behalf of the supplier
1
Transmission of Flow 1, invoice F1, and corresponding 
status
2
Receipt of invoice F1
3
Update of the "Payment received" status of F1 for its 
total amount
6b
Billing mandate
Provision of invoice F1
Receipt of the "Payment received" status for F1
6c
Receipt of invoice F2
3
Creation of an invoice for fees/commissions already 
paid F2 and transmission of Flow 1
1
Update of F2 status to ”Payment Received" (no 
payment by supplier, but netting, see 5b)
6a
Transmission of Flow 1, invoice F2, and corresponding 
status
2
Payment receipt of the net amount of F1 fees
6d
Receipt of the Payment received status for F2
6b
Payment of the SELLER's sales amount, minus 
commissions (invoice F2)
5b
5
Receipt of "Payment received" status
7 7 Transmission of invoicing data
and "Submitted", "Rejected", "Refused" statuses
2
2
PA-RV
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page86 / 149
Obligations of the BUYER and its PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the Seller and its PA-RV:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the payment intermediary/third-party INVOICER:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Transmit information relating to the F1 invoice and its life cycle to the SELLER.
Obligations of the payment intermediary/third-party INVOICER entity's PA-T:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Optional features of the payment intermediary/third-party INVOICER entity's PA-T:
Ø Allow the Seller access to the PA-T to view invoices issued on their behalf and the associated life cycle statuses.
3.2.17 Case No. 18: Management of debit notes
A debit note is not an invoice. A debit note is a document issued by a SELLER to its BUYER stating an amount owed by the 
latter to the former. In principle, if accepted by the customer, this debit note will have given rise to or should give rise to an 
invoice.
According to this definition, debit notes do not fall within the scope of electronic invoicing and are not subject to flow 1 or 
international B2B sales e-reporting. However, an invoice related to this transaction should have been sent.
This case does not apply to debit notes that are treated as invoices when they are subject to VAT and include all the 
mandatory information (e.g., re-invoicing to a joint venture). In this case, these debit notes must therefore be electronic 
invoices, classified as invoices (BT-3 equal to 380, for example, for a commercial invoice).
If the debit note is issued by the buyer and shows a debt owed to them by the seller, then the seller should issue a credit 
note.
In practice, this debit note may also be issued in the form of a credit note by the BUYER (self-billed credit note, type code 
BT-3 equal to 261), if the latter has an invoicing mandate, and use an Accredited Platform to transmit this self-billed credit 
note to the SELLER.
3.2.18 Cases 19a and 19b: Invoice issued by a third party on behalf of the SELLER under an Invoicing Mandate
The SELLER may entrust a third party with the creation of its sales invoices. In this case, the third party MUST have an 
invoicing mandate, the terms and conditions of which are described in BOI-TVA-DECLA-30-20-10, Articles 340 to 560, and in 
BOI-TVA-DECLA-30-20-10-30, Articles 70 to 290.
It is therefore necessary to distinguish between two types of invoices issued by third parties:
• Invoices issued by third parties other than the BUYER (case no. 19a). In this case, there are two possibilities:
ü The SELLER provides the invoicing data to the third-party invoicer provider, including the invoice number. The 
third-party invoicer provider creates the invoice based on this data, in the target format agreed upon with the 
SELLER. The SELLER may decide to create a dedicated chronological numbering series, particularly if it creates 
some of its invoices itself.
ü The third-party invoicer creates the invoice on behalf of the SELLER based on data that it manages itself 
elsewhere, for example because it organized the sale (in the case of a marketplace or distributor). In this case, 
it must create a dedicated sequential numbering series for each SELLER for whom it creates invoices in its name 
and on its behalf.
• Invoices issued by the BUYER. This is referred to as self-billing (case no. 19b). The BUYER must have an invoicing 
mandate and, as they will be creating invoices based on their own data, they MUST create a chronological invoice 
numbering series for each SELLER (and not a global series for all SELLERS for whom they create self-billed invoices). 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page87 / 149
Invoices are sent from the BUYER to the SELLER, which impacts the processing procedures of Accredited Platforms 
(«Plateformes Agréées»), since in all other cases the invoice is always issued by a SELLER or a third party 
representing them to the BUYER.
In all cases, the SELLER must have the invoices that the third party has created on its behalf, as well as the life cycle statuses, 
for its accounting and tax returns. It is therefore up to the SELLER and its invoicing agent (Invoicer “Facturant”) to organize 
themselves to do this. In the case of self-billing, the SELLER has this by default.
The Agent, whether a third party or the BUYER, must be aware of the SELLER's VAT regime at a minimum when it has an 
impact on the invoice it creates on its behalf. In particular, it must know whether the SELLER is a franchisee and be aware 
that the latter may exceed its VAT exemption threshold (and therefore allow it to indicate this).
3.2.18.1 Case No. 19a: Invoice issued by a third-party invoicing with an invoicing mandate
The invoicer handles invoicing on behalf of the SELLER: they are responsible for creating the invoice and sending it to the 
BUYER. The SELLER receives payment for the invoice and therefore initiates the creation of the "paid" status if VAT is due 
upon receipt. 
Invoices falling within the scope of e-invoicing must be issued by an Accredited Platform. This may be:
• Either the SELLER's Accredited Platform, meaning the one that the SELLER uses to issue all of its invoices. In this 
case, the Agent is an intermediary who has access to the SELLER's Accredited Platforms («Plateformes Agréées»)
and can submit invoices on their behalf. If the Agent has multiple invoicing mandates, they must have access rights 
to multiple Accredited Platforms («Plateformes Agréées»).
• Either the Agent's Accredited Platform, i.e., the Accredited Platform that the Agent has chosen to issue the invoices 
it creates. However, in order to do so, and so as not to impact the recipient's processing, which is indifferent to 
whether the invoices it receives are sent by the SELLER or its Agent on its behalf, the e-invoicing platform issuing 
the invoices has first ensured that the invoicing mandate exists and has implemented it in such a way as to 
distinguish between invoices issued on behalf of each SELLER, which amounts to considering the SELLER as a user 
of the e-invoicing platform, even if they are not a customer. This nevertheless implies that the Agent, with the help 
of its e-invoicing solution, can give the SELLER access to the invoices it issues on its behalf (or send them to it), and 
potentially to the related life cycle statuses.
Consequently, whether the PA-E is the one chosen by the SELLER or that of the Agent, it processes the invoice in the same 
way, extracts flow 1, and transmits the invoice to the BUYER.
The Agent can declare itself in the invoice in the Third Party Billing block (EXT-FR-FE-BG-05) of the EXTENDED-CTC-FR profile. 
Tax doctrine BOI-TVA-DECLA-30-20-10-30 (§ 210) recommends, in order to avoid any ambiguity, that invoices issued by a 
third party specially authorized for this purpose should include a statement such as "invoice issued by A on behalf of and for 
the account of B." This can be done in a note (BG-1), with Subject Code (BT-21 = DCL), and the text in BT-22.
When the Agent uses its own PA-E, it is also necessary to arrange for the return of the statuses. This is because the 
implementation of network interoperability with dynamic discovery (PEPPOL) requires that statuses be returned to the 
SELLER's electronic address (BT-34).
It is therefore necessary to organize the SELLER's electronic address so that the statuses arrive where the Agent and the 
SELLER want them to arrive, namely:
• Either on the PA-E of the Invoicing Agent: the most common case. In this case, the Agent enters an electronic
address in BT-34 for which it has designated its PA-E as the status recipient. This address is not intended to be 
entered in the PPF Directory («Annuaire») since it is not supposed to receive invoices.
• Or directly on the SELLER's Accredited Platform, if the latter wishes to directly monitor the life cycle of invoices 
issued by the Agent on its behalf.
In practice, a distinction should be made between the description:
• a solution where the SELLER and the Agent have access to the same PA-E, whether it is the SELLER's PA-E to which 
the Agent has access or the Agent's PA-E to which the SELLER has access (Option 1), 
• or one where the SELLER does not have access to the PA-E of the Agent issuing the invoice (Option 2).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page88 / 149
3.2.18.1.1 Option 1: the SELLER and the Agent share the same e-invoicing platform for issuing invoices
In this case, the e-invoicing platform has set up the SELLER as the issuer of invoices and the AGENT as a third party with the 
authority to create invoices on behalf of the SELLER and to view the life cycle statuses, or even set some of them.
The PA-E may also offer pre-validation services for invoices to be issued by the SELLER.
Step Step name Responsible 
actor Description
0 Invoicing mandate Billing Agent 
and SELLER
The Invoicer and the Seller (Principal) enter into an Invoicing 
Mandate.
1 Creation of the invoice for 
the BUYER
Billing Agent, 
on behalf of 
the SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the Billing Agent creates the invoice (flow 2) via 
its information system or based on the invoicing data provided by 
the SELLER. It may entrust the creation of the invoice to an OD/SC 
(Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the status "Submitted" to the CdD of the PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
4a
Processing of the invoice 
and updating of statuses 
prior to payment
BUYER
The BUYER processes the invoice and sets the corresponding 
processing statuses with its PA-R ("Rejected," "Accepted," "In 
dispute," "Approved," "Partially approved," "Suspended," etc.) for 
transmission to the SELLER through its PA-E. 
In practice, the statuses are sent to the E-PA responsible for the 
invoicing address shown on the invoice in BT-34.
4b Receipt of invoice status SELLER The SELLER receives the invoice status following processing of the 
invoice by the BUYER in accordance with the terms of the life cycle.
5
Payment of the invoice
Creation and transmission 
of the "Payment 
transmitted" status.
BUYER 
/ PA-R
The BUYER pays the invoice to the SELLER. They can send a "Payment 
Sent" status to the SELLER via the PA-R (recommended).
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status
SELLER 
/ PA-E
If the VAT on the invoice is payable upon payment receipt, the 
SELLER creates the "Payment Received" status and transmits it to the 
CdD PPF via its PA-E. The PA-E also transmits the "Payment Received" 
status to the PA-R for the attention of the BUYER.
6c
Receipt of "Payment 
Received" status by the 
BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Payment 
Received"status by the PPF 
CdD
PPF Data 
Concentrator The PPF Data Concentrator (CdD PPF) receives the "Paid" status.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page89 / 149
Figure26: Invoice issued with invoicing mandate (option 1)
As the SELLER has access to the PA-E, both the SELLER and the Invoicer have access to the issued invoice and its life cycle
statuses.
Obligations of the BUYER and its PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the Invoicer:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Be familiar with the SELLER's VAT regime if it affects invoices and enable the SELLER to notify changes 
(particularly for franchisees on a basic basis).
Ø Inform the SELLER of the invoices it creates on its behalf, as well as the life cycle status of these invoices.
Obligations of the SELLER:
Ø Submit the "Paid" status or inform the Agent of the payment so that the Agent can submit it on their behalf.
Obligations of the PA-E:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Optional features of the PA-E for the payment intermediary/third-party Invoicer:
Ø Allow shared access to the Seller on the e-invoicing platform to access invoices issued on their behalf and the 
associated life cycle statuses.
Ø Allow the SELLER to directly assign the status "Paid."
SELLER INVOICER with invoicing mandate
PA-E
BUYER
PA-R
CdD
PPF
Transmission of Flow 1, invoice, and corresponding 
status
2
Receipt of invoice
3
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status
7
Conclusion of an invoicing mandate
Transmission and validation of the draft invoice
Creation of the invoice
1
Processing of the invoice and updating of the 
corresponding statuses
4a
Payment of the invoice and "Payment sent" status
5
Receipt of invoice statuses
4b
Invoice payment receipt
6a
Update of the Payment received status
6b
Receipt of "Payment received" status 
6c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page90 / 149
3.2.18.1.2 Option 2: The SELLER does not have access to the PA-E of the Invoicer
This case is quite similar to Option 1, except that the SELLER does not have access to the PA-E. It is therefore up to the 
Invoicer to share the invoice with the SELLER before it is issued (as part of their mandate), then send them the invoice 
created on their behalf, and then share the life cycle statuses with them.
Finally, the SELLER must inform the Agent when the invoice has been paid so that the Agent can create and send the "Paid" 
status, although it is possible for this status to be sent directly by the SELLER.
Figure27: Invoice issued with invoicing mandate (option 2)
Obligations of the BUYER and its PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Obligations of the Billing Agent:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Be familiar with the SELLER's VAT regime if it affects invoices and enable them to notify changes (particularly for 
franchisees on a basic basis).
Ø Inform the SELLER of the invoices it creates on its behalf, as well as the life cycle status of these invoices.
Ø Organize the SELLER's electronic address (BT-34) so that the life cycle statuses are sent directly to the PA-E, or 
even to the SELLER's Accredited Platform /OD/SC (Compatible Solution), if this has been chosen.
SELLER obligations:
Ø Submit the "Paid" status or inform the Agent of the payment so that the Agent can submit it on its behalf.
Obligations of the PA-E:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
PA-E
INVOICER
With invoicing mandate
SELLER
SC/OD / PA SELLER
BUYER
PA-R
CdD
PPF
Transmission of Flow 1, invoice, and corresponding 
status
2a
Receipt of invoice
3a
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2b
Receipt of "Payment received" status
7
Conclusion of a invoicing mandate
Transmission and validation of the draft invoice
Creation of the invoice
1a
Processing of the invoice and updating of the 
corresponding statuses
4a
Payment of the invoice
5
Receipt of invoice status
4b
Payment receipt for the invoice
6a
Update of the Payment received status
6c
Receipt of "Payment received" status 
6d
Receipt/access to invoice before processing, possibility 
of refusal
1c
Information on payment
6b
Sharing/making available to the SELLER
1b
Information on the progress of invoice processing
4b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page91 / 149
Optional features of the PA-E of the payment intermediary/third-party invoicer:
Ø Allow shared access to the Seller on the PA-E to access invoices issued on its behalf and the related life cycle
statuses.
3.2.18.2 Case No. 19b: Self-billing 
This case deals with self-billing, i.e., the creation and transmission of an invoice by the BUYER to the SELLER. This first 
requires that the BUYER have an invoicing mandate from the SELLER. This means knowing the SELLER's VAT regime when it 
has an impact on invoicing, and in particular if the SELLER is a franchisee or even if they are not subject to VAT. If the SELLER 
is a franchisee, the BUYER must allow for the possibility of the SELLER notifying them of a threshold exceeding.
Self-billed invoices must be sent by the BUYER to the SELLER. Consequently, the BUYER must record purchase invoices that 
it issues (instead of receiving them), and the SELLER must record sales invoices that it receives instead of issuing them.
For invoices within the scope of the reform for the "e-invoicing" component, these invoices MUST be electronic and MUST 
be exchanged through Accredited Platforms («Plateformes Agréées»). However, it is the PA-E issued by the BUYER that must 
produce flow 1 and transmit it to the CdD of the PPF.
The statuses also work in reverse, except for those relating to payment:
• The statuses "Submitted" / "Rejected on issue" / "Issued" are created by the BUYER's PA-E.
ü However, in the event of a "Rejected on issue" status, the SELLER must be informed, as the rejected invoice is 
supposed to be in their accounts, as well as its cancellation (and this invoice could not be exchanged between 
PAs). Otherwise, the SELLER will have gaps in the numbering sequence created by its BUYER on its behalf.
• The statuses "Received," "Rejected," and "Made Available" are created by the SELLER's PA-R. In the event of a 
"Rejected" status, the BUYER MUST cancel the invoice in its accounts and the SELLER MUST manage the fact that 
an invoice has been created in its name and on its behalf and has been "Rejected."
• The processing statuses are created by the SELLER, namely:
ü "Refused" status, normally rare as it is only used in the event of incorrect addressing, but which will result in 
the cancellation of the corresponding pre-filled VAT and may require the invoice to be canceled by both parties. 
In this case, since the recipient is the SELLER, they cannot consider that the invoice issued on their behalf does 
not have to be accounted for. The BUYER and the SELLER must therefore cancel the "Rejected" invoice in their 
respective accounts, justified by the "Rejected" status. In the event of an erroneous rejection, the SELLER and 
the BUYER may agree to disregard it and continue processing the invoice. However, the pre-filled VAT amounts 
for each of them will differ from their respective VAT returns by the amount of the items on the rejected invoice.
ü "Approved," "Partially Approved," and "Suspended" statuses, which may lead to credits or a "Completed" status 
transmitted by the BUYER.
• The "Completed" status is transmitted by the BUYER (issuer of the invoice).
• However, payment statuses continue to be issued by the BUYER (or on its behalf) for the "Payment Sent" status 
and by the SELLER (or on its behalf) for the "Received" status.
• In the event of factoring after the invoice has been transmitted, the corresponding status remains issued by the 
SELLER (or on its behalf).
Self-billed invoices must be declared using the following dedicated typecodes (BT-3):
• 389: self-billed invoice (equivalent to 380)
• 501: self-billed factored invoice (upon issuance, equivalent to 393: factored invoice)
• 500: self-billed pre-payment invoice (equivalent to 386: pre-payment invoice)
• 471: self-billed corrected invoice (equivalent to 384: corrected invoice)
• 473: self-billed corrected invoice factored (equivalent to 472: corrected invoice factored)
• 261: self-billed credit note (equivalent to 381: credit note)
• 502: self-billed credit note factored (equivalent to 396: factored credit note).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page92 / 149
The management rules are then the same as those for invoices and credit notes, bearing in mind that PA-E is the Accredited 
Platform of the BUYER (issuer of the invoice) and PA-R is the Accredited Platform of the SELLER (recipient of the invoice), 
and with the following addressing rules:
• The self-billed invoice (identified by the BT-3 value) MUST be issued by the BUYER (and therefore the PA-E MUST 
have the BUYER on file), addressed to the SELLER.
• The recipient's invoicing electronic address is the one in the SELLER block, in BT-34.
• The PPF Directory («Annuaire») MUST be consulted for the SELLER (SELLER's SIREN in BT-30). Where applicable, the 
SELLER's SIRET is in BT-29 with a schemeID (BT-29-1) equal to "0009". Similarly, a SELLER's CODE_ROUTAGE may be 
present in BT-29 with the schemeID "0224".
• The electronic address for receiving return statuses is the address in the BUYER block in BT-49. The PA-E and PA-R 
can organize exchanges so that statuses are exchanged between them throughout the life cycle, or so that they are 
exchanged between the Platforms in charge of the respective electronic addresses after a Platform change 
(particularly in the case of network interoperability with dynamic discovery such as the PEPPOL network).
The steps in management case No. 19b, illustrated in the diagram below, are as follows:
Step Name of step Responsible 
actor Description
0 Invoicing mandate BUYER and 
SELLER
The BUYER and the SELLER enter into a Invoicing Mandate giving the 
BUYER the ability to self-bill the SELLER.
1 Creation of the invoice by 
the BUYER to the SELLER BUYER
Following a commercial transaction, the BUYER creates the invoice 
(flow 2) via its information system. It may entrust the creation of the 
invoice to an OD/SC (Compatible Solution) or to its PA-E. It sends it to 
its PA-E for processing.
The invoice has a type code (BT-3) in the list of self-billing codes.
2
Transmission of flow 1, the 
invoice (flow 2) and related 
statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the SELLER. 
It must send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The SELLER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the SELLER for processing.
4a
4b
Rejection or refusal of the 
invoice by PA-R or the 
SELLER 
PA-R
SELLER
If the invoice is "Rejected" by PA-R or "Refused" by the SELLER, the 
corresponding status is transmitted to the BUYER.
4c Cancellation of the invoice 
following Rejection/Refusal
BUYER
SELLER
In the event of a Refusal or Rejection of an invoice upon receipt (PAR), the SELLER on the one hand and the BUYER on the other must 
cancel the invoice in their accounts, justified by the "Rejected" 
status. 
In the event of an erroneous Refusal, the BUYER and the SELLER may 
continue processing the invoice if they wish, resulting in a 
discrepancy between the pre-filled VAT and their declaration.
4d
4e
4f
In the event of acceptance 
of the invoice ("Accepted" 
status), processing of the 
invoice by the SELLER, with 
the following life cycles
SELLER
The SELLER processes the invoice and sets the corresponding 
processing statuses in its PA-R ("Accepted," "In dispute," "Approved," 
"Partially approved," "Suspended," etc.) for transmission to the 
BUYER via its PA-E. 
In practice, the statuses are sent to the PA-E responsible for the 
invoicing address shown on the invoice in BT-49.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page93 / 149
Step Name of step Responsible 
actor Description
4g Receipt of invoice status BUYER The SELLER receives the invoice status following processing of the 
invoice by the BUYER in accordance with the terms of the life cycle.
5
Payment of the invoice
Creation and transmission 
of the "Payment 
transmitted" status.
BUYER 
/ PA-E
The BUYER pays the invoice to the SELLER. They can send a "Payment 
Sent" status to the SELLER via the PA-E (recommended).
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status
SELLER 
/ PA-R
If the VAT on the invoice is payable upon rpayment eceipt, the 
SELLER creates the "Payment Received" status and transmits it to the 
CdD PPF via its PA-E. The PA-E also transmits the " Payment
Received" status to the PA-E for the attention of the BUYER.
6c
Receipt of "Payment 
Received" status by the 
BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Payment 
Received" status by the PPF 
CdD
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Payment 
Received" status.
Figure28: Self-billing
SELLER
PA-R
BUYER
PA-E
Receipt of invoice
3
CdD
PPF
Transmission of billing data
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status 7
"Payment received" status
6b
Processing statuses
4f
Receipt of invoice statuses
4g
Receipt of "Payment received" status 
6c
Conclusion of an invoicing mandate
Creation of the invoice
1
Processing of the invoice
4
Payment of the invoice
5
Payment receipt for the invoice
6a
Transmission of Flow 1, the invoice, and corresponding 
status
2
"Refused" or "Rejected" 4 Accepted
4d Receipt of "Refused" or "Rejected" status
4b
Cancellation of invoice justified by life cycle "Rejected"
4c
Processing and 
Cancellation of Invoices
4c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page94 / 149
BUYER's obligations:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR) and the practice of self-billing, 
including the management of a chronological series of invoice numbers dedicated to each SELLER.
Ø Have an invoicing mandate with its SELLER, taking into account its VAT regime.
Obligations of the BUYER's PA-E:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Know how to process self- billed invoices, i.e., process a self-billed -invoice on behalf of a BUYER listed on its 
platform to the SELLER identified in the invoice in block BG-4, then manage partially reversed life cycle
exchanges.
SELLER obligations:
Ø Know how to process minimum base formats and profiles (invoice and CDAR) and the practice of self-billing.
Ø Have an invoicing mandate with its BUYER.
Ø Know how to process self-billed invoices that have been rejected or even refused by the BUYER.
Obligations of the SELLER's PA-R:
Ø Know how to process minimum base formats and profiles (invoice and CDAR) and the practice of self-billing.
Ø Know how to accept self-billed invoices addressed to one of your users, identified in the SELLER (BG-4) block of 
the invoice.
Ø Know how to manage partially reversed life cycle status exchanges.
3.2.19 Cases 20 and 21: Pre-payment invoice and final invoice after advance payment
A payment on account for a purchase or the provision of services implies a firm commitment by both parties and constitutes 
an advance payment. All taxable entities are required to issue an invoice for advance payments paid to them (Article 289-
I.1.c of the CGI) before any of the transactions referred to in I.1.a and b of the same article are carried out (unless expressly 
provided otherwise). VAT is payable upon receipt of the advance payment for both the delivery of goods and the provision 
of services, even if option on “debits”.
The buyer pays an initial installment on the amount due for the purchase of goods or services. For example, when a company 
hires the services of a moving company, it must pay part of the total amount before the move is carried out. The moving 
company issues a pre-payment invoice and a final invoice after the advance payment has been paid following the move.
The pre-payment invoice is not recorded in the income statement, but in the balance sheet (customer advance payment for 
the SELLER or supplier advance payment for the BUYER). It has the following characteristics:
• The invoice type code (BT-3) SHALL be "advance payment": 386 for a pre-payment invoice, 503 for a CREDIT NOTE 
on a pre-payment invoice.
• The Billing Framework (“Cadre de Facturation“) is the one corresponding to the invoice, generally B1 / S1 / M1 
depending on whether it is an advance payment on Goods, Services or Mixed.
• The wording of the pre-payment invoice line can be a simple reference to a quote indicating a pre-payment 
percentage, for example, "30% pre-payment on quote xxx."
• VAT applies to each line, as in a standard invoice. If there are multiple applicable VAT rates, the corresponding 
number of pre-payment lines should be created, on a pro rata basis.
The final invoice after the advance payment must detail all the goods or services invoiced, which are recorded in the income 
statement (revenue or expenses). This leads to a VAT calculation for the entire service or delivery, even though part of it has 
already been paid for in the pre-payment invoice. There are two possible solutions to this issue:
• First, as is common to both approaches, the following rules must be followed:
ü Typecode (BT-3): the one corresponding to the invoice (i.e., 380, etc.)
ü Billing framework (“Cadre de Facturation”): B4 / S4 / M4 meaning "final invoice after advance payment."

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page95 / 149
ü Reference to the pre-payment invoice(s) in the header, in block BG-3, with:
§ BT-25: pre-payment invoice number
§ BT-26: Date of the pre-payment invoice
§ EXT-FR-FE-02: pre-payment invoice type code (only usable with the EXTENDED-CTC-FR profile)
• Then, first option: include the advance payment lines in the invoice lines:
ü Name/Description (BT-153/BT-154): indicating an advance payment transfer
ü Quantity: -1
ü Gross and Net unit price (BT-148 / BT-146): amount of the advance payment transferred
ü Reference to previous invoice in the line (EXT-FR-FE-BG-06): the reference to the pre-payment invoice, number, 
date, and above all the type code (EXT-FR-FE-137), equal to 386, which allows this line to be identified as an 
advance payment transfer for automatic processing.
ü Then applicable VAT, total without VAT as for all lines
ü As a result, line-item accounting allows for the recording of the entire turnover/expenses on the one hand and 
the counterparties of the advance payments on the other. The total without VAT of the invoice (BT-109) is equal 
to the net amount between the total without VAT of the entire service/delivery minus the advance payment 
without VAT. The VAT (BT-110) therefore corresponds to the net VAT due. The total amount with VAT (BT-112) 
corresponds to the net amount payable (BT-115).
ü Advantage: the VAT is correct. The pre-filling is correct.
ü Disadvantage: it is necessary to record the amount on the line, because recording it at the bottom without 
correction would result in recording revenue/expenses reduced by the pre-tax amount of the pre-payment(s).
• Second option: use the amount already paid at the bottom (BT-113):
ü The total without VAT (BT-109) corresponds to the total services/deliveries. This is the amount that is reported 
in flow 1.
ü VAT is calculated on this basis, so it includes the VAT already paid with the advance invoice. This is the amount 
that is reported in flow 1.
ü The total amount with VAT corresponds to the total amount of services/deliveries.
ü The amount already paid (BT-113) is equal to the total with VAT on the pre-payment invoice.
ü The net amount payable corresponds to the balance payable.
ü Disadvantage: the amount already paid does not indicate the VAT already paid on it. The data reported in flow 
1 does not take into account the VAT already paid through the advance payment. The pre-filled VAT will 
therefore be incorrect.
ü The presence of the previous invoice and the "Cadre de Facturation" B4/S4/M4 should be sufficient to inform 
the BUYER of the specific accounting requirements necessitating the previous invoice for the correct accounting 
of the invoice and VAT.
ü NOTE: It is possible to write the corrected VAT details in a note (bearing in mind that there is a Factur-X 
extension for this purpose, which is not currently integrated into the EXTENDED-CTC-FR profile).
The pre-payment invoice may be an invoice that has already been paid, when it is created after the advance payment has 
been paid. It may also be an invoice to be paid when the SELLER issues a pre-payment invoice to be paid.
Advance invoices that have already been paid follow the same rules as invoices that have already been paid, namely: 
• "Cadre de Facturation" B2 / S2 / M2 ("Submission of an invoice that has already been paid") or "Cadre de 
Facturation" B1 / S1 / M1;
• Amount paid (BT-113) equal to the total amount with VAT of the invoice (BT-112);
• Consequently, the Amount Payable (BT-115) is equal to 0;
• The Due date (BT-9), if present, is equal to the date of payment of the advance payment (prior to or equal to the 
invoice date).
• As VAT is payable upon payment, the SELLER SHALL send a “Payment Received” life cycle status (via a Lifecycle 6 
flow) to its PA-E so that it can be forwarded to the PPF's CdD and the BUYER's PA-R. This status MAY be sent at the 
same time as the invoice is issued and MUST be sent during the e-reporting period for payment data.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page96 / 149
• However, the administration cannot impose the transmission of payment data relating to advance payments on 
deliveries of goods as the law currently stands. Nevertheless, the "Received" statuses on these pre-payment
invoices for deliveries of goods can be sent to the PPF CdD, which will accept them, as future changes to the law 
may make them mandatory. 
The steps in cases 20 (advance payment invoice) and 21 (final invoice) are as follows for an advance payment invoice that 
has already been paid (phases 4 of invoice processing and statuses are hidden): 
Step Name of the stage Responsible 
actor Description
5 Payment of advance 
payment by the BUYER BUYER The BUYER pays an advance payment on the upcoming delivery or service to be provided.
6a Receipt of the advance 
payment by the SELLER SELLER The SELLER receives the advance payment.
1
Creation of the prepayment invoice already 
paid to the BUYER.
SELLER
Following payment of an advance payment, the SELLER creates the 
pre-payment invoice already paid (flow 2) via their information 
system. They can entrust its creation to an OD/SC (Compatible 
Solution) or to their PA-E. They send it to their PA-E for processing.
2
Transmission of flow 1, the 
invoice (flow 2), and the 
related statuses.
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the status "Submitted" to the CdD PPF.
3 Receipt of the advance 
invoice already paid PA-R
The BUYER's platform (PA-R) receives the advance invoice that has 
already been paid (flow 2 or flow 3), performs the regulatory checks, 
creates the transmission statuses (here "Received," then "Made 
Available") and makes the invoice available to the BUYER for 
processing.
6b
Issuance of the "Payment 
Received" status for the 
advance invoice already 
paid
SELLER 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER creates 
the "Payment Received" status and transmits it to the CdD-PPF via its 
PA-E. The PA-E also transmits the "Received" status to the PA-R for 
the attention of the BUYER.
6c
Receipt of the "Payment 
Received" status of the 
advance invoice already 
paid by the BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Payment 
Received" status for the 
pre-payment invoice 
already paid by the CdDPPF
CdD PPF The PPF Data Concentrator (CdD-PPF) receives the "Payment 
Received" status for the advance invoice already paid.
1 Creation of the final invoice 
for the BUYER SELLER
Once the delivery has been made or the service provided, the SELLER 
creates the final invoice after payment (flow 2) via its information 
system. It may entrust the creation of the invoice to an OD/SC 
(Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page97 / 149
Step Name of the stage Responsible 
actor Description
2
Transmission of flow 1, the 
final invoice (flow 2) and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the PPF Data
Concentrator (CdD PPF). It MUST also send the invoice (flow 2 or flow 
3) to the BUYER's PA-R. It must send the status "Submitted" to the 
CdD PPF.
3 Receipt of the final invoice PA-R
The BUYER's platform (PA-R) receives the final invoice after payment 
(flow 2 or flow 3), performs the regulatory checks, creates the 
transmission statuses (here "Received," then "Made Available") and 
makes the invoice available to the BUYER for processing.
5
Processing of the final 
invoice and payment of the 
balance
BUYER The BUYER processes the invoice and pays it to the SELLER.
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the final invoice (off-circuit).
6b
Issuance of the "Payment 
Received" status for the 
final invoice
SELLER 
/ PA-E
If the VAT on the invoice is payable upon payment receipt, the 
SELLER creates the "Payment Received" status and transmits it to the 
CdD PPF via its PA-E. The PA-E also transmits the "Payment Received" 
status to the PA-R for the attention of the BUYER.
6c
Receipt of the "Payment 
Received" status of the 
final invoice by the BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Payment 
Received" status for the 
final invoice by the CdD PPF
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Payment 
Received" status for the final invoice.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page98 / 149
Figure29: prepayment invoice after advance payment already paid and final invoice
As a reminder, the date of payment of the advance must be indicated on the advance invoice (Article 242 nonies A I 10° of 
Annex II to the CGI) once it has been determined and if it differs from the date of issue. This information must be mentioned 
on the invoice and transmitted in a structured format (BT-9 tag may be used, see management rule BR-FR-CO-07 of Standard 
XP Z12-012). However, even if the due date is entered with the date of payment of the advance payment, a "Received" 
status MUST also be transmitted to meet the e-reporting payment obligation, where applicable.
NOTE: in some cases, a delivery or service may be subject to one or more progress invoices. In this case, the revenue for the 
SELLER and the expense for the BUYER are recognized, and the corresponding invoice is not a pre-payment invoice, but a 
normal commercial invoice.
It is also possible to issue advance invoices that follow a standard process (see diagram below). It is also possible to issue a 
credit note on an advance invoice or a corrected invoice for an advance invoice, under the same conditions as a "standard" 
invoice.
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of the pre-payment invoice
3
Transmission of Flow 1, the invoice, 
and corresponding status
2
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Transmission of statuses (payment data)
7
Creation of the already paid pre-payment invoice
1
Advance Payment
5
Payment receipt
6
Update of « Payment received" status
6b
Receipt of final invoice
3 Transmission of Flow 1, the invoice, and corresponding 
status
2
Creation of the final invoice (with mention of the 
deposit)
1
Processing of the final invoice and payment
5
Collection
6a
Update of "Payment received" status
6b
2
Receipt of "Payment received" status 
6c
Receipt of "Payment received" status 
6c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page99 / 149
Figure30: prepayment invoice and final invoice
BUYER'S OBLIGATIONS:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
Ø Know how to identify advance invoices and final invoices and process the specific accounting and impact on VAT 
returns (particularly on the final invoice).
OBLIGATIONS OF THE BUYER'S PA-E:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
SELLER obligations:
Ø Know how to process the minimum standard formats and profiles (invoice and CDAR).
Ø Know how to create advance invoices and final invoices.
SELLER's PA-R obligations:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
SELLER
PA-E
BUYER
PA-R
PPF
Receipt of the pre-payment invoice
3 Transmission of Flow 1, the invoice, 
and corresponding status
2
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Transmission of statuses (payment data)
7
Creation of the pre-payment invoice (payable)
1
Processing of the pre-payment invoice and payment
5
Payment receipt
6a
Update of "Payment received" status
6b
Receipt of final invoice
3 Transmission of Flow 1, the invoice, and corresponding 
status
2
Creation of the final invoice (with mention of the 
advance paiment)
1
Processing of the final invoice and payment
5
Payment receipt
6a
Update of "Payment received" status
6b
2
Receipt of "Payment received" status 
6c
Receipt of "Payment received" status 
6c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page100 / 149
3.2.20 Case No. 22a: Invoice paid with early payment discount for services for which VAT is payable upon receipt of 
payment 
An early payment discount is an option offered to a customer to pay their invoice sooner than expected in exchange for a 
reduction in price. The amount of the early payment discount does not appear on the invoice issued; only a note detailing 
the terms of the early payment discount is included on the invoice. 
In the case of services, the administration may take the early payment discount granted into account based on the payment 
data provided. The "Payment Received" status will indicate the amount actually received, i.e., minus the early payment 
discount applied (with VAT minus early payment discount).
For deliveries of goods or operators who have opted for debits, as well as early payment discount net of tax, refer to case 
22b. 
As a reminder, the mention of an early payment discount is mandatory, which means either that no early payment discount
has been granted or that the early payment discount conditions are detailed. It is entered in an invoice note (BT-21/BT-22) 
with:
• Subject code: "AAB";
• Text: early payment discount mention.
The specific features of the life cycle or process are:
• Transmission of flow 1 and e-reporting of payment data via the SELLER's platform;
• The creation of an early payment discount does not require the issuance of a credit note if it is mentioned on the 
invoice (in the early payment discount conditions, i.e., in the note with subject code "AAB") that the deductible tax 
is limited to the price actually paid by the BUYER.
• To indicate the application of the early payment discount, the BUYER MAY send a "Payment Transmitted" status 
comprising 2 MDG-43 blocks:
ü The first (or several, one per applicable VAT rate if the BUYER wishes to provide this information) indicating the 
payment with:
§ MDT-207 (Data type code): MPA (meaning "Amount paid").
§ MDT-215 (Amount): the Amount paid (with VAT minus early payment discount applied).
§ MDT-224: applicable VAT rate (optional, only if the BUYER wishes to inform the SELLER of the applicable VAT 
details in their payment).
ü The second to indicate the application of the early payment discount:
§ MDT-207 (Data type code): ESC (meaning " early payment discount applied").
§ MDT-215 (Amount): the amount of the early payment discount applied, i.e., unpaid.
The steps in management case No. 22a, illustrated in the diagram below, are as follows:
Step Step name Responsible 
actor Description
1
Creation of the invoice for 
the BUYER, with early 
payment discount terms
SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page101 / 149
Step Step name Responsible 
actor Description
5a
5b
Payment of the invoice 
with application of the 
early payment discount
Creation and transmission 
of the "Payment Sent" 
status
BUYER 
/ PA-R
The BUYER pays the invoice to the SELLER with the early payment 
discount applied. They can send a "Payment Sent" status to the 
SELLER via PA-R (recommended). This status can also indicate that 
the early payment discount has been applied.
5c Receipt of "Payment Sent" 
status
SELLER 
/ PA-E The SELLER receives the "Payment Sent" status from its PA-E
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
6b Issuance of "Payment 
Received" status
SELLER 
/ PA-E
If the VAT on the invoice is payable upon receipt, the SELLER creates 
the status "Received" and transmits it to the CdD PPF via its PA-E. 
The PA-E also transmits the status "Received" to the PA-R for the 
attention of the BUYER.
6c
Receipt of the "Payment 
Received" status by the 
BUYER
BUYER
PA-R The BUYER receives the "Payment Received" status.
7
Receipt of "Payment 
Received" status by the PPF 
CdD
PPF Data 
Concentrator
The PPF Data Concentrator (CdD PPF) receives the "Paid" status. VAT 
is pre-filled based on the "Payment Received" status, up to the 
amount actually received (with VAT minus early payment discount).
Figure31: Invoice paid with allowance (case of service provision, VAT due upon receipt)
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice
3 Transmission of Flow 1, invoice, 
and corresponding status
2
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Transmission of statuses (payment data)
7
Creation of an invoice mentioning early payment 
discount terms
1
Payment of the invoice with the early payment 
discount applied
5a
Receipt of payment
6a
Transmission of the "Payment received" status for the 
amount actually received (with VAT – early payment 
discount applied)
6b
"Payment sent" status
5b
Receipt of "Payment sent" status
5c
Receipt of "Payment received" status 
6c

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page102 / 149
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
SELLER obligations:
Ø Transmit a "Received" status for the amount received.
Obligations of the BUYER:
Ø None. They may transmit a "Payment Transmitted" status to accompany the payment and indicate the 
application of the early payment discount.
3.2.21 Case No. 22b: Invoice paid with allowance for deliveries of goods (or provision of services with VAT option on 
debits)
In the case of an early payment discount, when the SELLER has delivered goods or opted to pay VAT on debits for services 
rendered, the payment details are not transmitted and are not taken into account by the administration. The administration 
therefore has no way of knowing the early payment discount granted by the SELLER to reduce the pre-filled VAT collected 
by the same amount. 
The SELLER could inform the tax authorities of the application of an early payment discount by issuing a credit note. This is 
an option, as credit notes for early payment discounts are not provided for in the legislation. This option is available to 
companies that wish to justify the payment difference following the application of the early payment discount in their 
accounts (and those of the BUYER), and to allow the VAT collected to be taken into account in the pre-filled forms produced 
by the tax authorities through credit note flow 1. 
As a reminder, the mention of an early payment discount is mandatory, meaning either that no early payment discount has 
been granted or that the early payment discount conditions are detailed. It is entered in an invoice note (BT-21/BT-22) with:
• Subject code: "AAB";
• Text: early payment discount mention.
The specific features of the associated life cycle or process are:
• Transmission of the invoice (flow 2) mentioning the terms and conditions for applying the allowanceearly payment 
discount;
• Transmission of the life cycle (flow 6);
• Transmission of a credit note (flow 2) mentioning the amount of the early payment discount applied.
In the case of a credit note, there are two approaches:
• A credit note with VAT. In this case, the credit note contains the following information:
ü The invoice typecode is credit note (381, 261, etc.)
ü The invoice to which the early payment discount applies is referenced in BG-3 (previous invoice).
ü The total without VAT (BT-109) corresponds to the amount of the early payment discount reported without VAT
ü The total VAT (BT-110) corresponds to the VAT calculated at the rate applicable in the invoice
ü The total with VAT (BT-112) corresponds to the amount of the early payment discount (with VAT).
ü If multiple VAT rates apply, the early payment discount credit must normally include the pro-rated VAT amount
• A tax-free credit note is permitted in the case of an early payment discount (tax-free early payment discount). In 
this case, the credit note contains the following information in particular:
ü The invoice type code is credit note (381, 261, etc.)
ü The invoice to which the early payment discount applies is referenced in BG-3 (previous invoice).
ü The total without VAT (BT-109) corresponds to the amount of the early payment discount reported without VAT
ü In VAT breakdown (BG-23), 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page103 / 149
§ The tax base (BT-116) is equal to the early payment discount amount
§ The VAT type code (BT-118) is "E."
§ The VATEX code (BT-121) is equal to VATEX-FR-CNWVAT
§ The VAT amount (BT-117) is equal to 0
ü In foot totals
§ The total without VAT (BT-109) is equal to the early payment discount amount
§ The VAT total (BT-110) is equal to 0
§ The total with VAT (BT-112) is equal to the early payment discount amount
For information, the VATEX-FR-CNWVAT code applies to all types of tax-net credit notes. 
The steps in management case no. 22b, illustrated in the diagram below, are as follows:
Step Step name Responsible 
party Description
1
Creation of the invoice for 
the BUYER, with early 
payment discount terms
SELLER
Following a commercial transaction (order/delivery, service contract, 
spot purchase, etc.), the SELLER creates the invoice (flow 2) via its 
information system. It may entrust the creation of the invoice to an 
OD/SC (Compatible Solution) or to its PA-E. It sends it to its PA-E for 
processing.
2
Transmission of flow 1, the 
invoice (flow 2), and 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the invoice (flow 2 or flow 3) to the PA-R of the BUYER. 
It must send the status "Submitted" to the CdD PPF.
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the invoice (flow 2 or flow 3), 
performs the regulatory checks, creates the transmission statuses 
(here "Received," then "Made Available") and makes the invoice 
available to the BUYER for processing.
5a
5b
Payment of the invoice 
with application of the 
early payment discount
Creation and transmission 
of the "Payment 
transmitted" status
BUYER 
/ PA-R
The BUYER pays the invoice to the SELLER by applying the early 
payment discount. They can send a "Payment Sent" status to the 
SELLER via PA-R (recommended). This status can also indicate that 
the early payment discount has been applied.
5c Receipt of "Payment 
Transmitted" status
SELLER 
/ PA-E The SELLER receives the "Payment Transmitted" status from its PA-E
6a Payment receipt for the 
invoice SELLER The SELLER receives payment for the invoice (outside the circuit).
1
Creation of the early 
payment discount CREDIT 
NOTE (with or without VAT)
SELLER
The SELLER creates a credit note corresponding to the early payment 
discount, either with applicable VAT or net of VAT with the VATEX 
code (BT-121) VATEX-FR-CNWVAT
2
Transmission of flow 1, the 
early payment discount
credit note (flow 2) and the 
related statuses
PA-E
Once the PA-E has carried out the regulatory compliance checks, 
including checks for duplicates and the existence of an active 
electronic invoicing address for the recipient, it MUST transmit the 
data required by the Administration (flow 1) to the CdD PPF. It MUST 
also transmit the early payment discount credit note (flow 2 or flow 
3) to the BUYER's PA-R. It must transmit the status "Submitted" to 
the CdD PPF).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page104 / 149
Step Step name Responsible 
party Description
3 Receipt of the invoice PA-R
The BUYER's platform (PA-R) receives the early payment discount
credit note (flow 2 or flow 3), performs the regulatory checks, 
creates the transmission statuses (here "Received," then "Made 
Available") and makes the early payment discount credit note 
available to the BUYER for processing.
Figure32: Invoice paid with allowance (case of delivery of goods or provision of services with VAT on debits)
Obligations of PA-E and PA-R:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR).
SELLER obligations:
Ø None. They can create a credit note for the amount of the early payment discount, either with VAT or net of tax.
Obligations of the BUYER:
Ø None. They can send a "Payment Sent" status to accompany the payment and indicate that the early payment 
discount has been applied.
3.2.22 Case No. 23: Self-billing flow between an individual and a professional
An individual who repeatedly sells or offers services to a professional is engaged in a regular commercial activity and is 
therefore subject to VAT. 
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoice
3 Transmission of Flow 1, invoice, 
and corresponding status
2
Transmission of invoicing data
and statuses "Submitted", "Rejected", "Refused"
2
Creation of an invoice mentioning the early payment 
discount conditions
1
Payment of the invoice with the early payment 
discount applied
5
Receipt of payment
6a
Transmission of the credit note, Flow 1, and 
corresponding status
2
Creation of a credit note for the amount of the early 
payment discount 
1
Receipt of credit note
3
"Payment sent" status, 
with early payment discount applied
5b
Receipt of "Payment sent" status
5c
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page105 / 149
They may not be liable for VAT if they benefit from the basic exemption scheme (Article 293 B of the French General Tax 
Code), but they are still subject to electronic invoicing. 
In most cases, it is the professional who issues the invoice (for example, an energy supplier who is here the BUYER of energy
produced by the person subject to the basic exemption scheme). This is therefore a case of self-billing between taxable 
entities as described in use case 19b of this document.
There is an exception for sales of photovoltaic energy by individuals from a system with a capacity not exceeding 3kWp (see 
BOI BIC CHAMP 80 30 and BOI-TVA-LIQ-30-20-90-20, § 260). These operators are not subject to VAT and are therefore not 
covered by e-invoicing or e-reporting.
3.2.23 Case No. 24: Management of Deposit (“Arrhes”)
Deposits (“Arrhes”) are defined as sums paid as a penalty (Article 1590 of the Civil Code): in this case, the BUYER may cancel 
the sale and renounce their purchase by forfeiting this sum. If the Depositconstitutes compensation, i.e., it does not 
constitute payment for a service (no consideration), it is not included in the VAT taxable base.
In commercial matters, sums paid in advance are more often in the nature of a Deposit on the sale price, which the parties 
cannot withdraw from.
Deposits constitute compensation intended to repair commercial damage. Deposits are outside the scope of VAT; they are 
not covered by e-invoicing or e-reporting. It is recommended that the nature of this sum be specified in the contract or 
receipt given to the buyer.
3.2.24 Case No. 25: Management of vouchers and gift cards
This use case is still under discussion and is therefore subject to significant additions.
Gift vouchers and cards may be single-use or multi-use depending on whether, at the time of issue, the place of delivery of 
the goods or provision of services and the VAT due on those goods or services are known or not.
Examples:
• A card entitling the payee to a certain number of performances in a theater for which the place of taxation and the 
VAT rate are determined constitutes a single-use voucher.
• A gift card that gives access to various goods or services in a network of stores, for which the place of taxation and 
the VAT rate are undetermined, constitutes a multi-use voucher.
3.2.24.1 Principle of the single-use voucher (BUU)
Step 1 - Issuance of the voucher:
The sale of a single-use voucher is subject to VAT when, at the time of issue, the place of delivery of the goods or services 
to which the voucher relates and the related VAT are known (tax base, rate, territoriality). The sale of a single-use voucher 
is subject to VAT on each transfer, and this VAT is payable under the conditions applicable to the underlying transaction: 
delivery of goods or provision of services (see Article 269 of the General Tax Code and BOI-TVA – BASE-20-40). Thus, if the 
underlying transaction related to the BUU constitutes a delivery of goods, VAT will be payable upon the delivery of the 
voucher, and if the underlying transaction related to the voucher constitutes a provision of services, VAT will be payable 
upon receipt of the price relating to the purchase of the voucher. The physical delivery of the goods or the actual provision 
of the services in exchange for a BUU accepted in full or partial consideration by the SELLER or the service provider is not 
considered a separate transaction. 
Each subsequent transfer of the single-use voucher will also be subject to VAT, which will be payable under the same 
conditions as for the first transfer.
Each transfer of the single-use voucher by a taxable entity will fall within the scope of e-invoicing (sale of gift cards to a 
taxable entity) or e-reporting (sale to individuals) for the company selling it.
Step 2 - Use of the gift voucher:
The use of the single-use voucher by its payee (voucher holder) in exchange for the delivery of goods or the provision of 
services is not subject to VAT.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page106 / 149
However, when the issuer of the BUU is separate from the service provider or supplier of the services or delivery related to 
the BUU, the service provider or supplier is deemed to have delivered or supplied the goods or services related to this 
voucher to the taxable entity and must therefore invoice the issuer for this service (e-invoicing). 
In this case, the supplier of the service or delivery of goods sells to the issuer of the BUU, who is therefore the BUYER on this 
invoice. The issuer of the BUU has in fact sold the service or goods in advance to the recipient of the BUU. If the latter is an 
entity not subject to tax:
• The BUU Issuer declares its sale of BUU in e-reporting at the time of sale of the BUU.
• The service provider or goods supplier invoices the BUU issuer following its use.
• The BUU Issuer invoices the supplier for its commission or management fees, if applicable.
The VAT relating to this transaction will be payable under the same conditions as the underlying transaction. Thus, when 
the voucher gives access to a service, VAT will be payable when the supplier invoices the issuer. When the voucher gives 
access to goods, VAT is payable when the goods are delivered in exchange for the voucher.
When the issuer of the voucher is also the supplier or service provider, the delivery of the voucher in exchange for goods or
services to the customer is not subject to VAT, as it is not considered a separate transaction from the sale of the voucher 
(see step 1).
Commissions or management fees may be incurred throughout the voucher marketing chain and are subject to VAT. They 
must be invoiced separately, including the relevant VAT. This invoice for commissions or management fees falls within the 
scope of e-invoicing.
Figure33: Management of single-use vouchers for the provision of services to an individual by a third-party service provider
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
3.2.24.2 Principles of the multi-use voucher (BUM)
Step 1 - Issuing the multi-use voucher:
The sale of a multi-use voucher is not subject to VAT if, at the time of issue, the place of delivery of the goods or provision 
of services and the VAT due on those goods or services are not known. 
Issuer of the VOUCHER – B2B BUYER 
PA-EBR
THIRD-PARTY SERVICE SELLER
PA-VE
Individual
CdD
PPF
Transmission of Flow 1, invoice 
and corresponding status
2
E-reporting B2C corresponding to the sale of the 
voucher
2
Receipt of invoicing data, B2C transaction 
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of service invoice 
3
Purchase of a voucher (gift card) Sale of single-use voucher
Use of the voucher 
Processing invoices and updating statuses
4a
Receiving Processing statuses
4b
Payment of the invoice
5
Payment receipt of the invoice
6a
Update of "Payment received" status
6b
Receipt of Payment received status
6c
Provision of service 
and creation of the service invoice
1
Creation of the commission invoice 1
Transmission of Flow 1, the commission invoice 
and corresponding status
Receipt of commission invoice 2
3
Invoice processing and status updates
4a
Receipt of processing status
4b
Payment of invoice
5
Payment receipt of the invoice
6a
Update of "Payment received" status
6b
Receipt of "Payment received" status
6c
Receipt of "Payment received" status
7
PA-VR PA-EBE
2 7

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page107 / 149
The amounts paid for the purchase of multi-purpose vouchers are outside the scope of VAT and are not subject to electronic 
invoicing or e-reporting.
Step 2 - Use of the multi-purpose gift voucher:
The use of the multi-purpose voucher by its payee (voucher holder) in exchange for the delivery of goods or the provision 
of services is subject to VAT. If the payee of the voucher is a private individual, this is a transaction between a taxable entity
and a non-taxable entity that falls within the scope of e-reporting. 
VAT is payable under the same conditions as the underlying transaction. Thus, the tax is payable on the date of acceptance 
of the multi-purpose voucher by the supplier in the case of a voucher giving access to goods.
If the multi-purpose voucher gives access to a service for the benefit of its payee (the voucher holder), the tax becomes 
payable upon receipt of the price of the transaction, i.e., when the reimbursement made by the BUM issuer is received. 
When the issuer of the voucher is also the service provider, the redemption of the voucher in exchange for a service will 
make VAT payable.
Commissions or management fees may be incurred throughout the BUM marketing chain and are subject to VAT. They must 
be invoiced separately, including the relevant VAT, and this invoicing will fall within the scope of e-invoicing.
Figure34: Management of a Multi-Purpose Voucher for the provision of services to an individual by a third-party supplier
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
3.2.25 Case No. 26: Invoices with contractual reservation clauses
This case corresponds to the sale of services or goods under a contractual reservation clause, which retains part of the price 
to be paid only if the reservation is lifted. 
The service or goods are invoiced in full, with, for example, 95% payable upon submission of the invoice and 5% once the 
reservation clause has been lifted, if it is lifted. The EN16931 and EXTENDED-CTC-FR profiles do not allow you to enter a 
payment schedule with an amount under a retention clause. The retention guarantee can therefore only be expressed in a 
note:
Issuer of the VOUCHER – B2B BUYER 
PA-EBR
THIRD-PARTY SERVICE SELLER
PA-VE
Individual
CdD
PPF
Receipt of invoicing data, B2C transactions 
and statuses "Submitted", "Rejected", "Refused" 2
Purchase of a voucher (gift card) Sale of multi-purpose vouchers
Use of voucher 
Payment receipt of MPV refund Refund of the MPV
E-reporting of corresponding B2C payment 
to the collection of the service
7
Provision of service against payment with MPV 1
Creation of the commission invoice
1
Transmission of Flow 1, commission invoice 
and corresponding status
Receipt of commission invoice 2
3
Processing of the invoice and status update
4a
Receipt of processing status
4b
Payment of the invoice
5
Payment receipt of the invoice
6a
Update of "Payment received" status
6b
Receipt of "Payment received" status
6c
Receipt of "Payment received" status 
7 E-reporting of payments on transactions (felow 10.4) 
PA-VR PA-EBE
2 7
MPV reimbursement request Receipt of MPV request
E-reporting of B2C transaction sales for service sales 
(flow 10.3)
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page108 / 149
• Subject code (BT-21): ABU
• Text (BT-22): reservation clause
In the case of deliveries of goods or services with the option of paying VAT on debits, VAT is debited, i.e., when the invoice 
is issued:
• In the absence of e-reporting of payment data, this retention has no impact on VAT until it becomes final.
• The implementation of the retention of guarantee must give rise to a credit note by the SELLER.
Services without the option to pay VAT on debits:
• In this case, only the amounts received are subject to VAT. VAT pre-filling is based on the "Payment Received" 
status transmitted by the SELLER.
• The SELLER therefore sets a "Received" life cycle status as soon as the initial partial payment is received (e.g., 95%), 
which will be transmitted to the BUYER and the CdD PPF. If the reservation clause is lifted and as soon as the balance 
is paid, the SELLER sets a corresponding "Received" life cycle status.
• The implementation of the retention of guarantee shall give rise to a credit note by the SELLER. In this case, it is not 
necessary to transmit a "Paid" status for the balance or for the credit note.
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
3.2.26 Case No. 27: Management of toll tickets sold to a taxable entity
In principle, toll tickets issued to a taxable entity fall within the scope of electronic invoicing, but there is administrative 
tolerance regarding them. 
Receipts issued at toll booths that mention the following will therefore be considered as documents equivalent to invoices:
• The VAT rate and amount;
• A sequential issue number;
• A space reserved for the user.
As the customer is not known to the taxable SELLER, these transactions may be considered B2C transactions and therefore 
subject to B2C e-reporting.
This case is handled as follows:
• The SELLER of toll tickets will declare their sales through B2C e-reporting (flow 10.3: daily sales total).
• The administration will thus be able to pre-fill the VAT received from the SELLER but not the VAT deductible by the 
taxable BUYER.
• Toll machines therefore do not need to be adapted to add information to their receipts.
• The BUYER will be able to process toll receipts for accounting purposes and declare the deductible VAT, which will 
therefore differ from the pre-filled amount for these services.
If toll services are provided as part of a subscription or credit card scheme enabling the SELLER to identify the BUYER, the 
service must be invoiced (usually on a monthly basis) and falls within the scope of electronic invoicing.
For other machines (e.g., parking meters), tickets issued to a taxable entity fall within the scope of electronic invoicing. 
Vending machine solution providers must develop features that allow the taxable BUYER to enter the information necessary 
to receive an electronic invoice on the vending machine. In the event that the vending machine is not yet up to date when 
the reform comes into force, given the large number of vending machines in use, the customer will not be known to the 
SELLER subject to VAT, and these transactions will therefore be treated temporarily as B2C transactions until the machine is 
updated, and will be subject to B2C e-reporting, following the same process as for toll tickets.
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page109 / 149
3.2.27 Case No. 28: Management of restaurant bills issued by a SELLER subject to tax established in France
In principle, restaurant bills to a taxable entity fall within the scope of electronic invoicing, but there is an administrative 
tolerance for bills under €150 without VAT:
• Restaurant bills under €150 without VAT may not mention the customer's identification details.
• When the amount of the service is less than €25 and the customer (non-trader) does not request it, the service 
provider is not obliged to issue an invoice.
To take this tolerance into account, the following solution has been adopted for transactions under €150 without VAT
(including those under €25) for which the customer subject to tax does not request an invoice:
• The SELLER (restaurant owner) declares their sale in the B2C e-reporting system (flow 10.3 for sales, by category, 
and 10.4 for the receipt of service payments), as a daily total of sales and payments received.
If the taxable customer requests an invoice, there are two possible approaches:
• Either the request is made at the time of the transaction AND the SELLER is able to create an invoice and exclude 
the service from their cash register so that it is not included in their B2C e-reporting, in which case they create the 
invoice by collecting the BUYER's electronic invoicing address and therefore their SIREN number, which allows them 
to identify the BUYER. This invoice falls within the scope of e-invoicing.
• Either the request is made after payment of the transaction at the cash register, or it is made at the time of sale 
but the SELLER is unable to exclude the sale from their cash register system: the SELLER can then create an 
electronic invoice with a "Cadre de Facturation" (BT-23) equal to S7 or B7, which means that the VAT has already 
been subject to B2C e-reporting and that the transmitted flow 1 must be used solely for the BUYER's VAT pre-filling;
see use case no. 30.
For sales of more than €150 without VAT to a taxable entity, an invoice is mandatory and is subject to e-invoicing if the 
restaurant owner is a taxable entity established in France.
This invoice may fall under use case 30 (electronic invoice following a sale processed in transaction e-reporting), for example 
if the sale was recorded in cash register software that led to its inclusion in B2C e-reporting.
Management of a restaurant bill where the customer is not a taxable entity:
Figure35: Management of restaurant bills for a non-taxable customer
BUYER
SELLER
PA-E
CdD
PPF
Receipt of invoice and payment
3
Receipt of invoicing data 
or transaction data (Flow 1, 10.1, 10.3)
and the statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7
Restaurant bill
1
E-reporting of B2C transaction sales for service sales 
(Flow 10.3)
2
E-reporting of transaction receipts (Flow 10.4)
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page110 / 149
Management of a note for which the customer is liable but is not identified as such at the time of the transaction, or a note 
for less than €150 without VAT:
Figure36: Transaction data declaration for bills under €150
If the customer requests an invoice after the fact and the restaurant owner reports the sale as B2C, see management case 
no. 30 (VAT already collected - Transactions initially processed in B2C e-reporting, subject to an invoice after the fact).
NB: if the transaction is corrected directly in the cash register software, do not apply this case and issue a standard invoice 
with VAT (see below).
Management of a restaurant bill for which the customer is liable for tax and has expressly requested an invoice, or 
restaurant bills exceeding €150 without VAT: this is the nominal case. If the SELLER has already included the sale in their B2C 
e-reporting (or if they cannot cancel it), they must use a "Cadre de Facturation" (BT-23) 7: S7 or B7(see use case no. 30).
Figure37: Issuing/transmitting an electronic invoice for bills exceeding €150
BUYER (taxable person)
PA-R
SELLER
PA-E
CdD
PPF
Receipt of invoice and payment < €150 excl. tax
3
Receipt ofinvoicing data 
or transaction data (flow 1, 10.1, 10.3)
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7
Restaurant bill less than €150 excluding tax
1
E-reporting of B2C transaction sales for service sales 
(Flow 10.3)
2
E-reporting of transaction receipts (Flow 10.4)
2
BUYER (taxable person)
PA-R
CdD
PPF
SELLER
PA-E
Receipt of invoicing data (Flow 1) 
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of statuses (payment data) 7
Restaurant bill > €150 excluding tax or if the customer 
requests an invoice
1
Receipt of the invoice
3
Creation of the invoice
1
Transmission of Flow 1, commission invoice 
and corresponding status
2
Processing of the invoice and status update
4a
Receipt of Processing statuses
4b
Payment of invoice
5
Payment receipt of the invoice
6a
Receipt of "Payment received" status
6c
Update of "Payment received" status
6b

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page111 / 149
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
SELLER requirements:
Ø Know how to create a B2B invoice for a restaurant bill, either with a "Cadre de Facturation" (BT-23) equal to S7
or B7 while retaining the sale in e-reporting (flows 10.3 and 10.4), or in another "Cadre de Facturation" after 
canceling or excluding the sale from the B2C e-reporting circuit.
3.2.28 Case No. 29: Single Taxable Entity within the meaning of Article 256 C of the CGI
This management case concerns transactions external to the single taxable entity, i.e., transactions between a member of 
a single taxable entity and a third party to that single taxable entity. 
Transactions between members of a single taxable entity are outside the scope of electronic invoicing. They may be 
exchanged electronically, including between Accredited Platforms («Plateformes Agréées»), but without the flow 1 or 
"mandatory" statuses being transmitted to the CdD PPF.
As part of the VAT group's compliance in France with VAT Directive 2006/112/EC (already implemented in 20 Member States 
of the European Union), a VAT identifier and a SIREN number will be created for the single taxable entity, who becomes 
solely liable for VAT for all of its members. These two identifiers must be indicated, where applicable, on all sales invoices 
issued by members of the single taxable entity, in addition to their own identifiers. 
For sales invoices issued by a member of a single taxable entity:
• The data relating to the member of the single taxable entity itself must appear in the SELLER block (BG-4). 
• The data relating to the single taxable entity must be entered as follows: 
ü The taxable entity's SIREN number: in BT-29 (private SELLER identifier) with scheme identifier (BT-29-1) equal 
to 0231;
ü In the absence of a block dedicated to the single taxable entity in the EN16931 standard to date, data relating 
to the single taxable entity other than the SIREN number, namely the company name, intra-community VAT 
number, and postal address, must be included in the SELLER'S TAX REPRESENTATIVE block (BG-11).
• The mention "MEMBRE_ASSUJETTI_UNIQUE" (SINGLE TAXABLE ENTITY MEMBER) must be included in a note in BT22, with a subject code TXD in BT-21.
Members of a Single Taxable Entity, although no longer subject to VAT themselves, remain subject to the obligation to 
receive electronic invoices issued by VAT taxable entities established in France. To do so, they will be listed in the PPF 
Directory («Annuaire») and will be able to create electronic invoicing addresses for receiving invoices and will have to use 
an Accredited Platform to receive these invoices. The pre-filling of VAT information relating to the receipt of invoices by 
members of a Single Taxable Entity (deductibility for "traditional" invoices and payment receipt for self-billed invoices) will 
be done as part of the Single Taxable Entity's VAT return.
With regard to e-reporting, as a reminder, each taxable entity is required to submit one 10 flow per e-reporting period and 
per type (one for international B2C and B2B sales, one for international acquisitions, and one for receipts).
However, in the case of Single Taxable Entities, for both international B2C and B2B sales, international B2B sales, as well as 
for international acquisitions and the associated required payment data, it will be permitted to provide 10 flows per member 
of the Single Taxable Entity, and not just a total for all members of the Single Taxable Entity, even if these 10 flows are 
transmitted by the same Accredited Platforms («Plateformes Agréées») for different members of the Single Taxable Entity.
3.2.29 Case No. 30: VAT already collected - Transactions initially processed in B2C e-reporting, subject to a retrospective
invoice 
This management case illustrates the management of transactions between taxable entities subject to e-invoicing that have 
been e-reported (B2C), for example because they have been recorded in software or a cash register system that is the source 
of the e-reporting of transactions (B2C), and which must therefore be subject to an electronic invoice exchanged between 
Accredited Platforms («Plateformes Agréées») and producing a flow 1, thus resulting in double VAT accounting (through 
flow 1 of the invoice AND flows 10.3 and 10.4 of e-reporting of transactions).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page112 / 149
It therefore applies to all cases where the transaction is recorded in a cash register, is subject to B2C e-reporting, and is then 
subject to an electronic invoice at the customer's request. It can be used in particular in the case of restaurant bills that are 
subject to a subsequent invoice.
However, if the error is corrected in the cash register software and is not subject to prior B2C e-reporting, the invoice follows 
the standard electronic invoicing process.
Example: 
A floral decorator provides both services (event manager) and sells goods (plants) to B2B (hotels) and B2C (individuals) 
customers. They record their transactions using cash register software or a cash register system. They are subject to 
electronic invoicing for their B2B transactions and e-reporting for their B2C transactions. For the latter, they transmit their 
transaction and payment data at frequency F. 
Step 1 – Accounting for the B2C transaction:
The floral decorator carries out a B2C transaction paid for in cash. The Z-ticket from their day's work refers to all the 
transactions carried out that day. The floral decorator transmits its cumulative transaction data via an e-reporting flow (flow 
10.3) and, if it has not opted to pay VAT on debits, an e-reporting payment flow (flow 10.4) for the services provided, for 
each day of the e-reporting period. 
Step 2 - Issuing a B2B invoice:
One of its customers is a business and requests an invoice in order to exercise the right to deduct VAT on the transaction. 
The floral designer then issues an electronic invoice. To avoid double counting the pre-tax amount and VAT, this invoice 
must mention the corresponding "Cadre de Facturation" created for this purpose, "VAT already collected," entered in BT23 with the values S7 or B7. This "Cadre de Facturation" allows the corresponding invoice data (flow 1) to be transmitted 
to the tax authorities, indicating that it has already been transmitted via e-reporting (flows 10.3 and 10.4), thus making it 
possible to determine the customer's deductible VAT while avoiding double counting of turnover and VAT collected for the 
merchant. 
The specific features of the data and associated management rules are:
• Business process type (billing framework – “Cadre de Facturation”) (BT-23): B7/S7, meaning "VAT already 
collected".
• VAT accounting and reporting: the SELLER must ensure that the sale is not accounted for twice (once through the 
accounting reported by the cash register software and once through the accounting of the invoice).
The specific features of the transmission of invoicing/transaction and payment data are:
• Transmission of an e-report of cumulative transaction data (flow 10.3) and payment data (flow 10.4), if applicable, 
recorded by the cash register software. 
• Transmission of the invoice with the "VAT already collected" "Cadre de Facturation" (flow 2)
NOTE: The "VAT already collected" "Cadre de Facturation" (B7/S7) can be used in other situations, particularly for invoices 
to be issued when VAT has been declared by a company on its VAT return for the period due and the invoice is issued 
subsequently. In this situation, using Invoicing Box 7 (B7/S7) will mean that flow 1 of the invoice is not taken into account 
for the SELLER's VAT pre-filling (VAT collected), but is taken into account for the BUYER's VAT pre-filling (VAT deductible). 
Thus, the SELLER's VAT pre-fill will not be duplicated since it will already have been pre-calculated through B2C e-reporting.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page113 / 149
Figure38: Electronic invoice management following a sale that has been subject to e-reporting of the transaction (B2C)
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
SELLER obligations:
Ø Know how to create a B2B invoice with a "Cadre de Facturation" (BT-23) equal to S7 or B7 while retaining the 
sale in e-reporting (flows 10.3 and 10.4).
Ø Ensure that the sale and VAT collected are not recorded twice, for example by making a correction entry for B2C 
sales due to the invoice being recorded with the "VAT already collected" "Cadre de Facturation".
3.2.30 Case No. 31: "Mixed" invoices mentioning a main transaction and an ancillary transaction
This case illustrates the management of "mixed" invoices/transactions or "single complex transactions" (involving several 
categories of transactions, one of which is ancillary to the other). The transaction category is either a supply of goods or a 
supply of services.
In accordance with II of Article 257 ter of the CGI, these are activities whose elements are so closely linked that they 
objectively form a single, inseparable economic service, the breakdown of which would be artificial.
Principle: 
In the event that an invoice mentions two transactions: a first transaction considered to be the main transaction, and a 
second transaction considered to be secondary (associated with the main transaction). The terms and conditions for 
transmitting invoicing/transmission and payment data for these two transactions are then determined by the category of 
the main transaction. Payment data must only be transmitted for transactions falling within the category of services in order
to determine the VAT base. 
BUYER
SELLER
PA-E
CdD
PPF
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7 Receipt of invoicing data 
or transaction data (flow 1, 10.1, 10.3)
and "Submitted", "Rejected", "Refused" statuses
2
BUYER (single taxable entity)
PA-R
Payment receipt of the sale Payment for a purchase
Receipt of ticket
The customer declares that they are a single 
taxable entity and requests an invoice
Requests an invoice
0 Creation of the invoice with the "VAT on Payment 
received" S7/M7 (“Cadre de Facturation” : BT-23)
1
Receipt of the invoice 
3
Printing of a receipt and recording of the transaction in 
cash register software/system
2
Transmission of data Flow 1, the commission invoice 
and corresponding status
2
E-reporting of B2C transaction sales for the sale of the 
service (flow 10.3)
2
E-reporting of transaction receipts (flow 10.4)
6
Receipt of "Payment received" status
6c
Update of "Payment received" status
6b
2
7

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page114 / 149
For sales to entities not subject to VAT established in France that are subject to e-reporting of transactions (flows 10.3 and 
10.4), it is up to the SELLER to define the category of the transaction, delivery of goods or provision of services, for the 
transmission of transaction data. It is therefore the category of the main transaction that will be chosen for the secondary 
transactions attached to it.
If the sale covered by e-reporting includes transactions involving the delivery of goods and independent services, then the 
e-reporting of transactions must separate them by combining, on the one hand, sales corresponding to the category 
"delivery of goods" and, on the other hand, sales corresponding to the category "provision of services."
For sales covered by e-invoicing, the category is indicated in the Billing Framework (BT-23 : “Cadre de Facturation”) by the 
first character:
• B: delivery of goods
• S: provision of services
• M: invoice that includes separate lines for the delivery of goods and the provision of services (i.e., that cannot be 
classified as secondary to either goods or services).
Example: 
A clothing store offers a tailor-made alteration service in addition to selling its products. Transactions are carried out in cash 
with immediate payment. They are recorded using software or a cash register system. 
Payment data must be transmitted for transactions falling under the category of "Services" in order to determine the VAT 
base:
• If, following its sale, a product needs to be altered, the alteration is considered ancillary to the sale. This service 
is therefore considered to fall under the category of "Delivery of Goods." It will therefore not be subject to ereporting of payment. 
• If a customer wishes to have a garment altered (purchased as part of another transaction), then this alteration 
will be considered as the main transaction and therefore falls under the category of "Services." It must 
therefore be reported in an e-reporting payment declaration for the amount of the services invoiced (excluding 
the option to pay VAT on debits).
Case 1: the alteration is associated with the sale of a suit. The main transaction is a delivery of goods. 
Figure39: Mixed invoice with main and ancillary transactions categorized as a sale (case of an alteration on sale)
BUYER (not subject to VAT)
SELLER
PA-E
CdD
PPF
Receipt of invoicing data 
or transaction data (Flow 1, 10.1, 10.3)
and statuses "Submitted", "Rejected", "Refused"
2
Payment of the transaction for a purchase of goods 
and order for an associated service Payment receipt for the transaction
Receipt of the ticket Printing of a receipt and recording of the transaction in 
cash register software/system
1
E-reporting of B2C transaction sales for the sale of 
services (Flow 10.3): "Delivery of goods" category
2

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page115 / 149
Case 2: the alteration is the main transaction (with or without an ancillary sale). The transaction is a service provision. If 
goods are sold at the same time, it is a mixed sale. In e-reporting, sales in the "Delivery of Goods" category must be separated 
from those in the "Provision of Services" category
Figure40: E-reporting sale of transaction with INDEPENDENT Provision of Services AND Delivery of Goods
The specific features of transaction and payment data transmission are:
• Transmission of e-reporting of cumulative transaction data (flow 10.3) recorded by the cash register software, by 
category of transaction performed (see G1.68 of the external specifications, Annex 7):
ü TT-77: Date of the cumulative amount subject to e-reporting (10.3)
ü TT-78: Currency code
ü TT-80: VAT payment option (if option for debits for sales of services)
ü TT-81 (Transaction category): 
§ TLB1: Deliveries of goods subject to value added tax; 
§ TPS1: Provision of services subject to value added tax;
§ TNT1: Deliveries of goods and services not subject to value added tax in France, including intra-Community 
distance sales referred to in 1° of I of Article 258 A and Article 259 B of the General Tax Code;
§ TMA1: Transactions subject to the regimes provided for in e) of 1 of Article 266 and Articles 268 and 297 A 
of the General Tax Code (VAT margin scheme).
ü TT-82: Daily cumulative amount without VAT in the currency specified in TT-78
ü TT-83: VAT amount of the daily total in euros
ü TT-84: number of transactions, i.e., sales for the category designated in TT-81. Consequently, when a sale is 
both a sale of goods and a provision of services, it counts as one transaction in each of the two categories
ü TG-32: provides the VAT details for this daily total, by applicable VAT rate
§ TT-86: VAT rate
§ TT-87: Base without VAT, in the currency in TT-78
§ TT-88: VAT amount in EUROS
• Transmission of an e-report containing cumulative payment data (10.4) recorded by the cash register software for 
transactions falling within the category of services (TPS1) only.
Point of attention no. 1: A distinction must be made between 
• invoices containing lines for the delivery of goods and lines for the provision of services, the former being ancillary 
to the latter, which makes them either invoices for the delivery of goods or for the provision of services, 
BUYER (not subject to VAT)
SELLER
PA-E
CdD
PPF
Receipt of invoicing data 
or transaction data (Flow 1, 10.1, 10.3)
and statuses "Submitted", "Rejected", "Refused"
2
Payment of the transaction for a purchase of goods 
and services INDEPENDENT Payment receipt for the transaction
Receipt of the ticket Printing of a receipt and recording of the transaction in 
cash register software/system
1
E-reporting of B2C transaction sales for the sale of 
services (Flow 10.3): 
"Delivery of goods" category
"Provision of Services" category
2
E-reporting of transaction receipts for services (Flow 
10.4)
6
Receipt of "Payment received" status 
E-reporting of transaction payments (Flow 10.4) 
7

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page116 / 149
• invoices containing lines for the delivery of goods and lines for INDEPENDENT services are referred to as "dual" 
invoices and are subject to a "Cadre de Facturation" (BT-23) beginning with an "M." 
Although a category of dual transactions is included among the mandatory data to be transmitted to the administration to 
take into account operators' practices (Billing Framework “Cadre de Facturation” (BT-23) beginning with "M"), it is 
recommended, insofar as possible, to issue separate invoices for deliveries of goods and services, given the different rules 
on when they become chargeable, in order to identify which transactions the e-reporting of payments relates to.
Point of attention no. 2: Opting for debits does not exempt you from distinguishing between transactions for the delivery 
of goods and transactions for the provision of services. However, it does mean that you do not have to submit e-payment
reports.
Point to note #3: This case should also be distinguished from the sale of "packages" comprising items subject to a separate 
VAT regime. For example, a toy book consists of a book subject to 10% VAT and a toy subject to 20% VAT. For the current 
EN16931 and EXTENDED-CTC-FR profiles, two separate invoice lines will be required in this case.
3.2.31 Case No. 32: Monthly payments
This use case is still under discussion and is therefore subject to further additions.
This management case illustrates the procedures for transmitting payment data relating to monthly payments made before 
an invoice is issued.
Firstly, in B2B, if these are advance payments, they must be invoiced as advance payments, with VAT and e-reporting of 
payment if they relate to services (flow 6). The final invoice can include the advance payments as negative lines (see cases 
20 and 21) to show the balance and retain any residual VAT.
If they are consumption estimates, they must also be invoiced commercially, resulting in recognition as revenue for the 
SELLER and as expenses for the BUYER.
In the case of B2C sales, or in the absence of invoices, these monthly payments must be declared via an e-reporting
transaction (corresponding to a delivery of goods or a service subject to VAT) supplemented by an e-reporting payment if 
required. 
When issuing the invoice:
• In the case of a B2C invoice, only the balance will be declared in the e-reporting of transactions (10.3, 10.4). It is 
necessary to determine how to handle a negative balance.
• If it is a B2B invoice, which would mean that the BUYER would then have reported themselves as VAT-registered to 
the SELLER, who would have been unaware of this until now, it is necessary to:
ü Cancel the e-reporting carried out for the various monthly payments by deducting them from one of the daily 
records of flow 10.3 and, if necessary, flow 10.4, preferably on the invoice date.
ü The amount already received should be entered in BT-113 of the invoice so that the net amount payable (BT115) corresponds to the balance due, even if it is negative.
ü The invoice should be sent as any other invoice and will result in a flow 1 containing the full amount of VAT, of 
which the portion already received will be offset by the e-reporting on the above transaction.
ü In the case of an international B2B invoice, flow 10.1 must be produced in the same way, with the same 
consequences.
Example: Illustration of a B2C use case
An energy supplier offers its residential customers the option of paying monthly based on an estimate of their annual 
consumption. At the end of the year, the energy supplier issues an adjustment invoice, the amount of which is calculated 
based on the actual consumption of each of its customers. 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page117 / 149
Case 1a: the SELLER has not opted for option on debits and the adjustment invoice shows an outstanding balance:
Figure41: Monthly payments and additional amount payable in a B2C context, no direct debit option (option 1a)
The specific features of invoicing/transaction and payment data transmission are:
• Transmission of an e-reporting of cumulative data for the transaction period (flow 10.3) and payments (flow 10.4) 
relating to monthly payments received, if applicable;
• Issuance of an adjustment invoice with the additional amounts to be paid to the customer, the total amount 
without VAT and VAT, and a reminder of the monthly payments already made. The adjustment invoice may be an 
invoice with negative amounts;
• At the same time, transmission of a net e-report equal to the amount of the adjustment invoice minus the recovery 
of advance payments;
• Transmission of payment data (flow 10.4) including the amount received corresponding to the net amount of the 
adjustment invoice.
SELLER
PA-E
BUYER (not subject to VAT)
CdD
PPF
Establishment of a monthly payment schedule Receipt of the monthly payment schedule
Payment of monthly installments M1 to M11
Creation and transmission of the adjustment invoice Receipt of the adjustment invoice
Collection of the adjustment invoice Payment of the adjustment invoice 
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7 Receipt of invoicing data 
or transaction data (flows 1, 10.1, 10.3)
and "Submitted", "Rejected", "Refused" statuses
2
6a
E-reporting of B2C transaction sales for service sales 
(Flow 10.3): 
"Services" category
2
E-reporting of transaction receipts for Services (Flow 
10.4)
6b
E-reporting of B2C transaction sales (Flow 10.3): 
"Services" category
2
E-reporting of transaction receipts for Services (Flow 
10.4)
6b
Payment receipt of monthly payments M1 to M11 6a

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page118 / 149
Case 1b: The SELLER has opted for debits and the adjustment invoice shows a balance due.
Figure42: Monthly payments in the context of a B2C sale, option for debits and additional amount to be paid (case 1b)
The specific features of the transmission of invoicing/transaction and payment data are:
• Transmission of an e-report containing cumulative data for the transaction period (flow 10.3) relating to monthly 
payments received, where applicable;
• Issuance of an adjustment invoice with the additional amounts to be paid to the customer, the total amount 
without VAT and VAT, and a reminder of the monthly payments already made. The adjustment invoice may be an 
invoice with negative amounts;
• At the same time, transmission of a net e-report equal to the amount of the adjustment invoice minus the recovery 
of advance payments.
The ruling published on June 5, 2025 (BOI-RES-RVA-000209-20250605) specifies that monthly payments received from 
individuals (B2C) must be considered as advance payments and that, pursuant to the first paragraph of 2.a of Article 269 of 
the CGI, VAT becomes chargeable upon receipt of each advance payment, up to the amount received, and upon completion 
of the consumption period covered by the invoice, up to the amount adjusted, in the amount received, and at the end of 
the consumption period covered by the invoice, in the amount adjusted.
The draft finance bill for 2026 provides that "I. – Data relating to the payment of transactions referred to in Articles 289 bis 
and 290 for which tax is payable upon receipt pursuant to Article 269(2) and Article 298 bis(I)(2), with the exception of those
for which tax is payable by the buyer, shall be communicated to the administration in electronic form, in accordance with 
the transmission standards defined by order of the Minister responsible for the budget, by the Accredited Platform chosen 
by the taxable entity.
It is therefore up to each company to determine, based on its situation, whether or not its transaction is subject to VAT on 
receipt and to draw the appropriate conclusions regarding the transmission of a 10.4 flow, including in the case of an option 
on debits.
SELLER
PA-E
BUYER (not subject to VAT)
CdD
PPF
Establishment of a monthly payment schedule Receipt of the monthly payment schedule
Payment of monthly installments M1 to M11
Creation and transmission of the adjustment invoice Receipt of the adjustment invoice
Collection of the adjustment invoice Payment of the adjustment invoice 
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7 Receipt of invoicing data 
or transaction data (flows 1, 10.1, 10.3)
and "Submitted", "Rejected", "Refused" statuses
2
6a
E-reporting of B2C transaction sales for service sales 
(Flow 10.3): 
"Services" category
2
E-reporting of B2C transaction sales (Flow 10.3): 
"Services" category
2
Option on debits
Payment receipt of monthly payments M1 to M11 6a

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page119 / 149
Case 2a: The SELLER has not opted for debits AND the adjustment invoice shows an overpayment:
Figure43: Monthly payments and final overpayment in a B2C transaction, no option for debits (option 2a)
The specific features of the transmission of invoicing/transaction and payment data are:
• Transmission of an e-reporting of cumulative transaction data (flow 10.3) and payments (flow 10.4) relating to 
monthly payments received;
• Issuance of an adjustment invoice to the customer indicating actual consumption and the total amounts without 
VAT and VAT. The adjustment invoice may be an invoice with negative amounts;
• At the same time, transmission of a negative net e-report equal to the amount of the adjustment invoice minus 
the recovery of advance payments;
• Transmission of payment data (flow 10.4) including the amount paid (i.e., negative) corresponding to the 
overpayment appearing on the adjustment invoice.
Case 2b: The SELLER has opted for debits AND the adjustment invoice shows an overpayment:
If the supplier has opted for VAT payment on debits: the adjustment invoice showing the overpayment as a negative 
amount will be taken into account. No payment data will be transmitted despite the overpayment. 
The specific features of the transmission of invoicing/transaction data are:
• Transmission of an e-reporting of cumulative transaction data (flow 10.3) relating to monthly payments received;
• Issuance of an adjustment invoice to the customer. The adjustment invoice may be an invoice with negative 
amounts.
SELLER
PA-E
BUYER (not subject to VAT)
CdD
PPF
Establishment of a monthly payment schedule Receipt of the monthly payment schedule
Payment of monthly installments M1 to M11
Creation and transmission of the adjustment invoice Receipt of the adjustment invoice
Payment of the adjustment invoice Payment receipt of the adjustment invoice 
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7 Receipt of invoicing data 
or transaction data (flows 1, 10.1, 10.3)
and "Submitted", "Rejected", "Refused" statuses
2
6a
E-reporting of B2C transaction sales for service sales 
(Flow 10.3): 
"Services" category
2
E-reporting of transaction receipts for Services (Flow 
10.4)
6b
E-reporting of B2C transaction sales (Flow 10.3): 
"Services" category
NEGATIVE AMOUNT
2
E-reporting of transaction receipts for Services (Flow 
10.4)
NEGATIVE AMOUNT
6b
Payment receipt of monthly payments M1 to M11 6a

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page120 / 149
• At the same time, transmission of a negative net e-report equal to the amount of the adjustment invoice minus the 
recovery of advance payments;
Once the supplier has opted to pay VAT on debits, they do not have to transmit payment data, unlike in case 2a.
Figure44: Monthly payments and final overpayment in a B2C transaction, option for debits (case 2b)
The ruling published on June 5, 2025 (BOI-RES-RVA-000209-20250605) specifies that monthly payments received from 
individuals (B2C) must be considered as advance payments and that, pursuant to the first paragraph of 2.a of Article 269 of 
the CGI, VAT becomes payable upon receipt of each advance payment, in the amount received, and at the end of the 
consumption period covered by the invoice, in the amount adjusted.
The draft finance bill for 2026 provides that "I. – Data relating to the payment of transactions referred to in Articles 289 bis 
and 290 for which tax is payable upon receipt pursuant to Article 269(2) and Article 298 bis(I)(2), with the exception of those
for which tax is payable by the buyer, shall be communicated to the administration in electronic form, in accordance with 
the transmission standards defined by order of the Minister responsible for the budget, by the Accredited Platform chosen 
by the taxable entity.
It is therefore up to each company to determine, based on its situation, whether or not its transaction is subject to VAT on 
receipt of payment and to draw the appropriate conclusions regarding the transmission of a 10.4 flow, including in the case 
of an option on debits.
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process minimum base formats and profiles (invoices and CDAR), and e-reporting (flow 10).
3.2.32 Case No. 33: Transactions subject to the margin scheme -profit
VAT on the margin is not calculated on the sale price, but on the difference between the sale price and the purchase price. 
The amount of VAT on the margin is not shown on the invoice, which poses a difficulty in e-invoicing.
SELLER
PA-E
BUYER (not subject to VAT)
CdD
PPF
Establishment of a monthly payment schedule Receipt of the monthly payment schedule
Payment of monthly installments M1 to M11
Receipt of the adjustment invoice Creation and transmission of the adjustment invoice
NEGATIVE BALANCE 
Payment of the adjustment invoice Payment receipt of the adjustment invoice 
Receipt of "Payment received" status 
E-reporting of payment on transaction (Flow 10.4) 
7 Receipt of invoicing data 
or transaction data (flows 1, 10.1, 10.3)
and "Submitted", "Rejected", "Refused" statuses
2
6a
E-reporting of B2C transaction sales for service sales 
(Flow 10.3): 
"Services" category
2
E-reporting of B2C transaction sales (Flow 10.3): 
"Services" category
Negative amount
2
Option on debits
Payment receipt of monthly payments M1 to M11 6a

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page121 / 149
The VAT margin scheme applies to transactions referred to in Article 266(1)(e) [travel agencies and tour operators] and 
Articles 268 [building land] and 297 A [second-hand goods, works of art, collectors' items or antiques] of the General Tax 
Code.
Example 1: A travel agency invoices a taxable entity for a single service for the organization of a seminar (flight – hotels –
rooms). 
Figure45: Transactions subject to the margin scheme, B2B invoice subject to e-invoicing
The specific data and management rules applicable to e-invoicing are as follows:
• For lines subject to the margin scheme (as there may be sales subject to the margin scheme and others not subject 
to the margin scheme): 
ü all amounts (BT-131) and unit prices (BT-146, BT-147, BT-148) are inclusive of tax
ü the VAT Type Code (BT-151) is "E"
• For VAT breakdown data (BG-23) subject to the margin scheme
ü VAT Type Code (BT 118): "E".
ü VAT exemption reason code (BT-121): 
§ VATEX-FR-F: second-hand sale
§ VATEX-FR-I: sale of works of art
§ VATEX-FR-J: sale of antiques
§ VATEX-EU-D: Travel agency sales
ü Base without VAT (BT-116): the sum of the line amounts (in practice, the total with VAT).
ü VAT rate (BT-119): equal to 0.
ü VAT amount in VAT breakdown (BT-117): 0
SELLER
PA-E
BUYER
PA-R
CdD
PPF
Receipt of invoicing data (Flow 1) 
and statuses "Submitted", "Rejected", "Refused"
2
Receipt of "Payment received" status 7
"Payment received" status
6b
Receipt of invoice statuses
4c
Processingstatuses
4b
Receipt of "Payment received" status 
6c
Creation of the invoice including VAT on the margin
1
Payment receipt and reconciliation
6a
Purchase and payment of a trip/service
Payment of invoice
5a
Transmission of Flow 1, invoice 
and corresponding status
2
Payment receipt of the sale
Receipt of invoice
3
Processing of the invoice
4a

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page122 / 149
• For total data:
ü Total without VAT (BT-109): the sum of the line amounts (BT-131), document-level charges, minus documentlevel allowances. In practice, this is also a total with VAT for lines subject to margin and a total without VAT for 
the other lines.
ü Total VAT (BT-110): 0, if there are only lines subject to margin.
ü Total with VAT (BT-112): equal to BT-109 if there are only margin lines.
There are no specific requirements for the transmission of invoicing and payment data, namely:
• Transmission of the invoice (flow 2), with flow 1 transmitted to the CdD PPF;
• Transmission of the "Paid" status of the invoice life cycle (flow 6), if applicable.
However, this means that flow 1 will not contain the VAT corresponding to the lines relating to the margin. The pre-filled 
VAT will therefore be incorrect because it will not be able to calculate the VAT on the margin. The SELLER therefore declares
the VAT on the margin in their VAT return (CA3 / CA12) and notes a discrepancy with the pre-filled VAT.
Example 2: A travel agency invoices a flight + hotel to an individual.
In this case, the SELLER must submit an e-reporting transaction (flow 10.3) and an e-reporting payment (flow 10.4), unless 
they have opted for debits. They are then required to e-report the pre-tax base of the margin and the VAT actually due:
• TT-77: Date of the cumulative amount subject to e-reporting (10.3).
• TT-78: Currency code in which all amounts in the record (TG-31) will be expressed, except for the VAT amount (TT83, which MUST be in EUROS. For activities in France, this currency is generally the EURO.
• TT-80: VAT payment option (if option for debits for sales of services).
• TT-81 (Transaction category): TMA1: Transactions subject to the rules set out in Article 266(1)(e) and Articles 268 
and 297 A of the French General Tax Code (VAT margin scheme).
• TT-82: Amount of the pre-tax margin base on which VAT is calculated (sale price – purchase price), daily total in the 
currency specified in TT-78. If the SELLER is unable to determine its margin in real time, an amount corresponding 
to an average margin rate estimated by the company is accepted.
• TT-83: VAT amount of the daily cumulative margin in euros.
• TG-32: provides VAT details for this daily total, by applicable VAT rate.
ü TT-86: VAT rate.
ü TT-87: Pre-tax basis of the daily cumulative profit margin to which the VAT rate in TT-86 applies. If the SELLER is 
unable to determine its margin in real time, this is the portion of the estimated average margin subject to the 
VAT rate indicated in TT-86.
ü TT-88: VAT amount in euros.
PLEASE NOTE: the number of transactions per category is no longer required. However, the TT-84 data remains in flow 10.3, 
as an option for companies that wish to provide it.
PLEASE NOTE: unlike e-invoicing, e-reporting requires the transmission of the profit margin and the VAT actually collected. 
VAT pre-filling can therefore be carried out. The difficulty lies in the SELLER's ability to know their profit margin per sale or 
per day (and per applicable VAT rate). If the operator does not know their margin per sale per day, they can use an average 
margin rate. In this case, the VAT pre-filling will not be accurate. The SELLER will have to file their VAT return with the exact 
profit margin calculation. If the SELLER issues an electronic invoice, it follows the same rules as example 1 above.
PLEASE NOTE: It may be necessary or desirable to indicate VAT on the margin in certain cases, such as for transactions 
referred to in Article 266(1)(e) [travel agencies and tour operators] and Article 268 [building land]. This point is under review, 
given that EN16931 currently applies VAT on the basis of the pre-tax amount, which is the sum of the pre-tax amounts of 
lines in the same category and VAT rate. 
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page123 / 149
SELLER obligations:
Ø Determine their profit margin on sales covered by e-invoicing for their VAT return.
Ø Have the daily profit margin for its B2C sales, if necessary by applying an average margin rate.
3.2.33 Case No. 34: Partial payment receipt and cancellation of payment receipt
Each partial payment (in the case of an advance payment, for example) must be declared with a life cycle flow bearing the 
status "Payment Received." The "amount" field will show the amount paid. 
In the event of a payment cancellation following a reconciliation error or fraudulent payment (misappropriation, theft, 
hacking, etc.), it will be possible to issue a life cycle with the status "Received" and a negative amount (amount field).
In the event of a payment receipt that cannot be reconciled with its invoice before the e-reporting date, a B2C payment 
receipt should be declared with the most likely VAT details (flow 10.4), then, once the reconciliation has been carried out, 
perform e-reporting of the negative B2C payment (flow 10.4 canceling the previous one) and e-reporting of the payment 
following reconciliation (flow 2 for invoices subject to e-invoicing, flow 10.2 for international B2B sales invoices).
3.2.34 Case No. 35: Author's notes 
This use case is still under discussion and is therefore subject to further developments.
OPERATIONS PERFORMED 
BY THE AUTHOR
PAYING INSTITUTION
APPLICABLE MECHANISM (EI / ER)
Receipt of copyright 
royalties
Publishers, copyright royalty 
receipt and distribution societies, 
or producers. 
Gives rise to VAT withholding by the payer *
E-reporting by the paying institution (no invoice but a 
statement of royalties payable to the author)
Receipt of copyright 
royalties
Other than publishers, copyright 
royalty receipt and distribution 
societies, or producers. 
Does not give rise to VAT withholding by the payer 
E-invoicing by the author (invoice issued by the author) if 
the customer is B or G 
E-reporting if the customer is C (e.g., non-taxable 
association). 
Other transactions not 
subject to withholding -
E-invoicing by the author (invoice issued by the author) if 
customer B/G
E-reporting by the author if customer C.
(*) Amount declared on the VAT return of the establishment that withheld VAT.
The specific features are as follows:
• Statements of duties are not covered by e-invoicing. 
• Transactions giving rise to VAT withholding by publishers, rights payment and distribution companies, and 
producers fall within the scope of e-reporting for these companies. There is therefore no need to create a dedicated 
"Cadre de Facturation".
• For copyrights that are not paid by these organizations, authors are liable for VAT, unless they benefit from the 
basic exemption scheme, and are subject, like any taxable entity, to e-invoicing/e-reporting obligations depending 
on whether or not their customer is a taxable entity.
• For transactions other than copyright, authors remain subject to the common law regime and may fall within the 
scope of e-invoicing or e-reporting. 
The case of author's notes is presented in order to clarify the application of electronic invoicing and e-reporting to this use 
case. No additional obligations beyond the provisions of Article 285 bis of the CGI are envisaged.
3.2.35 Case No. 36: Transactions subject to professional secrecy and exchanges of sensitive data
In order to comply with transactions subject to professional secrecy (see Article 226-13 of the Penal Code - in particular 
banking secrecy Article L. 511-33 of the Monetary and Financial Code or business secrecy Article L. 151-1 of the Commercial 
Code) as well as sensitive data covered by Ministerial Instruction No. 900/ARM/CAB/NP of March 15, 2021, the operators 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page124 / 149
concerned may use a generic description of the precise name of the goods or services provided, which must be mentioned 
in the "item name" field (BT-153) of flows 2 and 1. 
However, in order to meet their obligations to their customers, they must specify the transaction carried out. This 
information may be transmitted via tag BT-154 (item description) in flow 2, thus allowing both parties to the transaction to 
have details of it. Only the parties mentioned on the invoice will have access to this field, which is only present in stream 2. 
There is also other line data that can be used to define the service, references, attributes, etc., which are not transmitted in 
stream 1 and therefore remain confidential between the SELLER and the BUYER.
Only field BT-153 (item name), which contains very general information, will be transmitted to the tax authorities.
NOTE: In general, special attention should be paid to personal data (GDPR) contained in an invoice, even if it is 
not transmitted to the tax authorities.
3.2.36 Case No. 37: SEP (Sociétés en Participation)
In order to carry out certain projects for an end buyer, several service providers may join forces in a joint response and thus 
form a Temporary Business Group (Groupement Momentané d’Entreprises: GME). The GME may decide to organize its 
relations between the members of the group through a Joint Venture.
Joint ventures, also known as "SEPs," are secret companies created for the purpose of carrying out a project. The members 
of the SEP are partners in the project. The SEP has no legal personality and exists only between the partners. 
The management of the SEP is entrusted to a MANAGER (“GÉRANT”), who is chosen by the partners from among themselves.
The BUYER of the project is not aware of the existence of SEP. They are invoiced by an AGENT or CONTRACTOR who is the 
representative of the members of the Temporary Business Group (and therefore of SEP) to the BUYER for the invoicing of 
the GME. The AGENT / CONTRACTOR is one of the partners of SEP and a member of the GME. This AGENT / CONTRACTOR 
invoices in his own name, but on behalf of the SEP.
This means that the AGENT / CONTRACTOR is identified as the SELLER in the sales invoices to the BUYER, but that the sales 
proceeds are in fact recorded in the SEP (by the MANAGER (“GÉRANT”) upon presentation of the sales invoices issued by 
the AGENT / CONTRACTOR in his own name and on behalf of the SEP) and not as a sale by the AGENT / CONTRACTOR to the 
BUYER.
For SEP purchases, from the suppliers' point of view, it is the Manager who acts as the BUYER. It is therefore the MANAGER 
who is invoiced by the SEP's suppliers (he or she appears as the BUYER on the invoice). 
In particular, the partners in the SEP invoice their share of the services to the SEP. They are therefore the SELLER on their
sales invoices and the BUYER is the MANAGER. Consequently, on the invoices for services provided by the MANAGER to the 
SEP, the MANAGER is identified as both the SELLER and the BUYER.
In terms of VAT, the MANAGER (“GÉRANT”) may decide either to include the VAT relating to the SEP in their own VAT 
returns, or to declare the VAT relating to the SEP's activity separately.
The reform will not change the way SEPs operate in terms of the accounting and tax treatment of SEP sales and purchase 
invoices, but the pre-filled information will not correspond to the VAT returns of the various parties involved, namely the 
CONTRACTOR, MANAGER, and SEP. 
Furthermore, if invoices are generated by management software linked to accounting software, the latter must be able 
to isolate invoices issued by the CONTRACTOR, Representative of the SEP to the END BUYER. Similarly, the solutions used 
to manage purchase invoices received by the MANAGER/representative of the SEP for its purchases from the SEP's 
suppliers must be able to isolate these purchase invoices on behalf of the SEP from purchase invoices on behalf of the 
MANAGER itself.
Thus, sales invoices issued by the CONTRACTOR (or in its name if their creation and transmission is entrusted to a third party, 
for example the MANAGER), and if the CONTRACTOR and the BUYER are subject to VAT so that the invoice is subject to the 
electronic invoicing obligation, then the pre-filling for VAT collection will be attributed to the CONTRACTOR, who will also 
be required to transmit the payment receipt status if VAT is due upon receipt.
Purchase invoices are sent to the MANAGER. They are therefore subject to the electronic invoicing obligation, unless the 
supplier or the MANAGER are not subject to it or the service is not covered by the e-invoicing component of the reform. In 
the case of international purchases, it is therefore the MANAGER who must declare the corresponding 10.1 flows.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page125 / 149
All this leads to the pre-filling of the MANAGER's deductible VAT being increased by the deductible VAT on these purchases, 
even though it will appear in the SEP's VAT return (unless the MANAGER has opted for an integrated return).
That being said, it is necessary to see how exchanges can take place so that the MANAGER has access to invoices issued 
or received on behalf of the SEP.
With regard to SEP sales invoices, the CONTRACTOR may choose an Accredited Platform that will provide the MANAGER 
with access to invoices issued on behalf of the SEP. To maintain the anonymity of the SEP while restricting the MANAGER's 
access to only those invoices issued by the AGENT on behalf of the SEP, a project reference can be used and included in the 
invoice (for example, BT-11). 
It is also possible, and preferable, for the CONTRACTOR to choose to entrust the creation of invoices to the MANAGER, 
who can then use the Accredited Platform of their choice (PA-TE) and manage the life cycle statuses, including the payment 
receipt status, as the MANAGER is often best placed to track the payment receipt of sales invoices (often paid into a bank 
account dedicated to the SEP). In practice, this still means that the Accredited Platform thus chosen is concerned with the 
existence of an invoicing mandate between the CONTRACTOR and the MANAGER, which amounts to identifying the 
CONTRACTOR as a user of the PA-E Accredited Platform, granting delegation rights to the MANAGER so that they can act on 
behalf of the CONTRACTOR (See third-party management, section 3.2.1.2).
The MANAGER must nevertheless inform the CONTRACTOR of the issuance of invoices and their payment receipt to enable 
the latter to better measure the differences between their pre-filled VAT return and their VAT declaration. He or she may 
also ask the Approved Issuing Platform (PA-TE) for SEP sales invoices to open access for the AGENT or to send them the 
invoices issued in their name as well as the related life cycle statuses.
As the SEP's purchase invoices must be sent to the MANAGER acting as the BUYER, the BUYER's electronic invoicing address 
(BT-49) on invoices must be of the type SIRENGérant_XXX. In order to clearly distinguish these invoices from the MANAGER's 
own purchase invoices, it is recommended that the MANAGER create a dedicated electronic invoicing address, either for 
each SEP it manages or globally for its SEP management activity. They must then ask suppliers to include a reference on 
invoices (either a purchase order number (BT-13), a project number (BT-11) or a buyer reference (BT-10), etc.) in order to 
assign invoices to the correct SEP. The electronic invoicing address could therefore be, for example, 
SIRENGérant_gestiontiers.
Figure46: Management of sales and purchase invoices on behalf of an SEP (joint venture)
For status management:
• If the creation of invoices from the CONTRACTOR to the BUYER has been entrusted to the MANAGER (who is 
therefore a third-party invoicer), the latter also manages the sales invoice lifecycle, which may involve cancelling 
the invoice, creating corrected invoices or credit notes, and setting the status to “Payment Received” if necessary.
SELLER
BUYER
MANAGER SEP
SIRENGérant_gestiontiers
PA-TR 
(Manager)
PA-TE (Manager)
On behalf of the 
Contractor
CONTRACTOR
PA-R (Buyer)
PA-E (seller)
SEP
(Société En Participation)
From the Buyer's perspective, 
the Seller is the CONTRACTOR.
Sales Invoice to the BUYER.
The SELLER is the CONTRACTOR.
The sales invoice is sent via a 
PA-TE to which the Manager has 
access.
The Manager provides an 
electronic invoicing address 
for SEP purchases.
The Supplier shall send its invoices to 
the MANAGER at its dedicated address 
SIRENGérant_gestiontiers
The MANAGER records the purchases and sales of the SEP, as well as VAT, 
based on the sales invoices of the CONTRACTOR and the purchase invoices of 
the MANAGER relating to the SEP's activity
From the Supplier’s perspective
the Buyer is the MANAGER

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page126 / 149
• If the CONTRACTOR retains control of sales invoices on behalf of the GME (and the SEP), he must forward them to 
the MANAGER and keep him informed of the life cycles. The CONTRACTOR may also arrange for the MANAGER to 
access the PA-E chosen by the CONTRACTOR to enable the MANAGER to track these invoices and statuses (or send 
them to the MANAGER via CDAR messages, as described in section 3.2.1.2).
• The MANAGER manages the life cycle of purchase invoices with suppliers.
• Suppliers and the Buyer manage their invoices as normal, without any impact from the fact that the underlying 
transactions are carried out by an SEP.
In terms of VAT, the situation is therefore as follows:
• If the CONTRACTOR is different from the MANAGER or if they are also the MANAGER and have not chosen to include 
the SEP's VAT in their own VAT return, then the CONTRACTOR 's pre-filled VAT return is incorrect in the amount of 
VAT collected on sales invoices that were issued in their name but on behalf of the SEP.
• If the MANAGER has not chosen to include the SEP's VAT in their own VAT return, the MANAGER's pre-filled VAT 
return is incorrect in respect of the deductible VAT on purchase invoices relating to the SEP's activity.
• If the MANAGER is also the CONTRACTOR and has opted to include the SEP's VAT return in their own, the pre-filled 
discrepancies merge and offset each other. This is because the sales invoices are then issued by the MANAGER on 
behalf of the SEP, but the SEP's VAT is included in that of the MANAGER. Similarly, the SEP's deductible VAT is 
allocated to the Manager for VAT pre-filling, which corresponds to the Manager's return, which includes the SEP's 
VAT in their own.
Finally, if the MANAGER invoices the SEP (as each partner does), this invoice will have the Manager as the SELLER and the 
MANAGER as the BUYER. It will be sent by the MANAGER's PA-E to the PA-R that the MANAGER has chosen to receive 
invoices dedicated to its SEP management. There is nothing to prevent this from working.
Finally, as SEPs can have a SIREN number and be subject to VAT themselves, they may be listed in the PPF Directory 
(«Annuaire»), which is public, as they are listed in the SIRENE database, which is also public. It is therefore important not to 
activate an electronic invoicing address for this SEP and to avoid any publicity about its existence, so that it remains hidden 
from the general public (and is only known to those who know its name or SIREN number).
Obligations of (Accredited Platforms («Plateformes Agréées»)):
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
Optional features of the MANAGER's PA-TE:
Ø Allow shared access to the CONTRACTOR on the PA-E to access invoices issued in their name on behalf of the SEP 
and the related life cycle statuses.
3.2.37 Case No. 38: Invoices with sub-lines and line groupings
In certain sectors, composite items require a detailed description of their composition. Examples include:
• The sale of kits that combine various items, such as a tool pack, where it is necessary to specify which tools are 
included in the pack. The price may be the result of the unit prices and quantities of each item in the pack, or a 
total price, taking into account any commercial allowance, where applicable.
• The sale of items composed of several sub-items to which different VAT rates apply, such as a toy book for which 
the book is subject to 10% VAT and the toy to 20% VAT.
Certain invoicing practices may also lead to subtotals being expressed, for example by order in a multi-order invoice, by 
delivery in a multi-delivery invoice, etc.
This can potentially lead to duplicate amount information being displayed: once in a detail or information line, and once in 
a group or subtotal line.
To allow these invoices to continue to exist, as they are the result of widespread practices that are considered useful and 
necessary by the companies that use them, data and management rules have been added to the EXTENDED-CTC-FR profile, 
described in Standard XP Z12-012.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page127 / 149
The usage is as follows:
• In order to organize a structure between lines, i.e., to indicate that certain lines depend on and are linked to another 
line, a "Parent Line ID" data field has been added, which must therefore indicate a line number (BT-126) that exists 
in the invoice.
• Then, to avoid counting an invoiced amount more than once, lines can be qualified using a "line subtype" field, 
which can take the following three values:
ü DETAIL: means that this line must be taken into account in the calculation of totals and VAT breakdowns
ü INFORMATION: means that the line is present for information purposes, as a detail of the line to which it is 
attached. For example, a tool pack has a main line with the price and the line amount without VAT (BT-131) 
taken into account in the calculation of totals, and "INFORMATION" lines are used solely to detail what is 
included in the pack.
ü GROUP: means that the line is a grouping, a subtotal, which should not be taken into account in the calculation 
of totals and VAT. This applies to the "sub-lines" attached to it, which are therefore classified as "DETAIL."
• "Standard" lines, without a "Parent line identifier" or "line subtype," may also be present in an invoice with sublines.
GROUP and INFORMATION lines are not subject to the requirement to include mandatory line information (unit price, 
quantity, VAT data, line amount without VAT). They are also not transmitted in flows 1 or 10.1.
However, a "GROUP" line that has the current line amount without VAT (BT-131) MUST then have this amount equal to the 
sum of the "line amounts without VAT (BT-131)" of the lines directly linked to it of type DETAIL or GROUP (so that the 
presentation of the subtotals remains accurate). As a result, when a GROUP line has a line total without VAT (BT-131), all 
GROUP lines directly or indirectly linked to it (at several levels below) MUST also have a line amount without VAT (BT-131).
When sub-lines are used to describe the details of a composite item, it may be useful to enter the quantities of each subitem (in each sub-line) for ONE UNIT of the composite item. This can be done using data EXT-FR-FE-191.
All of this is specified in the management rules for the EXTENDED-CTC-FR profile.
Example of use 1: Use the "INFORMATION" lines to complete the item description: The sale of 2 "Toolbox" kits, each 
containing 3 pliers, 5 hammers, and 1 screwdriver (for a total of 6 pliers, 10 hammers, and 2 screwdrivers). The price is set
at the KIT level, with the "INFORMATION" lines providing details. The lines in blue are grouped together. The line 1 could 
also have been classified as "DETAIL." Line 2 is an independent additional information line. Line 3 is a standard line.
Only the required data from lines 1 and 3 will be transmitted in stream 1 or 10.1.
Figure47: Invoice for a kit with details of its composition using INFORMATION sub-lines
Lines
LineID Parent LineID LineStatus 
ReasonCode Name Invoiced Quantity
Per Package 
Qty
Unit of measure PU Net
VAT 
Category 
code
VAT rate Invoice line 
net amount
BT-126
BT-X-304
EXT-FR-FE-162
BT-X-8
EXT-FR-FE-163 BT-153 BT-129
BT-X-561
EXT-FR-FE-191 BT-130 BT-146 BT-151 BT-152 BT-131
1 Toolbox 2 C62 (unit) 199,00 S 19% 398,00
1.1 1 INFORMATION Pliers 6 3 C62 (unit) 0,00
1.2 1 INFORMATION Hammer 10 5 C62 (unit) 0,00
1.3 1 INFORMATION Screwdriver 2 1 C62 (unit) 0,00
2 INFORMATION Free bag 1 C62 (unit) 0,00
3 Nails 500 C62 (unit) 0,02 S 19% 10,00
VAT Breakdown
Basis 
Amount Category Code VAT rate VAT Amount Totals
BT-115 BT-118 BT-119 BT-117 408,00
408,00 S 19% 77,52 77,52
485,52
Total without VAT (BT-109)
Total VAT (BT-110)
Total with VAT (BT-112)

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page128 / 149
Example of use 2: composite items with multiple VAT rates: Toy book. The totals and VAT are calculated on the DETAIL lines 
(50 and 75). The GROUP line does not provide VAT information as it would be meaningless. It is not transmitted in flow 1 or 
10.1.
Figure48: Invoice for a toy book where the price is determined on the sub-items
Example of use 3: sub-lines to link them to a main line: a transport service, with a main line, which may contain several data 
items and references (here, item invoiced for a parcel number, but there may also be the pick-up address, delivery address, 
customer references, etc.) that do not need to be repeated in each sub-line for additional services (various extras).
Figure49: Invoice with a main line and supplements attached to it
NOTE: the line number does not need to replicate the structure (1.1, 1.2). The Parent line identifier is sufficient for this 
purpose.
Lines
LineID Parent LineID LineStatus 
ReasonCode Name Invoiced Quantity
Per Package 
Qty
Unit of measure PU Net
VAT 
Category 
code
VAT rate Invoice line net 
amount
BT-126
BT-X-304
EXT-FR-FE-162
BT-X-8
EXT-FR-FE-163 BT-153 BT-129
BT-X-561
EXT-FR-FE-191 BT-130 BT-146 BT-151 BT-152 BT-131
1 GROUP Book-toy 5 C62 (unit) 25,00 125,00
2 1 DETAIL Book-toy 5 1 C62 (unit) 10,00 S 10% 50,00
3 1 DETAIL Toy 5 1 C62 (unit) 15,00 S 20% 75,00
VAT Breakdown
Basis 
Amount Category Code VAT rate VAT Amount Totals
BT-115 BT-118 BT-119 BT-117 125,00
75,00 S 20% 15,00 20,00
50,00 S 10% 5,00 145,00
Total without VAT (BT-109)
Total VAT (BT-110)
Total with VAT (BT-112)
Lines
LineID Parent LineID LineStatus 
ReasonCode Name Invoiced Object
Invoiced 
Quantity
Unit of 
measure PU Net
VAT 
Category 
code
VAT rate
Line Total 
amount 
without VAT
BT-126 BT-X-304
EXT-FR-FE-162
BT-X-8
EXT-FR-FE-163 BT-153 BT-128 BT-129 BT-130 BT-146 BT-151 BT-152 BT-131
1 Delivery Package number 1 C62 (unit) 25,00 S 20% 25,00
2 1 DETAIL Diesel supplement 1 C62 (unit) 3,00 S 20% 3,00
3 1 DETAIL Week end supplement 1 C62 (unit) 5,00 S 20% 5,00
VAT Breakdown
Basis Amount Category Code VAT rate VAT Amount Totals
BT-115 BT-118 BT-119 BT-117 33,00
33,00 S 20% 6,60 6,60
39,60
Total without VAT (BT-109)
Total VAT (BT-110)
Total with VAT (BT-112)

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page129 / 149
Example of use 4: multiple levels of sub-lines: the sale of 2 displays, each consisting of 3 packs of Kenya Roast, 6 packs of 
Dark Roast, and 3 bundles consisting of 3 packs of Columbia Roast and 3 MUGs, with potentially different applicable VAT 
rates (for the example). This illustrates the fact that lines can be organized on several levels. Again, only DETAIL lines count 
in the calculation of totals and VAT breakdowns, and are transmitted in flows 1 and 10.1.
Figure50: Invoice for a composite item with 2 levels
This ability to construct an invoice with sub-lines and a multi-level sub-line hierarchy makes it possible to address many 
situations, including avoiding excessive repetition of line information. For example, in the case of multiple deliveries, it may 
be appropriate to indicate the delivery address only on a main line (without a "line subtype") or even on an INFORMATION 
line and then attach the details of the lines corresponding to that delivery.
This will also be central to the creation of multi-vendor invoices, see use case 39.
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
3.2.38 Case No. 39: Transparent intermediary consolidating sales from multiple Sellers for the same buyer – MultiVendor Invoice
In certain cases, an intermediary acts as an aggregator between SELLERS and a BUYER, as if it were an opaque intermediary 
(purchasing goods or services from Sellers and reselling them in bulk to a BUYER), but in fact as a simple transparent 
intermediary, i.e., acting on behalf of the SELLERS vis-à-vis the BUYER, or on behalf of the BUYER vis-à-vis the SELLERS (in 
this case as the BUYER's AGENT).
In many cases, a main SELLER also invoices for ancillary or complementary services to its own, which are provided and 
therefore sold by other SELLERS. These services are included in the invoice issued by the Main SELLER.
The most common example is the water supply bill, which includes sanitation services and services provided by certain 
public bodies. Regulations require that only one invoice be issued in such cases.
The current solution means that the water distributor invoices services provided by other SELLERS on its own invoice, then 
sends the accounting and VAT information to each SELLER for their accounting and VAT returns.
If nothing changes, the water distributor's pre-filled VAT return will be incorrect, as the third-party services will be allocated 
to it in terms of VAT collected. The same will apply to the other SELLERS.
Lines
LineID Parent LineID
LineStatus 
ReasonCode Name Invoiced Quantity Per Package Qty
Unit of 
measure PU Net
VAT 
Category 
code
VAT rate
Line Total 
amount without 
VAT
BT-126 BT-X-304
EXT-FR-FE-162
BT-X-8
EXT-FR-FE-163
BT-153 BT-129 BT-X-561
EXT-FR-FE-191
BT-130 BT-146 BT-151 BT-152 BT-131
1 GROUP Coffee display 2 C62 (unit) 216,00
1.1 1 DETAIL Kenya Roast 6 3 C62 (unit) 5,00 S 7% 30,00
1.2 1 DETAIL Dark Roast 12 6 C62 (unit) 5,00 S 7% 60,00
1.3 1 GROUP Colombia Bundle 6 3 C62 (unit) 126,00
1.3.1 1.3 DETAIL Colombia Roast 18 3 C62 (unit) 5,00 S 7% 90,00
1.3.2 1.3 DETAIL Mug 18 3 C62 (unit) 2,00 S 19% 36,00
VAT Breakdown
Basis 
Amount Category Code VAT rate VAT Amount Totals
BT-115 BT-118 BT-119 BT-117 216,00
36,00 S 19% 6,84 19,44
180,00 S 7% 12,60 235,44
Total without VAT (BT-109)
Total VAT (BT-110)
Total with VAT (BT-112)

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page130 / 149
There are many examples of this in practice when a third party intervenes in a commercial transaction, acting as a 
consolidator for a BUYER, for example:
• Leasing services (vehicles, various equipment), where insurance or maintenance services are added to leasing 
invoices even though they are provided and normally invoiced by third parties.
• Toll road subscriptions, which bill on behalf of several highway toll companies.
• Fuel purchase cards leading to a periodic multi-supplier invoice.
• Taxi booking companies that produce a periodic invoice with all the trips for a given month, which are in practice 
unit services provided by taxi companies to the end BUYER.
• Travel agencies that act as booking intermediaries, generally also acting as third-party payers to the various 
transport or other service providers, and invoice their services as well as reimbursement lines for individual travel 
services, indicating the analytical data required for processing by the BUYER.
There are two main solutions for addressing these use cases:
• Organize an exchange of individual invoices between each SELLER and the BUYER, to which the third party has 
access in order to act as the BUYER's AGENT intermediary, integrating expense lines into its own invoices for overall 
processing by the BUYER on an aggregated service.
• Enable the issuance of a multi-vendor invoice, producing as many 1 (or 10.1) flows as there are individual invoices 
per SELLER, and allowing the BUYER to process the invoice as a single invoice.
3.2.38.1 Individual unit invoices and "global" invoice from the third party transparent to the BUYER
In practice, this case is extremely similar to case no. 15 of a BUYER'S AGENT acting on behalf of a BUYER with multiple 
SELLERS and also intermediating the payment, as is the case with Media Agencies vis-à-vis Advertising Agencies and on 
behalf of Advertisers.
The resolution is therefore the same, namely:
• The third party agrees with the BUYER to create an electronic invoicing address dedicated to specific multi-seller 
purchases, which is specific to the BUYER and entrusted to an Accredited Platform capable of offering the third 
party the right to act on behalf of the Buyer with regard to invoices received at this address (e.g., 
SIRENacheteur_achatexterne).
• SELLERS are informed of this electronic invoicing address to be used with two variants (a mix of the two is possible, 
since each SELLER chooses what they want to do).
ü Either the SELLERS create and transmit their invoices through their PA-E, which are then received on the PA-TR 
of the THIRD PARTY (and the BUYER), to be processed by the third-party BUYER'S AGENT.
ü Or the THIRD PARTY also acts as a third-party INVOICER on behalf of the SELLER (under invoicing mandate) and 
therefore creates invoices on their behalf, for example because they have all the transactional information due 
to their role in the purchasing phase. These invoices are sent via an PA-E capable of offering the THIRD PARTY 
the option of acting on behalf of each SELLER.
ü Thus, flows 1 or 10.1 are transmitted for each invoice, allowing the pre-filled VAT to be consistent.
• The THIRD PARTY can therefore process the invoices addressed to the BUYER on its behalf and then invoice the 
BUYER with a "global" invoice consisting of:
ü The THIRD PARTY's own intermediation services, subject to VAT.
ü Expense lines for each of the services covered by the individual invoices, enabling payment for the individual 
services to be requested and, where applicable, additional analytical information to be provided using item 
attributes. This corresponds to requests for funds made in the media sector.
ü However, VAT information is not included in the expense lines and is therefore not present in the VAT 
breakdown (BG-23). This requires the BUYER to either process the individual invoices in parallel or to request 
that the analytical information also contain VAT information and references allowing the expense lines to be 
linked to the individual invoices (e.g., a voucher number for travel services).
Life cycle statuses are normally managed on an invoice basis. Processing statuses (Rejected, Disputed, Approved, Payment 
sent) are set by the THIRD PARTY or by the BUYER via PA-TR.
If the THIRD PARTY acts as an INVOICER on behalf of certain SELLERS, it must arrange to obtain payment information from 
the SELLER in order to set the payment status on its behalf (only if VAT is due upon payment).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page131 / 149
Figure51: Transparent third party acting as BUYER'S AGENT
3.2.38.2 Multi-Vendor Invoices
When a "main" SELLER handles the invoicing of third-party SELLERS for invoices addressed to the same BUYER as part of an 
integrated service, another solution is to allow the main SELLER to create a Multi-Vendor invoice in which it will group 
together the deliveries or services of other secondary SELLERS in its invoice. This way, the BUYER can have a single "standard" 
invoice that they can manage without worrying about the fact that there are several SELLERS. 
From the SELLERS' point of view, the multi-vendor invoice is equivalent to combining individual invoices with the same 
invoice date, the same BUYER, and most of the references and general information into a single "consolidated" invoice.
However, to allow different types of invoices to continue to exist (invoice, credit note, corrected invoice, advance invoice, 
etc.), there is no specific type code (BT-3) to identify these invoices, but a dedicated "Cadre de Facturation" "B8, S8, M8".
The use of this Multi-Vendor invoice is at the initiative of the SELLERS, but must not impose any specific processing on the 
BUYER and its Approved Receiving Platform. Consequently, it is not permitted to use it for self-billing (i.e., created by the 
BUYER on behalf of the SELLERS).
Consequently, the main principles for creating a multi-vendor invoice, which should be thought of as a consolidated invoice 
combining deliveries and services from several secondary SELLERS behind a main SELLER, who acts as a transparent 
intermediary for the secondary SELLERS, are as follows:
• Use a billing framework (BT-23) equal to B8 / S8 / M8 to indicate that it is a multi-vendor invoice
• Use sub-lines with the following for each unit invoice:
ü a "GROUP" line identifying:
§ the SELLER, 
§ a unit invoice number (coded in BT-128 with qualifier BT-128-1 = AFL),
§ a "Cadre de Facturation" (coded in BT-128 with qualifier BT-128-1 = AVV),
§ and the totals (equivalent to the Total HT (BT-109), the VAT amount (BT-110 and BT-111), and the total TTC 
(BT-112)),
ü invoice lines of type "DETAIL", with a reason for exemption in inline text (EXT-FR-FE-179) beginning with the 
unit invoice number between #, serving as a key for the breakdown of VAT per unit invoice. As added in the 
EXTENDED-CTC-FR profile and as will be included in the revision of Standard EN16931, the VAT breakdown detail 
lines will also include the reason for exemption in code and text (in addition to the VAT category and VAT rate).
SELLER
BUYER
THIRD PARTY
SIRENacheteur_achatexterne
PA-TE PA-R
PA-E
The THIRD PARTY submits its service invoice, 
with disbursement lines corresponding to the 
unit invoices, and, where applicable, attaches 
the unit invoices.
The THIRD PARTY has access to 
individual purchase invoices to assist 
the PURCHASER in processing them.
SELLERS shall send their invoices to the 
BUYER at its electronic invoicing 
address: SIRENacheteur_achatexterne
PA-TR
SELLER
PA-E

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page132 / 149
The Approved Issuing Platform (PA-E) MUST:
• Check the compliance of the invoice by applying the specific French management rules described in Standard XP 
Z12-012. In the event of rejection, a single rejection status for the main invoice is sent to the PPF (identified with 
the SIREN number of the main SELLER, invoice date (BT-2) and invoice number (BT-1)).
• Build unit invoices based on the multi-vendor invoice to extract flows 1 (and potentially 10.1 if the PA-E offers this 
option). If necessary, these invoices can be made available to each SELLER as accounting and tax supporting 
documents.
• For invoices subject to e-invoicing requirements, send the "Submitted" status for each unit invoice to the PPF CdD, 
along with the corresponding flow 1.
• Receive life cycle statuses from the BUYER and allow the Main SELLER to manage them on behalf of all SELLERS.
• Replicate the "Rejected" / "Refused" statuses received from the BUYER on the multi-vendor invoice to the 
secondary SELLERS' individual invoices and forward them to the PPF's CdD (because the BUYER's Accredited 
Platform only referenced the invoice number in BT-1, corresponding to the primary SELLER's individual invoice).
• Allow the main SELLER to send the "Paid" statuses corresponding to each unit invoice on behalf of the secondary 
SELLERS (bearing in mind that it is generally the main SELLER who is paid on behalf of all SELLERS and then 
redistributes the payments to each SELLER).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases Page133 /149
Figure52: Example of a multi-vendor invoice
Situation 3 Sellers
Seller : Seller A BT-1 F20250025
Invoicer : Seller A BT-2 01/10/2025
BT-3 380
Seller A issues an invoice with 3 groups of lines, per final Seller (including Seller A) BT-8 5
Multi-Seller Invoice BT-23 S8
N° ligne N° ligne Parent CODE Type ligne Name Item Description Unit Price Quantity SELLER NAME SELLER LEGAL 
ID SELLER VAT ID
SELLER 
COUNTRY 
Code
TAX POINT DATE 
(BT-8)
Line 
Amount VAT Code VAT Rate Code VATEX VATEX Text VAT total on line
Line Amount 
with VAT
BT-126 EXT-FR-FE-162 
(BT-X-304) EXT-FR-FE-163 (BT-X-8) BT-153 BT-154 BT-146 BT-129 EXT-FR-FE-164 EXT-FR-FE-167 EXT-FR-FE-168 EXT-FR-FE-177 BT-128 BT-128-1 EXT-FR-FE-180 BT-128 BT-128-1 BT-131 BT-151 BT-152 EXT-FR-FE-179 EXT-FR-FE-178 EXT-FR-FE-181 EXT-FR-FE-184
1 GROUP Total SELLER A SELLER A 123456782 FRxx123456782 FR F20250025 AFL 5 S1 AVV 2 500,00 350,00 2 850,00
2 1 DETAIL Service A balbla 1 000,00 1,00 123456782 F20250025 1 000,00 S 20,00% #F20250025#
3 1 DETAIL Service B balbla 500,00 3,00 123456782 F20250025 1 500,00 S 10,00% #F20250025#
4 GROUP Total SELLER X SELLER X 321654879 FRxx321654879 FR 321654879_F20250025 AFL 72 S1 AVV 5 500,00 1 100,00 6 600,00
5 4 DETAIL Service X balbla 300,00 5,00 321654879 321654879_F20250025 1 500,00 S 20,00% #321654879_F20250025#
6 4 DETAIL Service Z balbla 1 000,00 4,00 321654879 321654879_F20250025 4 000,00 S 20,00% #321654879_F20250025#
7 GROUP Total SELLER 00 SELLER 00 254136987 FRxx254136987 FR 254136987_F20250025 AFL 72 S1 AVV 850,00 170,00 1 020,00
8 7 DETAIL Service 25 balbla 12,00 50,00 254136987 254136987_F20250025 600,00 S 20,00% #254136987_F20250025#
9 7 DETAIL Service 32 balbla 25,00 10,00 254136987 254136987_F20250025 250,00 S 20,00% #254136987_F20250025#
Base VAT VATEX Code VATEX Text VAT Code VAT Rate VAT Amount
BT-116 BT-121 BT-120 BT-118 BT-119 BT-117
1 000,00 #F20250025# S 20,00% 200,00
1 500,00 #F20250025# S 10,00% 150,00 BT-109 Total Without VAT 8 850,00
5 500,00 #321654879_F20250025# S 20,00% 1 100,00 BT-110 Total VAT 1 620,00
0,00 #321654879_F20250025# S 10,00% 0,00 BT-112 Total With VAT 10 470,00
850,00 #254136987_F20250025# S 20,00% 170,00 BT-113 Amount Paid 0,00
0,00 #254136987_F20250025# S 10,00% 0,00
BT-115 Amount TO BE 
PAID 10 470,00
Invoice ID for Seller and DRR
Business process type 
(Cadre de facturation) 
for Seller and DRR
TOTALS
MULTI-SELLER INVOICE EXAMPLE
Business Process (Cadre de facturation) S8/B8/M8 means "Multi Seller 
Invoice". Business Process (cadre de facturation) of "sub invoices" is in BT128, SchemeID = AVV

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases Page134 /149
The table below illustrates how invoice information is implemented in the multi-vendor invoice and in unit invoices, based 
on the example above:
Invoice information Data in the invoice 
Multi-Vendor
Data in the single-vendor invoice 
(for flow 1 or 10.1)
Invoice number BT-1: F20250025 Only included for the main SELLER's unit 
invoice
Invoice date BT-2: 10/01/2025 BT-2: 10/01/2025
Invoice type: Invoice (380) BT-3: 380 BT-3: 380
Billing framework BT-23: S8 because the invoice only 
includes unit service invoices
This is the online "Cadre de Facturation"
that is used
VAT liability code BT-8: 5 (means "Option on Debits") The online code will be used, as it may 
differ from one SELLER to another
GROUP line for each SELLER. For the record, GROUP lines are not present in flow 1).
Line subtype EXT-FR-FE-163 = GROUP EXT-FR-FE-163 = GROUP 
Item name: BT-153: allows you to give a title to the 
block under the line per unit invoice
BT-153
Unit price, quantity, allowances, 
and charges online
No apparent interest in the GROUP line
Information about the SELLER in 
the line (company name, legal ID, 
VAT number, country code, etc.)
EXT-FR-FE-BG-13: SELLER online
EXT-FR-FE-164: Company name
EXT-FR-FE-167: Legal identifier (SIREN)
EXT-FR-FE-168: VAT number 
EXT-FR-FE-177: Country code
BG-4: SELLER 
BT-27: Company name
BT-30: Legal identifier (SIREN)
BT-48: SELLER VAT number
BT-40: SELLER country code
Specific information for unit 
invoices
BT-128 (BT-128-1 =AFL): F20250025
BT-128 (BT-128-1 = AVV): S1
EXT-FR-FE-180: 5 or 72
BT-1: Unit invoice number
BT-23: Billing framework
BT-8: VAT liability
Totals for each "unit invoice" BT-131 (Total line amount without VAT): 
2,500
EXT-FR-FE-181 (Total VAT): 350
EXT-FR-FE-182 (VAT EUR)
EXT-FR-FE-184 (Total with VAT per line)
BT-109: Total amount without VAT
BT-110: Total without VAT
BT-111: VAT in EUROS
BT-112: TTC (=BT-109 + BT-110)
DETAIL line: this is a standard invoicing line, but with certain specific data to be entered
Legal ID SELLER on the line EXT-FR-FE-167 EXT-FR-FE-167
Invoice number on the line BT-128 (BT-128-1 =AFL) BT-128 (BT-128-1 =AFL)
VAT exemption in text (which can 
also be entered for a standard rate 
"S" or zero rate "Z")
EXT-FR-FE-178: Must begin with the 
invoice number on the line (from the 
underlying unit invoice) between ##: 
#F20250025#
EXT-FR-FE-178: #F20250025#

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page135 / 149
VAT breakdown: The exemption reason in text (BT-120) allows for VAT breakdown lines per unit invoice
The exemption reason in text (BT120) allows filtering by unit invoiceBT-116, BT-120, BT-121, BT-118, BT119, BT-117Only VAT breakdown blocks (BG-23) for 
which BT-120) begins with the invoice 
number between ## (#F20250025#)
Foot totals: only apply to multi-vendor invoices
Total without VAT BT-109: 8,850 Not included
Total VAT BT-110: 1,620 Not included
Total with VAT BT-112: 10,470 Not included
Some additional rules for unit invoices:
• If the total with VAT for unit invoices (in EXT-FR-FE-184 of the GROUP line of the multi-vendor invoice and BT-112 
in unit invoices) is missing, then it MUST be calculated as equal to BT-131 + EXT-FR-FE-181 from the GROUP line of 
the multi-vendor invoice, also equal to BT-109 + BT-110 in unit invoices. 
• By convention, the Amount Already Paid (BT-113 in the unit invoice) is set to the amount with VAT (BT-112) insofar 
as the payment is received by the main SELLER.
• The Net Amount Payable (BT-115) is equal to the Amount with VAT (BT-112) - Amount Already Paid (BT-113), i.e. 
0.
NOTE: Document-level allowances and charges are only applied to the main SELLER's unit invoice. It is recommended to 
avoid using them in favor of line-level allowances and charges.
Obligations of Accredited Platforms («Plateformes Agréées») upon receipt:
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10), and 
in particular the EXTENDED-CTC-FR profile, which includes sub-line management and management rules specific 
to multi-vendor invoices.
Obligations of Accredited Platforms («Plateformes Agréées») for issuance that have chosen to support multi-vendor 
invoices for issuance:
Ø Knowing how to process a multi-vendor invoice at issue is not an obligation for Accredited Platforms 
(«Plateformes Agréées»). If it is offered, the AP-E MUST comply with the obligations described below.
Ø Offer the main SELLER unit invoices, allowing them to share them with secondary SELLERS for their accounting 
and tax obligations.
Ø Extract flow 1 (or 10.1 if this option is offered) from the individual invoices, set the status to "Submitted" for 
each individual invoice.
Ø Allow the primary SELLER to distribute payments by unit invoice (in the case of partial payment).
Ø Replicate the "Rejected" and "Refused" statuses received from the BUYER to the PPF for the secondary SELLERS' 
unit invoices.
Obligations of the main SELLER, third-party invoicer:
Ø Know how to create compliant multi-vendor invoices.
Ø Manage life cycle statuses on behalf of all SELLERS.
Ø Send third-party SELLERS the accounting and tax information relating to the unit invoices included in the MultiVendor invoice.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page136 / 149
3.2.39 Case No. 40: Grouped payments, netting, or compensation in the event of cross-purchases/sales
In certain sectors, the BUYER may invoice the SELLER for services or goods, then deduct the amounts due on its own sales 
invoices to the SELLER from the payment of its purchase invoices.
This does not pose any particular invoicing problems as long as there are two invoices:
• one from the SELLER to the BUYER
• and a second from the BUYER (who therefore becomes the SELLER on the invoice) to the SELLER (who is then the 
BUYER on the invoice).
If the BUYER knows that their sales invoice will be paid by offsetting it against their purchase invoice, they can enter a 
payment method with code 97 (in BT-81), which means "clearing between partners."
With regard to the VAT due date for invoices where VAT is payable upon receipt, it is at the time of net payment that both 
invoices can be considered paid.
Moreover, this principle can be seen as a special case of a group payment practice, where a BUYER decides to pay a set of 
invoices in a single transaction, from which they can deduct the payment of certain credits, and why not also deduct by 
offsetting the amounts that their SELLER owes them for cross-services (and therefore sales invoices).
In this case, the BUYER must be able to inform the SELLER of the details of the amount paid to enable the latter to do its 
lettering and, if necessary, to produce its payment receipt status. The "Payment Sent" and "Payment Received" statuses, 
issued for each invoice, can then be used for this purpose, in addition to a payment notification message.
With regard to invoicing, in the case of cross-purchasing/cross-selling, and in particular when the BUYER uses self-billing
to facilitate the daily operations of its SELLERS, while selling services to them in return, these self-billed invoices issued by 
the BUYER on behalf of the SELLER may contain negative lines corresponding to services or deliveries of goods that the 
BUYER provides to the SELLER and which are thus automatically deducted from the amounts to be paid.
This practice should be avoided as it is likely to have the following consequences:
• The negative lines will be considered as lines reducing the SELLER's turnover, whereas they are expense lines for 
the SELLER (and turnover for the BUYER). 
• There is no way to distinguish between negative lines that are actually cancellations of previous lines, reversals on 
advance payment invoices, reversals on consumption estimates, etc., and negative lines that are in fact reverse 
invoicing.
• In addition to the accounting aspects, this can also lead to mixing collected VAT (on sales) and deductible VAT (on 
purchases) insofar as the VAT footer and totals are aggregated data from the lines.
• If the SELLER's VAT regime is the basic exemption, or if they are not subject to VAT, the sales lines fall under the 
SELLER's VAT regime (e.g., exemption for basic exemption), and the negative lines fall under the BUYER's VAT 
regime. Flow 1 will then show part of the invoice as basic exemption and another part with standard VAT, which 
will greatly disrupt the SELLER. The other risk is that the BUYER will "take advantage" of the SELLER's basic 
exemption regime for its negative lines of goods or services sold to the SELLER.
• The VAT pre-fill will be incorrect on both sides, as it is based on footer and total data, and cannot distinguish 
between negative lines for cancellations or reversals and negative lines for reverse sales.
The best practice is therefore to issue two invoices and proceed with a settlement by offsetting the amount of the sales 
invoice against the purchase invoice payment, informing the SELLER by using the "payment method" code" (BT-81) equal to 
"97" and, in the case of a group payment, informing the SELLER of the payment details.
This can also be done through the life cycle statuses:
• "Payment Transmitted" status issued by the BUYER to the SELLER of the purchase invoice for the amount with VAT
on the net payment date.
• "Received" status issued by the BUYER to the SELLER for the sales invoice, on the date of the cleared payment, for 
the amount with VAT.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page137 / 149
3.2.40 Case No. 41 Barter Companies
In the advertising sector, there are transactions where an advertiser sells goods or services in exchange for advertising 
services to be consumed in the coming months.
This practice has led to the emergence of barter companies (inter-company bartering), which are intermediaries that 
purchase goods or services from advertisers, paying only part of the price (at least the VAT) and keeping the rest on account
for a future advertising transaction.
At the same time, these barter companies offer goods and services to advertising agencies, which also pay them only 
partially (at least the VAT). 
Then, when advertising agencies offer advertising services to advertisers, payment for these services can be made through 
compensation between the amounts owed by the barter company to the advertiser and those owed by the advertising 
agency to the barter company.
This case does not detail the role of media agencies, described in case no. 15, which continues to apply when an advertiser 
uses a media agency. The Media Agency acts as a Third Party Buyer and may, where applicable, pay invoices on behalf of 
the advertiser after calling for funds, for the amounts that the advertiser wishes to pay in cash (and not by offsetting through 
its credits with the Barter company).
The main objective of this case is to detail how compensation payments will be organized alongside cash payments, their 
consequences on payment receipt statuses, and how to fill out invoices to indicate the use of compensation via a barter 
company.
Figure53: Illustration of an invoicing cycle through a barter company
In these exchanges, each sale is invoiced, for which at least the VAT amount is paid within the regulatory deadlines.
As a result, when VAT is due upon receipt, the VAT return is filed as if the entire invoice had been paid, since the VAT has 
been paid in full.
In practice, the Barter company only intervenes on the pre-tax portion of the various transactions.
To indicate the specific payment details on invoices, pending future changes to Standard EN16931, the proposed practice is 
as follows:
• For the Advertiser's sales invoice to the Barter company:
ü Use the Amount already paid (BT-113) to indicate the amount to be charged (receivable for the advertiser, 
payable for the Barter company).
Barter
Advertising Agency Advertiser
1. The advertiser sells goods or services to Barter:
• The advertiser invoices BARTER, for which 
VAT is paid by Barter, and the remainder is 
“on account” with Barter as Barter's debt to 
the advertiser
• The advertiser declares 100% of the VAT 
(including if selling services when paying VAT 
in installments)
2. Barter sells goods or services to advertising agencies:
• Barter invoices the advertising agency, for which VAT is paid 
by the advertising agency (which can deduct it), and the 
remainder is “on account” with Barter as a receivable from 
the advertising agency.
• Barter declares 100% of the VAT and pays it to the DGFIP 
(French Public Finance Directorate).
3. The Advertising Agency sells its services to the Advertiser
• The Advertising Agency invoices the Advertiser, for which VAT is paid by the Advertiser (who can deduct it) or by 
the Media Agency on its behalf
• The advertising agency declares 100% of the VAT and pays it to the DGFIP (French Tax Administration)
• The balance is paid by offsetting what the barter company owes the advertiser for its purchases under 1° and 
what the advertising agency owes the Barter company for its purchases under 2°

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page138 / 149
ü The Net Amount Payable (BT-115) therefore corresponds to the amount actually payable (higher than the VAT 
on the invoice).
• For the Barter company's sales invoice to the Advertising Agency:
ü Use the Amount already paid (BT-113) to indicate the amount to be charged (receivable for the Barter company, 
payable for the Régie).
ü The Net Amount Payable (BT-115) therefore corresponds to the amount actually payable (higher than the VAT 
on the invoice).
The Régie's sales invoice to the Advertiser with partial payment through the Barter company is similar to what is described 
in Use Case No. 4, when a third party is identified as having to pay part of the pre-tax amount of the invoice (for example, 
an insurer covering part of the pre-tax amount of a claim repair invoice).
The specific data and associated management rules are therefore as follows: 
• The INVOICE PAYER block (EXT-FR-FE-BG-02) can be used to mention the third-party PAYER in the invoice and 
identify the Barter company.
• The "Amount already paid" field (BT-113) MUST then be used to enter the amount of the invoice covered by the 
third-party PAYER (the Barter company).
• This will ensure that the "Net amount payable" (BT-115) is correct, equal to the total amount with VAT of the invoice 
(BT-112), minus the amount already paid (BT-113, also used to indicate an amount payable by a third party).
• The "invoice note" block (BG-1) can be used to indicate that part of the invoice is being offset via the Barter 
company. The subject code (BT-21) to be used is "PAI", which allows information about the payment to be 
indicated;
With regard to “Payment Received” statuses, in order to declare both a full payment from a VAT perspective and a future 
payment of a portion without VAT, the following information must be provided for each invoice with partial payment 
including the full amount of VAT: 
• A positive payment receipt of the amount with VAT (MDT-215), with the applicable VAT rate (MDT-224) and the 
type code MDT-207 = MEN. In the event of multiple VAT rates, there must be as many records of amounts received
as there are applicable VAT rates.
• A negative payment receipt of the amount without VAT, which will be credited for subsequent compensation and 
to which VAT does not apply (MDT-215 = amount without VAT not paid, as expressed in BT-113 of the invoice, MDT224 = 0). The standard code MDT-207 is MEN (a specific code may be dedicated in a later version of this document).
The same process can also be carried out without a barter company, i.e., the advertiser sells a good or service to the 
advertising agency, which only pays VAT. Then, when the advertising agency sells a service to the advertiser, the latter pays
at least the VAT and deducts the additional amount owed from what the advertising agency owes them. This is the same 
situation of partial payment with compensation, given that VAT is always paid in full upon presentation of the invoice, and 
that the “Patment Received” records must therefore show a total “payment received” entry with the VAT details of the 
invoice, supplemented, where applicable, by a negative “payment received” entry for the amount without VAT to be 
compensated.
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process the minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
3.2.41 Case No. 42: Tax exemption management
In a B2C sale, a non-taxable customer established in a third country may benefit from tax exemption (the customer), i.e., a 
VAT refund. To do this, the merchant offering this service must provide the BUYER with a BVE (Bulletin de Vente à 
l'Exportation, or Export Sales Bulletin), which will allow the customer to exercise their right to a refund when passing through 
customs and the merchant to be informed of this. This must be done within 6 months of the sale and delivery of the BVE.
There are several ways for a merchant to deal with this issue:
• Case 1: Initially declare the sale as domestic B2C, then wait for the customer to clear customs and for the BVE to 
be validated within the deadline in order to refund them, with two options:
ü Refund the customer directly

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page139 / 149
ü Use a tax refund operator, who will refund the customer and then re-invoice the merchant to be reimbursed 
(disbursement), with a management fee (presumably taxable, to be checked with tax specialists). This results in 
an invoice that is partially or totally outside the scope of the reform (if it is just a refund request).
• Case 2: the retailer makes a direct international B2C sale (without VAT), which they must reclassify as a domestic 
B2C sale if they do not obtain proof of validation from the BVE within six months.
• Case 3: the merchant uses a comprehensive service provided by a tax refund operator, which consists of starting 
the sale as a domestic B2C sale, then selling the same goods (already paid for by the customer) to the tax refund 
operator so that the latter can be the seller of the goods to the customer when they go through customs and apply 
for a tax refund. From the retailer's point of view, this is exactly the same as a sale to a private individual, which 
then turns out to be a sale to the company for which they are acting and which has requested a B2B invoice.
3.2.41.1 Case 1: Initial domestic B2C sale
The merchant makes a B2C sale, which results in a B2C 10.3 e-reporting record as follows:
• TT-81 {TRANSACTION CATEGORY} = TLB1
• TT-82: Total amount without VAT
• TT-83: Total VAT amount
• TG-32: Breakdown by VAT rate
ü TT-86: VAT rate
ü TT-87: VAT base
ü TT-88: VAT amount
When the customer clears customs and validates their BVE, the merchant reimburses them for the VAT (or reimburses the 
tax refund operator who did so on their behalf) and must correct their B2C e-reporting from category TLB1 to category TNT1.
To do this, they must cancel the above entry and make a new one corresponding to a sale in category TNT1:
• on the one hand, by including in their e-reporting 10.3 record for the "sales of goods" category (TLB1) the record 
detailed above but with negative amounts for the pre-tax and VAT amounts,
• and by including in their e-reporting 10.3 record for the "sales not subject to VAT in France" (TNT1) category, the 
above amount without VAT and VAT at 0%.
The merchant may charge a commission on this transaction, which is a sale of services, subject to VAT, and potentially 
subject to VAT upon payment receipt. This commission must therefore be treated as a B2C sale, resulting in a contribution 
to the daily total in the "sale of services" category TLS1 in TT-81 of flow 10.3, and in the total of flow 10.4, except in the case 
of an option on the merchant's debits.
Finally, if the VAT is refunded by a tax refund operator on behalf of the merchant and not following a sale of the goods to 
the tax refund operator as detailed in case no. 3, the tax refund operator may request a VAT refund from the merchant 
through an expense invoice only on the amount of VAT refunded, from which any fees charged to the customer on behalf 
of the merchant for the tax refund service may be deducted, this invoice being outside the scope of the reform. 
The tax refund operator invoices this management fee to the merchant, which is a domestic B2B service sale transaction 
and therefore falls within the scope of the electronic invoicing obligation with “Payment Received” status in the absence of 
a debit option. This management fee charged by the tax refund operator can be added to the expense invoice, which then 
becomes subject to the reform, and must be done under the EXTENDED-CTC-FR profile, which is currently the only one that 
allows invoices with lines outside the scope (VAT category = O) and lines with other VAT categories.
3.2.41.2 Case 2: B2C sale with VAT not applicable in France
The merchant makes a direct B2C sale without VAT in category TNT1 (VAT not applicable in France). The customer must go 
through customs to confirm the tax refund.
The merchant must therefore already make a B2C TNT1 e-report:
• TT-81 {TRANSACTION CATEGORY} = TNT1
• TT-82: Total amount without VAT
• TT-83: 0

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page140 / 149
• TG-32: Breakdown by VAT rate
ü TT-86: 0
ü TT-87: Total amount without VAT
ü TT-88: 0
If the customer does not provide proof of BVE validation (customs clearance) within 6 months, the merchant must correct 
the sale as a domestic goods sale. In this case, the amount paid becomes an amount with VAT, and the merchant will 
therefore lose the VAT.
To do this, they must first cancel the above entry by including the above details in their e-reporting 10.3 entry for the 
category "sales not subject to VAT in France" (TNT1), but with negative amounts without VAT. They must then include the 
following details in their e-reporting 10.3 entry for the category "sales of goods" (TLB1)" (TLB1) category (in the case of a 
single VAT rate and for amounts in euros): 
• TT-81 {TRANSACTION CATEGORY} = TLB1
• TT-82: Sale amount / (1 + VAT rate / 100), rounded to 2 decimal places.
• TT-83: Initial sale amount - TT-82
• TG-32: Breakdown by VAT rate
ü TT-86: VAT rate
ü TT-87: TT-82
ü TT-88: TT-83
This correction ensures consistency between the tax exemption that was ultimately not applied and the B2C sales ereporting.
3.2.41.3 Case 3: Tax exemption carried out entirely by a tax exemption operator
In this case, the retailer has a contract with a tax refund operator to entrust them with the entire tax refund operation. The 
principle is then to sell the goods purchased by customers requesting a tax refund to the tax refund operator, so that the 
latter becomes the seller when the customer passes through customs, and therefore acts as the retailer in case no. 1.
To do this, the retailer makes a domestic B2C sale, which is converted into a B2B sale after the customer requests a tax 
refund, exactly as an employee customer would do when requesting an invoice for their company.
The simplest solution is for the retailer to make a B2C sale, which will then be subject to domestic B2C e-reporting as 
described in case no. 1: contribution to the day's cumulative sales through the corresponding 10.3 flow: 
• TT-81 {TRANSACTION CATEGORY} = TLB1
• TT-82: Total amount without VAT
• TT-83: Total VAT amount
• TG-32: Breakdown by VAT rate
ü TT-86: VAT rate
ü TT-87: VAT base
ü TT-88: VAT amount
Then, once the customer has validated their tax refund and been reimbursed by the VAT refund operator, an invoice is 
issued by the retailer under "Cadre de Facturation" 7 (invoice for which VAT has been declared in B2C e-reporting), 
addressed to the VAT refund operator.
Since the tax refund operator is subject to VAT in France, this is a B2B invoice that is subject to the electronic invoicing 
requirement. The tax refund operator can therefore deduct the VAT.
As the sale has already been paid for, the amount already paid on the invoice (BT-113) is equal to the total with VAT, and 
the net amount payable is equal to 0. The specific characteristics of this invoice are therefore as follows:
• SELLER (BG-4): the merchant
• BUYER (BG-7): the tax refund operator
• Business process type (billing framework, “Cadre de Facturation” : BT-23): B7
• Invoicing lines corresponding to the details of the goods sold

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page141 / 149
• Amount already paid (BT-113): equal to the total with VAT (BT-112)
• Net amount payable (BT-115): 0
• Potentially, if there is a BVE reference known to the merchant or to the person creating the invoice on their behalf, 
this can be entered in BT-18 (Invoiced item identifier), with the qualifier (BT-18-1) equal to APT.
The merchant must ensure that the posting of this invoice is accompanied by a reverse entry for the B2C sale it replaces.
This invoice can be issued by the tax refund operator using self-billing (see case 19b). As a reminder, prior agreement 
between the tax refund operator and the merchant is required, which can take the form of a self-billing mandate associated 
with the tax refund service contract offered by the tax refund operator to the merchant.
The tax refund operator then finds itself in the same situation as the retailer described in case no. 1, except that it can 
directly declare the sale in category TNT1 once it has refunded the VAT (which compensates for the VAT deductibility it will 
be able to benefit from following the purchase from the retailer, as evidenced by the B2B invoice in the "Cadre de 
Facturation" B7).
Flow 10.3: e-reporting by the tax refund operator:
• TT-81 {TRANSACTION CATEGORY} = TNT1
• TT-82: Total amount without VAT
• TT-83: 0
• TG-32: Breakdown by VAT rate
ü TT-86: 0
ü TT-87: Total amount without VAT
ü TT-88: 0
Obligations of Accredited Platforms («Plateformes Agréées») :
Ø Know how to process minimum base formats and profiles (invoice and CDAR), and e-reporting (flow 10).
3.2.42 Case No. 43: E-reporting for international B2B
The purpose of this chapter is to review the e-reporting obligations for international B2B sales and international B2B 
acquisitions excluding imports of goods, and to link them to the underlying invoices issued or received.
Firstly, it should be noted that there is no electronic invoicing obligation for international B2B invoices. However, it is 
possible to exchange electronic invoices. In this chapter, we will only discuss cases where these electronic invoices are in 
one of the minimum standard formats.
Finally, the electronic invoicing requirement will come into effect with the implementation of the ViDA Directive in July 2030, 
under specific terms and conditions that are yet to be determined.
This chapter will be divided into five sections:
• A reminder of what an international B2B invoice is and the management rules that apply.
• International B2B sales.
• International B2B acquisitions.
• Payment Receipts on international B2B sales.
• Some additional information on practices specific to international B2B exchanges.
3.2.42.1 What are the specific features of an international B2B invoice compared to a domestic B2B invoice?
From a structural point of view, an international B2B invoice is identical to a domestic B2B invoice. A few additional 
management rules apply and are defined in Standard EN16931, described in Standard XP Z12-012, and in particular in its 
Annex A.
In particular, the management rules BR-IC-XX (Intra-Community supply), BR-G-XX (Export), and BR-AE-XX (Reverse charge) 
apply more specifically.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page142 / 149
The management rules specific to France apply to international B2B sales invoices, in order to ensure compliance with 
international B2B e-reporting requirements, for which the data required is that contained in the invoices or derived from 
them as a result of certain mapping rules. These rules also provide for a few differences between a domestic B2B invoice 
and an international B2B invoice:
• BR-FR-11: No obligation to provide a SIREN number listed in the PPF Directory (“Annuaire”) for international buyers 
(BT-47).
• BR-FR-21: The buyer's electronic address has no French specific constraints (and therefore does not begin with a 
SIREN number) for non-self-billed invoices.
• BR-FR-22: The seller's electronic address has no French specific constraints (and therefore does not begin with a 
SIREN number) for self-billed invoices.
This therefore introduces a difference in control processing for Accredited Platforms on the sending side (and subsequently 
in the issuance of an e-reporting flow 10 instead of a flow 1) between a domestic B2B invoice and an international B2B 
invoice. The XP Z12-012 standard provides a way of indicating the type of processing expected in the invoice through a Note 
with a subject code equal to BAR (rule BR-FR-20). This is one possible option, the alternative being that the Accredited 
Platform (“Plateforme Agréée”) and its VAT Registered customer agree on a method of qualifying the expected processing 
in parallel (channel for transmitting invoice data or the invoice between the VAT Registered entity and their Accredited 
Platform by type of processing, file naming, etc.).
It is important to ensure that the international recipient accepts electronic invoices that comply with the EN16931 standard 
in the format presented (UBL, CII, Factur-X, potentially third-party formats such as EDIFACT or PEPPOLBIS), as this is 
increasingly being deployed (and is the common target with the deployment of Vida by 2030). The use of the EXTENDEDCTC-FR profile also requires the recipient's acceptance, as an EXTENDED profile may eventually exist at the European level. 
The use of PEPPOL at least guarantees that all counterparties accept the third-party PEPPOLBIS 3.0 format (UBL compliant 
with the EN16931 standard, supplemented by a few additional Business Rules).
International B2B purchase invoices are created by international entities and do not have to comply with specific French 
Business Rules. It is up to the recipient (in this case, the French VAT Registered entity) to decide which formats and profiles 
they accept.
However, this means that certain information required for international B2B e-reporting may not be present in invoices 
received from abroad and therefore needs to be completed by the French VAT Registered entity purchaser.
E-reporting (flow 10) must be submitted periodically (every ten days, every month, every two months). As it may happen 
that invoices are known to the French VAT Registered entity after a certain delay, particularly for international B2B purchase 
invoices, but in some cases also for sales invoices, it is possible that invoices with an invoice date prior to the start date of 
the Flow 10 period may be included in this Flow 10. This is particularly necessary if these invoices are included in the VAT 
return (CA3) corresponding to the period.
As it is not permitted to create invoices with an invoice date later than their creation date, the invoice data present in a flow 
10 (flow 10.1) cannot have an invoice date (TT-19) later than the date of submission of flow 10 (within 10 days of the end of 
the period for companies under the normal regime).
Normally, only invoice data with a date prior to the end date of the flow 10 period should be present in flow 10. However, 
it is possible that some flow 10.1 records may contain invoices with an invoice date between the end of the period and the 
date of submission of flow 10, in the event where they correspond to transactions whose Tax Point Date occurred before 
the end of the flow 10 period. As a result, the pre-filled VAT return will take these invoices into account for the period in 
which they fall.
3.2.42.2 International B2B sales
This chapter will be completed in the next version.
All VAT Registered entity in France are subject to e-reporting on their international B2B sales for transactions described in 
Article 290 of the CGI.
A distinction must be made between transactions to a European Union member country and those to a non-EU country, and 
then between sales of goods and sales of services:
• Intra-Community sales:
ü Sales of goods: these are “intra-Community supply,” which are subject to reverse charge VAT, i.e., VAT payable 
by the recipient (the purchaser) in its country. These sales invoices mainly have a “VAT category” code equal to 

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page143 / 149
“K” and a VAT exemption reason code “VATEX-EU-IC” and/or a VAT exemption reason in text “Intra-Community 
supply” or its equivalent in the language of the invoice. The VAT category “K” and a VAT rate of 0 are present in 
the VAT breakdown (BG-23, BT-118, BT-119 respectively), as well as the reason for exemption in text and code 
(BT-120, BT-121). They are also present in line (BG-25) and allowances and charges at Document level (BG-20, 
BG-21), with the reasons for exemption in code and text being added to the EXTENDED-CTC-FR profile in 
anticipation of the revision of Standard EN16931. All BR-IC-XXX management rules apply to the EN16931 profile. 
For the EXTENDED profile, rule BR-IC-08 is replaced by rule BR-FREXT-IC-08.
ü Sales of services in the EU: this is also a reverse charge mechanism which generally applies, i.e., VAT is payable 
by the customer in the country of destination but using the VAT category code “AE” and the exemption reason 
code “VATEX-EU-AE.” This rule must be confirmed at EU level (as part of the ViDA implementation work). All 
BR-AE-XXX management rules apply to the EN16931 profile. For the EXTENDED profile, rule BR-IC-08 is replaced 
by rule BR-FREXT-AE-08.
• Sales to a country outside the EU:
Sales of goods: these are exports of goods for which the VAT category is “G,” the VAT rate is “0,” and the VAT 
exemption reason is “VATEX-EU-G” and/or the text reason is “Export.”. All BR-G-XXX management rules apply 
to the EN16931 profile. For the EXTENDED profile, rule BR-G-08 is replaced by rule BR-FREXT-G-08.
ü Sales of services: the tax treatment is different from exports, which only concern goods. However, the VAT 
category code “G” should also be used, with VAT equal to 0, and VAT reason code VATEX-EU-G and/or a VAT 
exemption reason in text form indicating that it is a sale of services outside the EU.
E-reporting consists of producing a flow 10.1 (TG-8) record for each invoice from the e-reporting message described in 
external specifications 3.1 and its annexes and included in Annex A of standard XP Z12-012.
This information corresponds to that contained in the invoices, with a correspondence given in the “E-REPORTING - Flow 
10” sheet of Annex A of standard XP Z12-012. For certain data, a mapping between the information in the invoice and flow 
10.1 is necessary. These are described in the Franch Business Rules among the mapping rules (BR-FR-MAPXX) for which the 
column entitled “Map Flux 10” is checked in the “BR-France CTC” sheet of Annex A of standard XP Z12-012.
Particular attention must be paid to rule BR-FR-MAP-16 (G2.19 of external specifications 3.1) to enter the identifier of the 
Seller (TT-33) or Buyer (TT-37) depending on the geographical area of the party and the identification information:
• France: the SIREN and TT-33-1 / TT37-1 equal to 0002,
• EU: the intra-community VAT number and TT-33-1 / TT37-1 equal to 0223,
• Outside the EU: the country code + the first 16 characters of the company name and TT-33-1 / TT37-1 equal to 
0227,
• New Caledonia: the RIDET and TT-33-1 / TT37-1 equal to 0228,
• French Polynesia: TAHITI and TT-33-1 / TT37-1 equal to 0229.
Certain data is not required at the start of the reform, but only on September 1st, 2027 for those who are required to issue 
e-reporting as of September 1st, 2026 (large companies and mid-sized companies). These are blocks with rule G6.15 
(Allowances and charges at document level, invoice line data, date of issue of the previous invoice).
The obligation of Accredited Platforms is to receive an e-reporting file compiled by their VAT-registered customer, check 
it, and transmit it to the PPF.
However, this Accreditd Platform, like any Compatible Solution, may offer additional services to compile this e-reporting:
• On the one hand, by extracting the expected data from invoices (as is done for a 1 flow), then applying the mapping 
rules to create a single 10.1 flow record (TG-8). This is easier to do if the invoice is in a structured format that 
complies with the minimum base formats and profiles.
• On the other hand, by aggregating unit records from flow 10.1 for international B2B sales and flow 10.3 (B2C sales) 
to create the periodic e-reporting message for international B2B and B2C sales to be sent to the PPF.
The company can produce this e-reporting information from its information system (accounting management, commercial 
management) or directly from its issued invoices (particularly in the structured minimum base format as described in 
standard XP Z12-012). However, this information must correspond to that contained in its international B2B sales invoices.
Case of invoices in foreign currency: All information is provided in the currency of the invoice except for the VAT amount 
(TT-52), which is expected to be in EUROS only. It is also possible to provide all amounts in EURO equivalent (for example, if 
the data comes from accounting records). In this case, the currency of the invoice (TT-22) must be EUR.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page144 / 149
Impact of transfer of ownership for the delivery of goods for export: in the case of export sales, transport to the destination 
country may be carried out by the seller as part of an international stock transfer, with the sale and transfer of ownership 
of the goods taking place once they arrive at their destination. In this case, the sale and transfer of ownership take place on 
the purchaser's territory and fall outside the scope of international B2B sales e-reporting.
E-reporting of payments: given that the regulations have excluded from the scope of e-reporting invoices for which VAT is 
payable by the recipient, all invoices for international B2B services sales that fall into this category do not have to be ereported (flow 10.2) on these invoices.
3.2.42.3 International B2B acquisitions
All VAT Registered entity in France are subject to e-reporting on these international B2B acquisitions for the transactions 
described in Article 290 of the CGI.
They therefore receive three types of invoices from international sellers:
• Intra-Community invoices from sellers subject to VAT in another EU Member State. These invoices are in the VAT 
category “Intra-Community supply” (code “K”) or in the VAT category “VAT Reverse Charge” (code “AE”), which in 
both cases means that VAT is payable by the recipient (the taxable person who must report it electronically). If they 
are electronic and comply with the EN16931 standard, invoices have a VAT detail record (BG-23) with a category 
code (BT-118) equal to “K” or “AE,” a VAT rate (BT-119) equal to 0, and a code exemption reason (BT-121, known 
as the VATEX code), equal to “VATEX-EU-IC” or “VATEX-EU-AE” and/or a reason for exemption in text (BT-120) 
equal to “Intra-Community supply” or “Reverse Charge” (or its translation in the language of the invoice). All 
Business Rules of standard EN16931 BR-IC-XX or BR-AE-XX apply.
• Invoices for the purchase of services from international sellers (not established in another EU country), which are 
considered by the seller to be exports (therefore VAT category code “G” if they are electronic and comply with 
standard EN16931).
• Invoices for imports of goods from sellers not established in an EU country, which are excluded from e-reporting 
obligations.
E-reporting consists of producing a 10.1 (TG-8) record for each invoice from the e-reporting message described in external 
specifications 3.1 and its annexes and included in Annex A of standard XP Z12-012.
However, the information produced must be from the perspective of the VAT Registered BUYER in France. When the BUYER
is in a reverse charge VAT position, which is very often the case, whether for intra-Community supply invoices for the 
purchase of goods or services or invoices for the purchase of services outside the EU (imports of goods being excluded from 
e-reporting),
it is required to provide VAT information corresponding to the VAT collected as a result of reverse charge.
The following information must therefore be provided in the VAT breakdown (TG-23):
• TT-54: VAT base as shown on the invoice (in the VAT breakdown with category code K, AE, or G).
• TT-55: Amount of reverse-charged VAT, i.e., obtained by multiplying the tax base by the applicable VAT rate, as 
recorded in the accounts for reverse charge VAT.
• TT-56: VAT category code: at this stage, it is asked to enter the code “AE,” as this is VAT reverse charge from the 
perspective of the VAT Registered BUYER.
• TT-57: VAT rate applicable for VAT reverse charge, as recorded in the accounts for reverse-charged VAT
• TT-58: Reason for exemption in text: corresponding to the situation.
• TT-59: Reason for exemption in code: VATEX-EU-AE, but it may be useful to distinguish between intra-Community 
supply and reverse charge on the acquisition of international services outside the EU (to be considered in 
preparation for ViDA).
• TT-52: the sum of TT-55, converted into EUR if in another currency.
These purchase invoices are received directly by the VAT Registered entity, in any form. The VAT Registered entity is 
generally the only one who can distinguish between imports of goods and purchases of services outside the EU. The VAT 
Registered entity is also generally the only one who knows at what rate to reverse charge its VAT.
Only the data required at the start of e-reporting for international B2B sales (“DEMARRAGE”) will be required on an ongoing 
basis for e-reporting of international B2B acquisitions (and therefore not line data, document-level charges or allowances).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page145 / 149
This means that e-reporting of acquisitions is mainly carried out by the VAT Registered entity, who can use a compatible 
solution, or even its Accredited Platform (“Platforme Agréée”), to help them produce their e-reporting:
• On the one hand, by extracting the expected data from invoices (as is done for a flow 1, but without line data, 
document-level charges and allowances), then applying the mapping rules and, where applicable, the Business 
Rules specific to the VAT Registered BUYER to create a single flow 10.1 (TG-8) record.
• On the other hand, by aggregating unit records of international B2B acquisition flow 10.1 to create the periodic ereporting message for international B2B acquisitions to be sent to the PPF.
Case of invoices in foreign currency: All information is provided in the currency of the invoice except for the VAT amount 
(TT-52), which is expected in EUROS only. It is also possible to provide all amounts in EURO equivalent (for example, if the 
data comes from the accounting record). In this case, the invoice currency (TT-22) must be EUR.
NOTE: international B2B acquisitions are not subject to e-reporting of payment receipts by the Buyer subject to VAT in 
France.
3.2.42.4 Payments received on international B2B sales
All entities subject to VAT in France are required to submit e-reporting for payments received on international B2B sales for 
transactions described in Article 290 of the French General Tax Code (CGI) and for which VAT is due upon receipt of payment.
This applies to invoices for services for which the option “on debits” has not been activated, as well as pre-payment invoices.
On the other hand, Article 290 A, which governs e-reporting obligations for payments, expressly excludes from this 
obligation any transaction for which the VAT is payable by the customer, and therefore any transaction involving a VAT 
reverse charge mechanism by the customer. This therefore concerns the vast majority of intra-Community B2B sales.
The most relevant case is that covered by Article 290 II relating to taxable persons not established in France but having a 
VAT taxable activity in France, who must produce an e-report on their sales located in France on which they are liable for 
VAT. When it comes to services and they have not opted for “debits”, or when it comes to pre-payment invoices, e-reporting 
of payment is required.
For the invoices concerned, an e-reporting of collection is required, corresponding to flow 10.2 (TG-34), in which the 
following must be provided:
• The invoice number (TT-91)
• The invoice date (TT-102)
• A Payment (collection) block (TG-35) consisting of:
Payment date (TT-92)
Breakdown of payments by VAT rate (TG-36)
§ Rate (TT-93): as in the invoice
§ Currency code (TT-94): must be EUR
§ Amount received (TT-95): equivalent value in EUR in the case of payment in foreign currency.
In the event of an e-reporting error for a collection (amount error, invoice allocation error), a record with a negative amount 
can be used to correct it.
In the event of an invoice followed by a credit note subject to e-reporting of payment receipts, either a positive receipt is 
made for the invoice and a negative receipt for the credit note (on the same date), or no e-reporting of payment receipts or 
e-reporting of payment receipts for the balance only with reference to the invoice.
In the case of a corrected invoice, only the e-reporting of the payment on the correctedinvoice is made.
E-reporting of payments is carried out by the VAT Registered entity SELLER after reconciling its payments with the relevant 
international B2B sales invoices.
The Accredited Platform (“Plateforme Agréée”) or Compatible Solution can help the company to compile this e-reporting of 
payment receipts:
• On the one hand, by offering its customer the option of producing “Payment Received” life cycle statuses to 
transform them into individual “Payment Received” records for international B2B sales invoices (TG-34).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page146 / 149
• On the other hand, by aggregating unit records of 10.2 “Payment Received” records from international B2B sales 
and B2C “Aggregated Payment Received” record (10.4) to create the periodic e-reporting message on payment
receipts to be sent to the PPF.
3.2.42.5 Obligations of Accredited Platforms (“Plateformes Agréées”) and VAT Registered entities
Obligations of Accredited Platforms (“Platfeforme Agréée”):
Ø Know how to process (check and transmit) an e-reporting message (flow 10) transmitted by its VAT Registered 
customer.
Obligations of VAT taxpayers in France:
Ø If they carry out international B2B sales, periodically produce an e-reporting message for their international B2B 
and B2C sales.
Ø If they carry out international B2B sales subject to VAT on collection and for which VAT is not paid by the 
customer, periodically produce an e-reporting message for international B2B sales and B2C sales.
Ø If it makes intra-Community B2B acquisitions (of goods or services) or purchases services outside the EU, it must 
periodically produce an international B2B acquisition e-reporting message.
Optional services of Approved Platforms or Compatible Solutions:
Ø Offer services for the creation of individual e-reporting records (10.1, 10.2, 10.3, 10.4) based on international 
B2B invoices issued or received, applying data extraction and mapping rules described in Standard XP Z12-012 
and additional Business Rules or completion functions by the VAT Registered entity for missing information.
Ø Offer e-reporting unit record aggregation services during the VAT Registered entity's e-reporting periods to 
create the e-reporting files (flow 10) expected by the CdD of the PPF.
3.2.42.5.1 Case No. 43a: Triangular transactions
A triangular transaction involves three parties established in three EU countries carrying out a three-way transaction as 
follows:
• Seller A, located in EU Member State A, sells goods to Intermediary Buyer B, established in EU Member State B.
• Intermediary B sells the same goods to Buyer C, located in EU Member State C.
• Delivery is made directly from A to C.
As the goods are delivered from country A to country C, the application of VAT rules would require Intermediary B to register
for VAT in country C in order to reverse charge its purchase from seller A in country C, then resell the goods to buyer C 
subject to local VAT. However, a simplification measure is provided for in VAT Directive 2006/112/EC, recently amended by 
the ViDA Directive, which allows Intermediary B to register only in its EU Member State B.
In this case,
• A declares an intra-Community supply,
• B does not declare an intra-Community acquisition and does not reverse charge VAT on its purchase of the goods. 
Its sales invoice is an intra-Community supply with VAT reverse charged by the customer C, who must also mention 
that this is a triangular transaction.
• C makes an intra-Community acquisition and self-assesses the VAT.
Invoicing is done as follows (for electronic invoices):
• Sales invoice from A to B:
ü Seller: A
ü Buyer: B
ü Ship to (delivery address): C

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page147 / 149
ü VAT: "Intra-Community supply“ (K, VATEX-EU-IC, ”VAT exemption – Intra-Community supply – Article 138 
Directive 2006/112/EC"). A specific VATEX code could be useful to indicate a triangular transaction without 
reverse charge VAT by B. To be discussed in the context of European work on the implementation of ViDA.
• Sales invoice from B to C:
ü Seller: B
ü Buyer: C
ü Ship to (delivery address): C
ü VAT: “Intra-Community supply” (K, VATEX-EU-IC, “Reverse charge – Article 141 Directive 2006/112/EC –
Triangular transaction”). A dedicated VATEX code would be useful to indicate that this is a triangular transaction 
without e-reporting by B and with reverse charge of VAT by C. To be discussed as part of the European work on 
the implementation of ViDA.
ü “Triangular transaction” reference: must be included on the invoice. The reference “Reverse charge – Article 
141 Directive 2006/112/EC – Triangular transaction” as grounds for VAT exemption allows this to be mentioned.
As for the application of the reform in France, the consequences are as follows, depending on which party is established in 
France:
• If Seller A is in France (taxable person): their sale is a standard intra-Community supply of goods, which is subject 
to international B2B e-reporting, giving rise to a 10.1 flow.
• If intermediary B is in France (taxable):
ü As B does not self-assess VAT on its purchase from A due to the simplification measure for triangular 
transactions, it does not have to report this acquisition electronically.
ü As the resale from B to C is part of a triangular transaction leading to VAT exemption for B in country B and 
reverse charge VAT for C in country C, B is also not required to e-report this international B2B sale.
• If C is in France: C makes an intra-Community acquisition with VAT reverse-charged by the purchaser. It must 
therefore submit an e-report for acquisitions with reverse-charged VAT, and therefore, in its 10.1 flow:
ü TT-54: VAT base as shown on the invoice (in the VAT breakdown with category code K).
ü TT-55: Amount of reverse-charged VAT, i.e., obtained by multiplying the tax base by the applicable VAT rate, as 
recorded in the accounts for reverse-charged VAT.
ü TT-56: VAT category code: enter the code “AE,” as this is reverse-charged VAT from the perspective of the 
taxable purchaser C.
ü TT-57: VAT rate applicable for reverse charge VAT, as recorded in the accounts for reverse charge VAT.
ü TT-58: Reason for exemption in text: corresponding to the situation: “Reverse charge – Article 141 Directive 
2006/112/EC – Triangular transaction.”
ü TT-59: Reason for exemption in code: VATEX-EU-AE, pending the possible introduction of a VATEX code specific 
to this situation.
ü TT-52: the sum of TT-55, converted into EUR if in another currency.
Obligations of Accredited Platforms (“Platformes Agréées”):
Ø Know how to process (check and transmit) an e-reporting message (flow 10) transmitted by its VAT Regsitered
customer.
Obligations of Actors:
Ø Know how to manage a triangular transaction and the simplification measure, with its consequences on the data 
to be provided in invoices and in international B2B e-reporting (flow 10.1).
3.2.42.5.2 Case No. 43b: Stock transfers treated as intra-Community supply
When a company transfers stock between two European Union member states, this is treated as an intra-Community supply.
This transaction does not necessarily give rise to an invoice, but to another supporting document, which may be, for 
example, a pro forma invoice (which is not an invoice), following its own numbering series.

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page148 / 149
These transactions give rise to an e-reporting record (flow 10.1), either for international B2B sales (when France is the 
country of departure for the stock transfer) or for international B2B acquisitions (when France is the destination country for 
the intra-Community stock transfer).
If the supporting document is a pro forma invoice, it can be exchanged electronically, but it is a non-reform document. Its 
number can then be used for the invoice number in flow 10.1 (TT-19). The type code (TT-21) must then be equal to “380” 
to comply with the PPF CdD control business rules on invoice type codes.
Next, in the event of a transfer of stock from France to another EU member country, the e-reporting must be completed as 
follows:
• Seller ID (TT-33): SIREN number of the taxable person (TT-33-1 = 0002).
• Seller's VAT number (TT-34): VAT number in France.
• Buyer ID (TT-37): SIREN number of the taxable person (TT-37-1 = 0002), i.e. the same as for TT-33.
• BUYER's VAT number (TT-38): VAT number of the taxable person in the destination country (different from TT-34).
• VAT category: K, with VAT exemption reason equal to VATEX-EU-IC (“VAT exemption – Intra-Community supply –
Article 138 Directive 2006/112/EC”).
• No VAT (TT-52 = 0)
In the event of a transfer of stock from an EU member country to France, the e-reporting must be completed as follows:
• Seller ID (TT-33): SIREN number of the taxable person (TT-33-1 = 0002).
• Seller's VAT number (TT-34): VAT number in the country of departure of the stock.
• Buyer ID (TT-37): SIREN number of the taxable person (TT-37-1 = 0002), i.e. the same as for TT-33.
• BUYER'S VAT number (TT-38):
• VAT number of the taxable person in France.
• VAT category: AE, with VAT exemption reason equal to VATEX-EU-AE / “Reverse charge”
• VAT amount (TT-52): amount of self-invoiced VAT in EURO
• VAT breakdown (TG-23), by reverse charge VAT rate (if several)
ü Base (TT-54) = reverse charged amount excluding VAT,
ü VAT amount (TT-55): reverse-charge VAT,
ü VAT category (TT-56): AE,
ü Reason for exemption (TT-58 / TT-59): VATEX-EU-AE / “Reverse charge”.
NOTE: a stock transfer can also be made from France to a country or territory considered outside the EU for tax purposes 
(for example, in the context of a local sale in the destination country after stock transfer). In this case, there is no obligation 
to transmit a 10.1 flow for this transfer of stock outside the EU.
Obligations of Accredited Platforms (“Plateformes Agréées”):
Ø Know how to process (check and transmit) an e-reporting message (10.1 data flow) transmitted by its VAT 
Regsitered customer.
Obligations of Actors:
Ø Know how to produce the information necessary to compile the 10.1 data flow record for the required 
international B2B e-reporting.
3.2.43 Case No. 44: Transactions with entities established in the DROMs/COMs/TAAFs
The DROMs (Overseas Departments and Regions), COMs (Overseas Collectivities), and TAAFs (French Southern and Antarctic 
Lands) are divided into three geographical areas, resulting in two groups from a tax perspective:
• The DROMs of Martinique, Guadeloupe, and Réunion, which are considered part of “France” for tax purposes along 
with mainland France, although they benefit from specific VAT rates.
• The DROMs of French Guiana and Mayotte are excluded from the territory of application of VAT and are therefore 
considered as export territories (outside the EU).

XP Z12-014 - B2B use cases applicable in the context of the Electronic Invoicing Reform in France.
ANNEX A (normative): Description of the main specific use cases – V1.3 Page149 / 149
• The COMs and TAAFs are also excluded from the territory of application of VAT and are therefore considered as 
export territories (outside the EU), like French Guiana and Mayotte.
The rules for applying the reform are therefore based on this classification, which consists of two groups:
• The “France” group for VAT purposes, comprising mainland France, Guadeloupe, Martinique, and Réunion.
• The “Export” group for VAT purposes: French Guiana, Mayotte, the overseas departments (COM), and the overseas 
collectivities (TAAF).
For example:
• Transactions within the “France” group, for example between Guadeloupe and mainland France, are subject to the 
electronic invoicing requirement.
• Transactions involving the sale of goods from Guadeloupe to French Guiana are therefore considered international 
B2B export sales, subject to B2B sales e-reporting by the taxpayer established in Guadeloupe.
• Transactions involving the sale of goods from French Polynesia (COM) to Réunion constitute an import of goods 
and are therefore excluded from international B2B e-reporting of acquisitions.
• Transactions involving the sale of services from French Polynesia (COM) to Réunion constitute an acquisition of 
services, which must be subject to international B2B e-reporting of acquisitions by the taxpayer based in Réunion.
This has an impact on the country codes to be used in the addresses of the parties (SELLER or BUYER) and on what needs to 
be transmitted in flows 1 and 10.1.
In fact, there are several practices for the use of country codes:
• Use of FR, meaning that it is a French territory, mainly for the DROMs.
• Use of the ISO 3166 code corresponding to the geographical area, namely:
ü French Guiana: GF
ü French Polynesia: PF
ü French Southern Territories: TF
ü Guadeloupe: GP
ü Martinique (la): MQ
ü Mayotte: YT
ü New Caledonia (la): NC
ü Reunion (La): RE
ü Saint Barthélemy: BL
ü Saint Martin (French part): MF
ü Saint Pierre and Miquelon: PM
ü Wallis and Futuna: WF
In flow 1, which only applies to transactions within the France group (Metropolitan France, Guadeloupe, Martinique, 
Réunion), only the country code “FR” should be used for each of the Parties.
In flow 10.1, for entities established in the “France” group for VAT purposes, the country code must be set to “FR.” A mapping 
rule is therefore necessary if the country code of the underlying invoice is “GP,” “MQ,” or “RE.”
Conversely, for French Guiana and Mayotte, if the country code is “FR,” it must be replaced with ‘GF’ or “YT,” respectively. 
A mapping rule is therefore also necessary, based, for example, on the postal code (beginning with 973 for French Guiana 
and 976 for Mayotte).
Obligations of Accredited Platforms (“Plateformes Agréées”):
Ø Know how to process the minimum base formats and profiles and create flow 1 by applying the mapping rules 
(in particular the one on the country codes of BR-FR-MAP-14 actors).
Optional functionality of Accredited Platforms (“Plateformes Agréées”):
Ø Assist taxpayers in creating flows 10.1, taking into account the specific characteristics of the DROM/COM.