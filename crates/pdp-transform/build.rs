/// Build script pour pdp-transform : lie dynamiquement libsaxonc-he si disponible.
///
/// SaxonC-HE est installé dans /usr/local/lib (dylibs) et /opt/saxonc/.../include (headers).
/// Si les bibliothèques ne sont pas trouvées, la compilation continue sans FFI SaxonC
/// (le code utilise un feature flag `saxonc_ffi` pour conditionner l'utilisation).

fn main() {
    // Chercher libsaxonc-he dans les emplacements standards
    let lib_dirs = [
        "/usr/local/lib",
        "/opt/saxonc/SaxonCHE-macos-arm64-12-9-0/SaxonCHE/lib",
        "/opt/saxonc/lib",
    ];

    let mut found = false;
    for dir in &lib_dirs {
        let path = std::path::Path::new(dir);
        if path.join("libsaxonc-he.dylib").exists() || path.join("libsaxonc-he.so").exists() {
            println!("cargo:rustc-link-search=native={}", dir);
            found = true;
            break;
        }
    }

    if found {
        println!("cargo:rustc-link-lib=dylib=saxonc-he");
        println!("cargo:rustc-cfg=feature=\"saxonc_ffi\"");
        println!("cargo:warning=SaxonC-HE found, enabling in-process XSLT (saxonc_ffi)");
    } else {
        println!("cargo:warning=SaxonC-HE not found, using fork/exec fallback for XSLT");
    }

    // Rerun if library paths change
    for dir in &lib_dirs {
        println!("cargo:rerun-if-changed={}", dir);
    }
}
