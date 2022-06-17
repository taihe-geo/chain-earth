mod render;
mod app;
pub use app::{App};
mod core;
mod graphics;
mod plugin;
pub use plugin::{Plugin,CreatePlugin};
mod plugin_group;
pub use plugin_group::{PluginGroup,PluginGroupBuilder};
mod plugins;
mod events;
mod winit;
#[derive(Default)]
pub struct DeltaTime(f32);