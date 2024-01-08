use crate::game_state::GameState;
use crate::loading::FontAssets;
use bevy::prelude::*;

// system state
#[derive(Default, Resource)]
pub struct BuildingToolbarState {
    pub ui_entity: Option<Entity>,
}

// Tag for UI component
#[derive(Component)]
struct BuildingToolbarText;

#[derive(Default)]
pub struct BuildingToolbarPlugin;

impl Plugin for BuildingToolbarPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BuildingToolbarState::default());

        app.add_systems(OnEnter(GameState::Running), setup_building_toolbar);
        // app.add_systems(Update, update_interactable_enter_exit.run_if(in_state(GameState::Running)));
        // app.add_systems(Update, update_interactable_interaction.run_if(in_state(GameState::Running)));
        // app.add_systems(Update, update_mouse_click_interaction.run_if(in_state(GameState::Running)));
        app.add_systems(OnExit(GameState::Running), exit_building_toolbar);
    }
}

fn setup_building_toolbar(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    mut system_state: ResMut<BuildingToolbarState>,
) {
    system_state.ui_entity = Some(commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Auto, 
                height: Val::Px(100.0),
                margin: UiRect::all(Val::Px(10.)),
                justify_content: JustifyContent::Center,
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
                        value: "Build Toolbar TODO".to_string(),
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
            .insert(BuildingToolbarText)
            ;
        })
        .id());
}


fn exit_building_toolbar(
    mut commands: Commands,
    system_state: Res<BuildingToolbarState>,
) {
    if let Some(ui_entity) = system_state.ui_entity {
        commands.entity(ui_entity).despawn_recursive();();
    }
}
