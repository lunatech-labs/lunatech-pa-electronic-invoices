# Annuaire PPF — Copie locale et synchronisation

## Vue d'ensemble

L'annuaire centralisé du PPF référence **tous les assujettis à la TVA** et les structures publiques, avec leurs informations d'adressage et de routage. Le PPF en assure l'administration et le met à disposition des PDP via deux mécanismes :

1. **Consultation temps réel** — API REST PISTE (déjà implémenté dans `pdp-client/annuaire.rs`)
2. **Distribution en masse** — flux F14 via SFTP (à implémenter)

La PDP **doit** maintenir une copie locale de l'annuaire pour :
- Router les factures rapidement sans dépendre de la disponibilité de l'API PISTE
- Avoir une vue exhaustive de tous les assujettis (millions d'entrées)
- Permettre le routage offline en cas de panne du PPF

## Flux échangés

```
                  F14 (Export annuaire)                F13 (Actualisation)
                  hebdo complet + quotidien diff       modifications de nos clients
    PPF ─────────────────────────────────────▶ PDP ────────────────────────────▶ PPF
                  via SFTP tar.gz                      via SFTP tar.gz ou API
                  code: FFE1435A                       code: FFE1235A

                  F6 Annuaire (CDV)
                  statuts 400/401 pour les F13
    PPF ─────────────────────────────────────▶ PDP
                  via SFTP tar.gz
                  code: FFE0634A
```

### Cartographie des flux annuaire (DSE Figure 5)

```
Fournisseur                                         Acheteur
     ↑                                                  ↑
    F11 (consultation)                                  F12 (consultation)
     │                                                   │
    PAᴱ ◄──────── F14 ────────── PPF ──────── F14 ──────▶ PAᴿ
                                  ↑
                                 F13 (actualisation par PAᴿ)
                                 F6  (CDV annuaire)
```

## Flux F14 — Export de l'annuaire (PPF → PDP)

### Fréquence

| Type | Fréquence | Production | Contenu |
|------|-----------|------------|---------|
| **Complet** | Hebdomadaire | Nuit du dimanche au lundi | Toutes les données en vigueur |
| **Différentiel** | Quotidien | Toutes les 24h | Modifications des dernières 24h |

Lors du premier raccordement, un flux complet est mis à disposition avec toutes les données en vigueur à la date de constitution.

### Format

Transport : archive **tar.gz** via SFTP, nommée `FFE1435A_{CODE_APP}_{IDENTIFIANT_FLUX}.tar.gz`

Contenu : fichier XML conforme au XSD `Annuaire_Consultation_F14.xsd` :

```xml
<AnnuaireConsultationF14>
  <!-- Horodatage de production de cette extraction -->
  <HorodateProduction>20270315120000</HorodateProduction>
  <!-- Horodatage de la précédente extraction (absent si premier flux) -->
  <DernierHorodateProduction>20270314120000</DernierHorodateProduction>
  <!-- "COMPLET" ou "DIFFERENTIEL" -->
  <TypeFlux>DIFFERENTIEL</TypeFlux>

  <!-- Unités légales (entreprises) -->
  <BlocUnitesLegales>
    <UniteLegale>
      <IdInstance>1234</IdInstance>
      <MotifPresence>CREATION</MotifPresence>
      <Statut>ACTIF</Statut>
      <IdSIREN qualifiant="0002">123456789</IdSIREN>
      <Nom>DUPONT TECH SAS</Nom>
      <TypeEntite>PRIVE</TypeEntite>
      <Diffusible>OUI</Diffusible>
    </UniteLegale>
  </BlocUnitesLegales>

  <!-- Établissements -->
  <BlocEtablissements>
    <Etablissement>
      <IdInstance>5678</IdInstance>
      <MotifPresence>CREATION</MotifPresence>
      <Statut>ACTIF</Statut>
      <IdSIRET qualifiant="0009">12345678900001</IdSIRET>
      <TypeEtablissement>SIEGE</TypeEtablissement>
      <Nom>DUPONT TECH - Siège</Nom>
      <LigneAdresse1>8 rue Sainte</LigneAdresse1>
      <Localite>PARIS</Localite>
      <CP>75006</CP>
      <CodePays>FR</CodePays>
      <DonneesB2G>
        <EngagementJuridique>false</EngagementJuridique>
        <Service>false</Service>
        <EngJurServ>false</EngJurServ>
        <MOA>false</MOA>
        <MOAunique>false</MOAunique>
        <StatutMiseEnPaiement>false</StatutMiseEnPaiement>
      </DonneesB2G>
      <Diffusible>OUI</Diffusible>
    </Etablissement>
  </BlocEtablissements>

  <!-- Codes routage -->
  <BlocCodesRoutage>
    <CodeRoutage>
      <IdInstance>9012</IdInstance>
      <MotifPresence>CREATION</MotifPresence>
      <Statut>ACTIF</Statut>
      <IdSIRET qualifiant="0009">12345678900001</IdSIRET>
      <IdRoutage qualifiant="code_routage">Service juridique</IdRoutage>
      <Nom>Service juridique</Nom>
      <LigneAdresse1>8 rue Sainte</LigneAdresse1>
      <Localite>PARIS</Localite>
      <CP>75006</CP>
      <CodePays>FR</CodePays>
    </CodeRoutage>
  </BlocCodesRoutage>

  <!-- Plateformes de réception (PDP immatriculées) -->
  <BlocIdPlateformesReception>
    <IdPlateformeReception>
      <IdInstance>1</IdInstance>
      <MotifPresence>CREATION</MotifPresence>
      <Statut>ACTIF</Statut>
      <TypePlateforme>PDP</TypePlateforme>
      <Matricule>0135</Matricule>
      <IdSIREN qualifiant="0002">987654325</IdSIREN>
      <Nom>PDPA</Nom>
      <DateDebutImmatriculation>20260901</DateDebutImmatriculation>
    </IdPlateformeReception>
  </BlocIdPlateformesReception>

  <!-- Lignes d'annuaire (lien destinataire → plateforme de réception) -->
  <BlocLignesAnnuaire>
    <LigneAnnuaire>
      <IdInstance>3456</IdInstance>
      <MotifPresence>CREATION</MotifPresence>
      <Nature>Definition</Nature>
      <DateEffet>
        <DateDebut>20270201</DateDebut>
        <DateFin>20271231</DateFin>
      </DateEffet>
      <InfoAdressage>
        <Identifiant>ligne-001</Identifiant>
        <IdLinSIREN qualifiant="0002">123456789</IdLinSIREN>
        <IdLinSIRET qualifiant="0009">12345678900001</IdLinSIRET>
      </InfoAdressage>
      <IdPlateforme>0135</IdPlateforme>
    </LigneAnnuaire>
  </BlocLignesAnnuaire>
</AnnuaireConsultationF14>
```

### Structure d'une ligne d'annuaire (DSE Figure 23)

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Ligne d'annuaire                                                          │
├──────────── DESTINATAIRE ──────────────┬── PLATEFORME ──┬── VALIDITÉ ─────┤
│ SIREN  SIRET  Id.routage  Suffixe      │ Matricule Nature│ Début  Fin eff.│
│ 123456789  00001  Service A  -          │ 0135  Définition│ 01/02  31/12   │
└────────────────────────────────────────┴─────────────────┴────────────────┘
```

### Mailles d'adressage

L'annuaire supporte 4 niveaux de granularité pour le routage :

| Maille | Champs renseignés | Exemple |
|--------|-------------------|---------|
| **SIREN** (unité légale) | SIREN seul | Toutes les factures pour 123456789 → PDP 0135 |
| **SIRET** (établissement) | SIREN + SIRET | Factures pour l'établissement 12345678900001 → PDP 0135 |
| **Code routage** (service) | SIREN + SIRET + Id.routage | Factures pour "Service juridique" → PDP 0135 |
| **Suffixe** (adresse réseau) | SIREN + Suffixe | Factures avec suffixe ABCD01 → PDP 0135 |

### Nature des lignes

| Nature | Rôle |
|--------|------|
| **Définition** | Ligne active — porte les informations d'adressage et de routage |
| **Masquage** | Annule une ligne de Définition (ligne dont le début d'effet est postérieur à un événement) |

### Matricules spéciaux

| Matricule | Signification |
|-----------|---------------|
| `0000` | PPF (Portail Public de Facturation) |
| `9998` | Plateforme fictive (entreprise sans PDP — par défaut à l'initialisation) |
| `9999` | Chorus Pro (structures publiques) |
| `0001`-`9997` | PDP immatriculées |

## Flux F13 — Actualisation de l'annuaire (PDP → PPF)

La PDP de réception (PAᴿ) peut modifier les lignes d'annuaire de ses clients :
- **Actualiser** les lignes existantes (changer la maille, les dates)
- **Ajouter** des lignes (maille SIRET, code routage, suffixe)
- **Créer** des codes routage
- **Masquer** des lignes (mettre fin à une ligne, annuler une ligne future)

### Canaux d'actualisation

| Canal | Méthode | Détail |
|-------|---------|--------|
| **EDI** (SFTP) | Flux XML F13 | Archive tar.gz `FFE1235A_...tar.gz`, XSD `Annuaire_Actualisation_F12-F13.xsd` |
| **API** | REST PISTE | POST/PUT/PATCH/DELETE sur les ressources `code_routage` et `ligne_annuaire` |

### Format XML F13 (actualisation)

```xml
<AnnuaireActualisation>
  <!-- Codes routage à créer/modifier -->
  <BlocCodesRoutage>
    <CodeRoutage>
      <Statut>ACTIF</Statut>
      <IdSIRET qualifiant="0009">12345678900001</IdSIRET>
      <IdRoutage qualifiant="code_routage">Service B</IdRoutage>
      <Nom>Service B - Prestations</Nom>
      <LigneAdresse1>10 avenue des Champs</LigneAdresse1>
      <Localite>PARIS</Localite>
      <CP>75008</CP>
      <CodePays>FR</CodePays>
    </CodeRoutage>
  </BlocCodesRoutage>

  <!-- Lignes d'annuaire à créer/modifier -->
  <BlocLignesAnnuaire>
    <LigneAnnuaire>
      <Nature>Definition</Nature>
      <DateEffet>
        <DateDebut>20270901</DateDebut>
        <DateFin>20271231</DateFin>
      </DateEffet>
      <InfoAdressage>
        <Identifiant>
          <IdLinSIREN qualifiant="0002">123456789</IdLinSIREN>
          <IdLinSIRET qualifiant="0009">12345678900001</IdLinSIRET>
          <IdLinRoutage qualifiant="code_routage">Service B</IdLinRoutage>
        </Identifiant>
      </InfoAdressage>
      <IdPlateforme>0135</IdPlateforme>
    </LigneAnnuaire>
  </BlocLignesAnnuaire>
</AnnuaireActualisation>
```

### Cycle de vie des actualisations

Le PPF contrôle chaque objet métier (ligne d'annuaire) transmis dans un F13 :

```
PDP ──F13──▶ PPF : Contrôles techniques → Contrôles applicatifs → Contrôles fonctionnels
                                                                         │
                                                              ┌──────────┴──────────┐
                                                              ▼                     ▼
                                                        400 Acceptée           401 Rejetée
                                                              │                     │
                                                      PPF ──F6──▶ PDP        PPF ──F6──▶ PDP
                                                      (CDV annuaire)          (CDV annuaire)
```

| Statut | Code | Description |
|--------|------|-------------|
| Acceptée | 400 | La ligne d'annuaire est conforme et intégrée |
| Rejetée | 401 | La ligne d'annuaire est non conforme et rejetée |

### Motifs de rejet

| Code | Libellé | Description |
|------|---------|-------------|
| `REJ_RG` | Règles de gestion | Règles métier non respectées |
| `REJ_HAB` | Habilitations | Requête non autorisée (PDP pas habilitée pour ce SIREN) |
| `REJ_COH` | Cohérence des données | Données incohérentes (dates contradictoires, etc.) |
| `REJ_VAL_INC` | Valeurs incorrectes | Valeurs non autorisées (matricule inconnu, etc.) |

## Copie locale de l'annuaire

### Pourquoi une copie locale ?

L'API PISTE permet des lookups unitaires, mais :
- Elle ne permet pas de connaître **tous** les assujettis (pas de listing exhaustif)
- Les lookups unitaires en temps réel ajoutent de la latence au routage
- En cas d'indisponibilité du PPF, le routage serait bloqué
- Le volume (~11 millions d'assujettis, ~40 millions d'établissements) impose un cache local

### Implémentation à réaliser

#### 1. Consumer SFTP F14

Récupère les archives tar.gz F14 déposées par le PPF sur notre espace SFTP :

```
ppf-sftp-inbox/
  └── FFE1435A_{CODE_APP}_{ID_FLUX}.tar.gz   ← déposé par le PPF
```

- Polling régulier (toutes les heures) ou notification
- Extraction du tar.gz → fichier XML F14
- Validation XSD contre `Annuaire_Consultation_F14.xsd`

#### 2. Parser XML F14

Parser les 5 blocs du F14 :
- `BlocUnitesLegales` → table `unites_legales`
- `BlocEtablissements` → table `etablissements`
- `BlocCodesRoutage` → table `codes_routage`
- `BlocIdPlateformesReception` → table `plateformes`
- `BlocLignesAnnuaire` → table `lignes_annuaire`

#### 3. Stockage local

Base de données pour stocker l'annuaire. Options :

| Option | Avantages | Inconvénients |
|--------|-----------|---------------|
| **SQLite** | Simple, embarqué, pas de serveur | Limité en concurrent writes |
| **PostgreSQL** | Robuste, concurrent, full-text | Infrastructure supplémentaire |
| **Elasticsearch** | Déjà en place pour la traçabilité | Pas idéal pour du CRUD transactionnel |

Schéma simplifié (SQLite/PostgreSQL) :

```sql
CREATE TABLE unites_legales (
    id_instance     INTEGER PRIMARY KEY,
    siren           CHAR(9) NOT NULL,
    nom             TEXT NOT NULL,
    type_entite     TEXT NOT NULL,  -- PRIVE, PUBLIC
    statut          TEXT NOT NULL,  -- ACTIF, INACTIF
    diffusible      BOOLEAN NOT NULL,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(siren)
);

CREATE TABLE etablissements (
    id_instance         INTEGER PRIMARY KEY,
    siret               CHAR(14) NOT NULL,
    siren               CHAR(9) NOT NULL,
    type_etablissement  TEXT,
    nom                 TEXT NOT NULL,
    adresse_1           TEXT,
    adresse_2           TEXT,
    adresse_3           TEXT,
    localite            TEXT,
    code_postal         TEXT,
    code_pays           TEXT DEFAULT 'FR',
    engagement_juridique BOOLEAN DEFAULT FALSE,
    service             BOOLEAN DEFAULT FALSE,
    moa                 BOOLEAN DEFAULT FALSE,
    diffusible          BOOLEAN NOT NULL,
    updated_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(siret)
);

CREATE TABLE codes_routage (
    id_instance     INTEGER PRIMARY KEY,
    siret           CHAR(14) NOT NULL,
    id_routage      TEXT NOT NULL,
    nom             TEXT NOT NULL,
    statut          TEXT NOT NULL,
    adresse_1       TEXT,
    localite        TEXT,
    code_postal     TEXT,
    code_pays       TEXT DEFAULT 'FR',
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(siret, id_routage)
);

CREATE TABLE plateformes (
    id_instance             INTEGER PRIMARY KEY,
    matricule               CHAR(4) NOT NULL,
    siren                   CHAR(9),
    nom                     TEXT NOT NULL,
    nom_commercial          TEXT,
    type_plateforme         TEXT NOT NULL,  -- PDP, PPF
    date_debut_immat        DATE NOT NULL,
    date_fin_immat          DATE,
    updated_at              TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(matricule)
);

CREATE TABLE lignes_annuaire (
    id_instance     INTEGER PRIMARY KEY,
    siren           CHAR(9) NOT NULL,
    siret           CHAR(14),
    id_routage      TEXT,
    suffixe         TEXT,
    matricule       CHAR(4) NOT NULL,
    nature          TEXT NOT NULL,  -- Definition, Masquage
    date_debut      DATE NOT NULL,
    date_fin        DATE,
    date_fin_effective DATE,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index pour le routage rapide
CREATE INDEX idx_lignes_siren ON lignes_annuaire(siren);
CREATE INDEX idx_lignes_siret ON lignes_annuaire(siret);
CREATE INDEX idx_lignes_matricule ON lignes_annuaire(matricule);
CREATE INDEX idx_lignes_dates ON lignes_annuaire(date_debut, date_fin_effective);
```

#### 4. Application des flux

**Flux complet (hebdomadaire)** :
1. Charger le XML F14 complet
2. Remplacer intégralement les tables (`TRUNCATE` + `INSERT`)
3. Mettre à jour l'horodatage de dernière sync

**Flux différentiel (quotidien)** :
1. Charger le XML F14 différentiel
2. Vérifier `DernierHorodateProduction` correspond à notre dernière sync
3. Pour chaque entrée selon `MotifPresence` :
   - `CREATION` → `INSERT`
   - `MODIFICATION` → `UPDATE`
   - `SUPPRESSION` → `DELETE` ou marquage inactif
4. Mettre à jour l'horodatage

#### 5. Résolution de routage (consultation locale)

Remplacer les appels API PISTE par des requêtes locales :

```rust
/// Résout la plateforme de réception pour un destinataire.
/// Cherche la ligne d'annuaire la plus spécifique en vigueur.
fn resolve_routing(
    db: &AnnuaireDb,
    buyer_siren: &str,
    buyer_siret: Option<&str>,
    code_routage: Option<&str>,
    suffixe: Option<&str>,
    date: NaiveDate,  // date de la facture
) -> Option<RoutingResult> {
    // Ordre de priorité (plus spécifique en premier) :
    // 1. Suffixe
    // 2. Code routage (SIREN + SIRET + Id.routage)
    // 3. SIRET (SIREN + SIRET)
    // 4. SIREN seul
    //
    // Filtrer : nature=Definition, date_debut <= date, date_fin_effective is null ou >= date
}
```

#### 6. Émetteur F13 (actualisation)

Quand la PDP modifie les lignes d'annuaire de ses clients :
1. Construire le XML F13 (`AnnuaireActualisation`)
2. Valider contre le XSD `Annuaire_Actualisation_F12-F13.xsd`
3. Empaqueter en tar.gz avec le nommage `FFE1235A_{CODE_APP}_{ID_FLUX}.tar.gz`
4. Envoyer via SFTP au PPF
5. Attendre le CDV F6 (statut 400 Acceptée ou 401 Rejetée)
6. Si rejeté, alerter et stocker le motif de rejet

## XSD de référence

| Fichier | Usage |
|---------|-------|
| `specs/xsd/specs-externes-v3.1/annuaire/common/Annuaire_Commun.xsd` | Types partagés (SIREN, SIRET, lignes, codes routage, plateformes) |
| `specs/xsd/specs-externes-v3.1/annuaire/actualisation/Annuaire_Actualisation_F12-F13.xsd` | Flux d'actualisation F12/F13 (PDP → PPF) |
| `specs/xsd/specs-externes-v3.1/annuaire/consultation/Annuaire_Consultation_F14.xsd` | Export/consultation F14 (PPF → PDP) |

## Accord formel de choix de plateforme

Avant qu'une PDP puisse actualiser les lignes d'annuaire d'un client, celui-ci doit signer un **accord formel** (DSE Figure 20) désignant :
1. L'assujetti (SIREN)
2. La PDP désignée (SIREN + matricule)
3. La date de prise d'effet
4. Le périmètre des adresses de réception (SIREN, SIRET, suffixes)
5. L'éventuelle ancienne PDP
6. Un numéro de mandat unique

Cet accord est conservé par la PDP et peut être demandé par l'administration en cas de contrôle.
