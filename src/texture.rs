use std::{ops::Deref, sync::Arc};

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Pod, Zeroable)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(PartialEq)]
pub struct Texture {
    pub pixels: Vec<Pixel>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn white() -> Self {
        Self {
            pixels: vec![Pixel::WHITE],
            width: 1,
            height: 1,
        }
    }

    pub fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.pixels)
    }

    pub fn create_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("basil-texture"),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                sample_count: 1,
                mip_level_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            },
            self.data(),
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct SharedTexture {
    texture: Arc<Texture>,
}

impl SharedTexture {
    pub fn new(texture: Texture) -> Self {
        Self {
            texture: Arc::new(texture),
        }
    }
}

impl Deref for SharedTexture {
    type Target = Texture;

    fn deref(&self) -> &Self::Target {
        self.texture.as_ref()
    }
}
