mod menu;
mod render_ctx;
mod window;

// padrão
use std::mem;

// caixote
use crate::{
    menu::title::TitleState,

    window::Window
};

// tipagem utilizada para armazenar o estado que é compartilhado entre os estados de play
pub struct GlobalState {
    window: Window
}

// estados podem fechar (e reverter para próximo estado), puxar um novo estado no top deles mesmo, ou alterar para um estado totalmente diferente
pub enum StateResult {
    Close,

    Push(Box<dyn PlayState>),
    Switch(Box<dyn PlayState>)
}

pub trait PlayState {
    fn play(&mut self, global_state: &mut GlobalState) -> StateResult;
}

fn main() {
    let mut states: Vec<Box<dyn PlayState>> = vec![Box::new(TitleState::new())];

    let mut global_state = GlobalState {
        window: Window::new(),
    };

    while let Some(state_result) = states.last_mut().map(|last| last.play(&mut global_state)) {
        // implementar lógica de transferência de estado

        match state_result {
            StateResult::Close => {
                states.pop();
            },
            
            StateResult::Push(new_state) => {
                states.push(new_state);
            },

            StateResult::Switch(mut new_state) => {
                states.last_mut().map(|old_state| mem::swap(old_state, &mut new_state));
            }
        }
    }
}