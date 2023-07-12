mod mesh;
mod model;
mod renderer;
mod pipelines;
mod shader_set;

// re-exportações
pub use self::{
    mesh::Mesh,
    model::Model,

    shader_set::ShaderSet,

    renderer::{
        Renderer,

        TgtColorFmt,
        TgtDepthFmt
    }
};

#[cfg(feature = "gl")]
use gfx_device_gl as gfx_backend;

/// utilizado para representar um dos vários possíveis erros que podem ser omitidos pelo código de renderização
#[derive(Debug)]
pub enum RenderErr {}

/// utilizado para representar uma configuração de renderização específica
pub trait Pipeline {
    type Vertex;
}