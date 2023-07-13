pub mod consts;
pub mod mesh;
pub mod model;
pub mod pipelines;
pub mod renderer;

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
        Globals,

        character::{
            CharacterPipeline,
            Locals as CharacterLocals
        },

        skybox::{
            create_mesh as create_skybox_mesh,

            SkyboxPipeline,
            Locals as SkyboxLocals
        }
    }
};

#[cfg(feature = "gl")]
use gfx_device_gl as gfx_backend;

// biblioteca
use gfx;

/// utilizado para representar um dos vários possíveis erros que podem ser omitidos pelo código de renderização
#[derive(Debug)]
pub enum RenderError {
    PipelineError(gfx::PipelineStateError<String>),
    UpdateError(gfx::UpdateError<usize>)
}

/// utilizado para representar uma configuração de renderização específica
pub trait Pipeline {
    type Vertex: Clone +
        gfx::traits::Pod +
        gfx::pso::buffer::Structure<gfx::format::Format>;
}