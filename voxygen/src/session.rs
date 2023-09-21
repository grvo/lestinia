// padrão
use std::time::Duration;

// biblioteca
use vek::*;

// projeto
use common::clock::Clock;

use client::{
    self,
    
    Client
};

// caixote
use crate::{
    Error,
    
    PlayState,
    PlayStateResult,

    GlobalState,

    window::{
        Event,

        Key
    },
    
    render::Renderer,
    scene::Scene
};

const FPS: u64 = 60;

pub struct SessionState {
    scene: Scene,
    client: Client
}

/// representa uma atividade de sessão de jogo
impl SessionState {
    /// cria um novo `sessionstate`
    pub fn new(renderer: &mut Renderer) -> Self {
        Self {
            // cria uma cena para esta sessão
            scene: Scene::new(renderer, &client),
            client
        }
    }
}

// cor do fundo
const BG_COLOR: Rgba<f32> = Rgba { r: 0.0, g: 0.3, b: 1.0, a: 1.0 };

impl SessionState {
    /// ticka a sessão (e o client anexado nela)
    pub fn tick(&mut self, dt: Duration) -> Result<(), Error> {
        self.client.tick(client::Input {}, dt)?;

        Ok(())
    }

    /// limpar a sessão depois de um tick
    pub fn cleanup(&mut self) {
        self.client.cleanup();
    }
    
    /// renderizar a sessão para a tela
    ///
    /// esse método deve ser chamado uma vez por frame
    pub fn render(&mut self, renderer: &mut Renderer) {
        // limpar a tela
        renderer.clear(BG_COLOR);

        // renderizar a tela utilizando renderizador global
        self.scene.render_to(renderer);

        // finalizar o frame
        renderer.flush();
    }
}

impl PlayState for SessionState {
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult {
        // capturar cursor
        global_state.window.grab_cursor(true);

        // configurar clock de fps
        let mut clock = Clock::new();

        // carregar novos chunks. todo: remover isso
        for x in -6..7 {
            for y in -6..7 {
                for z in -1..2 {
                    self.client.load_chunk(Vec3::new(x, y, z));
                }
            }
        }

        // loop de jogo
        loop {
            // auxiliar eventos de janela
            for event in global_state.window.fetch_events() {
                let _handled = match event {
                    Event::Close => return PlayStateResult::Shutdown,

                    // quando a janela for redimensionada, mudar o raio de aspecto da câmera
                    Event::Resize(dims) => {
                        self.scene.camera_mut().set_aspect_ratio(dims.x as f32 / dims.y as f32);
                    },

                    // quando 'q' for pressionado, deixar sessão
                    Event::Char('q') => return PlayStateResult::Pop,

                    // manter captura de cursor
                    Event::KeyDown(Key::ToggleCursor) => {
                        global_state.window.grab_cursor(!global_state.window.is_cursor_grabbed());
                    },

                    // passar todos os outros eventos para a cena
                    event => {
                        self.scene.handle_input_event(event);
                    }
                };

                // TODO: fazer algo se o evento não for auxiliado?
            }

            // performar um tick em jogo
            self.tick(clock.get_last_delta())
                .expect("falha ao tickar a cena");

            // mantém a cena
            self.scene.maintain(global_state.window.renderer_mut(), &self.client);

            // renderiza a sessão
            self.render(global_state.window.renderer_mut());

            // mostrar o frame na janela
            global_state.window
                .swap_buffers()
                .expect("falha ao trocar buffers de janela");

            // esperar para o próximo tick
            clock.tick(Duration::from_millis(1000 / FPS));

            // limpar coisas depois de um tick
            self.cleanup();
        }
    }

    fn name(&self) -> &'static str {
        "sessão"
    }
}
