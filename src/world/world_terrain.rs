use crate::inputs::MouseCamera;
use crate::loading::TextureAssets;
use crate::character::CharacterFpsMotionConfig; 
use crate::game_state::GameState;

use bevy::{
    prelude::*,
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    render::mesh::{Indices,PrimitiveTopology},
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_rapier3d::prelude::*;
use isosurface::{
    distance::Signed, extractor::IndexedInterleavedNormals, sampler::Sampler,
    MarchingCubes,
};

use super::{CHUNK_LENGTH,IsosurfaceSource};

const CHUNK_SEGS: usize = 64;

pub struct WorldTerrainPlugin;

// system state
#[derive(Default, Resource)]
pub struct WorldTerrainState {
    pub last_loaded_pos: [i32;2],
    terrain_material: Handle<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>,
}

/// This plugin handles loading of nearest terrain chunks
/// and colliders
impl Plugin for WorldTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldTerrainState::default());
        app.add_plugins((MaterialPlugin::<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>::default(),));
        app.add_systems(OnEnter(GameState::WorldLoading), setup_active_terrain);
        app.add_systems(Update, update_active_terrain.run_if(in_state(GameState::Running)));
    }
}


// #[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
// struct ArrayTextureMaterial {
//     #[texture(0, dimension = "2d_array")]
//     #[sampler(1)]
//     array_texture: Handle<Image>,
// }

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct ArrayTextureMaterial {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    quantize_steps: u32,
    #[texture(101, dimension = "2d_array")]
    #[sampler(102)]
    array_texture: Handle<Image>,
}


impl MaterialExtension for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
}

fn setup_active_terrain(
    mut commands: Commands, 
    mut terrain_state: ResMut<WorldTerrainState>,
    mut images: ResMut<Assets<Image>>,
    texture_assets: Res<TextureAssets>,
    meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>>,
) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(10.0, 100.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Ambient light
        // commands.insert_resource(AmbientLight {
        //     color: Color::rgb(0.98, 0.95, 0.82),
        //     brightness: 1.0,
        // });

    // Point light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 50.0, 0.0),
        point_light: PointLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            intensity: 1000.0,
            range: 400.0,
            ..default()
        },
        ..default()
    });

        
    let image = images.get_mut(&texture_assets.texture_array).unwrap();

    // Create a new array texture asset from the loaded texture.
    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);

    // terrain_state.terrain_material = materials.add(Color::rgb(0.8, 0.99, 0.9).into());
    terrain_state.terrain_material = 
        materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::RED,
                // can be used in forward or deferred mode.
                opaque_render_method: OpaqueRendererMethod::Auto,
                // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                // in forward mode, the output can also be modified after lighting is applied.
                // see the fragment shader `extended_material.wgsl` for more info.
                // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                ..Default::default()
            },
            extension: ArrayTextureMaterial {
                quantize_steps: 3,
                array_texture: texture_assets.texture_array.clone(),
            },
        });
    terrain_state.last_loaded_pos = [0, 0];

    spawn_chunks_around_lastpos(commands, terrain_state, meshes);
}


fn spawn_chunks_around_lastpos(
    mut commands: Commands, 
    terrain_state: ResMut<WorldTerrainState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // current chunk
    let (pbrmesh, collider) = build_terrain_chunk_iso(
        terrain_state.last_loaded_pos[0], 
        terrain_state.last_loaded_pos[1]);
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(pbrmesh),
            material: terrain_state.terrain_material.clone(),
            transform: Transform::from_xyz(-CHUNK_LENGTH / 2.0, -100.0, -CHUNK_LENGTH / 2.0,),
            ..default()
        },
        collider,
        CollisionGroups::new(Group::GROUP_1 | Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2),
    ));

    // south?
    let (pbrmesh, collider) = build_terrain_chunk_iso(
        terrain_state.last_loaded_pos[0], 
        terrain_state.last_loaded_pos[1] + 1);
        
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(pbrmesh),
            material: terrain_state.terrain_material.clone(),
            transform: Transform::from_xyz(-CHUNK_LENGTH / 2.0, -100.0, 
                1. * CHUNK_LENGTH + -CHUNK_LENGTH / 2.0,),
            ..default()
        },
        collider,
        CollisionGroups::new(Group::GROUP_1 | Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2),
    ));
}

fn update_active_terrain(
    // movement_state: Res<MovementState>,
    // mut terrain_state: ResMut<WorldTerrainState>,
    // mut commands: Commands, 
    // mut meshes: ResMut<Assets<Mesh>>,
    mover_query: Query<(&Transform, &CharacterFpsMotionConfig), Without<MouseCamera>>,
) {
    let (mover_transform, _mover) = mover_query.single();
   
   let relative_pos = [mover_transform.translation.x, mover_transform.translation.z];
   if relative_pos[0] > (relative_pos[0] * CHUNK_LENGTH + CHUNK_LENGTH / 2.0) {
        // println!("moved chunk {:?}", relative_pos)
   }    
}

fn build_terrain_chunk_iso(
    chunkx: i32,
    chunkz: i32,
) -> (Mesh,Collider) {

    let iso = IsosurfaceSource::new(chunkx, chunkz);
    let sampler = Sampler::new(&iso);

    let mut mixed_vns = vec![];
    let mut mixed_indcs = vec![];
    let mut extractor = IndexedInterleavedNormals::new(&mut mixed_vns, &mut mixed_indcs, &sampler);
    // let mut marcher = PointCloud::<Signed>::new(subdivisions);
    let mut marcher = MarchingCubes::<Signed>::new(CHUNK_SEGS);
    marcher.extract(&sampler, &mut extractor);
    
    let scale = CHUNK_LENGTH;

    let mut vertices = vec![];
    let mut normals: Vec<[f32;3]> = vec![];
    let mut vertices_bevy = vec![];
    for n in 0..(mixed_vns.len()/6) {
        vertices.push(mixed_vns[n*6+0]*scale);
        vertices.push(mixed_vns[n*6+1]*scale);
        vertices.push(mixed_vns[n*6+2]*scale);
        normals.push([mixed_vns[n*6+3],
            -mixed_vns[n*6+4],
            mixed_vns[n*6+5]]);
        vertices_bevy.push(Vec3::new(
            mixed_vns[n*6+0]*scale,
            mixed_vns[n*6+1]*scale,
            mixed_vns[n*6+2]*scale,));
    }


    // let mut mixed_indcs: Vec<u32> = vec![];
    // for n in 0..(vertices_bevy.len()/2) {
    //     let nu = n as u32
    //     mixed_indcs.push(nu*2+0);
    //     mixed_indcs.push(nu*2+1);
    //     mixed_indcs.push(nu*2+2);
    // }
    
    let mut indices_bevy: Vec<[u32;3]> = vec![];
    for n in 0..(mixed_indcs.len()/3) {
        indices_bevy.push([mixed_indcs[n*3+0],mixed_indcs[n*3+1],mixed_indcs[n*3+2]]);
    }

    println!("vns {:?}", mixed_vns.len());
    println!("indcs {:?}", mixed_indcs.len());


    (
        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vertices_bevy.clone(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_UV_0,
                vertices_bevy.clone().into_iter().map(|v| [v.x%1.,v.z%1.]).collect::<Vec<[f32;2]>>(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                normals.clone(),
            )
            .with_indices(Some(Indices::U32(mixed_indcs.clone()))),
        Collider::round_convex_decomposition(&vertices_bevy, &indices_bevy.clone(), 0.1),
        // Collider::cuboid(10.0, 10.0, 10.0),
    )
}

// fn build_terrain_chunk_plane(
//     chunkx: i32,
//     chunkz: i32,
// ) -> (Mesh,Collider) {
//     let segx = (chunkx as f32 * CHUNK_LENGTH) as f32;
//     let segz = (chunkz as f32 * CHUNK_LENGTH) as f32;
//     let mut vertices: Vec<Vec3> = Vec::new();
//     let mut indices: Vec<[u32; 3]> = Vec::new();

//     for ix in 0..=CHUNK_SEGS {
//         for iz in 0..=CHUNK_SEGS {
//             let seglen = CHUNK_LENGTH / (CHUNK_SEGS as f32);
//             let y = heightfn(ix as f32 * seglen + segx, iz as f32 * seglen + segz);
//             vertices.push(Vec3::new(ix as f32 * seglen, y, iz as f32 * seglen));
//         }
//     }
//     for ix in 0..CHUNK_SEGS {
//         for iz in 0..CHUNK_SEGS {
//             // Start of the two relevant rows of vertices.
//             let row0 = ix * (CHUNK_SEGS + 1);
//             let row1 = (ix + 1) * (CHUNK_SEGS + 1);
//             // Two triangles making up a not-very-flat quad for each segment of the bowl.
//             indices.push([(row0 + iz) as u32, (row0 + iz + 1) as u32, (row1 + iz) as u32]);
//             indices.push([(row1 + iz) as u32, (row0 + iz + 1) as u32, (row1 + iz + 1) as u32]);
//         }
//     }

//     (
//         Mesh::new(PrimitiveTopology::TriangleList)
//             .with_inserted_attribute(
//                 Mesh::ATTRIBUTE_POSITION,
//                 vertices.clone(),
//             )
//             .with_inserted_attribute(
//                 Mesh::ATTRIBUTE_UV_0,
//                 vertices.clone().into_iter().map(|v| [v.x/2., v.z/2.]).collect::<Vec<[f32;2]>>(),
//             )
//             .with_inserted_attribute(
//                 Mesh::ATTRIBUTE_NORMAL,
//                 vertices.clone().into_iter().map({|v|
//                     // estimate normal from nearby slope
//                     Vec3::new(heightfn(v.x + segx - 0.1, v.z + segz) - 
//                         heightfn(v.x + segx + 0.1, v.z + segz),1.0,
//                      heightfn(v.x + segx, v.z + segz - 0.1) - 
//                         heightfn(v.x + segx, v.z + segz + 0.1)).normalize().into()
//                 }).collect::<Vec<[f32;3]>>(),
//             )
//             .with_indices(Some(Indices::U32(indices.clone().into_iter().flatten().collect()))),
//         Collider::trimesh(vertices, indices)
//     )
// }
