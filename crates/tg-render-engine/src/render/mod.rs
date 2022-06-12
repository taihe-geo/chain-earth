pub mod draw;
pub mod render_device;
use std::sync::{Arc};
use render_device::{RenderDevice};
use wgpu::{CommandEncoder};

/// The context with all information required to interact with the GPU.
///
/// The [`RenderDevice`] is used to create render resources and the
/// the [`CommandEncoder`] is used to record a series of GPU operations.
pub struct RenderContext {
    pub render_device: RenderDevice,
    pub command_encoder: CommandEncoder,
}
