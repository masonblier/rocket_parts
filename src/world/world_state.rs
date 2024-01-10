use crate::world::{
    LightsStatePlugin,SoundsStatePlugin,
    WorldInteraction};
use bevy::prelude::*;
use std::collections::HashMap;

// todo break apart into modules, no need for unified world state
#[derive(Default, Resource)]
pub struct WorldState {
    pub active_world: String,
    pub interactable_states: HashMap<Entity, InteractableState>,
    pub animatables: HashMap<String, AnimatableState>,
    pub animatable_lights: HashMap<String, Entity>,
    pub animatable_sounds: HashMap<String, WorldSoundState>,
}

#[derive(Clone, Debug, Default)]
pub struct InteractableState {
    pub interaction: WorldInteraction,
}

#[derive(Debug, Default)]
pub struct AnimatableState {
    pub scene_entity: Option<Entity>,
    pub clips: Vec<Handle<AnimationClip>>,
}

#[derive(Clone, Debug, Default)]
pub struct WorldSoundState {
    pub sound: String,
    pub position: Vec3,
    pub panning: f32,
    pub volume: f32,
    pub paused: bool,
}

pub struct WorldStatePlugin;

impl Plugin for WorldStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((
            LightsStatePlugin,
            SoundsStatePlugin,
        ));
    }
}
