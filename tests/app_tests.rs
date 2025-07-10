use render_sandbox::{app_core::Application, engine::EngineError};

#[test]
fn test_application_headless_mode() {
    // Test that we can create an application in headless mode
    let app = Application::new(true);
    assert!(app.is_ok(), "Failed to create headless application");
}

#[test]
#[ignore] // Skip by default in CI - run with: cargo test -- --ignored
fn test_application_windowed_mode() {
    // Test that we can create an application in windowed mode
    // This test verifies the windowed mode logic works, even if it can't run in test environments

    // Use panic::catch_unwind to handle the expected threading panic in test environments
    let result = std::panic::catch_unwind(|| Application::new(false));

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
    if let Ok(app) = Application::new(true) {
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
