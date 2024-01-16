use crate::inputs::MouseCamera;
use crate::character::CharacterFpsMotionConfig; 
use crate::game_state::GameState;
use crate::world::{CHUNK_LENGTH,IsosurfaceSource,TerrainMaterialPlugin,TerrainMaterialState};

use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
    render::mesh::{Indices,PrimitiveTopology},
};
use futures_lite::future;
use bevy_rapier3d::prelude::*;
use isosurface::{
    distance::Signed, extractor::IndexedInterleavedNormals, sampler::Sampler,
    MarchingCubes,
};
use std::collections::{HashSet,HashMap};

const CHUNK_SEGS: usize = 64;

pub struct WorldTerrainPlugin;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct MeshCacheKey([i32;2],bool);

// system state
#[derive(Default, Resource)]
pub struct WorldTerrainState {
    pub last_chunk_pos: Option<[i32;2]>,
    pub ent_cache: HashMap<MeshCacheKey,Entity>,
    pub mesh_cache: HashMap<MeshCacheKey,(Mesh,Collider)>,
}

// chunk entity component
#[derive(Component)]
pub struct TerrainChunk {
    mesh_cache_key: MeshCacheKey,
}

// fadeout despawning chunk
#[derive(Component)]
pub struct TerrainFadeout {
    timer: Timer,
}

/// This plugin handles loading of nearest terrain chunks
/// and colliders
impl Plugin for WorldTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldTerrainState::default());
        app.add_plugins((TerrainMaterialPlugin::default(),));
        app.add_systems(OnEnter(GameState::WorldLoading), setup_active_terrain);
        app.add_systems(Update, update_active_terrain.run_if(in_state(GameState::Running)));
        app.add_systems(Update, handle_terrian_loaded_tasks.run_if(in_state(GameState::Running)));
        app.add_systems(Update, handle_terrain_fadeout.run_if(in_state(GameState::Running)));
    }
}

fn setup_active_terrain(
    mut commands: Commands,
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
}

fn update_active_terrain(
    mut commands: Commands, 
    mut terrain_state: ResMut<WorldTerrainState>,
    chunks_query: Query<(Entity, &TerrainChunk)>,
    mover_query: Query<(&Transform, &CharacterFpsMotionConfig), Without<MouseCamera>>,
) {
    let (mover_transform, _mover) = mover_query.single();
    let relative_pos = [mover_transform.translation.x, mover_transform.translation.z];

    // use chunk shifted by 0.5 as center of 4x4 chunk grid
    let shifted_chunk_pos = [
        ((-0.5 * CHUNK_LENGTH + relative_pos[0]) / CHUNK_LENGTH).round() as i32,
        ((-0.5 * CHUNK_LENGTH + relative_pos[1]) / CHUNK_LENGTH).round() as i32,
    ];

    // check if changed
    if Some(shifted_chunk_pos) == terrain_state.last_chunk_pos {
        return;
    }
    terrain_state.last_chunk_pos = Some(shifted_chunk_pos);

    // spawn meshes, only center 4x4 is non-lod mesh
    let mut keep_ents = HashSet::<Entity>::new();
    for x in -1..3 {
        for z in -1..3 {
            let npos = [shifted_chunk_pos[0] + x, shifted_chunk_pos[1] + z];
            let lod_chunk = !(x >= 0 && x < 2 && z >= 0 && z < 2);
            keep_ents.insert(spawn_chunk(npos, lod_chunk, &mut commands, &mut terrain_state));
        }
    }

    // remove other chunks
    for (ent, chunk_comp) in chunks_query.iter() {
        if !keep_ents.contains(&ent) {
            commands.entity(ent).insert(TerrainFadeout { timer: Timer::from_seconds(1.0, TimerMode::Once) });
            terrain_state.ent_cache.remove(&chunk_comp.mesh_cache_key);
        }
    }
}


#[derive(Component)]
struct ComputeTransform(Task<CommandQueue>);

/// generates mesh data in async task
fn spawn_chunk_deferred(
    ckey: MeshCacheKey,
    commands: &mut Commands, 
) -> Entity {
    let thread_pool = AsyncComputeTaskPool::get();

    let deferred_entity = commands.spawn((
        TerrainChunk { mesh_cache_key: ckey.clone() },
    )).id();

    let task = thread_pool.spawn(async move {
        // compute here
        let (mesh, collider_opt): (Mesh, Option<Collider>) = if ckey.1 {
            (build_terrain_chunk_plane(ckey.0[0], ckey.0[1],), None)
        } else {
            let c = build_terrain_chunk_iso(ckey.0[0], ckey.0[1],);
            (c.0, Some(c.1))
        };

        // we use a raw command queue to pass a FnOne(&mut World) back to be
        // applied in a deferred manner.
        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut World| {
            let terrain_material = {
                world.resource::<TerrainMaterialState>().terrain_material.clone()
            };
            let mesh_handle = {
                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                meshes.add(mesh)
            };

            let mut cmd = world
                .entity_mut(deferred_entity);
            cmd
                // Add our new PbrBundle of components to our tagged entity
                .insert(MaterialMeshBundle {
                    mesh: mesh_handle,
                    material: terrain_material.clone(),
                    transform: Transform::from_xyz(
                        (ckey.0[0] as f32) * CHUNK_LENGTH - CHUNK_LENGTH / 2.0, 
                        -100.0, 
                        (ckey.0[1] as f32) * CHUNK_LENGTH - CHUNK_LENGTH / 2.0,),
                    ..default()
                })
                .insert(CollisionGroups::new(Group::GROUP_1 | Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2));

            if let Some(collider) = collider_opt {
                cmd.insert(collider);
            }

            cmd
                // Task is complete, so remove task component from entity
                .remove::<ComputeTransform>();
        });

        command_queue
    });

    // Spawn new entity and add our new task as a component
    commands.entity(deferred_entity).insert(ComputeTransform(task));

    deferred_entity
}

/// inserts finished mesh gen to gui
fn handle_terrian_loaded_tasks(world: &mut World) {
    let mut transform_tasks: QueryState<&mut ComputeTransform> = world.query::<&mut ComputeTransform>();
    let mut cqos = 
        transform_tasks.iter_mut(world)
        .map(|mut task| { block_on(future::poll_once(&mut task.0)) })
        .collect::<Vec<Option<CommandQueue>>>();
    cqos.iter_mut()
        .for_each(|cqo| { if let Some(commands_queue) = cqo.as_mut() { commands_queue.apply(world); } });
}

/// removes terrain after lod load. todo fadeout transition
fn handle_terrain_fadeout(
    mut commands: Commands, 
    time: Res<Time>,
    mut fadeouts_query: Query<(Entity, &mut TerrainFadeout)>,
) {
    for (fo_ent, mut fo) in fadeouts_query.iter_mut() {
        fo.timer.tick(time.delta());
        if fo.timer.finished() {
            commands.entity(fo_ent).despawn_recursive();
        }
    }
}

fn spawn_chunk(
    chunk_pos: [i32; 2],
    lod_chunk: bool,
    commands: &mut Commands, 
    terrain_state: &mut ResMut<WorldTerrainState>,
) -> Entity {
    // current chunk
    let ckey = MeshCacheKey(chunk_pos,lod_chunk);
    if !terrain_state.ent_cache.contains_key(&ckey) {
        let ent = spawn_chunk_deferred(ckey.clone(), commands);
        terrain_state.ent_cache.insert(ckey.clone(), ent);
    }
    return terrain_state.ent_cache.get(&ckey).unwrap().clone();
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
    
    let mut indices_bevy: Vec<[u32;3]> = vec![];
    for n in 0..(mixed_indcs.len()/3) {
        indices_bevy.push([mixed_indcs[n*3+0],mixed_indcs[n*3+1],mixed_indcs[n*3+2]]);
    }

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
    )
}

fn build_terrain_chunk_plane(
    chunkx: i32,
    chunkz: i32,
) -> Mesh {
    let segx = (chunkx as f32 * CHUNK_LENGTH) as f32;
    let segz = (chunkz as f32 * CHUNK_LENGTH) as f32;
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    let iso = IsosurfaceSource::new(chunkx, chunkz);

    for ix in 0..=CHUNK_SEGS {
        for iz in 0..=CHUNK_SEGS {
            let seglen = CHUNK_LENGTH / (CHUNK_SEGS as f32);
            let y = CHUNK_LENGTH * iso.heightfn(ix as f32 / (CHUNK_SEGS as f32), iz as f32 / (CHUNK_SEGS as f32));
            vertices.push(Vec3::new(ix as f32 * seglen, y, iz as f32 * seglen));
        }
    }
    for ix in 0..CHUNK_SEGS {
        for iz in 0..CHUNK_SEGS {
            // Start of the two relevant rows of vertices.
            let row0 = ix * (CHUNK_SEGS + 1);
            let row1 = (ix + 1) * (CHUNK_SEGS + 1);
            // Two triangles making up a not-very-flat quad for each segment of the bowl.
            indices.push([(row0 + iz) as u32, (row0 + iz + 1) as u32, (row1 + iz) as u32]);
            indices.push([(row1 + iz) as u32, (row0 + iz + 1) as u32, (row1 + iz + 1) as u32]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices.clone(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vertices.clone().into_iter().map(|v| [v.x/2., v.z/2.]).collect::<Vec<[f32;2]>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vertices.clone().into_iter().map({|v|
                // estimate normal from nearby slope
                Vec3::new(iso.heightfn(v.x + segx - 0.1, v.z + segz) - 
                    iso.heightfn(v.x + segx + 0.1, v.z + segz),1.0,
                    iso.heightfn(v.x + segx, v.z + segz - 0.1) - 
                    iso.heightfn(v.x + segx, v.z + segz + 0.1)).normalize().into()
            }).collect::<Vec<[f32;3]>>(),
        )
        .with_indices(Some(Indices::U32(indices.clone().into_iter().flatten().collect())))
}
