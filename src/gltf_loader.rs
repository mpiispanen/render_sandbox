use crate::resource_manager::ResourceManager;
use crate::scene::{Camera, Mesh, NodeContent, Scene, SceneNode, Transform};
use cgmath::{Quaternion, Vector3};
use std::path::Path;

/// Errors that can occur during GLTF loading
#[derive(Debug)]
pub enum GltfError {
    IoError(std::io::Error),
    GltfError(gltf::Error),
    ValidationError(String),
    UnsupportedFeature(String),
}

impl std::fmt::Display for GltfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GltfError::IoError(err) => write!(f, "IO error: {err}"),
            GltfError::GltfError(err) => write!(f, "GLTF error: {err}"),
            GltfError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            GltfError::UnsupportedFeature(feature) => write!(f, "Unsupported feature: {feature}"),
        }
    }
}

impl std::error::Error for GltfError {}

impl From<std::io::Error> for GltfError {
    fn from(err: std::io::Error) -> Self {
        GltfError::IoError(err)
    }
}

impl From<gltf::Error> for GltfError {
    fn from(err: gltf::Error) -> Self {
        GltfError::GltfError(err)
    }
}

/// GLTF loader for converting GLTF files to scene format
pub struct GltfLoader;

impl GltfLoader {
    /// Load a GLTF file and add its contents to the scene
    pub fn load_gltf<P: AsRef<Path>>(
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
        path: P,
        scene: &mut Scene,
    ) -> Result<(), GltfError> {
        log::info!("Loading GLTF file: {}", path.as_ref().display());

        let (gltf, buffers, _images) = gltf::import(path)?;

        // Process each scene in the GLTF file
        for gltf_scene in gltf.scenes() {
            log::debug!("Processing GLTF scene: {}", gltf_scene.index());

            // Process each root node in the scene
            for node in gltf_scene.nodes() {
                let scene_node = Self::process_node(device, resource_manager, &node, &buffers)?;
                scene.add_node(scene_node);
            }
        }

        log::info!("Successfully loaded GLTF file");
        Ok(())
    }

    /// Process a GLTF node and convert it to our scene format
    fn process_node(
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
        gltf_node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
    ) -> Result<SceneNode, GltfError> {
        log::debug!("Processing GLTF node: {}", gltf_node.index());

        // Create the scene node
        let mut scene_node = if let Some(name) = gltf_node.name() {
            SceneNode::with_name(name)
        } else {
            SceneNode::new()
        };

        // Set transform
        scene_node.transform = Self::convert_transform(gltf_node);

        // Process node content
        if let Some(gltf_mesh) = gltf_node.mesh() {
            let mesh = Self::process_mesh(device, resource_manager, &gltf_mesh, buffers)?;
            scene_node.content = Some(NodeContent::Mesh(mesh));
        } else if let Some(gltf_camera) = gltf_node.camera() {
            let camera = Self::process_camera(&gltf_camera)?;
            scene_node.content = Some(NodeContent::Camera(camera));
        }
        // Note: GLTF lights are extensions, not in core spec, so we'll skip for now

        Ok(scene_node)
    }

    /// Convert GLTF transform to our transform format
    fn convert_transform(gltf_node: &gltf::Node) -> Transform {
        let (translation, rotation, scale) = gltf_node.transform().decomposed();

        Transform {
            position: Vector3::new(translation[0], translation[1], translation[2]),
            rotation: Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]),
            scale: Vector3::new(scale[0], scale[1], scale[2]),
        }
    }

    /// Process a GLTF mesh and convert it to our mesh format
    fn process_mesh(
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
        gltf_mesh: &gltf::Mesh,
        buffers: &[gltf::buffer::Data],
    ) -> Result<Mesh, GltfError> {
        log::debug!("Processing GLTF mesh: {}", gltf_mesh.index());

        // For now, we'll just process the first primitive of the mesh
        let primitive = gltf_mesh
            .primitives()
            .next()
            .ok_or_else(|| GltfError::ValidationError("Mesh has no primitives".to_string()))?;

        // Get vertex data
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        // Read positions (required)
        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or_else(|| {
                GltfError::ValidationError("Mesh primitive has no positions".to_string())
            })?
            .collect();

        // Convert to flat vertex array (just positions for now)
        let vertices: Vec<f32> = positions
            .iter()
            .flat_map(|pos| pos.iter())
            .copied()
            .collect();

        // Read indices if available
        let indices: Option<Vec<u16>> = reader
            .read_indices()
            .map(|iter| {
                iter.into_u32()
                    .map(|i| {
                        if i > u16::MAX as u32 {
                            return Err(GltfError::ValidationError(format!(
                                "Index value {i} exceeds maximum u16 value (65535)"
                            )));
                        }
                        Ok(i as u16)
                    })
                    .collect::<Result<Vec<u16>, GltfError>>()
            })
            .transpose()?;

        // Create the mesh using the resource manager
        // Create vertex buffer
        let vertex_buffer = resource_manager.create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("GLTF Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        // Create index buffer if indices exist
        let (index_buffer, index_count) = if let Some(ref indices) = indices {
            let buffer = resource_manager.create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("GLTF Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            );
            (Some(buffer), Some(indices.len() as u32))
        } else {
            (None, None)
        };

        Ok(Mesh {
            vertex_buffer,
            index_buffer,
            vertex_count: positions.len() as u32,
            index_count,
        })
    }

    /// Process a GLTF camera and convert it to our camera format
    fn process_camera(gltf_camera: &gltf::Camera) -> Result<Camera, GltfError> {
        log::debug!("Processing GLTF camera: {}", gltf_camera.index());

        match gltf_camera.projection() {
            gltf::camera::Projection::Perspective(perspective) => Ok(Camera::perspective(
                perspective.yfov().to_degrees(),
                perspective.aspect_ratio().unwrap_or(16.0 / 9.0),
                perspective.znear(),
                perspective.zfar().unwrap_or(1000.0),
            )),
            gltf::camera::Projection::Orthographic(_) => Err(GltfError::UnsupportedFeature(
                "Orthographic cameras".to_string(),
            )),
        }
    }

    /// Create a simple test GLTF triangle mesh directly (for testing)
    pub fn create_test_triangle(
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
    ) -> Result<Mesh, GltfError> {
        log::info!("Creating test triangle from GLTF-style data");

        // Simple triangle vertices (similar to current test triangle)
        #[rustfmt::skip]
        let vertices = [
            0.0, 0.5, 0.0,   // Top
            -0.5, -0.5, 0.0, // Bottom left
            0.5, -0.5, 0.0,  // Bottom right
        ];

        let vertex_buffer = resource_manager.create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("GLTF Test Triangle Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        Ok(Mesh {
            vertex_buffer,
            index_buffer: None,
            vertex_count: 3,
            index_count: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gltf_error_display() {
        let err = GltfError::ValidationError("test error".to_string());
        assert_eq!(format!("{err}"), "Validation error: test error");
    }

    #[test]
    fn test_gltf_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let gltf_err = GltfError::from(io_err);
        assert!(matches!(gltf_err, GltfError::IoError(_)));
    }
}
