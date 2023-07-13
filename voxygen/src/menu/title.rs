// biblioteca
use vek::*;

// caixote
use crate::{
    PlayState,
    PlayStateResult,
    GlobalState,
    window::Event,
    render,
    session::SessionState
};

pub struct TitleState;

impl TitleState {
    /// cria um novo `titlestate`

    pub fn new() -> Self {
        Self
    }
}

// a cor do background
const BG_COLOR: Rgba<f32> = Rgba { r: 0.8, g: 1.0, b: 0.8, a: 1.0 };

impl PlayState for TitleState {
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult {
        loop {
            // ajuda os eventos da janela
            for event in global_state.window.fetch_events() {
                match event {
                    Event::Close => return PlayStateResult::Shutdown,

                    // quando o espaço é pressionado, iniciar uma sessão
                    Event::Char(' ') => return PlayStateResult::Push(
                        Box::new(SessionState::from_renderer(global_state.window.renderer_mut()))
                    ),

                    // ignorar todos os outros eventos
                    _ => {}
                }
            }

            // limpar a tela
            global_state.window
                .renderer_mut()
                .clear(BG_COLOR);

            // finalizar o frame
            global_state.window
                .renderer_mut()
                .flush();

            global_state.window
                .display()
                .expect("falha ao mostrar janela");
        }
    }

    fn name(&self) -> &'static str { "título" }
}