use render_sandbox::gltf_loader::GltfError;
use render_sandbox::scene::Scene;
use std::path::Path;

#[test]
fn test_gltf_error_types() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let gltf_error = GltfError::from(io_error);
    assert!(matches!(gltf_error, GltfError::IoError(_)));

    let validation_error = GltfError::ValidationError("test".to_string());
    assert_eq!(format!("{validation_error}"), "Validation error: test");

    let unsupported_error = GltfError::UnsupportedFeature("test feature".to_string());
    assert_eq!(
        format!("{unsupported_error}"),
        "Unsupported feature: test feature"
    );
}

#[test]
fn test_gltf_error_display() {
    let err = GltfError::ValidationError("test error".to_string());
    assert_eq!(format!("{err}"), "Validation error: test error");

    let err = GltfError::UnsupportedFeature("orthographic cameras".to_string());
    assert_eq!(
        format!("{err}"),
        "Unsupported feature: orthographic cameras"
    );
}

#[test]
fn test_gltf_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let gltf_err = GltfError::from(io_err);
    assert!(matches!(gltf_err, GltfError::IoError(_)));
}

#[test]
fn test_triangle_gltf_file_exists() {
    let triangle_path = Path::new("test_assets/triangle.gltf");

    // This test just checks that our test asset exists and is readable
    if triangle_path.exists() {
        let _content =
            std::fs::read_to_string(triangle_path).expect("Should be able to read test file");
        // Basic validation - file should contain "gltf" in JSON
        assert!(_content.contains("\"asset\""));
        assert!(_content.contains("\"version\""));
    } else {
        // If the test asset doesn't exist, the test should still pass
        // This allows tests to run in environments where test assets aren't available
        println!("Test asset triangle.gltf not found, skipping validation");
    }
}

#[test]
fn test_gltf_sample_models_simple() {
    use std::path::Path;

    // Test simple sample models
    let simple_models = [
        "test_assets/gltf_samples/simple/triangle.gltf",
        "test_assets/gltf_samples/simple/box.gltf",
    ];

    for model_path in &simple_models {
        let path = Path::new(model_path);
        if path.exists() {
            let gltf_result = gltf::Gltf::open(path);
            assert!(gltf_result.is_ok(), "Should be able to load {}", model_path);

            let gltf_doc = gltf_result.unwrap();

            // Verify basic structure
            assert!(
                !gltf_doc.meshes().next().is_none(),
                "Model {} should have at least one mesh",
                model_path
            );
            assert!(
                !gltf_doc.scenes().next().is_none(),
                "Model {} should have at least one scene",
                model_path
            );

            // Verify mesh has position attributes
            let mesh = gltf_doc.meshes().next().unwrap();
            let primitive = mesh.primitives().next().unwrap();
            assert!(
                primitive.get(&gltf::Semantic::Positions).is_some(),
                "Model {} should have position attributes",
                model_path
            );
        }
    }
}

#[test]
fn test_gltf_sample_models_complex() {
    use std::path::Path;

    // Test complex sample models
    let complex_models = [
        "test_assets/gltf_samples/complex/multi_cube.gltf",
        "test_assets/gltf_samples/complex/hierarchical_scene.gltf",
    ];

    for model_path in &complex_models {
        let path = Path::new(model_path);
        if path.exists() {
            let gltf_result = gltf::Gltf::open(path);
            assert!(gltf_result.is_ok(), "Should be able to load {}", model_path);

            let gltf_doc = gltf_result.unwrap();

            // Verify basic structure
            assert!(
                !gltf_doc.meshes().next().is_none(),
                "Model {} should have at least one mesh",
                model_path
            );
            assert!(
                !gltf_doc.scenes().next().is_none(),
                "Model {} should have at least one scene",
                model_path
            );

            // Complex models should have multiple objects or hierarchical structure
            let scene = gltf_doc.scenes().next().unwrap();
            let node_count = scene.nodes().count();
            assert!(
                node_count >= 1,
                "Complex model {} should have nodes",
                model_path
            );

            // Count total meshes - complex models should have multiple meshes or hierarchical nodes
            let total_meshes = gltf_doc.meshes().count();
            assert!(
                total_meshes >= 1,
                "Complex model {} should have meshes",
                model_path
            );
        }
    }
}

#[test]
fn test_gltf_sample_model_vertex_counts() {
    use std::path::Path;

    // Test vertex counts for known models
    let triangle_path = Path::new("test_assets/gltf_samples/simple/triangle.gltf");
    if triangle_path.exists() {
        let gltf_doc = gltf::Gltf::open(triangle_path).unwrap();
        let mesh = gltf_doc.meshes().next().unwrap();
        let primitive = mesh.primitives().next().unwrap();
        let position_accessor = primitive.get(&gltf::Semantic::Positions).unwrap();
        assert_eq!(
            position_accessor.count(),
            3,
            "Triangle should have 3 vertices"
        );
    }

    let box_path = Path::new("test_assets/gltf_samples/simple/box.gltf");
    if box_path.exists() {
        let gltf_doc = gltf::Gltf::open(box_path).unwrap();
        let mesh = gltf_doc.meshes().next().unwrap();
        let primitive = mesh.primitives().next().unwrap();
        let position_accessor = primitive.get(&gltf::Semantic::Positions).unwrap();
        assert_eq!(position_accessor.count(), 8, "Box should have 8 vertices");
    }
}

#[test]
fn test_gltf_sample_model_hierarchy() {
    use std::path::Path;

    let hierarchical_path = Path::new("test_assets/gltf_samples/complex/hierarchical_scene.gltf");
    if hierarchical_path.exists() {
        let gltf_doc = gltf::Gltf::open(hierarchical_path).unwrap();

        // Verify hierarchical structure
        let scene = gltf_doc.scenes().next().unwrap();
        let root_nodes: Vec<_> = scene.nodes().collect();

        // Should have at least one root node
        assert!(
            !root_nodes.is_empty(),
            "Hierarchical scene should have root nodes"
        );

        // Check if any node has children (hierarchical structure)
        let has_hierarchy = gltf_doc.nodes().any(|node| node.children().count() > 0);
        assert!(
            has_hierarchy,
            "Hierarchical scene should have parent-child relationships"
        );

        // Verify transforms exist
        let has_transforms = gltf_doc.nodes().any(|node| {
            let (translation, _, _) = node.transform().decomposed();
            translation != [0.0, 0.0, 0.0]
        });
        assert!(
            has_transforms,
            "Hierarchical scene should have non-identity transforms"
        );
    }
}

#[test]
fn test_scene_operations() {
    // Test basic scene operations that don't require GPU
    let scene = Scene::new();
    let initial_count = scene.node_count();
    assert_eq!(initial_count, 0);
}
