use ahash::RandomState;
mod render;
mod app;
pub use app::{App,Bundle,Bundler};
mod core;
mod graphics;
mod plugin;
pub use plugin::{Plugin,CreatePlugin};
mod plugin_group;
pub use plugin_group::{PluginGroup,PluginGroupBuilder};
mod plugins;
mod events;
mod winit;
pub use plugins::default::DefautlPlugins;
#[derive(Default)]
pub struct DeltaTime(f32);
pub type HashSet<K> = hashbrown::HashSet<K, RandomState>;
pub type HashMap<K, V> = hashbrown::HashMap<K, V, RandomState>;
