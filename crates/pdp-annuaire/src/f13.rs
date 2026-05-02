//! Générateur de flux F13 — actualisation de l'annuaire PPF.
//!
//! Le flux F13 (`FFE1235A`) est utilisé par la PDP pour pousser au PPF les
//! actualisations d'annuaire de ses clients :
//! - Création d'une nouvelle ligne (`MotifPresence = Creation`)
//! - Modification d'une ligne existante (`MotifPresence = Modification`)
//! - Suppression d'une ligne (`MotifPresence = Suppression`)
//!
//! Le PPF répond par un flux F6 annuaire (`FFE0634A`) avec un statut 400
//! (Acceptée) ou 401 (Rejetée) — voir `pdp_cdar::AnnuaireStatusCode`.
//!
//! Spécifications externes DSE AIFE V3.1, §3.4 (Flux 13).

use chrono::Utc;

use crate::model::{LigneAnnuaire, MotifPresence, NatureLigne};

/// Type d'opération F13 sur une ligne d'annuaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F13Operation {
    Creation,
    Modification,
    Suppression,
}

impl F13Operation {
    pub fn motif(&self) -> MotifPresence {
        match self {
            Self::Creation => MotifPresence::Creation,
            Self::Modification => MotifPresence::Modification,
            Self::Suppression => MotifPresence::Suppression,
        }
    }
}

/// Construit une `LigneAnnuaire` prête pour un flux F13 à partir des
/// informations métier minimales. Renseigne automatiquement le `MotifPresence`.
pub fn build_ligne_for_f13(
    operation: F13Operation,
    id_instance: i64,
    siren: &str,
    siret: Option<&str>,
    id_routage: Option<&str>,
    matricule_plateforme: &str,
    date_debut_yyyymmdd: &str,
    date_fin_yyyymmdd: Option<&str>,
) -> LigneAnnuaire {
    LigneAnnuaire {
        id_instance,
        motif_presence: operation.motif(),
        nature: NatureLigne::Definition,
        date_debut: date_debut_yyyymmdd.to_string(),
        date_fin: date_fin_yyyymmdd.map(String::from),
        date_fin_effective: None,
        identifiant: build_identifiant(siren, siret, id_routage),
        siren: siren.to_string(),
        siret: siret.map(String::from),
        id_routage: id_routage.map(String::from),
        suffixe: None,
        id_plateforme: matricule_plateforme.to_string(),
    }
}

/// Construit l'identifiant agrégé d'une ligne (SIREN + SIRET + code routage).
fn build_identifiant(siren: &str, siret: Option<&str>, id_routage: Option<&str>) -> String {
    let mut s = siren.to_string();
    if let Some(siret) = siret {
        s.push('/');
        s.push_str(siret);
    }
    if let Some(id) = id_routage {
        s.push('/');
        s.push_str(id);
    }
    s
}

/// Génère le XML F13 (actualisation annuaire) à partir d'un ensemble de
/// lignes. Le `horodate_production` est en `YYYYMMDDHHMMSS` ; si vide, on
/// utilise l'heure UTC courante.
pub fn generate_f13_xml(lignes: &[LigneAnnuaire], horodate_production: Option<&str>) -> String {
    let horodate = horodate_production
        .map(String::from)
        .unwrap_or_else(|| Utc::now().format("%Y%m%d%H%M%S").to_string());

    let mut xml = String::with_capacity(2048 + lignes.len() * 256);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str("\n<FluxF13>");
    xml.push_str("\n    <Header>");
    xml.push_str(&format!(
        "\n        <HorodateProduction>{}</HorodateProduction>",
        xml_escape(&horodate)
    ));
    // Flux F13 = différentiel uniquement
    xml.push_str("\n        <TypeFlux>Diff</TypeFlux>");
    xml.push_str("\n    </Header>");
    xml.push_str("\n    <BlocLignesAnnuaire>");
    for ligne in lignes {
        write_ligne(&mut xml, ligne);
    }
    xml.push_str("\n    </BlocLignesAnnuaire>");
    xml.push_str("\n</FluxF13>");
    xml
}

fn write_ligne(xml: &mut String, l: &LigneAnnuaire) {
    xml.push_str("\n        <LigneAnnuaire>");
    xml.push_str(&format!(
        "\n            <IdInstance>{}</IdInstance>",
        l.id_instance
    ));
    xml.push_str(&format!(
        "\n            <MotifPresence>{}</MotifPresence>",
        l.motif_presence.as_code()
    ));
    xml.push_str(&format!(
        "\n            <Nature>{}</Nature>",
        l.nature.as_code()
    ));
    xml.push_str(&format!(
        "\n            <DateDebut>{}</DateDebut>",
        xml_escape(&l.date_debut)
    ));
    if let Some(ref df) = l.date_fin {
        xml.push_str(&format!(
            "\n            <DateFin>{}</DateFin>",
            xml_escape(df)
        ));
    }
    if let Some(ref dfe) = l.date_fin_effective {
        xml.push_str(&format!(
            "\n            <DateFinEffective>{}</DateFinEffective>",
            xml_escape(dfe)
        ));
    }
    xml.push_str(&format!(
        "\n            <Identifiant>{}</Identifiant>",
        xml_escape(&l.identifiant)
    ));
    xml.push_str(&format!(
        "\n            <IdLinSIREN>{}</IdLinSIREN>",
        xml_escape(&l.siren)
    ));
    if let Some(ref s) = l.siret {
        xml.push_str(&format!(
            "\n            <IdLinSIRET>{}</IdLinSIRET>",
            xml_escape(s)
        ));
    }
    if let Some(ref r) = l.id_routage {
        xml.push_str(&format!(
            "\n            <IdLinRoutage>{}</IdLinRoutage>",
            xml_escape(r)
        ));
    }
    if let Some(ref s) = l.suffixe {
        xml.push_str(&format!(
            "\n            <Suffixe>{}</Suffixe>",
            xml_escape(s)
        ));
    }
    xml.push_str(&format!(
        "\n            <IdPlateforme>{}</IdPlateforme>",
        xml_escape(&l.id_plateforme)
    ));
    xml.push_str("\n        </LigneAnnuaire>");
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f13_operation_motif() {
        assert_eq!(F13Operation::Creation.motif(), MotifPresence::Creation);
        assert_eq!(F13Operation::Modification.motif(), MotifPresence::Modification);
        assert_eq!(F13Operation::Suppression.motif(), MotifPresence::Suppression);
    }

    #[test]
    fn test_build_ligne_creation_minimal() {
        let l = build_ligne_for_f13(
            F13Operation::Creation,
            42,
            "123456789",
            None,
            None,
            "0001",
            "20260502",
            None,
        );
        assert_eq!(l.id_instance, 42);
        assert_eq!(l.motif_presence, MotifPresence::Creation);
        assert_eq!(l.siren, "123456789");
        assert_eq!(l.identifiant, "123456789");
        assert!(l.siret.is_none());
        assert_eq!(l.id_plateforme, "0001");
        assert_eq!(l.date_debut, "20260502");
        assert!(l.date_fin.is_none());
    }

    #[test]
    fn test_build_ligne_with_siret_and_routage() {
        let l = build_ligne_for_f13(
            F13Operation::Modification,
            1,
            "123456789",
            Some("12345678901234"),
            Some("0224COMP"),
            "0001",
            "20260502",
            Some("20271231"),
        );
        assert_eq!(l.identifiant, "123456789/12345678901234/0224COMP");
        assert_eq!(l.motif_presence, MotifPresence::Modification);
        assert_eq!(l.siret.as_deref(), Some("12345678901234"));
        assert_eq!(l.id_routage.as_deref(), Some("0224COMP"));
        assert_eq!(l.date_fin.as_deref(), Some("20271231"));
    }

    #[test]
    fn test_generate_xml_empty() {
        let xml = generate_f13_xml(&[], Some("20260502120000"));
        assert!(xml.contains("<FluxF13>"));
        assert!(xml.contains("<HorodateProduction>20260502120000</HorodateProduction>"));
        assert!(xml.contains("<TypeFlux>Diff</TypeFlux>"));
        assert!(xml.contains("<BlocLignesAnnuaire>"));
        assert!(xml.contains("</FluxF13>"));
    }

    #[test]
    fn test_generate_xml_with_creation() {
        let l = build_ligne_for_f13(
            F13Operation::Creation,
            1,
            "123456789",
            Some("12345678901234"),
            None,
            "0001",
            "20260502",
            None,
        );
        let xml = generate_f13_xml(&[l], Some("20260502120000"));
        assert!(xml.contains("<MotifPresence>C</MotifPresence>"));
        assert!(xml.contains("<IdLinSIREN>123456789</IdLinSIREN>"));
        assert!(xml.contains("<IdLinSIRET>12345678901234</IdLinSIRET>"));
        assert!(xml.contains("<IdPlateforme>0001</IdPlateforme>"));
        assert!(!xml.contains("<DateFin>"));
    }

    #[test]
    fn test_generate_xml_with_suppression() {
        let l = build_ligne_for_f13(
            F13Operation::Suppression,
            7,
            "987654321",
            None,
            None,
            "0002",
            "20260101",
            Some("20261231"),
        );
        let xml = generate_f13_xml(&[l], Some("20260502120000"));
        assert!(xml.contains("<MotifPresence>S</MotifPresence>"));
        assert!(xml.contains("<DateFin>20261231</DateFin>"));
    }

    #[test]
    fn test_xml_escapes_special_chars() {
        let mut l = build_ligne_for_f13(
            F13Operation::Creation,
            1,
            "123456789",
            None,
            None,
            "0001",
            "20260502",
            None,
        );
        // Forcer un identifiant avec caractères spéciaux
        l.identifiant = "test<>&'\"".to_string();
        let xml = generate_f13_xml(&[l], Some("20260502120000"));
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;"));
        assert!(!xml.contains("test<>"));
    }

    #[test]
    fn test_generated_xml_well_formed() {
        // Vérifie que le XML F13 est bien formé : balises ouvrantes/fermantes
        // équilibrées et déclaration XML correcte.
        let l = build_ligne_for_f13(
            F13Operation::Creation,
            1,
            "123456789",
            Some("12345678901234"),
            None,
            "0001",
            "20260502",
            None,
        );
        let xml = generate_f13_xml(&[l], Some("20260502120000"));

        // Déclaration XML
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        // Racine ouverte/fermée
        assert_eq!(xml.matches("<FluxF13>").count(), 1);
        assert_eq!(xml.matches("</FluxF13>").count(), 1);
        // Bloc lignes ouvert/fermé
        assert_eq!(xml.matches("<BlocLignesAnnuaire>").count(), 1);
        assert_eq!(xml.matches("</BlocLignesAnnuaire>").count(), 1);
        // Une seule ligne
        assert_eq!(xml.matches("<LigneAnnuaire>").count(), 1);
        assert_eq!(xml.matches("</LigneAnnuaire>").count(), 1);
    }
}
