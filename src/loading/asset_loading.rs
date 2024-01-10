use crate::game_state::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Menu),
        )
        .add_collection_to_loading_state::<_, FontAssets>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, AudioAssets>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, IconAssets>(GameState::AssetLoading)
       .add_collection_to_loading_state::<_, TextureAssets>(GameState::AssetLoading)
        ;
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see https://github.com/NiklasEi/bevy_asset_loader)

#[derive(AssetCollection,Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection,Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/build_01.ogg")]
    pub build: Handle<AudioSource>,
}

#[derive(AssetCollection,Resource)]
pub struct IconAssets {
    #[asset(path = "textures/icon_metal_frame.png")]
    pub metal_frame: Handle<Image>,
    #[asset(path = "textures/icon_fuel_tank.png")]
    pub fuel_tank: Handle<Image>,
    #[asset(path = "textures/icon_nose_cone.png")]
    pub nose_cone: Handle<Image>,
    #[asset(path = "textures/icon_thruster.png")]
    pub thruster: Handle<Image>,
}

#[derive(AssetCollection,Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub texture_bevy: Handle<Image>,
    
    #[asset(path = "textures/array_texture.png")]
    pub texture_array: Handle<Image>,

    #[asset(path = "textures/explosion.png")]
    pub explosion_static: Handle<Image>,
}
