use crate::game_state::GameState;
use crate::inputs::{MouseCamera,MouseLookState};
use crate::world::{WorldState,WorldSoundState};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct SoundsStatePlugin;

pub enum SoundsEventAction {
    #[allow(dead_code)]
    Pause,
    #[allow(dead_code)]
    Resume,
    #[allow(dead_code)]
    Toggle,
}

#[derive(Event)]
pub struct SoundsEvent {
    pub action: SoundsEventAction,
    pub name: String,
}

impl Plugin for SoundsStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<SoundsEvent>();
        
        app.add_systems(OnEnter(GameState::Running), setup_sounds_interaction);
        app.add_systems(Update, update_sounds_interaction.run_if(in_state(GameState::Running)));
        app.add_systems(Update, update_sounds_states.run_if(in_state(GameState::Running)));
        app.add_systems(OnExit(GameState::Running), exit_sounds_interaction);
    }
}

fn setup_sounds_interaction(
    asset_server: Res<AssetServer>,
    mut audio: ResMut<DynamicAudioChannels>,
    mut world_state: ResMut<WorldState>,
) {
    for (sound_name, sounds_state) in world_state.animatable_sounds.iter() {
        if audio.is_channel(sound_name) {
            let channel = audio.channel(sound_name);
            if sounds_state.paused {
                channel.pause();
            } else {
                channel.resume();
            }
        } else {
            let channel = audio
                .create_channel(sound_name);
            channel
                .play(asset_server.load(&format!("audio/{}.ogg", sounds_state.sound)))
                .looped();
            if sounds_state.paused {
                channel.pause();
            }
        }
    }

    // footsteps

    if audio.is_channel("footsteps") {
        let channel = audio.channel("footsteps");
        if let Some(sound_state) = world_state.animatable_sounds.get("footsteps") {
            if sound_state.paused {
                channel.pause();
            } else {
                channel.resume();
            }
        } else {
            channel.pause();
            world_state.animatable_sounds.insert("footsteps".into(), WorldSoundState {
                sound: "steps_snow_dry".into(),
                position: Vec3::ZERO,
                paused: true,
                volume: 0.2,
                panning: 0.5,
            });
        }
    } else {
        let channel = audio
            .create_channel("footsteps");
        channel
            .play(asset_server.load(&format!("audio/steps_snow_dry.ogg")))
            .with_volume(0.2)
            .looped();
        channel.pause();
        world_state.animatable_sounds.insert("footsteps".into(), WorldSoundState {
            sound: "steps_snow_dry".into(),
            position: Vec3::ZERO,
            paused: true,
            volume: 0.2,
            panning: 0.5,
        });
    }
    // train
    if audio.is_channel("train") {
        let channel = audio.channel("train");
        if let Some(sound_state) = world_state.animatable_sounds.get("train") {
            if sound_state.paused {
                channel.pause();
            } else {
                channel.resume();
            }
        } else {
            channel.pause();
            world_state.animatable_sounds.insert("train".into(), WorldSoundState {
                sound: "train_rolling".into(),
                position: Vec3::ZERO,
                paused: true,
                volume: 0.5,
                panning: 0.5,
            });
        }
    } else {
        let channel = audio
            .create_channel("train");
        channel
            .play(asset_server.load(&format!("audio/train_rolling.ogg")))
            .with_volume(1.0)
            .looped();
        channel.pause();
        world_state.animatable_sounds.insert("train".into(), WorldSoundState {
            sound: "train_rolling".into(),
            position: Vec3::ZERO,
            paused: true,
            volume: 0.5,
            panning: 0.5,
        });
    }
}

fn update_sounds_interaction(
    audio: Res<DynamicAudioChannels>,
    mut world_state: ResMut<WorldState>,
    mut sounds_events: EventReader<SoundsEvent>,
) {
    for sounds_event in sounds_events.read() {
        if let Some(sounds_state) = world_state.animatable_sounds.get_mut(&sounds_event.name) {
            match sounds_event.action {
                SoundsEventAction::Toggle => {
                    if sounds_state.paused {
                        audio.channel(&sounds_event.name).resume();
                        sounds_state.paused = false;
                    } else {
                        audio.channel(&sounds_event.name).pause();
                        sounds_state.paused = true;
                    }
                }
                SoundsEventAction::Pause => {
                    audio.channel(&sounds_event.name).pause();
                    sounds_state.paused = true;
                }
                SoundsEventAction::Resume => {
                    audio.channel(&sounds_event.name).resume();
                    sounds_state.paused = false;
                }
            }
        }
    }
}

fn update_sounds_states(
    audio: Res<DynamicAudioChannels>,
    mut world_state: ResMut<WorldState>,
    query: Query<&GlobalTransform, With<MouseCamera>>,
    mouse_look: Res<MouseLookState>,
) {
    let camera_transform = query.single();

    // for each playing audio, update state and panning from (camera_pos - audio_pos)
    for (sound_name, sounds_state) in world_state.animatable_sounds.iter_mut() {
        if !sounds_state.paused {
            if sound_name == "train" || sound_name == "footsteps" {
                continue;
            }

            let diff_v = camera_transform.translation() - sounds_state.position;
            let panning = (mouse_look.right.dot(diff_v) * 0.5)
                .max(-1.0).min(1.0) * -0.5 + 0.5;
            let volume = 1.0 - 0.1 * diff_v.length().max(0.0).min(10.0);
            if (sounds_state.panning - panning).abs() > f32::EPSILON {
                audio.channel(sound_name).set_panning(panning as f64);
                sounds_state.panning = panning;
            }
            if (sounds_state.volume - volume).abs() > f32::EPSILON {
                audio.channel(sound_name).set_volume(volume as f64);
                sounds_state.volume = volume;
            }
        }
    }
}

fn exit_sounds_interaction(
    audio: Res<DynamicAudioChannels>,
    world_state: Res<WorldState>,
) {
    for (sound_name, _sounds_state) in world_state.animatable_sounds.iter() {
        audio.channel(sound_name).pause();
    }
}
