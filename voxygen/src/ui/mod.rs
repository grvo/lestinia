pub mod title;
pub mod test;

// todo: armazenar todo o render da ui

// biblioteca
use image::DynamicImage;

use conrod_core::{
	Ui as CrUi,
	UiBuilder,
	UiCell,

	render::Primitive,
	event::Input,

	image::{
		Map,

		Id as ImgId
	},

	widget::{
		Id as WidgId,
		id::Generator
	},

	input::{
		Global,
		Widget
	}
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
    },

	window::Window
};

#[derive(Debug)]
pub enum UiError {
    RenderError(RenderError)
}

pub struct Cache {
    model: Model<UiPipeline>,
    blank_texture: Texture<UiPipeline>
}

// todo: as funções deveriam estar retornando uierror em vez de error?
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

pub enum UiPrimitive {
	Image(Consts<UiLocals>, ImgId)
}

pub struct Ui {
	ui: CrUi,
	image_map: Map<Texture<UiPipeline>>,
    cache: Cache,

	// primitivos para desenhar no próximo render
	ui_primitives: Vec<UiPrimitive>
}

impl Ui {
    pub fn new(window: &mut Window) -> Result<Self, Error> {
		// recuperar o tamanho lógico do conteúdo da janela
		let (w, h) = window.logical_size();
		
        Ok(Self {
			ui: UiBuilder::new([w, h]).build(),
			image_map: Map::new(),
            cache: Cache::new(window.renderer_mut())?,
			ui_primitives: vec![]
        })
    }

	pub fn new_image(&mut self, renderer: &mut Renderer, image: &DynamicImage) -> Result<ImgId, Error> {
		Ok(self.image_map.insert(renderer.create_texture(image)?))
	}

	pub fn id_generator(&mut self) -> Generator {
		self.ui.widget_id_generator()
	}

	pub fn set_widgets(&mut self) -> UiCell {
        self.ui.set_widgets()
    }

	pub fn handle_event(&mut self, event: Input) {
        self.ui.handle_event(event);
    }

    pub fn widget_input(&self, id: WidgId) -> Widget {
        self.ui.widget_input(id)
    }

    pub fn global_input(&self) -> &Global {
        self.ui.global_input()
    }

	pub fn maintain(&mut self, renderer: &mut Renderer) {
		let ref mut ui = self.ui;

		// isso foi removido porque a ui precisa redimensionar sozinha para receber os eventos
		// de atualização de tamanhod a janela
		//
		// let res = renderer.get_resolution().map(|e| e as f64);
		//
		// if res[0] != ui.win_w || res[1] != ui.win_h {
		//     ui.win_w = res[0];
		//     ui.win_h = res[1];
		//
		//     ui.needs_redraw();
		// }

		// reunindo primitivos e recrie locais somente se ui_changed
		if let Some(mut primitives) = ui.draw_if_changed() {
			self.ui_primitives.clear();

			while let Some(prim) = primitives.next() {
				// transformar do conrod para nossas coordenadas dos renders
				//
				// conrod utiliza o centro da tela como origem
				// up e right são posições positivas
				let x = prim.rect.left();
				let y = prim.rect.top();

				let (w, h) = prim.rect.w_h();

				let bounds = [
					(x / ui.win_w + 0.5) as f32,
                    (-1.0 * (y / ui.win_h) + 0.5) as f32,
                    (w / ui.win_w) as f32,
                    (h / ui.win_h) as f32
				];

				// todo: remover isso
				let new_ui_locals = renderer.create_consts(&[UiLocals::new(bounds)])
					.expect("não foi possível criar uma nova const para ui locais");

				use conrod_core::render::{
					PrimitiveKind
				};

				// todo: utilizar scizzor
				let Primitive {
					kind,
					scizzor,
					id,
					
					..
				} = prim;

				match kind {
					PrimitiveKind::Image { image_id, color, source_rect } => {
                        // renderer.update_consts(&mut self.locals, &[UiLocals::new(
                        //     [0.0, 0.0, 1.0, 1.0]
                        // )]);
						
                        self.ui_primitives.push(UiPrimitive::Image(new_ui_locals, image_id));
                    }

					_ => {}

					// todo: adicionar esses
					// PrimitiveKind::Other {..} => {println!("primitive kind other with id {:?}", id);}
                    // PrimitiveKind::Rectangle { color } => {println!("primitive kind rect[x:{},y:{},w:{},h:{}] with color {:?} and id {:?}", x, y, w, h, color, id);}
                    // PrimitiveKind::Text {..} => {println!("primitive kind text with id {:?}", id);}
                    // PrimitiveKind::TrianglesMultiColor {..} => {println!("primitive kind multicolor with id {:?}", id);}
                    // PrimitiveKind::TrianglesSingleColor {..} => {println!("primitive kind singlecolor with id {:?}", id);}
				}
			}
		}
	}

    pub fn render(&self, renderer: &mut Renderer) {
		self.ui_primitives.iter().for_each(|ui_primitive| match ui_primitive {
            UiPrimitive::Image(ui_locals, image_id) => {
                let tex = self.image_map.get(&image_id).expect("a imagem não existe no mapa de imagens");
				
                renderer.render_ui_element(&self.cache.model(), &ui_locals, &tex);
            }
        });
	}
}
