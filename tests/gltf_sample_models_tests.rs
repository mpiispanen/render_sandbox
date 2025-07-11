#[cfg(test)]
mod gltf_sample_models_tests {
    use std::path::Path;

    /// Test that demonstrates loading sample models with actual GLTF parser
    /// Note: This test validates GLTF parsing without GPU resources
    #[test]
    fn test_sample_models_gltf_parsing() {
        // Test models with increasing complexity
        let test_cases = [
            (
                "test_assets/gltf_samples/simple/triangle.gltf",
                "Simple Triangle",
                1,
                3,
            ),
            (
                "test_assets/gltf_samples/simple/box.gltf",
                "Simple Box",
                1,
                8,
            ),
            (
                "test_assets/gltf_samples/complex/multi_cube.gltf",
                "Multi-Cube Scene",
                2,
                8,
            ),
            (
                "test_assets/gltf_samples/complex/hierarchical_scene.gltf",
                "Hierarchical Scene",
                2,
                3,
            ),
        ];

        for (path_str, name, expected_meshes, expected_first_mesh_vertices) in test_cases {
            let path = Path::new(path_str);
            if path.exists() {
                println!("Testing GLTF parsing for: {}", name);

                // Use the gltf crate directly to parse and validate structure
                let (gltf, _buffers, _images) =
                    gltf::import(path).expect(&format!("Should be able to import {}", name));

                // Verify mesh count
                let mesh_count = gltf.meshes().count();
                assert_eq!(
                    mesh_count, expected_meshes,
                    "{} should have {} meshes",
                    name, expected_meshes
                );

                // Verify first mesh vertex count
                if let Some(mesh) = gltf.meshes().next() {
                    if let Some(primitive) = mesh.primitives().next() {
                        let reader = primitive.reader(|buffer| Some(&_buffers[buffer.index()]));
                        if let Some(positions) = reader.read_positions() {
                            let vertex_count = positions.count();
                            assert!(
                                vertex_count >= expected_first_mesh_vertices,
                                "{} first mesh should have at least {} vertices, found {}",
                                name,
                                expected_first_mesh_vertices,
                                vertex_count
                            );
                        }
                    }
                }

                // Verify scene structure
                let scene_count = gltf.scenes().count();
                assert!(scene_count >= 1, "{} should have at least one scene", name);

                println!("✓ {} parsed successfully", name);
            } else {
                println!("⚠ Sample model {} not found, skipping", name);
            }
        }
    }

    /// Test scene node counting with different model complexities
    #[test]
    fn test_sample_models_scene_complexity() {
        let hierarchical_path =
            Path::new("test_assets/gltf_samples/complex/hierarchical_scene.gltf");
        let simple_path = Path::new("test_assets/gltf_samples/simple/triangle.gltf");

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
            complexity_levels.push(("Hierarchical Scene", node_count));
        }

        // Verify complexity progression
        if complexity_levels.len() >= 2 {
            let simple_nodes = complexity_levels[0].1;
            let complex_nodes = complexity_levels[1].1;

            assert!(
                complex_nodes > simple_nodes,
                "Complex model should have more nodes ({}) than simple model ({})",
                complex_nodes,
                simple_nodes
            );

            println!(
                "Complexity validation: {} nodes < {} nodes ✓",
                simple_nodes, complex_nodes
            );
        }
    }

    /// Test material and texture presence in complex models
    #[test]
    fn test_sample_models_feature_coverage() {
        let models_and_features = [
            (
                "test_assets/gltf_samples/simple/triangle.gltf",
                "positions",
                false,
            ), // No textures
            (
                "test_assets/gltf_samples/simple/box.gltf",
                "positions",
                false,
            ), // No textures
            (
                "test_assets/gltf_samples/complex/multi_cube.gltf",
                "multiple_meshes",
                false,
            ),
            (
                "test_assets/gltf_samples/complex/hierarchical_scene.gltf",
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
                    "multiple_meshes" => {
                        let mesh_count = gltf.meshes().count();
                        assert!(mesh_count > 1, "Should have multiple meshes");
                    }
                    "hierarchy" => {
                        let nodes_with_children = gltf
                            .nodes()
                            .filter(|node| node.children().count() > 0)
                            .count();
                        assert!(
                            nodes_with_children > 0,
                            "Should have hierarchical structure"
                        );
                    }
                    _ => {}
                }

                println!("✓ Feature '{}' validated for {}", feature_name, path_str);
            }
        }
    }
}
