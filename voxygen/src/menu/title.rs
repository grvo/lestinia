// biblioteca
use vek::*;
use image;

// caixote
use crate::{
    PlayState,
    PlayStateResult,
    
    GlobalState,

	window::{
		Event,
		Window
	},
	
	session::SessionState
};

// local
use super::title_ui::TitleUi;

pub struct TitleState {
    title_ui: TitleUi
}

impl TitleState {
    /// cria um novo `titlestate`
	
    pub fn new(window: &mut window) -> Self {
		Self {
			title_ui: TitleUi::new(window)
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
                        Box::new(SessionState::new(&mut global_state.window).unwrap()) // todo: auxiliar esse erro
                    ),

					// passar eventos para ui
					Event::UiEvent(input) => {
						self.title_ui.handle_event(input);
					}

                    // ignorar todos os outros eventos
                    _ => {}
                }
            }

			global_state.window.renderer_mut().clear(BG_COLOR);

            // mantém a ui
            self.title_ui.maintain(global_state.window.renderer_mut());

            // desenha a ui na tela
			self.title_ui.render(global_state.window.renderer_mut());

			// finalizar o frame
			global_state.window.renderer_mut().flush();

			global_state.window
				.swap_buffers()
				.expect("falha ao trocar buffers da janela");
        }
    }

    fn name(&self) -> &'static str { "título" }
}
