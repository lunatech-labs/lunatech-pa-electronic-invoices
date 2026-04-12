//! Test d'intégration : parsing du vrai annuaire PPF (10 Go)
//!
//! Ce test est ignoré par défaut car il nécessite le fichier réel.
//! Pour le lancer :
//!   cargo test -p pdp-annuaire --test integration_test -- --ignored

use pdp_annuaire::model::*;
use pdp_annuaire::parser::{parse_f14, F14Event};
use std::io::BufReader;
use std::time::Instant;

const REAL_F14_PATH: &str = "/Users/nicolas/Downloads/ppf-annuaire-export-full-20250713";

#[test]
#[ignore]
fn test_parse_vrai_annuaire_10go() {
    let file = std::fs::File::open(REAL_F14_PATH)
        .expect("Fichier annuaire PPF introuvable — télécharger le fichier F14 réel");
    // Buffer de 8 Mo pour le streaming
    let reader = BufReader::with_capacity(8 * 1024 * 1024, file);

    let start = Instant::now();
    let mut header: Option<F14Header> = None;
    let mut sample_ul: Option<UniteLegale> = None;
    let mut sample_pf: Vec<Plateforme> = Vec::new();
    let mut pf_count = 0usize;

    let stats = parse_f14(reader, |event| {
        match event {
            F14Event::Header(h) => header = Some(h),
            F14Event::UniteLegale(ref ul) if sample_ul.is_none() => {
                sample_ul = Some(ul.clone());
            }
            F14Event::Plateforme(ref pf) => {
                pf_count += 1;
                if sample_pf.len() < 5 {
                    sample_pf.push(pf.clone());
                }
            }
            _ => {}
        }
        // Progress logging every 1M elements
        Ok(())
    })
    .expect("Parsing du vrai F14 échoué");

    let elapsed = start.elapsed();

    println!("\n=== Résultats parsing annuaire PPF réel ===");
    println!("Durée totale : {:.1}s", elapsed.as_secs_f64());
    println!("Unités légales : {}", stats.unites_legales);
    println!("Établissements : {}", stats.etablissements);
    println!("Codes routage  : {}", stats.codes_routage);
    println!("Plateformes    : {}", stats.plateformes);
    println!("Lignes annuaire: {}", stats.lignes_annuaire);
    println!("Erreurs        : {}", stats.errors);

    let total = stats.unites_legales + stats.etablissements + stats.codes_routage
        + stats.plateformes + stats.lignes_annuaire;
    let throughput = total as f64 / elapsed.as_secs_f64();
    println!("Throughput     : {:.0} éléments/s", throughput);

    // Vérifications
    let h = header.expect("Header manquant");
    assert_eq!(h.type_flux, TypeFlux::Complet);
    assert_eq!(h.horodate_production, "20250713010022");

    // Volumes attendus (approximatifs)
    assert!(stats.unites_legales > 9_000_000, "Attendu >9M UL, obtenu {}", stats.unites_legales);
    assert!(stats.etablissements > 10_000_000, "Attendu >10M étab, obtenu {}", stats.etablissements);
    assert!(stats.codes_routage > 200_000, "Attendu >200K CR, obtenu {}", stats.codes_routage);
    assert!(stats.plateformes > 50, "Attendu >50 PF, obtenu {}", stats.plateformes);
    assert!(stats.lignes_annuaire > 9_000_000, "Attendu >9M LA, obtenu {}", stats.lignes_annuaire);

    // Échantillon
    if let Some(ul) = sample_ul {
        println!("\nPremière UL : {} (SIREN {})", ul.nom, ul.siren);
    }
    println!("Plateformes (5 premières) :");
    for pf in &sample_pf {
        println!("  {} — {} ({})", pf.matricule, pf.nom, pf.nom_commercial.as_deref().unwrap_or("-"));
    }

    // Taux d'erreur acceptable (< 0.1% — le fichier PPF réel contient quelques entrées corrompues)
    let error_rate = stats.errors as f64 / total as f64;
    println!("Taux d'erreur : {:.4}%", error_rate * 100.0);
    assert!(error_rate < 0.001, "Taux d'erreur trop élevé : {:.4}%", error_rate * 100.0);
}
