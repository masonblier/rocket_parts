use crate::loading::{FontAssets,LoadingUiEvent,LoadingUiEventAction};
use crate::game_state::GameState;
use bevy::prelude::*;

// system state
#[derive(Default, Resource)]
pub struct CreditsState {
    pub ui_entity: Option<Entity>,
    pub time_left: f32,
}


// Tags for UI components
#[derive(Component)]
struct CreditsText;

// plugin
pub struct CreditsStatePlugin;

impl Plugin for CreditsStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(CreditsState::default());
    
        app.add_systems(OnEnter(GameState::Credits), setup_credits_interaction);
        app.add_systems(Update, 
            update_credits_interaction.run_if(in_state(GameState::Credits)));
    }
}

fn setup_credits_interaction(
    mut commands: Commands,
    mut credits_state: ResMut<CreditsState>,
    font_assets: Res<FontAssets>,
    mut loading_ui_events: EventWriter<LoadingUiEvent>,
) {
    // hide loading ui
    loading_ui_events.send(LoadingUiEvent {
        action: LoadingUiEventAction::Hide,
        payload: None,
    });
    // loading ui
    credits_state.ui_entity = Some(commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0), 
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Percent(1.),Val::Percent(1.),Val::Percent(1.),Val::Percent(1.)),
                ..default()
            },
            background_color: Color::rgb(0.2, 0.2, 0.2).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Thanks for playing Graham's Relay".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                    alignment: TextAlignment::Center,
                },
                ..Default::default()
            })
            .insert(CreditsText { })
            ;
        }).id());
    // default time
    credits_state.time_left = 5.0;
}

fn update_credits_interaction(
    mut commands: Commands,
    mut credits_state: ResMut<CreditsState>,
    mut text_query: Query<&mut Text, With<CreditsText>>,
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    credits_state.time_left -= time.delta_seconds();

    if credits_state.time_left > 4.0 {
        let mut text = text_query.single_mut();
        let t = (5.0 - credits_state.time_left).max(0.0).min(1.0) * 0.8 + 0.2;
        text.sections[0].style.color = Color::rgb(t, t, t);
    }
    if credits_state.time_left < 1.0 {
        let mut text = text_query.single_mut();
        let t = credits_state.time_left;
        text.sections[0].style.color = Color::rgb(t, t, t);
    }
    if credits_state.time_left < f32::EPSILON {
        commands.entity(credits_state.ui_entity.unwrap()).despawn_recursive();
        game_state.set(GameState::Menu);
    }
}
