// padrão
use std::time::Duration;

// interno
use common::state::State;

pub enum ClientErr {
    ServerShutdown,

    Other(String)
}

pub struct Input {
    // todo: utilizar esta tipagem para gerenciar input do servidor
}

pub struct Server {
    state: State,

    // todo: adicionar estado `meta` aqui
}

impl Server {
    pub fn new() -> Self {
        Self {
            state: State::new()
        }
    }

    /// executar tick de cliente único, ajudar input e atualizar estado do jogo pela duração recebida
    pub fn tick(&mut self, input: Input, dt: Duration) -> Result<(), ClientErr> {
        // a função tick é o centro do universo lestinia
        // a maior parte das coisas pela parte do client são gerenciadas aqui
        //
        // deixar sempre a função organizada
        //
        // 1) coletar input do frontend, aplicar efeitos do input para o estado do jogo
        // 2) ir dentro de qualquer evento que precise de ajuda e aplicar eles no estado do jogo
        // 3) ir dentro de todas as comunicações recebidas da rede do client
        // 4) performar tick localstate único (ex: atualizar o mundo e suas entidades)
        // 5) ir dentro da atualização de terreno e aplicar todas as mudanças para o terreno
        // 6) enviar atualizações de estado relevantes para todos os clients
        // 7) finalizar o tick, passando controle para a thread principal e voltar para o frontend

        // tick para o localstate do client (passo 3)
        self.state.tick(dt);

        // finalizar o tick, passar controle de volta para o frontend (passo 6)
        Ok(())
    }
}