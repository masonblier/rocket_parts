use crate::actions::BuildingActionsState;
use crate::game_state::GameState;
use crate::loading::{FontAssets,IconAssets};
use bevy::prelude::*;


#[derive(Default)]
pub struct ToolbarItem {
    icon: Option<Handle<Image>>,
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
    pub toolbar_build_ent: Option<Entity>,
    pub toolbar_hammer_ent: Option<Entity>,
}

// Tag for UI component
#[derive(Component)]
struct BuildingToolbarText;

// Tag for UI component
#[derive(Component)]
pub struct ToolbarItemComp {
    pub toolbar_index: usize,
}

// Tag for UI component
#[derive(Component)]
pub struct ThrustersStatusText;

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
    let toolbar_tools = vec![
        ToolbarItem {
            icon: Some(icon_assets.hammer.clone()),
            text: "Hammer".into(),
        },
        ToolbarItem {
            icon: Some(icon_assets.unbuild.clone()),
            text: "Unbuild".into(),
        },
        ToolbarItem {
            icon: None,
            text: "None".into(),
        },
        ToolbarItem {
            icon: None,
            text: "".into(),
        },
    ];

    let toolbar_bps = vec![
        ToolbarItem {
            icon: Some(icon_assets.metal_frame.clone()),
            text: "Metal Frame".into(),
        },
        ToolbarItem {
            icon: Some(icon_assets.fuel_tank.clone()),
            text: "Fuel Tank".into(),
        },
        ToolbarItem {
            icon: Some(icon_assets.thruster.clone()),
            text: "Thruster".into(),
        },
        ToolbarItem {
            icon: Some(icon_assets.nose_cone.clone()),
            text: "Nose Cone".into(),
        },
        ToolbarItem {
            icon: Some(icon_assets.flight_seat.clone()),
            text: "Flight Seat".into(),
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

                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(300.), 
                            height: Val::Px(72.), 
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: Color::NONE.into(),
                        ..Default::default()
                    }).with_children(|parent| {
                        // tools toolbar
                        system_state.toolbar_hammer_ent = 
                        Some(parent.spawn(NodeBundle {
                            style: Style {
                                width: Val::Auto, 
                                height: Val::Auto, 
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                position_type: PositionType::Absolute,
                                ..Default::default()
                            },
                            background_color: Color::NONE.into(),
                            visibility: Visibility::Visible,
                            ..Default::default()
                        }).with_children(|parent| {
                            // toolbar icons
                            for (idx, toolbar_bp) in toolbar_tools.iter().enumerate() {
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
                                        let mut icon_ent = parent
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
                                            ));
                                            icon_ent
                                            .insert(ToolbarItemComp { toolbar_index: idx })
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
                                        if let Some(item_icon) = toolbar_bp.icon.clone() {
                                            icon_ent.insert(UiImage::new(item_icon));
                                        } else {
                                            icon_ent.insert(Visibility::Hidden);
                                        }

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
                        }).id());
                        
                        // building toolbar
                        system_state.toolbar_build_ent = 
                        Some(parent.spawn(NodeBundle {
                            style: Style {
                                width: Val::Auto, 
                                height: Val::Auto, 
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                position_type: PositionType::Absolute,
                                ..Default::default()
                            },
                            background_color: Color::NONE.into(),
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        }).with_children(|parent| {
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
                                        let mut icon_ent = parent
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
                                            ));
                                            icon_ent
                                            .insert(ToolbarItemComp { toolbar_index: idx })
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
                                        if let Some(item_icon) = toolbar_bp.icon.clone() {
                                            icon_ent.insert(UiImage::new(item_icon));
                                        } else {
                                            icon_ent.insert(Visibility::Hidden);
                                        }

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
                        }).id());
                    });

                    // build control tips
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
                        parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            text: Text {
                                sections: vec![TextSection {
                                    value: "R/T - Rotate".to_string(),
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
                        parent.spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: " ".to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 4.0,
                                        color: Color::NONE.into(),
                                    },
                                }],
                                ..default()
                            },
                            ..default()
                        });
                        parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            text: Text {
                                sections: vec![TextSection {
                                    value: "Z/X - Thrusters".to_string(),
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
                        
                        parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            text: Text {
                                sections: vec![TextSection {
                                    value: " ".to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 14.0,
                                        color: Color::rgba(0.9, 0.9, 0.9, 0.5),
                                    },
                                }],
                                linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                                alignment: TextAlignment::Left,
                            },
                            background_color: Color::NONE.into(),
                            ..Default::default()
                        }).insert(ThrustersStatusText { });
                    });
                });
            });
        })
    .id());
}

fn update_building_toolbar(
    building_actions: Res<BuildingActionsState>,
    mut system_state: ResMut<BuildingToolbarState>,
    mut icon_nodes: Query<(&ToolbarItemComp, &mut BackgroundColor)>,
    mut text_comps: Query<(Entity, &mut Text)>,
    mut vis_comps: Query<(Entity, &mut Visibility)>,
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
            let mut vis = vis_comps.get_mut(system_state.toolbar_hammer_ent.unwrap()).unwrap().1;
            vis.set(Box::new(Visibility::Hidden)).unwrap();
            let mut vis = vis_comps.get_mut(system_state.toolbar_build_ent.unwrap()).unwrap().1;
            vis.set(Box::new(Visibility::Visible)).unwrap();
        } else {
            let mut text_el = text_comps.get_mut(system_state.toolbar_build_select_text.unwrap()).unwrap().1;
            text_el.sections[0].style.color = 
                Color::rgba(0.9, 0.9, 0.9, 0.2);
            let mut text_el = text_comps.get_mut(system_state.toolbar_hammer_select_text.unwrap()).unwrap().1;
            text_el.sections[0].style.color = 
                Color::rgba(0.9, 0.9, 0.9, 0.5);
                let mut vis = vis_comps.get_mut(system_state.toolbar_hammer_ent.unwrap()).unwrap().1;
                vis.set(Box::new(Visibility::Visible)).unwrap();
                let mut vis = vis_comps.get_mut(system_state.toolbar_build_ent.unwrap()).unwrap().1;
                vis.set(Box::new(Visibility::Hidden)).unwrap();
        }

        // clear icons
        // if !system_state.bps_active {
        //     for (_bp_item, mut icon_color) in icon_nodes.iter_mut() {
        //         icon_color.0 = Color::rgba(0.1, 0.3, 0.7, 0.4).into();
        //     }
        //     system_state.active_index = None;
        // }
    }

    if Some(building_actions.active_index) != system_state.active_index {
        system_state.active_index = Some(building_actions.active_index);
        for (bp_item, mut icon_color) in icon_nodes.iter_mut() {
            if bp_item.toolbar_index == building_actions.active_index {
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
