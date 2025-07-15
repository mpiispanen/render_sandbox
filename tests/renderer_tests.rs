use clap::Parser;
/// Tests for renderer functionality
/// These tests focus on renderer configuration, initialization, and behavior
use render_sandbox::renderer::{RenderStats, RendererConfig};
use render_sandbox::{
    engine::{Engine, PlaceholderEngine},
    Args,
};

#[test]
fn test_renderer_config_default() {
    let config = RendererConfig::default();

    assert_eq!(config.msaa_samples, 1);
    assert!(config.enable_depth_testing);
    assert!(config.enable_culling);
    assert_eq!(config.clear_color, [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn test_renderer_config_custom() {
    let config = RendererConfig {
        enable_depth_testing: false,
        enable_culling: false,
        clear_color: [1.0, 0.0, 0.0, 1.0],
        msaa_samples: 4,
    };

    assert_eq!(config.msaa_samples, 4);
    assert!(!config.enable_depth_testing);
    assert!(!config.enable_culling);
    assert_eq!(config.clear_color, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn test_render_stats_default() {
    let stats = RenderStats::default();

    assert_eq!(stats.frame_count, 0);
    assert_eq!(stats.draw_calls, 0);
    assert_eq!(stats.vertices_rendered, 0);
    assert_eq!(stats.triangles_rendered, 0);
    assert_eq!(stats.render_passes, 0);
}

#[test]
fn test_render_stats_clone() {
    let stats = RenderStats {
        frame_count: 100,
        draw_calls: 50,
        ..Default::default()
    };

    let cloned_stats = stats.clone();
    assert_eq!(cloned_stats.frame_count, 100);
    assert_eq!(cloned_stats.draw_calls, 50);
}

#[test]
fn test_msaa_samples_validation_logic() {
    // Test the MSAA validation logic used by renderer
    // This mirrors the logic from the graphics API but tests it independently

    fn validate_msaa_samples(requested: u32) -> u32 {
        // WebGPU spec guaranteed values for maximum compatibility
        let valid_samples = [1, 4];
        valid_samples
            .iter()
            .rev()
            .find(|&&samples| samples <= requested)
            .copied()
            .unwrap_or(1)
    }

    // Test that renderer uses proper validation
    assert_eq!(validate_msaa_samples(1), 1);
    assert_eq!(validate_msaa_samples(2), 1);
    assert_eq!(validate_msaa_samples(3), 1);
    assert_eq!(validate_msaa_samples(4), 4);
    assert_eq!(validate_msaa_samples(8), 4);
    assert_eq!(validate_msaa_samples(16), 4);
}

#[test]
fn test_renderer_configuration_validation() {
    // Test various renderer configurations for validity

    // Test valid configurations
    let valid_configs = vec![
        RendererConfig {
            msaa_samples: 1,
            enable_depth_testing: true,
            enable_culling: true,
            clear_color: [0.0, 0.0, 0.0, 1.0],
        },
        RendererConfig {
            msaa_samples: 4,
            enable_depth_testing: false,
            enable_culling: false,
            clear_color: [1.0, 1.0, 1.0, 1.0],
        },
    ];

    for config in valid_configs {
        // Basic validation - MSAA samples should be valid WebGPU values
        assert!(config.msaa_samples == 1 || config.msaa_samples == 4);

        // Clear color components should be in valid range [0.0, 1.0]
        for component in config.clear_color {
            assert!((0.0..=1.0).contains(&component));
        }
    }
}

// Headless resolution tests - moved from headless_resolution_tests.rs
// These tests verify that CLI resolution arguments are used correctly by the renderer

#[test]
fn test_placeholder_engine_uses_args_resolution() {
    // Test with different resolutions to ensure args are being used
    let test_cases = vec![(800, 600), (1920, 1080), (1024, 768), (640, 480)];

    for (width, height) in test_cases {
        let args = Args::parse_from([
            "render_sandbox",
            "--headless",
            "--width",
            &width.to_string(),
            "--height",
            &height.to_string(),
        ]);

        // Create placeholder engine
        let engine = futures::executor::block_on(PlaceholderEngine::new(None, &args)).unwrap();

        // Get frame data
        if let Some(frame_data) = engine.get_rendered_frame_data() {
            let expected_size = (width * height * 4) as usize;
            assert_eq!(
                frame_data.len(),
                expected_size,
                "Frame data size should match resolution {width}x{height} = {expected_size} bytes"
            );
        } else {
            panic!("Expected frame data for headless mode");
        }
    }
}

#[test]
fn test_frame_data_size_calculation() {
    // Test the frame data size calculation logic
    fn calculate_frame_size(width: u32, height: u32) -> usize {
        (width * height * 4) as usize // RGBA format
    }

    assert_eq!(calculate_frame_size(800, 600), 1_920_000);
    assert_eq!(calculate_frame_size(1920, 1080), 8_294_400);
    assert_eq!(calculate_frame_size(1024, 768), 3_145_728);
    assert_eq!(calculate_frame_size(640, 480), 1_228_800);
}
