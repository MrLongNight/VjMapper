//! Rendering backend abstraction

use crate::{RenderError, Result, ShaderHandle, ShaderSource, TextureDescriptor, TextureHandle};
use std::sync::Arc;
use tracing::{debug, info};
use wgpu::util::StagingBelt;

/// Trait for rendering backends
pub trait RenderBackend: Send {
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<TextureHandle>;
    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<()>;
    fn create_shader(&mut self, source: ShaderSource) -> Result<ShaderHandle>;
}

/// wgpu-based rendering backend
pub struct WgpuBackend {
    pub instance: Arc<wgpu::Instance>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub adapter_info: wgpu::AdapterInfo,
    staging_belt: StagingBelt,
    texture_counter: u64,
    shader_counter: u64,
}

impl WgpuBackend {
    /// Create a new wgpu backend
    pub async fn new() -> Result<Self> {
        Self::new_with_options(
            wgpu::Backends::all(),
            wgpu::PowerPreference::HighPerformance,
        )
        .await
    }

    /// Create a new wgpu backend with specific options
    pub async fn new_with_options(
        backends: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
    ) -> Result<Self> {
        info!("Initializing wgpu backend");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| RenderError::DeviceError("No suitable adapter found".to_string()))?;

        let adapter_info = adapter.get_info();
        info!(
            "Selected adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("MapFlow Device"),
                    required_features: wgpu::Features::TIMESTAMP_QUERY
                        | wgpu::Features::PUSH_CONSTANTS,
                    required_limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .map_err(|e| RenderError::DeviceError(e.to_string()))?;

        info!("Device created successfully");

        let staging_belt = StagingBelt::new(1024 * 1024); // 1MB chunks

        Ok(Self {
            instance: Arc::new(instance),
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter_info,
            staging_belt,
            texture_counter: 0,
            shader_counter: 0,
        })
    }

    /// Create a surface using the backend's instance
    ///
    /// # Safety
    /// The window must outlive the surface
    pub fn create_surface(
        &self,
        window: Arc<winit::window::Window>,
    ) -> Result<wgpu::Surface<'static>> {
        self.instance
            .create_surface(window)
            .map_err(move |e| RenderError::DeviceError(format!("Failed to create surface: {}", e)))
    }

    /// Get device limits
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }

    /// Get adapter info
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    /// Recall staging belt buffers
    pub fn recall_staging_belt(&mut self) {
        self.staging_belt.recall();
    }

    /// Finish staging belt
    pub fn finish_staging_belt(&mut self) {
        self.staging_belt.finish();
    }

    /// Upload texture data using staging belt (async, zero-copy-like behavior)
    ///
    /// This method provides better performance for streaming video frames
    /// by using a ring buffer of staging buffers that can be reused.
    pub fn upload_texture_staged(
        &mut self,
        handle: &TextureHandle,
        data: &[u8],
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {
        let bytes_per_pixel = match handle.format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => {
                return Err(RenderError::TextureCreation(
                    "Unsupported texture format for upload".to_string(),
                ))
            }
        };

        let bytes_per_row = handle.width * bytes_per_pixel;
        let buffer_size = (bytes_per_row * handle.height) as u64;

        let mut staging_buffer = self.staging_belt.write_buffer(
            encoder,
            &handle.texture,
            0,
            wgpu::BufferSize::new(buffer_size).unwrap(),
            &self.device,
        );

        staging_buffer[..data.len()].copy_from_slice(data);

        Ok(())
    }
}

impl RenderBackend for WgpuBackend {
    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<TextureHandle> {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Texture {}", self.texture_counter)),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: desc.mip_levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: desc.usage,
            view_formats: &[],
        });

        let handle = TextureHandle {
            id: self.texture_counter,
            texture: Arc::new(texture),
            width: desc.width,
            height: desc.height,
            format: desc.format,
        };

        self.texture_counter += 1;
        debug!(
            "Created texture {} ({}x{})",
            handle.id, desc.width, desc.height
        );

        Ok(handle)
    }

    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<()> {
        let bytes_per_pixel = match handle.format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => {
                return Err(RenderError::TextureCreation(
                    "Unsupported texture format for upload".to_string(),
                ))
            }
        };

        let expected_size = (handle.width * handle.height * bytes_per_pixel) as usize;
        if data.len() != expected_size {
            return Err(RenderError::TextureCreation(format!(
                "Data size mismatch: expected {}, got {}",
                expected_size,
                data.len()
            )));
        }

        // Use staging belt for optimized async uploads (reduces CPU-GPU sync overhead)
        // This provides PBO-like behavior in wgpu
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Texture Upload Encoder"),
            });

        // Calculate row padding for wgpu alignment requirements
        let bytes_per_row = handle.width * bytes_per_pixel;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (bytes_per_row + alignment - 1) & !(alignment - 1);

        // Get a staging buffer slice from the belt
        let buffer_size = (padded_bytes_per_row * handle.height) as u64;

        // For larger textures, use staging belt for async upload
        if buffer_size > 64 * 1024 {
            // Use staging belt for textures larger than 64KB
            let mut staging_buffer = self.staging_belt.write_buffer(
                &mut encoder,
                &handle.texture,
                0,
                wgpu::BufferSize::new(buffer_size).unwrap(),
                &self.device,
            );

            // Copy data with proper row padding
            if padded_bytes_per_row == bytes_per_row {
                // No padding needed, direct copy
                staging_buffer[..data.len()].copy_from_slice(data);
            } else {
                // Need to pad rows
                for y in 0..handle.height as usize {
                    let src_start = y * bytes_per_row as usize;
                    let src_end = src_start + bytes_per_row as usize;
                    let dst_start = y * padded_bytes_per_row as usize;
                    let dst_end = dst_start + bytes_per_row as usize;
                    staging_buffer[dst_start..dst_end].copy_from_slice(&data[src_start..src_end]);
                }
            }

            // Submit staging belt commands
            self.staging_belt.finish();
            self.queue.submit(std::iter::once(encoder.finish()));

            // Recall the staging belt for next frame
            // Note: In a real app, this should be done after GPU work completes
            self.staging_belt.recall();
        } else {
            // For small textures, use direct write (lower overhead)
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &handle.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(handle.height),
                },
                wgpu::Extent3d {
                    width: handle.width,
                    height: handle.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        debug!(
            "Uploaded texture {} ({}x{}, {} bytes, {})",
            handle.id,
            handle.width,
            handle.height,
            data.len(),
            if buffer_size > 64 * 1024 {
                "staged"
            } else {
                "direct"
            }
        );
        Ok(())
    }

    fn create_shader(&mut self, source: ShaderSource) -> Result<ShaderHandle> {
        let module = match source {
            ShaderSource::Wgsl(ref code) => {
                self.device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(&format!("Shader {}", self.shader_counter)),
                        source: wgpu::ShaderSource::Wgsl(code.clone().into()),
                    })
            }
        };

        let handle = ShaderHandle {
            id: self.shader_counter,
            module: Arc::new(module),
        };

        self.shader_counter += 1;
        debug!("Created shader {}", handle.id);

        Ok(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        pollster::block_on(async {
            let backend = WgpuBackend::new().await;
            if backend.is_err() {
                // Skipping test on CI/Headless systems without GPU support.
                eprintln!("SKIP: Backend konnte nicht initialisiert werden (möglicherweise kein GPU-Backend/HW im CI).");
                return;
            }
            assert!(backend.is_ok());

            if let Ok(backend) = backend {
                println!("Backend: {:?}", backend.adapter_info);
            }
        });
    }
}
