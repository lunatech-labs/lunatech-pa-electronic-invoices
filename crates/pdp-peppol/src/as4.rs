//! Client et serveur AS4 pour le réseau PEPPOL.
//!
//! AS4 est le protocole de transport utilisé entre les Access Points PEPPOL.
//! Il repose sur SOAP 1.2 + MIME multipart + WS-Security (signature/chiffrement).
//!
//! Conforme à : Peppol AS4 Profile v2.0, CEF eDelivery AS4 v1.14
//!
//! # Architecture
//!
//! ```text
//! Envoi (Corner 2) :
//!   PeppolMessage → SBDH → SOAP Envelope + MIME → POST HTTPS → AP destinataire
//!
//! Réception (Corner 3) :
//!   POST HTTPS ← AP émetteur → MIME → SOAP → SBDH → PeppolMessage → Pipeline
//! ```

use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

use crate::error::PeppolError;
use crate::model::{As4SendResult, PeppolConfig, PeppolMessage, SmpEndpoint};
use crate::sbdh;

// ============================================================
// Constantes AS4 / PEPPOL
// ============================================================

/// Agreement Reference (PMode.Agreement)
pub const PEPPOL_AGREEMENT: &str = "urn:fdc:peppol.eu:2017:agreements:tia:ap_provider";

/// Party type identifier
pub const PEPPOL_PARTY_TYPE: &str = "urn:fdc:peppol.eu:2017:identifiers:ap";

/// MEP : One-Way/Push
pub const PEPPOL_MEP: &str = "http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/oneWay";

/// MEP Binding : Push
pub const PEPPOL_MEP_BINDING: &str = "http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/push";

/// Initiator Role
pub const PEPPOL_INITIATOR_ROLE: &str = "http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/initiator";

/// Responder Role
pub const PEPPOL_RESPONDER_ROLE: &str = "http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/responder";

/// Default MPC
pub const PEPPOL_DEFAULT_MPC: &str = "http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/defaultMPC";

/// Content-Type pour le payload SBDH compressé
pub const MIME_TYPE_SBDH: &str = "application/xml";

/// MIME boundary pour le multipart
const MIME_BOUNDARY: &str = "----=_Part_PEPPOL_AS4";

// ============================================================
// Client AS4 (envoi — Corner 2)
// ============================================================

/// Client AS4 pour l'envoi de messages PEPPOL.
///
/// Construit l'enveloppe SOAP + MIME multipart et envoie via HTTPS POST
/// vers l'Access Point destinataire.
pub struct As4Client {
    config: PeppolConfig,
    http: reqwest::Client,
}

impl As4Client {
    pub fn new(config: PeppolConfig) -> Self {
        let test_mode = config.test_mode;
        Self {
            config,
            http: reqwest::Client::builder()
                .danger_accept_invalid_certs(test_mode)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Envoie un message PEPPOL vers un endpoint AS4 découvert via SMP.
    pub async fn send(
        &self,
        message: &PeppolMessage,
        endpoint: &SmpEndpoint,
    ) -> Result<As4SendResult, PeppolError> {
        let message_id = format!("{}@{}", uuid::Uuid::new_v4(), self.config.ap_id);

        tracing::info!(
            message_id = %message_id,
            sender = %message.sender,
            receiver = %message.receiver,
            document_type = %message.document_type,
            endpoint = %endpoint.endpoint_url,
            "Envoi AS4 PEPPOL"
        );

        // 1. Construire le SBDH (enveloppe du document)
        let sbdh_xml = sbdh::build_sbdh(message);

        // 2. Extraire l'identifiant AP destinataire depuis le certificat SMP
        //    En PEPPOL, le eb:To/PartyId devrait être le CN du certificat de l'AP
        //    destinataire. On utilise le certificat du SMP endpoint, ou l'URL en fallback.
        let to_party_id = extract_ap_cn_from_cert(&endpoint.certificate)
            .unwrap_or_else(|| endpoint.endpoint_url.clone());

        // 3. Construire l'enveloppe SOAP ebMS3
        let soap_envelope = build_soap_envelope(
            &message_id,
            &self.config.ap_id,
            &to_party_id,
            &message.document_type_id.to_action(),
            &message.process_id.to_string(),
        );

        // 4. Construire le message MIME multipart (avec compression gzip du payload)
        let mime_body = build_mime_multipart(&soap_envelope, &sbdh_xml);

        // 5. Envoyer via HTTPS POST
        let response = self.http
            .post(&endpoint.endpoint_url)
            .header("Content-Type", format!("multipart/related; boundary=\"{}\"; type=\"application/soap+xml\"; start=\"<soap-envelope>\"", MIME_BOUNDARY))
            .header("SOAPAction", "")
            .body(mime_body)
            .send()
            .await
            .map_err(|e| PeppolError::As4Error(format!("Envoi AS4 échoué : {}", e)))?;

        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();

        if status.is_success() {
            tracing::info!(
                message_id = %message_id,
                status = %status,
                "Message AS4 envoyé avec succès"
            );

            Ok(As4SendResult {
                message_id,
                success: true,
                error: None,
                timestamp: Utc::now(),
                endpoint_url: endpoint.endpoint_url.clone(),
            })
        } else {
            // Parser l'erreur ebMS si présente
            let error_detail = parse_ebms_error(&response_body)
                .unwrap_or_else(|| format!("HTTP {} : {}", status, &response_body[..response_body.len().min(500)]));

            tracing::warn!(
                message_id = %message_id,
                status = %status,
                error = %error_detail,
                "Envoi AS4 échoué"
            );

            Ok(As4SendResult {
                message_id,
                success: false,
                error: Some(error_detail),
                timestamp: Utc::now(),
                endpoint_url: endpoint.endpoint_url.clone(),
            })
        }
    }
}

// ============================================================
// Serveur AS4 (réception — Corner 3)
// ============================================================

/// Résultat du parsing d'un message AS4 entrant.
#[derive(Debug, Clone)]
pub struct As4IncomingMessage {
    /// Message ID ebMS
    pub message_id: String,
    /// SBDH parsé (contient sender, receiver, payload)
    pub sbdh: sbdh::SbdhParsed,
    /// Action (document type identifier)
    pub action: String,
    /// Service (process identifier)
    pub service: String,
}

/// Parse un message AS4 entrant (MIME multipart).
///
/// Extrait l'enveloppe SOAP et le payload SBDH.
/// Gère les payloads compressés gzip (PEPPOL AS4 v2.0).
pub fn parse_incoming_as4(content_type: &str, body: &[u8]) -> Result<As4IncomingMessage, PeppolError> {
    // Extraire le boundary du Content-Type
    let boundary = extract_mime_boundary(content_type)
        .ok_or_else(|| PeppolError::As4Error("Boundary MIME manquant".into()))?;

    // Séparer les parties MIME en binaire
    let parts = split_mime_parts_binary(body, &boundary);

    if parts.len() < 2 {
        return Err(PeppolError::As4Error(format!(
            "Message AS4 doit contenir au moins 2 parties MIME, trouvé {}",
            parts.len()
        )));
    }

    // Partie 1 : SOAP Envelope (texte XML)
    let soap_xml = std::str::from_utf8(&parts[0])
        .map_err(|e| PeppolError::As4Error(format!("SOAP envelope non UTF-8 : {}", e)))?;
    let (message_id, action, service) = parse_soap_envelope(soap_xml)?;

    // Partie 2 : SBDH + payload (potentiellement compressé gzip)
    let sbdh_bytes = try_gzip_decompress_bytes(&parts[1]);
    let sbdh_xml = std::str::from_utf8(&sbdh_bytes)
        .map_err(|e| PeppolError::As4Error(format!("SBDH non UTF-8 après décompression : {}", e)))?;
    let sbdh_parsed = sbdh::parse_sbdh(sbdh_xml)
        .map_err(|e| PeppolError::As4Error(format!("SBDH invalide : {}", e)))?;

    tracing::info!(
        message_id = %message_id,
        sender = %sbdh_parsed.sender,
        receiver = %sbdh_parsed.receiver,
        action = %action,
        "Message AS4 entrant reçu"
    );

    Ok(As4IncomingMessage {
        message_id,
        sbdh: sbdh_parsed,
        action,
        service,
    })
}

/// Construit une réponse AS4 de succès (receipt signal).
pub fn build_as4_receipt(ref_message_id: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
               xmlns:eb="http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/">
  <soap:Header>
    <eb:Messaging>
      <eb:SignalMessage>
        <eb:MessageInfo>
          <eb:Timestamp>{}</eb:Timestamp>
          <eb:MessageId>{}@receipt</eb:MessageId>
          <eb:RefToMessageId>{}</eb:RefToMessageId>
        </eb:MessageInfo>
        <eb:Receipt/>
      </eb:SignalMessage>
    </eb:Messaging>
  </soap:Header>
  <soap:Body/>
</soap:Envelope>"#,
        Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        uuid::Uuid::new_v4(),
        xml_escape(ref_message_id),
    )
}

/// Construit une réponse AS4 d'erreur.
pub fn build_as4_error(ref_message_id: &str, error_code: &str, error_detail: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
               xmlns:eb="http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/">
  <soap:Header>
    <eb:Messaging>
      <eb:SignalMessage>
        <eb:MessageInfo>
          <eb:Timestamp>{}</eb:Timestamp>
          <eb:MessageId>{}@error</eb:MessageId>
          <eb:RefToMessageId>{}</eb:RefToMessageId>
        </eb:MessageInfo>
        <eb:Error errorCode="{}" severity="failure" errorDetail="{}"/>
      </eb:SignalMessage>
    </eb:Messaging>
  </soap:Header>
  <soap:Body/>
</soap:Envelope>"#,
        Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        uuid::Uuid::new_v4(),
        xml_escape(ref_message_id),
        xml_escape(error_code),
        xml_escape(error_detail),
    )
}

// ============================================================
// Construction SOAP / MIME
// ============================================================

/// Construit l'enveloppe SOAP ebMS3 pour un message AS4 sortant.
///
/// `to_party_id` est l'identifiant de l'AP destinataire (typiquement le CN
/// du certificat AP obtenu via SMP). Si absent, on utilise l'AP ID de
/// l'émetteur en fallback (ce qui peut fonctionner dans certains cas de test).
fn build_soap_envelope(
    message_id: &str,
    ap_id: &str,
    to_party_id: &str,
    action: &str,
    service: &str,
) -> String {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
               xmlns:eb="http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/">
  <soap:Header>
    <eb:Messaging>
      <eb:UserMessage mpc="{mpc}">
        <eb:MessageInfo>
          <eb:Timestamp>{timestamp}</eb:Timestamp>
          <eb:MessageId>{message_id}</eb:MessageId>
        </eb:MessageInfo>
        <eb:PartyInfo>
          <eb:From>
            <eb:PartyId type="{party_type}">{ap_id}</eb:PartyId>
            <eb:Role>{initiator_role}</eb:Role>
          </eb:From>
          <eb:To>
            <eb:PartyId type="{party_type}">{to_party_id}</eb:PartyId>
            <eb:Role>{responder_role}</eb:Role>
          </eb:To>
        </eb:PartyInfo>
        <eb:CollaborationInfo>
          <eb:AgreementRef>{agreement}</eb:AgreementRef>
          <eb:Service type="cenbii-procid-ubl">{service}</eb:Service>
          <eb:Action>{action}</eb:Action>
        </eb:CollaborationInfo>
        <eb:PayloadInfo>
          <eb:PartInfo href="cid:sbdh-payload">
            <eb:PartProperties>
              <eb:Property name="MimeType">{mime_type}</eb:Property>
              <eb:Property name="CompressionType">application/gzip</eb:Property>
            </eb:PartProperties>
          </eb:PartInfo>
        </eb:PayloadInfo>
      </eb:UserMessage>
    </eb:Messaging>
  </soap:Header>
  <soap:Body/>
</soap:Envelope>"#,
        mpc = PEPPOL_DEFAULT_MPC,
        timestamp = timestamp,
        message_id = xml_escape(message_id),
        party_type = PEPPOL_PARTY_TYPE,
        ap_id = xml_escape(ap_id),
        to_party_id = xml_escape(to_party_id),
        initiator_role = PEPPOL_INITIATOR_ROLE,
        responder_role = PEPPOL_RESPONDER_ROLE,
        agreement = PEPPOL_AGREEMENT,
        service = xml_escape(service),
        action = xml_escape(action),
        mime_type = MIME_TYPE_SBDH,
    )
}

/// Compresse le payload SBDH en gzip (requis par PEPPOL AS4 v2.0).
///
/// La spécification PEPPOL AS4 exige que le payload soit compressé en gzip
/// lorsque `CompressionType: application/gzip` est déclaré dans le SOAP envelope.
fn gzip_compress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// Construit le corps MIME multipart (SOAP + SBDH payload compressé gzip).
///
/// Retourne des bytes car la partie 2 est du binaire gzip.
fn build_mime_multipart(soap_envelope: &str, sbdh_payload: &str) -> Vec<u8> {
    // Compresser le SBDH payload en gzip conformément à PEPPOL AS4 v2.0
    let compressed = gzip_compress(sbdh_payload.as_bytes())
        .unwrap_or_else(|_| sbdh_payload.as_bytes().to_vec());

    let mut body = Vec::with_capacity(soap_envelope.len() + compressed.len() + 512);

    // Part 1 : SOAP Envelope (texte)
    body.extend_from_slice(format!("--{}\r\n", MIME_BOUNDARY).as_bytes());
    body.extend_from_slice(b"Content-Type: application/soap+xml; charset=UTF-8\r\n");
    body.extend_from_slice(b"Content-ID: <soap-envelope>\r\n");
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(soap_envelope.as_bytes());
    body.extend_from_slice(b"\r\n");

    // Part 2 : SBDH payload compressé gzip (binaire)
    body.extend_from_slice(format!("--{}\r\n", MIME_BOUNDARY).as_bytes());
    body.extend_from_slice(b"Content-Type: application/gzip\r\n");
    body.extend_from_slice(b"Content-ID: <sbdh-payload>\r\n");
    body.extend_from_slice(b"Content-Transfer-Encoding: binary\r\n");
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(&compressed);
    body.extend_from_slice(b"\r\n");

    // Fin
    body.extend_from_slice(format!("--{}--\r\n", MIME_BOUNDARY).as_bytes());

    body
}

// ============================================================
// Parsing
// ============================================================

fn parse_soap_envelope(xml: &str) -> Result<(String, String, String), PeppolError> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| PeppolError::As4Error(format!("SOAP XML invalide : {}", e)))?;

    let mut message_id = String::new();
    let mut action = String::new();
    let mut service = String::new();

    for node in doc.root().descendants() {
        if node.has_tag_name("MessageId") {
            message_id = node.text().unwrap_or("").to_string();
        }
        if node.has_tag_name("Action") {
            action = node.text().unwrap_or("").to_string();
        }
        if node.has_tag_name("Service") {
            service = node.text().unwrap_or("").to_string();
        }
    }

    if message_id.is_empty() {
        return Err(PeppolError::As4Error("MessageId manquant dans SOAP".into()));
    }

    Ok((message_id, action, service))
}

fn parse_ebms_error(xml: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    for node in doc.root().descendants() {
        if node.has_tag_name("Error") {
            let code = node.attribute("errorCode").unwrap_or("UNKNOWN");
            let detail = node.attribute("errorDetail").unwrap_or("");
            return Some(format!("ebMS Error {}: {}", code, detail));
        }
    }
    None
}

fn extract_mime_boundary(content_type: &str) -> Option<String> {
    for part in content_type.split(';') {
        let trimmed = part.trim();
        if trimmed.starts_with("boundary=") || trimmed.starts_with("boundary=\"") {
            let value = trimmed.trim_start_matches("boundary=").trim_matches('"');
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
fn split_mime_parts(body: &str, boundary: &str) -> Vec<String> {
    let delimiter = format!("--{}", boundary);
    let end_delimiter = format!("--{}--", boundary);

    body.split(&delimiter)
        .filter(|part| !part.is_empty() && !part.starts_with("--"))
        .filter(|part| !part.trim().is_empty() && *part != "\r\n")
        .map(|part| {
            // Retirer les headers MIME (tout avant la première ligne vide)
            if let Some(pos) = part.find("\r\n\r\n") {
                part[pos + 4..].trim_end_matches(&end_delimiter).trim().to_string()
            } else if let Some(pos) = part.find("\n\n") {
                part[pos + 2..].trim_end_matches(&end_delimiter).trim().to_string()
            } else {
                part.trim().to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Tente de décompresser un contenu gzip binaire. Si ce n'est pas du gzip valide,
/// retourne les bytes originaux tels quels.
fn try_gzip_decompress_bytes(data: &[u8]) -> Vec<u8> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    // Vérifier le magic number gzip (1f 8b)
    if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        if decoder.read_to_end(&mut decompressed).is_ok() {
            return decompressed;
        }
    }
    // Pas du gzip ou échec de décompression → retourner tel quel
    data.to_vec()
}

/// Sépare les parties MIME en travaillant sur des bytes bruts.
///
/// Nécessaire car le payload SBDH peut être compressé gzip (binaire non-UTF-8).
fn split_mime_parts_binary(body: &[u8], boundary: &str) -> Vec<Vec<u8>> {
    let delimiter = format!("--{}", boundary);
    let delimiter_bytes = delimiter.as_bytes();
    let end_delimiter = format!("--{}--", boundary);
    let end_delimiter_bytes = end_delimiter.as_bytes();

    let mut parts = Vec::new();
    let mut start = 0;

    // Trouver chaque occurrence du delimiter
    while let Some(pos) = find_bytes(&body[start..], delimiter_bytes) {
        let abs_pos = start + pos;
        if start > 0 {
            // Extraire la partie entre le delimiter précédent et celui-ci
            let part_data = &body[start..abs_pos];
            if let Some(extracted) = extract_mime_part_body(part_data) {
                if !extracted.is_empty() {
                    parts.push(extracted);
                }
            }
        }
        // Avancer après le delimiter + CRLF
        start = abs_pos + delimiter_bytes.len();
        // Sauter le CRLF ou LF après le delimiter
        if start < body.len() && body[start] == b'\r' {
            start += 1;
        }
        if start < body.len() && body[start] == b'\n' {
            start += 1;
        }
        // Vérifier si c'est le end delimiter
        if abs_pos + end_delimiter_bytes.len() <= body.len()
            && &body[abs_pos..abs_pos + end_delimiter_bytes.len()] == end_delimiter_bytes
        {
            break;
        }
    }

    // Dernière partie (après le dernier delimiter, avant le end delimiter)
    if start < body.len() {
        // Chercher le end delimiter
        let remaining = &body[start..];
        let end_pos = find_bytes(remaining, delimiter_bytes)
            .unwrap_or(remaining.len());
        let part_data = &remaining[..end_pos];
        if let Some(extracted) = extract_mime_part_body(part_data) {
            if !extracted.is_empty() {
                parts.push(extracted);
            }
        }
    }

    parts
}

/// Recherche une séquence de bytes dans un slice.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Extrait le body d'une partie MIME (après les headers, séparés par \r\n\r\n ou \n\n).
fn extract_mime_part_body(data: &[u8]) -> Option<Vec<u8>> {
    // Chercher la séparation headers/body : \r\n\r\n
    let sep_crlf = b"\r\n\r\n";
    let sep_lf = b"\n\n";

    let body = if let Some(pos) = find_bytes(data, sep_crlf) {
        &data[pos + 4..]
    } else if let Some(pos) = find_bytes(data, sep_lf) {
        &data[pos + 2..]
    } else {
        return None;
    };

    // Trimmer les \r\n finaux
    let mut end = body.len();
    while end > 0 && (body[end - 1] == b'\r' || body[end - 1] == b'\n') {
        end -= 1;
    }

    if end == 0 {
        return None;
    }

    Some(body[..end].to_vec())
}

/// Tente d'extraire le CN (Common Name) d'un certificat X.509 base64 (PEM body).
fn extract_ap_cn_from_cert(cert_b64: &str) -> Option<String> {
    // Nettoyer le base64 (retirer les en-têtes PEM éventuels et whitespace)
    let cleaned: String = cert_b64
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    if cleaned.is_empty() {
        return None;
    }

    // Décoder le DER
    let der = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &cleaned,
    ).ok()?;

    // Chercher le CN dans le DER (heuristique : chercher l'OID 2.5.4.3 = 55 04 03)
    // suivi d'un UTF8String (0x0C) ou PrintableString (0x13)
    let cn_oid = [0x55u8, 0x04, 0x03];
    for (i, window) in der.windows(3).enumerate() {
        if window == cn_oid {
            // Après l'OID, on a un tag + longueur + valeur
            let tag_pos = i + 3;
            if tag_pos + 2 >= der.len() {
                continue;
            }
            let tag = der[tag_pos];
            if tag != 0x0C && tag != 0x13 {
                continue;
            }
            let len = der[tag_pos + 1] as usize;
            let val_start = tag_pos + 2;
            if val_start + len > der.len() {
                continue;
            }
            if let Ok(cn) = std::str::from_utf8(&der[val_start..val_start + len]) {
                if !cn.is_empty() {
                    return Some(cn.to_string());
                }
            }
        }
    }

    None
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn test_build_soap_envelope() {
        let soap = build_soap_envelope(
            "test-msg-001@POP000123",
            "POP000123",
            "POP000456",
            "busdox-docid-qns::urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice",
            "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0",
        );

        assert!(soap.contains("soap:Envelope"));
        assert!(soap.contains("eb:UserMessage"));
        assert!(soap.contains("test-msg-001@POP000123"));
        assert!(soap.contains("POP000123"));
        assert!(soap.contains("POP000456"), "To PartyId doit contenir l'AP destinataire");
        assert!(!soap.contains("UNKNOWN"), "Ne doit plus contenir UNKNOWN");
        assert!(soap.contains(PEPPOL_AGREEMENT));
        assert!(soap.contains(PEPPOL_DEFAULT_MPC));
        assert!(soap.contains("Invoice"));
    }

    #[test]
    fn test_build_mime_multipart() {
        let soap = "<soap:Envelope/>";
        let sbdh = "<StandardBusinessDocument/>";
        let mime = build_mime_multipart(soap, sbdh);
        let mime_str = String::from_utf8_lossy(&mime);

        assert!(mime_str.contains(MIME_BOUNDARY));
        assert!(mime_str.contains("application/soap+xml"));
        assert!(mime_str.contains("soap-envelope"));
        assert!(mime_str.contains("sbdh-payload"));
        assert!(mime_str.contains(soap), "SOAP envelope doit rester en clair");
        // Le SBDH est maintenant compressé gzip, on ne le trouvera pas en clair
        assert!(mime_str.contains("application/gzip"), "Content-Type du payload doit être gzip");
    }

    #[test]
    fn test_gzip_compress_decompress_roundtrip() {
        let original = "<StandardBusinessDocument><Payload>Test data</Payload></StandardBusinessDocument>";
        let compressed = gzip_compress(original.as_bytes()).unwrap();
        assert_ne!(compressed, original.as_bytes(), "Compressed doit être différent");
        // Vérifier le magic number gzip
        assert_eq!(compressed[0], 0x1f);
        assert_eq!(compressed[1], 0x8b);
        // Décompresser via notre fonction utilitaire
        let decompressed = try_gzip_decompress_bytes(&compressed);
        assert_eq!(decompressed, original.as_bytes());
        assert_eq!(std::str::from_utf8(&decompressed).unwrap(), original);
    }

    #[test]
    fn test_gzip_decompress_non_gzip_passthrough() {
        // Si les données ne sont pas du gzip, elles doivent être retournées telles quelles
        let plain = b"<StandardBusinessDocument/>";
        let result = try_gzip_decompress_bytes(plain);
        assert_eq!(result, plain);
    }

    #[test]
    fn test_parse_soap_envelope() {
        let soap = build_soap_envelope(
            "msg-123@AP",
            "AP001",
            "AP002",
            "test-action",
            "test-service",
        );
        let (msg_id, action, service) = parse_soap_envelope(&soap).unwrap();
        assert_eq!(msg_id, "msg-123@AP");
        assert_eq!(action, "test-action");
        assert_eq!(service, "test-service");
    }

    #[test]
    fn test_extract_mime_boundary() {
        let ct = r#"multipart/related; boundary="----=_Part_123"; type="application/soap+xml""#;
        assert_eq!(extract_mime_boundary(ct), Some("----=_Part_123".to_string()));

        let ct2 = "multipart/related; boundary=simple_boundary";
        assert_eq!(extract_mime_boundary(ct2), Some("simple_boundary".to_string()));
    }

    #[test]
    fn test_split_mime_parts() {
        let body = format!(
            "--{boundary}\r\nContent-Type: text/xml\r\n\r\n<part1/>\r\n--{boundary}\r\nContent-Type: text/xml\r\n\r\n<part2/>\r\n--{boundary}--\r\n",
            boundary = "BOUNDARY"
        );
        let parts = split_mime_parts(&body, "BOUNDARY");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "<part1/>");
        assert_eq!(parts[1], "<part2/>");
    }

    #[test]
    fn test_build_as4_receipt() {
        let receipt = build_as4_receipt("msg-001@AP");
        assert!(receipt.contains("soap:Envelope"));
        assert!(receipt.contains("eb:Receipt"));
        assert!(receipt.contains("msg-001@AP"));
    }

    #[test]
    fn test_build_as4_error() {
        let error = build_as4_error("msg-001@AP", "EBMS:0004", "PEPPOL:NOT_SERVICED");
        assert!(error.contains("EBMS:0004"));
        assert!(error.contains("PEPPOL:NOT_SERVICED"));
        assert!(error.contains("msg-001@AP"));
    }

    #[test]
    fn test_parse_ebms_error() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:eb="http://docs.oasis-open.org/ebxml-msg/ebms/v3.0/ns/core/200704/">
            <soap:Header><eb:Messaging><eb:SignalMessage>
                <eb:Error errorCode="EBMS:0004" severity="failure" errorDetail="PEPPOL:NOT_SERVICED"/>
            </eb:SignalMessage></eb:Messaging></soap:Header></soap:Envelope>"#;
        let err = parse_ebms_error(xml).unwrap();
        assert!(err.contains("EBMS:0004"));
        assert!(err.contains("NOT_SERVICED"));
    }

    #[test]
    fn test_roundtrip_mime_as4() {
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::ubl_invoice(sender, receiver, b"<Invoice><ID>FA-001</ID></Invoice>".to_vec());

        let sbdh_xml = sbdh::build_sbdh(&msg);
        let soap = build_soap_envelope(
            "roundtrip-001@AP",
            "AP001",
            "AP002",
            &msg.document_type_id.to_action(),
            &msg.process_id.to_string(),
        );
        let mime_bytes = build_mime_multipart(&soap, &sbdh_xml);

        // Parser en binaire (nécessaire car la partie 2 est du gzip)
        let parts = split_mime_parts_binary(&mime_bytes, MIME_BOUNDARY);
        assert!(parts.len() >= 2, "Doit avoir au moins 2 parties MIME, trouvé {}", parts.len());

        // Partie 1 : SOAP (texte)
        let soap_str = std::str::from_utf8(&parts[0]).unwrap();
        let (msg_id, _, _) = parse_soap_envelope(soap_str).unwrap();
        assert_eq!(msg_id, "roundtrip-001@AP");

        // Partie 2 : SBDH compressé gzip → décompresser
        let sbdh_bytes = try_gzip_decompress_bytes(&parts[1]);
        let sbdh_str = std::str::from_utf8(&sbdh_bytes).unwrap();
        let sbdh_parsed = sbdh::parse_sbdh(sbdh_str).unwrap();
        assert_eq!(sbdh_parsed.sender.value, "111111111");
        assert_eq!(sbdh_parsed.receiver.value, "222222222");
        assert!(sbdh_parsed.payload.contains("FA-001"));
    }
}
