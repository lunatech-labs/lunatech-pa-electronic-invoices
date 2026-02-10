use pdp_core::model::InvoiceAttachment;

use crate::ppf::FluxFile;

/// Convertit une InvoiceAttachment en FluxFile pour inclusion dans un flux tar.gz PPF.
/// Les pièces jointes avec contenu embarqué sont converties en fichiers à inclure
/// dans l'archive tar.gz du flux.
pub fn attachment_to_flux_file(attachment: &InvoiceAttachment) -> Option<FluxFile> {
    let content = attachment.embedded_content.as_ref()?;
    let filename = attachment
        .filename
        .clone()
        .unwrap_or_else(|| {
            let ext = attachment
                .mime_code
                .as_deref()
                .and_then(extension_from_mime)
                .unwrap_or("bin");
            format!(
                "pj_{}.{}",
                attachment.id.as_deref().unwrap_or("unknown"),
                ext
            )
        });

    Some(FluxFile {
        filename,
        content: content.clone(),
    })
}

/// Déduit l'extension de fichier à partir d'un code MIME
fn extension_from_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "application/pdf" => Some("pdf"),
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "text/csv" => Some("csv"),
        "application/xml" => Some("xml"),
        _ => None,
    }
}

/// Codes MIME acceptés par le PPF dans les flux tar.gz
pub const ACCEPTED_MIME_TYPES: &[&str] = &[
    "application/ubl+xml",
    "application/cii+xml",
    "application/facturx+pdf",
    "application/tar+gzip;content=ubl",
    "application/tar+gzip;content=cii",
    "application/tar+gzip;content=facturx",
    "application/pdf",
    "image/png",
    "image/jpeg",
    "text/csv",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.oasis.opendocument.spreadsheet",
];

/// Vérifie si un code MIME est accepté par le PPF
pub fn is_accepted_mime(mime: &str) -> bool {
    ACCEPTED_MIME_TYPES.iter().any(|&m| m == mime)
}

/// Déduit le code MIME à partir de l'extension du fichier
pub fn mime_from_filename(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    match ext.as_str() {
        "pdf" => Some("application/pdf"),
        "xml" => Some("application/xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "csv" => Some("text/csv"),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        "ods" => Some("application/vnd.oasis.opendocument.spreadsheet"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_to_flux_file_with_filename() {
        let attachment = InvoiceAttachment {
            id: Some("ATT-001".to_string()),
            description: Some("Bon de commande".to_string()),
            external_uri: None,
            embedded_content: Some(b"Hello PDF content".to_vec()),
            mime_code: Some("application/pdf".to_string()),
            filename: Some("bon_commande.pdf".to_string()),
        };

        let flux_file = attachment_to_flux_file(&attachment).unwrap();
        assert_eq!(flux_file.filename, "bon_commande.pdf");
        assert_eq!(flux_file.content, b"Hello PDF content");
    }

    #[test]
    fn test_attachment_to_flux_file_without_filename() {
        let attachment = InvoiceAttachment {
            id: Some("ATT-002".to_string()),
            description: None,
            external_uri: None,
            embedded_content: Some(b"PNG data".to_vec()),
            mime_code: Some("image/png".to_string()),
            filename: None,
        };

        let flux_file = attachment_to_flux_file(&attachment).unwrap();
        assert_eq!(flux_file.filename, "pj_ATT-002.png");
    }

    #[test]
    fn test_attachment_to_flux_file_no_content() {
        let attachment = InvoiceAttachment {
            id: Some("ATT-003".to_string()),
            description: None,
            external_uri: Some("https://example.com/doc.pdf".to_string()),
            embedded_content: None,
            mime_code: None,
            filename: None,
        };

        assert!(attachment_to_flux_file(&attachment).is_none());
    }

    #[test]
    fn test_is_accepted_mime() {
        assert!(is_accepted_mime("application/pdf"));
        assert!(is_accepted_mime("image/png"));
        assert!(is_accepted_mime("application/ubl+xml"));
        assert!(!is_accepted_mime("application/zip"));
        assert!(!is_accepted_mime("text/html"));
    }

    #[test]
    fn test_mime_from_filename() {
        assert_eq!(mime_from_filename("doc.pdf"), Some("application/pdf"));
        assert_eq!(mime_from_filename("image.PNG"), Some("image/png"));
        assert_eq!(mime_from_filename("data.csv"), Some("text/csv"));
        assert_eq!(mime_from_filename("report.xlsx"), Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"));
        assert_eq!(mime_from_filename("unknown.bin"), None);
    }
}
