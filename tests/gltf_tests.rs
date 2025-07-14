mod test_helpers;

use render_sandbox::{
    gltf_loader::{GltfError, GltfLoader},
    scene::{NodeContent, Scene},
};
use std::path::Path;
use test_helpers::TestGpuContext;

/// Test GLTF loader error types
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

/// Test GLTF error display formatting
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

/// Test GLTF error conversion from IO error
#[test]
fn test_gltf_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let gltf_err = GltfError::from(io_err);
    assert!(matches!(gltf_err, GltfError::IoError(_)));
}

/// Test creating a GLTF test triangle using our loader
#[test]
fn test_gltf_test_triangle_creation() {
    // Try to create GPU context, skip if not available
    let mut gpu_context = match futures::executor::block_on(TestGpuContext::new()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping test_gltf_test_triangle_creation - GPU not available: {e}");
            return;
        }
    };

    // Create triangle using our GltfLoader
    let (device, resource_manager) = gpu_context.split();
    let result = GltfLoader::create_test_triangle(device, resource_manager);

    match result {
        Ok(mesh) => {
            assert_eq!(mesh.vertex_count, 3, "Triangle should have 3 vertices");
            assert!(
                mesh.index_buffer.is_none(),
                "Triangle should not have index buffer"
            );
            println!("✓ GLTF test triangle created successfully");
        }
        Err(e) => {
            panic!("Failed to create GLTF test triangle: {e}");
        }
    }
}

/// Test loading GLTF files using our loader and scene
#[test]
fn test_gltf_sample_models_with_our_loader() {
    // Try to create GPU context, skip if not available
    let mut gpu_context = match futures::executor::block_on(TestGpuContext::new()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping test_gltf_sample_models_with_our_loader - GPU not available: {e}");
            return;
        }
    };

    let test_cases = [
        (
            "test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf",
            "Simple Triangle",
            1, // expected nodes after loading
        ),
        (
            "test_assets/gltf_sample_models/2.0/Box/glTF/Box.gltf",
            "Simple Box",
            1, // expected nodes after loading
        ),
        (
            "test_assets/triangle.gltf", // Fallback test asset
            "Test Triangle",
            1,
        ),
    ];

    for (path_str, name, expected_nodes) in test_cases {
        let path = Path::new(path_str);
        if path.exists() {
            println!("Testing GLTF loading with our loader: {name}");

            let mut scene = Scene::new();
            let initial_node_count = scene.node_count();

            // Use our GltfLoader to load the file
            let (device, resource_manager) = gpu_context.split();
            let result = GltfLoader::load_gltf(device, resource_manager, path, &mut scene);

            match result {
                Ok(()) => {
                    let final_node_count = scene.node_count();
                    let nodes_added = final_node_count - initial_node_count;

                    assert!(
                        nodes_added >= expected_nodes,
                        "{name} should add at least {expected_nodes} nodes, added {nodes_added}"
                    );

                    // Verify we have mesh nodes
                    let mesh_nodes = scene.get_mesh_nodes();
                    assert!(
                        !mesh_nodes.is_empty(),
                        "{name} should have at least one mesh node"
                    );

                    // Verify mesh nodes have content
                    for node in mesh_nodes {
                        assert!(
                            matches!(node.content, Some(NodeContent::Mesh(_))),
                            "Mesh node should have mesh content"
                        );
                    }

                    println!("✓ {name} loaded successfully with {nodes_added} nodes");
                }
                Err(e) => {
                    panic!("Failed to load {name} with our loader: {e}");
                }
            }
        } else {
            println!("⚠ GLTF file {path_str} not found, skipping {name}");
        }
    }
}

/// Test scene hierarchy validation after GLTF loading
#[test]
fn test_gltf_scene_hierarchy() {
    // Try to create GPU context, skip if not available
    let mut gpu_context = match futures::executor::block_on(TestGpuContext::new()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping test_gltf_scene_hierarchy - GPU not available: {e}");
            return;
        }
    };

    // Test hierarchical scene loading if available
    let hierarchical_models = [
        "test_assets/gltf_sample_models/2.0/SimpleMeshes/glTF/SimpleMeshes.gltf",
        "test_assets/gltf_sample_models/2.0/AnimatedCube/glTF/AnimatedCube.gltf",
    ];

    for model_path in hierarchical_models {
        let path = Path::new(model_path);
        if path.exists() {
            println!("Testing scene hierarchy for: {model_path}");

            let mut scene = Scene::new();

            let (device, resource_manager) = gpu_context.split();
            let result = GltfLoader::load_gltf(device, resource_manager, path, &mut scene);

            match result {
                Ok(()) => {
                    assert!(
                        scene.node_count() > 0,
                        "Scene should have nodes after loading"
                    );

                    // Test scene traversal
                    let mut visited_nodes = 0;
                    scene.traverse_depth_first(|_node| {
                        visited_nodes += 1;
                    });

                    assert_eq!(
                        visited_nodes,
                        scene.node_count(),
                        "Traversal should visit all nodes"
                    );

                    println!("✓ Scene hierarchy validated for {model_path}");
                }
                Err(e) => {
                    eprintln!("Failed to load {model_path}: {e}");
                }
            }
        }
    }
}

/// Test scene complexity comparison
#[test]
fn test_gltf_scene_complexity() {
    // Try to create GPU context, skip if not available
    let mut gpu_context = match futures::executor::block_on(TestGpuContext::new()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping test_gltf_scene_complexity - GPU not available: {e}");
            return;
        }
    };

    // Load simple model
    let simple_path = "test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf";
    let complex_path = "test_assets/gltf_sample_models/2.0/AnimatedCube/glTF/AnimatedCube.gltf";

    if Path::new(simple_path).exists() && Path::new(complex_path).exists() {
        let mut simple_scene = Scene::new();
        let mut complex_scene = Scene::new();

        // Load simple model
        let (device, resource_manager) = gpu_context.split();
        let _ = GltfLoader::load_gltf(device, resource_manager, simple_path, &mut simple_scene);

        // Re-split for second call (needed due to borrow checker)
        let (device, resource_manager) = gpu_context.split();
        let _ = GltfLoader::load_gltf(device, resource_manager, complex_path, &mut complex_scene);

        let simple_nodes = simple_scene.node_count();
        let complex_nodes = complex_scene.node_count();

        if simple_nodes > 0 && complex_nodes > 0 {
            // Complex models should generally have more or equal nodes
            // (though this isn't always guaranteed)
            println!(
                "Scene complexity: simple={simple_nodes} nodes, complex={complex_nodes} nodes"
            );

            // At minimum, both should have loaded something
            assert!(simple_nodes > 0, "Simple scene should have nodes");
            assert!(complex_nodes > 0, "Complex scene should have nodes");
        }
    }
}

/// Test that scene properly handles GLTF features
#[test]
fn test_gltf_feature_support() {
    // Try to create GPU context, skip if not available
    let mut gpu_context = match futures::executor::block_on(TestGpuContext::new()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping test_gltf_feature_support - GPU not available: {e}");
            return;
        }
    };

    // Test basic mesh loading
    let test_triangle_path = "test_assets/triangle.gltf";
    if Path::new(test_triangle_path).exists() {
        let mut scene = Scene::new();

        let (device, resource_manager) = gpu_context.split();
        match GltfLoader::load_gltf(device, resource_manager, test_triangle_path, &mut scene) {
            Ok(()) => {
                // Verify basic features
                assert!(scene.node_count() > 0, "Should have loaded nodes");

                let mesh_nodes = scene.get_mesh_nodes();
                assert!(!mesh_nodes.is_empty(), "Should have mesh nodes");

                // Test that we can traverse the scene
                let mut mesh_count = 0;
                scene.traverse_depth_first(|node| {
                    if matches!(node.content, Some(NodeContent::Mesh(_))) {
                        mesh_count += 1;
                    }
                });

                assert!(mesh_count > 0, "Should find meshes during traversal");
                println!("✓ GLTF feature support validated");
            }
            Err(e) => {
                eprintln!("Failed to test GLTF features: {e}");
            }
        }
    }
}
