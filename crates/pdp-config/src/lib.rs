//! # pdp-config — Configuration YAML de la PDP
//!
//! Chargement et validation de la configuration globale de la plateforme
//! depuis un fichier YAML (`config.yaml`).
//!
//! # Modules
//!
//! - **[`model`]** — Structures de configuration : [`PdpConfig`], routes, PPF, AFNOR, PISTE, partenaires
//! - **[`loader`]** — Chargement depuis fichier YAML avec [`load_config`]
//!
//! # Sections de configuration
//!
//! - **`pdp`** — Identité de la PDP (SIREN, nom, code application)
//! - **`database`** — Connexion PostgreSQL (traçabilité)
//! - **`routes`** — Routes de traitement (source, destination, validation, CDV)
//! - **`ppf`** — Configuration SFTP PPF (hôte, clé RSA, SAS)
//! - **`afnor`** — Configuration AFNOR Flow/Directory Service (URL, auth)
//! - **`piste`** — Authentification PISTE (client_id, client_secret, URL token)
//! - **`partners`** — Partenaires PDP connus (SIREN, URL, certificats)
//!
//! # Exemple
//!
//! ```no_run
//! use pdp_config::{load_config, PdpConfig};
//!
//! let config = load_config("config.yaml").unwrap();
//! println!("PDP : {} ({})", config.pdp.name, config.pdp.siren.as_deref().unwrap_or("N/A"));
//! for route in &config.routes {
//!     println!("Route : {}", route.id);
//! }
//! ```
//!
//! # Fichier config.yaml
//!
//! ```yaml
//! pdp:
//!   siren: "123456789"
//!   name: "Ma PDP"
//!   code_application: "AAA123"
//!
//! database:
//!   url: "postgresql://pdp:pdp@localhost:5432/pdp_trace"
//!
//! routes:
//!   - id: route-ubl
//!     source: { type: file, path: ./data/in/ubl }
//!     destination: { type: file, path: ./data/out }
//!     validate: true
//! ```

pub mod model;
pub mod loader;
pub mod tenant;
pub mod registry;

pub use model::{PdpConfig, TenantConfig, DatabaseConfig};
pub use loader::load_config;
pub use tenant::{TenantEntry, discover_tenants, synthetic_tenant, is_valid_siren};
pub use registry::{TenantRegistry, TenantRegistryEntry};
