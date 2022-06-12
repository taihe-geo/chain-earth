use anyhow::{Error, Ok};
use wgpu::util::DeviceExt;
use std::borrow::Cow;
use std::time::{Duration, Instant};
use tg_render_engine::{Demo, Display};
use wgpu::RenderPipeline;
use std::iter;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Geometry {
    pub indices: [[f32; 3]],
    pub vertices: [[f32; 3]],
    // pub normals,
    // pub uvs:
}
