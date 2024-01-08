use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::building::{BpInfo,GridEntityRef};


pub fn insert_bp_snaps(
    commands: &mut Commands,
    bp_info: BpInfo,
    grid_entity: Entity,
    target_transform: Transform,
) {
    for snap in bp_info.snap {
        let snap_transform = target_transform.with_translation(target_transform.rotation.mul_vec3(snap.point));
        commands.spawn(SpatialBundle {
            transform: snap_transform,
            ..default()
        })
            .insert(snap.collider.clone())
            .insert(snap)
            .insert(CollisionGroups::new(Group::GROUP_3, Group::GROUP_3))
            .insert(GridEntityRef { entity: grid_entity, collider: None })
            .set_parent(grid_entity);
    }
}
