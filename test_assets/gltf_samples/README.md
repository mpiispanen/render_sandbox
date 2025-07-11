# glTF Sample Models

This directory contains sample glTF models inspired by the official @KhronosGroup/glTF-Sample-Models repository. These models are used for testing the glTF loading functionality in the render_sandbox project.

## Directory Structure

```
test_assets/gltf_samples/
├── simple/                 # Simple geometric models for basic testing
│   ├── triangle.gltf       # Simple triangle (3 vertices)
│   └── box.gltf           # Simple box/cube (8 vertices)
├── complex/               # Complex models for advanced testing
│   ├── multi_cube.gltf    # Multiple objects in a scene
│   └── hierarchical_scene.gltf  # Scene with node hierarchy
└── README.md             # This file
```

## Simple Models

### triangle.gltf
- **Purpose**: Basic geometric validation
- **Features**: Single triangle mesh with 3 vertices
- **Use Cases**: Minimal rendering tests, basic GLTF parsing validation
- **Vertices**: 3
- **Meshes**: 1

### box.gltf  
- **Purpose**: Simple 3D geometry testing
- **Features**: Cube/box mesh with 8 vertices
- **Use Cases**: 3D coordinate system validation, basic mesh rendering
- **Vertices**: 8
- **Meshes**: 1

## Complex Models

### multi_cube.gltf
- **Purpose**: Multi-object scene testing
- **Features**: Two separate cube meshes positioned in different locations
- **Use Cases**: Scene graph testing, multiple object rendering
- **Vertices**: 8 per cube (16 total)
- **Meshes**: 2
- **Nodes**: 2 (one per cube)

### hierarchical_scene.gltf
- **Purpose**: Scene hierarchy and transform testing
- **Features**: Parent-child node relationships, transforms, scaling
- **Use Cases**: Scene graph traversal, transform matrix validation, hierarchical rendering
- **Vertices**: Variable (triangles and quads)
- **Meshes**: 2 (triangle and quad)
- **Nodes**: 5 (hierarchical structure)
- **Special Features**: 
  - Root node with children
  - Translation transforms
  - Scale transforms
  - Parent-child relationships

## Testing Coverage

These models provide comprehensive testing coverage for:

1. **Basic GLTF Parsing**: All models test fundamental GLTF structure validation
2. **Vertex Data**: Different vertex counts and configurations
3. **Scene Structure**: From simple single-node scenes to complex hierarchies
4. **Transform Systems**: Identity transforms, translations, scaling
5. **Multiple Objects**: Scenes with multiple meshes and nodes
6. **Hierarchical Relationships**: Parent-child node structures

## Usage in Tests

The sample models are used in several test suites:

- `tests/gltf_tests.rs`: Structure validation and parsing tests
- `tests/gltf_integration_tests.rs`: Integration with the rendering system
- `tests/gltf_sample_models_tests.rs`: Comprehensive sample model testing

## Model Format

All models use:
- glTF 2.0 specification
- Embedded binary data (data URIs)
- Minimal required attributes (POSITION)
- No external dependencies (self-contained files)

This ensures maximum compatibility and simplifies testing infrastructure.