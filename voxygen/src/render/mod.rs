mod mesh;
mod model;
mod renderer;
mod shader_set;

// re-exportações
pub use self::{
    mesh::Mesh,
    model::Model,

    shader_set::ShaderSet,
    renderer::Renderer
};

// biblioteca
use rendy;

#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
type Backend = rendy::empty::Backend;

#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
type Backend = rendy::vulkan::Backend;

/// utilizado para representar um dos vários possíveis erros que podem ser omitidos pelo código de renderização
#[derive(Debug)]
pub enum RenderErr {}

/// utilizado para representar uma configuração de renderização específica
pub trait Pipeline {
    type Vertex;
}