use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaAnimatingStateDirective, TnuaGhostPlatform, TnuaGhostSensor,
    TnuaProximitySensor, TnuaToggle,
};
use bevy_tnua::control_helpers::{
    TnuaCrouchEnforcer, TnuaCrouchEnforcerPlugin, TnuaSimpleAirActionsCounter,
    TnuaSimpleFallThroughPlatformsHelper,
};
use bevy_tnua_rapier3d::*;

pub struct DemoPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles level
/// World logic is only active during the State `GameState::Playing`
impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(PhysicsPlugins::default());

        // app.add_systems(OnEnter(GameState::WorldLoading), setup_level)
            // .add_systems(Update, move_player.run_if(in_state(GameState::Playing)))
            ;
        // app.add_systems(Update, update_rapier_physics_active);
    }
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            // For some reason in Bevy 0.12 shadows no longer work in WASM
            shadows_enabled: !cfg!(target_arch = "wasm32"),
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });

    // let mut cmd = commands.spawn_empty();
    // cmd.insert(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Plane {
    //         size: 128.0,
    //         subdivisions: 0,
    //     })),
    //     material: materials.add(Color::WHITE.into()),
    //     ..Default::default()
    // });
    // cmd.insert(RigidBody::Static);
    // cmd.insert(Collider::halfspace(Vec3::Y));

    let obstacles_material = materials.add(Color::GRAY.into());
    for ([width, height, depth], transform) in [
        (
            [20.0, 0.1, 2.0],
            Transform::from_xyz(10.0, 10.0, 0.0).with_rotation(Quat::from_rotation_z(0.6)),
        ),
        ([4.0, 2.0, 2.0], Transform::from_xyz(-4.0, 1.0, 0.0)),
        ([6.0, 1.0, 2.0], Transform::from_xyz(-10.0, 4.0, 0.0)),
        ([6.0, 1.0, 2.0], Transform::from_xyz(0.0, 2.6, -5.0)),
    ] {
        let mut cmd = commands.spawn_empty();
        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(width, height, depth))),
            material: obstacles_material.clone(),
            transform,
            ..Default::default()
        });
        cmd.insert(RigidBody::Static);
        cmd.insert(Collider::cuboid(width, height, depth));
    }

    // Fall-through platforms
    let fall_through_obstacles_material = materials.add(Color::PINK.with_a(0.8).into());
    for y in [2.0, 4.5] {
        let mut cmd = commands.spawn_empty();
        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(6.0, 0.5, 2.0))),
            material: fall_through_obstacles_material.clone(),
            transform: Transform::from_xyz(6.0, y, 10.0),
            ..Default::default()
        });
        cmd.insert(RigidBody::Static);
        cmd.insert(Collider::cuboid(6.0, 0.5, 2.0));
        cmd.insert(CollisionLayers::new(
            [LayerNames::FallThrough],
            [LayerNames::FallThrough],
        ));
        cmd.insert(TnuaGhostPlatform);
    }

    commands.spawn((
        SceneBundle {
            scene: asset_server.load("collision-groups-text.glb#Scene0"),
            transform: Transform::from_xyz(10.0, 2.0, 1.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(4.0, 2.0, 4.0),
        CollisionLayers::new([LayerNames::PhaseThrough], [LayerNames::PhaseThrough]),
    ));

    commands.spawn((
        SceneBundle {
            scene: asset_server.load("sensor-text.glb#Scene0"),
            transform: Transform::from_xyz(15.0, 2.0, 1.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(4.0, 2.0, 4.0),
        Sensor,
    ));

    // spawn moving platform
    {
        let mut cmd = commands.spawn_empty();

        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(4.0, 1.0, 4.0))),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(-4.0, 6.0, 0.0),
            ..Default::default()
        });
        cmd.insert(Collider::cuboid(4.0, 1.0, 4.0));
        cmd.insert(RigidBody::Kinematic);
        // TODO
        // cmd.insert(MovingPlatform::new(
        //     4.0,
        //     &[
        //         Vec3::new(-4.0, 6.0, 0.0),
        //         Vec3::new(-8.0, 6.0, 0.0),
        //         Vec3::new(-8.0, 10.0, 0.0),
        //         Vec3::new(-8.0, 10.0, -4.0),
        //         Vec3::new(-4.0, 10.0, -4.0),
        //         Vec3::new(-4.0, 10.0, 0.0),
        //     ],
        // ));
    }

    // spawn spinning platform
    {
        let mut cmd = commands.spawn_empty();

        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 3.0,
                height: 1.0,
                resolution: 10,
                segments: 10,
            })),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(-2.0, 2.0, 10.0),
            ..Default::default()
        });
        cmd.insert(Collider::cylinder(1.0, 3.0));
        cmd.insert(AngularVelocity(Vec3::Y));
        cmd.insert(RigidBody::Kinematic);
    }
}


fn update_rapier_physics_active(
    mut physics_time: ResMut<Time<Physics>>,
    // setting_from_ui: Res<ExampleUiPhysicsBackendActive>,
) {
    // if setting_from_ui.0 {
        physics_time.unpause();
    // } else {
    //     physics_time.pause();
    // }
}
