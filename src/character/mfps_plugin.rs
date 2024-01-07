use crate::actions::Actions;
use crate::loading::{WorldProps};
use crate::GameState;
use crate::inputs::{KeyInputState,MouseCamera,MouseLookState};
use crate::character::{CharacterFpsArmsPlugin,MfpsArmsSceneHandler,MfpsArmsAnimationsHandler};

use bevy::prelude::*;
use bevy::gltf::Gltf;
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaAnimatingStateDirective, TnuaGhostPlatform, TnuaGhostSensor,
    TnuaProximitySensor, TnuaToggle,
};
use bevy_tnua::builtins::{
    TnuaBuiltinCrouch, TnuaBuiltinCrouchState, TnuaBuiltinDash, TnuaBuiltinJumpState,
};
use bevy_tnua::control_helpers::{
    TnuaCrouchEnforcer, TnuaCrouchEnforcerPlugin, TnuaSimpleAirActionsCounter,
    TnuaSimpleFallThroughPlatformsHelper,
};
use bevy_tnua_rapier3d::*;

pub struct CharacterFpsPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CharacterFpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CharacterFpsArmsPlugin::default()));
        app.add_systems(OnEnter(GameState::Running), setup_player);
        app.add_systems(Update, apply_controls.in_set(TnuaUserControlsSystemSet).run_if(in_state(GameState::Running)));
        app.add_systems(Update, animation_patcher_system.run_if(in_state(GameState::Running)));
        app.add_systems(Update, animate.run_if(in_state(GameState::Running)));
        app.add_systems(Update, update_camera_sync.run_if(in_state(GameState::Running)));
    }
}

#[derive(Component)]
pub struct CharacterMotionConfigForPlatformerExample {
    pub speed: f32,
    pub walk: TnuaBuiltinWalk,
    pub actions_in_air: usize,
    pub jump: TnuaBuiltinJump,
    pub crouch: TnuaBuiltinCrouch,
    pub dash_distance: f32,
    pub dash: TnuaBuiltinDash,
}

fn setup_player(
    mut commands: Commands, 
    world_props: Res<WorldProps>,
    camera_query: Query<Entity, With<MouseCamera>>,
) {
    let mut cmd = commands.spawn(SpatialBundle {
        visibility: Visibility::Visible,
        ..default()
    });
    cmd.insert(TransformBundle {
        local: Transform::from_xyz(0.0, 10.0, 0.0),
        ..Default::default()
    });
    cmd.with_children(|parent| {
        // legs
        parent.spawn(SceneBundle {
            scene: world_props.mfps_legs_scene_handle.clone(),
            transform: Transform::from_xyz(0., -1., 0.),
            ..Default::default()
        }).insert(MfpsLegsSceneHandler {
            names_from: world_props.mfps_legs_handle.clone(),
        });
    });
    
    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Collider::capsule_y(0.5, 0.5));
    cmd.insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_1));
    cmd.insert(TnuaRapier3dIOBundle::default());
    cmd.insert(TnuaControllerBundle::default());
    cmd.insert(CharacterMotionConfigForPlatformerExample {
        speed: 10.0,
        walk: TnuaBuiltinWalk {
            float_height: 2.0,
            ..Default::default()
        },
        actions_in_air: 1,
        jump: TnuaBuiltinJump {
            height: 4.0,
            ..Default::default()
        },
        crouch: TnuaBuiltinCrouch {
            float_offset: -0.9,
            ..Default::default()
        },
        dash_distance: 10.0,
        dash: Default::default(),
    });
    cmd.insert(TnuaToggle::default());
    cmd.insert(TnuaCrouchEnforcer::new(0.5 * Vec3::Y, |cmd| {
        cmd.insert(TnuaRapier3dSensorShape(Collider::cylinder(0.0, 0.5)));
    }));
    cmd.insert(TnuaGhostSensor::default());
    cmd.insert(TnuaSimpleFallThroughPlatformsHelper::default());
    cmd.insert(TnuaSimpleAirActionsCounter::default());
    cmd.insert(TnuaAnimatingState::<AnimationState>::default());
    cmd.insert(TnuaRapier3dSensorShape(Collider::cylinder(0.0, 0.51)));
    // TODO
    // cmd.insert(LockedAxes::new().lock_rotation_x().lock_rotation_z());
    cmd.insert(CollisionGroups {
        memberships: Group::GROUP_1,
        filters: Group::GROUP_1,
    });
    // cmd.insert(common::ui::TrackedEntity("Player".to_owned()));
    // cmd.insert(PlotSource::default());
}

#[allow(clippy::type_complexity)]
fn apply_controls(
    // mut egui_context: EguiContexts,
    key_state: Res<KeyInputState>,
    mut query: Query<(
        &CharacterMotionConfigForPlatformerExample,
        &mut TnuaController,
        &mut TnuaCrouchEnforcer,
        &mut TnuaProximitySensor,
        &TnuaGhostSensor,
        &mut TnuaSimpleFallThroughPlatformsHelper,
        // &FallingThroughControlScheme,
        &mut TnuaSimpleAirActionsCounter,
    )>,
    mouse_look: Res<MouseLookState>,
) {
    // if egui_context.ctx_mut().wants_keyboard_input() {
    //     for (_, mut controller, ..) in query.iter_mut() {
    //         controller.neutralize_basis();
    //     }
    //     return;
    // }

    let mouse_forward = (mouse_look.forward * Vec3::new(1.0, 0.0, 1.0)).normalize();
    let mouse_right = (mouse_look.right * Vec3::new(1.0, 0.0, 1.0)).normalize();
    let direction = (
        if key_state.forward { mouse_forward } else { Vec3::ZERO } +
        if key_state.backward { -mouse_forward } else { Vec3::ZERO } +
        if key_state.right { mouse_right } else { Vec3::ZERO } +
        if key_state.left { -mouse_right } else { Vec3::ZERO }
    ).clamp_length_max(1.0);

    let jump = key_state.jump;
    let dash = key_state.run;

    // let turn_in_place = keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

    let crouch = key_state.down;
    // let crouch_just_pressed = keyboard.any_just_pressed(crouch_buttons);

    for (
        config,
        mut controller,
        mut crouch_enforcer,
        mut sensor,
        ghost_sensor,
        mut fall_through_helper,
        // falling_through_control_scheme,
        mut air_actions_counter,
    ) in query.iter_mut()
    {
        // println!("{}", ghost_sensor.iter().count());
        air_actions_counter.update(controller.as_mut());

        // let crouch = falling_through_control_scheme.perform_and_check_if_still_crouching(
        //     crouch,
        //     crouch_just_pressed,
        //     fall_through_helper.as_mut(),
        //     sensor.as_mut(),
        //     ghost_sensor,
        //     1.0,
        // );

        let speed_factor =
            if let Some((_, state)) = controller.concrete_action::<TnuaBuiltinCrouch>() {
                if matches!(state, TnuaBuiltinCrouchState::Rising) {
                    1.0
                } else {
                    0.2
                }
            } else {
                1.0
            };

        controller.basis(TnuaBuiltinWalk {
            desired_velocity: direction * speed_factor * config.speed,
            desired_forward: direction.normalize_or_zero(),
            ..config.walk.clone()
        });

        if crouch {
            controller.action(crouch_enforcer.enforcing(config.crouch.clone()));
        }

        if jump {
            controller.action(TnuaBuiltinJump {
                allow_in_air: air_actions_counter.air_count_for(TnuaBuiltinJump::NAME)
                    <= config.actions_in_air,
                ..config.jump.clone()
            });
        }

        if dash {
            controller.action(TnuaBuiltinDash {
                displacement: direction.normalize() * config.dash_distance,
                desired_forward: direction.normalize(),
                allow_in_air: air_actions_counter.air_count_for(TnuaBuiltinDash::NAME)
                    <= config.actions_in_air,
                ..config.dash.clone()
            });
        }
    }
}

#[derive(Component)]
struct MfpsLegsSceneHandler {
    names_from: Handle<Gltf>,
}

#[derive(Component)]
pub struct AnimationsHandler {
    pub player_entity: Entity,
    pub animations: HashMap<String, Handle<AnimationClip>>,
}

fn animation_patcher_system(
    animation_players_query: Query<Entity, Added<AnimationPlayer>>,
    parents_query: Query<&Parent>,
    scene_handlers_query: Query<&MfpsLegsSceneHandler>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for player_entity in animation_players_query.iter() {
        let mut entity = player_entity;
        loop {
            if let Ok(MfpsLegsSceneHandler { names_from }) = scene_handlers_query.get(entity) {
                let gltf = gltf_assets.get(names_from).unwrap();
                let mut cmd = commands.entity(entity);
                cmd.remove::<MfpsLegsSceneHandler>();
                cmd.insert(AnimationsHandler {
                    player_entity,
                    animations: gltf.named_animations.clone(),
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

#[derive(Debug)]
enum AnimationState {
    Standing,
    Running(f32),
    Jumping,
    Falling,
    Crouching,
    Crawling(f32),
    Dashing,
}

fn animate(
    mut tnua_query: Query<(
        &mut TnuaAnimatingState<AnimationState>,
        &TnuaController,
    )>,
    mut animations_handlers_query: Query<(
        &AnimationsHandler,
    )>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
) {
    
    let (mut animating_state, controller) = tnua_query.single_mut();
    for (handler,) in animations_handlers_query.iter_mut() {
        let Ok(mut player) = animation_players_query.get_mut(handler.player_entity) else {
            continue;
        };

        match animating_state.update_by_discriminant({
            match controller.action_name() {
                Some(TnuaBuiltinJump::NAME) => {
                    let (_, jump_state) = controller
                        .concrete_action::<TnuaBuiltinJump>()
                        .expect("action name mismatch");
                    match jump_state {
                        TnuaBuiltinJumpState::NoJump => continue,
                        TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
                        TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => {
                            AnimationState::Jumping
                        }
                        TnuaBuiltinJumpState::MaintainingJump => AnimationState::Jumping,
                        TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
                        TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
                    }
                }
                Some(TnuaBuiltinCrouch::NAME) => {
                    let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>()
                    else {
                        continue;
                    };
                    let speed =
                        Some(basis_state.running_velocity.length()).filter(|speed| 0.01 < *speed);
                    let is_crouching = basis_state.standing_offset < -0.4;
                    match (speed, is_crouching) {
                        (None, false) => AnimationState::Standing,
                        (None, true) => AnimationState::Crouching,
                        (Some(speed), false) => AnimationState::Running(0.1 * speed),
                        (Some(speed), true) => AnimationState::Crawling(0.1 * speed),
                    }
                }
                Some(TnuaBuiltinDash::NAME) => AnimationState::Dashing,
                Some(other) => panic!("Unknown action {other}"),
                None => {
                    let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>()
                    else {
                        continue;
                    };
                    if basis_state.standing_on_entity().is_none() {
                        AnimationState::Falling
                    } else {
                        let speed = basis_state.running_velocity.length();
                        if 0.01 < speed {
                            AnimationState::Running(0.1 * speed)
                        } else {
                            AnimationState::Standing
                        }
                    }
                }
            }
        }) {
            TnuaAnimatingStateDirective::Maintain { state } => match state {
                AnimationState::Running(speed) | AnimationState::Crawling(speed) => {
                    player.set_speed(*speed);
                }
                AnimationState::Jumping | AnimationState::Dashing => {
                    if controller.action_flow_status().just_starting().is_some() {
                        player.seek_to(0.0);
                    }
                }
                _ => {}
            },
            TnuaAnimatingStateDirective::Alter {
                old_state: _,
                state,
            } => match state {
                AnimationState::Standing => {
                    player
                        .start(handler.animations["Idle"].clone_weak())
                        .set_speed(1.0)
                        .repeat();
                }
                AnimationState::Running(speed) => {
                    player
                        .start(handler.animations["Running"].clone_weak())
                        .set_speed(*speed)
                        .repeat();
                }
                AnimationState::Jumping => {
                    // player
                    //     .start(handler.animations["Jumping"].clone_weak())
                    //     .set_speed(2.0);
                }
                AnimationState::Falling => {
                    // player
                    //     .start(handler.animations["Falling"].clone_weak())
                    //     .set_speed(1.0);
                }
                AnimationState::Crouching => {
                    // player
                    //     .start(handler.animations["Crouching"].clone_weak())
                    //     .set_speed(1.0)
                    //     .repeat();
                }
                AnimationState::Crawling(speed) => {
                    // player
                    //     .start(handler.animations["Crawling"].clone_weak())
                    //     .set_speed(*speed)
                    //     .repeat();
                }
                AnimationState::Dashing => {
                    // player
                    //     .start(handler.animations["Dashing"].clone_weak())
                    //     .set_speed(10.0);
                }
            },
        }
    }
}

fn update_camera_sync(
    time: Res<Time>,
    key_state: Res<KeyInputState>,
    // movement_state: Res<MovementState>,
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, &CharacterMotionConfigForPlatformerExample), Without<MouseCamera>>,
    mut query: Query<&mut Transform, With<MouseCamera>>,
) {
    let (mover_transform, _mover) = mover_query.single();
    for mut camera in query.iter_mut() {
        // if movement_state.noclip {
        //     let camera_move = CAMERA_FLY_MOVE_SPEED * time.delta_seconds() * (
        //         if key_state.forward { mouse_look.forward.clone() } else { Vec3::ZERO } +
        //         if key_state.backward { -mouse_look.forward.clone() } else { Vec3::ZERO } +
        //         if key_state.right { mouse_look.right.clone() } else { Vec3::ZERO } +
        //         if key_state.left { -mouse_look.right.clone() } else { Vec3::ZERO } +
        //         if key_state.up { mouse_look.up.clone() } else { Vec3::ZERO } +
        //         if key_state.down { -mouse_look.up.clone() } else { Vec3::ZERO }
        //     ) * (
        //         if key_state.run { 3.0 } else { 1.0 }
        //     );

        //     let next_position = camera.translation + camera_move;
        //     camera.translation = next_position.clone();
        //     camera.look_at(next_position + mouse_look.forward, Vec3::Y);
        // } else {
            // if mover.third_person {
            //     let mover_position = mover_transform.translation.clone() + 0.8 * Vec3::Y;
            //     camera.translation = mover_position - (mouse_look.forward * (1.0 + mouse_look.zoom));
            //     camera.look_at(mover_position, Vec3::Y);
            // } else {
                let mouse_forward = (mouse_look.forward * Vec3::new(1.0, 0.0, 1.0)).normalize();
                let mover_position = mover_transform.translation.clone() + 0.8 * Vec3::Y + 0.15 * mouse_forward;
                camera.translation = mover_position;
                camera.look_at(mover_position + mouse_look.forward, Vec3::Y);
        //     }
        // }
    }
}
