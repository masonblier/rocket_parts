use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::building::{BpInfo,BpSnapPoint,DiscreteVec3,Grid,GridBlock};
use crate::character::CharacterFpsMotionConfig;

pub const BUILD_DIST: f32 = 3.;
pub const SNAPS_GROUP: Group = Group::GROUP_3;

pub struct BpSnapResult {
    pub snap_entity: Option<Entity>,
    pub grid_entity: Option<Entity>,
    pub local_transform: Transform,
}

#[derive(Component,Debug)]
pub struct GridSnapPoint {
    pub entity: Entity,
    pub collider: Option<(Vec3,Quat,Collider)>,
}


#[derive(Clone,Event)]
pub enum BpSnapsEvent {
    InsertSnaps(BpInfo, Entity, Transform),
}
#[derive(Clone,Event)]
pub struct BpSnapsRepeatEvent(BpSnapsEvent);

pub fn update_building_bp_snaps(
    mut commands: Commands,
    mut grids_query: Query<&mut Grid>,
    // snaps_query: Query<(&GlobalTransform, &BpSnapPoint)>,
    mut snaps_events: EventReader<BpSnapsEvent>,
    mut snaps_events_out: EventWriter<BpSnapsRepeatEvent>,
) {
    for snaps_event in snaps_events.read() {
        match snaps_event {
            BpSnapsEvent::InsertSnaps(bp_info, grid_entity, target_transform) => {
                if !grids_query.contains(*grid_entity) {
                    // grid entity not yet inserted, try again next frame
                    snaps_events_out.send(BpSnapsRepeatEvent(snaps_event.clone()));
                } else {
                    insert_bp_snaps(
                        &mut commands,
                        bp_info,
                        grid_entity.clone(),
                        &mut grids_query,
                        target_transform,
                    );
                }
            },
        }
    }
}

pub fn insert_bp_snaps(
    commands: &mut Commands,
    bp_info: &BpInfo,
    grid_entity: Entity,
    grids_query: &mut Query<&mut Grid>,
    target_transform: &Transform,
) {
    let dest_pos = DiscreteVec3::from(target_transform.translation);
    let mut grid = grids_query.get_mut(grid_entity).unwrap();

    // update grid solidity
    // todo move to better place
    grid.solid_blocks.insert(dest_pos.clone(), bp_info.solidity.clone());

    // remove snaps for destination block
    if grid.snaps_for_space.contains_key(&dest_pos) {
        grid.snaps_for_space.remove(&dest_pos).unwrap().into_iter().for_each(|old_snap| {
            commands.entity(old_snap).despawn_recursive();
        });
    }

    // insert snaps from bp, ignoring occupied positions
    for snap in bp_info.snap.iter() {
        let snap_point = target_transform.rotation.mul_vec3(snap.point);
        let snap_transform = Transform::from_translation(target_transform.translation + snap_point)
            .with_rotation(target_transform.rotation);
        let snap_target_pos = DiscreteVec3::from(target_transform.translation + 2. * snap_point);
        if grid.solid_blocks.contains_key(&snap_target_pos) {
            continue;
        }

        let snap_ent = commands.spawn(SpatialBundle {
            transform: snap_transform,
            ..default()
        })
            .insert(Collider::cuboid(snap.cuboid_dims.x, snap.cuboid_dims.y, snap.cuboid_dims.z))
            .insert(snap.clone())
            .insert(CollisionGroups::new(Group::GROUP_3, Group::GROUP_3))
            .insert(GridSnapPoint { 
                entity: grid_entity, 
                collider: None, 
            })
            // for debugging
            // .with_children(|parent| {
            //     parent.spawn(PbrBundle {
            //         mesh: meshes.add(shape::Cube { size: 1.0 }.into()),
            //         material: materials.add(Color::rgba(0.7,0.4,0.2,0.4).into()),
            //         transform: Transform::from_scale(snap.clone().cuboid_dims),
            //         ..default()
            //     });
            // })
            .set_parent(grid_entity)
            .id();
        if grid.snaps_for_space.contains_key(&snap_target_pos) {
            grid.snaps_for_space.get_mut(&snap_target_pos).unwrap().push(snap_ent);
        } else {
            grid.snaps_for_space.insert(snap_target_pos.clone(), vec![snap_ent]);
        }
    }
}

pub fn cast_snaps_ray(
    // mut world_state: ResMut<WorldState>,
    // mover_parent_query: Query<&GlobalTransform, With<MoverParent>>,
    mouse_forward: Vec3,
    cast_origin: Vec3,
    bp_info: &BpInfo,
    rapier_context: &Res<RapierContext>,
    gsp_query: &Query<&GridSnapPoint>,
    gb_query: &Query<&GridBlock>,
    snaps_query: Query<&BpSnapPoint>,
    transforms_query: &mut Query<(&mut Transform, Without<CharacterFpsMotionConfig>)>,
    local_rot: Quat,  
) -> Option<BpSnapResult> {
    // cast ray against snapping colliders
    let ray_groups = CollisionGroups::new(SNAPS_GROUP | Group::GROUP_2, SNAPS_GROUP | Group::GROUP_2);
    let ray_filter: QueryFilter<'_> = QueryFilter { groups: Some(ray_groups), ..Default::default()};
    if let Some((collided_entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            cast_origin, mouse_forward, BUILD_DIST, true, ray_filter
    ) {
        // snappable collider
        if let Ok(ge) = gsp_query.get(collided_entity) {
            let (snap_transform, _) = transforms_query.get(collided_entity).unwrap();
            let snap = snaps_query.get(collided_entity).unwrap();
            
            let rot_quat = Quat::from_rotation_arc(Vec3::Y, snap.normal);
            let snap_offset = snap_transform.rotation.mul_vec3(bp_info.bottom.length() * snap.normal);
            let local_transform = Transform::from_translation(
                snap_transform.translation + snap_offset)
                .with_rotation(local_rot.mul_quat(rot_quat));
            return Some(BpSnapResult {
                snap_entity: Some(collided_entity), 
                grid_entity: Some(ge.entity),
                local_transform,
            });
        } else {
            let rot_quat = Quat::from_rotation_arc(Vec3::Y, intersection.normal);
            let local_transform = Transform::from_translation(intersection.point - rot_quat.mul_vec3(bp_info.bottom))
                .with_rotation(local_rot.mul_quat(rot_quat));
            // grid block
            if let Ok(gb) = gb_query.get(collided_entity) {
                return Some(BpSnapResult {
                    snap_entity: None, 
                    grid_entity: Some(gb.entity),
                    local_transform,
                });
            } else {
                // world collidable
                return Some(BpSnapResult {
                    snap_entity: None,
                    grid_entity: None,
                    local_transform,
                });
            }
        }
    }
    // no collision
    None
}

pub fn update_building_bp_snaps_repeats(
    mut snaps_events: EventReader<BpSnapsRepeatEvent>,
    mut snaps_events_out: EventWriter<BpSnapsEvent>,
) {
    // needed because grid entity is not yet inserted before
    // event is read
    // todo solve this in a better way
    for snaps_event in snaps_events.read() {
        snaps_events_out.send(snaps_event.0.clone());
    }
}
