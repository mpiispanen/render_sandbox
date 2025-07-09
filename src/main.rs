use clap::Parser;
use render_sandbox::Args;

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
