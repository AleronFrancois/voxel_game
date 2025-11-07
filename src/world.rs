//use bevy::pbr::wireframe::WireframeColor;
//use bevy::pbr::wireframe::Wireframe;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::collections::HashMap;
use avian3d::prelude::Collider;
use avian3d::prelude::RigidBody;
use bevy::mesh::Indices;
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

            // Chunk position
            let chunk_translation = Vec3::new(
                chunk_position.x as f32 * CHUNK_SIZE_X as f32,
                0.0,
                chunk_position.z as f32 * CHUNK_SIZE_Z as f32,
            );

            // Extract vertices and indices first
            let (vertices, triangles) = get_data(&chunk_mesh);

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
                Transform::from_translation(chunk_translation),
                GlobalTransform::default(),
            )).id();

            // Spawn collider
            let collider_entity = commands.spawn((
                RigidBody::Static,
                Collider::trimesh(vertices, triangles),
                Transform::from_translation(chunk_translation),
                Name::new("ChunkCollider"),
            )).id();

            chunk_entities.map.insert(chunk_position, chunk_entity);
            chunk_entities.colliders.insert(chunk_position, collider_entity);
            update_chunks(chunk_position, &world, &chunk_entities, &mut meshes, &mut commands);
        }
    }
}


/// Extracts vertices and indices from the chunk mesh.
fn get_data(chunk_mesh: &Mesh) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let vertices = chunk_mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .to_vec();
    let vertices: Vec<Vec3> = vertices
        .iter()
        .map(|&[x, y, z]| Vec3::new(x, y, z))
        .collect();

    let indices_u32 = if let Some(Indices::U32(index)) = chunk_mesh.indices() {
        index
    } else {
        panic!("Expected U32 indices");
    };

    let mut triangles = Vec::with_capacity(indices_u32.len() / 3);
    for chunk in indices_u32.chunks(3) {
        triangles.push([chunk[0], chunk[1], chunk[2]]);
    }

    (vertices, triangles)
}


/// Unloads chunk entities.
pub fn unload_chunks(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut world: ResMut<WorldChunks>,
    mut chunk_entities: ResMut<ChunkEntities>,
) {
    if let Ok(player_transform) = player.single() {
        let player_chunk_x = (player_transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let player_chunk_z = (player_transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;

        world.chunks.retain(|chunk_pos, _chunk| {
            let distance_x = chunk_pos.x - player_chunk_x;
            let distance_z = chunk_pos.z - player_chunk_z;

            if distance_x.abs() > UNLOAD_DISTANCE || distance_z.abs() > UNLOAD_DISTANCE {
                if let Some(entity) = chunk_entities.map.remove(chunk_pos) {
                    commands.entity(entity).despawn();
                }
                if let Some(entity) = chunk_entities.colliders.remove(chunk_pos) {
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