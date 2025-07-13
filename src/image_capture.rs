//! Image capture functionality for headless rendering and testing
//!
//! This module provides functionality to capture rendered frames to image files
//! for testing and verification purposes in headless mode.

use crate::resource_manager::{Handle, ResourceManager};
use std::path::Path;

/// Errors that can occur during image capture operations
#[derive(Debug)]
pub enum ImageCaptureError {
    InvalidTexture(String),
    MappingFailed(String),
    WriteFailed(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for ImageCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageCaptureError::InvalidTexture(msg) => write!(f, "Invalid texture: {msg}"),
            ImageCaptureError::MappingFailed(msg) => write!(f, "Buffer mapping failed: {msg}"),
            ImageCaptureError::WriteFailed(msg) => write!(f, "Image write failed: {msg}"),
            ImageCaptureError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
        }
    }
}

impl std::error::Error for ImageCaptureError {}

/// Supported image output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
}

impl ImageFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Bmp => "bmp",
        }
    }

    /// Parse format from string
    pub fn from_str(s: &str) -> Result<Self, ImageCaptureError> {
        match s.to_lowercase().as_str() {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "bmp" => Ok(ImageFormat::Bmp),
            _ => Err(ImageCaptureError::UnsupportedFormat(format!(
                "Unsupported format: {s}"
            ))),
        }
    }
}

/// Image capture utility for headless rendering
pub struct ImageCapture {
    staging_buffer_id: Option<crate::resource_manager::HandleId>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl ImageCapture {
    /// Create a new image capture instance
    pub fn new(width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        Self {
            staging_buffer_id: None,
            width,
            height,
            format,
        }
    }

    /// Initialize the staging buffer for texture copying
    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &mut ResourceManager,
    ) -> Result<(), ImageCaptureError> {
        // Calculate bytes per pixel based on format
        let bytes_per_pixel = match self.format {
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => {
                return Err(ImageCaptureError::UnsupportedFormat(format!(
                    "Texture format {:?} not supported for image capture",
                    self.format
                )))
            }
        };

        // Calculate buffer size with alignment
        let unpadded_bytes_per_row = self.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;
        let buffer_size = (padded_bytes_per_row * self.height) as u64;

        // Create staging buffer for copying texture data
        let staging_buffer = resource_manager.create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("Image Capture Staging Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            },
        );

        self.staging_buffer_id = Some(staging_buffer.id());
        Ok(())
    }

    /// Capture a texture to the staging buffer
    pub fn capture_texture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture_handle: Handle<wgpu::Texture>,
        resource_manager: &ResourceManager,
    ) -> Result<(), ImageCaptureError> {
        let staging_buffer_handle = self
            .staging_buffer()
            .ok_or_else(|| ImageCaptureError::InvalidTexture("Staging buffer not initialized".to_string()))?;

        let texture = resource_manager
            .get_texture(texture_handle)
            .map_err(|e| ImageCaptureError::InvalidTexture(e.to_string()))?;

        let staging_buffer_resource = resource_manager
            .get_buffer(staging_buffer_handle)
            .map_err(|e| ImageCaptureError::InvalidTexture(e.to_string()))?;

        // Calculate bytes per pixel and row padding
        let bytes_per_pixel = match self.format {
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => {
                return Err(ImageCaptureError::UnsupportedFormat(format!(
                    "Texture format {:?} not supported",
                    self.format
                )))
            }
        };

        let unpadded_bytes_per_row = self.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;

        // Copy texture to staging buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: staging_buffer_resource,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }

    /// Read the captured data and save it to a file
    pub async fn save_to_file<P: AsRef<Path>>(
        &self,
        device: &wgpu::Device,
        resource_manager: &ResourceManager,
        path: P,
        format: ImageFormat,
    ) -> Result<(), ImageCaptureError> {
        let staging_buffer_handle = self
            .staging_buffer_id
            .ok_or_else(|| ImageCaptureError::InvalidTexture("Staging buffer not initialized".to_string()))
            .map(|id| Handle::from_id(id))?;

        let staging_buffer_resource = resource_manager
            .get_buffer(staging_buffer_handle)
            .map_err(|e| ImageCaptureError::InvalidTexture(e.to_string()))?;

        // Map the buffer for reading
        let buffer_slice = staging_buffer_resource.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);

        receiver
            .await
            .map_err(|e| ImageCaptureError::MappingFailed(format!("Channel error: {e}")))?
            .map_err(|e| ImageCaptureError::MappingFailed(format!("Buffer mapping error: {e:?}")))?;

        // Read the data
        let data = buffer_slice.get_mapped_range();

        // Calculate image parameters
        let bytes_per_pixel = match self.format {
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => {
                return Err(ImageCaptureError::UnsupportedFormat(format!(
                    "Texture format {:?} not supported",
                    self.format
                )))
            }
        };

        let unpadded_bytes_per_row = self.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;

        // Extract unpadded data
        let mut image_data = Vec::with_capacity((self.width * self.height * bytes_per_pixel) as usize);
        for row in 0..self.height {
            let start = (row * padded_bytes_per_row) as usize;
            let end = start + unpadded_bytes_per_row as usize;
            if end <= data.len() {
                image_data.extend_from_slice(&data[start..end]);
            }
        }

        // Convert BGRA to RGBA if needed
        let final_data = if matches!(
            self.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        ) && bytes_per_pixel == 4
        {
            // Convert BGRA to RGBA
            let mut rgba_data = Vec::with_capacity(image_data.len());
            for chunk in image_data.chunks_exact(4) {
                rgba_data.push(chunk[2]); // R
                rgba_data.push(chunk[1]); // G
                rgba_data.push(chunk[0]); // B
                rgba_data.push(chunk[3]); // A
            }
            rgba_data
        } else {
            image_data
        };

        drop(data); // Unmap the buffer

        // Create image and save
        let _color_type = image::ColorType::Rgba8;

        let img = image::ImageBuffer::from_raw(self.width, self.height, final_data)
            .ok_or_else(|| ImageCaptureError::WriteFailed("Failed to create image buffer".to_string()))?;

        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        match format {
            ImageFormat::Png => dynamic_img
                .save_with_format(path, image::ImageFormat::Png)
                .map_err(|e| ImageCaptureError::WriteFailed(e.to_string()))?,
            ImageFormat::Jpeg => dynamic_img
                .save_with_format(path, image::ImageFormat::Jpeg)
                .map_err(|e| ImageCaptureError::WriteFailed(e.to_string()))?,
            ImageFormat::Bmp => dynamic_img
                .save_with_format(path, image::ImageFormat::Bmp)
                .map_err(|e| ImageCaptureError::WriteFailed(e.to_string()))?,
        }

        Ok(())
    }

    /// Get the staging buffer handle if initialized
    pub fn staging_buffer(&self) -> Option<Handle<wgpu::Buffer>> {
        self.staging_buffer_id.map(|id| Handle::from_id(id))
    }

    /// Get the capture dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the texture format
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_parsing() {
        assert_eq!(ImageFormat::from_str("png").unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::from_str("PNG").unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::from_str("jpg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_str("jpeg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_str("bmp").unwrap(), ImageFormat::Bmp);
        assert!(ImageFormat::from_str("tiff").is_err());
    }

    #[test]
    fn test_image_format_extensions() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Bmp.extension(), "bmp");
    }

    #[test]
    fn test_image_capture_creation() {
        let capture = ImageCapture::new(800, 600, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(capture.dimensions(), (800, 600));
        assert_eq!(capture.format(), wgpu::TextureFormat::Rgba8UnormSrgb);
        assert!(capture.staging_buffer().is_none());
    }
}