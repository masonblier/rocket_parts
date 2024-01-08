use crate::inputs::CursorLockState;
use crate::game_state::GameState;
use crate::loading::FontAssets;
use crate::world::{AnimatableEvent,AnimatableEventAction,InteractableState,
    LightsEvent,LightsEventAction,
    SoundsEvent,SoundsEventAction,
    WorldFlagsEvent,WorldFlagsEventAction,WorldFlagsState};
use bevy::prelude::*;

const INITIAL_BLOCKED_DURATION: f32 = 0.4;
const INTERACTION_BLOCKED_DURATION: f32 = 2.2;

// system state
#[derive(Default, Resource)]
pub struct InteractablesState {
    pub active_interactable: Option<InteractableState>,
    pub active_interactable_entity: Option<Entity>,
    pub active_entered_interactable: Option<InteractableState>,
    pub active_entered_entity: Option<Entity>,
    pub ui_entity: Option<Entity>,
    pub blocked_rmn: f32,
}

// Tag for UI component
#[derive(Component)]
struct InteractablesOverlayText;

pub struct InteractableStatePlugin;

impl Plugin for InteractableStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(InteractablesState::default());

        app.add_systems(OnEnter(GameState::Running), setup_interactable_interaction);
        // app.add_systems(Update, update_interactable_enter_exit.run_if(in_state(GameState::Running)));
        // app.add_systems(Update, update_interactable_interaction.run_if(in_state(GameState::Running)));
        app.add_systems(Update, update_mouse_click_interaction.run_if(in_state(GameState::Running)));
        app.add_systems(OnExit(GameState::Running), exit_interactable_interaction);
    }
}

fn setup_interactable_interaction(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    mut interactables_state: ResMut<InteractablesState>,
) {
    interactables_state.ui_entity = Some(commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(220.0), 
                height: Val::Px(50.0),
                margin: UiRect::all(Val::Px(10.)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "B - Build Tool\nZ - Toggle Thrusters".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 24.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                    alignment: TextAlignment::Left,
                },
                ..Default::default()
            })
            .insert(InteractablesOverlayText)
            ;
        })
        .id());

    // delay inputs when first entering Running state
    interactables_state.blocked_rmn = INITIAL_BLOCKED_DURATION;
}

// fn update_interactable_enter_exit(
//     mut interactables_state: ResMut<InteractablesState>,
//     mut world_state: ResMut<WorldState>,
//     mover_parent_query: Query<&GlobalTransform, With<MoverParent>>,
//     rapier_context: Res<RapierContext>,
//     mut game_state: ResMut<State<GameState>>,
// ) {
//     // get interactable ray from player state
//     let mover_parent_transform = mover_parent_query.single();
//     let mover_pos = mover_parent_transform.translation() + 0.8 * Vec3::Y;

//     let ray_groups = InteractionGroups::new(0b0100, 0b0100);
//     let ray_filter = QueryFilter { groups: Some(ray_groups), ..Default::default()};

//     // cast for interactables
//     let (entity, interactable) = if interactables_state.blocked_rmn > 0.0001 {
//         (None, None)
//     } else if let Some((entity, _toi)) = rapier_context.cast_ray(
//         mover_pos, -0.01*Vec3::Y, 1.0, true, ray_filter
//     ) {
//         if let Some(interactable) = world_state.interactable_states.get(&entity) {
//             (Some(entity), Some(interactable.clone()))
//         } else { (None, None) }
//     } else { (None, None) };

//     // if active interactable changed
//     if interactables_state.active_entered_entity != entity {
//         interactables_state.active_entered_entity = entity;
//         interactables_state.active_entered_interactable = interactable;

//         if let Some(interactable) = &interactables_state.active_entered_interactable {
//             // enter interaction
//             if interactable.interaction.interaction == "enter" {
//                 for action in interactable.interaction.actions.iter() {
//                     if action.0 == "load_world" {
//                         world_state.active_world = "credits".into();
//                         game_state.set(GameState::WorldInit).unwrap();
//                     }
//                 }
//             }
//         } else {
//             // exit interaction
//         }
//     }
// }

// fn update_interactable_interaction(
//     cursor_lock_state: Res<CursorLockState>,
//     world_flags_state: Res<WorldFlagsState>,
//     mut interactables_state: ResMut<InteractablesState>,
//     world_state: Res<WorldState>,
//     camera_query: Query<&GlobalTransform, With<MouseCamera>>,
//     // mover_parent_query: Query<&GlobalTransform, With<MoverParent>>,
//     mouse_look: Res<MouseLookState>,
//     mut text_query: Query<&mut Text, With<InteractablesOverlayText>>,
//     // mover_query: Query<&Mover>,
// ) {
//     if !cursor_lock_state.enabled {
//         return;
//     }
    
//     // get interactable ray from player state
//     let mover = mover_query.single();
//     let mover_parent_transform = mover_parent_query.single();
//     let camera_transform = camera_query.single();
//     let ray_pos = if mover.third_person {
//         mover_parent_transform.translation() + 0.8 * Vec3::Y
//     } else {
//         camera_transform.translation()
//     };
//     let ray_len = if mover.third_person { 1.2 } else { 1.7 } ;
//     let ray_dir = if mover.third_person {
//         -mover_parent_transform.forward() * ray_len
//     } else {
//         mouse_look.forward * ray_len
//     };

//     let ray_groups = InteractionGroups::new(0b0100, 0b0100);
//     let ray_filter = QueryFilter { groups: Some(ray_groups), ..Default::default()};

//     // cast for interactables
//     let (entity, interactable) = if interactables_state.blocked_rmn > 0.0001 {
//         (None, None)
//     } else if let Some((entity, _toi)) = rapier_context.cast_ray(
//         ray_pos, ray_dir, 1.0, true, ray_filter
//     ) {
//         if let Some(interactable) = world_state.interactable_states.get(&entity) {
//             (Some(entity), Some(interactable.clone()))
//         } else { (None, None) }
//     } else { (None, None) };

//     // if active interactable changed
//     if interactables_state.active_interactable_entity != entity {
//         interactables_state.active_interactable_entity = entity;
//         interactables_state.active_interactable = interactable;

//         if let Some(interactable) = &interactables_state.active_interactable {
//             // check blockers
//             let blockers = check_blockers(interactable.interaction.blockers.clone(),
//                 inventory_state, world_flags_state);

//             if let Some(first_blocker) = blockers.first() {
//                 // show blocker text
//                 let mut text = text_query.single_mut();
//                 text.sections[0].value = "\n\n\nx\n\n".to_string() + &first_blocker.1;
//             } else {
//                 // show interaction text
//                 let mut text = text_query.single_mut();
//                 text.sections[0].value = "\n\n\n.\n\n".to_string() + &interactable.interaction.interaction_text;
//             }
//         } else {
//             // hide interaction text
//             let mut text = text_query.single_mut();
//             text.sections[0].value = "".to_string();
//         }
//     }
// }


fn update_mouse_click_interaction(
    mut commands: Commands,
    cursor_lock_state: Res<CursorLockState>,
    mouse_button_input: Res<Input<MouseButton>>,
    // mut movement_state: ResMut<MovementState>,
    mut animatable_events: EventWriter<AnimatableEvent>,
    mut lights_events: EventWriter<LightsEvent>,
    mut sounds_events: EventWriter<SoundsEvent>,
    mut world_flags_events: EventWriter<WorldFlagsEvent>,
    // audio_assets: Res<AudioAssets>,
    mut interactables_state: ResMut<InteractablesState>,
    world_flags_state: Res<WorldFlagsState>,
    time: Res<Time>,
) {
    if !cursor_lock_state.enabled {
        return;
    }

    if interactables_state.blocked_rmn > 0.0001 {
        interactables_state.blocked_rmn -= time.delta_seconds();
    }

    // check for mouse button press
    if mouse_button_input.just_pressed(MouseButton::Left) && interactables_state.blocked_rmn <= 0.0001 {
        if let Some(interactable) = &interactables_state.active_interactable {

            // check blockers
            let blockers = check_blockers(interactable.interaction.blockers.clone(),
                world_flags_state);
            if blockers.len() > 0 {
                return;
            }

            // send action events
            for action in interactable.interaction.actions.iter() {
                match action.0.as_str() {
                    // "audio_playonce" => {
                    //     audio_events.send(AudioEvent {
                    //         action: AudioEventAction::PlayOnce,
                    //         source: Some(audio_assets.big_switch.clone()),
                    //     });
                    // },
                    "animate" => {
                        let parts = action.1.split(".").collect::<Vec<&str>>();
                        let animatable_name = parts[0].to_string();
                        let animation_name = parts[1].to_string();
                        animatable_events.send(AnimatableEvent {
                            action: AnimatableEventAction::PlayOnce,
                            name: animatable_name,
                            animation: animation_name,
                        });
                    },
                    "toggle_light" => {
                        lights_events.send(LightsEvent {
                            action: LightsEventAction::Toggle,
                            name: action.1.to_string(),
                        });
                    },
                    "toggle_sound" => {
                        sounds_events.send(SoundsEvent {
                            action: SoundsEventAction::Toggle,
                            name: action.1.to_string(),
                        });
                    },
                    "enable_flag" => {
                        world_flags_events.send(WorldFlagsEvent {
                            action: WorldFlagsEventAction::Enable,
                            flag: action.1.clone(),
                        });
                    },
                    "hide_prop" => {
                        animatable_events.send(AnimatableEvent {
                            action: AnimatableEventAction::Despawn,
                            name: action.1.clone(),
                            animation: "".to_string(),
                        });
                    },
                    "despawn_self" => {
                        commands.entity(
                            interactables_state.active_interactable_entity.unwrap()).despawn();
                    },
                    _ => {
                        println!("Unknown interaction! {:?}", action);
                    },
                }
            }

            // block interaction for time
            interactables_state.blocked_rmn = INTERACTION_BLOCKED_DURATION;
            // todo proper event or something
            // movement_state.toggle_switch_rmn = INTERACTION_BLOCKED_DURATION;
        }
    }
}

fn check_blockers(
    blockers: Vec<(String, String)>,
    world_flags_state: Res<WorldFlagsState>,
) -> Vec<(String,String)> {
    blockers.clone().into_iter().filter({|blocker|
        if blocker.0.starts_with("flag_enabled") {
            let flag_name = blocker.0.split(".").collect::<Vec<&str>>()[1];
            let flag_wrapped = world_flags_state.flags.get(&flag_name.to_string());
            !(flag_wrapped.is_some() && *flag_wrapped.unwrap())
        } else {
            println!("invalid blocker {:?}", blocker);
            false
        }
    }).collect::<Vec<(String,String)>>()
}

fn exit_interactable_interaction(
    mut commands: Commands,
    interactables_state: Res<InteractablesState>,
) {
    if let Some(ui_entity) = interactables_state.ui_entity {
        commands.entity(ui_entity).despawn_recursive();();
    }
}
