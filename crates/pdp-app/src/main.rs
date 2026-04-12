mod server;

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
    Start,

    /// Exécute toutes les routes une seule fois
    Run,

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
        /// Format cible (UBL ou CII)
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
        Commands::Start => cmd_start(&cli.config).await,
        Commands::Run => cmd_run(&cli.config).await,
        Commands::RunRoute { route_id } => cmd_run_route(&cli.config, &route_id).await,
        Commands::ListRoutes => cmd_list_routes(&cli.config).await,
        Commands::Parse { file } => cmd_parse(&file).await,
        Commands::Validate { file } => cmd_validate(&file).await,
        Commands::Transform { file, to, output } => cmd_transform(&file, &to, output.as_deref()).await,
        Commands::Stats => cmd_stats(&cli.config).await,
        Commands::Errors => cmd_errors(&cli.config).await,
        Commands::FlowEvents { flow_id } => cmd_flow_events(&cli.config, &flow_id).await,
        Commands::Annuaire { action } => cmd_annuaire(&cli.config, action).await,
    }
}

async fn cmd_start(config_path: &std::path::Path) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;

    tracing::info!(
        pdp_id = %config.pdp.id,
        pdp_name = %config.pdp.name,
        routes = config.routes.len(),
        interval = config.polling.interval_secs,
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

    // Construire le router avec la route HTTP inbound
    let router = build_router(&config, base_dir, Some(http_exchange_rx)).await?;

    // Démarrer le serveur HTTP si configuré
    if let Some(ref http_config) = config.http_server {
        let trace_store = match pdp_trace::TraceStore::new(&config.elasticsearch.url).await {
            Ok(store) => Some(std::sync::Arc::new(store)),
            Err(e) => {
                tracing::warn!(error = %e, "Impossible de connecter le TraceStore au serveur HTTP");
                None
            }
        };

        // Connexion PostgreSQL pour l'annuaire PPF (optionnelle)
        let annuaire_store = if let Some(ref db_config) = config.database {
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
                    tracing::info!("Annuaire PPF connecté (PostgreSQL)");
                    Some(store)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Impossible de connecter PostgreSQL pour l'annuaire");
                    None
                }
            }
        } else {
            None
        };

        let app_state = std::sync::Arc::new(server::AppState {
            pdp_name: config.pdp.name.clone(),
            pdp_matricule: config.pdp.matricule.clone().unwrap_or_default(),
            flow_sender: flow_tx.clone(),
            webhook_secret: http_config.webhook_secret.clone(),
            bearer_tokens: http_config.bearer_tokens.clone(),
            trace_store,
            metrics: server::Metrics::default(),
            annuaire_store,
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

async fn cmd_run(config_path: &std::path::Path) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;

    tracing::info!("Exécution unique de toutes les routes");
    let base_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
    let router = build_router(&config, base_dir, None).await?;
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
    let router = build_router(&config, base_dir, None).await?;

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

    let result_xml = match target.to_uppercase().as_str() {
        "CII" | "UBL" => {
            let target_format = if target.to_uppercase() == "CII" {
                pdp_core::model::InvoiceFormat::CII
            } else {
                pdp_core::model::InvoiceFormat::UBL
            };
            let result = pdp_transform::convert(&invoice, target_format)?;
            String::from_utf8(result.content)?
        }
        _ => anyhow::bail!("Format cible non supporté: {}. Utilisez UBL ou CII.", target),
    };

    if let Some(out_path) = output {
        std::fs::write(out_path, &result_xml)?;
        println!("✅ Transformation {} -> {} écrite dans {}", format, target.to_uppercase(), out_path.display());
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

/// Construit le Router à partir de la configuration.
/// Si `http_rx` est fourni, une route "http-inbound" est ajoutée pour traiter
/// les flux reçus via l'API HTTP (ChannelConsumer).
///
/// Si `tenants_dir` est configuré, des routes auto-générées sont créées pour
/// chaque tenant découvert : `{siren}/in` → pipeline → `{siren}/out`.
async fn build_router(
    config: &pdp_config::PdpConfig,
    base_dir: &std::path::Path,
    http_rx: Option<tokio::sync::mpsc::Receiver<pdp_core::Exchange>>,
) -> Result<pdp_core::Router> {
    let store = match pdp_trace::TraceStore::new(&config.elasticsearch.url).await {
        Ok(s) => std::sync::Arc::new(s),
        Err(e) => {
            tracing::warn!(error = %e, "Elasticsearch indisponible — traçabilité désactivée");
            std::sync::Arc::new(pdp_trace::TraceStore::noop())
        }
    };

    // Construire les producers PPF et AFNOR si configurés
    let ppf_producer = build_ppf_producer(config)?;
    let (annuaire_client, partner_directory, afnor_producers) = build_afnor_clients(config)?;

    // Construire le AlertErrorHandler depuis la config
    let alert_config = config.alerts.as_ref();

    let mut router = pdp_core::Router::new();

    for route_config in &config.routes {
        if !route_config.enabled {
            tracing::info!(route_id = %route_config.id, "Route désactivée, skip");
            continue;
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

        // Construire la chaîne de processors
        let mut builder = pdp_core::RouteBuilder::new(&route_config.id)
            .description(&route_config.description)
            .from_source(consumer)
            // 1. Trace : réception
            .process(Box::new(pdp_trace::TraceProcessor::received(store.clone())))
            .process(Box::new(pdp_core::processor::LogProcessor::info("reception")))
            // 1b. Contrôles de réception (taille, extension, nom, doublons)
            .process(Box::new(pdp_core::reception::ReceptionProcessor::strict()))
            // 1c. CDAR 501 d'irrecevabilité si contrôles de réception échoués
            .process(Box::new(pdp_cdar::IrrecevabiliteProcessor::new(
                &config.pdp.id,
                &config.pdp.name,
            )))
            // 1d. Détection type de document (facture vs CDAR vs e-reporting)
            .process(Box::new(pdp_cdar::DocumentTypeRouter::new()))
            // 2. Parsing : détection format + extraction données (skip si CDAR)
            .process(Box::new(pdp_invoice::ParseProcessor::new()))
            .process(Box::new(pdp_trace::TraceProcessor::parsed(store.clone())))
            // 2b. Détection de doublons persistante (BR-FR-12/13 via Elasticsearch)
            .process(Box::new(pdp_trace::DuplicateCheckProcessor::new(store.clone())));

        // 3. Validation (optionnelle)
        if route_config.validate {
            builder = builder
                .process(Box::new(pdp_invoice::ValidateProcessor::new()))
                .process(Box::new(pdp_validate::XmlValidateProcessor::with_options(
                    &config.validation.specs_dir,
                    config.validation.xsd_enabled,
                    config.validation.en16931_enabled,
                    config.validation.br_fr_enabled,
                    true, // strict
                )))
                .process(Box::new(pdp_trace::TraceProcessor::validated(store.clone())));
        }

        // 4a. Génération Flux 1 PPF (données réglementaires pour la PPF)
        if config.ppf.is_some() {
            let ppf = config.ppf.as_ref().unwrap();
            let strategy = pdp_transform::Flux1ProfileStrategy::from_config(&ppf.flux1_profile);
            builder = builder.process(Box::new(pdp_transform::PpfFlux1Processor::new(
                std::path::Path::new(&ppf.flux1_output_dir),
                std::path::Path::new(&config.validation.specs_dir),
            ).with_strategy(strategy)));
        }

        // 4b. Transformation (optionnelle)
        if let Some(ref target) = route_config.transform_to {
            let target_format = match target.to_uppercase().as_str() {
                "CII" => pdp_core::model::InvoiceFormat::CII,
                "UBL" => pdp_core::model::InvoiceFormat::UBL,
                _ => {
                    tracing::warn!(target = %target, "Format de transformation non supporté");
                    continue;
                }
            };
            builder = builder
                .process(Box::new(pdp_transform::TransformProcessor::new(target_format)))
                .process(Box::new(pdp_trace::TraceProcessor::transformed(store.clone())));
        }

        // 5. Génération CDAR (optionnelle)
        if route_config.generate_cdar {
            if let Some(ref _receiver) = route_config.cdar_receiver {
                builder = builder.process(Box::new(pdp_cdar::CdarProcessor::new(
                    &config.pdp.id,
                    &config.pdp.name,
                )));
            }
        }

        // 5b. Résolution de routage (si destination PPF, après parsing et validation)
        if route_config.destination.endpoint_type == "ppf" {
            if let (Some(ref annuaire), Some(ref partner_dir)) = (&annuaire_client, &partner_directory) {
                builder = builder.process(Box::new(pdp_client::RoutingResolverProcessor::new(
                    annuaire.clone(),
                    partner_dir.clone(),
                )));
            }
        }

        // 6. Destination + trace finale
        builder = builder
            .to_destination(producer)
            .process(Box::new(pdp_trace::TraceProcessor::distributed(store.clone())));

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

            // Producer : FileEndpoint sur {siren}/out/
            // Si PPF configuré, utilise DynamicRoutingProducer avec fallback sur out/
            let producer: Box<dyn pdp_core::endpoint::Producer> = if let Some(ref ppf_prod) = ppf_producer {
                let mut dynamic = pdp_client::DynamicRoutingProducer::new(
                    &format!("{}-dynamic-dest", route_id),
                    ppf_prod.clone(),
                );
                for (matricule, producer) in &afnor_producers {
                    dynamic.add_afnor_producer(matricule, producer.clone());
                }
                dynamic = dynamic.with_fallback_path(out_path.to_str().unwrap_or("."));
                Box::new(dynamic)
            } else {
                Box::new(pdp_core::endpoint::FileEndpoint::output(
                    &format!("{}-dest", route_id),
                    out_path.to_str().unwrap_or("."),
                ))
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

            // Chaîne de processors identique aux routes manuelles
            let mut builder = pdp_core::RouteBuilder::new(&route_id)
                .description(&format!("Route auto-générée pour tenant {}", siren))
                .from_source(consumer)
                // 0. Tag tenant SIREN sur chaque exchange
                .process(Box::new(pdp_core::TenantTagProcessor::new(siren)))
                // 1. Trace : réception
                .process(Box::new(pdp_trace::TraceProcessor::received(store.clone())))
                .process(Box::new(pdp_core::processor::LogProcessor::info(&format!("reception-{}", siren))))
                // 1b. Contrôles de réception
                .process(Box::new(pdp_core::reception::ReceptionProcessor::strict()))
                // 1c. CDAR 501 d'irrecevabilité
                .process(Box::new(pdp_cdar::IrrecevabiliteProcessor::new(pdp_id, pdp_name)))
                // 1d. Détection type de document
                .process(Box::new(pdp_cdar::DocumentTypeRouter::new()))
                // 2. Parsing
                .process(Box::new(pdp_invoice::ParseProcessor::new()))
                .process(Box::new(pdp_trace::TraceProcessor::parsed(store.clone())))
                // 2b. Détection de doublons
                .process(Box::new(pdp_trace::DuplicateCheckProcessor::new(store.clone())));

            // 3. Validation
            builder = builder
                .process(Box::new(pdp_invoice::ValidateProcessor::new()))
                .process(Box::new(pdp_validate::XmlValidateProcessor::with_options(
                    &config.validation.specs_dir,
                    config.validation.xsd_enabled,
                    config.validation.en16931_enabled,
                    config.validation.br_fr_enabled,
                    true,
                )))
                .process(Box::new(pdp_trace::TraceProcessor::validated(store.clone())));

            // 4a. Génération Flux 1 PPF
            let tenant_ppf = tenant.config.ppf.as_ref().or(config.ppf.as_ref());
            if let Some(ppf) = tenant_ppf {
                let strategy = pdp_transform::Flux1ProfileStrategy::from_config(&ppf.flux1_profile);
                builder = builder.process(Box::new(pdp_transform::PpfFlux1Processor::new(
                    std::path::Path::new(&ppf.flux1_output_dir),
                    std::path::Path::new(&config.validation.specs_dir),
                ).with_strategy(strategy)));
            }

            // 5. Génération CDAR
            builder = builder.process(Box::new(pdp_cdar::CdarProcessor::new(pdp_id, pdp_name)));

            // 5b. Résolution de routage
            if let (Some(ref annuaire), Some(ref partner_dir)) = (&annuaire_client, &partner_directory) {
                builder = builder.process(Box::new(pdp_client::RoutingResolverProcessor::new(
                    annuaire.clone(),
                    partner_dir.clone(),
                )));
            }

            // 6. Destination + trace finale
            builder = builder
                .to_destination(producer)
                .process(Box::new(pdp_trace::TraceProcessor::distributed(store.clone())))
                .on_error(error_handler);

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

    // Route HTTP inbound : traite les flux reçus via l'API HTTP AFNOR
    if let Some(rx) = http_rx {
        let consumer: Box<dyn pdp_core::endpoint::Consumer> =
            Box::new(pdp_core::ChannelConsumer::new("http-inbound-source", rx));

        // Destination par défaut : même que la première route configurée, ou fichier local
        let default_output = config.routes.first()
            .map(|r| r.destination.path.clone())
            .unwrap_or_else(|| "output/http-inbound".to_string());

        // Construire le producer (destination) — utilise PPF si configuré, sinon fichier
        let producer: Box<dyn pdp_core::endpoint::Producer> = if ppf_producer.is_some() {
            let ppf_prod = ppf_producer.as_ref().unwrap();
            let mut dynamic = pdp_client::DynamicRoutingProducer::new(
                "http-inbound-dynamic-dest",
                ppf_prod.clone(),
            );
            for (matricule, producer) in &afnor_producers {
                dynamic.add_afnor_producer(matricule, producer.clone());
            }
            dynamic = dynamic.with_fallback_path(&default_output);
            Box::new(dynamic)
        } else {
            Box::new(pdp_core::endpoint::FileEndpoint::output(
                "http-inbound-dest",
                &default_output,
            ))
        };

        let mut builder = pdp_core::RouteBuilder::new("http-inbound")
            .description("Route pour les flux reçus via l'API HTTP AFNOR")
            .from_source(consumer)
            // 1. Trace : réception
            .process(Box::new(pdp_trace::TraceProcessor::received(store.clone())))
            .process(Box::new(pdp_core::processor::LogProcessor::info("http-reception")))
            // 1b. Contrôles de réception
            .process(Box::new(pdp_core::reception::ReceptionProcessor::strict()))
            // 1c. CDAR 501 d'irrecevabilité
            .process(Box::new(pdp_cdar::IrrecevabiliteProcessor::new(
                &config.pdp.id,
                &config.pdp.name,
            )))
            // 1d. Détection type de document
            .process(Box::new(pdp_cdar::DocumentTypeRouter::new()))
            // 2. Parsing
            .process(Box::new(pdp_invoice::ParseProcessor::new()))
            .process(Box::new(pdp_trace::TraceProcessor::parsed(store.clone())))
            // 2b. Détection de doublons persistante (BR-FR-12/13)
            .process(Box::new(pdp_trace::DuplicateCheckProcessor::new(store.clone())))
            // 3. Validation
            .process(Box::new(pdp_invoice::ValidateProcessor::new()))
            .process(Box::new(pdp_validate::XmlValidateProcessor::with_options(
                &config.validation.specs_dir,
                config.validation.xsd_enabled,
                config.validation.en16931_enabled,
                config.validation.br_fr_enabled,
                true,
            )))
            .process(Box::new(pdp_trace::TraceProcessor::validated(store.clone())));

        // 4a. Flux 1 PPF
        if let Some(ref ppf) = config.ppf {
            let strategy = pdp_transform::Flux1ProfileStrategy::from_config(&ppf.flux1_profile);
            builder = builder.process(Box::new(pdp_transform::PpfFlux1Processor::new(
                std::path::Path::new(&ppf.flux1_output_dir),
                std::path::Path::new(&config.validation.specs_dir),
            ).with_strategy(strategy)));
        }

        // 5. CDAR
        builder = builder.process(Box::new(pdp_cdar::CdarProcessor::new(
            &config.pdp.id,
            &config.pdp.name,
        )));

        // 5b. Résolution de routage
        if let (Some(ref annuaire), Some(ref partner_dir)) = (&annuaire_client, &partner_directory) {
            builder = builder.process(Box::new(pdp_client::RoutingResolverProcessor::new(
                annuaire.clone(),
                partner_dir.clone(),
            )));
        }

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

        // 6. Destination + trace finale + error handler
        builder = builder
            .to_destination(producer)
            .process(Box::new(pdp_trace::TraceProcessor::distributed(store.clone())))
            .on_error(Box::new(http_alert_handler));

        let route = builder.build()?;
        router.add_route(route)?;

        tracing::info!("Route 'http-inbound' ajoutée pour les flux API HTTP");
    }

    Ok(router)
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

    let producer_config = pdp_client::PpfSftpProducerConfig {
        code_application: ppf.code_application_piste.clone(),
        default_profil: ppf.flux1_profile.clone(),
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
