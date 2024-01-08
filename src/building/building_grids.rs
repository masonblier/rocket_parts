use bevy::prelude::*;
use bevy_rapier3d::prelude::*;


#[derive(Component,Default)]
pub struct GridEntity {
}

pub fn find_or_create_grid(
    commands: &mut Commands,
    grid_ent: Option<Entity>, 
    target_transform: Transform,
    grid_query: &Query<(&Transform, &GridEntity)>,
) -> (Entity, Transform) {
    if grid_ent.is_some() { 
        (grid_ent.unwrap(), *grid_query.get(grid_ent.unwrap()).unwrap().0)
    } else {
        (commands.spawn(SpatialBundle {
                transform: target_transform,
                ..default()
            })
            .insert(GridEntity::default())
            .insert(RigidBody::Dynamic)
            .insert(ExternalForce { ..default() })
            .insert(CollisionGroups::new(Group::GROUP_1 | Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2))
            .id(),
            target_transform,
        )
    }
}
