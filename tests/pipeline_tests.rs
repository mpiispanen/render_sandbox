use render_sandbox::{
    graphics_api::{GraphicsApi, WgpuGraphicsApi},
    pipeline::{
        ComputePipelineBuilder, GraphicsPipelineBuilder, ShaderRegistry, VertexAttribute,
        VertexLayout,
    },
    resource_manager::ResourceManager,
};

#[test]
fn test_vertex_attribute_properties() {
    // Test attribute sizes
    assert_eq!(VertexAttribute::Position3D.size(), 12); // 3 * f32
    assert_eq!(VertexAttribute::Position2D.size(), 8); // 2 * f32
    assert_eq!(VertexAttribute::Normal.size(), 12); // 3 * f32
    assert_eq!(VertexAttribute::Tangent.size(), 16); // 4 * f32
    assert_eq!(VertexAttribute::TextureCoord.size(), 8); // 2 * f32
    assert_eq!(VertexAttribute::Color.size(), 16); // 4 * f32

    // Test wgpu attribute creation
    let attr = VertexAttribute::Position3D.wgpu_attribute(0, 0);
    assert_eq!(attr.shader_location, 0);
    assert_eq!(attr.offset, 0);
    assert_eq!(attr.format, wgpu::VertexFormat::Float32x3);
}

#[test]
fn test_vertex_layout_build() {
    let layout = VertexLayout::position_normal();
    let (wgpu_layout, attributes) = layout.build();

    // Check stride calculation (3 * f32 for position + 3 * f32 for normal = 24 bytes)
    assert_eq!(wgpu_layout.array_stride, 24);
    assert_eq!(wgpu_layout.step_mode, wgpu::VertexStepMode::Vertex);
    assert_eq!(attributes.len(), 2);

    // Check attribute offsets
    assert_eq!(attributes[0].offset, 0); // Position starts at 0
    assert_eq!(attributes[1].offset, 12); // Normal starts after position (12 bytes)
}

#[test]
fn test_vertex_layout_creation() {
    // Test that different layouts can be created
    let _layout1 = VertexLayout::position_only();
    let _layout2 = VertexLayout::position_normal();
    let _layout3 = VertexLayout::position_normal_uv();
    let _layout4 = VertexLayout::position_color();

    // Test custom layout
    let _layout5 = VertexLayout::new()
        .with_attribute(VertexAttribute::Position3D)
        .with_attribute(VertexAttribute::Color);
}

#[test]
fn test_shader_registry_basic_operations() {
    let mut registry = ShaderRegistry::new();

    // Initially empty
    assert!(!registry.has_shader("test_shader"));
    assert!(registry.get_shader("test_shader").is_none());

    // Test clearing
    registry.clear();
}

#[test]
fn test_pipeline_builder_creation() {
    // Test that builders can be created and configured
    let _graphics_builder = GraphicsPipelineBuilder::new()
        .with_label("Test Pipeline".to_string())
        .with_vertex_shader("vertex_shader".to_string())
        .with_fragment_shader("fragment_shader".to_string())
        .with_vertex_layout(VertexLayout::position_color())
        .with_topology(wgpu::PrimitiveTopology::LineList)
        .with_cull_mode(None)
        .without_depth_test()
        .with_color_format(wgpu::TextureFormat::Rgba8UnormSrgb)
        .with_sample_count(4);

    let _compute_builder = ComputePipelineBuilder::new()
        .with_label("Test Compute Pipeline".to_string())
        .with_compute_shader("compute_shader".to_string());
}

#[test]
fn test_graphics_pipeline_creation_with_device() {
    // This test requires a valid device and is more of an integration test
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let device = graphics_api.device();
            let mut resource_manager = ResourceManager::new();
            let mut shader_registry = ShaderRegistry::new();

            // Create a simple shader for testing
            let shader_creation_result = shader_registry.create_shader_from_wgsl(
                device,
                &mut resource_manager,
                "test_shader".to_string(),
                r#"
                    struct VertexInput {
                        @location(0) position: vec3<f32>,
                    }

                    struct VertexOutput {
                        @builtin(position) clip_position: vec4<f32>,
                    }

                    @vertex
                    fn vs_main(model: VertexInput) -> VertexOutput {
                        var out: VertexOutput;
                        out.clip_position = vec4<f32>(model.position, 1.0);
                        return out;
                    }

                    @fragment
                    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                    }
                "#,
            );

            assert!(shader_creation_result.is_ok());
            assert!(shader_registry.has_shader("test_shader"));

            // Test pipeline creation
            let pipeline_result = GraphicsPipelineBuilder::new()
                .with_label("Test Pipeline".to_string())
                .with_vertex_shader("test_shader".to_string())
                .with_fragment_shader("test_shader".to_string())
                .with_vertex_layout(VertexLayout::position_only())
                .build(device, &resource_manager, &shader_registry);

            assert!(pipeline_result.is_ok());
        }
        Err(e) => {
            // If we can't create a graphics API (headless environment without proper GPU),
            // just skip this test
            println!("Skipping graphics pipeline creation test due to: {e}");
        }
    }
}

#[test]
fn test_pipeline_error_handling() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let device = graphics_api.device();
            let resource_manager = ResourceManager::new();
            let shader_registry = ShaderRegistry::new();

            // Test missing vertex shader
            let pipeline_result = GraphicsPipelineBuilder::new()
                .with_fragment_shader("nonexistent".to_string())
                .build(device, &resource_manager, &shader_registry);
            assert!(pipeline_result.is_err());

            // Test missing fragment shader
            let pipeline_result = GraphicsPipelineBuilder::new()
                .with_vertex_shader("nonexistent".to_string())
                .build(device, &resource_manager, &shader_registry);
            assert!(pipeline_result.is_err());

            // Test nonexistent shader
            let pipeline_result = GraphicsPipelineBuilder::new()
                .with_vertex_shader("nonexistent".to_string())
                .with_fragment_shader("also_nonexistent".to_string())
                .build(device, &resource_manager, &shader_registry);
            assert!(pipeline_result.is_err());
        }
        Err(e) => {
            println!("Skipping pipeline error handling test due to: {e}");
        }
    }
}

#[test]
fn test_default_shaders_creation() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let graphics_api_result = runtime.block_on(async { WgpuGraphicsApi::new(None).await });

    match graphics_api_result {
        Ok(graphics_api) => {
            let device = graphics_api.device();
            let mut resource_manager = ResourceManager::new();
            let mut shader_registry = ShaderRegistry::new();

            // Test creating default shaders
            let result = shader_registry.create_default_shaders(device, &mut resource_manager);
            assert!(result.is_ok());

            // Verify the shaders were created
            assert!(shader_registry.has_shader("forward_simple"));
            assert!(shader_registry.has_shader("forward_color"));
        }
        Err(e) => {
            println!("Skipping default shaders test due to: {e}");
        }
    }
}
