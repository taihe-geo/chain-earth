use crate::{plugins, DefautlPlugins, DeltaTime, Plugin, PluginGroup, PluginGroupBuilder};
use specs::{
    Builder, Component, Dispatcher, DispatcherBuilder, ReadStorage, RunNow, System, VecStorage,
    World, WorldExt,
};
use winit::event_loop::{ControlFlow, EventLoop};
pub struct TransformCount(pub u32);
pub struct App {
    pub world: World,
    pub runner: Box<dyn Fn(App)>,
    pub add_system_list: Vec<Box<dyn Fn(&mut DispatcherBuilder)>>,
}
impl Default for App {
    fn default() -> Self {
        let mut app = App::new();
        app.add_plugins(DefautlPlugins);
        app
    }
}
impl App {
    pub fn new() -> Self {
        let world = World::new();
        Self {
            world,
            runner: Box::new(run_once),
            add_system_list: Vec::new(),
        }
    }
    pub fn update(&self) {
        let mut dispatcher_builder = DispatcherBuilder::new();
        self.add_system_list.iter().for_each(|add_system| {
            add_system(&mut dispatcher_builder);
        });
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.dispatch(&self.world);
    }
    pub fn run(&mut self) {
        let mut app = std::mem::replace(self, App::new());
        let runner = std::mem::replace(&mut app.runner, Box::new(run_once));
        (runner)(app);
    }
    pub fn add_add_systems(
        &mut self,
        add_systems: impl Fn(&mut DispatcherBuilder) + 'static,
    ) -> &mut Self {
        self.add_system_list.push(Box::new(add_systems));
        self
    }
    pub fn add_plugin<T>(&mut self, plugin: T) -> &mut Self
    where
        T: Plugin,
    {
        plugin.build(self);
        self
    }
    pub fn add_plugins<T: PluginGroup>(&mut self, mut group: T) -> &mut Self {
        let mut plugin_group_builder = PluginGroupBuilder::default();
        group.build(&mut plugin_group_builder);
        plugin_group_builder.finish(self);
        self
    }
    pub fn set_runner(&mut self, run_fn: impl Fn(App) + 'static) -> &mut Self {
        self.runner = Box::new(run_fn);
        self
    }
}
fn run_once(mut app: App) {
    app.update();
}
