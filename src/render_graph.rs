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

    /// Initialize the render pass with GPU resources (called once before first execution)
    fn initialize(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &ResourceManager,
    ) -> Result<(), RenderGraphError>;

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

    /// Initialize all render passes with GPU resources
    pub fn initialize_passes(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &ResourceManager,
    ) -> Result<(), RenderGraphError> {
        log::debug!("Initializing all render passes");

        for (pass_id, pass) in self.passes.iter_mut() {
            log::debug!("Initializing pass: {pass_id}");
            pass.initialize(device, resource_manager)?;
        }

        log::debug!("All render passes initialized successfully");
        Ok(())
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
