// use specs::{ Entities, Entity, VecStorage, System, Component};
// use specs::{join, world::EntitiesRes, Component,DerefFlaggedStorage,Resources};
use specs::prelude::*;
use specs::DerefFlaggedStorage;
use specs_idvs::IdvStorage;
// use specs::{ DerefFlaggedStorage,SystemData,System,world::{Index as EntityId}};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::{
    ops::{Deref, DerefMut},
    process::Child,
};

use crate::Plugin;
pub struct Children(pub SmallVec<[Entity; 8]>);
impl Component for Children {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Children {
    /// Builds and returns a [`Children`] component with the given entities
    pub fn with(entity: &[Entity]) -> Self {
        Self(SmallVec::from_slice(entity))
    }

    /// Swaps the child at `a_index` with the child at `b_index`
    pub fn swap(&mut self, a_index: usize, b_index: usize) {
        self.0.swap(a_index, b_index);
    }
}
impl Deref for Children {
    type Target = [Entity];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}
// #[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parent(pub Entity);
impl Component for Parent {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Deref for Parent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Parent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub struct PreviousParent(pub Entity);
impl Component for PreviousParent {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Deref for PreviousParent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for PreviousParent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[derive(Default)]
pub struct HierarchySystem {
    pub dirty: BitSet,
    pub reader_id: Option<ReaderId<ComponentEvent>>,
}
impl<'a> System<'a> for HierarchySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, PreviousParent>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Children>,
        Read<'a, LazyUpdate>,
    );
    fn run(
        &mut self,
        (mut s_entities, mut s_previous_parent, mut s_parent, mut s_children, mut s_updater): Self::SystemData,
    ) {
        //parent不存在时清除entity的previous_parent
        for (entity, previoud_parent, ()) in
            (&s_entities, &mut s_previous_parent, !&s_parent).join()
        {
            s_updater.remove::<PreviousParent>(entity);
        }
        let events = s_parent.channel().read(self.reader_id.as_mut().unwrap());
        for event in events {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    self.dirty.add(*id);
                }
                // We don't need to take this event into account since
                // removed components will be filtered out by the join;
                // if you want to, you can use `self.dirty.remove(*id);`
                // so the bit set only contains IDs that still exist
                ComponentEvent::Removed(_) => (),
            }
        }
        let mut children_additions = HashMap::<Entity, SmallVec<[Entity; 8]>>::default();

        for (entity, previous_parent, parent_id) in
            (&s_entities, (&mut s_previous_parent).maybe(), &self.dirty).join()
        {
            if let Some(parent) = s_parent.get(entity) {
                if let Some(mut possible_previous_parent) = previous_parent {
                    if possible_previous_parent.0 == parent.0 {
                        continue;
                    }
                    if let Some(mut previous_parent_children) =
                        s_children.get_mut(possible_previous_parent.0)
                    {
                        previous_parent_children.0.retain(|e| *e != entity);
                    }
                    *possible_previous_parent = (PreviousParent(parent.0));
                } else {
                    s_updater.insert(entity, (Parent(parent.0)));
                }
                if let Some(mut parent_children) = s_children.get_mut(parent.0) {
                    if !parent_children.0.contains(&entity) {
                        parent_children.0.push(entity)
                    }
                } else {
                    children_additions
                        .entry(parent.0)
                        .or_insert_with(Default::default)
                        .push(entity);
                }

                children_additions
                    .iter()
                    .for_each(|(e, v)| s_updater.insert(*e, Children::with(v)))
            }
        }
    }
    fn setup(&mut self, res: &mut World) {
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<Parent>::fetch(&res).register_reader());
    }
}
pub struct HierarchyPlugin;
impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut crate::App) {
        app.add_add_systems(|dispaptch_builder| {
            dispaptch_builder.add(HierarchySystem::default(), "HierarchySystem", &[]);
        });
    }
}
