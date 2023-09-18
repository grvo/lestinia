pub mod block;
pub mod biome;

// re-exportações
pub use self::{
    block::Block,
    biome::BiomeKind
};

// biblioteca
use vek::*;

// caixote
use crate::{
    vol::VolSize,
    
    volumes::{
        vol_map::VolMap,
        chunk::Chunk
    }
};

// terrainchunksize
pub struct TerrainChunkSize;

impl VolSize for TerrainChunkSize {
    const SIZE: Vec3<u32> = Vec3 { x: 32, y: 32, z: 32 };
}

// terrainchunkmeta
pub struct TerrainChunkMeta {
    biome: BiomeKind
}

impl TerrainChunkMeta {
    pub fn void() -> Self {
        Self {
            biome: BiomeKind::Void
        }
    }
}

// tipo de terreno
pub type TerrainChunk = Chunk<Block, TerrainChunkSize, TerrainChunkMeta>;
pub type TerrainMap = VolMap<Block, TerrainChunkSize, TerrainChunkMeta>;
