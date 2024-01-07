use crate::game_state::GameState;
use crate::loading::{WorldProps,TextureAssets};
use crate::inputs::{MouseCamera, MouseLookState};
use crate::building::{BpInfo,BpInfos,HmSnapPoint};
use crate::character::{CharacterMotionConfigForPlatformerExample};

use std::f32::consts::PI;
use bevy::{prelude::*, gltf::Gltf, input::mouse::MouseButtonInput};
use bevy::input::mouse::MouseWheel;
use bevy_rapier3d::prelude::*;

const BUILD_DIST: f32 = 3.;
const THRUSTER_FORCE: f32 = 70.;
const SNAPS_GROUP: Group = Group::GROUP_3;

// system state
#[derive(Default, Resource)]
pub struct BuildingState {
    pub tool_active: bool,
    pub active_index: usize,
    pub shown_bp_entity: Option<Entity>,
    pub thrusters_active: bool,
    pub thrusters_animating: bool,
}


#[derive(Component,Default)]
pub struct GridEntity {
}
#[derive(Component,Debug)]
pub struct GridEntityRef {
    pub entity: Entity,
    pub collider: Option<(Vec3,Quat,Collider)>,
}


#[derive(Component,Default)]
pub struct ThrusterInteractable {
    grid: Option<Entity>,
}
#[derive(Component,Default)]
pub struct ThrusterSprite {
}

pub struct BuildingStatePlugin;
impl Plugin for BuildingStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BuildingState::default())
        .insert_resource(BpInfos::default())
        // .add_systems(OnEnter(GameState::Running), setup_building_interactive_states)
        .add_systems(Update, (
            update_building_state.run_if(in_state(GameState::Running)),
            update_building_tool_active.run_if(in_state(GameState::Running)),
            update_building_bp_transform.run_if(in_state(GameState::Running)),
            update_building_interactive_states.run_if(in_state(GameState::Running)),
        ));
    }
}

fn update_building_state(
    mut commands: Commands,
    assets_gltf: Res<Assets<Gltf>>,
    mut world_props: ResMut<WorldProps>,
    mouse_btn_input: Res<Input<MouseButton>>,
    mut building_state: ResMut<BuildingState>,
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, &CharacterMotionConfigForPlatformerExample)>,
    grid_query: Query<(&Transform, &GridEntity)>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    rapier_context: Res<RapierContext>,
    infos: Res<BpInfos>,
    snaps_query: Query<(&GlobalTransform, &HmSnapPoint)>,
    ge_entities: Query<&GridEntityRef>,
    collider_query: Query<&Collider>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {    
    let building_kit_names = infos.bps.keys().collect::<Vec<&String>>();

    // if tool not active
    if !building_state.tool_active {
        // hide bp model if still shown
        if let Some(shown_bp_entity) = building_state.shown_bp_entity {
            commands.entity(shown_bp_entity).despawn_recursive();
            building_state.shown_bp_entity = None;
        }
        
        return;
    }

    // check for bp change from mouse wheel scroll
    let mut next_index = building_state.active_index as i32;
    for mwe in mouse_wheel_events.iter() {
        next_index += mwe.y as i32;
        if next_index < 0 { next_index = (building_kit_names.len() as i32) - 1};
        if next_index >= (building_kit_names.len() as i32) { next_index = 0};
    }
    if building_state.active_index != (next_index as usize) {
        building_state.active_index = next_index as usize;
        if let Some(shown_bp_entity) = building_state.shown_bp_entity {
            commands.entity(shown_bp_entity).despawn_recursive();
            building_state.shown_bp_entity = None;
        }
    }


    // get projected position/rotation from collision raycast
    let (mover_transform, _mover) = mover_query.single();
    let scene_name = building_kit_names[building_state.active_index];
    let bp_info = infos.bps[&scene_name.to_string()].clone();
    let cast_result = cast_build_shape(mouse_look.forward, mover_transform, &bp_info, rapier_context, snaps_query, &ge_entities);
    let target_transform = cast_result.1;

    // show bp model if not shown
    if building_state.shown_bp_entity.is_none() {
        let bp_scene_name = building_kit_names[building_state.active_index].to_owned() + "_bp";
        building_state.shown_bp_entity = 
            spawn_gltf_instance(bp_scene_name.as_str(), 
                &mut commands, &assets_gltf, &world_props, target_transform);
    }

    if mouse_btn_input.just_pressed(MouseButton::Left) {
        let scene_name = building_kit_names[building_state.active_index];
        if let Some(gltf) = assets_gltf.get(&world_props.building_kit) {
            let (grid_entity, grid_transform) = if cast_result.2.is_some() { 
                (cast_result.2.unwrap(), grid_query.get(cast_result.2.unwrap()).unwrap().0)
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
                &target_transform,
            )
            };

            // insert solid gltf entity, colliders
            let local_translation = target_transform.translation - grid_transform.translation;
            let rot_quat = target_transform.rotation;
            let sb = SceneBundle {
                scene: gltf.named_scenes[scene_name].clone(),
                transform: target_transform.with_translation(local_translation),
                ..Default::default()
            };
            let mut build_block = commands.spawn(sb);
            build_block.insert(AdditionalMassProperties::Mass(2.));
            build_block.insert(GridEntityRef { entity: grid_entity, collider: Some((local_translation, rot_quat, bp_info.collider.clone())) });
            build_block.set_parent(grid_entity);
            // add block interactable extras
            if scene_name == "thruster" {
                build_block.insert(ThrusterInteractable { grid: Some(grid_entity) });
            }
            
            // insert building tool snap colliders
            for snap in bp_info.snap {
                let snap_transform = target_transform.with_translation(rot_quat.mul_vec3(snap.point));
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

            // update collider entities
            let mut colliders: Vec<(Vec3,Quat,Collider)> = ge_entities.iter().filter(|ge|{ ge.collider.is_some()})
                .map(|ge| { ge.collider.clone().unwrap() }).collect();
            colliders.push((local_translation, rot_quat, bp_info.collider.clone()));
            commands.entity(grid_entity).insert(Collider::compound(
                colliders
            ));
        }
    }
}

fn update_building_tool_active(
    mut building_state: ResMut<BuildingState>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // check tool toggle
    if keyboard_input.just_pressed(KeyCode::B) {
        building_state.tool_active = !building_state.tool_active;
    }

    // check thrusters toggle
    if keyboard_input.just_pressed(KeyCode::Z) {
        building_state.thrusters_active = !building_state.thrusters_active;
    }
}

fn update_building_bp_transform(
    mut commands: Commands,
    mut building_state: ResMut<BuildingState>,
    infos: Res<BpInfos>,
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, With<CharacterMotionConfigForPlatformerExample>)>,
    mut transforms_query: Query<(&mut Transform, Without<CharacterMotionConfigForPlatformerExample>)>,    
    rapier_context: Res<RapierContext>,
    snaps_query: Query<(&GlobalTransform, &HmSnapPoint)>,
    ge_entities: Query<&GridEntityRef>,
) {    
    let building_kit_names = infos.bps.keys().collect::<Vec<&String>>();

    // todo raycast
    if let Some(shown_bp_entity) = building_state.shown_bp_entity {
        if let Ok(mut bp_transform) = transforms_query.get_mut(shown_bp_entity) {
            let (mover_transform, _mover) = mover_query.single();
            let scene_name = building_kit_names[building_state.active_index];
            let bp_info = &infos.bps[&scene_name.to_string()];
            let cast_result = cast_build_shape(mouse_look.forward, mover_transform, bp_info, rapier_context, snaps_query, &ge_entities);
            bp_transform.0.clone_from(&cast_result.1);
        }
    }
}



#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn create_thruster_sprite(
    commands: &mut Commands,
    texture_handles: &Res<TextureAssets>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
) {
    let quad_width = 2.0;
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        quad_width,
        quad_width,
    ))));
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handles.explosion_static.clone()),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        unlit: true,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0)
            .with_rotation(Quat::from_rotation_y(PI / 2.)),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -0.8, 0.0)
            .with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
}
fn update_building_interactive_states(
    mut commands: Commands,
    time: Res<Time>,
    mut building_state: ResMut<BuildingState>,
    thursters_query: Query<(Entity, &ThrusterInteractable, &GlobalTransform)>,
    thurster_sprites_query: Query<(Entity, &ThrusterSprite)>,
    texture_handles: Res<TextureAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ext_forces: Query<&mut ExternalForce>
) {
    if !building_state.thrusters_active {
        if building_state.thrusters_animating {
            // cleanup sprites
            thurster_sprites_query.for_each(|ts| { commands.entity(ts.0).despawn_recursive(); });
            // remove external force from parent grid
            thursters_query.for_each(|t| { 
                let mut ext_force = ext_forces.get_mut(t.1.grid.unwrap()).unwrap();
                ext_force.force = Vec3::ZERO;
            });
            building_state.thrusters_animating = false;
        }
        return;
    }
    
    if building_state.thrusters_active && !building_state.thrusters_animating {
        thursters_query.for_each(|(entity, ti, thruster_gt )| {
            // setup combustion sprites
            create_thruster_sprite(&mut commands, &texture_handles, &mut meshes, &mut materials, entity);
            // add external force onto parent grid
            let mut ext_force = ext_forces.get_mut(ti.grid.unwrap()).unwrap();
            ext_force.force = thruster_gt.up() * THRUSTER_FORCE;
        });
        building_state.thrusters_animating = true;
    }

    // TODO animate sprites...
}


fn spawn_gltf_instance(
    scene_name: &str,
    commands: &mut Commands,
    assets_gltf: &Res<Assets<Gltf>>,
    world_props: &ResMut<WorldProps>,
    transform: Transform,
) -> Option<Entity> {
    // if the GLTF has loaded, we can navigate its contents
    if let Some(gltf) = assets_gltf.get(&world_props.building_kit) {
        Some(commands.spawn(SceneBundle {
            scene: gltf.named_scenes[scene_name].clone(),
            transform,
            ..Default::default()
        }).id())
    } else {
        None
    }
}

fn cast_build_shape(
    // mut world_state: ResMut<WorldState>,
    // mover_parent_query: Query<&GlobalTransform, With<MoverParent>>,
    mouse_forward: Vec3,
    mover_transform: &Transform,
    bp_info: &BpInfo,
    rapier_context: Res<RapierContext>,
    snaps_query: Query<(&GlobalTransform, &HmSnapPoint)>,
    ge_entities: &Query<&GridEntityRef>,
) -> (Option<Entity>, Transform, Option<Entity>) {
    // get interactable ray from player state
    let mover_pos = mover_transform.translation + 0.8 * Vec3::Y;

    // cast ray against snapping colliders
    let ray_groups = CollisionGroups::new(SNAPS_GROUP, SNAPS_GROUP);
    let ray_filter = QueryFilter { groups: Some(ray_groups), ..Default::default()};
    if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            mover_pos, mouse_forward, BUILD_DIST, true, ray_filter
    ) {
        if let Ok((snap_transform, snap)) = snaps_query.get(entity) {
            let ge = ge_entities.get(entity).unwrap();
            let snap_point = snap_transform.translation();
            let rot_quat = Quat::from_rotation_arc(Vec3::Y, snap.normal);
            return (Some(entity), Transform::from_translation(snap_point - rot_quat.mul_vec3(bp_info.bottom))
                .with_rotation(rot_quat), Some(ge.entity));
        }
    }

    // cast ray against world collidables
    let ray_groups = CollisionGroups::new(Group::GROUP_2, Group::GROUP_2);
    let ray_filter = QueryFilter { groups: Some(ray_groups), ..Default::default()};
    if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            mover_pos, mouse_forward, BUILD_DIST, true, ray_filter
    ) {
        let ger = ge_entities.get(entity).map(|ge| Some(ge.entity)).unwrap_or(None);
        let rot_quat = Quat::from_rotation_arc(Vec3::Y, intersection.normal);
        (Some(entity), Transform::from_translation(intersection.point - rot_quat.mul_vec3(bp_info.bottom))
            .with_rotation(rot_quat), ger)
    } else {
        (None, Transform::from_translation(mover_transform.translation + mouse_forward * BUILD_DIST - bp_info.bottom), None)
    }

}
