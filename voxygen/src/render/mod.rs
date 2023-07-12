mod consts;
mod mesh;
mod model;
mod pipelines;
mod renderer;

// re-exportações
pub use self::{
    consts::Consts,
    mesh::{Mesh, Quad},
    model::Model,

    renderer::{
        Renderer,

        TgtColorFmt,
        TgtDepthFmt
    },

    pipelines::{
        character::CharacterPipeline,
        skybox::SkyboxPipeline
    }
};

#[cfg(feature = "gl")]
use gfx_device_gl as gfx_backend;

// biblioteca
use gfx;

/// utilizado para representar um dos vários possíveis erros que podem ser omitidos pelo código de renderização
#[derive(Debug)]
pub enum RenderErr {
    PipelineErr(gfx::PipelineStateError<String>),
    UpdateErr(gfx::UpdateError<usize>)
}

/// utilizado para representar uma configuração de renderização específica
pub trait Pipeline {
    type Vertex: Clone +
        gfx::traits::Pod +
        gfx::pso::buffer::Structure<gfx::format::Format>;
}