pub mod consts;
pub mod mesh;
pub mod model;
pub mod pipelines;
pub mod renderer;
pub mod texture;

mod util;

// re-exportações
pub use self::{
    consts::Consts,
    model::Model,
    texture::Texture,

    mesh::{
        Mesh,
        Tri,
        Quad
    },

    renderer::{
        Renderer,

        TgtColorFmt,
        TgtDepthFmt
    },

    pipelines::{
        Globals,

        figure::{
            FigurePipeline,
            Locals as FigureLocals,
            BoneData as FigureBoneData
        },

        skybox::{
            create_mesh as create_skybox_mesh,

            SkyboxPipeline,
            Locals as SkyboxLocals
        },

        terrain::{
            TerrainPipeline,
            Locals as TerrainLocals
        },

        ui::{
            push_quad_to_mesh as push_ui_quad_to_mesh,

			Mode as UiMode,
            UiPipeline
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
    UpdateError(gfx::UpdateError<usize>),
	TexUpdateError(gfx::UpdateError<[u16; 3]>),
    CombinedError(gfx::CombinedError)
}

/// utilizado para representar um dos vários possíveis erros que podem ser omitidos pelo sub-sistema de renderização
///
/// nota que as pipelines estão para renderizar o backend
/// e é necessário modificar a renderização de sub-sistema ao adicionar novas pipelines
///
/// # exemplos
///
/// - `SkyboxPipeline`
/// - `FigurePipeline`
pub trait Pipeline {
    type Vertex: Clone +
        gfx::traits::Pod +
        gfx::pso::buffer::Structure<gfx::format::Format>;
}
