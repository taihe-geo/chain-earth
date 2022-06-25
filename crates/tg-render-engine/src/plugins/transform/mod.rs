// mod global_transform;
// mod transform;

// pub use global_transform::*;
// pub use transform::*;
use crate::{App, Plugin};
use nalgebra_glm::Mat4;
use specs::{
    Builder, Component, ReadStorage, System, SystemData, VecStorage, WorldExt, WriteStorage,DerefFlaggedStorage, DenseVecStorage
};
use specs_idvs::IdvStorage;


pub struct GlobalTransform(Mat4);
impl Component for GlobalTransform {
    type Storage =  DerefFlaggedStorage<Self,DenseVecStorage<Self>>;
}
impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::default())
    }
}
pub struct Transform(Mat4);
impl Component for Transform {
    type Storage =  DerefFlaggedStorage<Self,DenseVecStorage<Self>>;
}
impl Default for Transform {
    fn default() -> Self {
        Self(Mat4::default())
    }
}
struct TransformSystem;

impl<'a> System<'a> for TransformSystem {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, GlobalTransform>,
    );

    fn run(&mut self, (mut transform, global_transform): Self::SystemData) {

    }
}
pub struct TransformPlugin;
impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_add_systems(|dispatcher_builder| {
            dispatcher_builder.add(TransformSystem, "TransformSystem", &[]);
        });
    }
}
