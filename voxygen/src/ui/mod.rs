// pub mod element;
// pub mod size_request;
// pub mod span;

// re-exportações
/* pub use self::{
    span::Span,

    size_request::SizeRequest
}; */

// todo: qual era o propósito do request de tamanho?
// todo: armazenar todo o render da ui
// todo: será preciso armazenar locals para cada widget?
// todo: tamanhos? : renderer.get_resolution().map(|e| e as f32)

// biblioteca
use image::DynamicImage;

use conrod_core::{
	Ui as CrUi,
	UiBuilder,
	UiCell,

	image::{
		Map,

		Id as ImgId
	},

	widget::Id as WidgId
};

// caixote
use crate::{
    Error,

    render::{
        RenderError,
        Renderer,

        Model,
        Texture,
		
        UiPipeline,
		UiLocals,

		Consts,

        create_ui_quad_mesh
    }
};

// local
/* use self::element::{
    Element,

    Bounds
}; */

#[derive(Debug)]
pub enum UiError {
    RenderError(RenderError)
}

pub struct Cache {
    model: Model<UiPipeline>,
    blank_texture: Texture<UiPipeline>
}

impl Cache {
    pub fn new(renderer: &mut Renderer) -> Result<Self, Error> {
        Ok(Self {
            model: renderer.create_model(&create_ui_quad_mesh())?,
            blank_texture: renderer.create_texture(&DynamicImage::new_rgba8(1, 1))?
        })
    }

    pub fn model(&self) -> &Model<UiPipeline> {
        &self.model
    }

    pub fn blank_texture(&self) -> &Texture<UiPipeline> {
        &self.blank_texture
    }
}

pub struct Ui {
	ui: CrUi,
	image_map: Map<Texture<UiPipeline>>,
    cache: Cache,
	locals: Consts<UiLocals>
}

impl Ui {
    pub fn new(renderer: &mut Renderer, dim: [f64; 2]) -> Result<Self, Error> {
        Ok(Self {
			ui: UiBuilder::new(dim).build(),
			image_map: Map::new(),
            cache: Cache::new(renderer)?,
			locals: renderer.create_consts(&[UiLocals::default()])?
        })
    }

	pub fn new_image(&mut self, renderer: &mut Renderer, image: &DynamicImage) -> Result<ImgId, Error> {
		Ok(self.image_map.insert(renderer.create_texture(image)?))
	}

	pub fn new_widget(&mut self) -> WidgId {
		self.ui.widget_id_generator().next()
	}

	pub fn set_widgets<F>(&mut self, f: F) where F: FnOnce(&mut UiCell) {
		f(&mut self.ui.set_widgets());
	}

	// todo: mudar o render para &self e utilizar maintain para operações mutáveis
    pub fn maintain(&mut self, renderer: &mut Renderer) {
        // renderer.get_resolution().map(|e| e as f32)
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        if let Some(mut primitives) = Some(self.ui.draw()) {
			// renderizar os primitivos de uma vez
			while let Some(prim) = primitives.next() {
				use conrod_core::render::{
					Primitive,
					PrimitiveKind
				};

				let Primitive {
					kind,
					scizzor,
					rect,

					..
				} = prim;

				match kind {
					PrimitiveKind::Image {
						image_id,
						color,
						source_rect
					} => {
						renderer.update_consts(&mut self.locals, &[UiLocals::new(
                            [0.0, 0.0, 1.0, 1.0]
                        )]);

						let tex = self.image_map.get(&image_id).expect("imagem não existe no mapa de imagens");

						renderer.render_ui_element(&self.cache.model(), &self.locals, &tex);
					}

					PrimitiveKind::Other {..} => {println!("tipo primitivo outro com id");}
                    PrimitiveKind::Rectangle {..} => {println!("tipo primitivo de rect");}
                    PrimitiveKind::Text {..} => {println!("tipo primitivo de texto");}
                    PrimitiveKind::TrianglesMultiColor {..} => {println!("tipo primitivo multicolorido");}
                    PrimitiveKind::TrianglesSingleColor {..} => {println!("tipo primitivo de cor único");}
				}
			}
		}
    }
}
