use crate::actions::BuildingActionsState;
use crate::building::ThrustersStatusText;
use crate::game_state::GameState;
use crate::loading::TextureAssets;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use std::f32::consts::PI;

const THRUSTERS_CUTOFF: f32 = 12.;

// system state
#[derive(Default, Resource)]
pub struct ThrustersState {
    pub thrusters_power: f32,
    pub thrusters_animating: bool,
}


#[derive(Component,Default)]
pub struct ThrusterInteractable {
    pub grid: Option<Entity>,
}
#[derive(Component,Default)]
pub struct ThrusterSprite {
}


#[derive(Default)]
pub struct ThrustersStatePlugin;

impl Plugin for ThrustersStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ThrustersState::default())
            // .add_systems(OnEnter(GameState::Running), setup_thrusters_state)
            .add_systems(Update, (
                update_thursters_state.run_if(in_state(GameState::Running)),));
    }
}


#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn create_thruster_sprite(
    commands: &mut Commands,
    texture_handles: &Res<TextureAssets>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
) {
    let quad_width = 2.0;
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        quad_width,
        quad_width,
    ))));
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handles.explosion_static.clone()),
        alpha_mode: AlphaMode::Blend,
        // double_sided: true,
        unlit: true,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0)
            .with_rotation(Quat::from_rotation_y(PI / 2.)),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -0.8, 0.0)
            .with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0)
            .with_rotation(Quat::from_rotation_y(PI)),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0)
            .with_rotation(Quat::from_rotation_y(3. * PI / 2.)),
        ..default()
    })
    .insert(ThrusterSprite { })
    // .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .set_parent(parent);
}

fn update_thursters_state(
    mut commands: Commands,
    // time: Res<Time>,
    building_actions: Res<BuildingActionsState>,
    mut thrusters_state: ResMut<ThrustersState>,
    thursters_query: Query<(Entity, &ThrusterInteractable, &GlobalTransform)>,
    thurster_sprites_query: Query<(Entity, &ThrusterSprite)>,
    texture_handles: Res<TextureAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ext_forces: Query<&mut ExternalForce>,
    mut toolbar_text: Query<(&ThrustersStatusText, &mut Text)>,
) {
    if building_actions.thrusters_power < THRUSTERS_CUTOFF {
        if thrusters_state.thrusters_animating {
            // cleanup sprites
            thurster_sprites_query.for_each(|ts| { commands.entity(ts.0).despawn_recursive(); });
            // remove external force from parent grid
            thursters_query.for_each(|t| { 
                let mut ext_force = ext_forces.get_mut(t.1.grid.unwrap()).unwrap();
                ext_force.force = Vec3::ZERO;
            });
            for (_, mut text) in toolbar_text.iter_mut() {
                text.sections[0].value = " ".into();
            }
            thrusters_state.thrusters_animating = false;
        }
        return;
    }

    if (building_actions.thrusters_power - thrusters_state.thrusters_power).abs() > 0.001 {
        thrusters_state.thrusters_power = building_actions.thrusters_power;
        thursters_query.for_each(|(_entity, ti, thruster_gt )| {
            // add external force onto parent grid
            let mut ext_force = ext_forces.get_mut(ti.grid.unwrap()).unwrap();
            ext_force.force = thruster_gt.up() * thrusters_state.thrusters_power;
        });
        for (_, mut text) in toolbar_text.iter_mut() {
            text.sections[0].value = format!("{:.0}%", thrusters_state.thrusters_power).into();
        }
    }
    
    if !thrusters_state.thrusters_animating {
        thursters_query.for_each(|(entity, _ti, _thruster_gt )| {
            // setup combustion sprites
            create_thruster_sprite(&mut commands, &texture_handles, &mut meshes, &mut materials, entity);
        });
        thrusters_state.thrusters_animating = true;
    }

    // TODO animate sprites...
}
