// biblioteca
use vek::*;

use gfx::{
    self,

    traits::{
        Device,
        FactoryExt
    }
};

// local
use super::{
    consts::Consts,
    model::Model,
    mesh::Mesh,

    Pipeline,
    RenderError,

    gfx_backend,

    pipelines::{
        Globals,
        character,
        skybox
    }
};

/// representa o formato da cor da janela
pub type TgtColorFmt = gfx::format::Srgba8;

/// representa o formato da profundidade da janela
pub type TgtDepthFmt = gfx::format::DepthStencil;

/// auxilia o alvo da cor da janela
pub type TgtColorView = gfx::handle::RenderTargetView<gfx_backend::Resources, TgtColorFmt>;

/// auxilia o alvo da profundidade da janela
pub type TgtDepthView = gfx::handle::DepthStencilView<gfx_backend::Resources, TgtDepthFmt>;

/// tipagem que encapsula estado renderizado
pub struct Renderer {
    device: gfx_backend::Device,
    encoder: gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,
    factory: gfx_backend::Factory,

    tgt_color_view: TgtColorView,
    tgt_depth_view: TgtDepthView,

    skybox_pipeline: GfxPipeline<skybox::pipe::Init<'static>>,
    character_pipeline: GfxPipeline<character::pipe::Init<'static>>
}

impl Renderer {
    /// cria um novo `renderer` por meio de uma variedade de componentes de backend e alvos de janela
    pub fn new(
        device: gfx_backend::Device,
        mut factory: gfx_backend::Factory,

        tgt_color_view: TgtColorView,
        tgt_depth_view: TgtDepthView
    ) -> Result<Self, RenderError> {
        // constrói uma pipeline para as skyboxes renderizadas
        let skybox_pipeline = create_pipeline(
            &mut factory,
            skybox::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.frag"))
        )?;

        // constrói uma pipeline para os caracteres renderizados
        let character_pipeline = Self::new_pipeline(
            &mut factory,
            character::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/character.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/character.frag"))
        )?;

        Ok(Self {
            device,
            encoder: factory.create_command_buffer().into(),
            factory,

            tgt_color_view,
            tgt_depth_view,

            skybox_pipeline,
            character_pipeline
        })
    }

    /// enfileira a limpeza de cor e profundidade para um novo frame a ser renderizado
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
    pub fn create_consts<T: Copy + gfx::traits::Pod>(&mut self) -> Result<Consts<T>, RenderError> {
        Ok(Consts::new(&mut self.factory))
    }

    /// cria uma nova variedade de constantes com um valor
    pub fn create_consts_with<T: Copy + gfx::traits::Pod>(
        &mut self,
        val: T
    ) -> Result<Consts<T>, RenderError> {
        let mut consts = self.create_consts()?;

        consts.update(&mut self.encoder, val)?;

        Ok(consts)
    }

    /// atualiza o conjunto de constantes com novo valor
    pub fn update_consts<T: Copy + gfx::traits::Pod>(
        &mut self,
        consts: &mut Consts<T>,
        val: T
    ) -> Result<(), RenderError> {
        consts.update(&mut self.encoder, val)
    }

    /// criar novo modelo por meio do mesh fornecido
    pub fn create_model<P: Pipeline>(&mut self, mesh: &Mesh<P>) -> Result<Model<P>, RenderError> {
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

struct GfxPipeline<P: gfx::pso::PipelineInit> {
    pso: gfx::pso::PipelineState<gfx_backend::Resources, P::Meta>
}

/// cria uma nova pipeline para o shader e fragmento de vertex fornecido
fn create_pipeline<'a, P: gfx::pso::PipelineInit>(
    factory: &mut gfx_backend::Factory,
    
    pipe: P,

    vs: &[u8],
    fs: &[u8]
) -> Result<GfxPipeline<P>, RenderError> {
    let program = factory
        .link_program(vs, fs)
        .map_err(|err| RenderError::PipelineError(gfx::PipelineStateError::Program(err)))?;

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
                .map_err(|err| RenderError::PipelineError(match err {
                    gfx::PipelineStateError::Program(err) =>
                        gfx::PipelineStateError::Program(err),

                    gfx::PipelineStateError::DescriptorInit(err) =>
                        gfx::PipelineStateError::DescriptorInit(err.into()),

                    gfx::PipelineStateError::DeviceCreate(err) =>
                        gfx::PipelineStateError::DeviceCreate(err)
                }))?
    })
}