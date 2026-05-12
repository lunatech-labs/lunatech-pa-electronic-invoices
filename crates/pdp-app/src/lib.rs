//! Bibliothèque de l'application PDP : expose les modules `server`, `ui`,
//! `webhooks` aux tests d'intégration. Le binaire `pdp` (cf. `[[bin]]` dans
//! `Cargo.toml`) garde son propre `main.rs` qui inclut ces modules de façon
//! identique pour ne rien dupliquer.

pub mod security;
pub mod server;
pub mod session;
pub mod ui;
pub mod webhooks;
pub mod webhooks_subscriber;
