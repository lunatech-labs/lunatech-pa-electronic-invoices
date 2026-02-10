# TODO — PDP Facture

Liste des tâches restantes et améliorations prévues, par ordre de priorité.

## Haute priorité

### 1. Génération Typst des factures PDF

Remplacer la génération PDF actuelle par [Typst](https://typst.app/), un système de composition moderne et rapide.

- [ ] Créer un template Typst pour les factures (Invoice + CreditNote)
- [ ] Intégrer la bibliothèque `typst` en Rust pour la génération PDF
- [ ] Supporter les données multi-lignes, remises, TVA multiple
- [ ] Générer le PDF/A-3 pour Factur-X avec Typst
- [ ] Tests de rendu sur tous les cas d'usage (UC1-UC5)

### 2. Adresses électroniques (BT-34 / BT-49) — schemeID

Les champs `seller_endpoint_scheme` (BT-34-1) et `buyer_endpoint_scheme` (BT-49-1) sont obligatoires pour l'e-invoicing (BR-FR-12/13, `schemeID="0225"`).

- [ ] Compléter les parsers CII/UBL pour extraire le `schemeID`
- [ ] Mettre à jour les serializers CII/UBL pour écrire le `schemeID`
- [ ] Ajouter `URIUniversalCommunication` / `EndpointID` dans toutes les fixtures de test
- [ ] Ajouter une validation BR-FR-12/13 en erreur (pas seulement warning)

### 3. Communication SFTP avec le PPF

Implémenter le protocole SFTP pour l'échange de fichiers avec le PPF (pas d'API REST).

- [ ] Client SFTP avec authentification par clé RSA
- [ ] Nommage des enveloppes tar.gz selon la convention AIFE
- [ ] Nommage des fichiers F1 (`Base_` / `Full_` prefix)
- [ ] Gestion des SAS de dépôt et récupération
- [ ] Traitement des CDV de flux (500 Recevable / 501 Irrecevable)

## Moyenne priorité

### 4. Validation BR-FR complète

- [ ] BR-FR-01 à BR-FR-03 : format ID facture (≤35 chars, caractères autorisés, années 2000-2099)
- [ ] BR-FR-05/06/07 : notes obligatoires (PMT, PMD, AAB) et codes sujets
- [ ] BR-FR-08 : cadres de facturation autorisés
- [ ] BR-FR-09 : cohérence SIRET/SIREN
- [ ] BR-FR-10/11 : SIREN vendeur/acheteur obligatoire
- [ ] BR-FR-20 : note BAR (B2B, B2BINT, B2C, OUTOFSCOPE, ARCHIVEONLY)
- [ ] BR-FR-21/22 : adresse acheteur commence par SIREN + schemeID 0225

### 5. E-reporting (Flux 10)

- [ ] Modèle de données pour transactions et paiements
- [ ] Sérialisation au format spécifique PPF
- [ ] Règles BR-FR-MAP-23 (conversion dates UBL → CII)
- [ ] Tests avec exemples officiels

### 6. Annuaire PPF (Flux 13/14)

- [ ] Consultation de l'annuaire pour résolution des adresses électroniques
- [ ] Actualisation de l'annuaire (Flux 13)
- [ ] Export annuaire (Flux 14)

## Basse priorité

### 7. Factur-X BASIC WL → structuré

- [ ] Génération de lignes à partir de la ventilation TVA (toléré jusqu'au 01/09/2027)
- [ ] Marquage du document comme converti

### 8. Support PEPPOL AS4

- [ ] Intégration du protocole AS4 pour échange inter-PDP
- [ ] Gestion des certificats et SMP/SML

### 9. Interface d'administration

- [ ] Dashboard de suivi des factures et statuts CDV
- [ ] Consultation des logs et erreurs de validation
- [ ] Configuration des clients (émetteurs/récepteurs)
