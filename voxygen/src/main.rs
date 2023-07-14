pub mod error;
pub mod menu;
pub mod render;
pub mod scene;
pub mod session;
pub mod window;

// re-exportações
pub use crate::error::Error;

// padrão
use std::mem;

// biblioteca
use log;
use pretty_env_logger;

// caixote
use crate::{
    menu::title::TitleState,
    window::Window
};

/// tipagem utilizada para armazenar o estado que é compartilhado entre os estados de play
pub struct GlobalState {
    window: Window
}

impl GlobalState {
    /// chamado depois de alguma mudança no estado de jogar que ocorreu
    pub fn on_play_state_changed(&mut self) {
        self.window.untrap_cursor();
    }
}

// estados podem fechar (e reverter para próximo estado), puxar um novo estado no top deles mesmo, ou alterar para um estado totalmente diferente
pub enum PlayStateResult {
    /// abre todos os estados de jogo na ordem inversa e desliga o programa
    Shutdown,

    /// fecha o estado play atual e poppa ele para o estado de play guardado
    Pop,

    /// empurra um novo estado de reprodução para a pilha de estado de reprodução
    Push(Box<dyn PlayState>),

    /// alterna o estado de jogo atual com um novo estado de jogo
    Switch(Box<dyn PlayState>)
}

/// característica que representa um estado de jogo jogável. pode ser um menu, uma sessão de jogo, o título, etc.
pub trait PlayState {
    /// joga o estado até que alguma mudança de estado seja necessária
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult;

    /// obtém um nome descritivo para este tipo de estado
    fn name(&self) -> &'static str;
}

fn main() {
    // logging inicial
    pretty_env_logger::init();

    // configura o estado de play inicial
    let mut states: Vec<Box<dyn PlayState>> = vec![Box::new(TitleState::new())];

    states.last().map(|current_state| {
        log::info!("jogo iniciado com o estado '{}'", current_state.name())
    });

    // configura o estado global
    let mut global_state = GlobalState {
        window: Window::new()
            .expect("falha ao criar janela")
    };

    while let Some(state_result) = states.last_mut().map(|last| last.play(&mut global_state)) {
        // implementar lógica de transferência de estado

        match state_result {
            PlayStateResult::Shutdown => {
                log::info!("desligando todos os estados...");

                while states.last().is_some() {
                    states.pop().map(|old_state| {
                        log::info!("estado poppado '{}'", old_state.name());

                        global_state.on_play_state_changed();
                    });
                }
            },

            PlayStateResult::Pop => {
                states.pop().map(|old_state| {
                    log::info!("estado poppado '{}'", old_state.name());

                    global_state.on_play_state_changed();
                });
            },
            
            PlayStateResult::Push(new_state) => {
                log::info!("estado puxado '{}'", new_state.name());

                states.push(new_state);

                global_state.on_play_state_changed();
            },

            PlayStateResult::Switch(mut new_state) => {
                states.last_mut().map(|old_state| {
                    log::info!("alternando para o estado '{}' do estado '{}'", new_state.name(), old_state.name());

                    mem::swap(old_state, &mut new_state);

                    global_state.on_play_state_changed();
                });
            }
        }
    }
}