// todo: figurar aonde exatamente o código deve estar localizado

// biblioteca
use conrod_core::{
	Positionable,
	Sizeable,
	Labelable,

	Widget,
	widget_ids,
	
	image::Id as ImgId,
	text::font::Id as FontId,

	event::Input,

	widget::{
		Image,
		Button,

		Canvas
	}
};

// caixote
use crate::{
	window::Window,
	render::Renderer,
	ui::Ui
};

widget_ids!{
	struct Ids {
		bag,
		bag_contents,
		bag_close,

		menu_top,
		menu_mid,
		menu_bot,
		menu_canvas,
		menu_buttons[]
	}
}

// todo: fazer widget_ids! macro e mimic para ids de imagens ou encontrar outra solução para simplificar adição de novas imagens
struct Imgs {
	menu_button: ImgId,

	bag: ImgId,
	bag_hover: ImgId,
	bag_press: ImgId,
	bag_contents: ImgId,

	menu_top: ImgId,
	menu_mid: ImgId,
	menu_bot: ImgId,

	close_button: ImgId,
	close_button_hover: ImgId,
	close_button_press: ImgId
}

impl Imgs {
	fn new(ui: &mut Ui, renderer: &mut Renderer) -> Imgs {
		let mut load = |filename| {
			let image = image::open(&[env!("CARGO_MANIFEST_DIR"), "/test_assets/", filename].concat()).unwrap();

			ui.new_image(renderer, &image).unwrap()
		};

		Imgs {
			menu_button: load("test_menu_button_blank.png"),
			
            bag: load("test_bag.png"),
            bag_hover: load("test_bag_hover.png"),
            bag_press: load("test_bag_press.png"),
            bag_contents: load("test_bag_contents.png"),
			
            menu_top: load("test_menu_top.png"),
            menu_mid: load("test_menu_midsection.png"),
            menu_bot: load("test_menu_bottom.png"),
			
            close_button: load("test_close_btn.png"),
            close_button_hover: load("test_close_btn_hover.png"),
            close_button_press: load("test_close_btn_press.png")
		}
	}
}

pub struct TestHud {
	ui: Ui,
	ids: Ids,
	imgs: Imgs,
	
	bag_open: bool,
	menu_open: bool,
	
	font_id: FontId
}

impl TestHud {
	pub fn new(window: &mut Window) -> Self {
		let mut ui = Ui::new(window).unwrap();

		// gerar ids
		let mut ids = Ids::new(ui.id_generator());

		ids.menu_buttons.resize(5, &mut ui.id_generator());

		// carregar imagens
		let imgs = Imgs::new(&mut ui, window.renderer_mut());

		// carregar fontes
		let font_id = ui.new_font(conrod_core::text::font::from_file(
			concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/font/Metamorphous-Regular.ttf")
		).unwrap());

		Self {
			ui,
			imgs,
			ids,

			bag_open: false,
			menu_open: false,

			font_id
		}
	}

	fn update_layout(&mut self) {
		// muito útil
		let TestHud {
			ref mut ui,
			ref imgs,
			ref ids,

			ref mut bag_open,
			ref mut menu_open,

			..
		} = *self;

		let ref mut ui_cell = ui.set_widgets();
		
		// bolsa
		if Button::image(imgs.bag)
            .bottom_right_with_margin(20.0)
            .hover_image(if *bag_open { imgs.bag } else { imgs.bag_hover })
            .press_image(if *bag_open { imgs.bag } else { imgs.bag_press })
            .w_h(51.0, 58.0)
            .set(ids.bag, ui_cell)
			.was_clicked() {
				*bag_open = true;
			}

		// conteúdo de bolsa
		if *bag_open {
            // conteúdos
            Image::new(imgs.bag_contents)
                .w_h(212.0, 246.0)
                .x_y_relative_to(ids.bag, -92.0-25.5+12.0, 109.0+29.0-13.0)
                .set(ids.bag_contents, ui_cell);
			
            // fechar botão
            if Button::image(imgs.close_button)
                .w_h(20.0, 20.0)
                .hover_image(imgs.close_button_hover)
                .press_image(imgs.close_button_press)
                .top_right_with_margins_on(ids.bag_contents, 0.0, 10.0)
                .set(ids.bag_close, ui_cell)
				.was_clicked() {
					*bag_open = false;
				}
        }

		// fazer imagem baseado no container para botões
		// isso pode ser feito em um tipo de widget caso seja útil
		if *menu_open {
			let num = ids.menu_buttons.len();

			// canvas pode ser assegurado com tudo junto
			Canvas::new()
				.w_h(106.0, 54.0 + num as f64 * 30.0)
				.middle_of(ui_cell.window)
				.set(ids.menu_canvas, ui_cell);

			// topo do menu
			Image::new(imgs.menu_top)
				.w_h(106.0, 28.0)
				.mid_top_of(ids.menu_canvas)

				// não funciona por causa do bug no conrod, mas a linha acima é equivalente
				// .parent(ids.menu_canvas)
				// .mid_top()
				.set(ids.menu_top, ui_cell);

			// bottom do menu
			// nota: padrão de conrod para o último parente utilizado
			Image::new(imgs.menu_bot)
                .w_h(106.0, 26.0)
                .mid_bottom()
                .set(ids.menu_bot, ui_cell);

			// background de midsection
			Image::new(imgs.menu_mid)
                .w_h(106.0, num as f64 * 30.0)
                .mid_bottom_with_margin(26.0)
                .set(ids.menu_mid, ui_cell);

			// botões de menu
			if num > 0 {
                Button::image(imgs.menu_button)
                    .mid_top_with_margin_on(ids.menu_mid, 8.0)
                    .w_h(48.0, 20.0)
                    .label(&format!("Button {}", 1))
                    .label_rgb(1.0, 0.4, 1.0)
                    .label_font_size(7)
                    .set(ids.menu_buttons[0], ui_cell);
            }
			
            for i in 1..num {
                Button::image(imgs.menu_button)
                    .down(10.0)
                    .label(&format!("Button {}", i + 1))
                    .label_rgb(1.0, 0.4, 1.0)
                    .label_font_size(7)
                    .set(ids.menu_buttons[i], ui_cell);
            }
		}
	}

	pub fn toggle_menu(&mut self) {
        self.menu_open = !self.menu_open;
    }

	pub fn handle_event(&mut self, input: Input) {
        self.ui.handle_event(input);
    }

    pub fn maintain(&mut self, renderer: &mut Renderer) {
        self.update_layout();
		
        self.ui.maintain(renderer);
    }

    pub fn render(&self, renderer: &mut Renderer) {
        self.ui.render(renderer);
    }
}
