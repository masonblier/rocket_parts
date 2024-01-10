use crate::inputs::CursorLockState;
use crate::game_state::GameState;
use crate::world::WorldState;
use bevy::prelude::*;

pub struct LightsStatePlugin;

pub enum LightsEventAction {
    #[allow(dead_code)]
    Toggle,
}

#[derive(Event)]
pub struct LightsEvent {
    pub action: LightsEventAction,
    pub name: String,
}

impl Plugin for LightsStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<LightsEvent>()
        .add_systems(Update, 
            update_light_interaction.run_if(in_state(GameState::Running)));
    }
}


fn update_light_interaction(
    cursor_lock_state: Res<CursorLockState>,
    mut world_state: ResMut<WorldState>,
    mut lights_events: EventReader<LightsEvent>,
    mut point_lights: Query<&mut PointLight>,
) {
    if !cursor_lock_state.enabled {
        return;
    }

    for lights_event in lights_events.read() {
        if let Some(light_entity) = world_state.animatable_lights.get_mut(&lights_event.name) {
            match lights_event.action {
                LightsEventAction::Toggle => {
                    let mut point_light = point_lights.get_mut(*light_entity).unwrap();
                    if point_light.intensity <= 0.0001 {
                        point_light.intensity = 50.;
                    } else {
                        point_light.intensity = 0.;
                    }
                }
            }
        }
    }
}
