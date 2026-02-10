//! Construction et parsing du Standard Business Document Header (SBDH).
//!
//! Le SBDH est l'enveloppe obligatoire pour tout document transmis via le
//! réseau PEPPOL. Il contient les métadonnées de routage (émetteur, destinataire,
//! type de document, processus) et encapsule le document métier (facture ou CDAR).
//!
//! Conforme à : PEPPOL Envelope Specification (SBDH) + UN/CEFACT SBDH 1.3

use chrono::Utc;
use crate::model::{DocumentTypeId, ParticipantId, PeppolMessage, ProcessId};

/// Construit un SBDH XML complet encapsulant le document métier.
///
/// Structure :
/// ```xml
/// <StandardBusinessDocument>
///   <StandardBusinessDocumentHeader>
///     <HeaderVersion>1.0</HeaderVersion>
///     <Sender><Identifier Authority="scheme">value</Identifier></Sender>
///     <Receiver><Identifier Authority="scheme">value</Identifier></Receiver>
///     <DocumentIdentification>
///       <Standard>...</Standard>
///       <TypeVersion>...</TypeVersion>
///       <InstanceIdentifier>...</InstanceIdentifier>
///       <Type>...</Type>
///       <CreationDateAndTime>...</CreationDateAndTime>
///     </DocumentIdentification>
///     <BusinessScope>
///       <Scope>
///         <Type>DOCUMENTID</Type>
///         <InstanceIdentifier>document_type_id</InstanceIdentifier>
///         <Identifier>scheme</Identifier>
///       </Scope>
///       <Scope>
///         <Type>PROCESSID</Type>
///         <InstanceIdentifier>process_id</InstanceIdentifier>
///         <Identifier>scheme</Identifier>
///       </Scope>
///       <Scope>
///         <Type>COUNTRY_C1</Type>
///         <InstanceIdentifier>FR</InstanceIdentifier>
///       </Scope>
///     </BusinessScope>
///   </StandardBusinessDocumentHeader>
///   <!-- payload XML inséré ici -->
/// </StandardBusinessDocument>
/// ```
pub fn build_sbdh(message: &PeppolMessage) -> String {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Déterminer le Standard et Type à partir du document type
    let (standard, type_version, doc_type) = extract_sbdh_doc_info(&message.document_type_id);

    let payload_str = String::from_utf8_lossy(&message.payload);

    // Retirer la déclaration XML du payload s'il y en a une
    let payload_content = strip_xml_declaration(&payload_str);

    let mut xml = String::with_capacity(4096 + message.payload.len());

    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<StandardBusinessDocument xmlns="http://www.unece.org/cefact/namespaces/StandardBusinessDocumentHeader">
  <StandardBusinessDocumentHeader>
    <HeaderVersion>1.0</HeaderVersion>
    <Sender>
      <Identifier Authority=""#);
    xml.push_str(&xml_escape(&message.sender.scheme));
    xml.push_str(r#"">"#);
    xml.push_str(&xml_escape(&message.sender.value));
    xml.push_str(r#"</Identifier>
    </Sender>
    <Receiver>
      <Identifier Authority=""#);
    xml.push_str(&xml_escape(&message.receiver.scheme));
    xml.push_str(r#"">"#);
    xml.push_str(&xml_escape(&message.receiver.value));
    xml.push_str(r#"</Identifier>
    </Receiver>
    <DocumentIdentification>
      <Standard>"#);
    xml.push_str(&xml_escape(&standard));
    xml.push_str(r#"</Standard>
      <TypeVersion>"#);
    xml.push_str(&xml_escape(&type_version));
    xml.push_str(r#"</TypeVersion>
      <InstanceIdentifier>"#);
    xml.push_str(&xml_escape(&message.message_id));
    xml.push_str(r#"</InstanceIdentifier>
      <Type>"#);
    xml.push_str(&xml_escape(&doc_type));
    xml.push_str(r#"</Type>
      <CreationDateAndTime>"#);
    xml.push_str(&now);
    xml.push_str(r#"</CreationDateAndTime>
    </DocumentIdentification>
    <BusinessScope>
      <Scope>
        <Type>DOCUMENTID</Type>
        <InstanceIdentifier>"#);
    xml.push_str(&xml_escape(&message.document_type_id.value));
    xml.push_str(r#"</InstanceIdentifier>
        <Identifier>"#);
    xml.push_str(&xml_escape(&message.document_type_id.scheme));
    xml.push_str(r#"</Identifier>
      </Scope>
      <Scope>
        <Type>PROCESSID</Type>
        <InstanceIdentifier>"#);
    xml.push_str(&xml_escape(&message.process_id.value));
    xml.push_str(r#"</InstanceIdentifier>
        <Identifier>"#);
    xml.push_str(&xml_escape(&message.process_id.scheme));
    xml.push_str(r#"</Identifier>
      </Scope>
      <Scope>
        <Type>COUNTRY_C1</Type>
        <InstanceIdentifier>"#);
    let country = message.metadata.get("country_c1").map(|s| s.as_str()).unwrap_or("FR");
    xml.push_str(&xml_escape(country));
    xml.push_str(r#"</InstanceIdentifier>
      </Scope>
    </BusinessScope>
  </StandardBusinessDocumentHeader>
"#);

    // Insérer le payload
    xml.push_str(payload_content);
    xml.push('\n');

    xml.push_str("</StandardBusinessDocument>\n");

    xml
}

/// Parse un SBDH XML et extrait les métadonnées + payload.
///
/// Retourne (sender, receiver, document_type_id, process_id, instance_id, payload_xml)
pub fn parse_sbdh(xml: &str) -> Result<SbdhParsed, SbdhError> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| SbdhError::ParseError(format!("XML invalide : {}", e)))?;

    let root = doc.root_element();

    // Trouver le header
    let header = find_child(&root, "StandardBusinessDocumentHeader")
        .ok_or_else(|| SbdhError::ParseError("StandardBusinessDocumentHeader manquant".into()))?;

    // Sender
    let sender_node = find_child(&header, "Sender")
        .ok_or_else(|| SbdhError::ParseError("Sender manquant".into()))?;
    let sender = parse_participant(&sender_node)?;

    // Receiver
    let receiver_node = find_child(&header, "Receiver")
        .ok_or_else(|| SbdhError::ParseError("Receiver manquant".into()))?;
    let receiver = parse_participant(&receiver_node)?;

    // DocumentIdentification
    let doc_id = find_child(&header, "DocumentIdentification")
        .ok_or_else(|| SbdhError::ParseError("DocumentIdentification manquant".into()))?;
    let instance_id = find_child_text(&doc_id, "InstanceIdentifier")
        .unwrap_or_default();

    // BusinessScope
    let mut document_type_id = None;
    let mut process_id = None;

    if let Some(scope_node) = find_child(&header, "BusinessScope") {
        for scope in scope_node.children().filter(|n| n.has_tag_name("Scope")) {
            let scope_type = find_child_text(&scope, "Type").unwrap_or_default();
            let scope_instance = find_child_text(&scope, "InstanceIdentifier").unwrap_or_default();
            let scope_identifier = find_child_text(&scope, "Identifier").unwrap_or_default();

            match scope_type.as_str() {
                "DOCUMENTID" => {
                    document_type_id = Some(DocumentTypeId::new(&scope_identifier, &scope_instance));
                }
                "PROCESSID" => {
                    process_id = Some(ProcessId::new(&scope_identifier, &scope_instance));
                }
                _ => {}
            }
        }
    }

    // Payload : tout ce qui suit le header dans le document
    let payload = extract_payload(xml);

    Ok(SbdhParsed {
        sender,
        receiver,
        instance_id,
        document_type_id: document_type_id
            .ok_or_else(|| SbdhError::ParseError("DOCUMENTID scope manquant".into()))?,
        process_id: process_id
            .ok_or_else(|| SbdhError::ParseError("PROCESSID scope manquant".into()))?,
        payload,
    })
}

/// Résultat du parsing d'un SBDH.
#[derive(Debug, Clone)]
pub struct SbdhParsed {
    pub sender: ParticipantId,
    pub receiver: ParticipantId,
    pub instance_id: String,
    pub document_type_id: DocumentTypeId,
    pub process_id: ProcessId,
    /// Payload XML (le document métier sans l'enveloppe SBDH)
    pub payload: String,
}

/// Erreurs de parsing SBDH.
#[derive(Debug, thiserror::Error)]
pub enum SbdhError {
    #[error("Erreur de parsing SBDH : {0}")]
    ParseError(String),
}

// ============================================================
// Fonctions utilitaires internes
// ============================================================

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn strip_xml_declaration(xml: &str) -> &str {
    let trimmed = xml.trim();
    if trimmed.starts_with("<?xml") {
        if let Some(pos) = trimmed.find("?>") {
            return trimmed[pos + 2..].trim_start();
        }
    }
    trimmed
}

/// Extrait le Standard, TypeVersion et Type à partir du DocumentTypeId
fn extract_sbdh_doc_info(dt: &DocumentTypeId) -> (String, String, String) {
    // Format typique : urn:oasis:...:Invoice-2::Invoice##customization::2.1
    // On extrait : Standard = urn:oasis:...:Invoice-2
    //              Type = Invoice
    //              TypeVersion = 2.1
    let value = &dt.value;

    if let Some(double_colon_pos) = value.find("::") {
        let standard = &value[..double_colon_pos];
        let rest = &value[double_colon_pos + 2..];

        if let Some(hash_pos) = rest.find("##") {
            let doc_type = &rest[..hash_pos];
            let after_hash = &rest[hash_pos + 2..];

            // TypeVersion est après le dernier ::
            let type_version = if let Some(last_colon) = after_hash.rfind("::") {
                &after_hash[last_colon + 2..]
            } else {
                "1.0"
            };

            return (standard.to_string(), type_version.to_string(), doc_type.to_string());
        }

        return (standard.to_string(), "1.0".to_string(), rest.to_string());
    }

    (value.clone(), "1.0".to_string(), "Document".to_string())
}

fn find_child<'a>(node: &'a roxmltree::Node<'a, 'a>, local_name: &str) -> Option<roxmltree::Node<'a, 'a>> {
    node.children().find(|n| n.has_tag_name(local_name))
}

fn find_child_text(node: &roxmltree::Node, local_name: &str) -> Option<String> {
    find_child(node, local_name)
        .and_then(|n| n.text())
        .map(|t| t.to_string())
}

fn parse_participant(node: &roxmltree::Node) -> Result<ParticipantId, SbdhError> {
    let id_node = find_child(node, "Identifier")
        .ok_or_else(|| SbdhError::ParseError("Identifier manquant dans Sender/Receiver".into()))?;

    let scheme = id_node.attribute("Authority").unwrap_or("").to_string();
    let value = id_node.text().unwrap_or("").to_string();

    Ok(ParticipantId::new(&scheme, &value))
}

/// Extrait le payload XML (tout entre </StandardBusinessDocumentHeader> et </StandardBusinessDocument>)
fn extract_payload(xml: &str) -> String {
    let header_end = "</StandardBusinessDocumentHeader>";
    let doc_end = "</StandardBusinessDocument>";

    if let Some(start) = xml.find(header_end) {
        let after_header = &xml[start + header_end.len()..];
        if let Some(end) = after_header.rfind(doc_end) {
            return after_header[..end].trim().to_string();
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PeppolMessage;

    #[test]
    fn test_build_sbdh_ubl_invoice() {
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::ubl_invoice(
            sender,
            receiver,
            b"<Invoice xmlns=\"urn:oasis:names:specification:ubl:schema:xsd:Invoice-2\"><ID>FA-001</ID></Invoice>".to_vec(),
        );

        let sbdh = build_sbdh(&msg);

        assert!(sbdh.contains("StandardBusinessDocument"));
        assert!(sbdh.contains("StandardBusinessDocumentHeader"));
        assert!(sbdh.contains("111111111"));
        assert!(sbdh.contains("222222222"));
        assert!(sbdh.contains("DOCUMENTID"));
        assert!(sbdh.contains("PROCESSID"));
        assert!(sbdh.contains("<Invoice"));
        assert!(sbdh.contains("FA-001"));
    }

    #[test]
    fn test_build_sbdh_cdar() {
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let msg = PeppolMessage::cdar(
            sender,
            receiver,
            b"<CrossDomainAcknowledgementAndResponse/>".to_vec(),
        );

        let sbdh = build_sbdh(&msg);
        assert!(sbdh.contains("CrossDomainAcknowledgementAndResponse"));
    }

    #[test]
    fn test_roundtrip_sbdh() {
        let sender = ParticipantId::from_siren("111111111");
        let receiver = ParticipantId::from_siren("222222222");
        let payload = b"<Invoice xmlns=\"urn:test\"><ID>TEST-001</ID></Invoice>";
        let msg = PeppolMessage::ubl_invoice(sender.clone(), receiver.clone(), payload.to_vec());

        let sbdh_xml = build_sbdh(&msg);
        let parsed = parse_sbdh(&sbdh_xml).unwrap();

        assert_eq!(parsed.sender.scheme, "0002");
        assert_eq!(parsed.sender.value, "111111111");
        assert_eq!(parsed.receiver.scheme, "0002");
        assert_eq!(parsed.receiver.value, "222222222");
        assert_eq!(parsed.instance_id, msg.message_id);
        assert_eq!(parsed.document_type_id.scheme, "busdox-docid-qns");
        assert_eq!(parsed.process_id.scheme, "cenbii-procid-ubl");
        assert!(parsed.payload.contains("TEST-001"));
    }

    #[test]
    fn test_strip_xml_declaration() {
        assert_eq!(
            strip_xml_declaration("<?xml version=\"1.0\"?><root/>"),
            "<root/>"
        );
        assert_eq!(strip_xml_declaration("<root/>"), "<root/>");
    }

    #[test]
    fn test_extract_sbdh_doc_info_ubl() {
        let dt = DocumentTypeId::ubl_invoice();
        let (standard, version, doc_type) = extract_sbdh_doc_info(&dt);
        assert!(standard.contains("Invoice-2"));
        assert_eq!(doc_type, "Invoice");
        assert_eq!(version, "2.1");
    }

    #[test]
    fn test_extract_sbdh_doc_info_cii() {
        let dt = DocumentTypeId::cii_invoice();
        let (standard, version, doc_type) = extract_sbdh_doc_info(&dt);
        assert!(standard.contains("CrossIndustryInvoice"));
        assert_eq!(doc_type, "CrossIndustryInvoice");
        assert_eq!(version, "D16B");
    }
}
