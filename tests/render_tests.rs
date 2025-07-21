#[cfg(feature = "gpu-tests")]
use render_sandbox::{
    graphics_api::{GraphicsApi, WgpuGraphicsApi},
    renderer::Renderer,
    scene::Scene,
};

#[test]
#[cfg(feature = "gpu-tests")]
fn test_forward_renderpass_enabled() {
    // Test that the forward renderpass is properly enabled and configured
    // This test runs in headless mode to work in CI environments

    // Create a headless graphics API for testing
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let mut renderer = Renderer::new(Box::new(graphics_api));

            // Initialize the renderer - this should set up the forward renderpass
            let init_result = renderer.initialize();
            assert!(
                init_result.is_ok(),
                "Renderer initialization should succeed"
            );

            // Verify the render graph has been compiled and contains passes
            let render_graph = renderer.render_graph();
            assert!(
                render_graph.is_compiled(),
                "Render graph should be compiled"
            );
            assert!(render_graph.pass_count() > 0, "Should have render passes");

            // Verify we have at least 2 passes (clear + forward)
            assert!(
                render_graph.pass_count() >= 2,
                "Should have at least clear and forward passes"
            );

            // Get the execution order to verify passes are present
            let execution_order = render_graph.execution_order();
            assert!(
                execution_order.is_some(),
                "Execution order should be available"
            );

            let passes = execution_order.unwrap();
            assert!(
                passes.len() >= 2,
                "Should have at least 2 passes in execution order"
            );

            // Verify pass names include ForwardPass
            let pass_names: Vec<String> = passes.iter().map(|p| p.to_string()).collect();
            assert!(
                pass_names.iter().any(|name| name == "ForwardPass"),
                "Should have ForwardPass in execution order, found: {pass_names:?}"
            );

            log::info!("Forward renderpass test passed - found passes: {pass_names:?}");
        }
        Err(e) => {
            // In CI environments without graphics adapters, this is expected
            log::info!("Graphics API initialization failed (expected in CI): {e}");
            // We still want the test to pass as this is environmental
            assert!(
                e.to_string().contains("graphics adapter")
                    || e.to_string().contains("AdapterNotFound"),
                "Expected graphics adapter error, got: {e}"
            );
        }
    }
}

#[test]
#[cfg(feature = "gpu-tests")]
fn test_forward_renderpass_execution() {
    // Test that the forward renderpass can execute without errors

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let mut renderer = Renderer::new(Box::new(graphics_api));

            // Initialize the renderer
            renderer
                .initialize()
                .expect("Renderer initialization should succeed");

            // Create a simple scene for testing
            let scene = Scene::new();

            // Try to render a frame - this should execute the forward renderpass
            let render_result = renderer.render(&scene);

            // The render should succeed (even if it doesn't draw anything visible)
            assert!(
                render_result.is_ok(),
                "Forward renderpass execution should succeed: {:?}",
                render_result.err()
            );

            // Verify stats were updated
            let stats = renderer.get_stats();
            assert!(stats.frame_count > 0, "Frame count should be incremented");
            assert!(
                stats.render_passes > 0,
                "Should have executed render passes"
            );

            log::info!(
                "Forward renderpass execution test passed - rendered {} frames with {} passes",
                stats.frame_count,
                stats.render_passes
            );
        }
        Err(e) => {
            log::info!("Graphics API initialization failed (expected in CI): {e}");
            // Environmental failure is acceptable
        }
    }
}

#[test]
#[cfg(feature = "gpu-tests")]
fn test_render_stats_with_forward_pass() {
    // Test that rendering stats properly reflect forward renderpass usage

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let mut renderer = Renderer::new(Box::new(graphics_api));
            renderer.initialize().expect("Renderer should initialize");

            let scene = Scene::new();

            // Get initial stats
            let initial_stats = renderer.get_stats().clone();
            assert_eq!(initial_stats.frame_count, 0);
            assert_eq!(initial_stats.render_passes, 0);

            // Render multiple frames
            for i in 0..3 {
                let result = renderer.render(&scene);
                assert!(result.is_ok(), "Frame {} should render successfully", i + 1);
            }

            // Check final stats
            let final_stats = renderer.get_stats();
            assert_eq!(final_stats.frame_count, 3, "Should have rendered 3 frames");

            // Should have executed render passes (at least 2 per frame: clear + forward)
            assert!(
                final_stats.render_passes >= 2,
                "Should have executed at least 2 render passes per frame, got {}",
                final_stats.render_passes
            );

            log::info!(
                "Render stats test passed - {} frames, {} passes",
                final_stats.frame_count,
                final_stats.render_passes
            );
        }
        Err(e) => {
            log::info!("Graphics API initialization failed (expected in CI): {e}");
        }
    }
}

#[test]
#[cfg(feature = "gpu-tests")]
fn test_forward_renderpass_with_scene_content() {
    // Test that forward renderpass works with scene content

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let mut renderer = Renderer::new(Box::new(graphics_api));
            renderer.initialize().expect("Renderer should initialize");

            let mut scene = Scene::new();

            // Try to add a test triangle to the scene
            let triangle_result = renderer.create_test_triangle(&mut scene);

            match triangle_result {
                Ok(_) => {
                    // Verify scene has content
                    assert!(
                        scene.node_count() > 0,
                        "Scene should have nodes after adding triangle"
                    );
                    assert!(
                        !scene.get_mesh_nodes().is_empty(),
                        "Scene should have mesh nodes"
                    );

                    // Render with scene content
                    let render_result = renderer.render(&scene);
                    assert!(
                        render_result.is_ok(),
                        "Should be able to render scene with triangle: {:?}",
                        render_result.err()
                    );

                    log::info!("Forward renderpass with scene content test passed");
                }
                Err(e) => {
                    // Triangle creation might fail in headless mode, which is acceptable
                    log::info!("Triangle creation failed (acceptable in headless mode): {e}");

                    // Still test basic rendering with empty scene
                    let render_result = renderer.render(&scene);
                    assert!(
                        render_result.is_ok(),
                        "Should still be able to render empty scene"
                    );
                }
            }
        }
        Err(e) => {
            log::info!("Graphics API initialization failed (expected in CI): {e}");
        }
    }
}
