use clap::Parser;

pub mod app_core;
pub mod engine;
pub mod gltf_loader;
pub mod graphics_api;
pub mod pipeline;
pub mod render_graph;
pub mod render_passes;
pub mod renderer;
pub mod resource_manager;
pub mod scene;

/// A graphics rendering application
#[derive(Parser, Debug)]
#[command(name = "render_sandbox")]
#[command(about = "A sandbox for trying out different graphics rendering algorithms")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Rendering resolution width
    #[arg(short = 'w', long = "width", default_value = "800")]
    pub width: u32,

    /// Rendering resolution height  
    #[arg(long = "height", default_value = "600")]
    pub height: u32,

    /// Output file path
    #[arg(short = 'o', long = "output", default_value = "output.png")]
    pub output: String,

    /// Output format (png, jpg, bmp)
    #[arg(short = 'f', long = "format", default_value = "png")]
    pub format: String,

    /// Number of samples for anti-aliasing
    #[arg(short = 's', long = "samples", default_value = "1")]
    pub samples: u32,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short = 'l', long = "log-level", default_value = "info")]
    pub log_level: String,

    /// Run in headless mode (no window)
    #[arg(long = "headless")]
    pub headless: bool,

    /// Path to GLTF file to load
    #[arg(
        short = 'g',
        long = "gltf",
        default_value = "test_assets/triangle.gltf"
    )]
    pub gltf_path: String,
}
