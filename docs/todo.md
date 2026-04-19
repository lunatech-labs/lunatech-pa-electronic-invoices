# TODO — Ferrite (PDP Facture)

Liste des tâches restantes et améliorations prévues, par ordre de priorité.

**Dernière mise à jour** : 2026-04-19
**Tests** : 860+ passent, 0 échec

---

## Fait

- [x] Multi-tenancy par SIREN (`TenantRegistry`, `TenantEntry`, auto-config)
- [x] Routes auto-générées par tenant (`{siren}/in → pipeline → {siren}/out`)
- [x] Validation XSD du Flux 1 PPF avant envoi (bloque si invalide)
- [x] Système d'alertes (`AlertErrorHandler`, classification Critical/Warning/Info)
- [x] Rapports d'alerte JSON avec actions recommandées
- [x] Webhook de notification pour alertes critiques
- [x] Documentation Peppol étendue (protocole AS4, WS-Security, PKI, migration Oxalis)
- [x] Documentation annuaire PPF (F14 complet/différentiel, F13 actualisation, copie locale)
- [x] Import F14 streaming via channel mpsc (mémoire bornée ~5 Mo au lieu de ~7 Go)
- [x] Recherche annuaire enrichie (adresses, B2G, codes routage, plateformes)
- [x] Logo Ferrite (SVG responsive + PNG)

### Séparation PDP émettrice / PDP réceptrice

- [x] `PipelineMode` enum (Emission/Reception) dans la config des routes
- [x] Pipeline émission : validation → Flux 1 PPF (TOUJOURS) → CDAR 200 → routage
- [x] Pipeline réception : validation → PAS de Flux 1 → CDAR 202 "Reçue" → livraison
- [x] `CdarProcessor::emission()` (CDV 200) et `::reception()` (CDV 202)
- [x] Détection intra-PDP dans `RoutingResolverProcessor` (our_matricule)
- [x] Canal mpsc intra-PDP (émission → réception locale sans réseau)
- [x] CLI `--mode emitter|receiver|both` sur `start` et `run`
- [x] Route HTTP inbound corrigée → pipeline réception (plus de Flux 1)
- [x] Route `intra-pdp-reception` via `ChannelConsumer`
- [x] Tests unitaires CDV 202, CliMode, PipelineMode, Destination::IntraPdp

## Haute priorité

### 1. Réception inter-PDP — affinage

L'architecture émission/réception est en place. Reste à affiner :

- [ ] Livraison au bon tenant en réception (`{siren}/out/` selon l'acheteur)
- [ ] Notification de l'acheteur après réception (webhook, email, ou polling)
- [ ] Gestion du CDV retour acheteur→vendeur (CDV 204, 210, etc. à relayer)

### 2. Annuaire PPF — Copie locale (Flux 14/13)

Maintenir une copie locale de l'annuaire PPF pour le routage offline et performant (voir `docs/annuaire.md`).

- [x] Parser XML F14 streaming (crate `pdp-annuaire`, quick-xml, 10 Go en 81s)
- [x] Stockage PostgreSQL (5 tables, batch insert, index routage)
- [x] Résolution de routage locale 4 mailles (suffixe > code routage > SIRET > SIREN)
- [x] CLI import (`pdp annuaire import <fichier>`)
- [x] CLI consultation (`pdp annuaire stats/lookup/route`)
- [x] API Directory Service conforme AFNOR XP Z12-013 Annexe B
- [x] PostgreSQL dans docker-compose, config `database` dans PdpConfig
- [x] Tests unitaires (7) + test intégration fichier réel 10 Go
- [ ] Consumer SFTP F14 (récupération automatique tar.gz depuis le PPF)
- [ ] Application du flux différentiel quotidien (24h)
- [ ] Émetteur F13 (actualisation des lignes d'annuaire de nos clients)
- [ ] Traitement CDV F6 annuaire (statuts 400 Acceptée / 401 Rejetée)

### 3. Nettoyage du répertoire specs/

Le répertoire `specs/` contient ~13 MB de duplication et des versions multiples inutilisées.

- [x] Supprimer `specs/xsd/specs-externes-v3.1/` (doublon, -6.8 MB)
- [x] Supprimer `specs/xsd/e-invoicing/` (doublon de cii+ubl, -6.7 MB)
- [x] Supprimer les anciennes versions Schematron/XSLT EN16931 (1.3.14.2, 1.3.15)
- [x] Supprimer les variantes CDAR inutilisées (d22b-uncoupled, d23b, d23b-uncoupled)
- [x] Renommer tous les répertoires avec numéros de version
- [x] Mettre à jour tous les chemins dans le code (xsd.rs, schematron.rs)
- [ ] Vérifier que tous les tests passent après nettoyage

### 4. Document d'architecture globale

Créer un vrai document d'architecture système (pas juste la liste des crates).

- [ ] Nicolas décrit sa vision de l'architecture cible
- [ ] Composants et leur déploiement (mono-binaire vs micro-services)
- [ ] Infrastructure (stockage, messaging, monitoring)
- [ ] Diagrammes de flux de données
- [ ] Séparation des responsabilités

## Moyenne priorité

### 5. Autorisation et déclaration des tenants

Actuellement les tenants sont auto-configurés (juste un répertoire SIREN suffit). Il faudra vérifier qu'un tenant est autorisé à utiliser la PDP.

- [ ] Accord formel de choix de plateforme (mandat signé)
- [ ] Vérification de l'habilitation avant traitement
- [ ] Enregistrement dans l'annuaire PPF (F13) lors de l'onboarding
- [ ] Workflow de changement de PDP (clôture des lignes de l'ancienne PA)

### 6. Rate limiting HTTP

- [ ] Limiter le nombre de requêtes par tenant/token
- [ ] Réponse 429 Too Many Requests avec Retry-After
- [ ] Configuration par tenant ou globale

### 7. E-reporting (Flux 10)

- [ ] Modèle de données pour transactions et paiements
- [ ] Sérialisation au format spécifique PPF
- [ ] Règles BR-FR-MAP-23 (conversion dates UBL → CII)
- [ ] Tests avec exemples officiels

### 8. Abstraction object store

SFTP comme couche mince vers un object store (S3/MinIO).

- [ ] Interface `ObjectStore` (put, get, list, delete)
- [ ] Implémentation filesystem (actuelle)
- [ ] Implémentation S3/MinIO
- [ ] Le protocole SFTP sauvegarde dans l'object store au lieu du filesystem
- [ ] Les répertoires tenant `{siren}/in/` et `{siren}/out/` deviennent des préfixes S3

### 12. Convention de nommage fichiers CDAR et factures

Revoir et formaliser la convention de nommage pour les fichiers CDAR et les factures (identifiants de documents, noms de fichiers retour, nommage SFTP). À discuter avec Nicolas.

- [ ] Définir la convention pour les noms de fichiers CDAR retournés (`CDV_{id}.xml`)
- [ ] Définir la convention pour les noms de fichiers factures (entrée/sortie)
- [ ] Aligner le `document_id` (MDT-4) et le `document_name` (MDT-5) avec les specs AFNOR
- [ ] Documenter les conventions dans `docs/cdar.md`

## Basse priorité

### 9. Réécriture Oxalis (Access Point Peppol en Rust)

Remplacer le gateway Java Oxalis par une implémentation Rust intégrée (voir `docs/peppol.md`).

- [ ] Implémentation AS4 (SOAP 1.2, ebMS 3.0, MIME multipart)
- [ ] WS-Security (XML-DSIG RSA-SHA256, BinarySecurityToken)
- [ ] PKI Peppol (validation chaîne de certificats, CRL)
- [ ] Enregistrement SMP (publication des capacités de réception)
- [ ] Receipts et signaux d'erreur AS4
- [ ] Retry avec backoff exponentiel
- [ ] Déduplication des messages (MessageId, 7 jours)
- [ ] Migration progressive (shadow → canary → principal → décommissionnement Oxalis)
- [ ] Tests d'interopérabilité avec Oxalis et phase4

### 10. Factur-X BASIC WL → structuré

- [ ] Génération de lignes à partir de la ventilation TVA (toléré jusqu'au 01/09/2027)
- [ ] Marquage du document comme converti

### 11. Interface d'administration

- [ ] Dashboard de suivi des factures et statuts CDV
- [ ] Consultation des logs et erreurs de validation
- [ ] Gestion des tenants (ajout, suppression, configuration)
- [ ] Suivi des alertes critiques
