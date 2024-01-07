use crate::inputs::{CursorLockState};
use crate::game_state::GameState;
use crate::world::WorldState;
use bevy::prelude::*;

pub struct AnimatableStatePlugin;

pub enum AnimatableEventAction {
    PlayOnce,
    Despawn,
}

#[derive(Event)]
pub struct AnimatableEvent {
    pub action: AnimatableEventAction,
    pub name: String,
    pub animation: String,
}

impl Plugin for AnimatableStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<AnimatableEvent>()
        .add_systems(Update, 
            update_animatable_interaction.run_if(in_state(GameState::Running)));
    }
}


fn update_animatable_interaction(
    mut commands: Commands,
    cursor_lock_state: Res<CursorLockState>,
    mut world_state: ResMut<WorldState>,
    mut animatable_events: EventReader<AnimatableEvent>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
) {
    if !cursor_lock_state.enabled {
        return;
    }

    for animatable_event in animatable_events.iter() {
        if let Some(animatable_state) = world_state.animatables.get_mut(&animatable_event.name) {
            match animatable_event.action {
                AnimatableEventAction::PlayOnce => {
                    for (parent, mut player) in animation_players.iter_mut() {
                        if animatable_state.scene_entity.is_some() && animatable_state.scene_entity.unwrap() == parent.get() {
                            player.play(animatable_state.clips[0].clone_weak());
                        }
                    }
                }
                AnimatableEventAction::Despawn => {
                    commands.entity(animatable_state.scene_entity.unwrap()).despawn_recursive();
                }
            }
        }
    }
}
