use std::{time::Instant,sync::Arc};

use crate::{
    events::{event, Events, ManualEventReader},
    plugins::render::settings::WgpuSettings,
    winit::{
        event::{
            CreateWindow, CursorMoved, RequestRedraw, WindowCloseRequested, WindowCreated,
            WindowResized,
        },
        input::MouseMotion,
        window::{WindowDescriptor, WindowId},
        windows::Windows,
        winit_config::{UpdateMode, WinitSettings},
        winit_windows::WinitWindows,
    },
    App, Plugin,
};
use nalgebra_glm::{DVec2, Vec2};
use specs::{Read, ReadStorage, System, WorldExt, Write, WriteStorage};
use wgpu::{Instance, RequestAdapterOptions, Device, Queue, AdapterInfo};
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let options = WgpuSettings::default();
        app.world.insert(options.clone());
        if let Some(backends) = options.backends {
            let instance = wgpu::Instance::new(backends);
            let surface = {
                let windows = app.world.write_resource::<Windows>();
                let raw_handle = windows.get_primary().map(|window| unsafe {
                    let handle = window.raw_window_handle().get_handle();
                    instance.create_surface(&handle)
                });
                raw_handle
            };
            let request_adapter_options = wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: surface.as_ref(),
                ..Default::default()
            };
            let (device, queue, adapter_info) = pollster::block_on(initialize_renderer(
                &instance,
                &options,
                &request_adapter_options,
            ));
            app.world.insert(device.clone());
            app.world.insert(queue.clone());
            app.world.insert(adapter_info.clone());
            app.world.insert(instance);
            app.world.insert(surface);
        }
    }
}
pub type RenderDevice = Arc<Device>;
pub type RenderQueue = Arc<Queue>;
async fn initialize_renderer(
    instance: &Instance,
    options: &WgpuSettings,
    request_adapter_options: &RequestAdapterOptions<'_>,
) -> (Arc<Device>, RenderQueue, AdapterInfo) {
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
    (RenderDevice::from(device), queue, adapter_info)
}
