// todo: armazenar todo o render da ui

// biblioteca
use image::DynamicImage;

use conrod_core::{
	Ui as CrUi,
	UiBuilder,
	UiCell,

	render::Primitive,
	event::Input,
	input::Widget,

	image::{
		Map,

		Id as ImgId
	},

	widget::{
		Id as WidgId,
		id::Generator
	},

	text::{
		Font,
		GlyphCache,

		font::Id as FontId
	}
};

// caixote
use crate::{
    Error,

    render::{
        RenderError,
        Renderer,

        Model,
		Mesh,
        Texture,
		
        UiPipeline,
		UiMode,

		push_ui_quad_to_mesh
    },

	window::Window
};

#[derive(Debug)]
pub enum UiError {
    RenderError(RenderError)
}

pub struct Cache {
    blank_texture: Texture<UiPipeline>,
	
	glyph_cache: GlyphCache<'static>,
	glyph_cache_tex: Texture<UiPipeline>
}

// todo: as funções deveriam estar retornando uierror em vez de error?
impl Cache {
    pub fn new(renderer: &mut Renderer) -> Result<Self, Error> {
		// todo: remover mapa caso não tenha sido enviado (ou remover esse comentário)
		let (w, h) = renderer.get_resolution().map(|e| e).into_tuple();

		const SCALE_TOLERANCE: f32 = 0.1;
		const POSITION_TOLERANCE: f32 = 0.1;
		
        Ok(Self {
            blank_texture: renderer.create_texture(&DynamicImage::new_rgba8(1, 1))?,

			glyph_cache: GlyphCache::builder()
				.dimensions(w as u32, h as u32)
				.scale_tolerance(SCALE_TOLERANCE)
				.position_tolerance(POSITION_TOLERANCE)
				.build(),

			glyph_cache_tex: renderer.create_dynamic_texture((w, h).into())?
        })
    }

    pub fn blank_texture(&self) -> &Texture<UiPipeline> {
        &self.blank_texture
    }

	pub fn glyph_cache_tex(&self) -> &Texture<UiPipeline> {
		&self.glyph_cache_tex
	}
	
	pub fn glyph_cache_mut_and_tex(&mut self) -> (&mut GlyphCache<'static>, &Texture<UiPipeline>) {
		(&mut self.glyph_cache, &self.glyph_cache_tex)
	}
}

pub enum DrawCommand {
	Image(Model<UiPipeline>, ImgId),

	// texto e geometria não-texturada
	Plain(Model<UiPipeline>)
}

pub struct Ui {
	ui: CrUi,
	image_map: Map<Texture<UiPipeline>>,
    cache: Cache,

	// primitivos para desenhar no próximo render
	draw_commands: Vec<DrawCommand>
}

impl Ui {
    pub fn new(window: &mut Window) -> Result<Self, Error> {
		// recuperar o tamanho lógico do conteúdo da janela
		let (w, h) = window.logical_size();
		
        Ok(Self {
			ui: UiBuilder::new([w, h]).build(),
			image_map: Map::new(),
            cache: Cache::new(window.renderer_mut())?,
			draw_commands: vec![]
        })
    }

	pub fn new_image(&mut self, renderer: &mut Renderer, image: &DynamicImage) -> Result<ImgId, Error> {
		Ok(self.image_map.insert(renderer.create_texture(image)?))
	}

	pub fn new_font(&mut self, font: Font) -> FontId {
		self.ui.fonts.insert(font)
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

	pub fn maintain(&mut self, renderer: &mut Renderer) {
		let ref mut ui = self.ui;

		// reunindo primitivos e recrie locais somente se ui_changed
		if let Some(mut primitives) = ui.draw_if_changed() {
			self.draw_commands.clear();

			let mut mesh = Mesh::new();
			let mut current_img = None;

			// altera para o estado `plain` e completa o `command` anterior caso ainda não tenha estado `plain`
			macro_rules! switch_to_plain_state {
				() => {
					if let Some(image_id) = current_img.take() {
						self.draw_commands.push(DrawCommand::Image(renderer.create_model(&mesh).unwrap(), image_id));

						mesh.clear();
					}
				};
			}

			while let Some(prim) = primitives.next() {
				// todo: utilizar scizzor
				let Primitive {kind, scizzor, id, rect} = prim;
				
				// transformar do conrod para nossas coordenadas dos renders
				//
				// conrod utiliza o centro da tela como origem
				// up e right são posições positivas
				/* let x = rect.left();
				let y = rect.top();

				let (w, h) = rect.w_h();

				let bounds = [
					(x / ui.win_w + 0.5) as f32,
                    (-1.0 * (y / ui.win_h) + 0.5) as f32,
                    (w / ui.win_w) as f32,
                    (h / ui.win_h) as f32
				]; */

				use conrod_core::render::PrimitiveKind;

				match kind {
					// todo: utilizar source_rect
					
					PrimitiveKind::Image { image_id, color, source_rect } => {
						// alterar para o estado `image` para essa imagem caso ainda não foi feito
						let new_image_id = image_id;

						match current_img {
							// caso já tenha esteja no modo de desenho para essa imagem, está feito
							Some(image_id) if image_id == new_image_id => (),

							// caso estejamos no estado de desenho `plain`, alterar para o estado de desenho de image
							None => {
								self.draw_commands.push(DrawCommand::Plain(renderer.create_model(&mesh).unwrap()));

								mesh.clear();

								current_img = Some(new_image_id);
							}

							// caso esteja sendo desenhada uma imagem diferente, alterar para *essa* imagem
							Some(image_id) => {
								self.draw_commands.push(DrawCommand::Image(renderer.create_model(&mesh).unwrap(), image_id));

								mesh.clear();

								current_img = Some(new_image_id);
							}
						}

						let color = color.unwrap_or(conrod_core::color::WHITE).to_fsa();

						// let (image_w, image_h) = image_map.get(&image_id).unwrap().1;
						// let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);
						//
						// obtém os sides do retângulo da fonte como coordenadas uv
						//
						// alcance de coordenadas de textura:
						// - esquerda para direita: 0.0 para 1.0
						// - bottom para o topo: 1.0 para 0.0
						//
						// notar que o bottom e o topo são flippados em comparação ao glium
						// então não adicionar para imagens flip enquanto carregadas
						
						/* let (uv_l, uv_r, uv_t, uv_b) = match source_rect {
                            Some(src_rect) => {
                                let (l, r, b, t) = src_rect.l_r_b_t();
								
                                ((l / image_w) as f32,
                                (r / image_w) as f32,
                                (b / image_h) as f32,
                                (t / image_h) as f32)
                            }
							
                            None => (0.0, 1.0, 0.0, 1.0)
                        }; */

						let (uv_l, uv_r, uv_t, uv_b) = (0.0, 1.0, 0.0, 1.0);
						let (l, r, b, t) = rect.l_r_b_t();

						// converter do alcance scalar do conrod para o alcance do gl -1.0 para 1.0
						let (l, r, b, t) = (
							(l / ui.win_w * 2.0) as f32,
                            (r / ui.win_w * 2.0) as f32,
                            (b / ui.win_h * 2.0) as f32,
                            (t / ui.win_h * 2.0) as f32
						);

						push_ui_quad_to_mesh(
							&mut mesh,

							[l, t , r, b],
                            [uv_l, uv_t, uv_r, uv_b],
							
                            color,
                            UiMode::Image
						);
                    }

					PrimitiveKind::Text { color, text, font_id } => {
                        switch_to_plain_state!();
						
                        // obtém largura da janela
                        let (screen_w, screen_h) = renderer.get_resolution().map(|e| e as f32).into_tuple();
						
                        // calcula o fator do dpi
                        let dpi_factor = screen_w / ui.win_w as f32;

                        let positioned_glyphs = text.positioned_glyphs(dpi_factor);
                        let (glyph_cache, cache_tex) = self.cache.glyph_cache_mut_and_tex();
						
                        // lista os glifos a serem armazenados
                        for glyph in positioned_glyphs {
                            glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                        }

                        glyph_cache.cache_queued(|rect, data| {
                            let offset = [rect.min.x as u16, rect.min.y as u16];
                            let size = [rect.width() as u16, rect.height() as u16];

                            let new_data = data.iter().map(|x| [255, 255, 255, *x]).collect::<Vec<[u8; 4]>>();

                            renderer.update_texture(cache_tex, offset, size, &new_data);
                        }).unwrap();

                        // todo: considerar o gama...
                        let color = color.to_fsa();

                        for g in positioned_glyphs {
                            if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(font_id.index(), g) {
                                let (uv_l, uv_r, uv_t, uv_b) = (
                                    uv_rect.min.x,
                                    uv_rect.max.x,
                                    uv_rect.min.y,
                                    uv_rect.max.y
                                );
								
                                let (l, t, r, b) = (
                                    (screen_rect.min.x as f32 / screen_w - 0.5) *  2.0,
                                    (screen_rect.min.y as f32 / screen_h - 0.5) * -2.0,
                                    (screen_rect.max.x as f32 / screen_w - 0.5) *  2.0,
                                    (screen_rect.max.y as f32 / screen_h - 0.5) * -2.0
                                );
								
                                push_ui_quad_to_mesh(
                                    &mut mesh,
									
                                    [l, t , r, b],
                                    [uv_l, uv_t, uv_r, uv_b],
									
                                    color,
                                    UiMode::Text
                                );
                            }
                        }
					}

					_ => {}

					// todo: adicionar esses
					// PrimitiveKind::Other {..} => {println!("primitive kind other with id {:?}", id);}
                    // PrimitiveKind::Rectangle { color } => {println!("primitive kind rect[x:{},y:{},w:{},h:{}] with color {:?} and id {:?}", x, y, w, h, color, id);}
                    // PrimitiveKind::TrianglesMultiColor {..} => {println!("primitive kind multicolor with id {:?}", id);}
                    // PrimitiveKind::TrianglesSingleColor {..} => {println!("primitive kind singlecolor with id {:?}", id);}
				}
			}

			// entrar para o comando final
			match current_img {
                None => self.draw_commands.push(DrawCommand::Plain(renderer.create_model(&mesh).unwrap()))
                Some(image_id) => self.draw_commands.push(DrawCommand::Image(renderer.create_model(&mesh).unwrap(), image_id))
            }
		}
	}

    pub fn render(&self, renderer: &mut Renderer) {
		for draw_command in self.draw_commands.iter() {
            match draw_command {
                DrawCommand::Image(model, image_id) => {
                    let tex = self.image_map.get(&image_id).expect("a imagem não existe no mapa de imagens");
					
                    renderer.render_ui_element(&model, &tex);
                },
				
                DrawCommand::Plain(model) => {
                    let tex = self.cache.glyph_cache_tex();
					
                    renderer.render_ui_element(&model, &tex);
                }
            }
        }
	}
}
