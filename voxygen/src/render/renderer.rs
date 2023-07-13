// biblioteca
use vek::*;

use gfx::{
    self,

    traits::{
        Device,
        FactoryExt
    }
};

// caixote
use crate::VoxygenErr;

// local
use super::{
    consts::Consts,
    model::Model,
    mesh::Mesh,

    Pipeline,
    RenderErr,

    gfx_backend,

    pipelines::{
        Globals,
        character,
        skybox
    }
};

pub type TgtColorFmt = gfx::format::Srgba8;
pub type TgtDepthFmt = gfx::format::DepthStencil;

pub type TgtColorView = gfx::handle::RenderTargetView<gfx_backend::Resources, TgtColorFmt>;
pub type TgtDepthView = gfx::handle::DepthStencilView<gfx_backend::Resources, TgtDepthFmt>;

pub struct Renderer {
    device: gfx_backend::Device,
    encoder: gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,
    factory: gfx_backend::Factory,

    tgt_color_view: TgtColorView,
    tgt_depth_view: TgtDepthView,

    skybox_pipeline: GfxPipeline<skybox::pipe::Init<'static>>
    // character_pipeline: GfxPipeline<character::pipe::Init<'static>>
}

impl Renderer {
    pub fn new(
        device: gfx_backend::Device,
        mut factory: gfx_backend::Factory,

        tgt_color_view: TgtColorView,
        tgt_depth_view: TgtDepthView
    ) -> Result<Self, RenderErr> {
        // constrói uma pipeline para as skyboxes renderizadas
        let skybox_pipeline = Self::create_pipeline(
            &mut factory,
            skybox::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.frag"))
        )?;

        // constrói uma pipeline para os caracteres renderizados
        /*
        let character_pipeline = Self::new_pipeline(
            &mut factory,
            character::pipe::new(),

            include_bytes!("shaders/character.vert"),
            include_bytes!("shaders/character.frag")
        )?;
        */

        Ok(Self {
            device,
            encoder: factory.create_command_buffer().into(),
            factory,

            tgt_color_view,
            tgt_depth_view,

            skybox_pipeline
            // character_pipeline
        })
    }

    pub fn clear(&mut self, col: Rgba<f32>) {
        self.encoder.clear(&self.tgt_color_view, col.into_array());
        self.encoder.clear_depth(&self.tgt_depth_view, 1.0);
    }

    /// performa todas as chamadas de draw nesse frame e limpa os itens descartados
    pub fn flush(&mut self) {
        self.encoder.flush(&mut self.device);
        self.device.cleanup();
    }

    /// cria uma nova pipeline para o shader de vertex e fragmento de shader fornecido
    fn create_pipeline<'a, P: gfx::pso::PipelineInit>(
        factory: &mut gfx_backend::Factory,
        pipe: P,

        vs: &[u8],
        fs: &[u8]
    ) -> Result<GfxPipeline<P>, RenderErr> {
        let program = factory
            .link_program(vs, fs)
            .map_err(|err| RenderErr::PipelineErr(gfx::PipelineStateError::Program(err)))?;

        Ok(GfxPipeline {
            pso: factory.create_pipeline_from_program(
                    &program,

                    gfx::Primitive::TriangleList,
                    gfx::state::Rasterizer {
                        front_face: gfx::state::FrontFace::CounterClockwise,
                        cull_face: gfx::state::CullFace::Back,
                        method: gfx::state::RasterMethod::Fill,
                        offset: None,
                        samples: Some(gfx::state::MultiSample)
                    },

                    pipe
                )
                    // fazer algumas coisas divertidas para contornar uma esquisitice nas regras de propriedade de erro do gfx
                    .map_err(|err| RenderErr::PipelineErr(match err {
                        gfx::PipelineStateError::Program(err) => gfx::PipelineStateError::Program(err),
                        gfx::PipelineStateError::DescriptorInit(err) => gfx::PipelineStateError::DescriptorInit(err.into()),
                            gfx::PipelineStateError::DeviceCreate(err) => gfx::PipelineStateError::DeviceCreate(err)
                    }))?

            program
        })
    }

    /// criar novo modelo por meio do mesh fornecido
    pub fn create_model<P: Pipeline>(&mut self, mesh: &Mesh<P>) -> Result<Model<P>, RenderErr> {
        Ok(Model::new(
            &mut self.factory,

            mesh
        ))
    }

    /// enfileirar a renderização do modelo de skybox fornecido no frame que virá em breve
    pub fn render_skybox(
        &mut self,

        model: &Model<skybox::SkyboxPipeline>,

        locals: &Consts<skybox::Locals>,
        globals: &Consts<Globals>
    ) {
        self.encoder.draw(
            &model.slice,
            &self.skybox_pipeline.pso,

            &skybox::pipe::Data {
                vbuf: model.vbuf.clone(),
                locals: locals.buf.clone(),
                globals: globals.buf.clone(),

                tgt_color: self.tgt_color_view.clone(),
                tgt_depth: self.tgt_depth_view.clone()
            }
        );
    }
}

pub struct GfxPipeline<P: gfx::pso::PipelineInit> {
    program: gfx::handle::Program<gfx_backend::Resources>,

    pso: gfx::pso::PipelineState<gfx_backend::Resources, P::Meta>
}