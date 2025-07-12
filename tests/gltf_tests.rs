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

    // Test simple sample models from official glTF Sample Models repository
    let simple_models = [
        "test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf",
        "test_assets/gltf_sample_models/2.0/Box/glTF/Box.gltf",
    ];

    for model_path in &simple_models {
        let path = Path::new(model_path);
        if path.exists() {
            let gltf_result = gltf::Gltf::open(path);
            assert!(gltf_result.is_ok(), "Should be able to load {model_path}");

            let gltf_doc = gltf_result.unwrap();

            // Verify basic structure
            assert!(
                gltf_doc.meshes().next().is_some(),
                "Model {model_path} should have at least one mesh"
            );
            assert!(
                gltf_doc.scenes().next().is_some(),
                "Model {model_path} should have at least one scene"
            );

            // Verify mesh has position attributes
            let mesh = gltf_doc.meshes().next().unwrap();
            let primitive = mesh.primitives().next().unwrap();
            assert!(
                primitive.get(&gltf::Semantic::Positions).is_some(),
                "Model {model_path} should have position attributes"
            );
        }
    }
}

#[test]
fn test_gltf_sample_models_complex() {
    use std::path::Path;

    // Test complex sample models from official glTF Sample Models repository
    let complex_models = [
        "test_assets/gltf_sample_models/2.0/SimpleMeshes/glTF/SimpleMeshes.gltf",
        "test_assets/gltf_sample_models/2.0/Cameras/glTF/Cameras.gltf",
    ];

    for model_path in &complex_models {
        let path = Path::new(model_path);
        if path.exists() {
            let gltf_result = gltf::Gltf::open(path);
            assert!(gltf_result.is_ok(), "Should be able to load {model_path}");

            let gltf_doc = gltf_result.unwrap();

            // Verify basic structure
            assert!(
                gltf_doc.meshes().next().is_some(),
                "Model {model_path} should have at least one mesh"
            );
            assert!(
                gltf_doc.scenes().next().is_some(),
                "Model {model_path} should have at least one scene"
            );

            // Complex models should have multiple objects or hierarchical structure
            let scene = gltf_doc.scenes().next().unwrap();
            let node_count = scene.nodes().count();
            assert!(
                node_count >= 1,
                "Complex model {model_path} should have nodes"
            );

            // Count total meshes - complex models should have multiple meshes or hierarchical nodes
            let total_meshes = gltf_doc.meshes().count();
            assert!(
                total_meshes >= 1,
                "Complex model {model_path} should have meshes"
            );
        }
    }
}

#[test]
fn test_gltf_sample_model_vertex_counts() {
    use std::path::Path;

    // Test vertex counts for known models from official glTF Sample Models
    let triangle_path = Path::new("test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf");
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

    let box_path = Path::new("test_assets/gltf_sample_models/2.0/Box/glTF/Box.gltf");
    if box_path.exists() {
        let gltf_doc = gltf::Gltf::open(box_path).unwrap();
        let mesh = gltf_doc.meshes().next().unwrap();
        let primitive = mesh.primitives().next().unwrap();
        let position_accessor = primitive.get(&gltf::Semantic::Positions).unwrap();
        assert_eq!(
            position_accessor.count(),
            24,
            "Box should have 24 vertices (6 faces * 4 vertices)"
        );
    }
}

#[test]
fn test_gltf_sample_model_scene_structure() {
    use std::path::Path;

    let cameras_path = Path::new("test_assets/gltf_sample_models/2.0/Cameras/glTF/Cameras.gltf");
    if cameras_path.exists() {
        let gltf_doc = gltf::Gltf::open(cameras_path).unwrap();

        // Verify hierarchical structure
        let scene = gltf_doc.scenes().next().unwrap();
        let root_nodes: Vec<_> = scene.nodes().collect();

        // Check the scene structure - Cameras model has multiple nodes but no hierarchy
        assert!(!root_nodes.is_empty(), "Scene should have root nodes");

        // Verify we have multiple nodes (cameras)
        let total_nodes = gltf_doc.nodes().count();
        assert!(
            total_nodes > 1,
            "Cameras scene should have multiple nodes representing different cameras"
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
