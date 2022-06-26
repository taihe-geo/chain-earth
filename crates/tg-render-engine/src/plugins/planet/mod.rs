use crate::{App, Plugin, app::Bundler};
use nalgebra_glm as glm;
pub mod entity;
pub struct PlanetPluginParameter {
    radius: glm::Vec3,
}
pub struct PlanetPlugin {
    radius: glm::Vec3,
}
impl PlanetPlugin {
    pub fn new(params: PlanetPluginParameter) -> Self {
        Self {
            radius: params.radius,
        }
    }
}
impl Default for PlanetPlugin {
    fn default() -> Self {
        let params = PlanetPluginParameter {
            radius: [1., 1., 1.].into(),
        };
        Self::new(params)
    }
}
impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.world.create_entity_with_bundle(entity::PlanetBundle::new(self.radius));
        app.add_add_systems(move |dispatch_builder| {
        });
    }
}
