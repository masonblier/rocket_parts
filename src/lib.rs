#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod character;
mod game_state;
mod inputs;
mod loading;
mod menu;
mod building;
mod props;
mod world;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::game_state::GameState;
use crate::loading::{LoadingUiStatePlugin,PreLoadingPlugin,
    AssetLoadingPlugin,SceneLoadingPlugin};
use crate::menu::MenuPlugin;
use crate::character::CharacterFpsPlugin;
use crate::inputs::{KeyInputPlugin, MouseInputPlugin};
use crate::building::BuildingStatePlugin;
use crate::props::PropsStatesPlugin;
use crate::world::{WorldAssetLoaderPlugin,WorldLoadingPlugin,WorldStatePlugin,
    WorldTerrainPlugin};

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_rapier3d::*;
use bevy_tnua::control_helpers::TnuaCrouchEnforcerPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            TnuaRapier3dPlugin,
            TnuaControllerPlugin,
            TnuaCrouchEnforcerPlugin,
        ))
        .add_plugins((
            LoadingUiStatePlugin,
            PreLoadingPlugin,
            AssetLoadingPlugin,
            SceneLoadingPlugin,
            WorldAssetLoaderPlugin,
            WorldLoadingPlugin,
            KeyInputPlugin,
            MouseInputPlugin,
        ))
        .add_plugins((
            WorldStatePlugin,
            WorldTerrainPlugin,
            BuildingStatePlugin,
            PropsStatesPlugin,
            // WaterStatePlugin,
            MenuPlugin,
            ActionsPlugin,
            InternalAudioPlugin,
            CharacterFpsPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
