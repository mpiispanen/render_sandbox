use render_sandbox::{
    graphics_api::{GraphicsApi, WgpuGraphicsApi},
    image_capture::{ImageCapture, ImageFormat},
    resource_manager::ResourceManager,
};

#[tokio::test]
async fn test_image_capture_integration() {
    // Test image capture functionality with a real GPU device
    let graphics_api_result = WgpuGraphicsApi::new(None).await;

    match graphics_api_result {
        Ok(graphics_api) => {
            let device = graphics_api.device();
            let mut resource_manager = ResourceManager::new();

            // Create image capture for a small test image
            let mut capture = ImageCapture::new(64, 64, wgpu::TextureFormat::Rgba8UnormSrgb);

            // Initialize the capture
            let init_result = capture.initialize(device, &mut resource_manager);
            assert!(init_result.is_ok());

            // Verify staging buffer was created
            assert!(capture.staging_buffer().is_some());

            // Test dimensions and format
            assert_eq!(capture.dimensions(), (64, 64));
            assert_eq!(capture.format(), wgpu::TextureFormat::Rgba8UnormSrgb);

            // Create a simple test texture to capture
            let test_texture_handle = resource_manager.create_texture(
                device,
                &wgpu::TextureDescriptor {
                    label: Some("Test Texture"),
                    size: wgpu::Extent3d {
                        width: 64,
                        height: 64,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                },
            );

            // Create a command encoder to test capture
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Test Capture Encoder"),
            });

            // Test capture (this would normally be done after rendering)
            let capture_result =
                capture.capture_texture(&mut encoder, test_texture_handle, &resource_manager);
            assert!(capture_result.is_ok());

            println!("Image capture integration test completed successfully");
        }
        Err(e) => {
            println!("Skipping image capture integration test due to: {}", e);
        }
    }
}

#[test]
fn test_image_capture_basic_functionality() {
    // Test basic functionality without requiring a GPU device
    let capture = ImageCapture::new(800, 600, wgpu::TextureFormat::Bgra8UnormSrgb);

    assert_eq!(capture.dimensions(), (800, 600));
    assert_eq!(capture.format(), wgpu::TextureFormat::Bgra8UnormSrgb);
    assert!(capture.staging_buffer().is_none());
}

#[test]
fn test_image_format_utilities() {
    // Test all supported formats
    assert_eq!(ImageFormat::from_str("png").unwrap(), ImageFormat::Png);
    assert_eq!(ImageFormat::from_str("jpg").unwrap(), ImageFormat::Jpeg);
    assert_eq!(ImageFormat::from_str("jpeg").unwrap(), ImageFormat::Jpeg);
    assert_eq!(ImageFormat::from_str("bmp").unwrap(), ImageFormat::Bmp);

    // Test case insensitive
    assert_eq!(ImageFormat::from_str("PNG").unwrap(), ImageFormat::Png);
    assert_eq!(ImageFormat::from_str("JPEG").unwrap(), ImageFormat::Jpeg);

    // Test error cases
    assert!(ImageFormat::from_str("tiff").is_err());
    assert!(ImageFormat::from_str("gif").is_err());

    // Test extensions
    assert_eq!(ImageFormat::Png.extension(), "png");
    assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
    assert_eq!(ImageFormat::Bmp.extension(), "bmp");
}
