use crate::game_state::GameState;
use crate::inputs::CursorLockState;
use crate::loading::FontAssets;
use crate::menu::ButtonColors;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
// use bevy_rapier3d::prelude::*;

// system state
#[derive(Default, Resource)]
pub struct PauseMenuState {
    pub ui_entity: Option<Entity>,
}

// plugin
pub struct PauseMenuStatePlugin;

#[derive(Component)]
struct ChangeState(GameState);

impl Plugin for PauseMenuStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(PauseMenuState::default())
        ;
    
        app.add_systems(OnEnter(GameState::Paused), enter_pause_menu);
        app.add_systems(Update, 
            click_play_button.run_if(in_state(GameState::Paused)));
        app.add_systems(OnExit(GameState::Paused), exit_pause_menu);
    }
}

fn enter_pause_menu(
    mut commands: Commands,
    mut pause_menu_state: ResMut<PauseMenuState>,
    font_assets: Res<FontAssets>,
    // mut rapier_conf: ResMut<RapierConfiguration>,
    mut cursor_lock_controls: ResMut<CursorLockState>,
    mut windows: Query<&mut Window>,
) {
    // pause menu ui
    let button_colors = ButtonColors::default();
    pause_menu_state.ui_entity = Some(commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(160.0), 
                height: Val::Px(50.0),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: button_colors.normal.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Resume".to_string(),
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
        }).id());

    // pause physics
    // rapier_conf.physics_pipeline_active = false;

    // exit cursor lock
    let mut window = windows.single_mut();
    if window.cursor.grab_mode != CursorGrabMode::None {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        cursor_lock_controls.enabled = false;
    }
}

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
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
) {
    for (interaction, mut color, button_colors, _change_state) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Running);
                // request cursor lock
                let mut window = windows.single_mut();
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
                cursor_lock_controls.enabled = true;
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

fn exit_pause_menu(
    mut commands: Commands,
    pause_menu: Res<PauseMenuState>,
    // mut rapier_conf: ResMut<RapierConfiguration>,
) {
    // despawn ui
    if let Some(ui_entity) = pause_menu.ui_entity {
        commands.entity(ui_entity).despawn_recursive();
    }

    // resume physics
    // rapier_conf.physics_pipeline_active = true;
}
