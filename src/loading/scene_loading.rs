use crate::game_state::GameState;
use crate::loading::{LoadingUiState};
use bevy::asset::LoadState;
use bevy::{prelude::*, gltf::Gltf};

pub struct SceneLoadingPlugin;

#[derive(Default, Resource)]
pub struct SceneLoadingState {
    pub loaded: bool,
}

#[derive(Default, Resource)]
pub struct WorldProps {
    pub mfps_arms_handle: Handle<Gltf>,
    pub mfps_legs_handle: Handle<Gltf>,
    pub mfps_arms_scene_handle: Handle<Scene>,
    pub mfps_legs_scene_handle: Handle<Scene>,
    
    pub building_kit: Handle<Gltf>,
}

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for SceneLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SceneLoadingState>()
            .init_resource::<WorldProps>();
        app.add_systems(OnEnter(GameState::SceneLoading), setup_scene_loading);
        app.add_systems(Update, 
            update_scene_loading.run_if(in_state(GameState::SceneLoading)));
    }
}

fn setup_scene_loading(
    mut commands: Commands,
    mut loading_ui_state: ResMut<LoadingUiState>,
    mut world_props: ResMut<WorldProps>,
    asset_server: Res<AssetServer>,
) {
    world_props.mfps_arms_handle = asset_server.load("character/mfps_arms.glb");
    world_props.mfps_legs_handle = asset_server.load("character/mfps_legs.glb");
    world_props.mfps_arms_scene_handle = asset_server.load("character/mfps_arms.glb#Scene0");
    world_props.mfps_legs_scene_handle = asset_server.load("character/mfps_legs.glb#Scene0");
    
    world_props.building_kit = asset_server.load("props/building_kit.glb");
}

fn update_scene_loading(
    mut scene_loading: ResMut<SceneLoadingState>,
    mut state: ResMut<NextState<GameState>>,
    loading_ui_state: Res<LoadingUiState>,
        world_props: Res<WorldProps>,
    asset_server: Res<AssetServer>,
) {
    if scene_loading.loaded {
        return;
    }

    if asset_server.load_state(&world_props.mfps_arms_handle) != LoadState::Loaded ||
        asset_server.load_state(&world_props.mfps_legs_handle) != LoadState::Loaded ||
        asset_server.load_state(&world_props.mfps_arms_scene_handle) != LoadState::Loaded ||
        asset_server.load_state(&world_props.mfps_legs_scene_handle) != LoadState::Loaded ||
        asset_server.load_state(&world_props.building_kit) != LoadState::Loaded
    {
        return;
    }
    info!("Scene loaded!");

    scene_loading.loaded = true;
    state.set(GameState::WorldInit);
}
