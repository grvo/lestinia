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
    key_state::KeyState,

    window::{
        Event,

        Key,

		Window
    },
    
    render::Renderer,
    scene::Scene,

	menu::test_hud::TestHud
};

const FPS: u64 = 60;

pub struct SessionState {
    scene: Scene,
    client: Client,
    key_state: KeyState,

	// todo: remover isso
	test_hud: TestHud
}

/// representa uma atividade de sessão de jogo
impl SessionState {
    /// cria um novo `sessionstate`
    pub fn new(window: &mut Window) -> Result<Self, Error> {
        let client = Client::new(([127, 0, 0, 1], 59003))?.with_test_state(); // <--- todo: remover isso
        
        Ok(Self {
            // cria uma cena para esta sessão
            scene: Scene::new(window.renderer_mut(), &client),
            client,
            key_state: KeyState::new(),
			test_hud: TestHud::new(window)
        })
    }
}

// cor do fundo
const BG_COLOR: Rgba<f32> = Rgba { r: 0.0, g: 0.3, b: 1.0, a: 1.0 };

impl SessionState {
    /// ticka a sessão (e o client anexado nela)
    pub fn tick(&mut self, dt: Duration) -> Result<(), Error> {
        // calcular o vetor de input de movimento do jogador por meio da key atual pressionada e direção da câmera
        let ori = self.scene.camera().get_orientation();

        let unit_vecs = (
            Vec2::new(ori[0].cos(), -ori[0].sin()),
            Vec2::new(ori[1].sin(), ori[0].cos())
        );

        let dir_vec = self.key_state.dir_vec();
        let move_dir = unit_vecs.0 * dir_vec[0] + unit_vecs.1 * dir_vec[1];

        self.client.tick(client::Input { move_dir }, dt)?;

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

		// desenhar a ui para a tela
		self.test_hud.render(renderer);

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

                    // quando 'q' for pressionado, deixar sessão
                    Event::Char('q') => return PlayStateResult::Pop,

					// quando 'm' for pressionado, abrir/fechar o menu de teste
                    Event::Char('m') => self.test_hud.toggle_menu(),

                    // manter captura de cursor
                    Event::KeyDown(Key::ToggleCursor) => {
                        global_state.window.grab_cursor(!global_state.window.is_cursor_grabbed());
                    },

                    // key de movimento pressionada
                    Event::KeyDown(Key::MoveForward) => self.key_state.up = true,
                    Event::KeyDown(Key::MoveBack) => self.key_state.down = true,
                    Event::KeyDown(Key::MoveLeft) => self.key_state.left = true,
                    Event::KeyDown(Key::MoveRight) => self.key_state.right = true,
                    
                    // key de movimento lançada
                    Event::KeyUp(Key::MoveForward) => self.key_state.up = false,
                    Event::KeyUp(Key::MoveBack) => self.key_state.down = false,
                    Event::KeyUp(Key::MoveLeft) => self.key_state.left = false,
                    Event::KeyUp(Key::MoveRight) => self.key_state.right = false,

					// passar eventos para ui
					Event::UiEvent(input) => {
						self.test_hud.handle_event(input);
					}

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

			// mantém a ui
			self.test_hud.maintain(global_state.window.renderer_mut());

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
