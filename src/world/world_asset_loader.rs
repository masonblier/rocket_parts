use bevy::{
    asset::{AssetLoader, io::Reader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::utils::thiserror;
use thiserror::Error;
// use bevy_rapier3d::prelude::*;
use serde::Deserialize;
use ron;

#[derive(Asset, TypePath, Deserialize)]
pub struct WorldAsset {
    pub colliders: Vec<WorldCollider>,
    pub interactables: Vec<WorldInteractable>,
    pub lights: Vec<WorldLight>,
    pub props: Vec<WorldProp>,
    pub sounds: Vec<WorldSound>,
}

// represents data for convex colliders defined for a world
#[derive(Debug, Deserialize)]
pub struct WorldCollider {
    pub shape: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// represents interactable target collider
#[derive(Debug, Deserialize)]
pub struct WorldInteractable {
    pub shape: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub interaction: Option<WorldInteraction>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WorldInteraction {
    pub interaction: String,
    pub interaction_text: String,
    pub actions: Vec<(String, String)>,
    pub blockers: Vec<(String, String)>,
}

// represents gltf prop
#[derive(Debug, Deserialize)]
pub struct WorldProp {
    pub prop: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub animatable: Option<String>,
}

// represents 3d positioned sound
#[derive(Debug, Deserialize)]
pub struct WorldSound {
    pub sound: String,
    pub translation: Vec3,
    pub paused: bool,
    pub animatable: Option<String>,
}

// represents light
#[derive(Debug, Deserialize)]
pub struct WorldLight {
    pub light_type: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub watts: f32,
    pub animatable: Option<String>,
}


/// Possible errors that can be produced by [`WorldAssetLoader  `]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum WorldAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

#[derive(Default)]
pub struct WorldAssetLoader;

impl AssetLoader for WorldAssetLoader {
    type Asset = WorldAsset;
    type Settings = ();
    type Error = WorldAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let world_asset = ron::de::from_bytes::<WorldAsset>(&bytes)?;
            // load_context.set_default_asset(LoadedAsset::new(world_asset));
            Ok(world_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["world"]
    }
}

pub struct WorldAssetLoaderPlugin;

impl Plugin for WorldAssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<WorldAsset>()
            .init_asset_loader::<WorldAssetLoader>();
    }
}
