#![feature(label_break_value)]

pub mod error;
pub mod input;

// re-exportações
pub use specs::Entity as EcsEntity;

pub use crate::{
    error::Error,
    input::Input
};

use std::{
    time::Duration,
    net::SocketAddr
};

use vek::*;
use threadpool;

use specs::Builder;

use common::{
    comp,
    
    state::State,
    terrain::TerrainChunk,
    net::PostBox,
    
    msg::{
        ClientMsg,
        ServerMsg
    }
};

use world::World;

const SERVER_TIMEOUT: f64 = 5.0; // segundos

pub enum Event {
    Chat(String)
}

pub struct Client {
    thread_pool: threadpool::ThreadPool,

    last_ping: f64,
    postbox: PostBox<ClientMsg, ServerMsg>,

    tick: u64,
    state: State,
    player: Option<Uid>,

    // teste
    world: World,
    
    pub chunk: Option<TerrainChunk>
}

impl Client {
    /// cria um novo `client`
    #[allow(dead_code)]
    pub fn new<A: Into<SocketAddr>>(addr: A) -> Result<Self, Error> {
        let state = State::new();
        let mut postbox = PostBox::to_server(addr)?;

        postbox.send(ClientMsg::Chat(String::from("olá, mundo!")));
        postbox.send(ClientMsg::Chat(String::from("mundo, olá!")));
        
        Ok(Self {
            thread_pool: threadpool::Builder::new()
                .thread_name("lestinia-worker".into())
                .build(),

            last_ping: state.get_time(),
            postbox,

            tick: 0,
            state,
            player: None,

            // teste
            world: World::new(),
            chunk: None
        })
    }

    /// obtém a referência do threadpool do worker do client.
    ///
    /// esse pool deve ser utilizado para qualquer opearação expansiva que roda fora da thread principal
    /// como por exemplo, thread que bloqueiam operações i/o
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn state(&self) -> &state {
        &self.state
    }

    /// obtém a referência mutável para estado do jogo do cliente
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// obtém uma entidade por meio de seu uid, criando um caso ainda não exista
    pub fn get_or_create_entity_from_uid(&mut self, uid: u64) -> EcsEntity {
        // encontra a entidade ecs por meio de uid
        let ecs_entity = self.state().ecs_world()
            .read_resource::<comp::UidAllocator>()
            .retrieve_entity_internal(uid.into());

        // retorna a entidade ou cria uma
        if let Some(ecs_entity) = ecs_entity {
            ecs_entity
        } else {
            let ecs_entity = self.state.ecs_world_mut().create_entity()
                .build();

            // aloca no uid específico recebido
            self.state
                .ecs_world_mut()
                .write_resource::<comp::UidAllocator>()
                .allocate(ecs_entity, Some(uid.into()));

            ecs_entity
        }
    }

    /// obtém a entidade player
    #[allow(dead_code)]
    pub fn player(&self) -> Option<EcsEntity> {
        self.player
    }

    /// obtém o número de tick atual
    #[allow(dead_code)]
    pub fn get_tick(&self) -> u64 {
        self.tick
    }

    /// envia uma mensagem do chat para o servidor
    #[allow(dead_code)]
    pub fn send_chat(&mut self, msg: String) {
        self.postbox.send(ClientMsg::Chat(msg))
    }

    /// executar tick de cliente único, ajudar input e atualizar estado do jogo pela duração recebida
    #[allow(dead_code)]
    pub fn tick(&mut self, input: Input, dt: Duration) -> Result<Vec<Event>, Error> {
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

        // constrói uma lista de eventos para esse frame, para ser passado no frontend
        let mut frontend_events = Vec::new();

        // auxiliar novas mensagens do servidor
        frontend_events.append(&mut self.handle_new_messages()?);

        // passo 3
        if let Some(ecs_entity) = self.player {
            // todo: remover isso
            const PLAYER_VELOCITY: f32 = 100.0;

            // todo: determinar aceleração
            self.state.write_component(ecs_entity, comp::phys::Vel(Vec3::from(input.move_dir * PLAYER_VELOCITY)));
        }

        // tick para o localstate do client (passo 3)
        self.state.tick(dt);

        // atualizar o servidor por conta dos atributos físicos do jogador
        if let Some(ecs_entity) = self.player {
            match (
                self.state.read_storage().get(ecs_entity).cloned(),
                self.state.read_storage().get(ecs_entity).cloned(),
                self.state.read_storage().get(ecs_entity).cloned()
            ) {
                (Some(pos), Some(vel), Some(dir)) => {
                    self.postbox.send(ClientMsg::PlayerPhysics {
                        pos, vel, dir
                    });
                },

                _ => {}
            }
        }

        // finalizar o tick, passar controle de volta para o frontend (passo 6)
        self.tick += 1;
        
        Ok(frontend_events)
    }

    /// limpa o client depois de um tick
    #[allow(dead_code)]
    pub fn cleanup(&mut self) {
        // limpar o estado local
        self.state.cleanup();
    }

    /// auxiliar novas mensagens do servidor
    fn handle_new_messages(&mut self) -> Result<Vec<Event>, Error> {
        let mut frontend_events = Vec::new();

        // passo 1
        let new_msgs = self.postbox.new_messages();

        if new_msgs.len() > 0 {
            self.last_ping = self.state.get_time();

            for msg in new_msgs {
                match msg {            
                    ServerMsg::Shutdown => return Err(Error::ServerShutdown),

                    ServerMsg::Ping => self.postbox.send(ClientMsg::Pong),
                    ServerMsg::Pong => {},

                    ServerMsg::Chat(msg) => frontend_events.push(Event::Chat(msg)),

                    ServerMsg::SetPlayerEntity(uid) => {
                        let ecs_entity = self.state
                            .get_entity(uid)
                            .unwrap_or_else(|| self.state.build_uid_entity_with_uid(uid).build());

                        self.player = Some(ecs_entity);
                    },

                    ServerMsg::EntityPhysics { uid, pos, vel, dir } => {
                        let ecs_entity = self.state
                            .get_entity(uid)
                            .unwrap_or_else(|| self.state.build_uid_entity_with_uid(uid).build());
                        
                        self.state.write_component(ecs_entity, pos);
                        self.state.write_component(ecs_entity, vel);
                        self.state.write_component(ecs_entity, dir);
                    },

                    ServerMsg::EntityDeleted(uid) => {
                        self.state.delete_entity(uid);
                    }
                }
            }
        } else if let Some(err) = self.postbox.status() {
            return Err(err.into());
        } else if self.state.get_time() - self.last_ping > SERVER_TIMEOUT * 0.5 {
            self.postbox.send(ClientMsg::Ping);
        } else if self.state.get_time() - self.last_ping > SERVER_TIMEOUT {
            return Err(Error::ServerTimeout);
        }

        Ok(frontend_events)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.postbox.send(ClientMsg::Disconnect);
    }
}
