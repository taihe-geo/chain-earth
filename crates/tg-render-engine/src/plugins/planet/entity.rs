use crate::{
    plugins::{
        hierarchy::{Children, Parent},
        transform::{GlobalMatrix, LocalMatrix, Pos},
    },
    Bundle, Bundler,
};
use nalgebra_glm::{Vec2, Vec3};
use specs::{Builder, Component, DenseVecStorage, Entity, EntityBuilder, World, WorldExt};
#[derive(Component, Default)]
pub struct Radius3(Vec3);
#[derive(Component, Default)]
pub struct Radius2(Vec2);
#[derive(Component, Default)]
pub struct Radius(f32);
#[derive(Default)]
pub struct PlanetBundle {
    pub local_matrix: LocalMatrix,
    pub global_matrix: GlobalMatrix,
    pub pos: Pos,
    pub parent: Option<Parent>,
    pub children: Option<Children>,
    pub radius: Radius3,
}
impl PlanetBundle {
    pub fn new(radius: Vec3) -> Self {
        Self {
            radius: Radius3(radius),
            ..Default::default()
        }
    }
}
impl Bundle for PlanetBundle {
    fn bundle(self, world: &mut World) -> Entity {
            world
            .create_entity()
            .with(self.local_matrix)
            .with(self.global_matrix)
            .with(self.pos)
            .maybe_with(self.parent)
            .maybe_with(self.children)
            .with(self.radius)
            .build()
    }
}
