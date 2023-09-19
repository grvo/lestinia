// padrão
use std::time::Duration;

// biblioteca
use specs::Entity as EcsEntity;
use vek::*;
use threadpool;

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
    thread_pool: threadpool::ThreadPool,

    tick: u64,
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
            thread_pool: threadpool::Builder::new()
                .thread_name("lestinia-worker".into())
                .build(),

            tick: 0,
            state: State::new(),
            player: None,

            // teste
            world: World::new(),
            chunk: None
        }
    }

    /// obtém a referência do threadpool do worker do client.
    ///
    /// esse pool deve ser utilizado para qualquer opearação expansiva que roda fora da thread principal
    /// como por exemplo, thread que bloqueiam operações i/o
    pub fn thread_pool(&self) -> &threadpool::ThreadPool {
        &self.thread_pool
    }

    // todo: obtém o rid disso
    pub fn with_test_state(mut self) -> Self {
        self.chunk = Some(self.world.generate_chunk(Vec3::zero()));

        self
    }

    /// todo: obtém o rid disso
    pub fn load_chunk(&mut self, pos: Vec3<i32>) {
        self.state.terrain_mut().insert(pos, self.world.generate_chunk(pos));

        self.state.changes_mut().new_chunks.push(pos);
    }

    /// obtém a referência para o estado do jogo do cliente
    pub fn state(&self) -> &state {
        &self.state
    }

    /// obtém a referência mutável para estado do jogo do cliente
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// obtém o número de tick atual
    pub fn get_tick(&self) -> u64 {
        self.tick
    }

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
        self.tick += 1;
        
        Ok(())
    }

    /// limpar o client depois de um tick
    pub fn cleanup(&mut self) {
        // limpar o estado local
        self.state.cleanup();
    }
}
