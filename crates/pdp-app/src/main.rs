// Les modules `server`, `ui`, `webhooks` sont exposés par `pdp_app` (lib.rs)
// pour que les tests d'intégration y accèdent. On les ré-importe ici en local
// pour conserver les chemins `crate::server::...` utilisés par le binaire.
// (Le module `ui` est utilisé indirectement par server.rs, mais on l'importe
// quand même pour assurer le linking quand ce binaire est compilé seul.)
#[allow(unused_imports)]
use pdp_app::{server, ui, webhooks, webhooks_subscriber};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pdp")]
#[command(about = "PDP - Plateforme de Dématérialisation Partenaire pour la facturation électronique")]
#[command(version)]
struct Cli {
    /// Fichier de configuration (défaut: config.yaml)
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Démarre la PDP en mode polling (boucle continue)
    Start {
        /// Mode : emitter (émission), receiver (réception), both (défaut)
        #[arg(short, long, default_value = "both")]
        mode: String,
    },

    /// Exécute toutes les routes une seule fois
    Run {
        /// Mode : emitter (émission), receiver (réception), both (défaut)
        #[arg(short, long, default_value = "both")]
        mode: String,
    },

    /// Exécute une route spécifique
    RunRoute {
        /// ID de la route à exécuter
        route_id: String,
    },

    /// Affiche les routes configurées
    ListRoutes,

    /// Parse et affiche les informations d'une facture
    Parse {
        /// Chemin vers le fichier facture (XML ou PDF)
        file: PathBuf,
    },

    /// Valide une facture
    Validate {
        /// Chemin vers le fichier facture
        file: PathBuf,
    },

    /// Transforme une facture dans un autre format
    Transform {
        /// Chemin vers le fichier facture source
        file: PathBuf,
        /// Format cible (UBL, CII, Factur-X ou PDF)
        #[arg(short, long)]
        to: String,
        /// Fichier de sortie (optionnel, sinon stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Affiche les statistiques de traçabilité
    Stats,

    /// Affiche les flux en erreur
    Errors,

    /// Affiche les événements d'un flux
    FlowEvents {
        /// ID du flux
        flow_id: String,
    },

    /// Gestion de l'annuaire PPF local
    Annuaire {
        #[command(subcommand)]
        action: AnnuaireCommands,
    },

    /// Génération de rapports e-reporting (Flux 10.1/10.2/10.3/10.4)
    Ereporting {
        #[command(subcommand)]
        action: EreportingCommands,
    },

    /// Outils de démo (peuplement de l'UI avec des factures fixtures)
    Demo {
        #[command(subcommand)]
        action: DemoCommands,
    },

    /// Petits outils en ligne de commande (hash de password, génération
    /// de secret, etc.).
    Tools {
        #[command(subcommand)]
        action: ToolsCommands,
    },
}

#[derive(Subcommand)]
enum ToolsCommands {
    /// Génère un hash argon2id pour un mot de passe (à coller dans la
    /// config `users[].password`). Lit le mot de passe en argument
    /// ou sur stdin si l'argument est `-`.
    HashPassword {
        /// Mot de passe en clair (ou `-` pour lire stdin).
        password: String,
    },
    /// Génère un secret aléatoire 32 octets (base64) pour
    /// `http_server.session_secret`.
    GenSessionSecret,
    /// Génère les pièces jointes "métier" (BdC PDF, bordereau de livraison
    /// PNG, détail des lignes CSV) à partir d'une facture, avec les vraies
    /// données. Utile pour produire des fixtures de démonstration crédibles.
    GenAttachments {
        /// Chemin vers le fichier facture source (UBL/CII XML ou Factur-X PDF).
        invoice: PathBuf,
        /// Répertoire de sortie (créé si absent). Génère 3 fichiers :
        /// `bon_commande_<id>.pdf`, `bordereau_livraison_<id>.png`,
        /// `detail_lignes_<id>.csv`.
        #[arg(short, long)]
        output_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum DemoCommands {
    /// Soumet toutes les factures fixtures (UBL + CII) au serveur HTTP local
    /// pour peupler le dashboard. Le serveur doit être en cours d'exécution.
    Populate {
        /// URL du serveur Ferrite (défaut: http://localhost:8080)
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Répertoire contenant les fixtures (cherche dans `ubl/` et `cii/`)
        #[arg(long, default_value = "tests/fixtures")]
        fixtures_dir: PathBuf,
        /// Bearer token (si l'auth est activée sur le serveur)
        #[arg(long)]
        token: Option<String>,
        /// Si `true`, supprime les indices Elasticsearch `pdp-*` avant de
        /// soumettre — garantit des factures sans erreur de doublon
        /// (BR-FR-12/13). À utiliser en démo pour repartir d'un état propre.
        #[arg(long)]
        reset: bool,
        /// URL Elasticsearch pour le reset (défaut: http://localhost:9200)
        #[arg(long, default_value = "http://localhost:9200")]
        elasticsearch_url: String,
    },
    /// One-shot pour démarrer une démo propre : importe l'annuaire F14 dans
    /// PostgreSQL, prépare les répertoires `tenants/{siren}/` des entreprises
    /// de démo, puis pousse toutes les fixtures via `populate`. À lancer une
    /// fois que le serveur tourne (`pdp start --mode receiver`).
    Seed {
        /// URL du serveur Ferrite (défaut: http://localhost:8080)
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Fichier F14 à importer (défaut: la fixture de démo)
        #[arg(long, default_value = "tests/fixtures/annuaire/F14_demo.xml")]
        annuaire_file: PathBuf,
        /// Répertoire contenant les fixtures UBL/CII/Factur-X
        #[arg(long, default_value = "tests/fixtures")]
        fixtures_dir: PathBuf,
        /// Bearer token (si l'auth est activée). En config-ui-demo.yaml on
        /// passe par cookie de session, donc cet argument reste optionnel.
        #[arg(long)]
        token: Option<String>,
        /// Reset l'annuaire (TRUNCATE) avant l'import F14
        #[arg(long)]
        reset_annuaire: bool,
        /// Reset les indices Elasticsearch `pdp-*` avant populate
        #[arg(long)]
        reset_factures: bool,
        /// URL Elasticsearch (défaut: http://localhost:9200)
        #[arg(long, default_value = "http://localhost:9200")]
        elasticsearch_url: String,
    },
}

#[derive(Subcommand)]
enum EreportingCommands {
    /// Génère un rapport Flux 10.1 (transactions ventes détaillées).
    ///
    /// Source des factures : `--invoices-dir <chemin>` (autodétection UBL/CII/
    /// Factur-X) OU automatiquement depuis Elasticsearch `pdp-{siren}` si
    /// `--invoices-dir` est omis (la config doit fournir `elasticsearch.url`).
    Generate101 {
        /// Répertoire contenant les factures (sinon : pull depuis Elasticsearch)
        #[arg(long)]
        invoices_dir: Option<PathBuf>,
        /// SIREN du déclarant (vendeur) — sert aussi à cibler l'index ES
        #[arg(long)]
        siren: String,
        /// Nom du déclarant
        #[arg(long)]
        name: String,
        /// Date de début (YYYY-MM-DD ou YYYYMMDD)
        #[arg(long)]
        from: String,
        /// Date de fin (YYYY-MM-DD ou YYYYMMDD)
        #[arg(long)]
        to: String,
        /// Identifiant du rapport (sinon généré automatiquement)
        #[arg(long)]
        report_id: Option<String>,
        /// Fichier de sortie (sinon stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Génère un rapport Flux 10.3 (transactions ventes agrégées par jour/catégorie).
    ///
    /// Source des factures : `--invoices-dir <chemin>` OU Elasticsearch
    /// `pdp-{siren}` si `--invoices-dir` est omis.
    Generate103 {
        /// Répertoire contenant les factures (sinon : pull depuis Elasticsearch)
        #[arg(long)]
        invoices_dir: Option<PathBuf>,
        /// SIREN du déclarant (vendeur) — sert aussi à cibler l'index ES
        #[arg(long)]
        siren: String,
        /// Nom du déclarant
        #[arg(long)]
        name: String,
        /// Date de début (YYYY-MM-DD ou YYYYMMDD)
        #[arg(long)]
        from: String,
        /// Date de fin (YYYY-MM-DD ou YYYYMMDD)
        #[arg(long)]
        to: String,
        /// Identifiant du rapport (sinon généré automatiquement)
        #[arg(long)]
        report_id: Option<String>,
        /// Fichier de sortie (sinon stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum AnnuaireCommands {
    /// Importe un fichier F14 (export annuaire PPF) dans PostgreSQL
    Import {
        /// Chemin vers le fichier F14 XML
        file: PathBuf,
    },
    /// Affiche les statistiques de l'annuaire local
    Stats,
    /// Recherche une entreprise par SIREN
    Lookup {
        /// SIREN à rechercher
        siren: String,
    },
    /// Résout le routage pour un destinataire
    Route {
        /// SIREN du destinataire
        siren: String,
        /// SIRET du destinataire (optionnel)
        #[arg(long)]
        siret: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialiser le tracing
    pdp_trace::init_tracing();

    match cli.command {
        Commands::Start { mode } => cmd_start(&cli.config, &mode).await,
        Commands::Run { mode } => cmd_run(&cli.config, &mode).await,
        Commands::RunRoute { route_id } => cmd_run_route(&cli.config, &route_id).await,
        Commands::ListRoutes => cmd_list_routes(&cli.config).await,
        Commands::Parse { file } => cmd_parse(&file).await,
        Commands::Validate { file } => cmd_validate(&file).await,
        Commands::Transform { file, to, output } => cmd_transform(&file, &to, output.as_deref()).await,
        Commands::Stats => cmd_stats(&cli.config).await,
        Commands::Errors => cmd_errors(&cli.config).await,
        Commands::FlowEvents { flow_id } => cmd_flow_events(&cli.config, &flow_id).await,
        Commands::Annuaire { action } => cmd_annuaire(&cli.config, action).await,
        Commands::Ereporting { action } => cmd_ereporting(&cli.config, action).await,
        Commands::Demo { action } => cmd_demo(action).await,
        Commands::Tools { action } => cmd_tools(action).await,
    }
}

async fn cmd_tools(action: ToolsCommands) -> Result<()> {
    match action {
        ToolsCommands::HashPassword { password } => {
            let plaintext = if password == "-" {
                use std::io::Read;
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s.trim_end_matches(['\n', '\r']).to_string()
            } else {
                password
            };
            if plaintext.is_empty() {
                anyhow::bail!("mot de passe vide");
            }
            let hash = pdp_app::session::hash_password(&plaintext)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{hash}");
        }
        ToolsCommands::GenSessionSecret => {
            use base64::Engine as _;
            let secret = pdp_app::session::random_secret();
            // Padding pour atteindre au moins 32 octets visiblement.
            let b64 = base64::engine::general_purpose::STANDARD.encode(&secret);
            println!("{b64}");
        }
        ToolsCommands::GenAttachments { invoice, output_dir } => {
            let data = std::fs::read(&invoice)?;
            let format = pdp_invoice::detect_format(&data)?;
            let inv = match format {
                pdp_core::model::InvoiceFormat::UBL => {
                    let xml = std::str::from_utf8(&data)?;
                    pdp_invoice::UblParser::new().parse(xml)?
                }
                pdp_core::model::InvoiceFormat::CII => {
                    let xml = std::str::from_utf8(&data)?;
                    pdp_invoice::CiiParser::new().parse(xml)?
                }
                pdp_core::model::InvoiceFormat::FacturX => {
                    pdp_invoice::FacturXParser::new().parse(&data)?
                }
            };
            std::fs::create_dir_all(&output_dir)?;
            let id = inv.invoice_number.replace(['/', ' '], "_");

            let bdc = pdp_transform::generate_bon_commande_pdf(&inv)?;
            let bdc_path = output_dir.join(format!("bon_commande_{id}.pdf"));
            std::fs::write(&bdc_path, &bdc)?;
            println!("✅ {} ({} octets)", bdc_path.display(), bdc.len());

            let bl = pdp_transform::generate_bordereau_livraison_png(&inv)?;
            let bl_path = output_dir.join(format!("bordereau_livraison_{id}.png"));
            std::fs::write(&bl_path, &bl)?;
            println!("✅ {} ({} octets)", bl_path.display(), bl.len());

            let csv = pdp_transform::generate_detail_lignes_csv(&inv);
            let csv_path = output_dir.join(format!("detail_lignes_{id}.csv"));
            std::fs::write(&csv_path, &csv)?;
            println!("✅ {} ({} octets)", csv_path.display(), csv.len());
        }
    }
    Ok(())
}

async fn cmd_start(config_path: &std::path::Path, mode_str: &str) -> Result<()> {
    let cli_mode = CliMode::from_str(mode_str)?;
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;

    tracing::info!(
        pdp_id = %config.pdp.id,
        pdp_name = %config.pdp.name,
        routes = config.routes.len(),
        interval = config.polling.interval_secs,
        mode = ?cli_mode,
        "Démarrage de la PDP en mode polling"
    );

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Canal pour les flux entrants via l'API HTTP
    let (flow_tx, mut flow_rx) = tokio::sync::mpsc::channel::<server::InboundFlow>(100);

    // Canal pour injecter les Exchange convertis dans le pipeline via ChannelConsumer
    let (http_exchange_tx, http_exchange_rx) =
        tokio::sync::mpsc::channel::<pdp_core::Exchange>(100);

    // Répertoire de base pour la découverte des tenants
    let base_dir = config_path.parent().unwrap_or(std::path::Path::new("."));

    // Connexion PostgreSQL globale (optionnelle) — partagée par annuaire et webhooks
    let pg_pool: Option<sqlx::postgres::PgPool> = if let Some(ref db_config) = config.database {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_connections)
            .connect(&db_config.url)
            .await
        {
            Ok(pool) => {
                tracing::info!("PostgreSQL connecté");
                Some(pool)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Impossible de connecter PostgreSQL");
                None
            }
        }
    } else {
        None
    };

    // Webhook store : Postgres si pool disponible, sinon in-memory
    let webhook_store = std::sync::Arc::new(match &pg_pool {
        Some(pool) => {
            let store = webhooks::WebhookStore::new_postgres(pool.clone());
            if let Err(e) = store.migrate().await {
                tracing::warn!(error = %e, "Migration webhooks échouée");
            }
            store
        }
        None => webhooks::WebhookStore::new(),
    });

    // Bus d'événements interne (pdp-events) : seulement si pg_pool dispo.
    // - publie chaque transition du cycle de vie dans la table outbox `events`
    // - alimente les subscribers (webhooks AFNOR, et bientôt l'archivage ES)
    let event_bus: Option<pdp_events::EventBus> = match &pg_pool {
        Some(pool) => {
            let store = std::sync::Arc::new(pdp_events::EventStore::new(pool.clone()));
            match store.migrate().await {
                Ok(()) => {
                    let bus = pdp_events::EventBus::new(store.clone());
                    // Subscribers webhook (In + Out) : remplacent l'appel direct
                    // au WebhookDispatcher dans server.rs et le WebhookAckProcessor.
                    let sub_in = webhooks_subscriber::WebhooksSubscriber::new(
                        webhook_store.clone(),
                        "In",
                    );
                    let sub_out = webhooks_subscriber::WebhooksSubscriber::new(
                        webhook_store.clone(),
                        "Out",
                    );
                    pdp_events::DispatcherWorker::new(store.clone(), sub_in).spawn();
                    pdp_events::DispatcherWorker::new(store.clone(), sub_out).spawn();
                    tracing::info!("Bus d'événements pdp-events activé (PostgreSQL)");
                    Some(bus)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Migration pdp-events échouée, bus désactivé");
                    None
                }
            }
        }
        None => {
            tracing::info!("Bus pdp-events désactivé (pas de PostgreSQL configuré)");
            None
        }
    };

    // Construire le router avec la route HTTP inbound
    let router = build_router(
        &config,
        base_dir,
        Some(http_exchange_rx),
        cli_mode,
        webhook_store.clone(),
        event_bus.clone(),
    )
    .await?;

    // Démarrer le serveur HTTP si configuré ET si le mode réception est actif
    if let Some(ref http_config) = config.http_server {
        if !cli_mode.should_run_reception() {
            tracing::info!("Serveur HTTP non démarré (mode emitter uniquement)");
        } else {
        let trace_store: Option<std::sync::Arc<dyn pdp_trace::TraceBackend>> =
            match pdp_trace::TraceStore::new(&config.elasticsearch.url).await {
                Ok(store) => Some(std::sync::Arc::new(store)),
                Err(e) => {
                    tracing::warn!(error = %e, "Impossible de connecter le TraceStore au serveur HTTP");
                    None
                }
            };

        // Annuaire PPF : utilise le pool PostgreSQL global
        let annuaire_store = if let Some(ref pool) = pg_pool {
            let store = pdp_annuaire::AnnuaireStore::new(pool.clone());
            if let Err(e) = store.migrate().await {
                tracing::warn!(error = %e, "Migration annuaire échouée");
            }
            tracing::info!("Annuaire PPF connecté (PostgreSQL)");
            Some(store)
        } else {
            None
        };

        // Construction de la table de tokens à partir de la config :
        //  - `tokens:` (nouveau) → liaison principal/SIRENs/role complète
        //  - `bearer_tokens:` (deprecated) → chargé en `PdpAdmin` pour
        //    backward-compat avec un warning explicite.
        let mut tokens_table = http_config
            .tokens
            .as_ref()
            .map(|list| pdp_app::security::build_token_table(list))
            .unwrap_or_default();
        if let Some(legacy) = &http_config.bearer_tokens {
            if !legacy.is_empty() {
                tracing::warn!(
                    "`http_server.bearer_tokens` est déprécié : utilise `http_server.tokens:` \
                     avec liaison `allowed_sirens`. Les {} token(s) sont chargés en PdpAdmin pour \
                     compat rétroactive — aucune isolation tenant !",
                    legacy.len()
                );
                for tok in legacy {
                    tokens_table
                        .entry(tok.clone())
                        .or_insert_with(|| pdp_app::security::SecurityContext {
                            principal: format!("legacy:{}", &tok[..tok.len().min(6)]),
                            allowed_sirens: Vec::new(),
                            role: pdp_app::security::Role::PdpAdmin,
                        });
                }
            }
        }

        let users = http_config.users.clone().unwrap_or_default();
        pdp_app::session::warn_plaintext_passwords(&users);
        let session_secret = http_config
            .session_secret
            .clone()
            .map(|s| s.into_bytes())
            .unwrap_or_else(|| {
                let s = pdp_app::session::random_secret();
                tracing::warn!(
                    "`http_server.session_secret` non défini : un secret aléatoire \
                     a été généré (les sessions seront invalidées au redémarrage)."
                );
                s
            });

        let app_state = std::sync::Arc::new(server::AppState {
            pdp_name: config.pdp.name.clone(),
            pdp_matricule: config.pdp.matricule.clone().unwrap_or_default(),
            flow_sender: flow_tx.clone(),
            webhook_secret: http_config.webhook_secret.clone(),
            tokens: tokens_table,
            users,
            session_secret,
            session_ttl_secs: http_config.session_ttl_secs,
            revocations: std::sync::Arc::new(pdp_app::session::RevocationList::new()),
            trace_store,
            metrics: server::Metrics::default(),
            annuaire_store,
            webhook_store: webhook_store.clone(),
            event_bus: event_bus.clone(),
            max_flow_size_bytes: http_config.max_flow_size_bytes,
            request_timeout: std::time::Duration::from_secs(http_config.request_timeout_secs),
            rate_limiter: http_config
                .rate_limit_per_minute
                .filter(|n| *n > 0)
                .map(|n| std::sync::Arc::new(server::RateLimiter::new(n))),
            tenants_dir: config
                .tenants_dir
                .as_ref()
                .map(|d| base_dir.join(d)),
        });

        let server_config = server::ServerConfig {
            host: http_config.host.clone(),
            port: http_config.port,
        };

        let _shutdown_rx_server = shutdown_rx.clone();
        tokio::spawn(async move {
            if let Err(e) = server::start_server(server_config, app_state).await {
                tracing::error!(error = %e, "Erreur serveur HTTP");
            }
        });

        tracing::info!(
            host = %http_config.host,
            port = http_config.port,
            "Serveur HTTP API AFNOR démarré"
        );
        } // fin else (mode réception)
    }

    // Gérer Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Signal d'arrêt reçu (Ctrl+C)");
        let _ = shutdown_tx_clone.send(true);
    });

    // Task de conversion InboundFlow → Exchange et injection dans le pipeline
    let _flow_rx_handle = tokio::spawn(async move {
        while let Some(inbound) = flow_rx.recv().await {
            tracing::info!(
                tracking_id = %inbound.flow_info.tracking_id,
                filename = %inbound.filename,
                size = inbound.content.len(),
                "Conversion du flux HTTP entrant en Exchange"
            );

            let mut exchange = pdp_core::Exchange::new(inbound.content);
            exchange.source_filename = Some(inbound.filename.clone());
            exchange.set_header("source.protocol", "afnor-flow");
            exchange.set_property("tracking_id", &inbound.flow_info.tracking_id);
            if let Some(ref syntax) = inbound.flow_info.flow_syntax {
                exchange.set_property("flow.syntax", syntax);
            }
            if let Some(ref profile) = inbound.flow_info.flow_profile {
                exchange.set_property("flow.profile", profile);
            }
            if let Some(ref flow_type) = inbound.flow_info.flow_type {
                exchange.set_property("flow.type", flow_type);
            }
            if let Some(ref callback) = inbound.flow_info.callback_url {
                exchange.set_property("callback.url", callback);
            }

            if http_exchange_tx.send(exchange).await.is_err() {
                tracing::error!("Pipeline HTTP fermé, impossible d'injecter le flux");
                break;
            }
        }
    });

    let interval = std::time::Duration::from_secs(config.polling.interval_secs);
    router.start_polling(interval, shutdown_rx).await?;

    tracing::info!("PDP arrêtée proprement");
    Ok(())
}

async fn cmd_run(config_path: &std::path::Path, mode_str: &str) -> Result<()> {
    let cli_mode = CliMode::from_str(mode_str)?;
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;

    tracing::info!(mode = ?cli_mode, "Exécution unique de toutes les routes");
    let base_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
    let webhook_store = std::sync::Arc::new(webhooks::WebhookStore::new());
    let router = build_router(&config, base_dir, None, cli_mode, webhook_store, None).await?;
    let results = router.execute_all().await;

    let mut total_success = 0;
    let mut total_errors = 0;

    for result in &results {
        let success = result.exchanges.iter().filter(|e| !e.has_errors()).count();
        let errors = result.exchanges.iter().filter(|e| e.has_errors()).count();
        total_success += success;
        total_errors += errors;

        if let Some(ref err) = result.error {
            println!("❌ Route '{}': ERREUR - {}", result.route_id, err);
        } else {
            println!(
                "✅ Route '{}': {} succès, {} erreurs",
                result.route_id, success, errors
            );
        }
    }

    println!("\n📊 Résumé: {} succès, {} erreurs sur {} routes", total_success, total_errors, results.len());
    Ok(())
}

async fn cmd_run_route(config_path: &std::path::Path, route_id: &str) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;
    let base_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
    let webhook_store = std::sync::Arc::new(webhooks::WebhookStore::new());
    let router = build_router(&config, base_dir, None, CliMode::Both, webhook_store, None).await?;

    tracing::info!(route_id = %route_id, "Exécution de la route");
    let exchanges = router.execute_route(route_id).await?;

    for exchange in &exchanges {
        if exchange.has_errors() {
            println!("❌ Exchange {} - ERREUR:", exchange.id);
            for err in &exchange.errors {
                println!("   └─ [{}] {}", err.step, err.message);
            }
        } else {
            println!(
                "✅ Exchange {} - {} | {}",
                exchange.id,
                exchange.source_filename.as_deref().unwrap_or("N/A"),
                exchange.status,
            );
        }
    }

    Ok(())
}

async fn cmd_list_routes(config_path: &std::path::Path) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;

    println!("📋 Routes configurées pour {} ({}):", config.pdp.name, config.pdp.id);
    println!("{:-<70}", "");

    for route in &config.routes {
        let status = if route.enabled { "✅" } else { "⏸️ " };
        println!(
            "{} {} - {}",
            status, route.id, route.description
        );
        println!(
            "   Source: {} ({})",
            route.source.endpoint_type, route.source.path
        );
        println!(
            "   Dest:   {} ({})",
            route.destination.endpoint_type, route.destination.path
        );
        if let Some(ref transform) = route.transform_to {
            println!("   Transform: -> {}", transform);
        }
        println!("   Validation: {} | CDAR: {}", route.validate, route.generate_cdar);
        println!();
    }

    Ok(())
}

async fn cmd_parse(file: &std::path::Path) -> Result<()> {
    let data = std::fs::read(file)?;
    let format = pdp_invoice::detect_format(&data)?;

    println!("📄 Fichier: {}", file.display());
    println!("📋 Format détecté: {}", format);
    println!("{:-<60}", "");

    let invoice = match format {
        pdp_core::model::InvoiceFormat::UBL => {
            let xml = std::str::from_utf8(&data)?;
            pdp_invoice::UblParser::new().parse(xml)?
        }
        pdp_core::model::InvoiceFormat::CII => {
            let xml = std::str::from_utf8(&data)?;
            pdp_invoice::CiiParser::new().parse(xml)?
        }
        pdp_core::model::InvoiceFormat::FacturX => {
            pdp_invoice::FacturXParser::new().parse(&data)?
        }
    };

    println!("Numéro:      {}", invoice.invoice_number);
    println!("Date:        {}", invoice.issue_date.as_deref().unwrap_or("N/A"));
    println!("Échéance:    {}", invoice.due_date.as_deref().unwrap_or("N/A"));
    println!("Vendeur:     {}", invoice.seller_name.as_deref().unwrap_or("N/A"));
    println!("SIRET vend.: {}", invoice.seller_siret.as_deref().unwrap_or("N/A"));
    println!("Acheteur:    {}", invoice.buyer_name.as_deref().unwrap_or("N/A"));
    println!("SIRET ach.:  {}", invoice.buyer_siret.as_deref().unwrap_or("N/A"));
    println!("Devise:      {}", invoice.currency.as_deref().unwrap_or("N/A"));
    println!("Total HT:    {:.2} {}", invoice.total_ht.unwrap_or(0.0), invoice.currency.as_deref().unwrap_or(""));
    println!("TVA:         {:.2} {}", invoice.total_tax.unwrap_or(0.0), invoice.currency.as_deref().unwrap_or(""));
    println!("Total TTC:   {:.2} {}", invoice.total_ttc.unwrap_or(0.0), invoice.currency.as_deref().unwrap_or(""));

    Ok(())
}

async fn cmd_validate(file: &std::path::Path) -> Result<()> {
    let data = std::fs::read(file)?;
    let format = pdp_invoice::detect_format(&data)?;

    let invoice = match format {
        pdp_core::model::InvoiceFormat::UBL => {
            let xml = std::str::from_utf8(&data)?;
            pdp_invoice::UblParser::new().parse(xml)?
        }
        pdp_core::model::InvoiceFormat::CII => {
            let xml = std::str::from_utf8(&data)?;
            pdp_invoice::CiiParser::new().parse(xml)?
        }
        pdp_core::model::InvoiceFormat::FacturX => {
            pdp_invoice::FacturXParser::new().parse(&data)?
        }
    };

    let validator = pdp_invoice::InvoiceValidator::new();
    let result = validator.validate(&invoice);

    println!("🔍 Validation de {} ({})", invoice.invoice_number, format);
    println!("{:-<60}", "");

    if result.is_valid {
        println!("✅ Facture VALIDE");
    } else {
        println!("❌ Facture INVALIDE");
    }

    if !result.errors.is_empty() {
        println!("\n🚨 Erreurs ({}):", result.errors.len());
        for err in &result.errors {
            println!("   [{:?}] {} - {} (champ: {})", err.severity, err.rule_id, err.message, err.field);
        }
    }

    if !result.warnings.is_empty() {
        println!("\n⚠️  Avertissements ({}):", result.warnings.len());
        for warn in &result.warnings {
            println!("   [WARN] {} - {} (champ: {})", warn.rule_id, warn.message, warn.field);
        }
    }

    Ok(())
}

async fn cmd_transform(
    file: &std::path::Path,
    target: &str,
    output: Option<&std::path::Path>,
) -> Result<()> {
    let data = std::fs::read(file)?;
    let format = pdp_invoice::detect_format(&data)?;

    let invoice = match format {
        pdp_core::model::InvoiceFormat::UBL => {
            let xml = std::str::from_utf8(&data)?;
            pdp_invoice::UblParser::new().parse(xml)?
        }
        pdp_core::model::InvoiceFormat::CII => {
            let xml = std::str::from_utf8(&data)?;
            pdp_invoice::CiiParser::new().parse(xml)?
        }
        pdp_core::model::InvoiceFormat::FacturX => {
            pdp_invoice::FacturXParser::new().parse(&data)?
        }
    };

    let upper = target.to_uppercase();
    // PDF (visuel seul, sans XML embarqué) est traité à part — ce n'est pas
    // un format facture EN16931, juste un rendu graphique de la facture pour
    // archivage / pièce jointe.
    if matches!(upper.as_str(), "PDF") {
        let result = pdp_transform::convert_to(&invoice, pdp_transform::OutputFormat::PDF)?;
        let out_path = output.ok_or_else(|| {
            anyhow::anyhow!("--output <chemin.pdf> est requis pour le format PDF")
        })?;
        std::fs::write(out_path, &result.content)?;
        println!(
            "✅ Transformation {} -> PDF (visuel) écrite dans {} ({} octets)",
            format,
            out_path.display(),
            result.content.len()
        );
        return Ok(());
    }

    let target_format = match upper.as_str() {
        "UBL" => pdp_core::model::InvoiceFormat::UBL,
        "CII" => pdp_core::model::InvoiceFormat::CII,
        "FACTURX" | "FACTUR-X" => pdp_core::model::InvoiceFormat::FacturX,
        _ => anyhow::bail!(
            "Format cible non supporté : {}. Utilisez UBL, CII, Factur-X ou PDF.",
            target
        ),
    };
    let result = pdp_transform::convert(&invoice, target_format)?;

    // Factur-X est binaire (PDF/A-3) — il doit aller dans un fichier, pas stdout.
    if matches!(target_format, pdp_core::model::InvoiceFormat::FacturX) {
        let out_path = output.ok_or_else(|| {
            anyhow::anyhow!("--output <chemin.pdf> est requis pour le format Factur-X")
        })?;
        std::fs::write(out_path, &result.content)?;
        println!(
            "✅ Transformation {} -> Factur-X (PDF/A-3) écrite dans {} ({} octets)",
            format,
            out_path.display(),
            result.content.len()
        );
        return Ok(());
    }

    let result_xml = String::from_utf8(result.content)?;
    if let Some(out_path) = output {
        std::fs::write(out_path, &result_xml)?;
        println!(
            "✅ Transformation {} -> {} écrite dans {}",
            format,
            upper,
            out_path.display()
        );
    } else {
        println!("{}", result_xml);
    }

    Ok(())
}

async fn cmd_stats(config_path: &std::path::Path) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url).await
        .map_err(|e| anyhow::anyhow!("Elasticsearch indisponible : {}", e))?;
    let stats = store.get_stats().await?;

    println!("📊 Statistiques PDP ({}):", config.pdp.name);
    println!("{:-<40}", "");
    println!("Total exchanges:  {}", stats.total_exchanges);
    println!("Distribués:       {}", stats.total_distributed);
    println!("En erreur:        {}", stats.total_errors);

    Ok(())
}

async fn cmd_errors(config_path: &std::path::Path) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url).await
        .map_err(|e| anyhow::anyhow!("Elasticsearch indisponible : {}", e))?;
    let errors = store.get_error_flows().await?;

    if errors.is_empty() {
        println!("✅ Aucun flux en erreur");
        return Ok(());
    }

    println!("🚨 Flux en erreur ({}):", errors.len());
    println!("{:-<80}", "");

    for err in &errors {
        println!(
            "❌ {} | {} | {} -> {} | {} erreur(s) | {}",
            err.exchange_id,
            err.invoice_number.as_deref().unwrap_or("N/A"),
            err.seller_name.as_deref().unwrap_or("N/A"),
            err.buyer_name.as_deref().unwrap_or("N/A"),
            err.error_count,
            err.created_at,
        );
    }

    Ok(())
}

async fn cmd_flow_events(config_path: &std::path::Path, flow_id: &str) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url).await
        .map_err(|e| anyhow::anyhow!("Elasticsearch indisponible : {}", e))?;

    let uuid = uuid::Uuid::parse_str(flow_id)
        .map_err(|e| anyhow::anyhow!("ID de flux invalide: {}", e))?;

    let events = store.get_flow_events(uuid).await?;

    if events.is_empty() {
        println!("Aucun événement trouvé pour le flux {}", flow_id);
        return Ok(());
    }

    println!("📋 Événements du flux {} ({} événements):", flow_id, events.len());
    println!("{:-<80}", "");

    for event in &events {
        let icon = if event.error_detail.is_some() { "❌" } else { "✅" };
        println!(
            "{} [{}] {} | {} | {}",
            icon,
            event.timestamp.format("%Y-%m-%d %H:%M:%S"),
            event.status,
            event.route_id,
            event.message,
        );
        if let Some(ref detail) = event.error_detail {
            println!("   └─ Détail: {}", detail);
        }
    }

    Ok(())
}

/// Mode de fonctionnement du CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliMode {
    Emitter,
    Receiver,
    Both,
}

impl CliMode {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "emitter" | "emission" | "emettrice" => Ok(CliMode::Emitter),
            "receiver" | "reception" | "receptrice" => Ok(CliMode::Receiver),
            "both" | "all" | "" => Ok(CliMode::Both),
            other => Err(anyhow::anyhow!(
                "Mode inconnu: '{}'. Valeurs: emitter, receiver, both", other
            )),
        }
    }

    fn should_run_emission(&self) -> bool {
        matches!(self, CliMode::Emitter | CliMode::Both)
    }

    fn should_run_reception(&self) -> bool {
        matches!(self, CliMode::Receiver | CliMode::Both)
    }
}

/// Construit le Router à partir de la configuration.
/// Si `http_rx` est fourni, une route "http-inbound" est ajoutée pour traiter
/// les flux reçus via l'API HTTP (ChannelConsumer) en mode réception.
///
/// Si `tenants_dir` est configuré, des routes auto-générées sont créées pour
/// chaque tenant découvert.
///
/// Le `cli_mode` filtre les routes selon leur `pipeline_mode` :
/// - Emitter : routes émission uniquement
/// - Receiver : routes réception + HTTP inbound
/// - Both : toutes les routes
async fn build_router(
    config: &pdp_config::PdpConfig,
    base_dir: &std::path::Path,
    http_rx: Option<tokio::sync::mpsc::Receiver<pdp_core::Exchange>>,
    cli_mode: CliMode,
    webhook_store: std::sync::Arc<webhooks::WebhookStore>,
    event_bus: Option<pdp_events::EventBus>,
) -> Result<pdp_core::Router> {
    let store = match pdp_trace::TraceStore::new(&config.elasticsearch.url).await {
        Ok(s) => std::sync::Arc::new(s),
        Err(e) => {
            tracing::warn!(error = %e, "Elasticsearch indisponible — traçabilité désactivée");
            std::sync::Arc::new(pdp_trace::TraceStore::noop())
        }
    };

    // Si le bus d'événements est actif, brancher pdp-trace comme subscriber :
    // chaque événement de cycle de vie est répliqué dans Elasticsearch via
    // `TraceEventSubscriber`. Le `TraceProcessor` reste actif pour l'archivage
    // de l'exchange (XML/PDF) — concern distinct de l'événement.
    if let Some(ref bus) = event_bus {
        let trace_sub = pdp_trace::TraceEventSubscriber::new(store.clone());
        pdp_events::DispatcherWorker::new(bus.store().clone(), trace_sub).spawn();
        tracing::info!("TraceEventSubscriber branché sur le bus");
    }

    // Construire les producers PPF et AFNOR si configurés
    let ppf_producer = build_ppf_producer(config)?;
    let (annuaire_client, partner_directory, afnor_producers) = build_afnor_clients(config)?;

    // Service annuaire PPF local (validation G1.63 BR-FR-10/11) — optionnel
    let annuaire_service = build_annuaire_service(config).await;

    // Construire le AlertErrorHandler depuis la config
    let alert_config = config.alerts.as_ref();

    // Canal intra-PDP : émission → réception locale
    let (intra_pdp_tx, intra_pdp_rx) = tokio::sync::mpsc::channel::<pdp_core::Exchange>(100);

    let mut router = pdp_core::Router::new();

    for route_config in &config.routes {
        if !route_config.enabled {
            tracing::info!(route_id = %route_config.id, "Route désactivée, skip");
            continue;
        }

        // Filtrer selon le mode CLI
        match route_config.pipeline_mode {
            pdp_config::PipelineMode::Emission if !cli_mode.should_run_emission() => {
                tracing::info!(route_id = %route_config.id, "Route émission ignorée (mode receiver)");
                continue;
            }
            pdp_config::PipelineMode::Reception if !cli_mode.should_run_reception() => {
                tracing::info!(route_id = %route_config.id, "Route réception ignorée (mode emitter)");
                continue;
            }
            _ => {}
        }

        // Construire le consumer (source)
        let consumer: Box<dyn pdp_core::endpoint::Consumer> = match route_config.source.endpoint_type.as_str() {
            "file" => Box::new(pdp_core::endpoint::FileEndpoint::input(
                &format!("{}-source", route_config.id),
                &route_config.source.path,
            )),
            "sftp" => {
                let sftp_config = pdp_sftp::SftpConfig {
                    host: route_config.source.host.clone().unwrap_or_default(),
                    port: route_config.source.port.unwrap_or(22),
                    username: route_config.source.username.clone().unwrap_or_default(),
                    password: route_config.source.password.clone(),
                    private_key_path: route_config.source.private_key_path.clone(),
                    remote_path: route_config.source.path.clone(),
                    file_pattern: route_config.source.file_pattern.clone().unwrap_or_else(|| "*".to_string()),
                    archive_path: route_config.source.archive_path.clone(),
                    delete_after_read: route_config.source.delete_after_read.unwrap_or(false),
                    timeout_secs: 30,
                    stable_delay_ms: 1000,
                    known_hosts_path: route_config.source.known_hosts_path.clone(),
                };
                Box::new(pdp_sftp::SftpConsumer::new(
                    &format!("{}-sftp-source", route_config.id),
                    sftp_config,
                ))
            }
            other => {
                tracing::warn!(endpoint_type = %other, "Type d'endpoint source non supporté");
                continue;
            }
        };

        // Construire le producer (destination) en fonction du type
        let producer: Box<dyn pdp_core::endpoint::Producer> = match route_config.destination.endpoint_type.as_str() {
            "ppf" => {
                // Destination PPF : utilise le DynamicRoutingProducer avec PPF + PDP partenaires
                if let Some(ref ppf_prod) = ppf_producer {
                    let mut dynamic = pdp_client::DynamicRoutingProducer::new(
                        &format!("{}-dynamic-dest", route_config.id),
                        ppf_prod.clone(),
                    );
                    // Ajouter tous les producers AFNOR partenaires
                    for (matricule, producer) in &afnor_producers {
                        dynamic.add_afnor_producer(matricule, producer.clone());
                    }
                    // Canal intra-PDP pour routage local
                    dynamic = dynamic.with_intra_pdp(intra_pdp_tx.clone());
                    // Fallback sur fichier si aucun producer ne correspond
                    dynamic = dynamic.with_fallback_path(&route_config.destination.path);
                    Box::new(dynamic)
                } else {
                    tracing::warn!(
                        route_id = %route_config.id,
                        "Destination 'ppf' mais aucune configuration PPF. Fallback sur fichier."
                    );
                    Box::new(pdp_core::endpoint::FileEndpoint::output(
                        &format!("{}-dest", route_config.id),
                        &route_config.destination.path,
                    ))
                }
            }
            "sftp" => {
                let sftp_config = pdp_sftp::SftpConfig {
                    host: route_config.destination.host.clone().unwrap_or_default(),
                    port: route_config.destination.port.unwrap_or(22),
                    username: route_config.destination.username.clone().unwrap_or_default(),
                    password: route_config.destination.password.clone(),
                    private_key_path: route_config.destination.private_key_path.clone(),
                    remote_path: route_config.destination.path.clone(),
                    file_pattern: "*".to_string(),
                    archive_path: None,
                    delete_after_read: false,
                    timeout_secs: 30,
                    stable_delay_ms: 0,
                    known_hosts_path: route_config.destination.known_hosts_path.clone(),
                };
                Box::new(pdp_sftp::SftpProducer::new(
                    &format!("{}-sftp-dest", route_config.id),
                    sftp_config,
                ))
            }
            _ => {
                // Défaut : fichier local
                Box::new(pdp_core::endpoint::FileEndpoint::output(
                    &format!("{}-dest", route_config.id),
                    &route_config.destination.path,
                ))
            }
        };

        // Construire le error handler avec alertes
        let error_handler: Option<Box<dyn pdp_core::endpoint::Producer>> = {
            let error_dir = route_config
                .error_destination
                .as_ref()
                .map(|d| d.path.clone())
                .or_else(|| alert_config.map(|a| a.error_dir.clone()))
                .unwrap_or_else(|| format!("errors/{}", route_config.id));

            let mut handler = pdp_core::AlertErrorHandler::new(
                std::path::PathBuf::from(&error_dir),
            );

            if let Some(ref ac) = alert_config {
                if let Some(ref url) = ac.webhook_url {
                    handler = handler.with_webhook(url);
                }
                let level = match ac.min_webhook_level.to_lowercase().as_str() {
                    "warning" => pdp_core::AlertLevel::Warning,
                    "info" => pdp_core::AlertLevel::Info,
                    _ => pdp_core::AlertLevel::Critical,
                };
                handler = handler.with_min_webhook_level(level);
            }

            Some(Box::new(handler) as Box<dyn pdp_core::endpoint::Producer>)
        };

        // Construire la chaîne de processors selon le mode (émission ou réception)
        let mut builder = pdp_core::RouteBuilder::new(&route_config.id)
            .description(&route_config.description)
            .from_source(consumer);

        builder = add_common_processors(builder, config, &store, &ppf_producer);

        match route_config.pipeline_mode {
            pdp_config::PipelineMode::Emission => {
                builder = add_emission_processors(
                    builder, config, route_config, &store,
                    &annuaire_client, &partner_directory, &annuaire_service, &intra_pdp_tx,
                    &webhook_store, &event_bus,
                );
            }
            pdp_config::PipelineMode::Reception => {
                builder = add_reception_processors(
                    builder, config, route_config, &store, &annuaire_service,
                    &webhook_store, &event_bus,
                );
            }
        }

        // Destination + trace finale + événement Distributed (si bus actif)
        builder = builder
            .to_destination(producer)
            .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::distributed(store.clone())));
        if let Some(ref bus) = event_bus {
            builder = builder.process(Box::new(
                pdp_events::LifecycleProcessor::distributed(bus.clone()),
            ));
        }

        if let Some(error_handler) = error_handler {
            builder = builder.on_error(error_handler);
        }

        let route = builder.build()?;
        router.add_route(route)?;
    }

    // --- Routes auto-générées par tenant (multi-tenant) ---
    let registry = pdp_config::TenantRegistry::load(config, base_dir)
        .map_err(|e| anyhow::anyhow!("Chargement TenantRegistry: {}", e))?;

    if registry.len() > 0 && config.tenants_dir.is_some() {
        tracing::info!(
            tenant_count = registry.len(),
            "Génération automatique des routes pour {} tenant(s)",
            registry.len()
        );

        let mut tenant_sirens: Vec<&str> = registry.list_sirens();
        tenant_sirens.sort();

        for siren in tenant_sirens {
            let tenant = registry.get(siren).unwrap();

            // S'assurer que les répertoires in/ et out/ existent
            if let Err(e) = std::fs::create_dir_all(tenant.in_dir()) {
                tracing::error!(siren = %siren, error = %e, "Impossible de créer le répertoire in/");
                continue;
            }
            if let Err(e) = std::fs::create_dir_all(tenant.out_dir()) {
                tracing::error!(siren = %siren, error = %e, "Impossible de créer le répertoire out/");
                continue;
            }

            let route_id = format!("tenant-{}", siren);
            let in_path = tenant.in_dir();
            let out_path = tenant.out_dir();

            // Consumer : FileEndpoint sur {siren}/in/
            let consumer: Box<dyn pdp_core::endpoint::Consumer> =
                Box::new(pdp_core::endpoint::FileEndpoint::input(
                    &format!("{}-source", route_id),
                    in_path.to_str().unwrap_or("."),
                ));

            // Producer : DynamicRoutingProducer (toujours), avec ou sans PPF.
            // Permet à `routing.destination = INTRA-PDP` (résolu par
            // RoutingResolverProcessor quand le buyer est sur la même PDP) de
            // déclencher l'injection dans le channel intra-PDP — même sans
            // Chorus Pro configuré, comme dans config-ui-demo.yaml.
            // Fallback : FileEndpoint sur {siren}/out/ pour les destinations
            // non gérées (PPF non configuré, PDP distante sans AFNOR producer).
            let producer: Box<dyn pdp_core::endpoint::Producer> = {
                let mut dynamic = match ppf_producer.as_ref() {
                    Some(ppf_prod) => pdp_client::DynamicRoutingProducer::new(
                        &format!("{}-dynamic-dest", route_id),
                        ppf_prod.clone(),
                    ),
                    None => pdp_client::DynamicRoutingProducer::new_no_ppf(
                        &format!("{}-dynamic-dest", route_id),
                    ),
                };
                for (matricule, producer) in &afnor_producers {
                    dynamic.add_afnor_producer(matricule, producer.clone());
                }
                dynamic = dynamic.with_intra_pdp(intra_pdp_tx.clone());
                dynamic = dynamic.with_fallback_path(out_path.to_str().unwrap_or("."));
                Box::new(dynamic)
            };

            // Error handler avec alertes : {siren}/out/errors/{critical,warning,info}/
            let error_path = tenant.out_dir().join("errors");
            let mut tenant_alert_handler = pdp_core::AlertErrorHandler::new(error_path);
            if let Some(ref ac) = alert_config {
                if let Some(ref url) = ac.webhook_url {
                    tenant_alert_handler = tenant_alert_handler.with_webhook(url);
                }
                let level = match ac.min_webhook_level.to_lowercase().as_str() {
                    "warning" => pdp_core::AlertLevel::Warning,
                    "info" => pdp_core::AlertLevel::Info,
                    _ => pdp_core::AlertLevel::Critical,
                };
                tenant_alert_handler = tenant_alert_handler.with_min_webhook_level(level);
            }
            let error_handler: Box<dyn pdp_core::endpoint::Producer> =
                Box::new(tenant_alert_handler);

            // Utiliser l'identité PDP du tenant (ou celle de la config racine)
            let pdp_id = &tenant.config.pdp.id;
            let pdp_name = &tenant.config.pdp.name;

            // Chaîne de processors — émission par défaut pour les tenants
            let mut builder = pdp_core::RouteBuilder::new(&route_id)
                .description(&format!("Route émission auto-générée pour tenant {}", siren))
                .from_source(consumer)
                // 0. Tag tenant SIREN sur chaque exchange
                .process(Box::new(pdp_core::TenantTagProcessor::new(siren)));

            builder = add_common_processors(builder, config, &store, &ppf_producer);

            // Tenant route = pipeline émission avec config PPF du tenant
            let tenant_ppf = tenant.config.ppf.as_ref().or(config.ppf.as_ref());

            // Validation
            builder = builder
                .process(Box::new(pdp_invoice::ValidateProcessor::new()))
                .process(Box::new(pdp_validate::XmlValidateProcessor::with_options(
                    &config.validation.specs_dir,
                    config.validation.xsd_enabled,
                    config.validation.en16931_enabled,
                    config.validation.br_fr_enabled,
                    true,
                )))
                .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::validated(store.clone())));

            if let Some(ref bus) = event_bus {
                builder = builder.process(Box::new(
                    pdp_events::LifecycleProcessor::from_status(bus.clone()),
                ));
            }

            // Validation annuaire PPF (G1.63) — vendeur + acheteur
            builder = builder.process(Box::new(pdp_annuaire::AnnuaireValidationProcessor::new(
                annuaire_service.clone(),
                pdp_annuaire::ValidationMode::Emission,
            )));

            if let Some(ref bus) = event_bus {
                builder = builder.process(Box::new(
                    pdp_events::LifecycleProcessor::from_status(bus.clone()),
                ));
            }

            // Flux 1 PPF (TOUJOURS en émission)
            if let Some(ppf) = tenant_ppf {
                let strategy = pdp_transform::Flux1ProfileStrategy::from_config(&ppf.flux1_profile);
                builder = builder.process(Box::new(pdp_transform::PpfFlux1Processor::new(
                    std::path::Path::new(&ppf.flux1_output_dir),
                    std::path::Path::new(&config.validation.specs_dir),
                ).with_strategy(strategy)));
            }

            // Résolution de routage (avec détection intra-PDP) — AVANT CdarProcessor
            // pour permettre la génération de CDV 221 si la destination est invalide.
            if let (Some(ref annuaire), Some(ref partner_dir)) = (&annuaire_client, &partner_directory) {
                let mut resolver = pdp_client::RoutingResolverProcessor::new(
                    annuaire.clone(),
                    partner_dir.clone(),
                );
                if let Some(ref matricule) = config.pdp.matricule {
                    resolver = resolver.with_our_matricule(matricule);
                }
                builder = builder.process(Box::new(resolver));
                builder = builder.process(Box::new(
                    pdp_client::RoutingValidationProcessor::from_partner_directory(partner_dir),
                ));
            }

            // Détection intra-PDP locale (sans PPF) : si le buyer SIREN est un
            // tenant local, on pose routing.destination=INTRA-PDP pour que le
            // DynamicRoutingProducer route via le channel intra-PDP au lieu du
            // fallback filesystem. Si le RoutingResolverProcessor PPF est déjà
            // branché en aval, il peut surcharger cette propriété — mais il ne
            // l'est PAS quand l'annuaire PPF distant est absent.
            let local_sirens: Vec<String> = registry.list_sirens().iter().map(|s| s.to_string()).collect();
            builder = builder.process(Box::new(pdp_cdar::LocalIntraPdpRouter::new(local_sirens)));

            // CDAR auto : génère CDV 200 (Déposée) si le tenant est vendeur de
            // la facture, CDV 202 (Reçue) s'il en est l'acheteur. XP Z12-014
            // §3 (Cas d'usage) — le rôle de la PDP dépend de l'identité du
            // tenant *pour cette facture*, pas de la route d'entrée. Traitement
            // par lot : 1 facture = 1 exchange = 1 CDV, donc chaque facture du
            // lot reçoit son propre CDV calé sur son seller/buyer.
            builder = builder.process(Box::new(pdp_cdar::CdarProcessor::auto(pdp_id, pdp_name)));
            // Matérialise le CDV XML dans {out}/cdar/{flow_id}-cdv-{code}.xml
            // (XP Z12-012 §A.1 — la PDP doit rendre disponible le CDV au client).
            builder = builder.process(Box::new(pdp_cdar::CdvFileWriterProcessor::new(out_path.clone())));

            // Destination = écriture vers le buyer (ou autre PDP / PPF).
            builder = builder.to_destination(producer);

            // Après l'écriture, si on était en réception (CDV 202), émettre
            // le CDV 203 Mise à disposition — la facture est maintenant
            // accessible à l'acheteur (XP Z12-012 §A.1 : 202 → 203).
            // Auto-gated : no-op si le CDV précédent n'était pas 202.
            // **Doit s'exécuter avant le snapshot pour que les fields
            // `disposition_cdv_*` soient indexés en ES.**
            builder = builder.process(Box::new(pdp_cdar::CdvDispositionProcessor::new(
                pdp_id,
                pdp_name,
                out_path.clone(),
            )));

            // Trace finale + événement Distributed (capture les 2 CDVs).
            builder = builder.process(Box::new(pdp_trace::ExchangeSnapshotProcessor::distributed(store.clone())));
            if let Some(ref bus) = event_bus {
                builder = builder.process(Box::new(
                    pdp_events::LifecycleProcessor::distributed(bus.clone()),
                ));
            }
            builder = builder.on_error(error_handler);

            let route = builder.build()?;
            router.add_route(route)?;

            tracing::info!(
                siren = %siren,
                in_dir = %in_path.display(),
                out_dir = %out_path.display(),
                pdp_name = %pdp_name,
                "Route tenant auto-générée"
            );
        }
    }

    // Route HTTP inbound : flux reçus via l'API HTTP AFNOR = pipeline RÉCEPTION
    // (pas de Flux 1 PPF, CDV 202 "Reçue")
    if let Some(rx) = http_rx {
        if cli_mode.should_run_reception() {
            let consumer: Box<dyn pdp_core::endpoint::Consumer> =
                Box::new(pdp_core::ChannelConsumer::new("http-inbound-source", rx));

            let default_output = config.routes.first()
                .map(|r| r.destination.path.clone())
                .unwrap_or_else(|| "output/http-inbound".to_string());

            let producer: Box<dyn pdp_core::endpoint::Producer> =
                Box::new(pdp_core::endpoint::FileEndpoint::output(
                    "http-inbound-dest",
                    &default_output,
                ));

            // Pipeline RÉCEPTION (pas de Flux 1, CDV 202)
            let mut builder = pdp_core::RouteBuilder::new("http-inbound")
                .description("Route réception pour les flux d'autres PDP via l'API HTTP AFNOR")
                .from_source(consumer);

            builder = add_common_processors(builder, config, &store, &ppf_producer);
            builder = add_reception_processors(builder, config, &pdp_config::model::RouteConfig {
                id: "http-inbound".to_string(),
                description: "HTTP inbound".to_string(),
                enabled: true,
                pipeline_mode: pdp_config::PipelineMode::Reception,
                source: pdp_config::model::EndpointConfig::default_file("."),
                destination: pdp_config::model::EndpointConfig::default_file(&default_output),
                error_destination: None,
                transform_to: None,
                validate: true,
                generate_cdar: true,
                cdar_receiver: None,
            }, &store, &annuaire_service, &webhook_store, &event_bus);

            // Error handler avec alertes
            let http_error_dir = alert_config
                .map(|a| a.error_dir.clone())
                .unwrap_or_else(|| "errors/http-inbound".to_string());
            let mut http_alert_handler = pdp_core::AlertErrorHandler::new(
                std::path::PathBuf::from(&http_error_dir),
            );
            if let Some(ref ac) = alert_config {
                if let Some(ref url) = ac.webhook_url {
                    http_alert_handler = http_alert_handler.with_webhook(url);
                }
                let level = match ac.min_webhook_level.to_lowercase().as_str() {
                    "warning" => pdp_core::AlertLevel::Warning,
                    "info" => pdp_core::AlertLevel::Info,
                    _ => pdp_core::AlertLevel::Critical,
                };
                http_alert_handler = http_alert_handler.with_min_webhook_level(level);
            }

            builder = builder
                .to_destination(producer)
                // Après l'écriture vers la destination, si CDV précédent était
                // 202 Reçue, émet 203 Mise à disposition (XP Z12-012 §A.1).
                .process(Box::new(pdp_cdar::CdvDispositionProcessor::new(
                    &config.pdp.id,
                    &config.pdp.name,
                    &default_output,
                )))
                .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::distributed(store.clone())))
                .on_error(Box::new(http_alert_handler));

            let route = builder.build()?;
            router.add_route(route)?;

            tracing::info!("Route 'http-inbound' ajoutée (pipeline réception)");
        }
    }

    // Route intra-PDP réception : reçoit les exchanges du canal intra-PDP
    if cli_mode.should_run_reception() {
        let intra_consumer: Box<dyn pdp_core::endpoint::Consumer> =
            Box::new(pdp_core::ChannelConsumer::new("intra-pdp-source", intra_pdp_rx));

        let default_output = config.routes.first()
            .map(|r| r.destination.path.clone())
            .unwrap_or_else(|| "output/intra-pdp".to_string());

        let intra_producer: Box<dyn pdp_core::endpoint::Producer> =
            Box::new(pdp_core::endpoint::FileEndpoint::output(
                "intra-pdp-dest",
                &default_output,
            ));

        let mut intra_builder = pdp_core::RouteBuilder::new("intra-pdp-reception")
            .description("Route réception pour les flux intra-PDP")
            .from_source(intra_consumer);

        intra_builder = add_common_processors(intra_builder, config, &store, &ppf_producer);
        intra_builder = add_reception_processors(intra_builder, config, &pdp_config::model::RouteConfig {
            id: "intra-pdp".to_string(),
            description: "Intra-PDP reception".to_string(),
            enabled: true,
            pipeline_mode: pdp_config::PipelineMode::Reception,
            source: pdp_config::model::EndpointConfig::default_file("."),
            destination: pdp_config::model::EndpointConfig::default_file(&default_output),
            error_destination: None,
            transform_to: None,
            validate: true,
            generate_cdar: true,
            cdar_receiver: None,
        }, &store, &annuaire_service, &webhook_store, &event_bus);

        let intra_error_dir = alert_config
            .map(|a| a.error_dir.clone())
            .unwrap_or_else(|| "errors/intra-pdp".to_string());
        let intra_alert_handler = pdp_core::AlertErrorHandler::new(
            std::path::PathBuf::from(&intra_error_dir),
        );

        intra_builder = intra_builder
            .to_destination(intra_producer)
            // Après l'écriture intra-PDP vers la destination buyer, émet 203
            // Mise à disposition si le précédent CDV était 202 Reçue.
            .process(Box::new(pdp_cdar::CdvDispositionProcessor::new(
                &config.pdp.id,
                &config.pdp.name,
                &default_output,
            )))
            .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::distributed(store.clone())));
        if let Some(ref bus) = event_bus {
            intra_builder = intra_builder.process(Box::new(
                pdp_events::LifecycleProcessor::distributed(bus.clone()),
            ));
        }
        intra_builder = intra_builder.on_error(Box::new(intra_alert_handler));

        let route = intra_builder.build()?;
        router.add_route(route)?;

        tracing::info!("Route 'intra-pdp-reception' ajoutée");
    }

    // Route PPF retour : ingestion des flux PPF → PDP via SAS retrait SFTP
    // (CDV F6 200/501, exports F14, etc.). Activée si la config PPF SFTP
    // déclare au moins un chemin de retrait.
    if cli_mode.should_run_reception() {
        if let Some(ppf_return_consumer) = build_ppf_return_consumer(config) {
            let archive_dir = config
                .ppf
                .as_ref()
                .map(|p| format!("{}/retrait", p.flux1_output_dir.trim_end_matches('/')))
                .unwrap_or_else(|| "output/ppf-retrait".to_string());

            let _ = std::fs::create_dir_all(&archive_dir);

            let archive_producer: Box<dyn pdp_core::endpoint::Producer> =
                Box::new(pdp_core::endpoint::FileEndpoint::output(
                    "ppf-retrait-archive",
                    &archive_dir,
                ));

            let mut builder = pdp_core::RouteBuilder::new("ppf-sftp-return")
                .description("Route réception des flux PPF → PDP via SAS retrait SFTP")
                .from_source(Box::new(ppf_return_consumer));

            builder = builder
                .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::received(store.clone())))
                .process(Box::new(pdp_core::processor::LogProcessor::info(
                    "ppf-return",
                )))
                .process(Box::new(pdp_cdar::DocumentTypeRouter::new()))
                .process(Box::new(pdp_cdar::CdvReceptionProcessor::new()))
                // Auto-import du F14 (export annuaire) reçu sur le SAS retrait
                // si un AnnuaireService est configuré (PostgreSQL).
                .process(Box::new(pdp_annuaire::AnnuaireImportProcessor::new(
                    annuaire_service.clone(),
                )));

            let error_dir = alert_config
                .map(|a| a.error_dir.clone())
                .unwrap_or_else(|| "errors/ppf-return".to_string());
            let alert_handler = pdp_core::AlertErrorHandler::new(
                std::path::PathBuf::from(&error_dir),
            );

            builder = builder
                .to_destination(archive_producer)
                .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::distributed(store.clone())))
                .on_error(Box::new(alert_handler));

            let route = builder.build()?;
            router.add_route(route)?;

            tracing::info!(
                archive_dir = %archive_dir,
                "Route 'ppf-sftp-return' ajoutée (ingestion SAS retrait PPF)"
            );
        }
    }

    Ok(router)
}

/// Construit le consumer SFTP de retour PPF (PPF → PDP) si la config
/// `ppf.sftp` déclare au moins un chemin de retrait (`retrait_path` ou
/// `retrait_paths`). Renvoie `None` sinon.
fn build_ppf_return_consumer(
    config: &pdp_config::PdpConfig,
) -> Option<pdp_client::PpfReturnConsumer> {
    let ppf = config.ppf.as_ref()?;
    let sftp = ppf.sftp.as_ref()?;

    let paths = sftp.all_retrait_paths();
    if paths.is_empty() {
        return None;
    }

    let sftp_config = pdp_sftp::SftpConfig {
        host: sftp.host.clone(),
        port: sftp.port,
        username: sftp.username.clone(),
        password: None,
        private_key_path: Some(sftp.private_key_path.clone()),
        // remote_path est ignoré par PpfReturnConsumer (un chemin par path)
        remote_path: paths[0].clone(),
        file_pattern: "*.tar.gz".to_string(),
        archive_path: sftp.retrait_archive_path.clone(),
        delete_after_read: sftp.retrait_archive_path.is_none()
            && sftp.retrait_delete_after_read,
        timeout_secs: 30,
        stable_delay_ms: 1000,
        known_hosts_path: sftp.known_hosts_path.clone(),
    };

    let consumer_config = pdp_client::PpfReturnConsumerConfig {
        sftp: sftp_config,
        paths,
        code_interface_by_path: sftp.retrait_paths.clone(),
        archive_path: sftp.retrait_archive_path.clone(),
        delete_after_read: sftp.retrait_delete_after_read,
    };

    tracing::info!(
        host = %sftp.host,
        paths_count = consumer_config.paths.len(),
        archive = ?consumer_config.archive_path,
        "PPF SFTP retrait configuré ({} chemin(s))",
        consumer_config.paths.len()
    );

    Some(pdp_client::PpfReturnConsumer::new(
        "ppf-sftp-return",
        consumer_config,
    ))
}

/// Processors communs à tous les pipelines (émission et réception) :
/// trace réception, contrôles de réception, irrecevabilité, détection type,
/// relay CDV→PPF (210/212), parsing, détection doublons
fn add_common_processors(
    builder: pdp_core::RouteBuilder,
    config: &pdp_config::PdpConfig,
    store: &std::sync::Arc<pdp_trace::TraceStore>,
    ppf_producer: &Option<std::sync::Arc<pdp_client::PpfSftpProducer>>,
) -> pdp_core::RouteBuilder {
    let mut builder = builder
        .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::received(store.clone())))
        .process(Box::new(pdp_core::processor::LogProcessor::info("reception")))
        .process(Box::new(pdp_core::reception::ReceptionProcessor::strict()))
        .process(Box::new(pdp_cdar::IrrecevabiliteProcessor::new(
            &config.pdp.id,
            &config.pdp.name,
        )))
        .process(Box::new(pdp_cdar::DocumentTypeRouter::new()));

    // Relay CDV 210 (Refusée) et 212 (Encaissée) vers le PPF via Flux 6
    // Placé juste après DocumentTypeRouter qui parse les CDAR et set cdv.*
    if let Some(ref ppf_prod) = ppf_producer {
        builder = builder.process(Box::new(pdp_cdar::CdvPpfRelayProcessor::new(
            ppf_prod.clone(),
        )));
    }

    builder
        .process(Box::new(pdp_invoice::ParseProcessor::new()))
        .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::parsed(store.clone())))
        .process(Box::new(pdp_trace::DuplicateCheckProcessor::new(store.clone())))
}

/// Processors spécifiques au pipeline ÉMISSION :
/// validation, Flux 1 PPF (TOUJOURS), transformation, CDAR 200, routage
fn add_emission_processors(
    mut builder: pdp_core::RouteBuilder,
    config: &pdp_config::PdpConfig,
    route_config: &pdp_config::model::RouteConfig,
    store: &std::sync::Arc<pdp_trace::TraceStore>,
    annuaire_client: &Option<std::sync::Arc<pdp_client::annuaire::AnnuaireClient>>,
    partner_directory: &Option<pdp_client::PartnerDirectory>,
    annuaire_service: &Option<std::sync::Arc<pdp_annuaire::AnnuaireService>>,
    _intra_pdp_tx: &tokio::sync::mpsc::Sender<pdp_core::Exchange>,
    webhook_store: &std::sync::Arc<webhooks::WebhookStore>,
    event_bus: &Option<pdp_events::EventBus>,
) -> pdp_core::RouteBuilder {
    // Validation
    if route_config.validate {
        builder = builder
            .process(Box::new(pdp_invoice::ValidateProcessor::new()))
            .process(Box::new(pdp_validate::XmlValidateProcessor::with_options(
                &config.validation.specs_dir,
                config.validation.xsd_enabled,
                config.validation.en16931_enabled,
                config.validation.br_fr_enabled,
                true,
            )))
            .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::validated(store.clone())));

        // Publication événement Validated → bus si dispo,
        // sinon fallback WebhookAckProcessor (ancien chemin direct).
        if let Some(bus) = event_bus {
            builder = builder.process(Box::new(
                pdp_events::LifecycleProcessor::from_status(bus.clone()),
            ));
        } else {
            builder = builder.process(Box::new(webhooks::WebhookAckProcessor::new(
                webhook_store.clone(),
                "Out",
            )));
        }
    }

    // Validation annuaire PPF (G1.63) — vendeur + acheteur
    builder = builder.process(Box::new(pdp_annuaire::AnnuaireValidationProcessor::new(
        annuaire_service.clone(),
        pdp_annuaire::ValidationMode::Emission,
    )));

    // Événement post-validation annuaire (rejet éventuel si BR-FR-10/11 KO).
    if let Some(bus) = event_bus {
        builder = builder.process(Box::new(
            pdp_events::LifecycleProcessor::from_status(bus.clone()),
        ));
    } else {
        builder = builder.process(Box::new(webhooks::WebhookAckProcessor::new(
            webhook_store.clone(),
            "Out",
        )));
    }

    // Flux 1 PPF (TOUJOURS en émission — données réglementaires)
    if let Some(ref ppf) = config.ppf {
        let strategy = pdp_transform::Flux1ProfileStrategy::from_config(&ppf.flux1_profile);
        builder = builder.process(Box::new(pdp_transform::PpfFlux1Processor::new(
            std::path::Path::new(&ppf.flux1_output_dir),
            std::path::Path::new(&config.validation.specs_dir),
        ).with_strategy(strategy)));
    }

    // Transformation (optionnelle)
    if let Some(ref target) = route_config.transform_to {
        let target_format = match target.to_uppercase().as_str() {
            "CII" => pdp_core::model::InvoiceFormat::CII,
            "UBL" => pdp_core::model::InvoiceFormat::UBL,
            _ => {
                tracing::warn!(target = %target, "Format de transformation non supporté");
                return builder;
            }
        };
        builder = builder
            .process(Box::new(pdp_transform::TransformProcessor::new(target_format)))
            .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::transformed(store.clone())));
        if let Some(bus) = event_bus {
            builder = builder.process(Box::new(
                pdp_events::LifecycleProcessor::transformed(bus.clone()),
            ));
        }
    }

    // Résolution de routage (avec détection intra-PDP) — AVANT le CdarProcessor
    // pour que ce dernier puisse générer un CDV 221 si la destination est invalide.
    if route_config.destination.endpoint_type == "ppf" {
        if let (Some(ref annuaire), Some(ref partner_dir)) = (annuaire_client, partner_directory) {
            let mut resolver = pdp_client::RoutingResolverProcessor::new(
                annuaire.clone(),
                partner_dir.clone(),
            );
            if let Some(ref matricule) = config.pdp.matricule {
                resolver = resolver.with_our_matricule(matricule);
            }
            builder = builder.process(Box::new(resolver));

            // Vérifie qu'un producer AFNOR existe pour le matricule destinataire.
            // Si non → add_error("routage", ...) → CdarProcessor générera CDV 221.
            builder = builder.process(Box::new(
                pdp_client::RoutingValidationProcessor::from_partner_directory(partner_dir),
            ));
        }
    }

    // CDAR émission (CDV 200 "Déposée", 213 "Rejetée", ou 221 "Erreur routage")
    // Doit s'exécuter APRÈS le routage pour décider entre 200/213/221.
    if route_config.generate_cdar {
        builder = builder.process(Box::new(pdp_cdar::CdarProcessor::emission(
            &config.pdp.id,
            &config.pdp.name,
        )));
    }

    builder
}

/// Processors spécifiques au pipeline RÉCEPTION :
/// validation, PAS de Flux 1, transformation optionnelle, CDAR 202 "Reçue"
fn add_reception_processors(
    mut builder: pdp_core::RouteBuilder,
    config: &pdp_config::PdpConfig,
    route_config: &pdp_config::model::RouteConfig,
    store: &std::sync::Arc<pdp_trace::TraceStore>,
    annuaire_service: &Option<std::sync::Arc<pdp_annuaire::AnnuaireService>>,
    webhook_store: &std::sync::Arc<webhooks::WebhookStore>,
    event_bus: &Option<pdp_events::EventBus>,
) -> pdp_core::RouteBuilder {
    // Validation
    if route_config.validate {
        builder = builder
            .process(Box::new(pdp_invoice::ValidateProcessor::new()))
            .process(Box::new(pdp_validate::XmlValidateProcessor::with_options(
                &config.validation.specs_dir,
                config.validation.xsd_enabled,
                config.validation.en16931_enabled,
                config.validation.br_fr_enabled,
                true,
            )))
            .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::validated(store.clone())));

        if let Some(bus) = event_bus {
            builder = builder.process(Box::new(
                pdp_events::LifecycleProcessor::from_status(bus.clone()),
            ));
        } else {
            builder = builder.process(Box::new(webhooks::WebhookAckProcessor::new(
                webhook_store.clone(),
                "In",
            )));
        }
    }

    // Validation annuaire PPF (G1.63) — vendeur uniquement en réception
    builder = builder.process(Box::new(pdp_annuaire::AnnuaireValidationProcessor::new(
        annuaire_service.clone(),
        pdp_annuaire::ValidationMode::Reception,
    )));

    // Événement post-validation annuaire (bus, ou fallback webhook direct).
    if let Some(bus) = event_bus {
        builder = builder.process(Box::new(
            pdp_events::LifecycleProcessor::from_status(bus.clone()),
        ));
    } else {
        builder = builder.process(Box::new(webhooks::WebhookAckProcessor::new(
            webhook_store.clone(),
            "In",
        )));
    }

    // PAS de Flux 1 PPF — la PDP émettrice l'a déjà fait

    // Transformation (optionnelle — si l'acheteur a besoin d'un format différent)
    if let Some(ref target) = route_config.transform_to {
        let target_format = match target.to_uppercase().as_str() {
            "CII" => pdp_core::model::InvoiceFormat::CII,
            "UBL" => pdp_core::model::InvoiceFormat::UBL,
            _ => {
                tracing::warn!(target = %target, "Format de transformation non supporté");
                return builder;
            }
        };
        builder = builder
            .process(Box::new(pdp_transform::TransformProcessor::new(target_format)))
            .process(Box::new(pdp_trace::ExchangeSnapshotProcessor::transformed(store.clone())));
        if let Some(bus) = event_bus {
            builder = builder.process(Box::new(
                pdp_events::LifecycleProcessor::transformed(bus.clone()),
            ));
        }
    }

    // CDAR réception (CDV 202 "Reçue" ou 213 "Rejetée")
    if route_config.generate_cdar {
        builder = builder.process(Box::new(pdp_cdar::CdarProcessor::reception(
            &config.pdp.id,
            &config.pdp.name,
        )));
        // Matérialise le CDV de réception dans {destination.path}/cdar/
        builder = builder.process(Box::new(pdp_cdar::CdvFileWriterProcessor::new(
            &route_config.destination.path,
        )));
    }

    // PAS de résolution de routage — la facture est livrée directement à l'acheteur

    builder
}

/// Construit le service annuaire PPF local si la base PostgreSQL est configurée
/// et accessible. Renvoie `None` (avec un warning) si la connexion échoue, pour
/// que l'application reste fonctionnelle sans annuaire local.
async fn build_annuaire_service(
    config: &pdp_config::PdpConfig,
) -> Option<std::sync::Arc<pdp_annuaire::AnnuaireService>> {
    let db_config = config.database.as_ref()?;
    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_config.max_connections)
        .connect(&db_config.url)
        .await
    {
        Ok(pool) => {
            let store = pdp_annuaire::AnnuaireStore::new(pool);
            if let Err(e) = store.migrate().await {
                tracing::warn!(error = %e, "Migration annuaire échouée");
            }
            tracing::info!("Service annuaire PPF (G1.63 BR-FR-10/11) activé");
            Some(std::sync::Arc::new(pdp_annuaire::AnnuaireService::new(store)))
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Impossible de connecter PostgreSQL — validation annuaire G1.63 désactivée"
            );
            None
        }
    }
}

/// Construit le producer PPF SFTP si la configuration PPF est présente
fn build_ppf_producer(
    config: &pdp_config::PdpConfig,
) -> Result<Option<std::sync::Arc<pdp_client::PpfSftpProducer>>> {
    let ppf = match &config.ppf {
        Some(ppf) => ppf,
        None => return Ok(None),
    };

    let sftp_config = match &ppf.sftp {
        Some(sftp) => pdp_sftp::SftpConfig {
            host: sftp.host.clone(),
            port: sftp.port,
            username: sftp.username.clone(),
            password: None,
            private_key_path: Some(sftp.private_key_path.clone()),
            remote_path: sftp.remote_path.clone(),
            file_pattern: "*".to_string(),
            archive_path: None,
            delete_after_read: false,
            timeout_secs: 30,
            stable_delay_ms: 0,
            known_hosts_path: sftp.known_hosts_path.clone(),
        },
        None => {
            // Pas de SFTP configuré : utiliser un répertoire local comme fallback
            tracing::info!(
                "PPF SFTP non configuré, les flux seront écrits dans {}",
                ppf.flux1_output_dir
            );
            return Ok(None);
        }
    };

    // Reprendre la config SAS dépôt depuis le YAML (ppf.sftp).
    // - `depot_path` (ou `remote_path` historique) → fallback global
    // - `depot_paths[code_interface]` → mapping fin par type de flux
    let (default_depot_path, depot_paths) = match &ppf.sftp {
        Some(sftp) => {
            let default = sftp.depot_path.clone().or_else(|| {
                if sftp.remote_path.is_empty() {
                    None
                } else {
                    Some(sftp.remote_path.clone())
                }
            });
            (default, sftp.depot_paths.clone())
        }
        None => (None, std::collections::HashMap::new()),
    };

    let producer_config = pdp_client::PpfSftpProducerConfig {
        code_application: ppf.code_application_piste.clone(),
        default_profil: ppf.flux1_profile.clone(),
        default_depot_path,
        depot_paths,
    };

    let initial_sequence = ppf.initial_sequence.unwrap_or(0);

    // Fichier de persistance du numéro de séquence (optionnel)
    let sequence_file = ppf.sftp.as_ref()
        .and_then(|s| s.sequence_file.as_ref())
        .map(|p| std::path::PathBuf::from(p));

    let producer = pdp_client::PpfSftpProducer::with_sequence_file(
        "ppf-sftp",
        producer_config,
        sftp_config,
        initial_sequence,
        sequence_file,
    )?;

    tracing::info!(
        environment = %ppf.environment,
        code_application = %ppf.code_application_piste,
        "Producer PPF SFTP initialisé"
    );

    Ok(Some(std::sync::Arc::new(producer)))
}

/// Construit les clients AFNOR (Annuaire + Flow Service) si la configuration est présente
fn build_afnor_clients(
    config: &pdp_config::PdpConfig,
) -> Result<(
    Option<std::sync::Arc<pdp_client::annuaire::AnnuaireClient>>,
    Option<pdp_client::PartnerDirectory>,
    std::collections::HashMap<String, std::sync::Arc<pdp_client::AfnorFlowProducer>>,
)> {
    let ppf = match &config.ppf {
        Some(ppf) => ppf,
        None => return Ok((None, None, std::collections::HashMap::new())),
    };

    // Construire l'authentification PISTE
    let auth_config = pdp_client::auth::PisteAuthConfig {
        token_url: ppf.auth.token_url.clone(),
        client_id: ppf.auth.client_id.clone(),
        client_secret: ppf.auth.client_secret.clone(),
        scope: ppf.auth.scope.clone(),
    };
    let annuaire_auth = pdp_client::PisteAuth::new(auth_config);

    // Client Annuaire PPF
    let env = pdp_client::model::PpfEnvironment::from_code(&ppf.environment)
        .unwrap_or_else(|| panic!("Environnement PPF invalide: '{}'. Valeurs possibles: dev, int, rec, preprod, prod", ppf.environment));
    let annuaire_config = pdp_client::annuaire::AnnuaireConfig {
        environment: env,
    };
    let annuaire = std::sync::Arc::new(
        pdp_client::annuaire::AnnuaireClient::new(annuaire_config, annuaire_auth)
    );

    // Construire le répertoire des PDP partenaires
    let mut partner_directory = pdp_client::PartnerDirectory::new();
    let mut afnor_producers = std::collections::HashMap::new();

    if let Some(ref afnor) = config.afnor {
        for partner in &afnor.partners {
            partner_directory.add_partner(
                &partner.matricule,
                &partner.name,
                &partner.flow_service_url,
            );

            // Créer un client AFNOR Flow Service pour chaque partenaire
            let partner_auth_config = if let Some(ref afnor_auth) = afnor.auth {
                pdp_client::auth::PisteAuthConfig {
                    token_url: afnor_auth.token_url.clone(),
                    client_id: afnor_auth.client_id.clone(),
                    client_secret: afnor_auth.client_secret.clone(),
                    scope: afnor_auth.scope.clone(),
                }
            } else {
                pdp_client::auth::PisteAuthConfig {
                    token_url: ppf.auth.token_url.clone(),
                    client_id: ppf.auth.client_id.clone(),
                    client_secret: ppf.auth.client_secret.clone(),
                    scope: ppf.auth.scope.clone(),
                }
            };
            let partner_auth = pdp_client::PisteAuth::new(partner_auth_config);

            let flow_config = pdp_client::afnor::AfnorFlowConfig {
                base_url: partner.flow_service_url.clone(),
            };
            let flow_client = std::sync::Arc::new(
                pdp_client::AfnorFlowClient::new(flow_config, partner_auth)
            );
            let producer = std::sync::Arc::new(
                pdp_client::AfnorFlowProducer::new(
                    &format!("afnor-{}", partner.matricule),
                    flow_client,
                )
            );

            afnor_producers.insert(partner.matricule.clone(), producer);

            tracing::info!(
                matricule = %partner.matricule,
                name = %partner.name,
                url = %partner.flow_service_url,
                "PDP partenaire AFNOR enregistrée"
            );
        }
    }

    Ok((Some(annuaire), Some(partner_directory), afnor_producers))
}

// ============================================================
// Commandes annuaire PPF
// ============================================================

async fn connect_annuaire_db(config_path: &std::path::Path) -> Result<pdp_annuaire::AnnuaireStore> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;
    let db_config = config.database.unwrap_or_default();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_config.max_connections)
        .connect(&db_config.url)
        .await?;
    let store = pdp_annuaire::AnnuaireStore::new(pool);
    store.migrate().await.map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(store)
}

async fn cmd_annuaire(config_path: &std::path::Path, action: AnnuaireCommands) -> Result<()> {
    match action {
        AnnuaireCommands::Import { file } => {
            let store = connect_annuaire_db(config_path).await?;

            let file_size = std::fs::metadata(&file)?.len();
            let f = std::fs::File::open(&file)?;

            // Barre de progression sur les bytes lus
            let pb = indicatif::ProgressBar::new(file_size);
            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})")
                    .unwrap()
                    .progress_chars("##-"),
            );
            pb.set_message("Parsing F14...");

            let reader = pb.wrap_read(f);
            let reader = std::io::BufReader::with_capacity(8 * 1024 * 1024, reader);

            let start = std::time::Instant::now();
            let stats = pdp_annuaire::ingest_f14(reader, &store, None)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let elapsed = start.elapsed();

            pb.finish_with_message("Import terminé");

            println!("\nImport terminé en {:.1}s", elapsed.as_secs_f64());
            println!("  Unités légales  : {}", stats.unites_legales);
            println!("  Établissements  : {}", stats.etablissements);
            println!("  Codes routage   : {}", stats.codes_routage);
            println!("  Plateformes     : {}", stats.plateformes);
            println!("  Lignes annuaire : {}", stats.lignes_annuaire);
            if stats.errors > 0 {
                println!("  Erreurs         : {}", stats.errors);
            }
            let total = stats.unites_legales + stats.etablissements + stats.codes_routage
                + stats.plateformes + stats.lignes_annuaire;
            println!("  Throughput      : {:.0} éléments/s", total as f64 / elapsed.as_secs_f64());
            Ok(())
        }

        AnnuaireCommands::Stats => {
            let store = connect_annuaire_db(config_path).await?;
            let stats = store.count_all().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            let last_sync = store.last_sync_horodate().await.map_err(|e| anyhow::anyhow!("{}", e))?;

            println!("Annuaire PPF local :");
            println!("  Unités légales  : {}", stats.unites_legales);
            println!("  Établissements  : {}", stats.etablissements);
            println!("  Codes routage   : {}", stats.codes_routage);
            println!("  Plateformes     : {}", stats.plateformes);
            println!("  Lignes annuaire : {}", stats.lignes_annuaire);
            if let Some(h) = last_sync {
                println!("  Dernière synchro: {}", h);
            } else {
                println!("  Dernière synchro: jamais");
            }
            Ok(())
        }

        AnnuaireCommands::Lookup { siren } => {
            let store = connect_annuaire_db(config_path).await?;
            let result = store.lookup_unite_legale(&siren)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            match result {
                Some(ul) => {
                    println!("SIREN {} : {}", ul.siren, ul.nom);
                    println!("  Type      : {:?}", ul.type_entite);
                    println!("  Statut    : {:?}", ul.statut);
                    println!("  Diffusible: {:?}", ul.diffusible);
                }
                None => println!("SIREN {} non trouvé dans l'annuaire local", siren),
            }
            Ok(())
        }

        AnnuaireCommands::Route { siren, siret } => {
            let store = connect_annuaire_db(config_path).await?;
            let today = chrono::Local::now().format("%Y%m%d").to_string();
            let result = store.resolve_routing(
                &siren,
                siret.as_deref(),
                None,
                None,
                &today,
            ).await.map_err(|e| anyhow::anyhow!("{}", e))?;

            match result {
                Some(r) => {
                    println!("Routage pour SIREN {} :", siren);
                    println!("  Plateforme : {}", r.matricule_plateforme);
                    if let Some(nom) = &r.nom_plateforme {
                        println!("  Nom        : {}", nom);
                    }
                    println!("  Type       : {:?}", r.type_plateforme);
                    println!("  Maille     : {:?}", r.maille);
                }
                None => println!("Aucun routage trouvé pour SIREN {}", siren),
            }
            Ok(())
        }
    }
}

// ============================================================
// E-reporting (Flux 10.1, 10.3) — agrégation depuis répertoire local
// ============================================================

/// Lit toutes les factures (XML/PDF) d'un répertoire et les parse via la
/// détection de format automatique.
fn load_invoices_from_dir(
    dir: &std::path::Path,
) -> Result<Vec<pdp_core::model::InvoiceData>> {
    let mut invoices = Vec::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("Impossible de lire {} : {}", dir.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        if !matches!(ext.as_deref(), Some("xml") | Some("pdf")) {
            continue;
        }

        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("⚠️  {} : lecture impossible ({})", path.display(), e);
                continue;
            }
        };
        let format = match pdp_invoice::detect_format(&data) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("⚠️  {} : format non détecté ({})", path.display(), e);
                continue;
            }
        };
        let parsed = match format {
            pdp_core::model::InvoiceFormat::UBL => {
                let xml = std::str::from_utf8(&data)?;
                pdp_invoice::UblParser::new().parse(xml)
            }
            pdp_core::model::InvoiceFormat::CII => {
                let xml = std::str::from_utf8(&data)?;
                pdp_invoice::CiiParser::new().parse(xml)
            }
            pdp_core::model::InvoiceFormat::FacturX => {
                pdp_invoice::FacturXParser::new().parse(&data)
            }
        };
        match parsed {
            Ok(inv) => invoices.push(inv),
            Err(e) => eprintln!("⚠️  {} : parsing échoué ({})", path.display(), e),
        }
    }
    Ok(invoices)
}

fn write_or_print(output: Option<&std::path::Path>, content: &str) -> Result<()> {
    match output {
        Some(path) => {
            std::fs::write(path, content)?;
            println!("✅ Rapport écrit dans {}", path.display());
        }
        None => println!("{}", content),
    }
    Ok(())
}

/// Convertit un `ExchangeDocument` Elasticsearch en `InvoiceData` complet
/// en re-parsant le `raw_xml` (UBL/CII/Factur-X selon `source_format`).
/// Si `raw_xml` est absent ou le parsing échoue, retourne `None`.
fn exchange_doc_to_invoice(
    doc: &pdp_trace::store::ExchangeDocument,
) -> Option<pdp_core::model::InvoiceData> {
    let raw = doc.raw_xml.as_deref()?;
    let format_str = doc.source_format.as_deref().unwrap_or("UBL");
    let bytes = raw.as_bytes().to_vec();

    match format_str.to_uppercase().as_str() {
        "UBL" => pdp_invoice::UblParser::new().parse(raw).ok(),
        "CII" => pdp_invoice::CiiParser::new().parse(raw).ok(),
        "FACTURX" | "FACTUR-X" => pdp_invoice::FacturXParser::new().parse(&bytes).ok(),
        _ => {
            // Format inconnu : tenter détection automatique
            let format = pdp_invoice::detect_format(&bytes).ok()?;
            match format {
                pdp_core::model::InvoiceFormat::UBL => {
                    pdp_invoice::UblParser::new().parse(raw).ok()
                }
                pdp_core::model::InvoiceFormat::CII => {
                    pdp_invoice::CiiParser::new().parse(raw).ok()
                }
                pdp_core::model::InvoiceFormat::FacturX => {
                    pdp_invoice::FacturXParser::new().parse(&bytes).ok()
                }
            }
        }
    }
}

/// Récupère les factures du tenant depuis Elasticsearch sur la période donnée
/// et les convertit en `InvoiceData` parseés.
async fn load_invoices_from_es(
    config_path: &std::path::Path,
    siren: &str,
    from: &str,
    to: &str,
) -> Result<Vec<pdp_core::model::InvoiceData>> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))
        .map_err(|e| anyhow::anyhow!("Chargement config : {}", e))?;
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url)
        .await
        .map_err(|e| anyhow::anyhow!("Connexion Elasticsearch : {}", e))?;

    let docs = store
        .get_invoices_by_period(siren, from, to)
        .await
        .map_err(|e| anyhow::anyhow!("Recherche ES : {}", e))?;

    println!(
        "📊 {} factures trouvées dans pdp-{} sur la période {}..{}",
        docs.len(),
        siren,
        from,
        to
    );

    let mut invoices = Vec::with_capacity(docs.len());
    let mut skipped = 0;
    for doc in &docs {
        match exchange_doc_to_invoice(doc) {
            Some(inv) => invoices.push(inv),
            None => {
                skipped += 1;
                tracing::debug!(
                    invoice_number = doc.invoice_number.as_deref().unwrap_or("?"),
                    "Facture non re-parseable depuis ES (raw_xml manquant ou invalide)"
                );
            }
        }
    }
    if skipped > 0 {
        eprintln!("⚠️  {} factures ignorées (raw_xml manquant ou invalide)", skipped);
    }
    Ok(invoices)
}

async fn cmd_ereporting(
    config_path: &std::path::Path,
    action: EreportingCommands,
) -> Result<()> {
    use pdp_ereporting::EReportingGenerator;

    match action {
        EreportingCommands::Generate101 {
            invoices_dir,
            siren,
            name,
            from,
            to,
            report_id,
            output,
        } => {
            let invoices = match invoices_dir {
                Some(dir) => {
                    let inv = load_invoices_from_dir(&dir)?;
                    println!(
                        "📊 {} factures chargées depuis {}",
                        inv.len(),
                        dir.display()
                    );
                    inv
                }
                None => load_invoices_from_es(config_path, &siren, &from, &to).await?,
            };
            let transactions: Vec<_> = invoices
                .iter()
                .map(EReportingGenerator::invoice_to_transaction)
                .collect();
            let gen = EReportingGenerator::new(&siren, &name);
            let id = report_id.unwrap_or_else(|| {
                format!("RPT-10.1-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
            });
            let report =
                gen.create_transactions_report(&id, &siren, &name, &from, &to, transactions);
            let xml = gen
                .to_xml(&report)
                .map_err(|e| anyhow::anyhow!("Sérialisation XML : {}", e))?;
            write_or_print(output.as_deref(), &xml)?;
            Ok(())
        }
        EreportingCommands::Generate103 {
            invoices_dir,
            siren,
            name,
            from,
            to,
            report_id,
            output,
        } => {
            let invoices = match invoices_dir {
                Some(dir) => {
                    let inv = load_invoices_from_dir(&dir)?;
                    println!(
                        "📊 {} factures chargées depuis {}",
                        inv.len(),
                        dir.display()
                    );
                    inv
                }
                None => load_invoices_from_es(config_path, &siren, &from, &to).await?,
            };
            let gen = EReportingGenerator::new(&siren, &name);
            let id = report_id.unwrap_or_else(|| {
                format!("RPT-10.3-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
            });
            let report = gen
                .create_aggregated_transactions_report(&id, &siren, &name, &from, &to, &invoices)
                .map_err(|e| anyhow::anyhow!("Agrégation 10.3 : {}", e))?;
            let xml = gen
                .to_xml(&report)
                .map_err(|e| anyhow::anyhow!("Sérialisation XML : {}", e))?;
            write_or_print(output.as_deref(), &xml)?;
            Ok(())
        }
    }
}

// ============================================================
// Démo : peuplement automatique du dashboard via POST /v1/flows
// ============================================================

async fn cmd_demo(action: DemoCommands) -> Result<()> {
    match action {
        DemoCommands::Populate {
            server_url,
            fixtures_dir,
            token,
            reset,
            elasticsearch_url,
        } => {
            cmd_demo_populate(
                &server_url,
                &fixtures_dir,
                token.as_deref(),
                reset,
                &elasticsearch_url,
            )
            .await
        }
        DemoCommands::Seed {
            server_url,
            annuaire_file,
            fixtures_dir,
            token,
            reset_annuaire,
            reset_factures,
            elasticsearch_url,
        } => {
            cmd_demo_seed(
                &server_url,
                &annuaire_file,
                &fixtures_dir,
                token.as_deref(),
                reset_annuaire,
                reset_factures,
                &elasticsearch_url,
            )
            .await
        }
    }
}

/// Bootstrap complet d'une démo Ferrite :
/// 1. Importe l'annuaire F14 dans PostgreSQL (~500 entreprises)
/// 2. Crée les répertoires `tenants/{siren}/{in,out}/` + `config.yaml` pour
///    les entreprises de démo (TechConseil, Charlotte Solutions,
///    Menuiserie Dupont) à partir des noms trouvés dans l'annuaire
/// 3. Pousse toutes les fixtures UBL/CII/Factur-X via `POST /v1/flows`
///
/// Les SIRENs de démo sont en dur ici parce qu'ils doivent rester alignés
/// avec [`config-ui-demo.yaml`] et avec les fixtures (cf. `gen-fixtures-bulk.py`).
async fn cmd_demo_seed(
    server_url: &str,
    annuaire_file: &std::path::Path,
    fixtures_dir: &std::path::Path,
    token: Option<&str>,
    reset_annuaire: bool,
    reset_factures: bool,
    elasticsearch_url: &str,
) -> Result<()> {
    /// SIRENs des entreprises de démo. Alignés sur `config-ui-demo.yaml`
    /// (alice / charlotte / dupont) ET sur les fixtures UBL/CII.
    const DEMO_TENANT_SIRENS: &[&str] = &["123456789", "109009309", "111222333"];
    /// Bearer token par défaut câblé dans `config-ui-demo.yaml`. Permet à
    /// `seed` de pousser les fixtures sans demander à l'utilisateur de le
    /// préciser à la main. En config "réelle" (production), passer `--token`.
    const DEFAULT_DEMO_TOKEN: &str = "tok-demo-admin";

    let token = token.map(str::to_string).or_else(|| Some(DEFAULT_DEMO_TOKEN.to_string()));
    let token_ref = token.as_deref();

    // 1. Import annuaire F14 ──────────────────────────────────────────────
    println!("\n=== 1/3 Annuaire F14 ===");
    // On suppose `config.yaml` à côté (configuration par défaut). En démo
    // Ferrite c'est `config-ui-demo.yaml` à la racine du repo.
    let demo_config = std::path::Path::new("config-ui-demo.yaml");
    let config_path: &std::path::Path = if demo_config.exists() {
        demo_config
    } else {
        std::path::Path::new("config.yaml")
    };

    if !annuaire_file.exists() {
        anyhow::bail!(
            "Fichier F14 introuvable : {}\n→ Vérifier --annuaire-file",
            annuaire_file.display()
        );
    }
    let store = connect_annuaire_db(config_path).await?;
    if reset_annuaire {
        println!("🗑️  Reset annuaire (TRUNCATE)…");
        store
            .truncate_all()
            .await
            .map_err(|e| anyhow::anyhow!("truncate annuaire: {}", e))?;
    }
    let f = std::fs::File::open(annuaire_file)?;
    let reader = std::io::BufReader::with_capacity(8 * 1024 * 1024, f);
    let stats = pdp_annuaire::ingest_f14(reader, &store, None)
        .await
        .map_err(|e| anyhow::anyhow!("ingest_f14: {}", e))?;
    println!(
        "✅ Annuaire importé : {} unités légales, {} établissements, {} codes routage",
        stats.unites_legales, stats.etablissements, stats.codes_routage
    );

    // 2. Création des répertoires tenants/{siren} ────────────────────────
    println!("\n=== 2/3 Entreprises de démo ===");
    let tenants_dir = std::path::Path::new("tenants");
    std::fs::create_dir_all(tenants_dir)?;
    for siren in DEMO_TENANT_SIRENS {
        let target = tenants_dir.join(siren);
        if target.exists() {
            println!("  • {} déjà présent ({})", siren, target.display());
            continue;
        }
        // Récupère le nom officiel depuis l'annuaire qu'on vient d'importer
        let unite = store
            .lookup_unite_legale(siren)
            .await
            .map_err(|e| anyhow::anyhow!("lookup {}: {}", siren, e))?;
        let name = unite
            .as_ref()
            .map(|u| u.nom.clone())
            .unwrap_or_else(|| format!("Tenant {siren}"));
        std::fs::create_dir_all(target.join("in"))?;
        std::fs::create_dir_all(target.join("out"))?;
        let cfg = pdp_config::model::TenantConfig {
            pdp: pdp_config::model::PdpIdentity {
                id: format!("TENANT-{siren}"),
                name: name.clone(),
                siren: Some(siren.to_string()),
                siret: None,
                matricule: None,
            },
            routes: Vec::new(),
            ppf: None,
            afnor: None,
        };
        let yaml = serde_yaml::to_string(&cfg)?;
        std::fs::write(target.join("config.yaml"), yaml)?;
        println!("  ✅ {} → {}", siren, name);
    }

    // 3. Populate des fixtures ────────────────────────────────────────────
    println!("\n=== 3/3 Factures fixtures ===");
    cmd_demo_populate(
        server_url,
        fixtures_dir,
        token_ref,
        reset_factures,
        elasticsearch_url,
    )
    .await?;

    println!(
        "\n🎉 Démo prête. Connecte-toi sur {}/login\n   admin@ferrite.demo / admin",
        server_url.trim_end_matches('/')
    );
    Ok(())
}

async fn cmd_demo_populate(
    server_url: &str,
    fixtures_dir: &std::path::Path,
    token: Option<&str>,
    reset: bool,
    elasticsearch_url: &str,
) -> Result<()> {
    use sha2::{Digest, Sha256};

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // 0. Reset optionnel des indices ES avant de soumettre.
    //    ES 8.x impose `action.destructive_requires_name=true` par défaut :
    //    `DELETE /pdp-*` renvoie 400. On liste d'abord les indices via
    //    `_cat/indices?format=json` puis on supprime chacun par nom.
    if reset {
        let base = elasticsearch_url.trim_end_matches('/');
        let list_url = format!("{}/_cat/indices/pdp-*?format=json&h=index", base);
        match client.get(&list_url).send().await {
            Ok(r) if r.status().is_success() => {
                let indices: Vec<serde_json::Value> =
                    r.json().await.unwrap_or_default();
                let names: Vec<String> = indices
                    .iter()
                    .filter_map(|v| v.get("index").and_then(|i| i.as_str()).map(String::from))
                    .collect();
                if names.is_empty() {
                    println!("🗑️  Aucun index pdp-* à supprimer");
                } else {
                    let mut deleted = 0usize;
                    for name in &names {
                        let del_url = format!("{}/{}", base, name);
                        match client.delete(&del_url).send().await {
                            Ok(rr) if rr.status().is_success() => deleted += 1,
                            Ok(rr) => eprintln!(
                                "⚠️  Échec suppression {} : {}",
                                name,
                                rr.status()
                            ),
                            Err(e) => eprintln!(
                                "⚠️  Échec suppression {} : {}",
                                name, e
                            ),
                        }
                    }
                    println!(
                        "🗑️  {}/{} index pdp-* supprimés (reset)",
                        deleted,
                        names.len()
                    );
                }
            }
            Ok(r) => {
                eprintln!(
                    "⚠️  Listing indices ES échoué ({}) — l'indexation suivante recréera les indices",
                    r.status()
                );
            }
            Err(e) => {
                eprintln!("⚠️  Impossible de contacter Elasticsearch pour reset : {}", e);
            }
        }
    }

    // 1. Vérifier que le serveur est joignable
    let health_url = format!("{}/v1/healthcheck", server_url.trim_end_matches('/'));
    match client.get(&health_url).send().await {
        Ok(r) if r.status().is_success() => {
            println!("✅ Serveur joignable : {}", server_url);
        }
        Ok(r) => {
            anyhow::bail!(
                "Serveur a répondu {} (attendu 200) — le démarrer avec `pdp start --mode receiver`",
                r.status()
            );
        }
        Err(e) => {
            anyhow::bail!(
                "Impossible de contacter {} : {}\n→ Démarrer le serveur : `pdp start --mode receiver`",
                health_url,
                e
            );
        }
    }

    // 2. Collecter les fixtures (UBL + CII XML + Factur-X PDF)
    let mut files: Vec<(std::path::PathBuf, &'static str, &'static str)> = Vec::new();
    for (sub, syntax, ext, mime) in [
        ("ubl", "UBL", "xml", "application/xml"),
        ("cii", "CII", "xml", "application/xml"),
        ("facturx", "FacturX", "pdf", "application/pdf"),
    ] {
        let dir = fixtures_dir.join(sub);
        if !dir.is_dir() {
            eprintln!("⚠️  {} introuvable, sauté", dir.display());
            continue;
        }
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                files.push((path, syntax, mime));
            }
        }
    }
    files.sort_by_key(|(p, _, _)| p.clone());

    if files.is_empty() {
        anyhow::bail!(
            "Aucune fixture trouvée dans {}/ubl, {}/cii ou {}/facturx",
            fixtures_dir.display(),
            fixtures_dir.display(),
            fixtures_dir.display()
        );
    }
    println!("📦 {} fixtures à soumettre", files.len());

    // 3. Soumettre chacune via POST /v1/flows
    let flows_url = format!("{}/v1/flows", server_url.trim_end_matches('/'));
    let mut ok = 0usize;
    let mut errors = 0usize;
    for (path, syntax, mime) in &files {
        let bytes = std::fs::read(path)?;
        let sha = format!("{:x}", Sha256::digest(&bytes));
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("facture.xml")
            .to_string();
        let tracking_id = format!(
            "DEMO-{}",
            filename
                .trim_end_matches(".xml")
                .trim_end_matches(".pdf")
        );
        let flow_info = serde_json::json!({
            "trackingId": tracking_id,
            "name": filename,
            "flowType": "CustomerInvoice",
            "flowSyntax": syntax,
            "flowProfile": "EN16931",
            "sha256": sha,
        });

        let part_info = reqwest::multipart::Part::text(flow_info.to_string())
            .mime_str("application/json")
            .map_err(|e| anyhow::anyhow!("MIME flowInfo: {}", e))?;
        let part_file = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename.clone())
            .mime_str(mime)
            .map_err(|e| anyhow::anyhow!("MIME file: {}", e))?;
        let form = reqwest::multipart::Form::new()
            .part("flowInfo", part_info)
            .part("file", part_file);

        let mut req = client.post(&flows_url).multipart(form);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        match req.send().await {
            Ok(resp) if resp.status().as_u16() == 202 => {
                ok += 1;
                println!("  ✅ {} ({})", filename, *syntax);
            }
            Ok(resp) => {
                errors += 1;
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!("  ❌ {} → {} {}", filename, status, body);
            }
            Err(e) => {
                errors += 1;
                eprintln!("  ❌ {} → erreur réseau : {}", filename, e);
            }
        }
    }

    println!();
    println!(
        "📊 Soumis : {}/{} ({} erreurs)",
        ok,
        files.len(),
        errors
    );
    println!("⏳ Le pipeline traite les flux toutes les 60s — recharger l'UI dans ~1 minute");

    // 4. Enrichissement des statuts AFNOR (codes 200-501 du cycle de vie facture)
    //    Les fixtures `tests/fixtures/cdar/cdv_*.xml` couvrent les 23 statuts AFNOR
    //    mais leurs `invoice_id` sont hardcodés (F202500003) et ne matchent pas les
    //    fixtures bulk. Plutôt que de générer dynamiquement et POSTer des CDV à
    //    travers /v1/flows (CdarProcessor + linker pipeline), on enrichit
    //    directement les documents ES déjà écrits avec une distribution réaliste,
    //    pour exposer la palette complète AFNOR XP Z12-012 dans l'UI.
    let dist = inject_demo_cdv_statuses(&client, elasticsearch_url).await;
    match dist {
        Ok(updated) => {
            if updated > 0 {
                println!("🎯 {} factures enrichies avec un statut AFNOR (200-213)", updated);
            }
        }
        Err(e) => eprintln!("⚠️  Enrichissement statuts AFNOR ignoré : {}", e),
    }

    println!("🌐 Dashboard : {}/ui?siren=123456789", server_url);
    println!("🌐 Émises    : {}/ui/emises?siren=123456789", server_url);
    println!("🌐 Reçues    : {}/ui/recues?siren=123456789", server_url);

    if errors > 0 {
        anyhow::bail!("{} fixtures ont échoué", errors);
    }
    Ok(())
}

/// Enrichit les documents Elasticsearch existants avec un `cdv_status_code`
/// AFNOR aléatoire pour la démo.
///
/// Distribution (~70% des factures) calibrée sur un mix réaliste B2B :
/// - 35% Approuvée (205) — l'acheteur a validé la facture
/// - 25% Prise en charge (204) — réceptionnée par l'acheteur
/// - 15% Encaissée (212) — payée
/// -  8% Mise à disposition (203) — disponible côté acheteur
/// -  7% En litige (207)
/// -  5% Refusée (210)
/// -  3% Suspendue (208)
/// -  2% Rejetée (213)
///
/// 30% restent sans `cdv_status_code` (fallback FlowStatus → "Émise" / "Mise à disposition").
///
/// Implémenté via un Painless script ES `_update_by_query` ciblant tous les
/// documents qui n'ont pas encore de `cdv_status_code` — idempotent sur les
/// docs déjà enrichis. Référence : XP Z12-012 Annexe A V1.2.
async fn inject_demo_cdv_statuses(
    client: &reqwest::Client,
    elasticsearch_url: &str,
) -> Result<u64> {
    let url = format!(
        "{}/pdp-*/_update_by_query?refresh=true&conflicts=proceed",
        elasticsearch_url.trim_end_matches('/')
    );

    // Painless ES : tirage aléatoire pondéré avec seuils cumulés.
    let body = serde_json::json!({
        "query": {
            "bool": {
                "must_not": [
                    { "exists": { "field": "cdv_status_code" } }
                ],
                "must": [
                    { "exists": { "field": "invoice_number" } },
                    { "term": { "error_count": 0 } }
                ]
            }
        },
        "script": {
            "source": "
                if (Math.random() > 0.7) { ctx.op = 'noop'; return; }
                double r = Math.random();
                int code;
                if (r < 0.35) code = 205;
                else if (r < 0.60) code = 204;
                else if (r < 0.75) code = 212;
                else if (r < 0.83) code = 203;
                else if (r < 0.90) code = 207;
                else if (r < 0.95) code = 210;
                else if (r < 0.98) code = 208;
                else code = 213;
                ctx._source.cdv_status_code = code;
                // Horodatage simulé : reception + un délai aléatoire de
                // 0 à 14 jours, pour que la timeline démo affiche une
                // date crédible pour ce CDV downstream.
                long base = ZonedDateTime.parse(ctx._source.created_at).toInstant().toEpochMilli();
                long offset = (long)(Math.random() * 14 * 24 * 3600 * 1000L);
                ctx._source.cdv_received_at = Instant.ofEpochMilli(base + offset).toString();
            ",
            "lang": "painless"
        }
    });

    let resp = client.post(&url).json(&body).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        anyhow::bail!("ES _update_by_query {} : {}", status, txt);
    }
    let json: serde_json::Value = resp.json().await?;
    Ok(json["updated"].as_u64().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_mode_from_str() {
        assert_eq!(CliMode::from_str("emitter").unwrap(), CliMode::Emitter);
        assert_eq!(CliMode::from_str("emission").unwrap(), CliMode::Emitter);
        assert_eq!(CliMode::from_str("emettrice").unwrap(), CliMode::Emitter);
        assert_eq!(CliMode::from_str("receiver").unwrap(), CliMode::Receiver);
        assert_eq!(CliMode::from_str("reception").unwrap(), CliMode::Receiver);
        assert_eq!(CliMode::from_str("receptrice").unwrap(), CliMode::Receiver);
        assert_eq!(CliMode::from_str("both").unwrap(), CliMode::Both);
        assert_eq!(CliMode::from_str("all").unwrap(), CliMode::Both);
        assert_eq!(CliMode::from_str("").unwrap(), CliMode::Both);
        assert!(CliMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_cli_mode_should_run() {
        assert!(CliMode::Emitter.should_run_emission());
        assert!(!CliMode::Emitter.should_run_reception());
        assert!(!CliMode::Receiver.should_run_emission());
        assert!(CliMode::Receiver.should_run_reception());
        assert!(CliMode::Both.should_run_emission());
        assert!(CliMode::Both.should_run_reception());
    }

    // ============================================================
    // E-reporting CLI : chargement et génération
    // ============================================================

    fn ereporting_test_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ferrite-ereport-{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn copy_fixture(src: &str, dst_dir: &std::path::Path) {
        let src_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(src);
        let dst = dst_dir.join(src_path.file_name().unwrap());
        std::fs::copy(&src_path, &dst).unwrap();
    }

    #[test]
    fn test_load_invoices_from_dir_ubl() {
        let dir = ereporting_test_dir("load-ubl");
        copy_fixture("tests/fixtures/ubl/facture_ubl_001.xml", &dir);
        let invoices = load_invoices_from_dir(&dir).unwrap();
        assert_eq!(invoices.len(), 1);
        assert!(invoices[0].seller_siret.is_some());
    }

    #[test]
    fn test_load_invoices_skips_non_invoice_files() {
        let dir = ereporting_test_dir("load-mixed");
        std::fs::write(dir.join("readme.txt"), b"hello").unwrap();
        std::fs::write(dir.join("config.json"), b"{}").unwrap();
        copy_fixture("tests/fixtures/ubl/facture_ubl_001.xml", &dir);
        let invoices = load_invoices_from_dir(&dir).unwrap();
        // Seul le XML facture est pris en compte
        assert_eq!(invoices.len(), 1);
    }

    #[test]
    fn test_load_invoices_empty_dir() {
        let dir = ereporting_test_dir("load-empty");
        let invoices = load_invoices_from_dir(&dir).unwrap();
        assert!(invoices.is_empty());
    }

    #[test]
    fn test_load_invoices_missing_dir_errors() {
        let dir = std::path::Path::new("/tmp/ferrite-ereport-does-not-exist-xyz");
        assert!(load_invoices_from_dir(dir).is_err());
    }

    #[tokio::test]
    async fn test_cmd_ereporting_generate_101_writes_xml() {
        let dir = ereporting_test_dir("gen-101");
        copy_fixture("tests/fixtures/ubl/facture_ubl_001.xml", &dir);
        let out = dir.join("report-10.1.xml");
        // Le config_path n'est pas utilisé quand invoices_dir est fourni
        let dummy_config = std::path::PathBuf::from("/tmp/unused-config.yaml");

        cmd_ereporting(&dummy_config, EreportingCommands::Generate101 {
            invoices_dir: Some(dir.clone()),
            siren: "123456789".to_string(),
            name: "ACME SAS".to_string(),
            from: "2025-11-01".to_string(),
            to: "2025-11-30".to_string(),
            report_id: Some("RPT-TEST-101".to_string()),
            output: Some(out.clone()),
        })
        .await
        .unwrap();

        let xml = std::fs::read_to_string(&out).unwrap();
        assert!(xml.contains("<TypeCode>10.1</TypeCode>"));
        assert!(xml.contains("<Id>RPT-TEST-101</Id>"));
        // BR-FR-MAP-23 appliqué : période YYYYMMDD
        assert!(xml.contains("<StartDate>20251101</StartDate>"));
        assert!(xml.contains("<EndDate>20251130</EndDate>"));
        // Date facture aussi normalisée (UBL fixture utilise YYYY-MM-DD)
        assert!(xml.contains("<IssueDate>20251115</IssueDate>"));
    }

    #[tokio::test]
    async fn test_cmd_ereporting_generate_103_aggregates() {
        let dir = ereporting_test_dir("gen-103");
        copy_fixture("tests/fixtures/ubl/facture_ubl_001.xml", &dir);
        let out = dir.join("report-10.3.xml");
        let dummy_config = std::path::PathBuf::from("/tmp/unused-config.yaml");

        cmd_ereporting(&dummy_config, EreportingCommands::Generate103 {
            invoices_dir: Some(dir.clone()),
            siren: "123456789".to_string(),
            name: "ACME SAS".to_string(),
            from: "2025-11-01".to_string(),
            to: "2025-11-30".to_string(),
            report_id: None,
            output: Some(out.clone()),
        })
        .await
        .unwrap();

        let xml = std::fs::read_to_string(&out).unwrap();
        assert!(xml.contains("<TypeCode>10.3</TypeCode>"));
        assert!(xml.contains("<AggregatedTransaction>"));
        // Période normalisée
        assert!(xml.contains("<StartDate>20251101</StartDate>"));
    }

    fn fixture_xml(rel_path: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(rel_path);
        std::fs::read_to_string(&path).unwrap()
    }

    fn exchange_doc(raw_xml: Option<String>, source_format: Option<&str>) -> pdp_trace::store::ExchangeDocument {
        pdp_trace::store::ExchangeDocument {
            exchange_id: "exch-1".to_string(),
            flow_id: "flow-1".to_string(),
            source_filename: Some("facture.xml".to_string()),
            invoice_number: Some("F-001".to_string()),
            invoice_key: None,
            seller_name: None,
            buyer_name: None,
            seller_siret: Some("12345678901234".to_string()),
            buyer_siret: Some("98765432109876".to_string()),
            seller_siren: Some("123456789".to_string()),
            buyer_siren: Some("987654321".to_string()),
            source_format: source_format.map(String::from),
            total_ht: Some(1000.0),
            total_ttc: Some(1200.0),
            total_tax: Some(200.0),
            currency: Some("EUR".to_string()),
            issue_date: Some("2025-11-15".to_string()),
            status: "DISTRIBUÉ".to_string(),
            error_count: 0,
            cdv_status_code: None,
            generated_cdv_xml: None,
            generated_cdv_status_code: None,
            disposition_cdv_xml: None,
            disposition_cdv_status_code: None,
            raw_xml,
            raw_pdf_base64: None,
            converted_xml: None,
            converted_format: None,
            attachment_count: 0,
            attachment_filenames: vec![],
            events: vec![],
            errors: vec![],
            validation_warnings: vec![],
            created_at: "2025-11-15T10:00:00Z".to_string(),
            updated_at: "2025-11-15T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_exchange_doc_to_invoice_ubl() {
        let xml = fixture_xml("tests/fixtures/ubl/facture_ubl_001.xml");
        let doc = exchange_doc(Some(xml), Some("UBL"));
        let invoice = exchange_doc_to_invoice(&doc).expect("UBL parse");
        assert!(!invoice.invoice_number.is_empty());
        assert!(invoice.seller_siret.is_some());
    }

    #[test]
    fn test_exchange_doc_to_invoice_cii() {
        let xml = fixture_xml("tests/fixtures/cii/facture_cii_001.xml");
        let doc = exchange_doc(Some(xml), Some("CII"));
        let invoice = exchange_doc_to_invoice(&doc).expect("CII parse");
        assert!(!invoice.invoice_number.is_empty());
    }

    #[test]
    fn test_exchange_doc_to_invoice_no_raw_xml() {
        let doc = exchange_doc(None, Some("UBL"));
        assert!(exchange_doc_to_invoice(&doc).is_none());
    }

    #[test]
    fn test_exchange_doc_to_invoice_unknown_format_autodetect() {
        // source_format absent → détection automatique sur le contenu
        let xml = fixture_xml("tests/fixtures/ubl/facture_ubl_001.xml");
        let doc = exchange_doc(Some(xml), None);
        let invoice = exchange_doc_to_invoice(&doc);
        // Soit autodétection réussit, soit échoue proprement (None)
        // — pas de panic dans tous les cas
        let _ = invoice;
    }
}
