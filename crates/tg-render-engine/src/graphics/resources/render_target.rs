use std::num::NonZeroU32;

use crate::graphics::renderer::DEPTH_FORMAT;

/// Used for rendering to a texture instead of to the frame buffer.
/// Supports 2D and 3D textures or cube maps.
pub struct RenderTarget {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    // Depth stuff
    pub depth_texture: Option<wgpu::Texture>,
    pub depth_texture_view: Option<wgpu::TextureView>,

    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        width: f32,
        height: f32,
        depth: u32,
        mip_count: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: depth,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format,
            usage: usage,
            label: None,
        });
        let mut texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        if depth == 6 {
            texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(format),
                dimension: Some(wgpu::TextureViewDimension::Cube),
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: 0,
                mip_level_count: Some(NonZeroU32::new(mip_count).unwrap()),
                base_array_layer: 0,
                array_layer_count: Some(NonZeroU32::new(6).unwrap()),
            });
        }
        Self {
            texture,
            texture_view,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }),
            depth_texture: None,
            depth_texture_view: None,
            width: width as u32,
            height: height as u32,
        }
    }

    pub fn with_depth(&mut self, device: &wgpu::Device) {
        self.depth_texture = Some(device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        }));

        self.depth_texture_view = Some(
            self.depth_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
    }

    pub fn complete(self) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        (self.texture, self.texture_view, self.sampler)
    }
}
