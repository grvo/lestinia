// padrão
use std::time::Duration;

// biblioteca
use specs::Entity as EcsEntity;
use vek::*;

// projeto
use common::{
    state::State,
    terrain::TerrainChunk
};

use world::World;

#[derive(Debug)]
pub enum Error {
    ServerShutdown,

    Other(String)
}

pub struct Input {
    // todo: usar essa tipagem para gerenciar input do client
}

pub struct Client {
    state: State,

    player: Option<EcsEntity>,

    // teste
    world: World,
    
    pub chunk: Option<TerrainChunk>
}

impl Client {
    /// cria um novo `client`
    pub fn new() -> Self {
        Self {
            state: State::new(),

            player: None,

            // teste
            world: World::new(),
            chunk: None
        }
    }

    /// TODO: obter rid disso
    pub fn with_test_state(mut self) -> Self {
        self.chunk = Some(self.world.generate_chunk(Vec3::zero()));

        self
    }

    /// obtém a referência para o estado do jogo do cliente
    pub fn state(&self) -> &state { &self.state }

    /// obtém a referência mutável para estado do jogo do cliente
    pub fn state_mut(&mut self) -> &mut State { &mut self.state }

    /// executar tick de cliente único, ajudar input e atualizar estado do jogo pela duração recebida
    pub fn tick(&mut self, input: Input, dt: Duration) -> Result<(), Error> {
        // a função tick é o centro do universo lestinia
        // a maior parte das coisas pela parte do client são gerenciadas aqui
        //
        // deixar sempre a função organizada
        //
        // 1) coletar input do frontend, aplicar efeitos do input para o estado do jogo
        // 2) ir dentro de qualquer evento que precise de ajuda e aplicar eles no estado do jogo
        // 3) performar tick localstate único (ex: atualizar o mundo e suas entidades)
        // 4) ir dentro da atualização de terreno e aplicar todas as mudanças para o terreno
        // 5) finalizar o tick, passando controle para a thread principal e voltar para o frontend

        // tick para o localstate do client (passo 3)
        self.state.tick(dt);

        // finalizar o tick, passar controle de volta para o frontend (passo 6)
        Ok(())
    }
}
