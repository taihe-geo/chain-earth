use specs::{Component, DenseVecStorage, DerefFlaggedStorage};
use wgpu::{RenderPassColorAttachment,Color,Operations};

use super::texture::TextureView;

pub struct ViewTarget {
    pub view: TextureView,
    pub sampled_target: Option<TextureView>,
}
impl Component for ViewTarget{
    type Storage = DerefFlaggedStorage<Self,DenseVecStorage<Self>>;
}

impl ViewTarget {
    pub fn get_color_attachment(&self, ops: Operations<Color>) -> RenderPassColorAttachment {
        RenderPassColorAttachment {
            view: if let Some(sampled_target) = &self.sampled_target {
                sampled_target
            } else {
                &self.view
            },
            resolve_target: if self.sampled_target.is_some() {
                Some(&self.view)
            } else {
                None
            },
            ops,
        }
    }
}