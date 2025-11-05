//use bevy::pbr::wireframe::WireframeColor;
//use bevy::pbr::wireframe::Wireframe;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::collections::HashMap;
use bevy::prelude::*;
use glam::IVec3;

use crate::player::*;
use crate::chunk::*;


const PER_FRAME: usize = 1;
const RENDER_DISTANCE: i32 = 13;
const UNLOAD_DISTANCE: i32 = 16;
pub const CHUNK_NEIGHBOURS: [IVec3; 6] = [
    IVec3::new(0,  0,  1), // Chunk infront
    IVec3::new(0,  0, -1), // Chunk behind
    IVec3::new(-1, 0,  0), // Chunk left
    IVec3::new(1,  0,  0), // Chunk right
    IVec3::new(0,  1,  0), // Chunk above
    IVec3::new(0, -1,  0), // Chunk below
];


// Stores all chunks with their position
#[derive(Resource, Default)]
pub struct WorldChunks {
    pub chunks: HashMap<IVec3, Chunk>,
}


// Stores all chunk entities
#[derive(Resource, Default)]
pub struct ChunkEntities {
    pub map: HashMap<IVec3, Entity>,
}


// Stores each chunk to be loaded
#[derive(Resource, Default)]
pub struct ChunkQueue {
    pub queue: VecDeque<IVec3>,
    pub queued_set: HashSet<IVec3>,
}

// Tracks the player's last chunk
#[derive(Resource, Default)]
pub struct PlayerChunk {
    pub last_chunk: IVec3,
}


/// Adds chunks to the queue.
pub fn queue_chunks(
    camera: Query<&Transform, With<CameraSettings>>,
    mut player_chunks: ResMut<PlayerChunk>,
    world: ResMut<WorldChunks>,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    if let Ok(transform) = camera.single() {
        let chunk_x = (transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let chunk_z = (transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;
        let current_chunk = IVec3::new(chunk_x, 0, chunk_z);

        if current_chunk != player_chunks.last_chunk {
            player_chunks.last_chunk = current_chunk;
        
            for distance_x in -RENDER_DISTANCE..=RENDER_DISTANCE {
                for distance_z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                    let chunk_position = IVec3::new(chunk_x + distance_x, 0, chunk_z + distance_z);
                    if !world.chunks.contains_key(&chunk_position) && !chunk_queue.queued_set.contains(&chunk_position) {
                        chunk_queue.queue.push_back(chunk_position);
                        chunk_queue.queued_set.insert(chunk_position);
                    }
                }
            }
        }
    }
}


/// Loads chunks from the queue.
pub fn load_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut world: ResMut<WorldChunks>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    for _chunks in 0..PER_FRAME {
        if let Some(chunk_position) = chunk_queue.queue.pop_front() {
            chunk_queue.queued_set.remove(&chunk_position);
            let prepared_geometry = prepare_geometry(&chunk_position);
            world.chunks.insert(chunk_position, prepared_geometry);
            let chunk_mesh = build_mesh(chunk_position, &world);

            let mesh_handle = meshes.add(chunk_mesh);
            let texture_handle = asset_server.load("texture_atlas.png");
            let material_handle = materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle),
                perceptual_roughness: 0.5,
                ..Default::default()
            });

            let chunk_entity = commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Transform::from_translation(Vec3::new(
                    chunk_position.x as f32 * CHUNK_SIZE_X as f32,
                    0.0,
                    chunk_position.z as f32 * CHUNK_SIZE_Z as f32,
                )),
                GlobalTransform::default(),
                /*Wireframe,
                WireframeColor {
                    color: Color::srgb(0.0, 1.0, 0.0)
                },*/
            )).id();

            chunk_entities.map.insert(chunk_position, chunk_entity);
            update_chunks(chunk_position, &world, &chunk_entities, &mut meshes, &mut commands);
        }
    }
}


/// Unloads chunk entities.
pub fn unload_chunks(
    mut commands: Commands,
    camera: Query<&Transform, With<CameraSettings>>,
    mut world: ResMut<WorldChunks>,
    mut chunk_entities: ResMut<ChunkEntities>,
) {
    if let Ok(transform) = camera.single() {
        let chunk_x = (transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let chunk_z = (transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;

        world.chunks.retain(|position, _| {
            let distance_x = position.x - chunk_x;
            let distance_z = position.z - chunk_z;

            if distance_x.abs() > UNLOAD_DISTANCE || distance_z.abs() > UNLOAD_DISTANCE {
                if let Some(entity) = chunk_entities.map.remove(position) {
                    commands.entity(entity).despawn();
                }
                false
            }
            else {
                true
            }
        });
    }
}


/// Updates chunk mesh visibility.
fn update_chunks(
    chunk_position: IVec3,
    world: &WorldChunks,
    chunk_entities: &ChunkEntities,
    meshes: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
) {
    for neighbour in CHUNK_NEIGHBOURS.iter() {
        let neighbor_position = chunk_position + *neighbour;

        if world.chunks.contains_key(&neighbor_position) {
            let neighbor_mesh = build_mesh(neighbor_position, world);
            let mesh_handle = meshes.add(neighbor_mesh);

            if let Some(entity) = chunk_entities.map.get(&neighbor_position) {
                commands.entity(*entity).insert(Mesh3d(mesh_handle));
            }
        }
    }
}