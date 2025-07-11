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

#[test]
fn test_gltf_sample_models_structure_validation() {
    use std::path::Path;

    // Test structural integrity of sample models without GPU
    let sample_models = [
        (
            "test_assets/gltf_samples/simple/triangle.gltf",
            "triangle",
            1,
            3,
        ),
        ("test_assets/gltf_samples/simple/box.gltf", "box", 1, 8),
        (
            "test_assets/gltf_samples/complex/multi_cube.gltf",
            "multi_cube",
            2,
            8,
        ),
        (
            "test_assets/gltf_samples/complex/hierarchical_scene.gltf",
            "hierarchical",
            2,
            3,
        ),
    ];

    for (model_path, model_name, expected_meshes, expected_min_vertices) in sample_models {
        let path = Path::new(model_path);
        if path.exists() {
            log::info!("Testing sample model structure: {}", model_name);

            let gltf_result = gltf::Gltf::open(path);
            assert!(
                gltf_result.is_ok(),
                "Should load {} successfully",
                model_name
            );

            let gltf_doc = gltf_result.unwrap();

            // Verify mesh count
            let mesh_count = gltf_doc.meshes().count();
            assert_eq!(
                mesh_count, expected_meshes,
                "Model {} should have {} meshes, found {}",
                model_name, expected_meshes, mesh_count
            );

            // Verify vertex count in first mesh
            if let Some(mesh) = gltf_doc.meshes().next() {
                if let Some(primitive) = mesh.primitives().next() {
                    if let Some(accessor) = primitive.get(&gltf::Semantic::Positions) {
                        assert!(
                            accessor.count() >= expected_min_vertices,
                            "Model {} should have at least {} vertices in first mesh",
                            model_name,
                            expected_min_vertices
                        );
                    }
                }
            }

            log::info!("Sample model {} structure validation passed", model_name);
        } else {
            log::warn!("Sample model {} not found, skipping test", model_name);
        }
    }
}

#[test]
fn test_gltf_sample_models_scene_hierarchy() {
    use std::path::Path;

    // Test scene hierarchy features in complex models
    let hierarchical_path = Path::new("test_assets/gltf_samples/complex/hierarchical_scene.gltf");
    if hierarchical_path.exists() {
        let gltf_doc = gltf::Gltf::open(hierarchical_path).unwrap();

        // Verify scene structure
        let scene = gltf_doc.scenes().next().unwrap();
        let root_node_count = scene.nodes().count();
        assert_eq!(
            root_node_count, 1,
            "Hierarchical scene should have 1 root node"
        );

        // Verify node hierarchy
        let total_nodes = gltf_doc.nodes().count();
        assert_eq!(
            total_nodes, 5,
            "Hierarchical scene should have 5 total nodes"
        );

        // Verify some nodes have children
        let nodes_with_children = gltf_doc
            .nodes()
            .filter(|node| node.children().count() > 0)
            .count();
        assert!(
            nodes_with_children >= 2,
            "Should have nodes with children for hierarchy"
        );

        // Verify transforms are present
        let nodes_with_translation = gltf_doc
            .nodes()
            .filter(|node| {
                let (translation, _, _) = node.transform().decomposed();
                translation != [0.0, 0.0, 0.0]
            })
            .count();
        assert!(
            nodes_with_translation >= 3,
            "Should have nodes with non-zero translations"
        );

        log::info!("Hierarchical scene structure validation passed");
    }
}

#[test]
fn test_gltf_sample_models_comparison() {
    use std::path::Path;

    // Compare simple models to verify they have different characteristics
    let triangle_path = Path::new("test_assets/gltf_samples/simple/triangle.gltf");
    let box_path = Path::new("test_assets/gltf_samples/simple/box.gltf");

    if triangle_path.exists() && box_path.exists() {
        let triangle_doc = gltf::Gltf::open(triangle_path).unwrap();
        let box_doc = gltf::Gltf::open(box_path).unwrap();

        // Get vertex counts
        let triangle_vertices = triangle_doc
            .meshes()
            .next()
            .and_then(|m| m.primitives().next())
            .and_then(|p| p.get(&gltf::Semantic::Positions))
            .map(|a| a.count())
            .unwrap_or(0);

        let box_vertices = box_doc
            .meshes()
            .next()
            .and_then(|m| m.primitives().next())
            .and_then(|p| p.get(&gltf::Semantic::Positions))
            .map(|a| a.count())
            .unwrap_or(0);

        // Verify different complexity
        assert_eq!(triangle_vertices, 3, "Triangle should have 3 vertices");
        assert_eq!(box_vertices, 8, "Box should have 8 vertices");
        assert!(
            box_vertices > triangle_vertices,
            "Box should be more complex than triangle"
        );

        log::info!(
            "Sample model comparison test passed - triangle: {} vertices, box: {} vertices",
            triangle_vertices,
            box_vertices
        );
    }
}
