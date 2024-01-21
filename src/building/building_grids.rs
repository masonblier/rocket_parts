use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;
use crate::character::{CharacterFpsMotionConfig,CHARACTER_GROUP};
use crate::world::WORLD_GROUP;


#[derive(Default,Clone)]
pub enum GridSolidity {
    #[default]
    Solid,
    Leaky,
}

#[derive(Default,Debug,Clone,PartialEq,Eq,Hash)]
pub struct DiscreteVec3([i32; 3]);
impl From<Vec3> for DiscreteVec3 {
    fn from(v: Vec3) -> Self {
        DiscreteVec3([
            v.x.round() as i32, 
            v.y.round() as i32, 
            v.z.round() as i32
        ])
    }
}

#[derive(Component,Default,Clone)]
pub struct Grid {
    pub solid_blocks: HashMap<DiscreteVec3,GridSolidity>,
    pub snaps_for_space: HashMap<DiscreteVec3,Vec<Entity>>,
}

#[derive(Component,Debug)]
pub struct GridBlock {
    pub entity: Entity,
    pub collider: Option<(Vec3, Quat, Collider)>,
}

pub fn find_or_create_grid(
    commands: &mut Commands,
    grid_ent: Option<Entity>, 
    target_transform: Transform,
    transforms_query: &mut Query<(&mut Transform, Without<CharacterFpsMotionConfig>)>,    
) -> (Entity, Transform) {
    if grid_ent.is_some() { 
        (grid_ent.unwrap(), *transforms_query.get(grid_ent.unwrap()).unwrap().0)
    } else {
        (commands.spawn(SpatialBundle {
                transform: target_transform,
                ..default()
            })
            .insert(Grid::default())
            .insert(RigidBody::Dynamic)
            .insert(ExternalForce { ..default() })
            .insert(CollisionGroups::new(CHARACTER_GROUP | WORLD_GROUP, CHARACTER_GROUP | WORLD_GROUP))
            .id(),
            target_transform,
        )
    }
}
