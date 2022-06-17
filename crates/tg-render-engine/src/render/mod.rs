mod draw;
mod render_device;
pub use draw::*;
pub use render_device::*;

use std::sync::{Arc};
use render_device::{RenderDevice};
use wgpu::{CommandEncoder,Queue};

/// 和GPU交互需要的所有信息
///
/// [`RenderDevice`] 用来渲染资源，[`CommandEncoder`]用来记录一系列的GPU操作
pub struct RenderContext {
    pub render_device: RenderDevice,
    pub command_encoder: CommandEncoder,
}

/// This queue is used to enqueue tasks for the GPU to execute asynchronously.
pub type RenderQueue = Arc<Queue>;