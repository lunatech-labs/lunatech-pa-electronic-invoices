//! Parser XML streaming pour les flux F14 de l'annuaire PPF.
//!
//! Utilise quick-xml en mode pull (Reader) pour traiter des fichiers
//! de 10+ Go sans les charger intégralement en mémoire.
//! Chaque élément est parsé individuellement et transmis via un callback.

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::BufRead;
use tracing::{debug, warn};

use crate::model::*;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Erreur XML : {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Champ obligatoire manquant : {0}")]
    MissingField(String),
    #[error("Valeur invalide pour {field} : {value}")]
    InvalidValue { field: String, value: String },
    #[error("IO : {0}")]
    Io(#[from] std::io::Error),
}

/// Callback appelé pour chaque élément parsé du F14
pub enum F14Event {
    Header(F14Header),
    UniteLegale(UniteLegale),
    Etablissement(Etablissement),
    CodeRoutage(CodeRoutage),
    Plateforme(Plateforme),
    LigneAnnuaire(LigneAnnuaire),
}

/// Parse un flux F14 depuis un reader quelconque (fichier, stdin, réseau).
/// Appelle le callback pour chaque élément parsé.
/// Retourne les statistiques d'import.
pub fn parse_f14<R, F>(reader: R, mut callback: F) -> Result<ImportStats, ParseError>
where
    R: BufRead,
    F: FnMut(F14Event) -> Result<(), ParseError>,
{
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut buf = Vec::with_capacity(4096);
    let mut stats = ImportStats::default();

    // État du parser
    let mut current_bloc = BlocState::None;
    let mut element_buf = String::with_capacity(8192);
    let mut in_element = false;
    let mut element_depth = 0u32;
    let mut header = HeaderBuilder::default();
    let mut header_field: Option<String> = None;
    let mut header_emitted = false;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let name_bytes = qname.as_ref();
                let name = std::str::from_utf8(name_bytes).unwrap_or("");

                match name {
                    "BlocUnitesLegales" | "BlocEtablissements" | "BlocCodesRoutage"
                    | "BlocIdPlateformesReception" | "BlocLignesAnnuaire" => {
                        // Émettre le Header avant le premier bloc de données
                        if !header_emitted {
                            if let Some(h) = header.build() {
                                callback(F14Event::Header(h))?;
                            }
                            header_emitted = true;
                        }
                        current_bloc = match name {
                            "BlocUnitesLegales" => BlocState::UnitesLegales,
                            "BlocEtablissements" => BlocState::Etablissements,
                            "BlocCodesRoutage" => BlocState::CodesRoutage,
                            "BlocIdPlateformesReception" => BlocState::Plateformes,
                            "BlocLignesAnnuaire" => BlocState::LignesAnnuaire,
                            _ => unreachable!(),
                        };
                    }

                    "HorodateProduction" | "DernierHorodateProduction" | "TypeFlux"
                        if current_bloc == BlocState::None =>
                    {
                        header_field = Some(name.to_string());
                    }

                    "UniteLegale" | "Etablissement" | "CodeRoutage"
                    | "IdPlateformeReception" | "LigneAnnuaire" => {
                        in_element = true;
                        element_depth = 1;
                        element_buf.clear();
                        write_start_event(&mut element_buf, e);
                    }

                    _ if in_element => {
                        element_depth += 1;
                        write_start_event(&mut element_buf, e);
                    }

                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let qname = e.name();
                let name = std::str::from_utf8(qname.as_ref()).unwrap_or("");

                if in_element {
                    element_buf.push_str("</");
                    element_buf.push_str(name);
                    element_buf.push('>');
                    element_depth -= 1;

                    if element_depth == 0 {
                        in_element = false;
                        match parse_and_emit(name, &element_buf, &current_bloc, &mut stats) {
                            Ok(event) => {
                                if let Err(e) = callback(event) {
                                    warn!("Erreur callback pour {}: {}", name, e);
                                    stats.errors += 1;
                                }
                            }
                            Err(e) => {
                                if stats.errors < 5 {
                                    warn!("Erreur parsing {} : {} — XML: {}",
                                        name, e, &element_buf[..element_buf.len().min(200)]);
                                }
                                stats.errors += 1;
                            }
                        }
                    }
                } else {
                    match name {
                        "BlocUnitesLegales" | "BlocEtablissements" | "BlocCodesRoutage"
                        | "BlocIdPlateformesReception" | "BlocLignesAnnuaire" => {
                            debug!("Fin du bloc {:?}, stats: {:?}", current_bloc, stats);
                            current_bloc = BlocState::None;
                        }
                        "HorodateProduction" | "DernierHorodateProduction" | "TypeFlux" => {
                            header_field = None;
                        }
                        "AnnuaireConsultationF14" => {
                            // Fallback : émettre le Header ici si pas de blocs (F14 vide/différentiel sans données)
                            if !header_emitted {
                                if let Some(h) = header.build() {
                                    callback(F14Event::Header(h))?;
                                }
                                header_emitted = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            Ok(Event::Text(ref e)) => {
                if in_element {
                    let text = e.unescape()
                        .unwrap_or_else(|_| std::borrow::Cow::Borrowed(
                            std::str::from_utf8(e.as_ref()).unwrap_or("")
                        ));
                    element_buf.push_str(&text);
                } else if let Some(ref field) = header_field {
                    let text = e.unescape().unwrap_or_default();
                    match field.as_str() {
                        "HorodateProduction" => header.horodate = Some(text.to_string()),
                        "DernierHorodateProduction" => {
                            header.dernier_horodate = Some(text.to_string())
                        }
                        "TypeFlux" => header.type_flux = Some(text.to_string()),
                        _ => {}
                    }
                }
            }

            Ok(Event::Empty(ref e)) if in_element => {
                let qname = e.name();
                let name = std::str::from_utf8(qname.as_ref()).unwrap_or("");
                element_buf.push('<');
                element_buf.push_str(name);
                element_buf.push_str("/>");
            }

            Err(e) => return Err(ParseError::Xml(e)),

            _ => {}
        }

        buf.clear();
    }

    Ok(stats)
}

/// Reconstruit un tag d'ouverture avec ses attributs
fn write_start_event(buf: &mut String, e: &quick_xml::events::BytesStart<'_>) {
    let qname = e.name();
    let name = std::str::from_utf8(qname.as_ref()).unwrap_or("");
    buf.push('<');
    buf.push_str(name);
    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        let val = std::str::from_utf8(&attr.value).unwrap_or("");
        buf.push(' ');
        buf.push_str(key);
        buf.push_str("=\"");
        buf.push_str(val);
        buf.push('"');
    }
    buf.push('>');
}

// --- Parsing des éléments individuels via mini-parser quick-xml ---

fn parse_and_emit(
    tag: &str,
    xml: &str,
    bloc: &BlocState,
    stats: &mut ImportStats,
) -> Result<F14Event, ParseError> {
    match (tag, bloc) {
        ("UniteLegale", BlocState::UnitesLegales) => {
            let ul = parse_unite_legale(xml)?;
            stats.unites_legales += 1;
            Ok(F14Event::UniteLegale(ul))
        }
        ("Etablissement", BlocState::Etablissements) => {
            let etab = parse_etablissement(xml)?;
            stats.etablissements += 1;
            Ok(F14Event::Etablissement(etab))
        }
        ("CodeRoutage", BlocState::CodesRoutage) => {
            let cr = parse_code_routage(xml)?;
            stats.codes_routage += 1;
            Ok(F14Event::CodeRoutage(cr))
        }
        ("IdPlateformeReception", BlocState::Plateformes) => {
            let pf = parse_plateforme(xml)?;
            stats.plateformes += 1;
            Ok(F14Event::Plateforme(pf))
        }
        ("LigneAnnuaire", BlocState::LignesAnnuaire) => {
            let la = parse_ligne_annuaire(xml)?;
            stats.lignes_annuaire += 1;
            Ok(F14Event::LigneAnnuaire(la))
        }
        _ => Err(ParseError::InvalidValue {
            field: "tag".into(),
            value: tag.into(),
        }),
    }
}

/// Helper : parse un fragment XML et extrait les champs texte par tag name
struct MiniParser {
    fields: Vec<(String, String)>,
    attrs: Vec<(String, String, String)>, // (parent_tag, attr_key, attr_value)
}

impl MiniParser {
    fn parse(xml: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut fields = Vec::new();
        let mut attrs = Vec::new();
        let mut buf = Vec::new();
        let mut current_tag: Option<String> = None;
        let mut current_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(Event::Start(ref e)) => {
                    let name = std::str::from_utf8(e.name().as_ref())
                        .unwrap_or("")
                        .to_string();
                    for attr in e.attributes().flatten() {
                        let key =
                            std::str::from_utf8(attr.key.as_ref()).unwrap_or("").to_string();
                        let val = std::str::from_utf8(&attr.value).unwrap_or("").to_string();
                        attrs.push((name.clone(), key, val));
                    }
                    current_tag = Some(name);
                    current_text.clear();
                }
                Ok(Event::Text(ref e)) => {
                    if current_tag.is_some() {
                        let text = e.unescape()
                            .unwrap_or_else(|_| std::borrow::Cow::Borrowed(
                                std::str::from_utf8(e.as_ref()).unwrap_or("")
                            ));
                        current_text.push_str(&text);
                    }
                }
                Ok(Event::End(_)) => {
                    if let Some(tag) = current_tag.take() {
                        if !current_text.is_empty() {
                            fields.push((tag, current_text.clone()));
                        }
                        current_text.clear();
                    }
                }
                Err(e) => return Err(ParseError::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(Self { fields, attrs })
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn require(&self, name: &str) -> Result<&str, ParseError> {
        self.get(name)
            .ok_or_else(|| ParseError::MissingField(name.to_string()))
    }

    fn get_attr(&self, tag: &str, attr: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(t, k, _)| t == tag && k == attr)
            .map(|(_, _, v)| v.as_str())
    }
}

fn parse_id_instance(p: &MiniParser) -> Result<i64, ParseError> {
    let raw = p.require("IdInstance")?;
    raw.parse().map_err(|_| ParseError::InvalidValue {
        field: "IdInstance".into(),
        value: raw.into(),
    })
}

fn parse_unite_legale(xml: &str) -> Result<UniteLegale, ParseError> {
    let p = MiniParser::parse(xml)?;
    Ok(UniteLegale {
        id_instance: parse_id_instance(&p)?,
        motif_presence: MotifPresence::from_code(p.require("MotifPresence")?)
            .ok_or_else(|| ParseError::InvalidValue {
                field: "MotifPresence".into(),
                value: p.get("MotifPresence").unwrap_or("").into(),
            })?,
        statut: Statut::from_code(p.require("Statut")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Statut".into(),
                value: p.get("Statut").unwrap_or("").into(),
            }
        })?,
        siren: p.require("IdSIREN")?.to_string(),
        nom: p.require("Nom")?.to_string(),
        type_entite: TypeEntite::from_code(p.require("TypeEntite")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "TypeEntite".into(),
                value: p.get("TypeEntite").unwrap_or("").into(),
            }
        })?,
        diffusible: Diffusible::from_code(p.require("Diffusible")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Diffusible".into(),
                value: p.get("Diffusible").unwrap_or("").into(),
            }
        })?,
    })
}

fn parse_etablissement(xml: &str) -> Result<Etablissement, ParseError> {
    let p = MiniParser::parse(xml)?;

    let donnees_b2g = if p.get("EngagementJuridique").is_some() {
        Some(DonneesB2G {
            engagement_juridique: p.get("EngagementJuridique") == Some("true"),
            service: p.get("Service") == Some("true"),
            eng_jur_serv: p.get("EngJurServ") == Some("true"),
            moa: p.get("MOA") == Some("true"),
            moa_unique: p.get("MOAunique") == Some("true"),
            statut_mise_en_paiement: p.get("StatutMiseEnPaiement") == Some("true"),
        })
    } else {
        None
    };

    Ok(Etablissement {
        id_instance: parse_id_instance(&p)?,
        motif_presence: MotifPresence::from_code(p.require("MotifPresence")?)
            .ok_or_else(|| ParseError::InvalidValue {
                field: "MotifPresence".into(),
                value: p.get("MotifPresence").unwrap_or("").into(),
            })?,
        statut: Statut::from_code(p.require("Statut")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Statut".into(),
                value: p.get("Statut").unwrap_or("").into(),
            }
        })?,
        siret: p.require("IdSIRET")?.to_string(),
        type_etablissement: TypeEtablissement::from_code(
            p.require("TypeEtablissement")?,
        )
        .ok_or_else(|| ParseError::InvalidValue {
            field: "TypeEtablissement".into(),
            value: p.get("TypeEtablissement").unwrap_or("").into(),
        })?,
        nom: p.require("Nom")?.to_string(),
        adresse_1: p.get("LigneAdresse1").map(String::from),
        adresse_2: p.get("LigneAdresse2").map(String::from),
        adresse_3: p.get("LigneAdresse3").map(String::from),
        localite: p.get("Localite").map(String::from),
        code_postal: p.get("CP").map(String::from),
        subdivision_pays: p.get("SubdivisionPays").map(String::from),
        code_pays: p.get("CodePays").map(String::from),
        donnees_b2g,
        diffusible: Diffusible::from_code(p.require("Diffusible")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Diffusible".into(),
                value: p.get("Diffusible").unwrap_or("").into(),
            }
        })?,
    })
}

fn parse_code_routage(xml: &str) -> Result<CodeRoutage, ParseError> {
    let p = MiniParser::parse(xml)?;
    Ok(CodeRoutage {
        id_instance: parse_id_instance(&p)?,
        motif_presence: MotifPresence::from_code(p.require("MotifPresence")?)
            .ok_or_else(|| ParseError::InvalidValue {
                field: "MotifPresence".into(),
                value: p.get("MotifPresence").unwrap_or("").into(),
            })?,
        statut: Statut::from_code(p.require("Statut")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Statut".into(),
                value: p.get("Statut").unwrap_or("").into(),
            }
        })?,
        siret: p.require("IdSIRET")?.to_string(),
        id_routage: p.require("IdRoutage")?.to_string(),
        qualifiant_routage: p
            .get_attr("IdRoutage", "qualifiant")
            .unwrap_or("0224")
            .to_string(),
        nom: p.require("Nom")?.to_string(),
        adresse_1: p.get("LigneAdresse1").map(String::from),
        adresse_2: p.get("LigneAdresse2").map(String::from),
        adresse_3: p.get("LigneAdresse3").map(String::from),
        localite: p.get("Localite").map(String::from),
        code_postal: p.get("CP").map(String::from),
        subdivision_pays: p.get("SubdivisionPays").map(String::from),
        code_pays: p.get("CodePays").map(String::from),
        engagement_juridique: p.get("EngagementJuridique").map(|v| v == "true"),
    })
}

fn parse_plateforme(xml: &str) -> Result<Plateforme, ParseError> {
    let p = MiniParser::parse(xml)?;
    Ok(Plateforme {
        id_instance: parse_id_instance(&p)?,
        motif_presence: MotifPresence::from_code(p.require("MotifPresence")?)
            .ok_or_else(|| ParseError::InvalidValue {
                field: "MotifPresence".into(),
                value: p.get("MotifPresence").unwrap_or("").into(),
            })?,
        statut: Statut::from_code(p.require("Statut")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Statut".into(),
                value: p.get("Statut").unwrap_or("").into(),
            }
        })?,
        type_plateforme: TypePlateforme::from_code(p.require("TypePlateforme")?)
            .ok_or_else(|| ParseError::InvalidValue {
                field: "TypePlateforme".into(),
                value: p.get("TypePlateforme").unwrap_or("").into(),
            })?,
        matricule: p.require("Matricule")?.to_string(),
        siren: p.get("IdSIREN").map(String::from),
        nom: p.require("Nom")?.to_string(),
        nom_commercial: p.get("NomCommercial").map(String::from),
        contact: p.get("Contact").map(String::from),
        date_debut_immatriculation: p.require("DateDebutImmatriculation")?.to_string(),
        date_fin_immatriculation: p.get("DateFinImmatriculation").map(String::from),
    })
}

fn parse_ligne_annuaire(xml: &str) -> Result<LigneAnnuaire, ParseError> {
    let p = MiniParser::parse(xml)?;
    Ok(LigneAnnuaire {
        id_instance: parse_id_instance(&p)?,
        motif_presence: MotifPresence::from_code(p.require("MotifPresence")?)
            .ok_or_else(|| ParseError::InvalidValue {
                field: "MotifPresence".into(),
                value: p.get("MotifPresence").unwrap_or("").into(),
            })?,
        nature: NatureLigne::from_code(p.require("Nature")?).ok_or_else(|| {
            ParseError::InvalidValue {
                field: "Nature".into(),
                value: p.get("Nature").unwrap_or("").into(),
            }
        })?,
        date_debut: p.require("DateDebut")?.to_string(),
        date_fin: p.get("DateFin").map(String::from),
        date_fin_effective: p.get("DateFinEffective").map(String::from),
        identifiant: p.require("Identifiant")?.to_string(),
        siren: p.require("IdLinSIREN")?.to_string(),
        siret: p.get("IdLinSIRET").map(String::from),
        id_routage: p.get("IdLinRoutage").map(String::from),
        suffixe: p.get("Suffixe").map(String::from),
        id_plateforme: p.require("IdPlateforme")?.to_string(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlocState {
    None,
    UnitesLegales,
    Etablissements,
    CodesRoutage,
    Plateformes,
    LignesAnnuaire,
}

#[derive(Default)]
struct HeaderBuilder {
    horodate: Option<String>,
    dernier_horodate: Option<String>,
    type_flux: Option<String>,
}

impl HeaderBuilder {
    fn build(&self) -> Option<F14Header> {
        let horodate = self.horodate.clone()?;
        let type_flux_str = self.type_flux.as_deref()?;
        let type_flux = TypeFlux::from_code(type_flux_str)?;
        Some(F14Header {
            horodate_production: horodate,
            dernier_horodate_production: self.dernier_horodate.clone(),
            type_flux,
        })
    }
}
