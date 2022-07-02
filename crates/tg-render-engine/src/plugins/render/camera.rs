use nalgebra_glm::{UVec2, Vec2};

use crate::winit::{window::WindowId, windows::Windows};

use super::{window_render_plugin::ExtractedWindows, texture::TextureView};

#[derive(Debug, Clone,  PartialEq, Eq, Hash)]
pub enum RenderTarget {
    /// Window to which the camera's view is rendered.
    Window(WindowId),
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Window(Default::default())
    }
}

impl RenderTarget {
    pub fn get_texture_view<'a>(
        &self,
        windows: &'a ExtractedWindows,
    ) -> Option<&'a TextureView> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(window_id)
                .and_then(|window| window.swap_chain_texture.as_ref()),
            // RenderTarget::Image(image_handle) => {
            //     images.get(image_handle).map(|image| &image.texture_view)
            // }
        }
    }
    pub fn get_physical_size(&self, windows: &Windows,) -> Option<UVec2> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(*window_id)
                .map(|window| UVec2::new(window.physical_width(), window.physical_height())),
            // RenderTarget::Image(image_handle) => images.get(image_handle).map(|image| {
            //     let Extent3d { width, height, .. } = image.texture_descriptor.size;
            //     UVec2::new(width, height)
            // }),
        }
    }
    pub fn get_logical_size(&self, windows: &Windows,) -> Option<Vec2> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(*window_id)
                .map(|window| Vec2::new(window.width(), window.height())),
            // RenderTarget::Image(image_handle) => images.get(image_handle).map(|image| {
            //     let Extent3d { width, height, .. } = image.texture_descriptor.size;
            //     Vec2::new(width as f32, height as f32)
            // }),
        }
    }
    // Check if this render target is contained in the given changed windows or images.
    fn is_changed(
        &self,
        changed_window_ids: &[WindowId],
        // changed_image_handles: &HashSet<&Handle<Image>>,
    ) -> bool {
        match self {
            RenderTarget::Window(window_id) => changed_window_ids.contains(window_id),
            // RenderTarget::Image(image_handle) => changed_image_handles.contains(&image_handle),
        }
    }
}
