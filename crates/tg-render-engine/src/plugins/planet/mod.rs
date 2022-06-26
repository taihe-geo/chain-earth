use crate::{
    app::Bundler,
    winit::window::{PresentMode, WindowId},
    winit::windows::Windows,
    App, Plugin,
};
use nalgebra_glm as glm;
use specs::WorldExt;
use wgpu::Device;

use super::render::window_render_plugin::ExtractedWindows;
pub mod entity;
pub struct PlanetPluginParameter {
    radius: glm::Vec3,
}
pub struct PlanetPlugin {
    radius: glm::Vec3,
}
impl PlanetPlugin {
    pub fn new(params: PlanetPluginParameter) -> Self {
        Self {
            radius: params.radius,
        }
    }
}
impl Default for PlanetPlugin {
    fn default() -> Self {
        let params = PlanetPluginParameter {
            radius: [1., 1., 1.].into(),
        };
        Self::new(params)
    }
}
impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .create_entity_with_bundle(entity::PlanetBundle::new(self.radius));
        let device = app.world.read_resource::<Device>();
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let windows = app.world.read_resource::<ExtractedWindows>();
        let primary_window = windows.get(&WindowId::primary()).expect("缺少主窗口");
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        });
    }
}
