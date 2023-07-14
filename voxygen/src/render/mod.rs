pub mod consts;
pub mod mesh;
pub mod model;
pub mod pipelines;
pub mod renderer;

mod util;

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

/// utilizado para representar um dos diversos erros possíveis que podem ser omitidos pelo sub-sistema de renderização
#[derive(Debug)]
pub enum RenderError {
    PipelineError(gfx::PipelineStateError<String>),
    UpdateError(gfx::UpdateError<usize>)
}

/// utilizado para representar um dos vários possíveis erros que podem ser omitidos pelo sub-sistema de renderização
///
/// nota que as pipelines estão para renderizar o backend
/// e é necessário modificar a renderização de sub-sistema ao adicionar novas pipelines
///
/// # exemplos
///
/// - `SkyboxPipeline`
/// - `CharacterPipeline`
pub trait Pipeline {
    type Vertex: Clone +
        gfx::traits::Pod +
        gfx::pso::buffer::Structure<gfx::format::Format>;
}