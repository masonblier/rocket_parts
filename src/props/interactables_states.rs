use crate::actions::BuildingActionsState;
use crate::character::{CharacterFpsMotionConfig,MoverState};
use crate::game_state::GameState;
use crate::inputs::{MouseLookState,KeyInputState};
use crate::loading::FontAssets;
use crate::world::WORLD_GROUP;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const INTERACT_DIST: f32 = 1.8;
pub const INTERACT_GROUP: Group = Group::GROUP_4;

// system state
#[derive(Default, Resource)]
pub struct InteractablesState {
    ui_entity: Option<Entity>,
    status_text: Option<Entity>,
    cast_result: Option<(InteractableInfo,Entity)>,
}

// interactables cast component
#[derive(Clone,Component,Debug)]
pub struct InteractableInfo {
    pub hover_text: String,
}

// plugin
#[derive(Default)]
pub struct InteractablesStatePlugin;

impl Plugin for InteractablesStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InteractablesState::default());

        app.add_systems(OnEnter(GameState::Running), setup_interactables_ui);
        app.add_systems(Update, update_interactables_ui.run_if(in_state(GameState::Running)));
        app.add_systems(OnExit(GameState::Running), exit_interactables_ui);

        app.add_systems(Update, (
                update_interactables_cast.run_if(in_state(GameState::Running)),
            ));
    }
}


fn setup_interactables_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    mut system_state: ResMut<InteractablesState>,
) {
    system_state.ui_entity = Some(
        // column for rows of ui elements
        commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.), 
                height: Val::Percent(100.), 
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            system_state.status_text = Some(parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                text: Text {
                    sections: vec![TextSection {
                        value: "Interactables Status".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 24.0,
                            color: Color::rgba(1., 1., 1., 0.5),
                        },
                    }],
                    linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                    alignment: TextAlignment::Left,
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            }).id());
    }).id());
}

fn update_interactables_ui(
    system_state: Res<InteractablesState>,
    mut text_comps: Query<(Entity, &mut Text)>,
    mut vis_comps: Query<(Entity, &mut Visibility)>,
) {
    let mut vis = vis_comps.get_mut(system_state.status_text.unwrap()).unwrap().1;
    if !vis.eq(&Visibility::Hidden) {
        if system_state.cast_result.is_none() {
            vis.set(Box::new(Visibility::Hidden)).unwrap();
            return;
        }
    } else {
        if system_state.cast_result.is_none() {
            return;
        }
        vis.set(Box::new(Visibility::Visible)).unwrap();
    }

    
    let mut text_el = text_comps.get_mut(system_state.status_text.unwrap()).unwrap().1;
    text_el.sections[0].value = system_state.cast_result.as_ref().unwrap().0.hover_text.clone();
}

fn exit_interactables_ui(
    mut commands: Commands,
    system_state: Res<InteractablesState>,
) {
    if let Some(ui_entity) = system_state.ui_entity {
        commands.entity(ui_entity).despawn_recursive();();
    }
}

fn update_interactables_cast(
    mut interactables_state: ResMut<InteractablesState>,
    building_actions: Res<BuildingActionsState>,
    mouse_look: Res<MouseLookState>,
    mut mover_query: Query<(&Transform, &mut MoverState, With<CharacterFpsMotionConfig>)>,
    interactables_query: Query<&InteractableInfo>,
    rapier_context: Res<RapierContext>,
    key_state: Res<KeyInputState>,
) {
    // not while building
    if building_actions.building_active {
        interactables_state.cast_result = None;
        return;
    }
    
    // get interactable ray from player state
    let (mover_transform, mut mover_state, _mover) = mover_query.single_mut();    
    let cast_origin = mover_transform.translation + 0.8 * Vec3::Y;
 
    // not while already seated
    if mover_state.seated_in_next.is_some() {
        // unseat
        if key_state.action_use {        
            mover_state.seated_in_next = None;
        }
        return;
    }

    // raycast
    let ray_groups = CollisionGroups::new(INTERACT_GROUP | WORLD_GROUP, INTERACT_GROUP | WORLD_GROUP);
    let ray_filter: QueryFilter<'_> = QueryFilter { groups: Some(ray_groups), ..Default::default()};
    interactables_state.cast_result = None;
    if let Some((collided_entity, _intersection)) = rapier_context.cast_ray_and_get_normal(
            cast_origin, mouse_look.forward * INTERACT_DIST, INTERACT_DIST, true, ray_filter
    ) {
        // todo if collided entity is interactable.  update interactable state
        if let Ok(int_info) = interactables_query.get(collided_entity) {            
            interactables_state.cast_result = Some((int_info.clone(), collided_entity));
        }
    }

    if key_state.action_use {
        if interactables_state.cast_result.is_some() {
            mover_state.seated_in_next = Some(interactables_state.cast_result.as_ref().unwrap().1);
        }
    }
}
