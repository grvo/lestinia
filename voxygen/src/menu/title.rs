// biblioteca
use vek::*;
use image;

// caixote
use crate::{
    PlayState,
    PlayStateResult,
    
    GlobalState,
    window::Event,
    session::SessionState,

    render::Renderer,

    ui::{
        Ui,

        element::{
            Widget,

            image::Image
        }
    }
};

pub struct TitleState {
    ui: Ui
}

impl TitleState {
    /// cria um novo `titlestate`

    pub fn new(renderer: &mut Renderer) -> Self {
        let img = Image::new(renderer, &image::open(concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/test.png")).unwrap()).unwrap();
        let widget = Widget::new(renderer, img).unwrap();

        Self {
            ui: Ui::new(renderer, widget).unwrap()
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

            // limpar a tela
            global_state.window
                .renderer_mut()
                .clear(BG_COLOR);

            // mantém a ui
            self.ui.maintain(global_state.window.renderer_mut());

            // desenha a ui na tela
            self.ui.render(global_state.window.renderer_mut());

            // finalizar o frame
            global_state.window
                .renderer_mut()
                .flush();

            global_state.window
                .display()
                .expect("falha ao mostrar buffers de janela");
        }
    }

    fn name(&self) -> &'static str { "título" }
}
