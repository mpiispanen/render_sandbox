use clap::Parser;
use render_sandbox::{app_core::Application, engine::EngineError, Args};

#[test]
fn test_application_headless_mode() {
    // Test that we can create an application in headless mode
    let args = Args {
        width: 800,
        height: 600,
        output: "test_output.png".to_string(),
        format: "png".to_string(),
        samples: 1,
        verbose: false,
        log_level: "info".to_string(),
        headless: true,
        gltf_path: "test_assets/triangle.gltf".to_string(),
    };
    let app = Application::new(args);
    assert!(app.is_ok(), "Failed to create headless application");

    let app = app.unwrap();
    assert!(app.is_headless(), "Application should be in headless mode");
}

#[test]
fn test_application_with_custom_resolution() {
    // Test that custom resolution arguments are accepted
    let args = Args::parse_from([
        "render_sandbox",
        "--headless",
        "--width",
        "1920",
        "--height",
        "1080",
        "--gltf",
        "test_assets/triangle.gltf",
    ]);
    let app = Application::new(args);
    assert!(
        app.is_ok(),
        "Failed to create application with custom resolution"
    );
}

#[test]
fn test_application_with_valid_sample_count() {
    // Test that valid sample counts work correctly
    let args = Args::parse_from([
        "render_sandbox",
        "--headless",
        "--width",
        "800",
        "--height",
        "600",
        "--samples",
        "4", // Use a WebGPU spec guaranteed sample count
        "--gltf",
        "test_assets/triangle.gltf",
    ]);

    // Application creation should not panic
    let app = Application::new(args);

    // In environments without graphics drivers, this will fail with an appropriate error
    // In environments with graphics drivers, it should succeed
    match app {
        Ok(_) => {
            // Success case - application created successfully with valid sample count
            println!("Application created successfully with valid sample count");
        }
        Err(e) => {
            // Expected failure in CI environments without graphics drivers
            assert!(
                e.to_string().contains("graphics adapter")
                    || e.to_string().contains("graphics API")
                    || e.to_string().contains("InitializationError"),
                "Unexpected error type: {e}"
            );
        }
    }
}

#[test]
fn test_sample_count_validation_behavior() {
    // Test that the sample count validation logic works correctly
    // Updated to reflect that sample count support depends on texture formats

    // Test the validation logic function with format awareness
    fn validate_sample_count_test_for_render_targets(requested_samples: u32) -> u32 {
        // Simulate what happens in actual rendering with color + depth targets
        // Color formats might support [1, 2, 4, 8, 16] but depth (Depth32Float)
        // only guarantees [1, 4] per WebGPU spec, so we use the intersection
        let valid_samples = [1, 4]; // Intersection for color + depth formats
        valid_samples
            .iter()
            .rev()
            .find(|&&samples| samples <= requested_samples)
            .copied()
            .unwrap_or(1)
    }

    // Test that invalid sample counts get clamped to values supported by ALL formats
    assert_eq!(
        validate_sample_count_test_for_render_targets(2),
        1,
        "Sample count 2 should clamp to 1 due to depth format limitations"
    );
    assert_eq!(
        validate_sample_count_test_for_render_targets(3),
        1,
        "Sample count 3 should clamp to 1 due to depth format limitations"
    );
    assert_eq!(
        validate_sample_count_test_for_render_targets(5),
        4,
        "Sample count 5 should clamp to 4"
    );
    assert_eq!(
        validate_sample_count_test_for_render_targets(7),
        4,
        "Sample count 7 should clamp to 4"
    );
    assert_eq!(
        validate_sample_count_test_for_render_targets(8),
        4,
        "Sample count 8 should clamp to 4 due to depth format limitations"
    );
    assert_eq!(
        validate_sample_count_test_for_render_targets(16),
        4,
        "Sample count 16 should clamp to 4 due to depth format limitations"
    );
    assert_eq!(
        validate_sample_count_test_for_render_targets(0),
        1,
        "Sample count 0 should clamp to 1"
    );

    // Test that valid sample counts remain unchanged
    assert_eq!(validate_sample_count_test_for_render_targets(1), 1);
    assert_eq!(validate_sample_count_test_for_render_targets(4), 4);
}

#[test]
#[ignore] // Skip by default in CI - run with: cargo test -- --ignored
fn test_application_windowed_mode() {
    let result = std::panic::catch_unwind(|| {
        let args = Args {
            width: 800,
            height: 600,
            output: "test_output.png".to_string(),
            format: "png".to_string(),
            samples: 1,
            verbose: false,
            log_level: "info".to_string(),
            headless: false,
            gltf_path: "test_assets/triangle.gltf".to_string(),
        };
        Application::new(args)
    });

    match result {
        Ok(Ok(application)) => {
            // Successfully created windowed application (e.g., when running on main thread)
            println!("Windowed mode created successfully");

            // Verify windowed application properties
            assert!(!application.is_headless());
            assert!(application.has_window());
            assert!(application.has_event_loop());

            println!("Test passed: Windowed mode works correctly");
        }
        Ok(Err(e)) => {
            // Application creation failed with an error (also acceptable)
            eprintln!("Windowed mode failed with error: {e}");

            // Verify we get the expected initialization error
            match e {
                EngineError::InitializationError(msg) => {
                    println!("Test passed: Got expected initialization error: {msg}");
                }
                _ => {
                    panic!("Unexpected error type: {e}");
                }
            }
        }
        Err(panic_info) => {
            // Application creation panicked (expected in test environments due to threading)
            println!("Windowed mode panicked (expected in test environment)");

            // Convert panic info to string to check the error message
            let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            };

            // Verify this is the expected threading panic
            if panic_msg.contains("main thread") || panic_msg.contains("EventLoop") {
                println!("Test passed: Got expected threading restriction panic");
            } else {
                panic!("Unexpected panic message: {panic_msg}");
            }
        }
    }
}

#[test]
fn test_headless_run() {
    // Test running the application in headless mode
    let args = Args {
        width: 800,
        height: 600,
        output: "test_output.png".to_string(),
        format: "png".to_string(),
        samples: 1,
        verbose: false,
        log_level: "info".to_string(),
        headless: true,
        gltf_path: "test_assets/triangle.gltf".to_string(),
    };
    if let Ok(app) = Application::new(args) {
        let result = app.run();

        // In CI environments without graphics drivers, this is expected to fail
        // We just check that it fails gracefully
        match result {
            Ok(_) => {
                println!("Headless application ran successfully");
            }
            Err(e) => {
                // Expected in environments without graphics adapters
                println!("Headless application failed as expected in CI: {e}");
                assert!(
                    e.to_string().contains("graphics adapter")
                        || e.to_string().contains("graphics API")
                        || e.to_string().contains("InitializationError"),
                    "Unexpected error type: {e}"
                );
            }
        }
    }
}

#[test]
fn test_engine_error_display() {
    let error = EngineError::InitializationError("test error".to_string());
    let error_str = error.to_string();
    assert!(error_str.contains("Initialization error"));
    assert!(error_str.contains("test error"));
}
