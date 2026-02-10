//! Contrôles de réception des fichiers entrants.
//!
//! Implémente les vérifications à effectuer **avant le parsing** sur chaque
//! fichier reçu (XML ou PDF). Ces contrôles sont issus des spécifications
//! XP Z12-012 et des règles BR-FR :
//!
//! - **BR-FR-19** : taille fichier ≤ 100 Mo
//! - **Fichier non vide** : le body ne doit pas être vide
//! - **Extension reconnue** : .xml ou .pdf uniquement
//! - **Nom de fichier valide** : caractères autorisés (BR-FR-02 étendu aux noms de fichiers)
//! - **Unicité du nom** : détection de doublons (même nom déjà traité dans le même poll)

use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::error::{PdpError, PdpResult};
use crate::exchange::Exchange;
use crate::processor::Processor;

/// Taille maximale d'un fichier facture : 100 Mo (BR-FR-19)
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Extensions de fichiers acceptées
const ALLOWED_EXTENSIONS: &[&str] = &["xml", "pdf"];

/// Caractères autorisés dans un nom de fichier (BR-FR-02 étendu).
/// Lettres, chiffres, tiret, underscore, point, plus, slash.
fn is_valid_filename_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+' | '/')
}

/// Résultat d'un contrôle de réception
#[derive(Debug, Clone)]
pub struct ReceptionCheck {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
}

/// Effectue tous les contrôles de réception sur un exchange brut.
/// Retourne la liste des contrôles échoués (vide = tout OK).
pub fn check_reception(exchange: &Exchange, seen_filenames: Option<&HashSet<String>>) -> Vec<ReceptionCheck> {
    let mut failures = Vec::new();

    let filename = exchange.source_filename.as_deref().unwrap_or("");

    // 1. Fichier non vide
    if exchange.body.is_empty() {
        failures.push(ReceptionCheck {
            rule_id: "REC-01".to_string(),
            passed: false,
            message: format!("Fichier vide : '{}'", filename),
        });
    }

    // 2. Taille maximale (BR-FR-19 : 100 Mo)
    if exchange.body.len() > MAX_FILE_SIZE {
        failures.push(ReceptionCheck {
            rule_id: "BR-FR-19".to_string(),
            passed: false,
            message: format!(
                "Fichier trop volumineux : {} octets (max {} octets / 100 Mo) — '{}'",
                exchange.body.len(),
                MAX_FILE_SIZE,
                filename,
            ),
        });
    }

    // 3. Extension reconnue
    if !filename.is_empty() {
        let lower = filename.to_lowercase();
        let has_valid_ext = ALLOWED_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{}", ext)));
        if !has_valid_ext {
            failures.push(ReceptionCheck {
                rule_id: "REC-02".to_string(),
                passed: false,
                message: format!(
                    "Extension non reconnue : '{}' (attendu : {})",
                    filename,
                    ALLOWED_EXTENSIONS.join(", "),
                ),
            });
        }
    }

    // 4. Caractères autorisés dans le nom de fichier (BR-FR-02 étendu)
    if !filename.is_empty() {
        let invalid_chars: Vec<char> = filename.chars().filter(|c| !is_valid_filename_char(*c)).collect();
        if !invalid_chars.is_empty() {
            failures.push(ReceptionCheck {
                rule_id: "REC-03".to_string(),
                passed: false,
                message: format!(
                    "Caractères non autorisés dans le nom '{}' : {:?} (autorisés : A-Z, a-z, 0-9, -, _, ., +, /)",
                    filename,
                    invalid_chars,
                ),
            });
        }
    }

    // 5. Nom de fichier non vide
    if filename.is_empty() {
        failures.push(ReceptionCheck {
            rule_id: "REC-04".to_string(),
            passed: false,
            message: "Nom de fichier absent".to_string(),
        });
    }

    // 6. Unicité du nom de fichier (doublon dans le même batch)
    if let Some(seen) = seen_filenames {
        if !filename.is_empty() && seen.contains(filename) {
            failures.push(ReceptionCheck {
                rule_id: "REC-05".to_string(),
                passed: false,
                message: format!("Nom de fichier en doublon : '{}'", filename),
            });
        }
    }

    failures
}

/// Processor de contrôle de réception.
///
/// S'insère dans le pipeline **avant le parsing**. Vérifie :
/// - Fichier non vide (REC-01)
/// - Taille ≤ 100 Mo (BR-FR-19)
/// - Extension .xml ou .pdf (REC-02)
/// - Caractères autorisés dans le nom (REC-03)
/// - Nom de fichier présent (REC-04)
/// - Unicité du nom dans le batch courant (REC-05)
///
/// En cas d'échec, les erreurs sont ajoutées à l'exchange (non bloquant par défaut)
/// ou l'exchange est rejeté (mode strict).
pub struct ReceptionProcessor {
    /// Si true, un contrôle échoué provoque une erreur fatale (exchange rejeté).
    /// Si false, les erreurs sont ajoutées à l'exchange mais le traitement continue.
    strict: bool,
    /// Noms de fichiers déjà vus (pour la détection de doublons)
    seen_filenames: Mutex<HashSet<String>>,
}

impl ReceptionProcessor {
    /// Crée un processor de réception en mode strict (rejette les fichiers invalides)
    pub fn strict() -> Self {
        Self {
            strict: true,
            seen_filenames: Mutex::new(HashSet::new()),
        }
    }

    /// Crée un processor de réception en mode permissif (ajoute des warnings mais continue)
    pub fn permissive() -> Self {
        Self {
            strict: false,
            seen_filenames: Mutex::new(HashSet::new()),
        }
    }

    /// Réinitialise la liste des noms de fichiers vus (entre deux polls)
    pub fn reset_seen(&self) {
        self.seen_filenames.lock().unwrap().clear();
    }
}

#[async_trait]
impl Processor for ReceptionProcessor {
    fn name(&self) -> &str {
        "reception-check"
    }

    async fn process(&self, mut exchange: Exchange) -> PdpResult<Exchange> {
        let filename = exchange.source_filename.clone().unwrap_or_default();

        // Vérifier les doublons avec le set partagé
        let seen = self.seen_filenames.lock().unwrap().clone();
        let failures = check_reception(&exchange, Some(&seen));

        // Enregistrer le nom dans le set (même si invalide, pour détecter les doublons suivants)
        if !filename.is_empty() {
            self.seen_filenames.lock().unwrap().insert(filename.clone());
        }

        if failures.is_empty() {
            tracing::debug!(
                filename = %filename,
                exchange_id = %exchange.id,
                "Contrôles de réception OK"
            );
            return Ok(exchange);
        }

        // Construire le message d'erreur
        let error_messages: Vec<String> = failures.iter()
            .map(|f| format!("[{}] {}", f.rule_id, f.message))
            .collect();
        let summary = error_messages.join(" ; ");

        // Ajouter les erreurs à l'exchange
        for failure in &failures {
            exchange.add_error(
                "reception-check",
                &PdpError::ValidationError(format!("[{}] {}", failure.rule_id, failure.message)),
            );
        }

        // Marquer l'exchange comme irrecevable pour le processor CDAR en aval
        exchange.set_property("reception.failed", "true");
        exchange.set_property("reception.error_count", &failures.len().to_string());
        // Stocker les rule_ids pour le mapping vers les codes IRR_*
        let rule_ids: Vec<String> = failures.iter().map(|f| f.rule_id.clone()).collect();
        exchange.set_property("reception.rule_ids", &rule_ids.join(","));

        if self.strict {
            tracing::warn!(
                filename = %filename,
                exchange_id = %exchange.id,
                errors = %summary,
                "Fichier irrecevable — CDAR 501 sera généré en aval"
            );
        } else {
            tracing::warn!(
                filename = %filename,
                exchange_id = %exchange.id,
                errors = %summary,
                "Contrôles de réception : avertissements (mode permissif)"
            );
        }

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Tests unitaires check_reception =====

    #[test]
    fn test_check_valid_xml() {
        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture_001.xml");
        let failures = check_reception(&exchange, None);
        assert!(failures.is_empty(), "Fichier XML valide : {:?}", failures);
    }

    #[test]
    fn test_check_valid_pdf() {
        let exchange = Exchange::new(b"%PDF-1.4".to_vec()).with_filename("facture.pdf");
        let failures = check_reception(&exchange, None);
        assert!(failures.is_empty(), "Fichier PDF valide : {:?}", failures);
    }

    #[test]
    fn test_check_empty_body() {
        let exchange = Exchange::new(Vec::new()).with_filename("vide.xml");
        let failures = check_reception(&exchange, None);
        assert!(failures.iter().any(|f| f.rule_id == "REC-01"), "Doit détecter fichier vide");
    }

    #[test]
    fn test_check_file_too_large() {
        // Créer un body de 100 Mo + 1 octet
        let body = vec![0u8; MAX_FILE_SIZE + 1];
        let exchange = Exchange::new(body).with_filename("gros.xml");
        let failures = check_reception(&exchange, None);
        assert!(failures.iter().any(|f| f.rule_id == "BR-FR-19"), "Doit détecter fichier trop gros");
    }

    #[test]
    fn test_check_file_exactly_max_size() {
        let body = vec![0u8; MAX_FILE_SIZE];
        let exchange = Exchange::new(body).with_filename("exact.xml");
        let failures = check_reception(&exchange, None);
        assert!(!failures.iter().any(|f| f.rule_id == "BR-FR-19"), "100 Mo exact doit passer");
    }

    #[test]
    fn test_check_bad_extension() {
        let exchange = Exchange::new(b"data".to_vec()).with_filename("facture.csv");
        let failures = check_reception(&exchange, None);
        assert!(failures.iter().any(|f| f.rule_id == "REC-02"), "Doit rejeter .csv");
    }

    #[test]
    fn test_check_no_extension() {
        let exchange = Exchange::new(b"data".to_vec()).with_filename("facture");
        let failures = check_reception(&exchange, None);
        assert!(failures.iter().any(|f| f.rule_id == "REC-02"), "Doit rejeter sans extension");
    }

    #[test]
    fn test_check_invalid_chars_in_filename() {
        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture (1).xml");
        let failures = check_reception(&exchange, None);
        assert!(
            failures.iter().any(|f| f.rule_id == "REC-03"),
            "Doit détecter espaces et parenthèses : {:?}", failures
        );
    }

    #[test]
    fn test_check_valid_special_chars() {
        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("FA-2025_001+v2.xml");
        let failures = check_reception(&exchange, None);
        assert!(failures.is_empty(), "Tirets, underscores, plus sont autorisés : {:?}", failures);
    }

    #[test]
    fn test_check_no_filename() {
        let exchange = Exchange::new(b"<Invoice/>".to_vec());
        let failures = check_reception(&exchange, None);
        assert!(failures.iter().any(|f| f.rule_id == "REC-04"), "Doit détecter nom absent");
    }

    #[test]
    fn test_check_duplicate_filename() {
        let mut seen = HashSet::new();
        seen.insert("facture.xml".to_string());

        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture.xml");
        let failures = check_reception(&exchange, Some(&seen));
        assert!(failures.iter().any(|f| f.rule_id == "REC-05"), "Doit détecter doublon");
    }

    #[test]
    fn test_check_no_duplicate() {
        let mut seen = HashSet::new();
        seen.insert("facture_001.xml".to_string());

        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture_002.xml");
        let failures = check_reception(&exchange, Some(&seen));
        assert!(!failures.iter().any(|f| f.rule_id == "REC-05"), "Noms différents = pas de doublon");
    }

    #[test]
    fn test_check_multiple_errors() {
        // Fichier vide + mauvaise extension + caractères invalides
        let exchange = Exchange::new(Vec::new()).with_filename("bad file!.csv");
        let failures = check_reception(&exchange, None);
        assert!(failures.len() >= 3, "Doit détecter au moins 3 erreurs : {:?}", failures);
        assert!(failures.iter().any(|f| f.rule_id == "REC-01"));
        assert!(failures.iter().any(|f| f.rule_id == "REC-02"));
        assert!(failures.iter().any(|f| f.rule_id == "REC-03"));
    }

    // ===== Tests du Processor =====

    #[tokio::test]
    async fn test_processor_strict_valid() {
        let processor = ReceptionProcessor::strict();
        let exchange = Exchange::new(b"<Invoice/>".to_vec()).with_filename("facture.xml");
        let result = processor.process(exchange).await;
        assert!(result.is_ok(), "Fichier valide doit passer en mode strict");
    }

    #[tokio::test]
    async fn test_processor_strict_marks_empty() {
        let processor = ReceptionProcessor::strict();
        let exchange = Exchange::new(Vec::new()).with_filename("vide.xml");
        let result = processor.process(exchange).await;
        assert!(result.is_ok(), "Le processor retourne Ok même en mode strict");
        let ex = result.unwrap();
        assert!(ex.has_errors(), "Doit avoir des erreurs enregistrées");
        assert_eq!(ex.get_property("reception.failed").map(|s| s.as_str()), Some("true"));
        assert!(ex.get_property("reception.rule_ids").unwrap().contains("REC-01"));
    }

    #[tokio::test]
    async fn test_processor_permissive_allows_empty() {
        let processor = ReceptionProcessor::permissive();
        let exchange = Exchange::new(Vec::new()).with_filename("vide.xml");
        let result = processor.process(exchange).await;
        assert!(result.is_ok(), "Fichier vide doit passer en mode permissif");
        let ex = result.unwrap();
        assert!(ex.has_errors(), "Mais doit avoir des erreurs enregistrées");
    }

    #[tokio::test]
    async fn test_processor_strict_marks_bad_extension() {
        let processor = ReceptionProcessor::strict();
        let exchange = Exchange::new(b"data".to_vec()).with_filename("facture.json");
        let result = processor.process(exchange).await;
        assert!(result.is_ok(), "Le processor retourne Ok même en mode strict");
        let ex = result.unwrap();
        assert_eq!(ex.get_property("reception.failed").map(|s| s.as_str()), Some("true"));
        assert!(ex.get_property("reception.rule_ids").unwrap().contains("REC-02"));
    }

    #[tokio::test]
    async fn test_processor_detects_duplicates() {
        let processor = ReceptionProcessor::strict();

        let ex1 = Exchange::new(b"<A/>".to_vec()).with_filename("facture.xml");
        let result1 = processor.process(ex1).await;
        assert!(result1.is_ok(), "Premier fichier doit passer");
        assert!(result1.unwrap().get_property("reception.failed").is_none());

        let ex2 = Exchange::new(b"<B/>".to_vec()).with_filename("facture.xml");
        let result2 = processor.process(ex2).await;
        assert!(result2.is_ok(), "Retourne Ok avec marquage");
        let ex2 = result2.unwrap();
        assert_eq!(ex2.get_property("reception.failed").map(|s| s.as_str()), Some("true"));
        assert!(ex2.get_property("reception.rule_ids").unwrap().contains("REC-05"));
    }

    #[tokio::test]
    async fn test_processor_reset_seen() {
        let processor = ReceptionProcessor::strict();

        let ex1 = Exchange::new(b"<A/>".to_vec()).with_filename("facture.xml");
        processor.process(ex1).await.unwrap();

        processor.reset_seen();

        let ex2 = Exchange::new(b"<B/>".to_vec()).with_filename("facture.xml");
        let result = processor.process(ex2).await;
        assert!(result.is_ok(), "Après reset, le même nom doit passer");
    }

    #[tokio::test]
    async fn test_processor_different_filenames_ok() {
        let processor = ReceptionProcessor::strict();

        let ex1 = Exchange::new(b"<A/>".to_vec()).with_filename("facture_001.xml");
        let ex2 = Exchange::new(b"<B/>".to_vec()).with_filename("facture_002.xml");
        let ex3 = Exchange::new(b"<C/>".to_vec()).with_filename("facture_003.xml");

        assert!(processor.process(ex1).await.is_ok());
        assert!(processor.process(ex2).await.is_ok());
        assert!(processor.process(ex3).await.is_ok());
    }
}
