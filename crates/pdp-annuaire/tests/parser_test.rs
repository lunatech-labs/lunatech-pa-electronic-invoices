//! Tests du parser F14 avec l'extrait réel de l'annuaire PPF

use pdp_annuaire::model::*;
use pdp_annuaire::parser::{parse_f14, F14Event};
use std::io::BufReader;

fn fixture_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/annuaire/F14_extrait_test.xml")
}

#[test]
fn test_parse_f14_extrait_reel() {
    let file = std::fs::File::open(fixture_path()).expect("Fixture F14 introuvable");
    let reader = BufReader::new(file);

    let mut header: Option<F14Header> = None;
    let mut unites_legales = Vec::new();
    let mut etablissements = Vec::new();
    let mut codes_routage = Vec::new();
    let mut plateformes = Vec::new();
    let mut lignes_annuaire = Vec::new();

    let stats = parse_f14(reader, |event| {
        match event {
            F14Event::Header(h) => header = Some(h),
            F14Event::UniteLegale(ul) => unites_legales.push(ul),
            F14Event::Etablissement(etab) => etablissements.push(etab),
            F14Event::CodeRoutage(cr) => codes_routage.push(cr),
            F14Event::Plateforme(pf) => plateformes.push(pf),
            F14Event::LigneAnnuaire(la) => lignes_annuaire.push(la),
        }
        Ok(())
    })
    .expect("Parsing F14 échoué");

    // Vérifier le header
    let h = header.expect("Header manquant");
    assert_eq!(h.horodate_production, "20250713010022");
    assert_eq!(h.type_flux, TypeFlux::Complet);
    assert!(h.dernier_horodate_production.is_none());

    // Vérifier les compteurs
    assert_eq!(stats.unites_legales, 10);
    assert_eq!(stats.etablissements, 10);
    assert_eq!(stats.codes_routage, 5);
    assert_eq!(stats.plateformes, 5);
    assert_eq!(stats.lignes_annuaire, 10);
    assert_eq!(stats.errors, 0);

    // Vérifier les données parsées
    assert_eq!(unites_legales.len(), 10);
    assert_eq!(etablissements.len(), 10);
    assert_eq!(codes_routage.len(), 5);
    assert_eq!(plateformes.len(), 5);
    assert_eq!(lignes_annuaire.len(), 10);
}

#[test]
fn test_parse_unite_legale_reelle() {
    let file = std::fs::File::open(fixture_path()).expect("Fixture F14 introuvable");
    let reader = BufReader::new(file);

    let mut unites_legales = Vec::new();
    parse_f14(reader, |event| {
        if let F14Event::UniteLegale(ul) = event {
            unites_legales.push(ul);
        }
        Ok(())
    })
    .unwrap();

    // Première UL : MONSIEUR THIERRY JANOYER
    let ul = &unites_legales[0];
    assert_eq!(ul.id_instance, 342);
    assert_eq!(ul.siren, "000325175");
    assert_eq!(ul.nom, "MONSIEUR THIERRY JANOYER");
    assert_eq!(ul.motif_presence, MotifPresence::Creation);
    assert_eq!(ul.statut, Statut::Actif);
    assert_eq!(ul.type_entite, TypeEntite::Assujetti);
    assert_eq!(ul.diffusible, Diffusible::Oui);

    // UL non diffusible : [ND] [ND]...
    let ul_nd = &unites_legales[4];
    assert_eq!(ul_nd.siren, "036240778");
    assert_eq!(ul_nd.diffusible, Diffusible::Partiel);

    // Dernière UL : BOULANGERIE PATISSERIE L'EPI D'OR
    let ul_last = &unites_legales[9];
    assert_eq!(ul_last.siren, "036320596");
    assert!(ul_last.nom.contains("EPI D"));
}

#[test]
fn test_parse_etablissement_reel() {
    let file = std::fs::File::open(fixture_path()).expect("Fixture F14 introuvable");
    let reader = BufReader::new(file);

    let mut etablissements = Vec::new();
    parse_f14(reader, |event| {
        if let F14Event::Etablissement(etab) = event {
            etablissements.push(etab);
        }
        Ok(())
    })
    .unwrap();

    // Premier établissement : PROCONECT
    let etab = &etablissements[0];
    assert_eq!(etab.id_instance, 16312308);
    assert_eq!(etab.siret, "99999040100070");
    assert_eq!(etab.siren(), "999990401");
    // "S" dans le fichier réel = Siège (les établissements dans l'extrait sont tous S)
    assert_eq!(etab.type_etablissement, TypeEtablissement::Siege);
    assert_eq!(etab.nom, "PROCONECT - 94800 - VILLEJUIF - 15  RUE AUGUSTE PERRET");
    assert_eq!(etab.adresse_1.as_deref(), Some("15  RUE AUGUSTE PERRET"));
    assert_eq!(etab.adresse_2.as_deref(), Some("ZAC PETITE BRUYERE 15-17"));
    assert_eq!(etab.localite.as_deref(), Some("VILLEJUIF"));
    assert_eq!(etab.code_postal.as_deref(), Some("94800"));
    assert_eq!(etab.code_pays.as_deref(), Some("FR"));
    assert_eq!(etab.diffusible, Diffusible::Oui);
    assert!(etab.donnees_b2g.is_none()); // Pas d'établissement public dans l'extrait
}

#[test]
fn test_parse_code_routage_reel() {
    let file = std::fs::File::open(fixture_path()).expect("Fixture F14 introuvable");
    let reader = BufReader::new(file);

    let mut codes_routage = Vec::new();
    parse_f14(reader, |event| {
        if let F14Event::CodeRoutage(cr) = event {
            codes_routage.push(cr);
        }
        Ok(())
    })
    .unwrap();

    // Premier code routage : FACTURES_PUBLIQUES
    let cr = &codes_routage[0];
    assert_eq!(cr.id_instance, 41666);
    assert_eq!(cr.siret, "00718051600143");
    assert_eq!(cr.id_routage, "FACTURES_PUBLIQUES");
    assert_eq!(cr.qualifiant_routage, "0224");
    assert_eq!(cr.nom, "Service des factures publiques");
    assert_eq!(cr.adresse_1.as_deref(), Some("10 ESP ANNA MARLY"));
    assert_eq!(cr.localite.as_deref(), Some("SAINT-NAZAIRE"));
    assert_eq!(cr.code_postal.as_deref(), Some("44600"));
    assert_eq!(cr.engagement_juridique, Some(false));
}

#[test]
fn test_parse_plateforme_reelle() {
    let file = std::fs::File::open(fixture_path()).expect("Fixture F14 introuvable");
    let reader = BufReader::new(file);

    let mut plateformes = Vec::new();
    parse_f14(reader, |event| {
        if let F14Event::Plateforme(pf) = event {
            plateformes.push(pf);
        }
        Ok(())
    })
    .unwrap();

    // COMARCH
    let pf = &plateformes[0];
    assert_eq!(pf.id_instance, 59);
    assert_eq!(pf.matricule, "0017");
    assert_eq!(pf.type_plateforme, TypePlateforme::Pdp);
    assert_eq!(pf.nom, "COMARCH SPOLKA AKCYJNA");
    assert_eq!(pf.nom_commercial.as_deref(), Some("COMARCH"));
    assert_eq!(pf.contact.as_deref(), Some("info@comarch.pl"));
    assert_eq!(pf.date_debut_immatriculation, "20240905");
    assert_eq!(pf.date_fin_immatriculation.as_deref(), Some("99991231"));

    // TRADESHIFT
    let pf2 = &plateformes[1];
    assert_eq!(pf2.matricule, "0036");
    assert_eq!(pf2.nom_commercial.as_deref(), Some("TRADESHIFT BABELWAY"));
}

#[test]
fn test_parse_ligne_annuaire_reelle() {
    let file = std::fs::File::open(fixture_path()).expect("Fixture F14 introuvable");
    let reader = BufReader::new(file);

    let mut lignes = Vec::new();
    parse_f14(reader, |event| {
        if let F14Event::LigneAnnuaire(la) = event {
            lignes.push(la);
        }
        Ok(())
    })
    .unwrap();

    // Première ligne : maille SIREN, plateforme fictive 9998
    let la = &lignes[0];
    assert_eq!(la.id_instance, 370952);
    assert_eq!(la.nature, NatureLigne::Definition);
    assert_eq!(la.siren, "000325175");
    assert_eq!(la.identifiant, "000325175");
    assert!(la.siret.is_none()); // Maille SIREN uniquement
    assert!(la.id_routage.is_none());
    assert!(la.suffixe.is_none());
    assert_eq!(la.id_plateforme, "9998");
    assert_eq!(la.date_debut, "20250624");
    assert!(la.date_fin.is_none());

    // Ligne avec DateFinEffective
    let la5 = &lignes[4];
    assert_eq!(la5.id_instance, 9952230);
    assert_eq!(la5.id_plateforme, "0011");
    assert_eq!(la5.date_fin_effective.as_deref(), Some("99991231"));
}

#[test]
fn test_parse_f14_vide() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AnnuaireConsultationF14>
  <HorodateProduction>20260101120000</HorodateProduction>
  <TypeFlux>D</TypeFlux>
</AnnuaireConsultationF14>"#;

    let reader = std::io::BufReader::new(xml.as_bytes());
    let mut got_header = false;

    let stats = parse_f14(reader, |event| {
        if let F14Event::Header(h) = event {
            assert_eq!(h.type_flux, TypeFlux::Differentiel);
            got_header = true;
        }
        Ok(())
    })
    .unwrap();

    assert!(got_header);
    assert_eq!(stats.unites_legales, 0);
    assert_eq!(stats.etablissements, 0);
}
