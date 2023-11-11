// biblioteca
use vek::*;
use image;

// conrod
use conrod_core::widget::image::Image as ImageWidget;

use conrod_core::{
	Positionable,
	Sizeable,

	Widget
};

// caixote
use crate::{
    PlayState,
    PlayStateResult,
    
    GlobalState,
    window::Event,
    session::SessionState,

    render::{
		Consts,
		UiLocals,
		Renderer
	},

	ui::{
		Ui
	}
};

pub struct TitleState {
    ui: Ui
}

impl TitleState {
    /// cria um novo `titlestate`
	
    pub fn new(renderer: &mut Renderer) -> Self {
        let mut ui = Ui::new(renderer, [500.0, 500.0]).unwrap();
		let widget_id = ui.new_widget();
		let image = image::open(concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/test.png")).unwrap();
		let title_img = ui.new_image(renderer, &image).unwrap();

		ui.set_widgets(|ui_cell| {
			ImageWidget::new(title_img)
				.x_y(0.0, 0.0)
				.w_h(500.0, 500.0)
				.set(widget_id, ui_cell);
		});

        Self {
            ui
        }
    }
}

// a cor do background
const BG_COLOR: Rgba<f32> = Rgba { r: 0.0, g: 0.3, b: 1.0, a: 1.0 };

impl PlayState for TitleState {
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult {
        loop {
            // ajuda os eventos da janela
            for event in global_state.window.fetch_events() {
                match event {
                    Event::Close => return PlayStateResult::Shutdown,

                    // quando o espaço é pressionado, iniciar uma sessão
                    Event::Char(' ') => return PlayStateResult::Push(
                        Box::new(SessionState::new(global_state.window.renderer_mut()).unwrap()) // todo: auxiliar esse erro
                    ),

                    // ignorar todos os outros eventos
                    _ => {}
                }
            }

			global_state.window.renderer_mut().clear(BG_COLOR);

            // mantém a ui
            // self.ui.maintain(global_state.window.renderer_mut());

            // desenha a ui na tela
			self.ui.render(global_state.window.renderer_mut());

			// finalizar o frame
			global_state.window.renderer_mut().flush();

			global_state.window
				.swap_buffers()
				.expect("falha ao trocar buffers da janela");
        }
    }

    fn name(&self) -> &'static str { "título" }
}
