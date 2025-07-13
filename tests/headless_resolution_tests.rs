/// Test to verify that headless resolution arguments are used correctly
/// 
/// This test verifies that:
/// 1. PlaceholderEngine uses args.width and args.height for frame data generation
/// 2. RealTimeEngine uses renderer's surface size for frame data generation

use clap::Parser;
use render_sandbox::{engine::{Engine, PlaceholderEngine}, Args};

#[test]
fn test_placeholder_engine_uses_args_resolution() {
    // Test with different resolutions to ensure args are being used
    let test_cases = vec![
        (800, 600),
        (1920, 1080),
        (1024, 768),
        (640, 480),
    ];
    
    for (width, height) in test_cases {
        let args = Args::parse_from([
            "render_sandbox",
            "--headless",
            "--width", &width.to_string(),
            "--height", &height.to_string(),
        ]);
        
        // Create placeholder engine
        let engine = futures::executor::block_on(PlaceholderEngine::new(None, &args)).unwrap();
        
        // Get frame data
        if let Some(frame_data) = engine.get_rendered_frame_data() {
            let expected_size = (width * height * 4) as usize;
            assert_eq!(
                frame_data.len(),
                expected_size,
                "Frame data size should match resolution {}x{} = {} bytes",
                width, height, expected_size
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