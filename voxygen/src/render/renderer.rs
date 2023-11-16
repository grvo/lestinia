use vek::*;

use gfx::{
    self,

    traits::{
        Device,
        FactoryExt
    }
};

use image;

use super::{
    consts::Consts,
    mesh::Mesh,

    model::Model,
    texture::Texture,

    Pipeline,
    RenderError,

    gfx_backend,

    pipelines::{
        Globals,
        figure,
        skybox,
        terrain,
        ui
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
    figure_pipeline: GfxPipeline<figure::pipe::Init<'static>>,
    terrain_pipeline: GfxPipeline<terrain::pipe::Init<'static>>,
    ui_pipeline: GfxPipeline<ui::pipe::Init<'static>>
}

impl Renderer {
    /// cria um novo `renderer` por meio de uma variedade de componentes de backend e alvos de janela
    pub fn new(
        device: gfx_backend::Device,
        mut factory: gfx_backend::Factory,

        tgt_color_view: TgtColorView,
        tgt_depth_view: TgtDepthView
    ) -> Result<Self, RenderError> {
        // constrói uma pipeline para renderização de skyboxes
        let skybox_pipeline = create_pipeline(
            &mut factory,
            skybox::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/skybox.frag"))
        )?;

        // constrói uma pipeline para renderização de skyboxes
        let figure_pipeline = create_pipeline(
            &mut factory,
            figure::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/figure.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/figure.frag"))
        )?;

        // constrói uma pipeline para renderização de terrenos
        let terrain_pipeline = create_pipeline(
            &mut factory,
            terrain::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/terrain.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/terrain.frag"))
        )?;

        // constrói uma pipeline para renderização de elementos ui
        let ui_pipeline = create_pipeline(
            &mut factory,
            ui::pipe::new(),

            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/ui.vert")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/ui.frag"))
        )?;

        Ok(Self {
            device,
            encoder: factory.create_command_buffer().into(),
            factory,

            tgt_color_view,
            tgt_depth_view,

            skybox_pipeline,
            figure_pipeline,
            terrain_pipeline,
            ui_pipeline
        })
    }

    /// obtém referências para a renderização interna que é mostrada diretamente pela janela
    pub fn target_views(&self) -> (&TgtColorView, &TgtDepthView) {
        (&self.tgt_color_view, &self.tgt_depth_view)
    }

    /// obtém referências mutáveis para a renderização interna que é mostrada diretamente pela janela
    pub fn target_views_mut(&mut self) -> (&mut TgtColorView, &mut TgtDepthView) {
        (&mut self.tgt_color_view, &mut self.tgt_depth_view)
    }

    /// obtém a resolução do alvo de renderização
    pub fn get_resolution(&self) -> Vec2<u16> {
        Vec2::new(
            self.tgt_color_view.get_dimensions().0,
            self.tgt_color_view.get_dimensions().1
        )
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

    /// cria uma nova textura por meio da imagem fornecida
    pub fn create_texture<P: Pipeline>(&mut self, image: &image::DynamicImage) -> Result<Texture<P>, RenderError> {
        Texture::new(
            &mut self.factory,

            image
        )
    }

	/// cria uma nova textura dinâmica com dimensões específicas
	pub fn create_dynamic_texture<P: Pipeline>(
		&mut self,

		dims: Vec2<u16>
	) -> Result<Texture<P>, RenderError> {
		Texture::new_dynamic(
			&mut self.factory,

			dims.x,
			dims.y
		)
	}

	/// atualizar a textura com o offset, tamanho e dado fornecidos
	pub fn update_texture<P: Pipeline>(
		&mut self,

		texture: &Texture<P>,
		offset: [u16; 2],
		size: [u16; 2],
		data: &[[u8; 4]]
	) -> Result<(), RenderError> {
		texture.update(
			&mut self.encoder,

			offset,
			size,
			data
		)
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

    /// lista a renderização do chunk de terreno fornecido
    pub fn render_terrain_chunk(
        &mut self,
        
        model: &Model<terrain::TerrainPipeline>,
        globals: &Consts<Globals>,
        locals: &Consts<terrain::Locals>
    ) {
        self.encoder.draw(
            &model.slice,
            &self.terrain_pipeline.pso,

            &terrain::pipe::Data {
                vbuf: model.vbuf.clone(),
                locals: locals.buf.clone(),
                globals: globals.buf.clone(),

                tgt_color: self.tgt_color_view.clone(),
                tgt_depth: self.tgt_depth_view.clone()
            }
        );
    }

    /// lista a renderização do elemento ui fornecido para o frame a seguir
    pub fn render_ui_element(
        &mut self,

        model: &Model<ui::UiPipeline>,
        tex: &Texture<ui::UiPipeline>,

		scissor: Aabr<u16>
    ) {
		let Aabr { min, max } = scissor;
			
        self.encoder.draw(
            &model.slice,
            &self.ui_pipeline_pso,

            &ui::pipe::Data {
                vbuf: model.vbuf.clone(),

				scissor: gfx::Rect { x: min.y, y: min.y, w: max.x - min.x, h: max.y - min.y },
                tex: (tex.srv.clone(), tex.sampler.clone()),

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
