use crate::{
    plugins::{render::RenderPlugin, winit::WinitPlugin,hierarchy::{HierarchyPlugin},transform::{TransformPlugin}},
    PluginGroup, PluginGroupBuilder,
};
pub struct DefautlPlugins;

impl PluginGroup for DefautlPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(WinitPlugin::default());
        group.add(RenderPlugin);
        group.add(TransformPlugin);
        group.add(HierarchyPlugin);
    }
}
