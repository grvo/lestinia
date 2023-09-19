// padrão
use std::collections::HashMap;

// biblioteca
use vek::*;

// caixote
use crate::{
    Error,

    render::{
        Consts,
        Globals,
        Mesh,
        Model,
        Renderer,

        TerrainPipeline,
        TerrainLocals
    }
};

struct TerrainChunk {
    // dados de gpu
    model: Model<TerrainPipeline>,
    locals: Consts<TerrainLocals>
}

pub struct Terrain {
    chunks: HashMap<Vec3<i32>, TerrainChunk>
}

impl Terrain {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }

    pub fn maintain_gpu_data(&mut self, renderer: &mut Renderer) {
        // todo
    }

    pub fn render(&self, renderer: &mut Renderer, globals: &Consts<Globals>) {
        /* renderer.render_terrain_chunk(
            &self.model,
            globals,
            &self.locals,
            &self.bone_consts
        ); */
    }
}
