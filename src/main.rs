use clap::Parser;

/// A graphics rendering application
#[derive(Parser, Debug)]
#[command(name = "render_sandbox")]
#[command(about = "A sandbox for trying out different graphics rendering algorithms")]
#[command(version = "0.1.0")]
struct Args {
    /// Rendering resolution width
    #[arg(short = 'w', long = "width", default_value = "800")]
    width: u32,

    /// Rendering resolution height  
    #[arg(long = "height", default_value = "600")]
    height: u32,

    /// Output file path
    #[arg(short = 'o', long = "output", default_value = "output.png")]
    output: String,

    /// Output format (png, jpg, bmp)
    #[arg(short = 'f', long = "format", default_value = "png")]
    format: String,

    /// Number of samples for anti-aliasing
    #[arg(short = 's', long = "samples", default_value = "1")]
    samples: u32,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("Starting render_sandbox with configuration:");
        println!("  Resolution: {}x{}", args.width, args.height);
        println!("  Output: {}", args.output);
        println!("  Format: {}", args.format);
        println!("  Samples: {}", args.samples);
        println!("  Verbose: {}", args.verbose);
    }

    // Validate resolution
    if args.width == 0 || args.height == 0 {
        eprintln!("Error: Resolution must be greater than 0");
        std::process::exit(1);
    }

    // Validate format
    match args.format.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "bmp" => {}
        _ => {
            eprintln!(
                "Error: Unsupported format '{}'. Supported formats: png, jpg, jpeg, bmp",
                args.format
            );
            std::process::exit(1);
        }
    }

    println!(
        "Rendering {}x{} image to {} (format: {})",
        args.width, args.height, args.output, args.format
    );

    // TODO: Implement actual rendering logic here
    println!("Rendering complete!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["render_sandbox"]);
        assert_eq!(args.width, 800);
        assert_eq!(args.height, 600);
        assert_eq!(args.output, "output.png");
        assert_eq!(args.format, "png");
        assert_eq!(args.samples, 1);
        assert!(!args.verbose);
    }

    #[test]
    fn test_custom_resolution() {
        let args = Args::parse_from(["render_sandbox", "--width", "1920", "--height", "1080"]);
        assert_eq!(args.width, 1920);
        assert_eq!(args.height, 1080);
    }

    #[test]
    fn test_short_args() {
        let args = Args::parse_from([
            "render_sandbox",
            "-w",
            "1024",
            "--height",
            "768",
            "-o",
            "test.jpg",
            "-f",
            "jpg",
            "-s",
            "4",
            "-v",
        ]);
        assert_eq!(args.width, 1024);
        assert_eq!(args.height, 768);
        assert_eq!(args.output, "test.jpg");
        assert_eq!(args.format, "jpg");
        assert_eq!(args.samples, 4);
        assert!(args.verbose);
    }

    #[test]
    fn test_long_args() {
        let args = Args::parse_from([
            "render_sandbox",
            "--width",
            "2560",
            "--height",
            "1440",
            "--output",
            "render.png",
            "--format",
            "png",
            "--samples",
            "8",
            "--verbose",
        ]);
        assert_eq!(args.width, 2560);
        assert_eq!(args.height, 1440);
        assert_eq!(args.output, "render.png");
        assert_eq!(args.format, "png");
        assert_eq!(args.samples, 8);
        assert!(args.verbose);
    }

    #[test]
    fn test_validation_functions() {
        // Test format validation
        let valid_formats = ["png", "jpg", "jpeg", "bmp"];
        for format in valid_formats {
            let args = Args::parse_from(["render_sandbox", "--format", format]);
            assert_eq!(args.format, format);
        }
    }

    #[test]
    fn test_mixed_case_formats() {
        let args = Args::parse_from(["render_sandbox", "--format", "PNG"]);
        assert_eq!(args.format, "PNG");
    }
}
