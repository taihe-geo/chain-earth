use crate::{
    winit::{
        raw_window_handle::RawWindowHandleWrapper,
        window::{PresentMode, WindowId},
        windows::Windows,
    },
    App, HashMap, HashSet, Plugin, TypeName,
};
use log::debug;
use specs::{Read, ReadExpect, System, Write, WriteExpect};
use std::{
    any::{type_name, TypeId},
    ops::{Deref, DerefMut},
};
use tracing::{info, instrument};
use typename::TypeName;
use wgpu::{Instance, TextureFormat};

use super::{texture::TextureView, RenderDevice};
#[derive(Default)]
pub struct NonSendMarker;

pub struct WindowRenderPlugin;
impl Plugin for WindowRenderPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert(ExtractedWindows::default());
        app.world.insert(WindowSurfaces::default());
        app.add_add_systems(|dispatcher_builder| {
            dispatcher_builder.add(ExtractWindowSystem, ExtractWindowSystem::name(), &[]);
            dispatcher_builder.add(
                PrepareWindowsSystem,
                PrepareWindowsSystem::name(),
                &[ExtractWindowSystem::name()],
            );
        });
    }
}

pub struct ExtractedWindow {
    pub id: WindowId,
    pub handle: RawWindowHandleWrapper,
    pub physical_width: u32,
    pub physical_height: u32,
    pub present_mode: PresentMode,
    pub swap_chain_texture: Option<TextureView>,
    pub size_changed: bool,
}

#[derive(Default)]
pub struct ExtractedWindows {
    pub windows: HashMap<WindowId, ExtractedWindow>,
}

impl Deref for ExtractedWindows {
    type Target = HashMap<WindowId, ExtractedWindow>;

    fn deref(&self) -> &Self::Target {
        &self.windows
    }
}

impl DerefMut for ExtractedWindows {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.windows
    }
}
#[derive(TypeName)]
pub struct ExtractWindowSystem;
// impl TypeName for ExtractWindowSystem {}
impl<'a> System<'a> for ExtractWindowSystem {
    type SystemData = (ReadExpect<'a, Windows>, WriteExpect<'a, ExtractedWindows>);
    fn run(&mut self, (s_windows, mut s_extracted_windows): Self::SystemData) {
        for window in s_windows.iter() {
            let (new_width, new_height) = (
                window.physical_width().max(1),
                window.physical_height().max(1),
            );

            let mut extracted_window =
                s_extracted_windows
                    .entry(window.id())
                    .or_insert(ExtractedWindow {
                        id: window.id(),
                        handle: window.raw_window_handle(),
                        physical_width: new_width,
                        physical_height: new_height,
                        present_mode: window.present_mode(),
                        swap_chain_texture: None,
                        size_changed: false,
                    });

            // NOTE: Drop the swap chain frame here
            extracted_window.swap_chain_texture = None;
            extracted_window.size_changed = new_width != extracted_window.physical_width
                || new_height != extracted_window.physical_height;

            if extracted_window.size_changed {
                extracted_window.physical_width = new_width;
                extracted_window.physical_height = new_height;
            }
        }
    }
}
#[derive(Default)]
pub struct WindowSurfaces {
    surfaces: HashMap<WindowId, wgpu::Surface>,
    /// List of windows that we have already called the initial `configure_surface` for
    configured_windows: HashSet<WindowId>,
}
#[derive(TypeName)]
pub struct PrepareWindowsSystem;
impl<'a> System<'a> for PrepareWindowsSystem {
    type SystemData = (
        ReadExpect<'a, Windows>,
        WriteExpect<'a, ExtractedWindows>,
        WriteExpect<'a, WindowSurfaces>,
        ReadExpect<'a, RenderDevice>,
        ReadExpect<'a, Instance>,
    );
    fn run(
        &mut self,
        (
            s_windows,
            mut s_extracted_windows,
            mut s_window_surfaces,
            s_render_device,
            s_render_instance,
        ): Self::SystemData,
    ) {
        info!("prepare window system running");
        let window_surfaces = s_window_surfaces.deref_mut();
        for window in s_extracted_windows.windows.values_mut() {
            let surface = window_surfaces
                .surfaces
                .entry(window.id)
                .or_insert_with(|| unsafe {
                    // NOTE: On some OSes this MUST be called from the main thread.
                    s_render_instance.create_surface(&window.handle.get_handle())
                });

            let swap_chain_descriptor = wgpu::SurfaceConfiguration {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: window.physical_width,
                height: window.physical_height,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                present_mode: match window.present_mode {
                    PresentMode::Fifo => wgpu::PresentMode::Fifo,
                    PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
                    PresentMode::Immediate => wgpu::PresentMode::Immediate,
                },
            };

            // Do the initial surface configuration if it hasn't been configured yet
            if window_surfaces.configured_windows.insert(window.id) || window.size_changed {
                // s_render_device.configure_surface(surface, &swap_chain_descriptor);
                surface.configure(&s_render_device, &swap_chain_descriptor);
            }

            let frame = match surface.get_current_texture() {
                Ok(swap_chain_frame) => swap_chain_frame,
                Err(wgpu::SurfaceError::Outdated) => {
                    // s_render_device.configure_surface(surface, &swap_chain_descriptor);
                    surface.configure(&s_render_device, &swap_chain_descriptor);
                    surface
                        .get_current_texture()
                        .expect("Error reconfiguring surface")
                }
                err => {
                    info!("some error,{:?}", err);
                    err.expect("Failed to acquire next swap chain texture!")
                }
            };

            // window.swap_chain_texture = Some(
            //     frame
            //         .texture
            //         .create_view(&wgpu::TextureViewDescriptor::default()),
            // );
            window.swap_chain_texture = Some(TextureView::from(frame));
        }
    }
}
