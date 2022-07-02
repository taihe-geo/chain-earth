use std::{iter, sync::Arc, time::Instant};

use crate::{
    plugins::render::settings::WgpuSettings,
    winit::{window::WindowId, windows::Windows},
    App, Plugin, TypeName,
};
use log::info;
use tracing::{span, Level};
use typename::TypeName;
// use specs::{Read, ReadStorage, System, WorldExt, Write, WriteStorage};
use super::{
    camera::RenderTarget, view::ViewTarget, window_render_plugin::ExtractedWindows,
    WindowRenderPlugin,
};
use specs::prelude::*;
use wgpu::{Adapter, AdapterInfo, Device, Instance, Queue, RenderPipeline, RequestAdapterOptions};
pub struct EmptySystem;
impl<'a> System<'a> for EmptySystem {
    type SystemData = ();
    fn run(&mut self, data: Self::SystemData) {}
}
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
                raw_handle
            };
            // let compatible_surface = Some(surface).as_ref();
            let request_adapter_options = wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: surface.as_ref(),
                ..Default::default()
            };
            let (device, queue, adapter_info, adapter) = pollster::block_on(initialize_renderer(
                &instance,
                &options,
                &request_adapter_options,
            ));
            app.world.insert(device);
            app.world.insert(queue);
            app.world.insert(adapter_info);
            app.world.insert(adapter);
            app.world.insert(instance);
        }
        app.add_add_systems(|dispatcher_builder| {
            dispatcher_builder.add(RenderSystem, RenderSystem::name(), &[]);
        });
        app.add_plugin(WindowRenderPlugin);
    }
}
pub type RenderDevice = Arc<Device>;
pub type RenderQueue = Arc<Queue>;
pub type RenderInstance = Instance;
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

// #[derive(TypeName)]
// pub struct PreparedViewTargetSystem;
// impl<'a> System<'a> for PreparedViewTargetSystem {
//     type SystemData = (
//         Entities<'a>,
//         ReadExpect<'a, ExtractedWindows>,
//         ReadExpect<'a, RenderTarget>,
//         WriteStorage<'a, ViewTarget>,
//     );
//     fn run(
//         &mut self,
//         (s_entities, s_extected_windows, s_render_target, s_view_target): Self::SystemData,
//     ) {
//         for (entity) in s_entities.join() {
//             if let Some(texture_view) = s_render_target.get_texture_view(&s_extected_windows) {
//                 s_view_target.insert(
//                     entity,
//                     ViewTarget {
//                         view: texture_view.clone(),
//                         sampled_target: None,
//                     },
//                 );
//             }
//         }
//     }
// }
#[derive(TypeName)]
pub struct RenderSystem;
impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, ViewTarget>,
        WriteExpect<'a, ExtractedWindows>,
        ReadExpect<'a, RenderDevice>,
        ReadExpect<'a, RenderQueue>,
    );
    fn run(
        &mut self,
        (
            s_entities,
            s_view_target,
            mut s_extracted_windows,
            s_device,
            s_queue,
        ): Self::SystemData,
    ) {
        let scope = span!(Level::DEBUG,"render_system");
        let _enter = scope.enter();
        let mut encoder = s_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        if let Some(texture_view) = s_extracted_windows
            .get(&WindowId::primary())
            .and_then(|window| window.swap_chain_texture.as_ref())
        {
            info!("能拿到主窗口的texture_view");
            let render_pipeline = {
                let shader = s_device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(include_str!("../planet/shader.wgsl").into()),
                });
                let render_pipeline_layout =
                    s_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });
                s_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[wgpu::ColorTargetState {
                            // format: config.format,
                            format: wgpu::TextureFormat::Bgra8UnormSrgb,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                        // or Features::POLYGON_MODE_POINT
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                })
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            // for(entity,view_target) in (s_entities,s_view_target).join(){

            // }
            {
                render_pass.set_pipeline(&render_pipeline);
                render_pass.draw(0..3, 0..1);
            }
        }
        s_queue.submit(iter::once(encoder.finish()));
        info!("已经提交指令入队");
        for window in s_extracted_windows.values_mut() {
            if let Some(texture_view) = window.swap_chain_texture.take() {
                if let Some(surface_texture) = texture_view.take_surface_texture() {
                    surface_texture.present();
                }
            }
        }
    }
}
