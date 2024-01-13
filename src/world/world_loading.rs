use crate::loading::{LoadingUiEvent,LoadingUiEventAction,WorldProps};
use crate::game_state::GameState;
use crate::world::WorldState;
use bevy::{prelude::*, gltf::Gltf};
use bevy::scene::InstanceId;
use std::collections::HashMap;
use bevy_rapier3d::prelude::*;


pub struct WorldLoadingPlugin;

#[derive(Default, Resource)]
pub struct WorldLoadingState {
    animatable_scenes: HashMap<String, InstanceId>,
    inited: bool,
    done: bool,
    build_kit_preload_ent: Option<Entity>,
}


// component to tag unloadable world items
#[derive(Component,Default)]
pub struct WorldEntity;

impl Plugin for WorldLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WorldLoadingState>()
            .init_resource::<WorldState>();

        app.add_systems(OnEnter(GameState::WorldInit), setup_world_init);
        app.add_systems(Update, update_world_init.run_if(in_state(GameState::WorldInit)));
        app.add_systems(OnEnter(GameState::WorldLoading), setup_world_loading);
        app.add_systems(Update, update_world_loading.run_if(in_state(GameState::WorldLoading)));
    }
}

fn setup_world_init(
    mut commands: Commands,
    world_props: Res<WorldProps>,
    mut world_loading: ResMut<WorldLoadingState>,
    mut loading_ui_events: EventWriter<LoadingUiEvent>,
    world_ents: Query<(Entity,With<WorldEntity>)>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    world_loading.inited = false;

    // update loading ui text
    loading_ui_events.send(LoadingUiEvent {
        action: LoadingUiEventAction::SetText,
        payload: Some("Spawning".into()),
    });

    // clear any previous entities
    for (ent, _) in world_ents.iter() {
        commands.entity(ent).despawn_recursive();
    }

    // preload build-kit gltf
    world_loading.build_kit_preload_ent = Some(
        commands.spawn(SceneBundle {
            scene: assets_gltf.get(&world_props.building_kit).unwrap().scenes[0].clone(),
            transform: Transform::from_translation(1000. * Vec3::Y),
            ..Default::default()
        }).id());
}

fn update_world_init(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut world_loading: ResMut<WorldLoadingState>,
    world_state: Res<WorldState>,
) {
    if let Some(ent) = world_loading.build_kit_preload_ent {
        if commands.get_entity(ent).is_none() {
            return;
        }
    } else {
        return;
    }

    if !world_loading.inited {
        world_loading.inited = true;
        if world_state.active_world == "credits" {
            state.set(GameState::Credits);
        } else {
            state.set(GameState::WorldLoading);
        }

        if let Some(ent) = world_loading.build_kit_preload_ent {
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn setup_world_loading(
    mut world_loading: ResMut<WorldLoadingState>,
    mut world_state: ResMut<WorldState>,
) {
    world_loading.done = false;

    // reset state
    world_loading.animatable_scenes = HashMap::new();
    world_state.interactable_states = HashMap::new();
    world_state.animatables = HashMap::new();
    world_state.animatable_lights = HashMap::new();
    world_state.animatable_sounds = HashMap::new();

    // let world_asset = if world_state.active_world == "world03" {
    //     world_assets.get(&world_handles.world03).unwrap()
    // } else {
        // let world_asset = world_assets.get(&world_handles.world01).unwrap();
    // };

}

fn update_world_loading(
    mut world_loading: ResMut<WorldLoadingState>,
    world_state: Res<WorldState>,
    mut state: ResMut<NextState<GameState>>,
    mut loading_ui_events: EventWriter<LoadingUiEvent>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    if world_loading.done {
        return;
    }

    // get animation keys not yet loaded
    let waiting_keys = world_loading.animatable_scenes.keys().into_iter().filter(|&anim_key| {
        !world_state.animatables.contains_key(anim_key)
    }).collect::<Vec<&String>>();

    // if no waiting keys, all done
    if waiting_keys.len() == 0 {
        info!("World loaded: {:?}", 1);

        // hide loading ui
        loading_ui_events.send(LoadingUiEvent {
            action: LoadingUiEventAction::Hide,
            payload: None,
        });

        world_loading.done = true;
        state.set(GameState::Running);

        // resume physics
        rapier_config.physics_pipeline_active = true;
    } else {
        // check for waiting loaded scenes
        // for waiting_key in waiting_keys {
        //     let mut lowest_ent: Option<Entity> = None;
        //     if let Some(inst_iter) = scene_spawner.iter_instance_entities(*world_loading.animatable_scenes.get(waiting_key).unwrap()) {
        //         for inst in inst_iter {
        //             if !lowest_ent.is_some() || inst.id() < lowest_ent.unwrap() {
        //                 lowest_ent = Some(inst);
        //             }
        //         }
        //     }
        //     if lowest_ent.is_some() {
        //         let clips = if waiting_key.starts_with("switch") {
        //             vec![
        //                 asset_server.load("props/big_switch.glb#Animation3"),
        //             ]
        //         } else { Vec::new() };
        //         world_state.animatables.insert(waiting_key.to_string(), AnimatableState { scene_entity: lowest_ent.clone(), clips });
        //     }
        // }
    }
}
