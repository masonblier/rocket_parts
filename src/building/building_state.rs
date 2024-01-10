use crate::game_state::GameState;
use crate::loading::WorldProps;
use crate::actions::BuildingActionsState;
use crate::inputs::MouseLookState;
use crate::building::{BpInfo,BpInfos,BpSnapPoint,find_or_create_grid,GridEntity,
    insert_bp_snaps,BuildingToolbarPlugin};
use crate::props::ThrusterInteractable;
use crate::character::CharacterMotionConfigForPlatformerExample;

use bevy::{prelude::*, gltf::Gltf};
use bevy_rapier3d::prelude::*;

const BUILD_DIST: f32 = 3.;
const SNAPS_GROUP: Group = Group::GROUP_3;

// system state
#[derive(Default, Resource)]
pub struct BuildingState {
    pub active_index: usize,
    pub shown_bp_entity: Option<Entity>,
}


#[derive(Component,Debug)]
pub struct GridEntityRef {
    pub entity: Entity,
    pub collider: Option<(Vec3,Quat,Collider)>,
}

pub struct BuildingStatePlugin;
impl Plugin for BuildingStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BuildingState::default())
        .insert_resource(BpInfos::default())
        .add_plugins((BuildingToolbarPlugin::default(),))
        // .add_systems(OnEnter(GameState::Running), setup_building_interactive_states)
        .add_systems(Update, (
            update_building_state.run_if(in_state(GameState::Running)),
            update_building_bp_transform.run_if(in_state(GameState::Running)),
        ));
    }
}

fn update_building_state(
    mut commands: Commands,
    assets_gltf: Res<Assets<Gltf>>,
    world_props: Res<WorldProps>,
    mouse_btn_input: Res<Input<MouseButton>>,
    mut building_state: ResMut<BuildingState>,
    building_actions: Res<BuildingActionsState>,
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, &CharacterMotionConfigForPlatformerExample)>,
    grid_query: Query<(&Transform, &GridEntity)>,
    rapier_context: Res<RapierContext>,
    infos: Res<BpInfos>,
    snaps_query: Query<(&GlobalTransform, &BpSnapPoint)>,
    ge_entities: Query<&GridEntityRef>,
) {    
    let building_kit_names = infos.toolbar_order.clone();

    // if tool not active
    if !building_actions.building_active {
        // hide bp model if still shown
        if let Some(shown_bp_entity) = building_state.shown_bp_entity {
            commands.entity(shown_bp_entity).despawn_recursive();
            building_state.shown_bp_entity = None;
        }
        
        return;
    }

    // check for bp index change from BuildingActionsState
    if building_state.active_index != (building_actions.active_index as usize) {
        building_state.active_index = building_actions.active_index as usize;
        if let Some(shown_bp_entity) = building_state.shown_bp_entity {
            commands.entity(shown_bp_entity).despawn_recursive();
            building_state.shown_bp_entity = None;
        }
    }


    // get projected position/rotation from collision raycast
    let (mover_transform, _mover) = mover_query.single();
    let scene_name = &building_kit_names[building_state.active_index];
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
        let scene_name = &building_kit_names[building_state.active_index];
        if let Some(gltf) = assets_gltf.get(&world_props.building_kit) {
            let (grid_entity, grid_transform) = find_or_create_grid(
                &mut commands, cast_result.2, target_transform, &grid_query);

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
            insert_bp_snaps(&mut commands, bp_info.clone(), grid_entity, target_transform);
            

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

fn update_building_bp_transform(
    building_state: ResMut<BuildingState>,
    infos: Res<BpInfos>,
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, With<CharacterMotionConfigForPlatformerExample>)>,
    mut transforms_query: Query<(&mut Transform, Without<CharacterMotionConfigForPlatformerExample>)>,    
    rapier_context: Res<RapierContext>,
    snaps_query: Query<(&GlobalTransform, &BpSnapPoint)>,
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


fn spawn_gltf_instance(
    scene_name: &str,
    commands: &mut Commands,
    assets_gltf: &Res<Assets<Gltf>>,
    world_props: &Res<WorldProps>,
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
    snaps_query: Query<(&GlobalTransform, &BpSnapPoint)>,
    ge_entities: &Query<&GridEntityRef>,
) -> (Option<Entity>, Transform, Option<Entity>) {
    // get interactable ray from player state
    let mover_pos = mover_transform.translation + 0.8 * Vec3::Y;

    // cast ray against snapping colliders
    let ray_groups = CollisionGroups::new(SNAPS_GROUP, SNAPS_GROUP);
    let ray_filter = QueryFilter { groups: Some(ray_groups), ..Default::default()};
    if let Some((entity, _intersection)) = rapier_context.cast_ray_and_get_normal(
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
