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

    let router = build_router(&config).await?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Gérer Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Signal d'arrêt reçu (Ctrl+C)");
        let _ = shutdown_tx_clone.send(true);
    });

    let interval = std::time::Duration::from_secs(config.polling.interval_secs);
    router.start_polling(interval, shutdown_rx).await?;

    tracing::info!("PDP arrêtée proprement");
    Ok(())
}

async fn cmd_run(config_path: &std::path::Path) -> Result<()> {
    let config = pdp_config::load_config(config_path.to_str().unwrap_or("config.yaml"))?;

    tracing::info!("Exécution unique de toutes les routes");
    let router = build_router(&config).await?;
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
    let router = build_router(&config).await?;

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
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url).await?;
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
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url).await?;
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
    let store = pdp_trace::TraceStore::new(&config.elasticsearch.url).await?;

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

/// Construit le Router à partir de la configuration
async fn build_router(config: &pdp_config::PdpConfig) -> Result<pdp_core::Router> {
    let store = std::sync::Arc::new(
        pdp_trace::TraceStore::new(&config.elasticsearch.url).await?
    );

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

        // Construire le producer (destination)
        let producer: Box<dyn pdp_core::endpoint::Producer> = Box::new(
            pdp_core::endpoint::FileEndpoint::output(
                &format!("{}-dest", route_config.id),
                &route_config.destination.path,
            ),
        );

        // Construire le error handler
        let error_handler: Option<Box<dyn pdp_core::endpoint::Producer>> =
            route_config.error_destination.as_ref().map(|err_dest| {
                Box::new(pdp_core::endpoint::FileEndpoint::output(
                    &format!("{}-errors", route_config.id),
                    &err_dest.path,
                )) as Box<dyn pdp_core::endpoint::Producer>
            });

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
            // Si CDAR : parse immédiatement, set document.type=CDAR, les processors
            // suivants (Parse, Validate, Transform) skipperont automatiquement
            .process(Box::new(pdp_cdar::DocumentTypeRouter::new()))
            // 2. Parsing : détection format + extraction données (skip si CDAR)
            .process(Box::new(pdp_invoice::ParseProcessor::new()))
            .process(Box::new(pdp_trace::TraceProcessor::parsed(store.clone())));

        // 3. Validation (optionnelle)
        if route_config.validate {
            // 3a. Validation métier (champs obligatoires, cohérence)
            builder = builder
                .process(Box::new(pdp_invoice::ValidateProcessor::new()));

            // 3b. Validation XML (XSD + Schematron EN16931 + BR-FR)
            builder = builder
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

    Ok(router)
}
