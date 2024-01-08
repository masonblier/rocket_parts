use crate::game_state::GameState;
use crate::inputs::MouseCamera;
use crate::loading::LoadingUiState;
use bevy::prelude::*;

pub struct PreLoadingPlugin;

#[derive(Default, Resource)]
pub struct PreLoadingState {
    pub pre_loaded: bool,
    pub ui_entity: Option<Entity>,
}

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for PreLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PreLoadingState>();
        app.add_systems(OnEnter(GameState::PreLoading), setup_camera);
        app.add_systems(Update, 
            update_pre_loading.run_if(in_state(GameState::PreLoading)));
    }
}

fn setup_camera(
    mut commands: Commands,
) {
    // Camera
    commands.spawn(SpatialBundle {
        ..default()
    }).with_children(|parent| {
        parent.spawn( Camera3dBundle {
            ..Default::default()
        })
        .insert(UiCameraConfig {
            show_ui: true,
            ..default()
        });
    })
    .insert(MouseCamera::default());
}

fn update_pre_loading(
    font_assets: Res<Assets<Font>>,
    mut pre_loading: ResMut<PreLoadingState>,
    mut state: ResMut<NextState<GameState>>,
    loading_ui_state: Res<LoadingUiState>,
) {
    let font_asset = font_assets.get(&loading_ui_state.font_handle);
    if pre_loading.pre_loaded || font_asset.is_none() {
        return;
    }

    info!("Pre loaded: {:?}", font_asset.unwrap());
    pre_loading.pre_loaded = true;
    state.set(GameState::AssetLoading);
}
