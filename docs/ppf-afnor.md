# Communication PPF et AFNOR

## Communication PPF (Portail Public de Facturation)

### Dépôt de flux — SFTP tar.gz

La communication PDP → PPF se fait exclusivement par **SFTP** (pas d'API REST).

- **Protocole** : SFTP avec authentification par clé RSA (certificat X509v3)
- **Format** : archive `tar.gz` contenant un ou plusieurs fichiers XML
- **Nommage de l'enveloppe** : `{CODE_INTERFACE}_{CODE_APP}_{IDENTIFIANT_FLUX}.tar.gz`
  - Exemple : `FFE0111A_AAA123_AAA1230111000000000000001.tar.gz`
- **Nommage des fichiers F1** : `{profil}_{nom}.xml` (profil = `Base` ou `Full`)
- **Taille max** : 1 Go par flux, 120 Mo par fichier

### Codes interfaces

| Flux | Format | Code interface |
|------|--------|----------------|
| F1 données réglementaires | UBL | `FFE0111A` |
| F1 données réglementaires | CII | `FFE0112A` |
| F6 CDV factures | CDAR | `FFE0614A` |
| F6 CDV statuts obligatoires | CDAR | `FFE0654A` |
| F10 transactions/paiements | Spécifique | `FFE1025A` |
| F13 actualisation annuaire | Spécifique | `FFE1235A` |

### Annuaire PPF — API REST PISTE

- Recherche SIREN/SIRET, routing codes
- Auth : OAuth2 PISTE (client_credentials, JWT Bearer)

## Communication PDP↔PDP (AFNOR XP Z12-013)

- **Flow Service** — `POST /v1/flows` (envoi factures, CDV, e-reporting entre PDP)
- **Directory Service** — Recherche SIREN, routing codes, directory lines

### Producers (endpoints de sortie)

- `PpfSftpProducer` — Construit le tar.gz avec le bon nommage et le dépose via SFTP sur le PPF
- `AfnorFlowProducer` — Envoie vers PDP distante via AFNOR Flow Service
- `FileEndpoint` — Écriture sur filesystem local
