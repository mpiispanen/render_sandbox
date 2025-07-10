use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::util::DeviceExt;

/// Unique identifier for a resource handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleId(u64);

/// Type-safe handle for GPU resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    id: HandleId,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    fn new(id: HandleId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> HandleId {
        self.id
    }
}

/// Global handle counter
static HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_handle_id() -> HandleId {
    HandleId(HANDLE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Resource types that can be managed
pub enum Resource {
    Buffer(wgpu::Buffer),
    Texture(wgpu::Texture),
    BindGroup(wgpu::BindGroup),
    BindGroupLayout(wgpu::BindGroupLayout),
    RenderPipeline(wgpu::RenderPipeline),
    ComputePipeline(wgpu::ComputePipeline),
    Shader(wgpu::ShaderModule),
    Sampler(wgpu::Sampler),
}

/// Errors that can occur during resource management operations
#[derive(Debug)]
pub enum ResourceError {
    ResourceNotFound(HandleId),
    TypeMismatch(HandleId),
    CreationFailed(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::ResourceNotFound(id) => write!(f, "Resource not found: {:?}", id),
            ResourceError::TypeMismatch(id) => write!(f, "Resource type mismatch for: {:?}", id),
            ResourceError::CreationFailed(msg) => write!(f, "Resource creation failed: {msg}"),
        }
    }
}

impl std::error::Error for ResourceError {}

/// Manages GPU resources with handle-based access
pub struct ResourceManager {
    resources: HashMap<HandleId, Resource>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        log::info!("Creating resource manager");
        Self {
            resources: HashMap::new(),
        }
    }

    /// Create a buffer resource
    pub fn create_buffer(&mut self, device: &wgpu::Device, desc: &wgpu::BufferDescriptor) -> Handle<wgpu::Buffer> {
        let buffer = device.create_buffer(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created buffer handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::Buffer(buffer));
        
        handle
    }

    /// Create a buffer with initial data
    pub fn create_buffer_init(&mut self, device: &wgpu::Device, desc: &wgpu::util::BufferInitDescriptor) -> Handle<wgpu::Buffer> {
        let buffer = device.create_buffer_init(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created buffer with data handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::Buffer(buffer));
        
        handle
    }

    /// Create a texture resource
    pub fn create_texture(&mut self, device: &wgpu::Device, desc: &wgpu::TextureDescriptor) -> Handle<wgpu::Texture> {
        let texture = device.create_texture(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created texture handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::Texture(texture));
        
        handle
    }

    /// Create a shader module
    pub fn create_shader(&mut self, device: &wgpu::Device, desc: wgpu::ShaderModuleDescriptor) -> Handle<wgpu::ShaderModule> {
        let shader = device.create_shader_module(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created shader handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::Shader(shader));
        
        handle
    }

    /// Create a render pipeline
    pub fn create_render_pipeline(&mut self, device: &wgpu::Device, desc: &wgpu::RenderPipelineDescriptor) -> Handle<wgpu::RenderPipeline> {
        let pipeline = device.create_render_pipeline(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created render pipeline handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::RenderPipeline(pipeline));
        
        handle
    }

    /// Create a bind group layout
    pub fn create_bind_group_layout(&mut self, device: &wgpu::Device, desc: &wgpu::BindGroupLayoutDescriptor) -> Handle<wgpu::BindGroupLayout> {
        let layout = device.create_bind_group_layout(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created bind group layout handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::BindGroupLayout(layout));
        
        handle
    }

    /// Create a bind group
    pub fn create_bind_group(&mut self, device: &wgpu::Device, desc: &wgpu::BindGroupDescriptor) -> Handle<wgpu::BindGroup> {
        let bind_group = device.create_bind_group(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created bind group handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::BindGroup(bind_group));
        
        handle
    }

    /// Create a sampler
    pub fn create_sampler(&mut self, device: &wgpu::Device, desc: &wgpu::SamplerDescriptor) -> Handle<wgpu::Sampler> {
        let sampler = device.create_sampler(desc);
        let handle_id = next_handle_id();
        let handle = Handle::new(handle_id);
        
        log::debug!("Created sampler handle: {:?}", handle_id);
        self.resources.insert(handle_id, Resource::Sampler(sampler));
        
        handle
    }

    /// Get a buffer by handle
    pub fn get_buffer(&self, handle: Handle<wgpu::Buffer>) -> Result<&wgpu::Buffer, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::Buffer(buffer)) => Ok(buffer),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Get a texture by handle
    pub fn get_texture(&self, handle: Handle<wgpu::Texture>) -> Result<&wgpu::Texture, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::Texture(texture)) => Ok(texture),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Get a shader by handle
    pub fn get_shader(&self, handle: Handle<wgpu::ShaderModule>) -> Result<&wgpu::ShaderModule, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::Shader(shader)) => Ok(shader),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Get a render pipeline by handle
    pub fn get_render_pipeline(&self, handle: Handle<wgpu::RenderPipeline>) -> Result<&wgpu::RenderPipeline, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::RenderPipeline(pipeline)) => Ok(pipeline),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Get a bind group layout by handle
    pub fn get_bind_group_layout(&self, handle: Handle<wgpu::BindGroupLayout>) -> Result<&wgpu::BindGroupLayout, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::BindGroupLayout(layout)) => Ok(layout),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Get a bind group by handle
    pub fn get_bind_group(&self, handle: Handle<wgpu::BindGroup>) -> Result<&wgpu::BindGroup, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::BindGroup(bind_group)) => Ok(bind_group),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Get a sampler by handle
    pub fn get_sampler(&self, handle: Handle<wgpu::Sampler>) -> Result<&wgpu::Sampler, ResourceError> {
        match self.resources.get(&handle.id()) {
            Some(Resource::Sampler(sampler)) => Ok(sampler),
            Some(_) => Err(ResourceError::TypeMismatch(handle.id())),
            None => Err(ResourceError::ResourceNotFound(handle.id())),
        }
    }

    /// Remove a resource by handle
    pub fn remove_resource<T>(&mut self, handle: Handle<T>) -> bool {
        log::debug!("Removing resource handle: {:?}", handle.id());
        self.resources.remove(&handle.id()).is_some()
    }

    /// Get the number of managed resources
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Clear all resources
    pub fn clear(&mut self) {
        log::info!("Clearing all resources");
        self.resources.clear();
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}