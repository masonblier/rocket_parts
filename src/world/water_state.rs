use crate::game_state::GameState;
use crate::inputs::MouseLookState;
use crate::character::CharacterFpsMotionConfig;

use std::f32::consts::PI;
use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, AsBindGroup, ShaderRef
        },
        // view::RenderLayers,
    }, core_pipeline::clear_color::ClearColorConfig,
};

pub struct WaterStatePlugin;
impl Plugin for WaterStatePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((MaterialPlugin::<WaterMaterial>::default(),))
        .add_systems(OnEnter(GameState::WorldLoading), setup_water_plane)
        .add_systems(Update, 
            update_water_camera_sync.run_if(in_state(GameState::Running)))
            ;
    }
}


#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct WaterMaterial {}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component)]
pub struct WaterCamera;

const WATER_HEIGHT: f32 = -4.;

fn setup_water_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut wmaterials: ResMut<Assets<WaterMaterial>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
    // let cube_material_handle = materials.add(StandardMaterial {
    //     base_color: Color::rgb(0.8, 0.7, 0.6),
    //     reflectance: 0.02,
    //     unlit: false,
    //     ..default()
    // });

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    // let first_pass_layer = RenderLayers::layer(1);

    // camera for first pass
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Default,
                ..default()
            },
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .looking_at(Vec3::Y, Vec3::Z),
            ..default()
        },
        // first_pass_layer,
        WaterCamera
    ));


    let _material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });


    
    // Reflective water plane, with material containing the rendered first pass texture.
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 20.0, subdivisions: 10 })),
            // material: material_handle,
            material: wmaterials.add(WaterMaterial {}),
            transform: Transform::from_xyz(-8.0, WATER_HEIGHT, 0.0)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, -PI/2.)),
            ..default()
        },
    ));
}

fn update_water_camera_sync(
    mouse_look: Res<MouseLookState>,
    mover_query: Query<(&Transform, &CharacterFpsMotionConfig), Without<WaterCamera>>,
    mut query: Query<&mut Transform, With<WaterCamera>>,
) {
    let (mover_transform, _mover) = mover_query.single();
    for mut camera in query.iter_mut() {
        let mover_position = mover_transform.translation.clone() + 0.8 * Vec3::Y + 0.15 * mouse_look.forward;
        if mover_position.y >= WATER_HEIGHT && mouse_look.forward.y < 0. {
            let mouse_reflect_y = Vec3::new(mouse_look.forward.x, -mouse_look.forward.y, mouse_look.forward.z);
            let yd = mover_position.y - WATER_HEIGHT;
            let reflect_point = Vec3::new(
                -yd * mover_position.x / mover_position.y + mover_position.x, 
                WATER_HEIGHT, 
                -yd * mover_position.z / mover_position.y + mover_position.z);
            camera.translation = reflect_point;
            camera.look_at(reflect_point + mouse_reflect_y, Vec3::Y);
        }

    }
}
