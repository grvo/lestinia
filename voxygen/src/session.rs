// biblioteca
use vek::*;

// caixote
use crate::{
    PlayState,
    PlayStateResult,

    GlobalState,

    window::Event,
    render::Renderer,
    scene::Scene
};

pub struct SessionState {
    scene: Scene
}

/// representa uma atividade de sessão de jogo
impl SessionState {
    /// cria um novo `sessionstate`
    pub fn from_renderer(renderer: &mut Renderer) -> Self {
        Self {
            // cria uma cena para esta sessão
            scene: Scene::new(renderer)
        }
    }
}

// cor do fundo
const BG_COLOR: Rgba<f32> = Rgba { r: 0.0, g: 0.3, b: 1.0, a: 1.0 };

impl PlayState for SessionState {
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult {
        // loop de jogo
        loop {
            // auxiliar eventos de janela
            for event in global_state.window.fetch_events() {
                match event {
                    Event::Close => return PlayStateResult::Shutdown,

                    // quando 'q' for pressionado, deixar sessão
                    Event::Char('q') => return PlayStateResult::Pop,

                    // ignorar todos os outros eventos
                    _ => {}
                }
            }

            // limpar a tela
            global_state.window
                .renderer_mut()
                .clear(BG_COLOR);

            // renderizar a tela utilizando renderizador global
            self.scene
                .render_to(global_state.window.renderer_mut());

            // finalizar frame
            global_state.window
                .renderer_mut()
                .flush();

            global_state.window
                .display()
                .expect("falha ao mostrar a janela");
        }
    }

    fn name(&self) -> &'static str { "sessão" }
}