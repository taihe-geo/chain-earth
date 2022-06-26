use std::{sync::Arc, time::Instant};

use crate::{plugins::render::settings::WgpuSettings, winit::windows::Windows, App, Plugin};
use specs::{Read, ReadStorage, System, WorldExt, Write, WriteStorage};
use wgpu::{Adapter, AdapterInfo, Device, Instance, Queue, RequestAdapterOptions};

use super::WindowRenderPlugin;
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let options = WgpuSettings::default();
        app.world.insert(options.clone());
        if let Some(backends) = options.backends {
            let instance = wgpu::Instance::new(backends);
            // let mut w:f32=0.0;
            // let mut h:f32=0.0;
            let surface = {
                let windows = app.world.write_resource::<Windows>();
                let raw_handle = windows.get_primary().map(|window| unsafe {
                    let handle = window.raw_window_handle().get_handle();
                    instance.create_surface(&handle)
                });
                raw_handle.unwrap()
            };
            let request_adapter_options = wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: Some(surface).as_ref(),
                ..Default::default()
            };
            let (device, queue, adapter_info,adapter) = pollster::block_on(initialize_renderer(
                &instance,
                &options,
                &request_adapter_options,
            ));
            // let config = wgpu::SurfaceConfiguration {
            //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            //     format: surface.get_preferred_format(&adapter).unwrap(),
            //     width: size.width,
            //     height: size.height,
            //     present_mode: wgpu::PresentMode::Fifo,
            // };
            app.world.insert(device);
            app.world.insert(queue);
            app.world.insert(adapter_info);
            app.world.insert(adapter);
            app.world.insert(instance);
            app.world.insert(surface);
        }
        app.add_plugin(WindowRenderPlugin);
    }
}
pub type RenderDevice = Arc<Device>;
pub type RenderQueue = Arc<Queue>;
async fn initialize_renderer(
    instance: &Instance,
    options: &WgpuSettings,
    request_adapter_options: &RequestAdapterOptions<'_>,
) -> (Arc<Device>, RenderQueue, AdapterInfo, Adapter) {
    let adapter = instance
        .request_adapter(request_adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    let adapter_info = adapter.get_info();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: options.device_label.as_ref().map(|a| a.as_ref()),
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .unwrap();
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    (RenderDevice::from(device), queue, adapter_info, adapter)
}
