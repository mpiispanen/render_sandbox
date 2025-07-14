use crate::pipeline::{GraphicsPipelineBuilder, ShaderRegistry, VertexLayout};
use crate::render_graph::{
    PassId, RenderGraphError, RenderPass, ResourceDeclaration, ResourceId, ResourceUsage,
};
use crate::resource_manager::ResourceManager;

/// Demonstrates how to properly use ShaderRegistry and GraphicsPipelineBuilder.
/// This function shows the intended usage pattern that would be used once
/// the RenderPass trait interface supports mutable ResourceManager.
#[allow(dead_code)]
fn example_pipeline_creation_with_registry(
    device: &wgpu::Device,
    resource_manager: &mut ResourceManager,
    surface_format: wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline, RenderGraphError> {
    // Create and configure shader registry
    let mut shader_registry = ShaderRegistry::new();

    // Create default shaders (this demonstrates the intended pattern)
    shader_registry
        .create_default_shaders(device, resource_manager)
        .map_err(|e| RenderGraphError::ExecutionFailed(format!("Failed to create shaders: {e}")))?;

    // Use GraphicsPipelineBuilder with the registry
    let pipeline = GraphicsPipelineBuilder::new()
        .with_label("Forward Render Pipeline".to_string())
        .with_vertex_shader("forward_simple".to_string())
        .with_fragment_shader("forward_simple".to_string())
        .with_vertex_layout(VertexLayout::position_only())
        .with_color_format(surface_format)
        .build(device, resource_manager, &shader_registry)
        .map_err(|e| {
            RenderGraphError::ExecutionFailed(format!("Failed to create pipeline: {e}"))
        })?;

    Ok(pipeline)
}

/// Helper function to create a forward render pipeline using the pipeline abstraction.
/// This demonstrates the intended usage pattern of ShaderRegistry and GraphicsPipelineBuilder.
/// This function is kept for documentation purposes but is no longer used since we now
/// properly use the abstraction in the initialize method with mutable ResourceManager.
#[allow(dead_code)]
fn create_forward_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline, RenderGraphError> {
    // This is how we WOULD use ShaderRegistry if we had a mutable ResourceManager:
    //
    // let mut shader_registry = ShaderRegistry::new();
    // shader_registry.create_shader_from_wgsl(device, resource_manager, "forward_procedural", source)?;
    //
    // let pipeline = GraphicsPipelineBuilder::new()
    //     .with_label("Forward Render Pipeline".to_string())
    //     .with_vertex_shader("forward_procedural".to_string())
    //     .with_fragment_shader("forward_procedural".to_string())
    //     .with_vertex_layout(VertexLayout::new())
    //     .with_color_format(surface_format)
    //     .build(device, resource_manager, &shader_registry)?;

    // For now, we create the shader directly but use the pipeline builder patterns
    let forward_shader_source = r#"
        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
        }

        @vertex
        fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
            var out: VertexOutput;
            
            // Generate triangle vertices procedurally
            var positions = array<vec2<f32>, 3>(
                vec2<f32>(0.0, 0.5),   // Top
                vec2<f32>(-0.5, -0.5), // Bottom left  
                vec2<f32>(0.5, -0.5)   // Bottom right
            );
            
            out.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(0.8, 0.2, 0.3, 1.0);
        }
    "#;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Forward Procedural Shader"),
        source: wgpu::ShaderSource::Wgsl(forward_shader_source.into()),
    });

    // Use the VertexLayout abstraction (this part works with current interface)
    let vertex_layout = VertexLayout::new(); // Empty layout for procedural generation
    let (_vertex_buffer_layout, _attributes) = vertex_layout.build();

    // Use GraphicsPipelineBuilder pattern to get proper defaults
    let _pipeline_builder = GraphicsPipelineBuilder::new().with_color_format(surface_format);
    // Note: Can't use .build() due to interface constraints, but we use the same default values

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Forward Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Create depth stencil state using builder defaults
    let depth_stencil = Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float, // Default from builder
        depth_write_enabled: true,                 // Default from builder
        depth_compare: wgpu::CompareFunction::Less, // Default from builder
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });

    // Create color targets using builder settings
    let color_targets = [Some(wgpu::ColorTargetState {
        format: surface_format,
        blend: Some(wgpu::BlendState::REPLACE), // Default from builder
        write_mask: wgpu::ColorWrites::ALL,
    })];

    // Create the render pipeline using the same patterns as GraphicsPipelineBuilder
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Forward Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[], // No vertex buffers needed since we generate vertices procedurally
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &color_targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // Default from builder
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // Default from builder
            cull_mode: Some(wgpu::Face::Back), // Default from builder
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil,
        multisample: wgpu::MultisampleState {
            count: 1, // Default from builder
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    Ok(render_pipeline)
}

/// A forward renderer pass that renders meshes from the scene
pub struct ForwardRenderPass {
    id: PassId,
    resources: Vec<ResourceDeclaration>,
    clear_color: [f64; 4],
    resolution: (u32, u32),
    surface_format: wgpu::TextureFormat,
    render_pipeline: Option<wgpu::RenderPipeline>,
    initialized: bool,
}

impl ForwardRenderPass {
    pub fn new(name: &str) -> Self {
        Self {
            id: PassId::new(name),
            resources: vec![],
            clear_color: [0.0, 0.0, 0.0, 1.0],
            resolution: (800, 600), // Default resolution
            surface_format: wgpu::TextureFormat::Bgra8UnormSrgb, // Default format, should be overridden
            render_pipeline: None,
            initialized: false,
        }
    }

    pub fn with_resource(mut self, resource_id: &str, usage: ResourceUsage) -> Self {
        self.resources.push(ResourceDeclaration {
            id: ResourceId::new(resource_id),
            usage,
        });
        self
    }

    pub fn with_clear_color(mut self, color: [f64; 4]) -> Self {
        self.clear_color = color;
        self
    }

    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = (width, height);
        self
    }

    pub fn with_surface_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.surface_format = format;
        self
    }
}

impl RenderPass for ForwardRenderPass {
    fn id(&self) -> PassId {
        self.id.clone()
    }

    fn resources(&self) -> Vec<ResourceDeclaration> {
        self.resources.clone()
    }

    fn initialize(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
    ) -> Result<(), RenderGraphError> {
        if self.initialized {
            return Ok(());
        }

        log::debug!("Initializing forward render pass: {}", self.id);

        // Now we can properly use the pipeline abstraction with mutable ResourceManager!
        let render_pipeline =
            example_pipeline_creation_with_registry(device, resource_manager, self.surface_format)?;

        self.render_pipeline = Some(render_pipeline);
        self.initialized = true;

        log::debug!("Forward render pass initialized successfully with full pipeline abstraction");
        Ok(())
    }

    fn execute(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        resource_manager: &ResourceManager,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), RenderGraphError> {
        log::debug!(
            "Executing forward render pass with pipeline abstraction: {}",
            self.id
        );

        // Get the actual render targets from the resource manager
        let back_buffer_handle: crate::resource_manager::Handle<wgpu::Texture> = resource_manager
            .get_named_resource("BackBuffer")
            .ok_or_else(|| {
                RenderGraphError::ResourceNotFound(crate::render_graph::ResourceId::new(
                    "BackBuffer",
                ))
            })?;

        let back_buffer = resource_manager
            .get_texture(back_buffer_handle)
            .map_err(|e| {
                RenderGraphError::ExecutionFailed(format!("Failed to get BackBuffer: {e}"))
            })?;

        let depth_buffer_handle: crate::resource_manager::Handle<wgpu::Texture> = resource_manager
            .get_named_resource("DepthBuffer")
            .ok_or_else(|| {
                RenderGraphError::ResourceNotFound(crate::render_graph::ResourceId::new(
                    "DepthBuffer",
                ))
            })?;

        let depth_buffer = resource_manager
            .get_texture(depth_buffer_handle)
            .map_err(|e| {
                RenderGraphError::ExecutionFailed(format!("Failed to get DepthBuffer: {e}"))
            })?;

        // Create texture views for rendering
        let color_view = back_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the render pass with proper render targets
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.clear_color[0],
                        g: self.clear_color[1],
                        b: self.clear_color[2],
                        a: self.clear_color[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If we have a render pipeline, use it to render
        if let Some(ref pipeline) = self.render_pipeline {
            render_pass.set_pipeline(pipeline);

            // For now, render a hardcoded triangle to demonstrate the new pipeline system
            // In the future, this should iterate through scene meshes
            render_pass.draw(0..3, 0..1); // Draw a single triangle

            // TODO: In the enhanced version, this would look like:
            // for mesh_node in visible_mesh_nodes {
            //     let vertex_buffer = resource_manager.get_buffer(mesh_node.mesh.vertex_buffer)?;
            //     render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            //
            //     if let Some(index_buffer_handle) = mesh_node.mesh.index_buffer {
            //         let index_buffer = resource_manager.get_buffer(index_buffer_handle)?;
            //         render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            //         render_pass.draw_indexed(0..mesh_node.mesh.index_count.unwrap(), 0, 0..1);
            //     } else {
            //         render_pass.draw(0..mesh_node.mesh.vertex_count, 0..1);
            //     }
            // }
        }

        log::debug!(
            "Forward render pass executed with pipeline abstraction and proper render targets"
        );

        Ok(())
    }
}
