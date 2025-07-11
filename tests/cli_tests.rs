use clap::Parser;
use render_sandbox::Args;

#[test]
fn test_default_args() {
    let args = Args::parse_from(["render_sandbox"]);
    assert_eq!(args.width, 800);
    assert_eq!(args.height, 600);
    assert_eq!(args.output, "output.png");
    assert_eq!(args.format, "png");
    assert_eq!(args.samples, 1);
    assert!(!args.verbose);
    assert_eq!(args.log_level, "info");
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
        "-l",
        "debug",
    ]);
    assert_eq!(args.width, 1024);
    assert_eq!(args.height, 768);
    assert_eq!(args.output, "test.jpg");
    assert_eq!(args.format, "jpg");
    assert_eq!(args.samples, 4);
    assert!(args.verbose);
    assert_eq!(args.log_level, "debug");
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
        "--log-level",
        "warn",
    ]);
    assert_eq!(args.width, 2560);
    assert_eq!(args.height, 1440);
    assert_eq!(args.output, "render.png");
    assert_eq!(args.format, "png");
    assert_eq!(args.samples, 8);
    assert!(args.verbose);
    assert_eq!(args.log_level, "warn");
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

#[test]
fn test_log_level_options() {
    let valid_levels = ["error", "warn", "info", "debug", "trace"];
    for level in valid_levels {
        let args = Args::parse_from(["render_sandbox", "--log-level", level]);
        assert_eq!(args.log_level, level);
    }
}

#[test]
fn test_log_level_short_option() {
    let args = Args::parse_from(["render_sandbox", "-l", "error"]);
    assert_eq!(args.log_level, "error");
}

#[test]
fn test_log_level_mixed_case() {
    let args = Args::parse_from(["render_sandbox", "--log-level", "ERROR"]);
    assert_eq!(args.log_level, "ERROR");
}

#[test]
fn test_headless_flag() {
    let args = Args::parse_from(["render_sandbox", "--headless"]);
    assert!(args.headless);
}

#[test]
fn test_default_headless_false() {
    let args = Args::parse_from(["render_sandbox"]);
    assert!(!args.headless);
}
