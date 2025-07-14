use render_sandbox::render_graph::{RenderGraph, ResourceUsage};
use render_sandbox::render_passes::PlaceholderPass;
use render_sandbox::renderer::RendererConfig;
use render_sandbox::resource_manager::ResourceManager;
use render_sandbox::scene::{Camera, NodeContent, Scene, SceneNode, Transform};

#[test]
fn test_resource_manager_creation() {
    let resource_manager = ResourceManager::new();
    assert_eq!(resource_manager.resource_count(), 0);
}

#[test]
fn test_render_graph_creation() {
    let mut render_graph = RenderGraph::new();
    assert_eq!(render_graph.pass_count(), 0);
    assert!(!render_graph.is_compiled());

    // Add a simple pass
    let pass = PlaceholderPass::new("TestPass");
    render_graph.add_pass(Box::new(pass));
    assert_eq!(render_graph.pass_count(), 1);
}

#[test]
fn test_render_graph_compilation() {
    let mut render_graph = RenderGraph::new();

    // Add passes with dependencies
    let pass1 = PlaceholderPass::new("Pass1").with_resource("ResourceA", ResourceUsage::Write);
    let pass2 = PlaceholderPass::new("Pass2").with_resource("ResourceA", ResourceUsage::Read);

    render_graph.add_pass(Box::new(pass1));
    render_graph.add_pass(Box::new(pass2));

    // Compile should succeed
    assert!(render_graph.compile().is_ok());
    assert!(render_graph.is_compiled());

    // Check execution order
    let order = render_graph.execution_order().unwrap();
    assert_eq!(order.len(), 2);
}

#[test]
fn test_scene_creation() {
    let mut scene = Scene::new();
    assert_eq!(scene.node_count(), 0);

    // Add a node
    let node = SceneNode::with_name("TestNode");
    let node_id = scene.add_node(node);
    assert_eq!(scene.node_count(), 1);

    // Get the node back
    let retrieved_node = scene.get_node(node_id);
    assert!(retrieved_node.is_some());
    assert_eq!(retrieved_node.unwrap().name.as_deref(), Some("TestNode"));
}

#[test]
fn test_scene_hierarchy() {
    let mut scene = Scene::new();

    // Create parent node
    let parent = SceneNode::with_name("Parent");
    let parent_id = scene.add_node(parent);

    // Create child node
    let child = SceneNode::with_name("Child");
    let child_id = scene.add_child_node(parent_id, child);
    assert!(child_id.is_some());

    // Check hierarchy
    let parent_node = scene.get_node(parent_id).unwrap();
    assert_eq!(parent_node.children.len(), 1);
    assert_eq!(parent_node.children[0], child_id.unwrap());

    let child_node = scene.get_node(child_id.unwrap()).unwrap();
    assert_eq!(child_node.parent, Some(parent_id));
}

#[test]
fn test_camera_creation() {
    let camera = Camera::perspective(45.0, 16.0 / 9.0, 0.1, 100.0);
    assert_eq!(camera.fov, 45.0);
    assert_eq!(camera.aspect_ratio, 16.0 / 9.0);
    assert_eq!(camera.near, 0.1);
    assert_eq!(camera.far, 100.0);

    // Test matrices
    let transform = Transform::identity();
    let _projection = camera.projection_matrix();
    let _view = camera.view_matrix(&transform);
}

#[test]
fn test_scene_camera() {
    let mut scene = Scene::new();

    // Create camera node
    let camera = Camera::perspective(45.0, 16.0 / 9.0, 0.1, 100.0);
    let camera_node = SceneNode::with_name("MainCamera").with_content(NodeContent::Camera(camera));
    let camera_id = scene.add_node(camera_node);

    // Set as main camera
    assert!(scene.set_main_camera(camera_id));

    // Get main camera
    let main_camera = scene.get_main_camera();
    assert!(main_camera.is_some());
    assert_eq!(main_camera.unwrap().name.as_deref(), Some("MainCamera"));
}

#[test]
fn test_renderer_config() {
    let config = RendererConfig::default();
    assert!(config.enable_depth_testing);
    assert!(config.enable_culling);
    assert_eq!(config.clear_color, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(config.msaa_samples, 1);
}

#[test]
fn test_transform_matrix() {
    let transform = Transform::identity();
    let matrix = transform.to_matrix();

    // Identity matrix should have 1.0 on diagonal
    assert_eq!(matrix[0][0], 1.0);
    assert_eq!(matrix[1][1], 1.0);
    assert_eq!(matrix[2][2], 1.0);
    assert_eq!(matrix[3][3], 1.0);
}
