//! Modèles de données de l'annuaire PPF (Flux F14 / F13)
//!
//! Codes abrégés conformes au format réel PPF :
//! - MotifPresence : C (Création), M (Modification), S (Suppression)
//! - Statut : A (Actif), I (Inactif)
//! - TypeEntite : A (Assujetti privé), P (Public)
//! - Diffusible : O (Oui), P (Partiel), N (Non)
//! - Nature : D (Définition), M (Masquage)
//! - TypeEtablissement : S (Siège), E (Établissement secondaire)
//! - TypeFlux : C (Complet), D (Différentiel)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeFlux {
    Complet,
    Differentiel,
}

impl TypeFlux {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "C" | "COMPLET" => Some(Self::Complet),
            "D" | "DIFFERENTIEL" => Some(Self::Differentiel),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotifPresence {
    Creation,
    Modification,
    Suppression,
}

impl MotifPresence {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "C" | "CREATION" => Some(Self::Creation),
            "M" | "MODIFICATION" => Some(Self::Modification),
            "S" | "SUPPRESSION" => Some(Self::Suppression),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Statut {
    Actif,
    Inactif,
}

impl Statut {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "A" | "ACTIF" => Some(Self::Actif),
            "I" | "INACTIF" => Some(Self::Inactif),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Actif => "A",
            Self::Inactif => "I",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeEntite {
    /// A = Assujetti (personne morale)
    Assujetti,
    /// P = Personne physique
    PersonnePhysique,
}

impl TypeEntite {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "A" | "PRIVE" => Some(Self::Assujetti),
            "P" | "PUBLIC" => Some(Self::PersonnePhysique),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Assujetti => "A",
            Self::PersonnePhysique => "P",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diffusible {
    Oui,
    Partiel,
    Non,
}

impl Diffusible {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "O" | "OUI" => Some(Self::Oui),
            "P" | "PARTIEL" => Some(Self::Partiel),
            "N" | "NON" => Some(Self::Non),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatureLigne {
    Definition,
    Masquage,
}

impl NatureLigne {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "D" | "Definition" => Some(Self::Definition),
            "M" | "Masquage" => Some(Self::Masquage),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Definition => "D",
            Self::Masquage => "M",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeEtablissement {
    /// S = Siège
    Siege,
    /// P = Principal (établissement principal, valeur dominante dans le fichier PPF)
    Principal,
    /// E = Établissement secondaire
    Secondaire,
}

impl TypeEtablissement {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "S" | "SIEGE" => Some(Self::Siege),
            "P" | "PRINCIPAL" => Some(Self::Principal),
            "E" | "SECONDAIRE" => Some(Self::Secondaire),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Siege => "S",
            Self::Principal => "P",
            Self::Secondaire => "E",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypePlateforme {
    Pdp,
    Ppf,
    /// AP = Access Point (PEPPOL)
    AccessPoint,
    /// NA = Non Applicable
    NonApplicable,
}

impl TypePlateforme {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "PDP" => Some(Self::Pdp),
            "PPF" => Some(Self::Ppf),
            "AP" => Some(Self::AccessPoint),
            "NA" => Some(Self::NonApplicable),
            _ => None,
        }
    }
}

// --- Structures de données ---

/// En-tête du flux F14
#[derive(Debug, Clone)]
pub struct F14Header {
    pub horodate_production: String,
    pub dernier_horodate_production: Option<String>,
    pub type_flux: TypeFlux,
}

/// Unité légale (entreprise identifiée par SIREN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniteLegale {
    pub id_instance: i64,
    pub motif_presence: MotifPresence,
    pub statut: Statut,
    pub siren: String,
    pub nom: String,
    pub type_entite: TypeEntite,
    pub diffusible: Diffusible,
}

/// Établissement (identifié par SIRET)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Etablissement {
    pub id_instance: i64,
    pub motif_presence: MotifPresence,
    pub statut: Statut,
    pub siret: String,
    pub type_etablissement: TypeEtablissement,
    pub nom: String,
    pub adresse_1: Option<String>,
    pub adresse_2: Option<String>,
    pub adresse_3: Option<String>,
    pub localite: Option<String>,
    pub code_postal: Option<String>,
    pub subdivision_pays: Option<String>,
    pub code_pays: Option<String>,
    pub donnees_b2g: Option<DonneesB2G>,
    pub diffusible: Diffusible,
}

impl Etablissement {
    /// Extrait le SIREN (9 premiers caractères du SIRET)
    pub fn siren(&self) -> &str {
        &self.siret[..9]
    }
}

/// Données B2G pour les établissements publics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonneesB2G {
    pub engagement_juridique: bool,
    pub service: bool,
    pub eng_jur_serv: bool,
    pub moa: bool,
    pub moa_unique: bool,
    pub statut_mise_en_paiement: bool,
}

/// Code routage (service au sein d'un établissement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRoutage {
    pub id_instance: i64,
    pub motif_presence: MotifPresence,
    pub statut: Statut,
    pub siret: String,
    pub id_routage: String,
    pub qualifiant_routage: String,
    pub nom: String,
    pub adresse_1: Option<String>,
    pub adresse_2: Option<String>,
    pub adresse_3: Option<String>,
    pub localite: Option<String>,
    pub code_postal: Option<String>,
    pub subdivision_pays: Option<String>,
    pub code_pays: Option<String>,
    pub engagement_juridique: Option<bool>,
}

impl CodeRoutage {
    pub fn siren(&self) -> &str {
        &self.siret[..9]
    }
}

/// Plateforme de réception (PDP immatriculée ou PPF)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plateforme {
    pub id_instance: i64,
    pub motif_presence: MotifPresence,
    pub statut: Statut,
    pub type_plateforme: TypePlateforme,
    pub matricule: String,
    pub siren: Option<String>,
    pub nom: String,
    pub nom_commercial: Option<String>,
    pub contact: Option<String>,
    pub date_debut_immatriculation: String,
    pub date_fin_immatriculation: Option<String>,
}

/// Ligne d'annuaire (lien destinataire → plateforme de réception)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LigneAnnuaire {
    pub id_instance: i64,
    pub motif_presence: MotifPresence,
    pub nature: NatureLigne,
    pub date_debut: String,
    pub date_fin: Option<String>,
    pub date_fin_effective: Option<String>,
    pub identifiant: String,
    pub siren: String,
    pub siret: Option<String>,
    pub id_routage: Option<String>,
    pub suffixe: Option<String>,
    pub id_plateforme: String,
}

/// Résultat de résolution de routage
#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub matricule_plateforme: String,
    pub nom_plateforme: Option<String>,
    pub type_plateforme: Option<TypePlateforme>,
    /// Maille de résolution utilisée
    pub maille: RoutingMaille,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMaille {
    /// Résolu par suffixe (plus spécifique)
    Suffixe,
    /// Résolu par code routage (SIREN + SIRET + code)
    CodeRoutage,
    /// Résolu par SIRET
    Siret,
    /// Résolu par SIREN (moins spécifique)
    Siren,
}

/// Statistiques d'un import F14
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub unites_legales: usize,
    pub etablissements: usize,
    pub codes_routage: usize,
    pub plateformes: usize,
    pub lignes_annuaire: usize,
    pub errors: usize,
}

/// Helper pour parser les dates PPF (format YYYYMMDD)
pub fn parse_date_ppf(s: &str) -> Option<NaiveDate> {
    if s.len() != 8 {
        return None;
    }
    NaiveDate::parse_from_str(s, "%Y%m%d").ok()
}
