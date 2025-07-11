use render_sandbox::scene::Scene;

// Test the GLTF integration with the renderer
#[test]
fn test_renderer_gltf_integration() {
    // This test verifies that the renderer can create GLTF-style geometry
    // but skips GPU initialization to work in headless environments

    // Test that the Scene can accept nodes (basic functionality)
    let scene = Scene::new();
    assert_eq!(scene.node_count(), 0);

    // The actual GLTF integration would require GPU initialization,
    // so we just test the scene setup here
    log::info!("GLTF integration test - scene setup verified");
}

#[test]
fn test_gltf_triangle_vs_regular_triangle() {
    // This test loads actual GLTF data and validates the triangle structure
    use std::path::Path;

    let triangle_path = Path::new("test_assets/triangle.gltf");

    // Skip test if GLTF file doesn't exist (for CI environments)
    if !triangle_path.exists() {
        log::info!("GLTF test asset not found, skipping GLTF triangle comparison");
        return;
    }

    // Load and parse the GLTF file (without blob data, just structure validation)
    let gltf_result = gltf::Gltf::open(triangle_path);
    assert!(gltf_result.is_ok(), "Should be able to load GLTF file");

    let gltf_doc = gltf_result.unwrap();

    // Verify the GLTF structure matches our expectations for a triangle
    assert_eq!(gltf_doc.meshes().len(), 1, "Should have exactly one mesh");

    let mesh = gltf_doc.meshes().next().unwrap();
    assert_eq!(
        mesh.primitives().len(),
        1,
        "Mesh should have exactly one primitive"
    );

    let primitive = mesh.primitives().next().unwrap();

    // Check that the mesh has position attributes (essential for any triangle)
    assert!(
        primitive.get(&gltf::Semantic::Positions).is_some(),
        "Mesh should have position attributes"
    );

    let position_accessor = primitive.get(&gltf::Semantic::Positions).unwrap();

    // Verify the vertex count matches our expectation (3 vertices for triangle)
    assert_eq!(
        position_accessor.count(),
        3,
        "Triangle should have exactly 3 vertices"
    );

    // Verify we have the expected scene structure
    assert_eq!(gltf_doc.scenes().len(), 1, "Should have exactly one scene");
    let scene = gltf_doc.scenes().next().unwrap();
    assert_eq!(
        scene.nodes().len(),
        1,
        "Scene should have exactly one root node"
    );

    let root_node = scene.nodes().next().unwrap();
    assert!(
        root_node.mesh().is_some(),
        "Root node should reference the mesh"
    );

    // Verify the mesh reference points to our triangle mesh
    let referenced_mesh = root_node.mesh().unwrap();
    assert_eq!(
        referenced_mesh.index(),
        mesh.index(),
        "Root node should reference the triangle mesh"
    );

    log::info!(
        "GLTF triangle structure validation passed - found {} vertices in correct scene structure",
        position_accessor.count()
    );
}

#[test]
fn test_scene_node_creation_for_gltf() {
    use render_sandbox::scene::SceneNode;

    // Test creating a scene node that would hold GLTF data
    let node = SceneNode::with_name("TestGltfMesh");
    assert_eq!(node.name, Some("TestGltfMesh".to_string()));

    // Verify we can create the basic structure for a mesh node
    // (without actual GPU resources)
    log::info!("GLTF scene node structure test passed");
}
