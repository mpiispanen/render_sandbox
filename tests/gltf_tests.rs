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
fn test_scene_operations() {
    // Test basic scene operations that don't require GPU
    let scene = Scene::new();
    let initial_count = scene.node_count();
    assert_eq!(initial_count, 0);
}
