mod menu;
mod render;
mod window;

// padrão
use std::mem;

// biblioteca
use winit;
use failure;

// caixote
use crate::{
    menu::title::TitleState,

    window::Window
};

#[derive(Debug)]
pub enum VoxygenErr {
    WinitCreationErr(winit::CreationErr),

    Other(failure::Error)
}

// tipagem utilizada para armazenar o estado que é compartilhado entre os estados de play
pub struct GlobalState {
    window: Window
}

// estados podem fechar (e reverter para próximo estado), puxar um novo estado no top deles mesmo, ou alterar para um estado totalmente diferente
pub enum PlayStateResult {
    /// abre todos os estados de jogo na ordem inversa e desliga o programa
    Shutdown,

    /// fecha o estado play atual
    Close,

    /// empurra um novo estado de reprodução para a pilha de estado de reprodução
    Push(Box<dyn PlayState>),

    /// alterna o estado de jogo atual com um novo estado de jogo
    Switch(Box<dyn PlayState>)
}

pub trait PlayState {
    fn play(&mut self, global_state: &mut GlobalState) -> PlayStateResult;
}

fn main() {
    let mut states: Vec<Box<dyn PlayState>> = vec![Box::new(TitleState::new())];

    let mut global_state = GlobalState {
        window: Window::new()
            .expect("falha ao criar janela")
    };

    while let Some(state_result) = states.last_mut().map(|last| last.play(&mut global_state)) {
        // implementar lógica de transferência de estado

        match state_result {
            PlayStateResult::Shutdown => while states.last().is_some() {
                states.pop();
            },

            PlayStateResult::Close => {
                states.pop();
            },
            
            PlayStateResult::Push(new_state) => {
                states.push(new_state);
            },

            PlayStateResult::Switch(mut new_state) => {
                states.last_mut().map(|old_state| mem::swap(old_state, &mut new_state));
            }
        }
    }
}