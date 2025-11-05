use bitflags::bitflags;


#[derive(Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}


#[derive(Clone, Copy)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
}


bitflags! {
    #[derive(Copy, Clone)]
    pub struct BlockFaces: u8 {
        const FRONT_FACE  = 0b000001;
        const BACK_FACE   = 0b000010;
        const LEFT_FACE   = 0b000100;
        const RIGHT_FACE  = 0b001000;
        const TOP_FACE    = 0b010000;
        const BOTTOM_FACE = 0b100000;
    }
}


impl Block {
    pub fn new() -> Self {
        Self {
            block_type: BlockType::Air,
        }
    }
    

    pub fn is_solid(&self) -> bool {
        match self.block_type {
            BlockType::Air => false,
            _ => true,
        }
    }
}


const ATLAS_SIZE: f32 = 1.0 / 16.0;
const PADDING: f32 = 0.5 / 1024.0;


impl BlockType {
    pub fn get_texture(&self) -> [[f32; 2]; 4] {
        let (texture_x, texture_y) = match self {
            BlockType::Grass => (0, 0),
            BlockType::Dirt  => (2, 0),
            BlockType::Stone => (1, 0),
            BlockType::Sand  => (0, 3),
            _ => (0, 0),
        };

        let u_min = texture_x as f32 * ATLAS_SIZE + PADDING;
        let v_min = texture_y as f32 * ATLAS_SIZE + PADDING;
        let u_max = u_min + ATLAS_SIZE - 2.0 * PADDING;
        let v_max = v_min + ATLAS_SIZE - 2.0 * PADDING;

        [
            [u_min, v_min],
            [u_max, v_min],
            [u_max, v_max],
            [u_min, v_max],
        ]
    }
}


//  Block vertices
pub const VERTICES: [[[f32; 3]; 4]; 6] = [
    [   // Front face
        [0.0, 0.0, 1.0], [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
    ],
    [   // Back face
        [1.0, 0.0, 0.0], [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
    ],
    [   // Left face
        [0.0, 0.0, 0.0], [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0], [0.0, 1.0, 0.0],
    ],
    [   // Right face
        [1.0, 0.0, 1.0], [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0], [1.0, 1.0, 1.0],
    ],
    [   // Top face
        [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
    ],
    [   // Bottom face
        [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0], [0.0, 0.0, 1.0],
    ],
];


// Block normals
pub const NORMALS: [[[f32; 3]; 4]; 6] = [
    [   // Front face
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
    ],
    [   // Back face
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
    ],
    [   // Left face
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
    ],
    [   // Right face
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
    ],
    [   // Top face
        [0.0, 1.0, 0.0],  [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
    ],
    [   // Bottom face
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
    ],
];


// Block indices
pub const INDICES: [u32; 6] = [
    0, 1, 2, 0, 2, 3
];


pub fn offset_vertices(
    face_vertices: &[[f32; 3]; 4], 
    block_offset: [f32; 3], 
    chunk_vertices: &mut Vec<[f32; 3]>,
) {
    chunk_vertices.push([
        face_vertices[0][0] + block_offset[0],
        face_vertices[0][1] + block_offset[1],
        face_vertices[0][2] + block_offset[2],
    ]);
    chunk_vertices.push([
        face_vertices[1][0] + block_offset[0],
        face_vertices[1][1] + block_offset[1],
        face_vertices[1][2] + block_offset[2],
    ]);
    chunk_vertices.push([
        face_vertices[2][0] + block_offset[0],
        face_vertices[2][1] + block_offset[1],
        face_vertices[2][2] + block_offset[2],
    ]);
    chunk_vertices.push([
        face_vertices[3][0] + block_offset[0],
        face_vertices[3][1] + block_offset[1],
        face_vertices[3][2] + block_offset[2],
    ]);
}