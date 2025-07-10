use crate::resource_manager::Handle;
use cgmath::{Matrix4, Point3, Quaternion, Vector3, EuclideanSpace};
use std::collections::HashMap;

/// Unique identifier for scene nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Global node ID counter
static mut NODE_COUNTER: u64 = 1;

fn next_node_id() -> NodeId {
    unsafe {
        let id = NODE_COUNTER;
        NODE_COUNTER += 1;
        NodeId::new(id)
    }
}

/// 3D transformation (position, rotation, scale)
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    /// Create a new identity transform
    pub fn identity() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Create a transform with position
    pub fn from_position(position: Vector3<f32>) -> Self {
        Self {
            position,
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Convert to a 4x4 transformation matrix
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.position);
        let rotation = Matrix4::from(self.rotation);
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        
        translation * rotation * scale
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Mesh data stored as GPU resource handles
#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: Handle<wgpu::Buffer>,
    pub index_buffer: Option<Handle<wgpu::Buffer>>,
    pub vertex_count: u32,
    pub index_count: Option<u32>,
}

/// Light types
#[derive(Debug, Clone)]
pub enum Light {
    Directional {
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
    },
    Point {
        color: Vector3<f32>,
        intensity: f32,
        range: f32,
    },
    Spot {
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        range: f32,
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    },
}

/// Camera data
#[derive(Debug, Clone)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    /// Create a new perspective camera
    pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            near,
            far,
            aspect_ratio,
        }
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fov), self.aspect_ratio, self.near, self.far)
    }

    /// Get the view matrix based on transform
    pub fn view_matrix(&self, transform: &Transform) -> Matrix4<f32> {
        let eye = Point3::from_vec(transform.position);
        let forward = transform.rotation * Vector3::new(0.0, 0.0, -1.0);
        let up = transform.rotation * Vector3::new(0.0, 1.0, 0.0);
        let target = eye + forward;
        
        cgmath::Matrix4::look_at_rh(eye, target, up)
    }
}

/// Content that can be attached to a scene node
#[derive(Debug)]
pub enum NodeContent {
    Mesh(Mesh),
    Light(Light),
    Camera(Camera),
}

/// A node in the scene graph
#[derive(Debug)]
pub struct SceneNode {
    pub id: NodeId,
    pub transform: Transform,
    pub content: Option<NodeContent>,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub name: Option<String>,
    pub visible: bool,
}

impl SceneNode {
    /// Create a new empty scene node
    pub fn new() -> Self {
        Self {
            id: next_node_id(),
            transform: Transform::identity(),
            content: None,
            children: Vec::new(),
            parent: None,
            name: None,
            visible: true,
        }
    }

    /// Create a new scene node with a name
    pub fn with_name(name: &str) -> Self {
        Self {
            id: next_node_id(),
            transform: Transform::identity(),
            content: None,
            children: Vec::new(),
            parent: None,
            name: Some(name.to_string()),
            visible: true,
        }
    }

    /// Set the transform
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Set the content
    pub fn with_content(mut self, content: NodeContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Get the global transformation matrix
    pub fn global_transform(&self, scene: &Scene) -> Matrix4<f32> {
        if let Some(parent_id) = self.parent {
            if let Some(parent) = scene.get_node(parent_id) {
                return parent.global_transform(scene) * self.transform.to_matrix();
            }
        }
        self.transform.to_matrix()
    }
}

impl Default for SceneNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Hierarchical scene graph
pub struct Scene {
    nodes: HashMap<NodeId, SceneNode>,
    root_nodes: Vec<NodeId>,
    main_camera: Option<NodeId>,
}

impl Scene {
    /// Create a new empty scene
    pub fn new() -> Self {
        log::info!("Creating new scene");
        Self {
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            main_camera: None,
        }
    }

    /// Add a node to the scene
    pub fn add_node(&mut self, node: SceneNode) -> NodeId {
        let node_id = node.id;
        log::debug!("Adding node to scene: {:?} ({})", node_id, node.name.as_deref().unwrap_or("unnamed"));
        
        self.nodes.insert(node_id, node);
        self.root_nodes.push(node_id);
        
        node_id
    }

    /// Add a node as a child of another node
    pub fn add_child_node(&mut self, parent_id: NodeId, mut child: SceneNode) -> Option<NodeId> {
        let child_id = child.id;
        child.parent = Some(parent_id);
        
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
            self.nodes.insert(child_id, child);
            
            // Remove from root nodes if it was there
            self.root_nodes.retain(|&id| id != child_id);
            
            log::debug!("Added child node {:?} to parent {:?}", child_id, parent_id);
            Some(child_id)
        } else {
            log::warn!("Failed to add child node: parent {:?} not found", parent_id);
            None
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(&node_id)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&node_id)
    }

    /// Remove a node and all its children from the scene
    pub fn remove_node(&mut self, node_id: NodeId) -> bool {
        if let Some(node) = self.nodes.get(&node_id) {
            // Collect children before removing the node
            let children: Vec<NodeId> = node.children.clone();
            let parent_id = node.parent;
            
            // Remove from parent's children list
            if let Some(parent_id) = parent_id {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|&id| id != node_id);
                }
            } else {
                // Remove from root nodes
                self.root_nodes.retain(|&id| id != node_id);
            }
            
            // Remove the node
            self.nodes.remove(&node_id);
            
            // Recursively remove children
            for child_id in children {
                self.remove_node(child_id);
            }
            
            // Clear main camera if it was removed
            if self.main_camera == Some(node_id) {
                self.main_camera = None;
            }
            
            log::debug!("Removed node {:?} from scene", node_id);
            true
        } else {
            false
        }
    }

    /// Set the main camera
    pub fn set_main_camera(&mut self, camera_id: NodeId) -> bool {
        if let Some(node) = self.nodes.get(&camera_id) {
            if matches!(node.content, Some(NodeContent::Camera(_))) {
                self.main_camera = Some(camera_id);
                log::debug!("Set main camera to node {:?}", camera_id);
                true
            } else {
                log::warn!("Node {:?} is not a camera", camera_id);
                false
            }
        } else {
            log::warn!("Camera node {:?} not found", camera_id);
            false
        }
    }

    /// Get the main camera node
    pub fn get_main_camera(&self) -> Option<&SceneNode> {
        self.main_camera.and_then(|id| self.get_node(id))
    }

    /// Get all nodes with mesh content
    pub fn get_mesh_nodes(&self) -> Vec<&SceneNode> {
        self.nodes
            .values()
            .filter(|node| {
                node.visible && matches!(node.content, Some(NodeContent::Mesh(_)))
            })
            .collect()
    }

    /// Get all nodes with light content
    pub fn get_light_nodes(&self) -> Vec<&SceneNode> {
        self.nodes
            .values()
            .filter(|node| {
                node.visible && matches!(node.content, Some(NodeContent::Light(_)))
            })
            .collect()
    }

    /// Get all root nodes
    pub fn get_root_nodes(&self) -> &[NodeId] {
        &self.root_nodes
    }

    /// Update the scene (called each frame)
    pub fn update(&mut self, _delta_time: f32) {
        // Placeholder for scene updates
        // This is where animation, physics, and other scene logic would run
        log::trace!("Updating scene with {} nodes", self.nodes.len());
    }

    /// Get the total number of nodes in the scene
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Clear all nodes from the scene
    pub fn clear(&mut self) {
        log::info!("Clearing scene");
        self.nodes.clear();
        self.root_nodes.clear();
        self.main_camera = None;
    }

    /// Traverse the scene depth-first
    pub fn traverse_depth_first<F>(&self, mut visitor: F)
    where
        F: FnMut(&SceneNode),
    {
        for &root_id in &self.root_nodes {
            if let Some(node) = self.get_node(root_id) {
                self.traverse_node_depth_first(node, &mut visitor);
            }
        }
    }

    /// Helper function for depth-first traversal
    fn traverse_node_depth_first<F>(&self, node: &SceneNode, visitor: &mut F)
    where
        F: FnMut(&SceneNode),
    {
        visitor(node);
        
        for &child_id in &node.children {
            if let Some(child) = self.get_node(child_id) {
                self.traverse_node_depth_first(child, visitor);
            }
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}