use crate::actions::BuildingActionsState;
use crate::loading::WorldProps;
use crate::GameState;
use crate::inputs::MouseCamera;

use bevy::{prelude::*, pbr::NotShadowReceiver, render::view::NoFrustumCulling};
use bevy::gltf::Gltf;
use bevy::utils::HashMap;
use bevy_scene_hook::{HookedSceneBundle,SceneHook};

#[derive(Default)]
pub struct CharacterFpsArmsPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CharacterFpsArmsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::WorldLoading), setup_mfps_arms);
        app.add_systems(Update, mfps_arms_animation_patcher_system.run_if(in_state(GameState::Running)));
        app.add_systems(Update, animate_mfps_arms.run_if(in_state(GameState::Running)));

    }
}

#[derive(Component)]
pub struct MfpsArmsSceneHandler {
    pub names_from: Handle<Gltf>,
}

#[derive(Component)]
pub struct MfpsArmsAnimationsHandler {
    pub player_entity: Entity,
    pub animations: HashMap<String, Handle<AnimationClip>>,
    pub animation_state: AnimationState,
    pub active_index: i32,
}

fn setup_mfps_arms(
    mut commands: Commands, 
    world_props: Res<WorldProps>,
    camera_query: Query<Entity, With<MouseCamera>>,
) {
    let camera_entity = camera_query.single();
    commands.spawn(HookedSceneBundle {
        scene: SceneBundle {
            scene: world_props.mfps_arms_scene_handle.clone(),
            transform: Transform::from_xyz(0., -1.6, -0.2),
            ..Default::default()
        },
        hook: SceneHook::new(|entity, ent_commands| {
            if entity.get::<Handle<Mesh>>().is_some() {
                ent_commands.insert(NoFrustumCulling::default());
            }
        }),
    })
    .insert(MfpsArmsSceneHandler {
        names_from: world_props.mfps_arms_handle.clone(),
    })
    .set_parent(camera_entity)
    ;

    // skyball
    commands.spawn(HookedSceneBundle {
        scene: SceneBundle {
            scene: world_props.skyball.clone(),
            transform: Transform::from_scale(Vec3::splat(1000.)),
            ..Default::default()
        },
        hook: SceneHook::new(|entity, ent_commands| {
            if entity.get::<Handle<Mesh>>().is_some() {
                ent_commands.insert(NotShadowReceiver::default());
            }
        }),
    })
    ;
}

fn mfps_arms_animation_patcher_system(
    animation_players_query: Query<Entity, Added<AnimationPlayer>>,
    parents_query: Query<&Parent>,
    scene_handlers_query: Query<&MfpsArmsSceneHandler>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for player_entity in animation_players_query.iter() {
        let mut entity = player_entity;
        loop {
            if let Ok(MfpsArmsSceneHandler { names_from }) = scene_handlers_query.get(entity) {
                let gltf = gltf_assets.get(names_from).unwrap();
                let mut cmd = commands.entity(entity);
                cmd.remove::<MfpsArmsSceneHandler>();
                cmd.insert(MfpsArmsAnimationsHandler {
                    player_entity,
                    animations: gltf.named_animations.clone(),
                    animation_state: AnimationState::Init,
                    active_index: 0,
                });
                break;
            }
            entity = if let Ok(parent) = parents_query.get(entity) {
                **parent
            } else {
                break;
            };
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    Init,
    Idle,
    Seated,
    BuildToolOpen,
    BuildToolHold,
    BuildToolActive,
    UnbuildToolOpen,
    UnbuildToolHold,
}

const TOOL_STATES: [AnimationState; 3] = [
    AnimationState::BuildToolHold,
    AnimationState::UnbuildToolHold,
    AnimationState::Idle,
];
const TOOL_OPEN_STATES: [AnimationState; 3] = [
    AnimationState::BuildToolOpen,
    AnimationState::UnbuildToolOpen,
    AnimationState::Idle,
];

fn animate_mfps_arms(
    mut animations_handlers_query: Query<(
        &mut MfpsArmsAnimationsHandler,
    )>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
    building_actions: Res<BuildingActionsState>,
    mouse_btn_input: Res<Input<MouseButton>>,

) {
    for (mut handler,) in animations_handlers_query.iter_mut() {
        let Ok(mut player) = animation_players_query.get_mut(handler.player_entity) else {
            continue;
        };


        let mut change_action = handler.animation_state;

        if building_actions.building_active {
            change_action = AnimationState::Idle;
        } else {
            handler.active_index = (building_actions.active_index as i32).min(2);
            if handler.animation_state != TOOL_STATES[handler.active_index as usize] &&
            handler.animation_state != TOOL_OPEN_STATES[handler.active_index as usize] {
                change_action = TOOL_STATES[handler.active_index as usize];
            }

            if change_action == AnimationState::BuildToolHold && mouse_btn_input.pressed(MouseButton::Left) {
                change_action = AnimationState::BuildToolActive;
            }
        }

        // TODO open animation transitions

        // update animation player
        if change_action != handler.animation_state {
            match change_action {
                AnimationState::Init => { }
                AnimationState::Idle => {
                    player
                        .start(handler.animations["Idle"].clone_weak())
                        .set_speed(1.0)
                        .repeat();
                }
                AnimationState::Seated => {
                    player
                        .start(handler.animations["Seated"].clone_weak())
                        .set_speed(1.0)
                        .repeat();
                }
                AnimationState::BuildToolOpen => {
                    player
                        .start(handler.animations["BuildToolOpen"].clone_weak())
                        .repeat();
                }
                AnimationState::BuildToolHold => {
                    player
                        .start(handler.animations["BuildToolHold"].clone_weak())
                        .repeat();
                }
                AnimationState::BuildToolActive => {
                    player
                        .start(handler.animations["BuildToolActive"].clone_weak())
                        .repeat();
                }
                AnimationState::UnbuildToolOpen => {
                    player
                        .start(handler.animations["UnbuildToolOpen"].clone_weak())
                        .repeat();
                }
                AnimationState::UnbuildToolHold => {
                    player
                        .start(handler.animations["UnbuildToolHold"].clone_weak())
                        .repeat();
                }
            }

            handler.animation_state = change_action;
        }
    }
}
