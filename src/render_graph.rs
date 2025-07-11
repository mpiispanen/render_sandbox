use crate::resource_manager::ResourceManager;
use std::collections::{HashMap, HashSet, VecDeque};

/// Identifier for a render pass
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PassId(String);

impl PassId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl std::fmt::Display for PassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a render resource within the graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// How a pass uses a resource
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceUsage {
    Read,
    Write,
    ReadWrite,
}

/// Resource declaration for a render pass
#[derive(Debug, Clone)]
pub struct ResourceDeclaration {
    pub id: ResourceId,
    pub usage: ResourceUsage,
}

/// Errors that can occur during render graph operations
#[derive(Debug)]
pub enum RenderGraphError {
    CyclicDependency,
    PassNotFound(PassId),
    ResourceNotFound(ResourceId),
    CompilationFailed(String),
    ExecutionFailed(String),
}

impl std::fmt::Display for RenderGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderGraphError::CyclicDependency => {
                write!(f, "Cyclic dependency detected in render graph")
            }
            RenderGraphError::PassNotFound(id) => write!(f, "Render pass not found: {id}"),
            RenderGraphError::ResourceNotFound(id) => write!(f, "Resource not found: {id}"),
            RenderGraphError::CompilationFailed(msg) => {
                write!(f, "Render graph compilation failed: {msg}")
            }
            RenderGraphError::ExecutionFailed(msg) => {
                write!(f, "Render graph execution failed: {msg}")
            }
        }
    }
}

impl std::error::Error for RenderGraphError {}

/// Trait for executing a render pass
pub trait RenderPass: Send + Sync {
    /// Get the unique identifier for this pass
    fn id(&self) -> PassId;

    /// Get the resource declarations for this pass
    fn resources(&self) -> Vec<ResourceDeclaration>;

    /// Execute the render pass
    fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resource_manager: &ResourceManager,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), RenderGraphError>;
}

/// Compiled render graph with topologically sorted passes
#[derive(Debug)]
pub struct CompiledRenderGraph {
    passes: Vec<PassId>,
    #[allow(dead_code)]
    resource_dependencies: HashMap<ResourceId, Vec<PassId>>,
}

/// Dynamic render graph that manages render passes and their dependencies
pub struct RenderGraph {
    passes: HashMap<PassId, Box<dyn RenderPass>>,
    resource_declarations: HashMap<PassId, Vec<ResourceDeclaration>>,
    compiled: Option<CompiledRenderGraph>,
}

impl RenderGraph {
    /// Create a new render graph
    pub fn new() -> Self {
        log::info!("Creating new render graph");
        Self {
            passes: HashMap::new(),
            resource_declarations: HashMap::new(),
            compiled: None,
        }
    }

    /// Add a render pass to the graph
    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        let id = pass.id();
        let resources = pass.resources();

        log::debug!("Adding render pass: {id}");
        log::debug!("  Resources: {resources:?}");

        self.resource_declarations.insert(id.clone(), resources);
        self.passes.insert(id, pass);

        // Invalidate compilation
        self.compiled = None;
    }

    /// Remove a render pass from the graph
    pub fn remove_pass(&mut self, id: &PassId) -> bool {
        log::debug!("Removing render pass: {id}");

        let removed = self.passes.remove(id).is_some();
        self.resource_declarations.remove(id);

        if removed {
            // Invalidate compilation
            self.compiled = None;
        }

        removed
    }

    /// Clear all passes from the graph
    pub fn clear(&mut self) {
        log::info!("Clearing render graph");
        self.passes.clear();
        self.resource_declarations.clear();
        self.compiled = None;
    }

    /// Compile the render graph
    pub fn compile(&mut self) -> Result<(), RenderGraphError> {
        log::info!("Compiling render graph with {} passes", self.passes.len());

        // Build dependency graph
        let mut dependencies: HashMap<PassId, HashSet<PassId>> = HashMap::new();
        let mut resource_writers: HashMap<ResourceId, Vec<PassId>> = HashMap::new();
        let mut resource_readers: HashMap<ResourceId, Vec<PassId>> = HashMap::new();

        // First pass: collect all resource writers and readers
        for (pass_id, resources) in &self.resource_declarations {
            dependencies.insert(pass_id.clone(), HashSet::new());

            for resource in resources {
                match resource.usage {
                    ResourceUsage::Write | ResourceUsage::ReadWrite => {
                        resource_writers
                            .entry(resource.id.clone())
                            .or_default()
                            .push(pass_id.clone());
                    }
                    _ => {}
                }

                match resource.usage {
                    ResourceUsage::Read | ResourceUsage::ReadWrite => {
                        resource_readers
                            .entry(resource.id.clone())
                            .or_default()
                            .push(pass_id.clone());
                    }
                    _ => {}
                }
            }
        }

        // Second pass: build dependencies (readers depend on writers)
        for (resource_id, readers) in &resource_readers {
            if let Some(writers) = resource_writers.get(resource_id) {
                for reader in readers {
                    for writer in writers {
                        if reader != writer {
                            dependencies.get_mut(reader).unwrap().insert(writer.clone());
                        }
                    }
                }
            }
        }

        // Topological sort
        let sorted_passes = self.topological_sort(&dependencies)?;

        // Build resource dependency map
        let mut resource_dependencies: HashMap<ResourceId, Vec<PassId>> = HashMap::new();
        for (resource_id, passes) in resource_writers {
            resource_dependencies.insert(resource_id, passes);
        }

        self.compiled = Some(CompiledRenderGraph {
            passes: sorted_passes,
            resource_dependencies,
        });

        log::info!("Render graph compiled successfully");
        Ok(())
    }

    /// Perform topological sort to determine execution order
    fn topological_sort(
        &self,
        dependencies: &HashMap<PassId, HashSet<PassId>>,
    ) -> Result<Vec<PassId>, RenderGraphError> {
        let mut in_degree: HashMap<PassId, usize> = HashMap::new();
        let mut adj_list: HashMap<PassId, Vec<PassId>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for pass_id in self.passes.keys() {
            in_degree.insert(pass_id.clone(), 0);
            adj_list.insert(pass_id.clone(), Vec::new());
        }

        // Build graph and calculate in-degrees
        for (dependent, deps) in dependencies {
            for dependency in deps {
                adj_list
                    .get_mut(dependency)
                    .unwrap()
                    .push(dependent.clone());
                *in_degree.get_mut(dependent).unwrap() += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<PassId> = VecDeque::new();
        for (pass_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(pass_id.clone());
            }
        }

        let mut sorted = Vec::new();
        while let Some(current) = queue.pop_front() {
            sorted.push(current.clone());

            for neighbor in &adj_list[&current] {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }

        if sorted.len() != self.passes.len() {
            return Err(RenderGraphError::CyclicDependency);
        }

        Ok(sorted)
    }

    /// Execute the compiled render graph
    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resource_manager: &ResourceManager,
    ) -> Result<(), RenderGraphError> {
        let compiled = self
            .compiled
            .as_ref()
            .ok_or_else(|| RenderGraphError::CompilationFailed("Graph not compiled".to_string()))?;

        log::debug!(
            "Executing render graph with {} passes",
            compiled.passes.len()
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Graph Command Encoder"),
        });

        for pass_id in &compiled.passes {
            let pass = self
                .passes
                .get(pass_id)
                .ok_or_else(|| RenderGraphError::PassNotFound(pass_id.clone()))?;

            log::debug!("Executing pass: {pass_id}");

            pass.execute(device, queue, resource_manager, &mut encoder)
                .map_err(|e| {
                    RenderGraphError::ExecutionFailed(format!("Pass {pass_id} failed: {e}"))
                })?;
        }

        queue.submit(std::iter::once(encoder.finish()));

        log::debug!("Render graph execution completed");
        Ok(())
    }

    /// Get the number of passes in the graph
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Check if the graph is compiled
    pub fn is_compiled(&self) -> bool {
        self.compiled.is_some()
    }

    /// Get the execution order of passes (only available after compilation)
    pub fn execution_order(&self) -> Option<&[PassId]> {
        self.compiled.as_ref().map(|c| c.passes.as_slice())
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple placeholder render pass for testing
pub struct PlaceholderPass {
    id: PassId,
    resources: Vec<ResourceDeclaration>,
}

impl PlaceholderPass {
    pub fn new(name: &str) -> Self {
        Self {
            id: PassId::new(name),
            resources: vec![],
        }
    }

    pub fn with_resource(mut self, resource_id: &str, usage: ResourceUsage) -> Self {
        self.resources.push(ResourceDeclaration {
            id: ResourceId::new(resource_id),
            usage,
        });
        self
    }
}

impl RenderPass for PlaceholderPass {
    fn id(&self) -> PassId {
        self.id.clone()
    }

    fn resources(&self) -> Vec<ResourceDeclaration> {
        self.resources.clone()
    }

    fn execute(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _resource_manager: &ResourceManager,
        _encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), RenderGraphError> {
        log::debug!("Executing placeholder pass: {}", self.id);
        // Placeholder implementation - does nothing
        Ok(())
    }
}

/// A simple forward renderer pass that renders meshes
pub struct ForwardRenderPass {
    id: PassId,
    resources: Vec<ResourceDeclaration>,
    clear_color: [f64; 4],
    resolution: (u32, u32),
}

impl ForwardRenderPass {
    pub fn new(name: &str) -> Self {
        Self {
            id: PassId::new(name),
            resources: vec![],
            clear_color: [0.0, 0.0, 0.0, 1.0],
            resolution: (800, 600), // Default resolution
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
}

impl RenderPass for ForwardRenderPass {
    fn id(&self) -> PassId {
        self.id.clone()
    }

    fn resources(&self) -> Vec<ResourceDeclaration> {
        self.resources.clone()
    }

    fn execute(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _resource_manager: &ResourceManager,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), RenderGraphError> {
        log::debug!("Executing forward render pass: {}", self.id);

        // For now, create a simple offscreen texture to render to
        // In a full implementation, this would use the actual render targets
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Forward Render Target"),
            size: wgpu::Extent3d {
                width: self.resolution.0,
                height: self.resolution.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create a render pass that clears the screen
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_view,
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
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // For now, just clear the render target
        // TODO: Actual mesh rendering would happen here
        log::debug!("Forward render pass executed with clear");

        Ok(())
    }
}
