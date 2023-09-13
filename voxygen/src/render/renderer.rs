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
        figure,
        skybox
    }
};

/// representa o formato da cor de janela
pub type TgtColorFmt = gfx::format::Rgba8;

/// representa o formato da profundidade da janela
pub type TgtDepthFmt = gfx::format::DepthStencil;

/// uma ajuda para o alvo da cor da janela
pub type TgtColorView = gfx::handle::RenderTargetView<gfx_backend::Resources, TgtColorFmt>;

/// uma ajuda para o alvo da profundidade da janela
pub type TgtDepthView = gfx::handle::DepthStencilView<gfx_backend::Resources, TgtDepthFmt>;

/// um tipo que encapsula o estado renderizado. renderer é central à renderização de voxygen
/// e contém qualquer estado necessário para tal
pub struct Renderer {
    device: gfx_backend::Device,
    encoder: gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,
    factory: gfx_backend::Factory,

    tgt_color_view: TgtColorView,
    tgt_depth_view: TgtDepthView,

    skybox_pipeline: GfxPipeline<skybox::pipe::Init<'static>>,
    figure_pipeline: GfxPipeline<figure::pipe::Init<'static>>
}

impl Renderer {
    /// cria um novo `renderer` por meio de uma variedade de componentes de backend e alvos de janela
    pub fn new(
        device: gfx_backend::Device,
        mut factory: gfx_backend::Factory,

        tgt_color_view: TgtColorView,
        tgt_depth_view: TgtDepthView
    ) -> Result<Self, RenderError> {
        // constrói uma pipeline para renderizar skyboxes
        let skybox_pipeline = create_pipeline(
            &mut factory,
            skybox::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.frag"))
        )?;

        // constrói uma pipeline para renderizar skyboxes
        let figure_pipeline = create_pipeline(
            &mut factory,
            figure::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/figure.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/figure.frag"))
        )?;

        Ok(Self {
            device,
            encoder: factory.create_command_buffer().into(),
            factory,

            tgt_color_view,
            tgt_depth_view,

            skybox_pipeline,
            figure_pipeline
        })
    }

    /// lista a limpeza de cor e profundidade prontos para um novo frame ser renderizado
    /// TODO: fazer a versão disso que não limpe o alvo colorido por rapidez
    fn clear(&mut self, col: Rgba<f32>) {
        self.encoder.clear(&self.tgt_color_view, col.into_array());
        self.encoder.clear_depth(&self.tgt_depth_view, 1.0);
    }

    /// performar todas as listas de desenhos que chamam esse frame e limpa os itens descartados
    pub fn flush(&mut self) {
        self.encoder.flush(&mut self.device);
        self.device.cleanup();
    }

    /// cria um novo conjunto de constantes com os valores fornecidos
    pub fn create_consts<T: Copy + gfx::traits::Pod> {
        &mut self,
        vals: &[T]
    } -> Result<Consts<T>, RenderError> {
        let mut consts = Consts::new(&mut self.factory, vals.len());

        consts.update(&mut self.encoder, vals)?;

        Ok(consts)
    }

    /// atualizar lista de conjuntos com os valores fornecidos
    pub fn update_consts<T: Copy + gfx::traits::Pod> {
        &mut self,
        consts: &mut Consts<T>,
        vals: &[T]
    } -> Result<(), RenderError> {
        consts.update(&mut self.encoder, vals)
    }

    /// cria um novo modelo a partir do mesh fornecido
    pub fn create_model<P: Pipeline>(&mut self, mesh: &Mesh<P>) -> Result<Model<P>, RenderError> {
        Ok(Model::new(
            &mut self.factory,
            mesh
        ))
    }

    /// lista a renderização do modelo de skybox fornecido
    pub fn render_skybox(
        &mut self,

        model: &Model<skybox::SkyboxPipeline>,
        globals: &Consts<Globals>,
        locals: &Consts<skybox::Locals>
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

    /// lista a renderização com o modelo de figura fornecido
    pub fn render_figure(
        &mut self,

        model: &Model<skybox::FigurePipeline>,
        globals: &Consts<Globals>,
        locals: &Consts<figure::Locals>,
        bones: &Consts<figure::BoneData>
    ) {
        self.encoder.draw(
            &model.slice,
            &self.figure_pipeline.pso,

            &figure::pipe::Data {
                vbuf: model.vbuf.clone(),

                locals: locals.buf.clone(),
                globals: globals.buf.clone(),

                bones: bones.buf.clone(),

                tgt_color: self.tgt_color_view.clone(),
                tgt_depth: self.tgt_depth_view.clone()
            }
        );
    }
}

struct GfxPipeline<P: gfx::pso::PipelineInit> {
    pso: gfx::pso::PipelineState<gfx_backend::Resources, P::Meta>
}

/// cria uma nova pipeline a partir do shader de vertex fornecido
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
            // fazer coisas ao redor de oddity
            .map_err(|err| RenderError::PipelineError(match err {
                gfx::PipelineStateError::Program(err) =>
                    gfx::PipelineStateError::Program(err),
                gfx::PipelineStateError::DescriptorInit(err) =>
                    gfx::PipelineStateError::DescriptorInit(err.into()),
                gfx::PipelineStateError::DeviceCreate(err) =>
                    gfx::PipelineStateError::DeviceCreate(err),
            }))?
    })
}
