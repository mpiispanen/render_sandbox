use std::fs;
use std::path::Path;
use std::process::Command;

/// Test cases for visual regression testing
/// These tests generate images that are compared against golden masters
#[derive(Debug)]
struct TestCase {
    name: &'static str,
    width: u32,
    height: u32,
    samples: u32,
    description: &'static str,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "basic_render_800x600",
        width: 800,
        height: 600,
        samples: 1,
        description: "Basic rendering test at 800x600 resolution",
    },
    TestCase {
        name: "high_res_1920x1080",
        width: 1920,
        height: 1080,
        samples: 1,
        description: "High resolution rendering test",
    },
    TestCase {
        name: "square_512x512",
        width: 512,
        height: 512,
        samples: 1,
        description: "Square aspect ratio rendering test",
    },
    TestCase {
        name: "antialiased_4x",
        width: 800,
        height: 600,
        samples: 4,
        description: "Anti-aliased rendering with 4x MSAA",
    },
    TestCase {
        name: "minimal_400x300",
        width: 400,
        height: 300,
        samples: 1,
        description: "Minimal resolution rendering test",
    },
];

/// Ensure the render_sandbox binary exists
#[allow(dead_code)]
fn ensure_binary_exists() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = Path::new("./target/release/render_sandbox");
    if !binary_path.exists() {
        println!("render_sandbox binary not found, building...");
        let output = Command::new("cargo")
            .args(["build", "--release"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to build render_sandbox: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        println!("Successfully built render_sandbox");
    }
    Ok(())
}

/// Generate a test image using the render_sandbox binary
#[allow(dead_code)]
fn generate_test_image(test_case: &TestCase) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = format!("outputs/{}.png", test_case.name);

    let mut cmd = Command::new("./target/release/render_sandbox");
    cmd.arg("--output")
        .arg(&output_path)
        .arg("--width")
        .arg(test_case.width.to_string())
        .arg("--height")
        .arg(test_case.height.to_string())
        .arg("--format")
        .arg("png")
        .arg("--samples")
        .arg(test_case.samples.to_string())
        .arg("--headless"); // Run in headless mode for CI

    println!("Running command: {cmd:?}");

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("render_sandbox failed:\nstdout: {stdout}\nstderr: {stderr}").into());
    }

    // Verify the output file was created
    let path = Path::new(&output_path);
    if !path.exists() {
        return Err(format!("Output image was not created: {output_path}").into());
    }

    let file_size = fs::metadata(path)?.len();
    if file_size < 1000 {
        return Err(
            format!("Output image is too small ({file_size} bytes), likely invalid").into(),
        );
    }

    println!(
        "✅ Generated {} ({} bytes) - {}",
        output_path, file_size, test_case.description
    );
    Ok(())
}

/// Generate all visual regression test images
/// This test runs during `cargo test` and generates the images that will be compared
/// against golden masters by the CI image comparison workflow
///
/// This test requires GPU access and should only run on GPU-enabled runners
#[test]
#[cfg(feature = "gpu-tests")]
fn generate_visual_regression_images() {
    // Ensure outputs directory exists
    fs::create_dir_all("outputs").expect("Failed to create outputs directory");

    // Ensure binary exists
    if let Err(e) = ensure_binary_exists() {
        panic!("Failed to ensure binary exists: {e}");
    }

    println!("Generating visual regression test images...");
    println!("{}", "=".repeat(60));

    let mut success_count = 0;
    let total_count = TEST_CASES.len();

    for test_case in TEST_CASES {
        println!("\nGenerating: {}", test_case.name);
        println!("Description: {}", test_case.description);

        match generate_test_image(test_case) {
            Ok(()) => {
                success_count += 1;
            }
            Err(e) => {
                println!("❌ Failed to generate {}: {}", test_case.name, e);
                // For visual regression testing, we require GPU access - fail the test if we can't generate images
                panic!(
                    "Visual regression test failed for {}: {}",
                    test_case.name, e
                );
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Visual regression image generation completed!");
    println!("Success: {success_count}/{total_count} images processed");

    if success_count < total_count {
        panic!(
            "Failed to generate {} out of {} visual regression test images",
            total_count - success_count,
            total_count
        );
    }

    // List generated images
    if let Ok(entries) = fs::read_dir("outputs") {
        println!("\nGenerated test images:");
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "png" {
                    if let Ok(metadata) = entry.metadata() {
                        println!(
                            "  - {} ({} bytes)",
                            entry.file_name().to_string_lossy(),
                            metadata.len()
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_case_definitions() {
        // Ensure all test cases have valid configurations
        for test_case in TEST_CASES {
            assert!(!test_case.name.is_empty(), "Test case name cannot be empty");
            assert!(test_case.width > 0, "Width must be positive");
            assert!(test_case.height > 0, "Height must be positive");
            assert!(test_case.samples > 0, "Samples must be positive");
            assert!(
                !test_case.description.is_empty(),
                "Description cannot be empty"
            );

            // Ensure name is suitable for filename
            assert!(
                !test_case.name.contains(' '),
                "Test case name should not contain spaces"
            );
            assert!(
                !test_case.name.contains('/'),
                "Test case name should not contain slashes"
            );
        }
    }

    #[test]
    fn test_case_uniqueness() {
        // Ensure all test case names are unique
        let mut names = std::collections::HashSet::new();
        for test_case in TEST_CASES {
            assert!(
                names.insert(test_case.name),
                "Duplicate test case name: {}",
                test_case.name
            );
        }
    }
}
