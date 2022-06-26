// mod global_transform;
// mod transform;

// pub use global_transform::*;
// pub use transform::*;
use crate::{App, Plugin};
use nalgebra_glm::Mat4;
use nalgebra_glm::Vec3;
// use specs::{
//     Builder, Component, ReadStorage, System, SystemData, VecStorage, WorldExt, WriteStorage,DerefFlaggedStorage, DenseVecStorage
// };
use crate::plugins::hierarchy::{Children, Parent, PreviousParent};
use specs::prelude::*;
use specs::DerefFlaggedStorage;
use specs_idvs::IdvStorage;

pub struct Pos(pub Vec3);
impl Component for Pos {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Default for Pos {
    fn default() -> Self {
        Self(Vec3::default())
    }
}

pub struct GlobalMatrix(pub Mat4);
impl Component for GlobalMatrix {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Default for GlobalMatrix {
    fn default() -> Self {
        Self(Mat4::default())
    }
}
pub struct LocalMatrix(pub Mat4);
impl Component for LocalMatrix {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Default for LocalMatrix {
    fn default() -> Self {
        Self(Mat4::default())
    }
}
#[derive(Default)]
struct TransformSystem {
    modified: BitSet,
    reader_id: Option<ReaderId<ComponentEvent>>,
}
impl<'a> System<'a> for TransformSystem {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, LocalMatrix>,
        WriteStorage<'a, GlobalMatrix>,
        WriteStorage<'a, Children>,
        WriteStorage<'a, Parent>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (mut s_entities, mut s_matrix, mut s_global_matrix, s_children, s_parent,s_updater): Self::SystemData,
    ) {
        self.modified.clear();
        let events = s_matrix
            .channel()
            .read(self.reader_id.as_mut().expect("ReaderId not found"));
        for event in events {
            match event {
                ComponentEvent::Modified(id) => {
                    self.modified.add(*id);
                }
                _ => {}
            }
        }
        for (children, matrix, global_matrix, _, ()) in (
            (&s_children).maybe(),
            &s_matrix,
            &s_global_matrix,
            &self.modified,
            !&s_parent,
        )
            .join()
        {
            let mut changed = false;
            if let Some(children) = children {
                for child in children.iter() {
                    propagate_recursive(
                        global_matrix.0,
                        &s_matrix,
                        &s_global_matrix,
                        &s_children,
                        *child,
                        changed,
                        &s_updater,
                    );
                }
            }
        }
    }
    fn setup(&mut self, res: &mut World) {
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<LocalMatrix>::fetch(&res).register_reader());
    }
}
fn propagate_recursive<'a>(
    parent_global_matrix: Mat4,
    s_matrix: &WriteStorage<'a, LocalMatrix>,
    s_global_matrix: &WriteStorage<'a, GlobalMatrix>,
    s_children: &WriteStorage<'a, Children>,
    entity: Entity,
    mut changed: bool,
    s_updater: &Read<'a, LazyUpdate>,
) {
    let global_matrix = {
        if let (Some(matrix), Some(global_matrix)) =
            (s_matrix.get(entity), s_global_matrix.get(entity))
        {
            if changed {
                let res = parent_global_matrix * matrix.0;
                s_updater.insert(entity, LocalMatrix(res));
                res
            } else {
                global_matrix.0.clone()
            }
        } else {
            return;
        }
    };
    if let Some(children) = s_children.get(entity) {
        for child in children.iter() {
            propagate_recursive(
                global_matrix,
                s_matrix,
                s_global_matrix,
                s_children,
                *child,
                changed,
                s_updater,
            )
        }
    }
}
pub struct TransformPlugin;
impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.world.register::<Pos>();
        app.world.register::<GlobalMatrix>();
        app.world.register::<LocalMatrix>();
        app.add_add_systems(|dispatcher_builder| {
            dispatcher_builder.add(TransformSystem::default(), "TransformSystem", &[]);
        });
    }
}