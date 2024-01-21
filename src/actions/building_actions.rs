use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

use crate::building::BpInfos;

#[derive(Resource)]
pub struct BuildingActionsState {
    pub building_active: bool,
    pub active_index: usize,
    pub active_rotation: Quat,
    pub thrusters_power: f32,
}
impl Default for BuildingActionsState {
    fn default() -> BuildingActionsState {
        BuildingActionsState {
            building_active: false,
            active_index: 0,
            active_rotation: Quat::default(),
            thrusters_power: 11.,
        }
    }
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
        next_index += (mwe.y/mwe.y) as i32;
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
    if keyboard_input.just_pressed(KeyCode::Key5) {
        state.active_index = 4;
    }
    
    // check thrusters toggles
    if keyboard_input.just_pressed(KeyCode::Z) {
        state.thrusters_power *= 1.1;
    }
    if keyboard_input.just_pressed(KeyCode::X) {
        if state.thrusters_power > 12. {
            state.thrusters_power *= 0.909;
        }
    }

    // rotation toggles
    if state.building_active {
        if keyboard_input.just_pressed(KeyCode::T) {
            state.active_rotation = state.active_rotation.mul_quat(
                Quat::from_axis_angle(Vec3::Y, PI/2.));
        }
        if keyboard_input.just_pressed(KeyCode::R) {
            state.active_rotation = state.active_rotation.mul_quat(
                Quat::from_axis_angle(Vec3::Z, PI/2.));
        }
    }
}
