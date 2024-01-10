use bevy::prelude::*;

use crate::GameState;

mod building_actions;
pub use building_actions::*;
mod game_control;
pub use game_control::*;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<GameControlActions>()
        .init_resource::<BuildingActionsState>()
        .add_systems(
            Update,
            set_movement_actions.run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update,
            update_building_actions_state.run_if(in_state(GameState::Running)),
        );
    }
}
