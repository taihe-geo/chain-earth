use crate::{
    DeltaTime, Plugin,PluginGroup,PluginGroupBuilder
};
use specs::{
    Builder, Component, Dispatcher, DispatcherBuilder, ReadStorage, RunNow, System, VecStorage,
    World, WorldExt,
};
use winit::{
    event_loop::{ControlFlow, EventLoop},
};
pub struct TransformCount(pub u32);
pub struct App<'a, 'b> {
    pub world: World,
    pub dispatcher_builder: DispatcherBuilder<'a, 'b>,
    pub dispatcher: Option<Dispatcher<'a, 'b>>,
    pub runner: Box<dyn Fn(App)>,
}
impl<'a, 'b> App<'a, 'b> {
    pub fn new() -> Self {
        let world = World::new();
        Self {
            world,
            dispatcher_builder: DispatcherBuilder::new(),
            dispatcher: None,
            runner: Box::new(run_once),
        }
    }
    pub fn update(&mut self) {
        self.build();
        self.dispatcher.unwrap().dispatch(&self.world);
    }
    fn build(&mut self) {
        if let None = self.dispatcher {
            self.dispatcher = Some(self.dispatcher_builder.build());
        }
    }
    pub fn run(&mut self) {
        let mut app = std::mem::replace(self, App::new());
        let runner = std::mem::replace(&mut app.runner, Box::new(run_once));
        (runner)(app);
    }
    pub fn add_system<T>(&mut self, system: T, name: &str, dep: &[&str]) -> &mut Self
    where
        T: for<'c> System<'c> + Send + 'a,
    {
        self.dispatcher_builder.with(system, name, dep);
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
