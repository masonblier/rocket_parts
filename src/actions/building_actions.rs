use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

use crate::building::BpInfos;

#[derive(Default, Resource)]
pub struct BuildingActionsState {
    pub building_active: bool,
    pub active_index: usize,
    pub thrusters_active: bool,
}


pub fn update_building_actions_state(
    mut state: ResMut<BuildingActionsState>,
    infos: Res<BpInfos>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    if keyboard_input.just_pressed(KeyCode::B) {
        state.building_active = !state.building_active;
    }
    if keyboard_input.just_pressed(KeyCode::H) {
        state.building_active = false;
    }

    let mut next_index = state.active_index as i32;
    for mwe in mouse_wheel_events.read() {
        next_index += mwe.y as i32;
        if next_index < 0 { next_index = (infos.toolbar_order.len() as i32) - 1};
        if next_index >= (infos.toolbar_order.len() as i32) { next_index = 0};
    }
    state.active_index = next_index as usize;

    // toolbar key press
    if keyboard_input.just_pressed(KeyCode::Key1) {
        state.active_index = 0;
    }
    if keyboard_input.just_pressed(KeyCode::Key2) {
        state.active_index = 1;
    }
    if keyboard_input.just_pressed(KeyCode::Key3) {
        state.active_index = 2;
    }
    if keyboard_input.just_pressed(KeyCode::Key4) {
        state.active_index = 3;
    }
    
    // check thrusters toggle
    if keyboard_input.just_pressed(KeyCode::Z) {
        state.thrusters_active = !state.thrusters_active;
    }
}
