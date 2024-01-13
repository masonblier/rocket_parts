use crate::game_state::GameState;
use crate::loading::WorldProps;
use crate::actions::BuildingActionsState;
use crate::inputs::MouseLookState;
use crate::building::{BpInfo,BpInfos,BpSnapPoint,BpSnapsEvent,BpSnapsRepeatEvent,
    find_or_create_grid,GridBlock,GridSnapPoint,
    update_building_bp_snaps,update_building_bp_snaps_repeats,cast_snaps_ray,
    BuildingToolbarPlugin,BUILD_DIST};
use crate::props::ThrusterInteractable;
use crate::character::CharacterFpsMotionConfig;
use crate::world::WorldLoadingState;

use bevy::{prelude::*, gltf::Gltf};
use bevy_rapier3d::prelude::*;

const SNAP_DELAY: f32 = 0.3;

// system state
#[derive(Default, Resource)]
pub struct BuildingState {
    pub active_index: usize,
    pub shown_bp_entity: Option<Entity>,
    cast_result: BpCastResult,
    last_cast_time: Timer,
}

pub struct BuildingStatePlugin;
impl Plugin for BuildingStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BuildingState::default())
        .insert_resource(BpInfos::default())
        .add_event::<BpSnapsEvent>()
        .add_event::<BpSnapsRepeatEvent>()
        .add_plugins((BuildingToolbarPlugin::default(),))
        // .add_systems(OnEnter(GameState::WorldLoading), setup_building_interactive_states)
        .add_systems(Update, (
            update_building_state.run_if(in_state(GameState::Running)),
            update_building_bp_transform.run_if(in_state(GameState::Running)),
            update_building_bp_snaps.run_if(in_state(GameState::Running)),
            update_building_bp_snaps_repeats.run_if(in_state(GameState::Running)),
        ));
    }
}

fn update_building_state(
    mut commands: Commands,
    assets_gltf: Res<Assets<Gltf>>,
    world_props: Res<WorldProps>,
    mut world_loading: ResMut<WorldLoadingState>,
    mouse_btn_input: Res<Input<MouseButton>>,
    mut building_state: ResMut<BuildingState>,
    building_actions: Res<BuildingActionsState>,
    infos: Res<BpInfos>,
    gb_query: Query<&GridBlock>,
    mut snaps_events: EventWriter<BpSnapsEvent>,
    mut transforms_query: Query<(&mut Transform, Without<CharacterFpsMotionConfig>)>,    
) {    
    let building_kit_names = infos.toolbar_order.clone();

    // if tool not active
    if !building_actions.building_active {
        // hide bp model if still shown
        if let Some(shown_bp_entity) = building_state.shown_bp_entity {
            commands.entity(shown_bp_entity).insert(Visibility::Hidden);
            building_state.shown_bp_entity = None;
        }
        
        return;
    }

    // get projected position/rotation from collision raycast
    let scene_name = &building_kit_names[building_actions.active_index];
    let bp_info = infos.bps[&scene_name.to_string()].clone();
    let grid_transform = building_state.cast_result.grid_transform;

    // show bp model if not shown
    let bp_scene_name: String = scene_name.to_owned() + "_bp";
    if world_loading.build_kit_preload_ent.is_none() {
        world_loading.build_kit_preload_ent = 
            spawn_gltf_instance(bp_scene_name.as_str(), 
                &mut commands, &assets_gltf, &world_props, grid_transform);
    }
    commands.entity(world_loading.build_kit_preload_ent.unwrap()).insert(Visibility::Visible);
    building_state.shown_bp_entity = world_loading.build_kit_preload_ent;

    // update selected bp scene
    if building_actions.active_index != building_state.active_index {
        if let Some(gltf) = assets_gltf.get(&world_props.building_kit) {
            commands.entity(world_loading.build_kit_preload_ent.unwrap()).insert(gltf.named_scenes[&bp_scene_name].clone());
        }
        building_state.active_index = building_actions.active_index;
    }

    if mouse_btn_input.just_pressed(MouseButton::Left) {
        let scene_name = &building_kit_names[building_state.active_index];
        if let Some(gltf) = assets_gltf.get(&world_props.building_kit) {
            let (grid_entity, _grid_transform) = find_or_create_grid(
                &mut commands, building_state.cast_result.grid_entity, grid_transform, &mut transforms_query);

            // insert solid gltf entity, colliders
            let local_translation = building_state.cast_result.local_transform.translation;
            let rot_quat = building_state.cast_result.local_transform.rotation;
            let sb = SceneBundle {
                scene: gltf.named_scenes[scene_name].clone(),
                transform: Transform::from_translation(local_translation).with_rotation(rot_quat),
                ..Default::default()
            };
            let mut build_block = commands.spawn(sb);
            build_block.insert(AdditionalMassProperties::Mass(2.));
            build_block.insert(GridBlock { 
                entity: grid_entity, 
                collider: Some((local_translation, rot_quat, bp_info.collider.clone())), 
            });
            build_block.set_parent(grid_entity);
            // add block interactable extras
            if scene_name == "thruster" {
                build_block.insert(ThrusterInteractable { grid: Some(grid_entity) });
            }
            
            // insert building tool snap colliders
            snaps_events.send(BpSnapsEvent::InsertSnaps(
                bp_info.clone(), grid_entity, Transform::from_translation(local_translation).with_rotation(rot_quat)));

            // update collider entities
            let mut colliders: Vec<(Vec3,Quat,Collider)> = gb_query.iter()
                .filter(|gb|{ gb.entity == grid_entity && gb.collider.is_some()})
                .map(|gb| { gb.collider.clone().unwrap() }).collect();
            colliders.push((local_translation, rot_quat, bp_info.collider.clone()));
            commands.entity(grid_entity).insert(Collider::compound(
                colliders
            ));
        }
    }
}

fn update_building_bp_transform(
    time: Res<Time>,
    mut building_state: ResMut<BuildingState>,
    building_actions: Res<BuildingActionsState>,
    infos: Res<BpInfos>,
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, With<CharacterFpsMotionConfig>)>,
    mut transforms_query: Query<(&mut Transform, Without<CharacterFpsMotionConfig>)>,    
    rapier_context: Res<RapierContext>,
    snaps_query: Query<&BpSnapPoint>,
    gsp_query: Query<&GridSnapPoint>,
    gb_query: Query<&GridBlock>,
) {
    // debounce snap movement
    building_state.last_cast_time.tick(time.delta());
    if !building_state.last_cast_time.finished() {
        return;
    }
    
    let building_kit_names = infos.bps.keys().collect::<Vec<&String>>();

    // raycast
    if let Some(shown_bp_entity) = building_state.shown_bp_entity {
        let (mover_transform, _mover) = mover_query.single();
        let scene_name = building_kit_names[building_state.active_index];
        let bp_info = &infos.bps[&scene_name.to_string()];
        building_state.cast_result = cast_build_shape(
            mouse_look.forward, mover_transform, 
            bp_info, &rapier_context, snaps_query, &gsp_query, &gb_query,
            &mut transforms_query, building_actions.active_rotation,
        );
        if let Ok(mut bp_transform) = transforms_query.get_mut(shown_bp_entity) {
            bp_transform.0.clone_from(&building_state.cast_result.grid_transform.mul_transform(
                building_state.cast_result.local_transform));
        }
        // delay next cast if snapped to snap point
        if building_state.cast_result.snapped {
            building_state.last_cast_time = Timer::from_seconds(SNAP_DELAY, TimerMode::Once);
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

#[derive(Default)]
struct BpCastResult {
    snapped: bool,
    grid_transform: Transform, 
    local_transform: Transform, 
    grid_entity: Option<Entity>,
}
impl BpCastResult {
    fn new(snapped: bool, grid_transform: Transform, local_transform: Transform, grid_entity: Option<Entity>) -> Self {
        Self {
            snapped, grid_transform, local_transform, grid_entity
        }
    }
}

fn cast_build_shape(
    // mut world_state: ResMut<WorldState>,
    // mover_parent_query: Query<&GlobalTransform, With<MoverParent>>,
    mouse_forward: Vec3,
    mover_transform: &Transform,
    bp_info: &BpInfo,
    rapier_context: &Res<RapierContext>,
    snaps_query: Query<&BpSnapPoint>,
    gsp_query: &Query<&GridSnapPoint>,
    gb_query: &Query<&GridBlock>,
    transforms_query: &mut Query<(&mut Transform, Without<CharacterFpsMotionConfig>)>,
    local_rot: Quat,  
) -> BpCastResult {
    // get interactable ray from player state
    let mover_pos = mover_transform.translation + 0.4 * Vec3::Y;
    // default transform of item (in new grid)
    let rot_in_place_transform = Transform::from_rotation(local_rot);

    // check for building snaps
    if let Some(snap_result) = cast_snaps_ray(
        mouse_forward, mover_pos, bp_info,
        rapier_context, gsp_query, gb_query,
        snaps_query,
        transforms_query,
        local_rot
    ) {
        // snap to grid block
        if let Some(grid_entity) = snap_result.grid_entity {
            if let Ok((grid_transform, _)) = transforms_query.get(grid_entity) {
                return BpCastResult::new(
                    true,
                    grid_transform.clone(),
                    snap_result.local_transform, 
                    snap_result.grid_entity,
                );
            }
        }

        // snap to world
        return BpCastResult::new(
            false,
            snap_result.local_transform,
            rot_in_place_transform,
            None,
        );
    }

    // no snap
    let cast_end = Transform::from_translation(mover_transform.translation + 
        mouse_forward * BUILD_DIST - bp_info.bottom);
    return BpCastResult::new(
        false,
        cast_end,
        rot_in_place_transform,
        None,
    );
}
