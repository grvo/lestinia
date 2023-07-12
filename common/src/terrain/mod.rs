pub mod block;
pub mod biome;

// biblioteca
use vek::*;

// caixote
use crate::{
    vol::VolSize,
    volumes::vol_map::VolMap
};

// local
use self::{
    block::Block,
    biome::BiomeKind
};

// chunksize
pub struct ChunkSize;

impl VolSize for ChunkSize {
    const SIZE: Vec3<u32> = Vec3 { x: 32, y: 32, z: 32 };
}

// chunkmeta
pub struct ChunkMeta {
    biome: BiomeKind
}

// terrainmap
pub type TerrainMap = VolMap<Block, ChunkSize, ChunkMeta>;