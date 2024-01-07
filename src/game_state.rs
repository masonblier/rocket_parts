use bevy::prelude::*;
use bevy::ecs::schedule::States;
// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect, States)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    PreLoading,
    AssetLoading,
    SceneLoading,
    WorldInit,
    World01Loading,
    Credits,
    // Here the menu is drawn and waiting for player interaction
    Menu,
    // During this State the actual game logic is executed
    Running,
    // Game paused, can resume
    Paused,
    // Character loading and init
    CharacterLoading,
    // World loading states, level specific assets
    WorldLoading,
}
