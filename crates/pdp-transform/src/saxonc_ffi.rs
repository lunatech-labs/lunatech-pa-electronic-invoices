//! Bindings FFI pour SaxonC-HE (GraalVM native) — API C simple.
//!
//! Lie dynamiquement libsaxonc-he uniquement.
//! Utilise l'API C de SaxonC pour :
//! - Créer un isolat GraalVM (une seule fois)
//! - Créer un processeur Saxon (réutilisé entre les appels)
//! - Exécuter des transformations XSLT en mémoire (pas de fork/exec)
//!
//! Gain attendu : ~900ms par appel XSLT (compilation XSLT évitée après le 1er appel
//! car le processeur GraalVM garde un cache interne des stylesheets).

#![allow(non_camel_case_types, dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;

// --- Opaque GraalVM types ---

#[repr(C)]
pub struct graal_isolate_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct graal_isolatethread_t {
    _private: [u8; 0],
}

// --- Structs from SaxonCGlue.h ---

/// sxnc_environment — GraalVM environment
#[repr(C)]
pub struct sxnc_environment {
    pub isolate: *mut graal_isolate_t,
    pub thread: *mut graal_isolatethread_t,
    pub mainthread: *mut graal_isolatethread_t,
}

/// sxnc_parameter — (name, value) pair for XSLT parameters
#[repr(C)]
pub struct sxnc_parameter {
    pub name: *mut c_char,
    pub value: i64,
}

/// sxnc_property — (name, value) string pair for XSLT properties
#[repr(C)]
pub struct sxnc_property {
    pub name: *mut c_char,
    pub value: *mut c_char,
}

/// sxnc_processor — Saxon processor handle
#[repr(C)]
pub struct sxnc_processor {
    pub value: i64,
}

// --- Extern C functions from libsaxonc-he ---

extern "C" {
    pub fn initSaxonc(
        environi: *mut *mut sxnc_environment,
        proc: *mut *mut sxnc_processor,
        param: *mut *mut sxnc_parameter,
        prop: *mut *mut sxnc_property,
        cap: c_int,
        prop_cap: c_int,
    );

    pub fn freeSaxonc(
        environi: *mut *mut sxnc_environment,
        proc: *mut *mut sxnc_processor,
        param: *mut *mut sxnc_parameter,
        prop: *mut *mut sxnc_property,
    );

    pub fn create_graalvm_isolate(env: *mut sxnc_environment) -> c_int;

    pub fn graal_tear_down(thread: *mut graal_isolatethread_t);

    pub fn c_createSaxonProcessor(
        environi: *mut sxnc_environment,
        processor: *mut sxnc_processor,
        license: c_int,
    ) -> c_int;

    pub fn xsltApplyStylesheet(
        environi: *mut sxnc_environment,
        proc: *mut sxnc_processor,
        cwd: *mut c_char,
        source: *mut c_char,
        stylesheet: *mut c_char,
        parameters: *mut sxnc_parameter,
        properties: *mut sxnc_property,
        par_len: c_int,
        prop_len: c_int,
    ) -> *const c_char;

    pub fn xsltSaveResultToFile(
        environi: *mut sxnc_environment,
        proc: *mut sxnc_processor,
        cwd: *mut c_char,
        source: *mut c_char,
        stylesheet: *mut c_char,
        output: *mut c_char,
        parameters: *mut sxnc_parameter,
        properties: *mut sxnc_property,
        par_len: c_int,
        prop_len: c_int,
    );

    pub fn c_getErrorMessage(environi: *mut sxnc_environment) -> *const c_char;

    pub fn checkForException(environi: *mut sxnc_environment) -> *const c_char;

    pub fn setProperty(
        properties: *mut *mut sxnc_property,
        prop_len: *mut c_int,
        prop_cap: *mut c_int,
        name: *const c_char,
        value: *const c_char,
    );
}

// --- Safe Rust wrapper ---

/// Processeur SaxonC in-process. Crée un isolat GraalVM une seule fois
/// et réutilise le processeur Saxon pour toutes les transformations.
pub struct SaxonCProcessor {
    env: *mut sxnc_environment,
    proc: *mut sxnc_processor,
    params: *mut sxnc_parameter,
    props: *mut sxnc_property,
}

// SaxonC GraalVM est thread-safe une fois l'isolat créé
unsafe impl Send for SaxonCProcessor {}
unsafe impl Sync for SaxonCProcessor {}

const PARAM_CAP: c_int = 10;

impl SaxonCProcessor {
    /// Crée un nouveau processeur SaxonC.
    /// Retourne None si la bibliothèque SaxonC n'est pas disponible.
    pub fn new() -> Option<Self> {
        unsafe {
            let mut env: *mut sxnc_environment = ptr::null_mut();
            let mut proc: *mut sxnc_processor = ptr::null_mut();
            let mut params: *mut sxnc_parameter = ptr::null_mut();
            let mut props: *mut sxnc_property = ptr::null_mut();

            initSaxonc(&mut env, &mut proc, &mut params, &mut props, PARAM_CAP, PARAM_CAP);

            if env.is_null() || proc.is_null() {
                tracing::warn!("SaxonC FFI: initSaxonc a retourné des pointeurs null");
                return None;
            }

            let rc = create_graalvm_isolate(env);
            if rc != 0 {
                tracing::warn!("SaxonC FFI: impossible de créer l'isolat GraalVM (rc={})", rc);
                return None;
            }

            let ok = c_createSaxonProcessor(env, proc, 0);
            if ok == 0 {
                tracing::warn!("SaxonC FFI: impossible de créer le processeur Saxon");
                graal_tear_down((*env).thread);
                return None;
            }

            tracing::info!("SaxonC FFI: processeur initialisé (in-process, pas de fork/exec)");

            Some(Self { env, proc, params, props })
        }
    }

    /// Transforme un fichier XML source avec un fichier XSLT, retourne le résultat en String.
    pub fn transform_file_to_string(
        &self,
        source_file: &Path,
        stylesheet_file: &Path,
        params: &[(&str, &str)],
    ) -> Result<String, String> {
        let cwd = stylesheet_file.parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");

        let cwd_c = CString::new(cwd).map_err(|e| e.to_string())?;
        let source_c = CString::new(
            source_file.to_str().unwrap_or("")
        ).map_err(|e| e.to_string())?;
        let stylesheet_c = CString::new(
            stylesheet_file.to_str().unwrap_or("")
        ).map_err(|e| e.to_string())?;

        // Build properties for XSLT parameters (Saxon C API uses properties for params)
        let mut prop_storage: Vec<(CString, CString)> = Vec::new();
        let mut c_props: Vec<sxnc_property> = Vec::new();
        for (key, value) in params {
            let k = CString::new(*key).map_err(|e| e.to_string())?;
            let v = CString::new(*value).map_err(|e| e.to_string())?;
            prop_storage.push((k, v));
        }
        for (k, v) in &prop_storage {
            c_props.push(sxnc_property {
                name: k.as_ptr() as *mut c_char,
                value: v.as_ptr() as *mut c_char,
            });
        }

        let prop_ptr = if c_props.is_empty() { ptr::null_mut() } else { c_props.as_mut_ptr() };
        let prop_len = c_props.len() as c_int;

        unsafe {
            let result = xsltApplyStylesheet(
                self.env,
                self.proc,
                cwd_c.as_ptr() as *mut c_char,
                source_c.as_ptr() as *mut c_char,
                stylesheet_c.as_ptr() as *mut c_char,
                ptr::null_mut(), // no sxnc_parameter (XDM values)
                prop_ptr,
                0,
                prop_len,
            );

            if result.is_null() {
                let err = self.get_error();
                return Err(err.unwrap_or_else(|| "SaxonC: résultat null, erreur inconnue".to_string()));
            }

            let result_str = CStr::from_ptr(result).to_string_lossy().into_owned();
            Ok(result_str)
        }
    }

    /// Transforme un fichier XML source avec un fichier XSLT, écrit le résultat dans un fichier.
    pub fn transform_file_to_file(
        &self,
        source_file: &Path,
        stylesheet_file: &Path,
        output_file: &Path,
        params: &[(&str, &str)],
    ) -> Result<(), String> {
        let cwd = stylesheet_file.parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");

        let cwd_c = CString::new(cwd).map_err(|e| e.to_string())?;
        let source_c = CString::new(
            source_file.to_str().unwrap_or("")
        ).map_err(|e| e.to_string())?;
        let stylesheet_c = CString::new(
            stylesheet_file.to_str().unwrap_or("")
        ).map_err(|e| e.to_string())?;
        let output_c = CString::new(
            output_file.to_str().unwrap_or("")
        ).map_err(|e| e.to_string())?;

        let mut prop_storage: Vec<(CString, CString)> = Vec::new();
        let mut c_props: Vec<sxnc_property> = Vec::new();
        for (key, value) in params {
            let k = CString::new(*key).map_err(|e| e.to_string())?;
            let v = CString::new(*value).map_err(|e| e.to_string())?;
            prop_storage.push((k, v));
        }
        for (k, v) in &prop_storage {
            c_props.push(sxnc_property {
                name: k.as_ptr() as *mut c_char,
                value: v.as_ptr() as *mut c_char,
            });
        }

        let prop_ptr = if c_props.is_empty() { ptr::null_mut() } else { c_props.as_mut_ptr() };
        let prop_len = c_props.len() as c_int;

        unsafe {
            xsltSaveResultToFile(
                self.env,
                self.proc,
                cwd_c.as_ptr() as *mut c_char,
                source_c.as_ptr() as *mut c_char,
                stylesheet_c.as_ptr() as *mut c_char,
                output_c.as_ptr() as *mut c_char,
                ptr::null_mut(),
                prop_ptr,
                0,
                prop_len,
            );

            // Check for errors
            let err = self.get_error();
            if let Some(msg) = err {
                return Err(msg);
            }

            if !output_file.exists() {
                return Err(format!("SaxonC: fichier de sortie non créé: {}", output_file.display()));
            }

            Ok(())
        }
    }

    /// Récupère le message d'erreur courant.
    fn get_error(&self) -> Option<String> {
        unsafe {
            let msg = c_getErrorMessage(self.env);
            if msg.is_null() {
                None
            } else {
                let s = CStr::from_ptr(msg).to_string_lossy().into_owned();
                if s.is_empty() { None } else { Some(s) }
            }
        }
    }
}

impl Drop for SaxonCProcessor {
    fn drop(&mut self) {
        unsafe {
            if !self.env.is_null() && !(*self.env).thread.is_null() {
                graal_tear_down((*self.env).thread);
            }
            freeSaxonc(&mut self.env, &mut self.proc, &mut self.params, &mut self.props);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
    }

    #[test]
    fn test_saxonc_processor_creation() {
        let proc = SaxonCProcessor::new();
        assert!(proc.is_some(), "SaxonC processor should be created");
    }

    #[test]
    fn test_saxonc_cii_to_xr() {
        let proc = SaxonCProcessor::new().expect("SaxonC not available");
        let specs = specs_dir();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/cii/facture_cii_001.xml");
        let xsl = specs.join("xslt/mustang/stylesheets/cii-xr.xsl");

        let result = proc.transform_file_to_string(&source, &xsl, &[]);
        assert!(result.is_ok(), "CII→XR should succeed: {:?}", result.err());
        let xr = result.unwrap();
        assert!(xr.contains("xr:"), "Result should contain XR namespace");
    }

    #[test]
    fn test_saxonc_xr_to_fo() {
        let proc = SaxonCProcessor::new().expect("SaxonC not available");
        let specs = specs_dir();

        // First: CII → XR
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/cii/facture_cii_001.xml");
        let xsl_xr = specs.join("xslt/mustang/stylesheets/cii-xr.xsl");
        let xr = proc.transform_file_to_string(&source, &xsl_xr, &[]).unwrap();

        // Write XR to temp file
        let tmp_xr = std::env::temp_dir().join("test_saxonc_xr.xml");
        std::fs::write(&tmp_xr, &xr).unwrap();

        // Then: XR → FO
        let xsl_fo = specs.join("xslt/mustang/stylesheets/xr-pdf.xsl");
        let result = proc.transform_file_to_string(
            &tmp_xr, &xsl_fo,
            &[("foengine", "fop"), ("lang", "fr")],
        );
        let _ = std::fs::remove_file(&tmp_xr);

        assert!(result.is_ok(), "XR→FO should succeed: {:?}", result.err());
        let fo = result.unwrap();
        assert!(fo.contains("fo:root") || fo.contains("XSL/Format"),
            "Result should contain FO namespace");
    }

    #[test]
    fn test_saxonc_full_pipeline_timing() {
        use std::time::Instant;

        let proc = SaxonCProcessor::new().expect("SaxonC not available");
        let specs = specs_dir();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/cii/facture_cii_001.xml");
        let xsl_xr = specs.join("xslt/mustang/stylesheets/cii-xr.xsl");
        let xsl_fo = specs.join("xslt/mustang/stylesheets/xr-pdf.xsl");

        // Run 1: cold (includes XSLT compilation)
        let t1 = Instant::now();
        let xr = proc.transform_file_to_string(&source, &xsl_xr, &[]).unwrap();
        let cold_xr = t1.elapsed();

        let tmp_xr = std::env::temp_dir().join("test_saxonc_timing_xr.xml");
        std::fs::write(&tmp_xr, &xr).unwrap();

        let t2 = Instant::now();
        let _fo = proc.transform_file_to_string(
            &tmp_xr, &xsl_fo,
            &[("foengine", "fop"), ("lang", "fr")],
        ).unwrap();
        let cold_fo = t2.elapsed();

        // Run 2: warm (XSLT should be cached by GraalVM)
        let t3 = Instant::now();
        let xr2 = proc.transform_file_to_string(&source, &xsl_xr, &[]).unwrap();
        let warm_xr = t3.elapsed();

        std::fs::write(&tmp_xr, &xr2).unwrap();

        let t4 = Instant::now();
        let _fo2 = proc.transform_file_to_string(
            &tmp_xr, &xsl_fo,
            &[("foengine", "fop"), ("lang", "fr")],
        ).unwrap();
        let warm_fo = t4.elapsed();

        let _ = std::fs::remove_file(&tmp_xr);

        eprintln!("=== SaxonC FFI Timing ===");
        eprintln!("CII→XR cold: {:?}", cold_xr);
        eprintln!("XR→FO cold:  {:?}", cold_fo);
        eprintln!("CII→XR warm: {:?}", warm_xr);
        eprintln!("XR→FO warm:  {:?}", warm_fo);
        eprintln!("Total cold:  {:?}", cold_xr + cold_fo);
        eprintln!("Total warm:  {:?}", warm_xr + warm_fo);
    }
}
