use std::sync::Arc;

use specs::{World, WorldExt};
use wgpu::{Queue, Device};
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const FRAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub struct Renderer {
    pub(crate) surface: wgpu::Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    adapter: wgpu::Adapter,
    pub window: winit::window::Window,
}

impl Renderer {
    pub fn get_texture_format(&self)->wgpu::TextureFormat{
        self.surface.get_preferred_format(&self.adapter).unwrap()
    }
    pub(crate) async fn new(
        window: winit::window::Window,
        size: winit::dpi::PhysicalSize<u32>,
        world: &mut World,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();
        world.insert(Arc::new(queue));
        world.insert(Arc::new(device));
        
        Self {
            surface,
            size,
            adapter,
            window,
        }
    }

    pub fn render(&mut self){
        
    }
}
