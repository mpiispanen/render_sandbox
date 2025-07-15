//! Graphics and compute pipeline abstractions with builder pattern and sensible defaults
//!
//! This module provides high-level abstractions for creating graphics and compute pipelines
//! with reasonable defaults while allowing fine-grained control when needed.

use crate::resource_manager::{Handle, HandleId, ResourceManager};
use std::collections::HashMap;

/// Errors that can occur during pipeline operations
#[derive(Debug)]
pub enum PipelineError {
    ShaderNotFound(String),
    InvalidVertexLayout,
    PipelineCreationFailed(String),
    ResourceError(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::ShaderNotFound(name) => write!(f, "Shader not found: {name}"),
            PipelineError::InvalidVertexLayout => write!(f, "Invalid vertex layout"),
            PipelineError::PipelineCreationFailed(msg) => {
                write!(f, "Pipeline creation failed: {msg}")
            }
            PipelineError::ResourceError(msg) => write!(f, "Resource error: {msg}"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Standard vertex attribute definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexAttribute {
    Position3D,
    Position2D,
    Normal,
    Tangent,
    TextureCoord,
    Color,
}

impl VertexAttribute {
    /// Get the wgpu vertex attribute descriptor for this attribute
    pub fn wgpu_attribute(
        &self,
        location: u32,
        offset: wgpu::BufferAddress,
    ) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            offset,
            shader_location: location,
            format: match self {
                VertexAttribute::Position3D => wgpu::VertexFormat::Float32x3,
                VertexAttribute::Position2D => wgpu::VertexFormat::Float32x2,
                VertexAttribute::Normal => wgpu::VertexFormat::Float32x3,
                VertexAttribute::Tangent => wgpu::VertexFormat::Float32x4,
                VertexAttribute::TextureCoord => wgpu::VertexFormat::Float32x2,
                VertexAttribute::Color => wgpu::VertexFormat::Float32x4,
            },
        }
    }

    /// Get the size in bytes of this attribute
    pub fn size(&self) -> u64 {
        match self {
            VertexAttribute::Position3D => 12,  // 3 * f32
            VertexAttribute::Position2D => 8,   // 2 * f32
            VertexAttribute::Normal => 12,      // 3 * f32
            VertexAttribute::Tangent => 16,     // 4 * f32
            VertexAttribute::TextureCoord => 8, // 2 * f32
            VertexAttribute::Color => 16,       // 4 * f32
        }
    }
}

/// Vertex layout builder for creating vertex buffer layouts
#[derive(Debug, Clone)]
pub struct VertexLayout {
    attributes: Vec<VertexAttribute>,
    stride: wgpu::BufferAddress,
}

impl VertexLayout {
    /// Create a new empty vertex layout
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
            stride: 0,
        }
    }

    /// Add an attribute to the layout
    pub fn with_attribute(mut self, attribute: VertexAttribute) -> Self {
        self.stride += attribute.size();
        self.attributes.push(attribute);
        self
    }

    /// Build the wgpu vertex buffer layout
    /// Returns owned attributes to avoid lifetime issues
    pub fn build(
        &self,
    ) -> (
        wgpu::VertexBufferLayout<'static>,
        Vec<wgpu::VertexAttribute>,
    ) {
        let mut attributes = Vec::new();
        let mut offset = 0;

        for (location, &attr) in self.attributes.iter().enumerate() {
            attributes.push(attr.wgpu_attribute(location as u32, offset));
            offset += attr.size();
        }

        // Return both the layout and the attributes vector
        // The caller is responsible for keeping the attributes alive
        let layout = wgpu::VertexBufferLayout {
            array_stride: self.stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[], // Will be set by the caller
        };

        (layout, attributes)
    }

    /// Get common vertex layouts
    pub fn position_only() -> Self {
        Self::new().with_attribute(VertexAttribute::Position3D)
    }

    pub fn position_normal() -> Self {
        Self::new()
            .with_attribute(VertexAttribute::Position3D)
            .with_attribute(VertexAttribute::Normal)
    }

    pub fn position_normal_uv() -> Self {
        Self::new()
            .with_attribute(VertexAttribute::Position3D)
            .with_attribute(VertexAttribute::Normal)
            .with_attribute(VertexAttribute::TextureCoord)
    }

    pub fn position_color() -> Self {
        Self::new()
            .with_attribute(VertexAttribute::Position3D)
            .with_attribute(VertexAttribute::Color)
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::position_only()
    }
}

/// Shader registry for managing named shaders
pub struct ShaderRegistry {
    shaders: HashMap<String, HandleId>,
}

impl ShaderRegistry {
    /// Create a new shader registry
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    /// Register a shader with a name
    pub fn register_shader(&mut self, name: String, handle: Handle<wgpu::ShaderModule>) {
        log::debug!("Registering shader: {name}");
        self.shaders.insert(name, handle.id());
    }

    /// Get a shader by name
    pub fn get_shader(&self, name: &str) -> Option<Handle<wgpu::ShaderModule>> {
        self.shaders.get(name).map(|&id| Handle::from_id(id))
    }

    /// Create and register a shader from WGSL source
    pub fn create_shader_from_wgsl(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
        name: String,
        source: &str,
    ) -> Result<(), PipelineError> {
        let handle = resource_manager.create_shader(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some(&name),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            },
        );

        self.register_shader(name, handle);
        Ok(())
    }

    /// Create some common default shaders
    pub fn create_default_shaders(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
    ) -> Result<(), PipelineError> {
        // Simple forward shading shader
        let forward_shader_source = r#"
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
                return vec4<f32>(0.8, 0.2, 0.3, 1.0);
            }
        "#;

        self.create_shader_from_wgsl(
            device,
            resource_manager,
            "forward_simple".to_string(),
            forward_shader_source,
        )?;

        // Position + color shader
        let color_shader_source = r#"
            struct VertexInput {
                @location(0) position: vec3<f32>,
                @location(1) color: vec4<f32>,
            }

            struct VertexOutput {
                @builtin(position) clip_position: vec4<f32>,
                @location(0) color: vec4<f32>,
            }

            @vertex
            fn vs_main(model: VertexInput) -> VertexOutput {
                var out: VertexOutput;
                out.clip_position = vec4<f32>(model.position, 1.0);
                out.color = model.color;
                return out;
            }

            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return in.color;
            }
        "#;

        self.create_shader_from_wgsl(
            device,
            resource_manager,
            "forward_color".to_string(),
            color_shader_source,
        )?;

        Ok(())
    }

    /// Check if a shader exists
    pub fn has_shader(&self, name: &str) -> bool {
        self.shaders.contains_key(name)
    }

    /// Remove a shader from the registry
    pub fn remove_shader(&mut self, name: &str) -> Option<Handle<wgpu::ShaderModule>> {
        self.shaders.remove(name).map(Handle::from_id)
    }

    /// Clear all shaders
    pub fn clear(&mut self) {
        self.shaders.clear();
    }
}

impl Default for ShaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Graphics pipeline builder with sensible defaults
pub struct GraphicsPipelineBuilder {
    label: Option<String>,
    vertex_shader: Option<String>, // Use shader name instead of handle
    fragment_shader: Option<String>, // Use shader name instead of handle
    vertex_layout: VertexLayout,
    topology: wgpu::PrimitiveTopology,
    cull_mode: Option<wgpu::Face>,
    front_face: wgpu::FrontFace,
    depth_test: bool,
    depth_write: bool,
    depth_compare: wgpu::CompareFunction,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    blend_state: Option<wgpu::BlendState>,
    sample_count: u32,
}

impl GraphicsPipelineBuilder {
    /// Create a new graphics pipeline builder with sensible defaults
    pub fn new() -> Self {
        Self {
            label: None,
            vertex_shader: None,
            fragment_shader: None,
            vertex_layout: VertexLayout::default(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            front_face: wgpu::FrontFace::Ccw,
            depth_test: true,
            depth_write: true,
            depth_compare: wgpu::CompareFunction::Less,
            color_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            depth_format: Some(wgpu::TextureFormat::Depth32Float),
            blend_state: Some(wgpu::BlendState::REPLACE),
            sample_count: 1,
        }
    }

    /// Set the pipeline label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the vertex shader by name
    pub fn with_vertex_shader(mut self, shader_name: String) -> Self {
        self.vertex_shader = Some(shader_name);
        self
    }

    /// Set the fragment shader by name
    pub fn with_fragment_shader(mut self, shader_name: String) -> Self {
        self.fragment_shader = Some(shader_name);
        self
    }

    /// Set the vertex layout
    pub fn with_vertex_layout(mut self, layout: VertexLayout) -> Self {
        self.vertex_layout = layout;
        self
    }

    /// Set the primitive topology
    pub fn with_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Set the cull mode
    pub fn with_cull_mode(mut self, cull_mode: Option<wgpu::Face>) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    /// Disable depth testing
    pub fn without_depth_test(mut self) -> Self {
        self.depth_test = false;
        self.depth_write = false;
        self.depth_format = None;
        self
    }

    /// Set the color format
    pub fn with_color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.color_format = format;
        self
    }

    /// Set the sample count for MSAA
    pub fn with_sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    /// Build the graphics pipeline
    pub fn build(
        self,
        device: &wgpu::Device,
        resource_manager: &ResourceManager,
        shader_registry: &ShaderRegistry,
    ) -> Result<wgpu::RenderPipeline, PipelineError> {
        let vertex_shader_name = self
            .vertex_shader
            .ok_or_else(|| PipelineError::ShaderNotFound("vertex shader".to_string()))?;
        let fragment_shader_name = self
            .fragment_shader
            .ok_or_else(|| PipelineError::ShaderNotFound("fragment shader".to_string()))?;

        // Get shader handles from registry
        let vertex_shader = shader_registry
            .get_shader(&vertex_shader_name)
            .ok_or(PipelineError::ShaderNotFound(vertex_shader_name))?;
        let fragment_shader = shader_registry
            .get_shader(&fragment_shader_name)
            .ok_or(PipelineError::ShaderNotFound(fragment_shader_name))?;

        // Get shader modules - use references to avoid moving
        let vs_module = resource_manager
            .get_shader(vertex_shader)
            .map_err(|e| PipelineError::ResourceError(e.to_string()))?;
        let fs_module = resource_manager
            .get_shader(fragment_shader)
            .map_err(|e| PipelineError::ResourceError(e.to_string()))?;

        let (mut vertex_buffer_layout, attributes) = self.vertex_layout.build();
        vertex_buffer_layout.attributes = &attributes;

        // For procedural shaders that don't need vertex buffers, use empty slice
        let vertex_buffers: &[wgpu::VertexBufferLayout] = if attributes.is_empty() {
            &[] // No vertex buffers needed for procedural shaders
        } else {
            &[vertex_buffer_layout] // Use the vertex buffer layout for regular shaders
        };

        // Create pipeline layout (empty for now, can be extended later)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self
                .label
                .as_ref()
                .map(|l| format!("{l} Layout"))
                .as_deref(),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // Create depth stencil state if depth testing is enabled
        let depth_stencil = if self.depth_test {
            Some(wgpu::DepthStencilState {
                format: self
                    .depth_format
                    .unwrap_or(wgpu::TextureFormat::Depth32Float),
                depth_write_enabled: self.depth_write,
                depth_compare: self.depth_compare,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        } else {
            None
        };

        // Create color targets array to avoid temporary value issues
        let color_targets = [Some(wgpu::ColorTargetState {
            format: self.color_format,
            blend: self.blend_state,
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: self.label.as_deref(),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: vs_module,
                entry_point: "vs_main",
                buffers: vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: fs_module,
                entry_point: "fs_main",
                targets: &color_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: self.front_face,
                cull_mode: self.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: self.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        };

        // Create the pipeline directly instead of using resource manager
        let pipeline = device.create_render_pipeline(&render_pipeline_desc);
        Ok(pipeline)
    }
}

impl Default for GraphicsPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute pipeline builder with sensible defaults
pub struct ComputePipelineBuilder {
    label: Option<String>,
    compute_shader: Option<String>, // Use shader name instead of handle
}

impl ComputePipelineBuilder {
    /// Create a new compute pipeline builder
    pub fn new() -> Self {
        Self {
            label: None,
            compute_shader: None,
        }
    }

    /// Set the pipeline label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the compute shader by name
    pub fn with_compute_shader(mut self, shader_name: String) -> Self {
        self.compute_shader = Some(shader_name);
        self
    }

    /// Build the compute pipeline
    pub fn build(
        self,
        device: &wgpu::Device,
        resource_manager: &ResourceManager,
        shader_registry: &ShaderRegistry,
    ) -> Result<wgpu::ComputePipeline, PipelineError> {
        let compute_shader_name = self
            .compute_shader
            .ok_or_else(|| PipelineError::ShaderNotFound("compute shader".to_string()))?;

        // Get shader handle from registry
        let compute_shader = shader_registry
            .get_shader(&compute_shader_name)
            .ok_or(PipelineError::ShaderNotFound(compute_shader_name))?;

        // Get shader module
        let cs_module = resource_manager
            .get_shader(compute_shader)
            .map_err(|e| PipelineError::ResourceError(e.to_string()))?;

        // Create pipeline layout (empty for now, can be extended later)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self
                .label
                .as_ref()
                .map(|l| format!("{l} Layout"))
                .as_deref(),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let compute_pipeline_desc = wgpu::ComputePipelineDescriptor {
            label: self.label.as_deref(),
            layout: Some(&pipeline_layout),
            module: cs_module,
            entry_point: "cs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        };

        // Create the pipeline directly instead of using resource manager
        let pipeline = device.create_compute_pipeline(&compute_pipeline_desc);
        Ok(pipeline)
    }
}

impl Default for ComputePipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
