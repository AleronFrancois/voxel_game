use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use fastnoise_lite::FastNoiseLite;
use bevy::mesh::Indices;
use std::sync::LazyLock;
use bevy::mesh::Mesh;
use bevy::prelude::*;
use glam::IVec3;

use crate::block::*;
use crate::world::*;


pub const CHUNK_SIZE_X: usize = 16;
pub const CHUNK_SIZE_Y: usize = 16;
pub const CHUNK_SIZE_Z: usize = 16;
pub const CHUNK_SIZE_XY: usize = 
    CHUNK_SIZE_X * CHUNK_SIZE_Z;
pub const CHUNK_VOLUME: usize = 
    CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z;
    

pub static GET_COORDS: LazyLock<[(usize, usize, usize); CHUNK_VOLUME]> = LazyLock::new(|| {
    let mut block_coordinates = [(0, 0, 0); CHUNK_VOLUME];
    let mut block_index = 0;

    for block_y in 0..CHUNK_SIZE_Y {
        for block_z in 0..CHUNK_SIZE_Z {
            for block_x in 0..CHUNK_SIZE_X {
                block_coordinates[block_index] = (block_x, block_y, block_z);
                block_index += 1;
            }
        }
    }

    block_coordinates
});


#[derive(Clone, Copy)]
pub struct Chunk {
    pub blocks: [Block; CHUNK_VOLUME],
}


impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [Block::default(); CHUNK_VOLUME],
        }
    }


    pub fn get_index(block_x: usize, block_y: usize, block_z: usize) -> usize {
        block_x + block_z * CHUNK_SIZE_X + block_y * CHUNK_SIZE_XY
    }
}


pub fn prepare_geometry(chunk_position: &IVec3) -> Chunk {
    let mut chunk = Chunk::new();
    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    noise.set_seed(Some(213123));
    noise.set_frequency(Some(0.065));
    

    let base_x = chunk_position.x as f32 * CHUNK_SIZE_X as f32;
    let base_z = chunk_position.z as f32 * CHUNK_SIZE_Z as f32;

    for block_z in 0..CHUNK_SIZE_Z {
        for block_x in 0..CHUNK_SIZE_X {
            for block_y in 0..CHUNK_SIZE_Y {
                let world_x = base_x + block_x as f32;
                let world_z = base_z + block_z as f32;
                let height = noise.get_noise_2d(world_x, world_z) * 5.0 + 8.0;

                if block_y > height as usize {
                    continue;
                }

                let block_type = if block_y == height as usize {  
                    BlockType::Grass
                }
                else if block_y >= (height as usize).saturating_sub(3) {
                    BlockType::Dirt
                }
                else {
                    if BlockType::get_chance(BlockType::Coal) {
                        BlockType::Coal
                    } 
                    else {
                        BlockType::Stone
                    }
                };

                chunk.blocks[Chunk::get_index(block_x, block_y, block_z)] = Block { block_type };
            }
        }
    }
    
    chunk
}


pub fn build_mesh(chunk_position: IVec3, world: &WorldChunks) -> Mesh {
    let chunk = &world.chunks[&chunk_position];
    let mut chunk_vertices: Vec<[f32; 3]> = Vec::new();
    let mut chunk_normals: Vec<[f32; 3]> = Vec::new();
    let mut chunk_uvs: Vec<[f32; 2]> = Vec::new();
    let mut chunk_indices: Vec<u32> = Vec::new();
    let mut index_counter = 0;

    for block_index in 0..CHUNK_VOLUME {
        if !chunk.blocks[block_index].is_solid() {
            continue;
        }

        let (block_x, block_y, block_z) = GET_COORDS[block_index];
        let visible_faces = get_visibility((block_x, block_y, block_z), &chunk, &world, chunk_position);

        if visible_faces.is_empty() {
            continue;
        }

        let block = chunk.blocks[block_index];
        let block_offset = [
            block_x as f32, 
            block_y as f32, 
            block_z as f32,
        ];

        for face in visible_faces {
            let face_index = face.bits().trailing_zeros() as usize;
            let face_vertices = VERTICES[face_index];
            let face_normals = NORMALS[face_index];
            let face_uvs = block.block_type.get_texture();

            offset_vertices(
                &face_vertices, 
                block_offset, 
                &mut chunk_vertices, 
            );

            chunk_normals.extend_from_slice(&face_normals);
            chunk_uvs.extend_from_slice(&face_uvs);

            for index in 0..6 {
                chunk_indices.push(index_counter + INDICES[index]);
            }
            index_counter += 4;
        }
    }

    let mut chunk_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk_vertices);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, chunk_normals);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk_uvs);
    chunk_mesh.insert_indices(Indices::U32(chunk_indices));

    chunk_mesh
}


fn get_visibility(
    (block_x, block_y, block_z): (usize, usize, usize), 
    chunk: &Chunk,
    world: &WorldChunks,
    chunk_position: IVec3,
) -> BlockFaces {
    let mut visible_faces = BlockFaces::empty();

    if block_z + 1 >= CHUNK_SIZE_Z {
        if let Some(neighbor) = world.chunks.get(&(chunk_position + CHUNK_NEIGHBOURS[0])) {
            if !neighbor.blocks[Chunk::get_index(block_x, block_y, 0)].is_solid() {
                visible_faces |= BlockFaces::FRONT_FACE;
            }
        } 
        else {
            visible_faces |= BlockFaces::FRONT_FACE;
        }
    } 
    else if !chunk.blocks[Chunk::get_index(block_x, block_y, block_z + 1)].is_solid() {
        visible_faces |= BlockFaces::FRONT_FACE;
    }

    if block_z == 0 {
        if let Some(neighbor) = world.chunks.get(&(chunk_position + CHUNK_NEIGHBOURS[1])) {
            if !neighbor.blocks[Chunk::get_index(block_x, block_y, CHUNK_SIZE_Z - 1)].is_solid() {
                visible_faces |= BlockFaces::BACK_FACE;
            }
        } 
        else {
            visible_faces |= BlockFaces::BACK_FACE;
        }
    } 
    else if !chunk.blocks[Chunk::get_index(block_x, block_y, block_z - 1)].is_solid() {
        visible_faces |= BlockFaces::BACK_FACE;
    }

    if block_x == 0 {
        if let Some(neighbor) = world.chunks.get(&(chunk_position + CHUNK_NEIGHBOURS[2])) {
            if !neighbor.blocks[Chunk::get_index(CHUNK_SIZE_X - 1, block_y, block_z)].is_solid() {
                visible_faces |= BlockFaces::LEFT_FACE;
            }
        } 
        else {
            visible_faces |= BlockFaces::LEFT_FACE;
        }
    } 
    else if !chunk.blocks[Chunk::get_index(block_x - 1, block_y, block_z)].is_solid() {
        visible_faces |= BlockFaces::LEFT_FACE;
    }

    if block_x + 1 >= CHUNK_SIZE_X {
        if let Some(neighbor) = world.chunks.get(&(chunk_position + CHUNK_NEIGHBOURS[3])) {
            if !neighbor.blocks[Chunk::get_index(0, block_y, block_z)].is_solid() {
                visible_faces |= BlockFaces::RIGHT_FACE;
            }
        } 
        else {
            visible_faces |= BlockFaces::RIGHT_FACE;
        }
    } 
    else if !chunk.blocks[Chunk::get_index(block_x + 1, block_y, block_z)].is_solid() {
        visible_faces |= BlockFaces::RIGHT_FACE;
    }

    if block_y + 1 >= CHUNK_SIZE_Y {
        if let Some(neighbor) = world.chunks.get(&(chunk_position + CHUNK_NEIGHBOURS[4])) {
            if !neighbor.blocks[Chunk::get_index(block_x, 0, block_z)].is_solid() {
                visible_faces |= BlockFaces::TOP_FACE;
            }
        } 
        else {
            visible_faces |= BlockFaces::TOP_FACE;
        }
    } 
    else if !chunk.blocks[Chunk::get_index(block_x, block_y + 1, block_z)].is_solid() {
        visible_faces |= BlockFaces::TOP_FACE;
    }

    if block_y == 0 {
        if let Some(neighbor) = world.chunks.get(&(chunk_position + CHUNK_NEIGHBOURS[5])) {
            if !neighbor.blocks[Chunk::get_index(block_x, CHUNK_SIZE_Y - 1, block_z)].is_solid() {
                visible_faces |= BlockFaces::BOTTOM_FACE;
            }
        } 
        else {
            visible_faces |= BlockFaces::BOTTOM_FACE;
        }
    } 
    else if !chunk.blocks[Chunk::get_index(block_x, block_y - 1, block_z)].is_solid() {
        visible_faces |= BlockFaces::BOTTOM_FACE;
    }

    visible_faces
}