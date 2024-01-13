use crate::inputs::CursorLockState;
use crate::loading::{FontAssets,LoadingUiState,LoadingUiEvent,LoadingUiEventAction};
use crate::game_state::GameState;
use crate::menu::{CreditsStatePlugin,PauseMenuStatePlugin};
use crate::world::WorldState;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

// system state
#[derive(Default, Resource)]
pub struct MainMenuState {
    pub ui_entity: Option<Entity>,
}


// marks which button was pressed
#[derive(Clone,Copy)]
pub enum MenuButtonWhich {
    PlayWorld01,
    // PlayWorld03,
}
#[derive(Clone,Component,Copy)]
pub struct MenuButton {
    pub which: MenuButtonWhich,
}

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((CreditsStatePlugin,
                PauseMenuStatePlugin))
            .insert_resource(MainMenuState::default());
        
        app.add_systems(OnEnter(GameState::Menu), setup_menu);
        app.add_systems(Update, 
            click_play_button.run_if(in_state(GameState::Menu)));
        app.add_systems(OnExit(GameState::Menu), exit_menu);
    }
}


#[derive(Component)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.25, 0.15, 0.15).into(),
            hovered: Color::rgb(0.55, 0.35, 0.25).into(),
        }
    }
}

#[derive(Component)]
struct ChangeState(GameState);

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    mut main_menu_state: ResMut<MainMenuState>,
    mut loading_ui_events: EventWriter<LoadingUiEvent>,
    mut cursor_lock_controls: ResMut<CursorLockState>,
    mut windows: Query<&mut Window>,
) {
    let button_colors = ButtonColors::default();

    // spawn menu
    main_menu_state.ui_entity = Some(commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0), 
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::new(Val::Px(0.),Val::Px(0.),Val::Px(0.),Val::Percent(10.),),
                    ..Default::default()
                },
                text: Text {
                    sections: vec![TextSection {
                        value: "rocket parts".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 60.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                    alignment: Default::default(),
                },
                ..Default::default()
            });
            
            parent
                .spawn((ButtonBundle {
                    style: Style {
                        width: Val::Percent(20.0), 
                        height: Val::Percent(10.0),
                        margin: UiRect::new(Val::Px(0.),Val::Px(0.),Val::Px(0.),Val::Percent(10.),),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: button_colors.normal.into(),
                    ..Default::default()
                },
                ButtonColors {
                    normal: Color::NONE,
                    hovered: Color::rgb(0.25, 0.25, 0.25),
                },
                ChangeState(GameState::SceneLoading)))
                .insert(MenuButton { which: MenuButtonWhich::PlayWorld01 })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "Play world01".to_string(),
                                style: TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            }],
                            linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                            alignment: Default::default(),
                        },
                        ..Default::default()
                    });
                });
        })
        .id());


    // hide loading ui
    loading_ui_events.send(LoadingUiEvent {
        action: LoadingUiEventAction::Hide,
        payload: None,
    });
    // exit cursor lock
    let mut window = windows.single_mut();
    if window.cursor.grab_mode != CursorGrabMode::None {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        cursor_lock_controls.enabled = false;
        // prevent most default browser keyboard interactions
        window.prevent_default_event_handling = true;
    }
}

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut world_state: ResMut<WorldState>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut cursor_lock_controls: ResMut<CursorLockState>,
    mut windows: Query<&mut Window>,
    mut vis_query: Query<&mut Visibility>,
    loading_ui_state: Res<LoadingUiState>,
    main_menu_state: Res<MainMenuState>,
) {
    for (interaction, mut color, button_colors, change_state) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(_state) = change_state {
                    world_state.active_world = "world01".into();
                    info!("Going to world01!");

                    next_state.set(GameState::SceneLoading);
                    // request cursor lock
                    let mut window = windows.single_mut();
                    window.cursor.grab_mode = CursorGrabMode::Locked;
                    window.cursor.visible = false;
                    cursor_lock_controls.enabled = true;
                    // hide menu ui
                    let mut vis = vis_query.get_mut(main_menu_state.ui_entity.unwrap()).unwrap();
                    vis.set(Box::new(Visibility::Hidden)).unwrap();
                    vis.set_changed();
                    // show loading ui
                    let mut vis = vis_query.get_mut(loading_ui_state.ui_entity.unwrap()).unwrap();
                    vis.set(Box::new(Visibility::Visible)).unwrap();
                    vis.set_changed();
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
}
}

fn exit_menu(
    mut commands: Commands,
    main_menu_state: Res<MainMenuState>,
) {
    // despawn ui
    if let Some(ui_entity) = main_menu_state.ui_entity {
        commands.entity(ui_entity).despawn_recursive();
    }
}
