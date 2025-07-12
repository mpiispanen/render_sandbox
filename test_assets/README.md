# glTF Sample Models Integration

This directory contains the official KhronosGroup glTF Sample Models repository as a Git submodule. This provides access to a comprehensive set of glTF test models for validating the render_sandbox glTF loading functionality.

## Repository Information

- **Repository**: [KhronosGroup/glTF-Sample-Models](https://github.com/KhronosGroup/glTF-Sample-Models)
- **Purpose**: Official glTF 2.0 sample models for testing and validation
- **Location**: `test_assets/gltf_sample_models/`

## Models Used in Tests

The render_sandbox test suite uses the following models from the glTF 2.0 collection:

### Simple Models (`2.0/*/glTF/`)

1. **Triangle** (`Triangle/glTF/Triangle.gltf`)
   - **Purpose**: Basic triangle geometry for minimal rendering tests
   - **Vertices**: 3
   - **Meshes**: 1
   - **Use Case**: Fundamental geometry parsing validation

2. **Box** (`Box/glTF/Box.gltf`)
   - **Purpose**: Simple cube geometry for 3D coordinate validation
   - **Vertices**: 24 (6 faces × 4 vertices each)
   - **Meshes**: 1
   - **Use Case**: Basic 3D model structure validation

### Complex Models

3. **SimpleMeshes** (`SimpleMeshes/glTF/SimpleMeshes.gltf`)
   - **Purpose**: Multi-node scene testing
   - **Vertices**: 3 (first mesh)
   - **Meshes**: 1
   - **Nodes**: 2
   - **Use Case**: Scene graph with multiple nodes

4. **Cameras** (`Cameras/glTF/Cameras.gltf`)
   - **Purpose**: Multiple camera nodes for scene structure testing
   - **Vertices**: 4 (mesh vertices)
   - **Meshes**: 1
   - **Nodes**: 3 (multiple camera nodes)
   - **Use Case**: Complex scene structure validation

5. **AnimatedCube** (`AnimatedCube/glTF/AnimatedCube.gltf`)
   - **Purpose**: Animation and complex geometry testing
   - **Vertices**: 36
   - **Meshes**: 1
   - **Use Case**: Animation data validation

## Test Coverage

The integration provides comprehensive test coverage across multiple test suites:

### gltf_sample_models_tests.rs
- `test_sample_models_gltf_parsing()` - Validates parsing with actual glTF data
- `test_sample_models_scene_complexity()` - Tests node complexity progression
- `test_sample_models_feature_coverage()` - Validates specific glTF features

### gltf_integration_tests.rs
- `test_gltf_sample_models_structure_validation()` - Structural integrity tests
- `test_gltf_sample_models_scene_hierarchy()` - Scene graph validation
- `test_gltf_sample_models_comparison()` - Model comparison tests

### gltf_tests.rs
- `test_gltf_sample_models_simple()` - Simple model validation
- `test_gltf_sample_models_complex()` - Complex model validation
- `test_gltf_sample_model_vertex_counts()` - Vertex count verification
- `test_gltf_sample_model_scene_structure()` - Scene structure tests

## Submodule Management

### Initial Setup
```bash
git submodule add https://github.com/KhronosGroup/glTF-Sample-Models.git test_assets/gltf_sample_models
```

### Updating Submodule
```bash
cd test_assets/gltf_sample_models
git pull origin main
cd ../..
git add test_assets/gltf_sample_models
git commit -m "Update glTF Sample Models submodule"
```

### Cloning Repository with Submodules
```bash
git clone --recursive https://github.com/mpiispanen/render_sandbox.git
# OR
git clone https://github.com/mpiispanen/render_sandbox.git
cd render_sandbox
git submodule update --init --recursive
```

## Benefits of Submodule Approach

1. **Official Models**: Uses the authoritative glTF sample models from Khronos Group
2. **Comprehensive Coverage**: Access to 80+ models for various testing scenarios
3. **Up-to-date**: Can easily update to latest model versions
4. **Standards Compliance**: Ensures compatibility with official glTF specifications
5. **Reduced Maintenance**: No need to maintain custom models

## Model Selection Rationale

The selected models provide progressive complexity testing:

- **Triangle**: Minimal valid glTF (3 vertices, 1 mesh)
- **Box**: Standard geometric primitive (24 vertices, proper normals)
- **SimpleMeshes**: Multi-node scenes (scene graph testing)
- **Cameras**: Complex node structures (multiple cameras, transforms)
- **AnimatedCube**: Animation data (future animation system testing)

This progression allows testing from basic parsing to complex scene hierarchy validation.