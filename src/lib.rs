use clap::Parser;

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
}
