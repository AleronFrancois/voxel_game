//use bevy::pbr::wireframe::WireframeColor;
//use bevy::pbr::wireframe::Wireframe;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::collections::HashMap;
use avian3d::prelude::*;
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
    pub colliders: HashMap<IVec3, Entity>,
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


/// Generates a grid of chunks around the player and adds them to the queue.
pub fn queue_chunks(
    player: Query<&Transform, With<Player>>,
    mut player_chunks: ResMut<PlayerChunk>,
    world: ResMut<WorldChunks>,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    if let Ok(transform) = player.single() {
        let chunk_x = (transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let chunk_z = (transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;
        let current_chunk = IVec3::new(chunk_x, 0, chunk_z);

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


/// Loads a select few chunks from the queue per frame.
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
            // Build chunk mesh
            chunk_queue.queued_set.remove(&chunk_position);
            let prepared_geometry = prepare_geometry(&chunk_position);
            world.chunks.insert(chunk_position, prepared_geometry);
            let chunk_mesh = build_mesh(chunk_position, &world);

            // Glabal chunk position
            let global_position = Vec3::new(
                chunk_position.x as f32 * CHUNK_SIZE_X as f32,
                0.0,
                chunk_position.z as f32 * CHUNK_SIZE_Z as f32,
            );

            // Add mesh to asset storage
            let mesh_handle = meshes.add(chunk_mesh);
            let texture_handle = asset_server.load("texture_atlas.png");
            let material_handle = materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle),
                perceptual_roughness: 0.2,
                ..Default::default()
            });

            // Spawn mesh
            let chunk_entity = commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Transform::from_translation(global_position),
                GlobalTransform::default(),
            )).id();

            // Gets points for each solid block
            let points = get_points(&chunk_position, &world);

            // Spawn collider
            let collider_entity = commands.spawn((
                RigidBody::Static,
                Collider::voxels_from_points(Vec3::splat(1.0), &points),
                Transform::from_translation(global_position),
                Name::new("ChunkCollider"),
            )).id();

            // Add chunk and collider entities to storage
            chunk_entities.map.insert(chunk_position, chunk_entity);
            chunk_entities.colliders.insert(chunk_position, collider_entity);
            update_chunks(chunk_position, &world, &chunk_entities, &mut meshes, &mut commands);
        }
    }
}


/// Extracts solid block points from chunk.
pub fn get_points(chunk_position: &IVec3, world: &WorldChunks) -> Vec<Vec3> {
    let chunk = world.chunks.get(&chunk_position).unwrap();
        let mut points = Vec::new();
        for block_index in 0..CHUNK_VOLUME {
            let (block_x, block_y, block_z) = GET_COORDS[block_index];
            let block = chunk.blocks[Chunk::get_index(block_x, block_y, block_z)];
            if block.is_solid() {
                points.push(Vec3::new(block_x as f32, block_y as f32, block_z as f32));
            }
        }

        points // Returns block points
}


/// Unloads chunk entities around the player.
pub fn unload_chunks(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut world: ResMut<WorldChunks>,
    mut chunk_entities: ResMut<ChunkEntities>,
) {
    if let Ok(player_transform) = player.single() {
        let player_chunk_x = (player_transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let player_chunk_z = (player_transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;

        world.chunks.retain(|chunk_position, _chunk| {
            let distance_x = chunk_position.x - player_chunk_x;
            let distance_z = chunk_position.z - player_chunk_z;

            if distance_x.abs() > UNLOAD_DISTANCE || distance_z.abs() > UNLOAD_DISTANCE {
                if let Some(entity) = chunk_entities.map.remove(chunk_position) {
                    commands.entity(entity).despawn();
                }
                if let Some(entity) = chunk_entities.colliders.remove(chunk_position) {
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


/// Updates chunk mesh visibility around newly spawned chunk.
pub fn update_chunks(
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


