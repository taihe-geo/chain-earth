use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Vec3, Vec4};
use std::{borrow::Cow, sync::Arc};

use super::{ render_target::{RenderTarget}};

pub struct CurrentRenderTarget(pub Option<(Arc<RenderTarget>, wgpu::TextureView)>);
