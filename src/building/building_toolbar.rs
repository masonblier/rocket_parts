use crate::actions::BuildingActionsState;
use crate::building::BuildingState;
use crate::game_state::GameState;
use crate::loading::{FontAssets,IconAssets};
use bevy::prelude::*;


#[derive(Default)]
pub struct ToolbarBp {
    icon: Handle<Image>,
    text: String,
}

// system state
#[derive(Default, Resource)]
pub struct BuildingToolbarState {
    pub ui_entity: Option<Entity>,
    pub bps_active: bool,
    pub active_index: Option<usize>,
    pub toolbar_build_select_text: Option<Entity>,
    pub toolbar_hammer_select_text: Option<Entity>,
}

// Tag for UI component
#[derive(Component)]
struct BuildingToolbarText;

// Tag for UI component
#[derive(Component)]
pub struct ToolbarBpItem {
    pub toolbar_index: usize,
}

#[derive(Default)]
pub struct BuildingToolbarPlugin;

impl Plugin for BuildingToolbarPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(BuildingToolbarState::default());

        app.add_systems(OnEnter(GameState::Running), setup_building_toolbar);
        app.add_systems(Update, update_building_toolbar.run_if(in_state(GameState::Running)));
        app.add_systems(OnExit(GameState::Running), exit_building_toolbar);
    }
}

fn setup_building_toolbar(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    icon_assets: Res<IconAssets>,
    mut system_state: ResMut<BuildingToolbarState>,
) {
    let toolbar_bps = vec![
        ToolbarBp {
            icon: icon_assets.metal_frame.clone(),
            text: "Metal Frame".into(),
        },
        ToolbarBp {
            icon: icon_assets.fuel_tank.clone(),
            text: "Fuel Tank".into(),
        },
        ToolbarBp {
            icon: icon_assets.thruster.clone(),
            text: "Thruster".into(),
        },
        ToolbarBp {
            icon: icon_assets.nose_cone.clone(),
            text: "Nose Cone".into(),
        },
    ];

    system_state.ui_entity = Some(commands
        // column for rows of ui elements
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.), 
                height: Val::Percent(100.), 
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // horizontal toolbar bar row
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.), 
                    height: Val::Auto, 
                    margin: UiRect::all(Val::Px(10.)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                // toolbar box
                parent.spawn(NodeBundle {
                    style: Style {
                        width: Val::Auto, 
                        height: Val::Auto, 
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::rgba(0.3, 0.5, 0.9, 0.2).into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // toolbar/hammer mode select
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Auto, 
                            height: Val::Auto, 
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::axes(Val::Px(12.),Val::Px(2.),),
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: Color::NONE.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        system_state.toolbar_build_select_text = Some(parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            text: Text {
                                sections: vec![TextSection {
                                    value: "B - Build".to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 16.0,
                                        color: Color::rgba(0.9, 0.9, 0.9, 0.2),
                                    },
                                }],
                                linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                                alignment: TextAlignment::Left,
                            },
                            background_color: Color::rgba(0.3, 0.5, 0.9, 0.2).into(),
                            ..Default::default()
                        }).id());
                        parent.spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: " ".to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 10.0,
                                        color: Color::NONE.into(),
                                    },
                                }],
                                ..default()
                            },
                            ..default()
                        });
                        system_state.toolbar_hammer_select_text = Some(parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            text: Text {
                                sections: vec![TextSection {
                                    value: "H - Hammer".to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 16.0,
                                        color: Color::rgba(0.9, 0.9, 0.9, 0.5),
                                    },
                                }],
                                linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                                alignment: TextAlignment::Left,
                            },
                            background_color: Color::rgba(0.3, 0.5, 0.9, 0.2).into(),
                            ..Default::default()
                        }).id());
                    });
                    
                    // toolbar icons
                    for (idx, toolbar_bp) in toolbar_bps.iter().enumerate() {
                        // icon wrap
                        parent
                            .spawn(
                                NodeBundle {
                                    style: Style {
                                        width: Val::Px(64.0),
                                        height: Val::Px(64.0),
                                        ..default()
                                    },
                                    ..default()
                                }
                            )
                            .with_children(|parent| {

                                // icon
                                parent
                                    .spawn((
                                        NodeBundle {
                                            style: Style {
                                                width: Val::Px(64.0),
                                                height: Val::Px(64.0),
                                                margin: UiRect::axes(Val::Px(8.),Val::Px(2.),),
                                                ..default()
                                            },
                                            background_color: 
                                                Color::rgba(0.1, 0.3, 0.7, 0.4).into(),
                                            ..default()
                                        },
                                        UiImage::new(toolbar_bp.icon.clone()),
                                    ))
                                    .insert(ToolbarBpItem { toolbar_index: idx })
                                    .with_children(|parent| {
                                        // alt text
                                        // This UI node takes up no space in the layout and the `Text` component is used by the accessibility module
                                        // and is not rendered.
                                        parent.spawn((
                                            NodeBundle {
                                                style: Style {
                                                    display: Display::None,
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            Text::from_section(toolbar_bp.text.clone(), TextStyle::default()),
                                        ));
                                    });

                                // idx text
                                parent.spawn(TextBundle {
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        bottom: Val::Px(4.),
                                        right: Val::Px(4.),
                                        ..default()
                                    },
                                    text: Text {
                                        sections: vec![TextSection {
                                            value: (idx+1).to_string(),
                                            style: TextStyle {
                                                font: font_assets.fira_sans.clone(),
                                                font_size: 16.0,
                                                color: Color::rgba(0.9, 0.9, 0.9, 0.5),
                                            },
                                        }],
                                        linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                                        alignment: TextAlignment::Left,
                                    },
                                    background_color: Color::NONE.into(),
                                    ..Default::default()
                                });
                            });
                    }
                });
            });
        })
    .id());
}

fn update_building_toolbar(
    building_actions: Res<BuildingActionsState>,
    building_state: Res<BuildingState>,
    mut system_state: ResMut<BuildingToolbarState>,
    mut icon_nodes: Query<(&ToolbarBpItem, &mut BackgroundColor)>,
    mut text_comps: Query<(Entity, &mut Text)>,
) {
    if building_actions.building_active != system_state.bps_active {
        system_state.bps_active = building_actions.building_active;
        if system_state.bps_active {
            let mut text_el = text_comps.get_mut(system_state.toolbar_build_select_text.unwrap()).unwrap().1;
            text_el.sections[0].style.color = 
                Color::rgba(0.9, 0.9, 0.9, 0.5);
            let mut text_el = text_comps.get_mut(system_state.toolbar_hammer_select_text.unwrap()).unwrap().1;
            text_el.sections[0].style.color = 
                Color::rgba(0.9, 0.9, 0.9, 0.2);
        } else {
            let mut text_el = text_comps.get_mut(system_state.toolbar_build_select_text.unwrap()).unwrap().1;
            text_el.sections[0].style.color = 
                Color::rgba(0.9, 0.9, 0.9, 0.2);
            let mut text_el = text_comps.get_mut(system_state.toolbar_hammer_select_text.unwrap()).unwrap().1;
            text_el.sections[0].style.color = 
                Color::rgba(0.9, 0.9, 0.9, 0.5);
        }

        // clear icons
        if !system_state.bps_active {
            for (_bp_item, mut icon_color) in icon_nodes.iter_mut() {
                icon_color.0 = Color::rgba(0.1, 0.3, 0.7, 0.4).into();
            }
            system_state.active_index = None;
        }
    }

    if !building_actions.building_active {
        return;
    }

    if Some(building_state.active_index) != system_state.active_index {
        system_state.active_index = Some(building_state.active_index);
        for (bp_item, mut icon_color) in icon_nodes.iter_mut() {
            if bp_item.toolbar_index == building_state.active_index {
                icon_color.0 = Color::rgba(1., 1., 1., 1.,).into();
            } else {
                icon_color.0 = Color::rgba(0.1, 0.3, 0.7, 0.4).into();
            }
        }
    }
}

fn exit_building_toolbar(
    mut commands: Commands,
    system_state: Res<BuildingToolbarState>,
) {
    if let Some(ui_entity) = system_state.ui_entity {
        commands.entity(ui_entity).despawn_recursive();();
    }
}
