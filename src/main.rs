use clap::Parser;
use log::{debug, error, info};
use render_sandbox::{app_core::Application, Args};

fn main() {
    let args = Args::parse();

    // Initialize logger
    let log_level = match args.log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => {
            eprintln!(
                "Error: Invalid log level '{}'. Valid levels: error, warn, info, debug, trace",
                args.log_level
            );
            std::process::exit(1);
        }
    };

    env_logger::Builder::new().filter_level(log_level).init();

    debug!("Starting render_sandbox with arguments: {args:?}");

    if args.verbose {
        info!("Starting render_sandbox with configuration:");
        info!("  Resolution: {}x{}", args.width, args.height);
        info!("  Output: {}", args.output);
        info!("  Format: {}", args.format);
        info!("  Samples: {}", args.samples);
        info!("  Verbose: {}", args.verbose);
        info!("  Log level: {}", args.log_level);
        info!("  Headless: {}", args.headless);
    }

    // Validate resolution
    if args.width == 0 || args.height == 0 {
        error!("Resolution must be greater than 0");
        std::process::exit(1);
    }

    // Validate format
    match args.format.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "bmp" => {
            debug!("Format validation passed for: {}", args.format);
        }
        _ => {
            error!(
                "Unsupported format '{}'. Supported formats: png, jpg, jpeg, bmp",
                args.format
            );
            std::process::exit(1);
        }
    }

    info!(
        "Starting render_sandbox {}x{} (format: {}, headless: {})",
        args.width, args.height, args.format, args.headless
    );

    // Create and run the application
    match Application::new(args.headless, args.gltf_path) {
        Ok(app) => {
            if let Err(e) = app.run() {
                error!("Application error: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to create application: {e}");
            std::process::exit(1);
        }
    }

    info!("Application completed successfully");
}
