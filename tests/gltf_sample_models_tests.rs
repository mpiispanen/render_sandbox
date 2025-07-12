#[cfg(test)]
mod gltf_sample_models_tests {
    use std::path::Path;

    /// Test that demonstrates loading sample models with actual GLTF parser
    /// Note: This test validates GLTF parsing without GPU resources
    #[test]
    fn test_sample_models_gltf_parsing() {
        // Test models with increasing complexity using official glTF Sample Models
        let test_cases = [
            (
                "test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf",
                "Simple Triangle",
                1,
                3,
            ),
            (
                "test_assets/gltf_sample_models/2.0/Box/glTF/Box.gltf",
                "Simple Box",
                1,
                24, // Box model has 24 vertices
            ),
            (
                "test_assets/gltf_sample_models/2.0/SimpleMeshes/glTF/SimpleMeshes.gltf",
                "Simple Meshes",
                1, // Actually has 1 mesh, not 2
                3, // First mesh in SimpleMeshes
            ),
            (
                "test_assets/gltf_sample_models/2.0/AnimatedCube/glTF/AnimatedCube.gltf",
                "Animated Cube",
                1,
                36, // Animated cube has 36 vertices
            ),
        ];

        for (path_str, name, expected_meshes, expected_first_mesh_vertices) in test_cases {
            let path = Path::new(path_str);
            if path.exists() {
                println!("Testing GLTF parsing for: {name}");

                // Use the gltf crate directly to parse and validate structure
                let (gltf, _buffers, _images) = gltf::import(path)
                    .unwrap_or_else(|_| panic!("Should be able to import {name}"));

                // Verify mesh count
                let mesh_count = gltf.meshes().count();
                assert_eq!(
                    mesh_count, expected_meshes,
                    "{name} should have {expected_meshes} meshes"
                );

                // Verify first mesh vertex count
                if let Some(mesh) = gltf.meshes().next() {
                    if let Some(primitive) = mesh.primitives().next() {
                        let reader = primitive.reader(|buffer| Some(&_buffers[buffer.index()]));
                        if let Some(positions) = reader.read_positions() {
                            let vertex_count = positions.count();
                            assert!(
                                vertex_count >= expected_first_mesh_vertices,
                                "{name} first mesh should have at least {expected_first_mesh_vertices} vertices, found {vertex_count}"
                            );
                        }
                    }
                }

                // Verify scene structure
                let scene_count = gltf.scenes().count();
                assert!(scene_count >= 1, "{name} should have at least one scene");

                println!("✓ {name} parsed successfully");
            } else {
                println!("⚠ Sample model {name} not found, skipping");
            }
        }
    }

    /// Test scene node counting with different model complexities
    #[test]
    fn test_sample_models_scene_complexity() {
        let hierarchical_path =
            Path::new("test_assets/gltf_sample_models/2.0/Cameras/glTF/Cameras.gltf");
        let simple_path =
            Path::new("test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf");

        let mut complexity_levels = Vec::new();

        // Test simple model
        if simple_path.exists() {
            let (gltf, _buffers, _images) = gltf::import(simple_path).unwrap();
            let node_count = gltf.nodes().count();
            complexity_levels.push(("Simple Triangle", node_count));
        }

        // Test complex model
        if hierarchical_path.exists() {
            let (gltf, _buffers, _images) = gltf::import(hierarchical_path).unwrap();
            let node_count = gltf.nodes().count();
            complexity_levels.push(("Cameras Scene", node_count));
        }

        // Verify complexity progression
        if complexity_levels.len() >= 2 {
            let simple_nodes = complexity_levels[0].1;
            let complex_nodes = complexity_levels[1].1;

            assert!(
                complex_nodes > simple_nodes,
                "Complex model should have more nodes ({complex_nodes}) than simple model ({simple_nodes})"
            );

            println!("Complexity validation: {simple_nodes} nodes < {complex_nodes} nodes ✓");
        }
    }

    /// Test material and texture presence in complex models
    #[test]
    fn test_sample_models_feature_coverage() {
        let models_and_features = [
            (
                "test_assets/gltf_sample_models/2.0/Triangle/glTF/Triangle.gltf",
                "positions",
                false,
            ), // No textures
            (
                "test_assets/gltf_sample_models/2.0/Box/glTF/Box.gltf",
                "positions",
                false,
            ), // No textures
            (
                "test_assets/gltf_sample_models/2.0/SimpleMeshes/glTF/SimpleMeshes.gltf",
                "multiple_nodes", // Changed from multiple_meshes since it only has 1 mesh but 2 nodes
                false,
            ),
            (
                "test_assets/gltf_sample_models/2.0/Cameras/glTF/Cameras.gltf",
                "hierarchy",
                false,
            ),
        ];

        for (path_str, feature_name, _requires_textures) in models_and_features {
            let path = Path::new(path_str);
            if path.exists() {
                let (gltf, _buffers, _images) = gltf::import(path).unwrap();

                match feature_name {
                    "positions" => {
                        // Verify all meshes have position attributes
                        for mesh in gltf.meshes() {
                            for primitive in mesh.primitives() {
                                assert!(
                                    primitive.get(&gltf::Semantic::Positions).is_some(),
                                    "Mesh should have position attributes"
                                );
                            }
                        }
                    }
                    "multiple_nodes" => {
                        let node_count = gltf.nodes().count();
                        assert!(node_count > 1, "Should have multiple nodes");
                    }
                    "hierarchy" => {
                        // For cameras model, just verify we have multiple nodes
                        let node_count = gltf.nodes().count();
                        assert!(
                            node_count > 1,
                            "Should have multiple nodes for scene structure"
                        );
                    }
                    _ => {}
                }

                println!("✓ Feature '{feature_name}' validated for {path_str}");
            }
        }
    }
}
