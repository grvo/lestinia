#![feature(drain_filter)]

pub mod client;
pub mod error;
pub mod input;

// re-exportações
pub use crate::{
    error::Error,
    input::Input
};

use std::{
    time::Duration,
    net::SocketAddr
};

use specs::Entity as EcsEntity;

use common::{
    state::State,
    net::PostOffice,

    msg::{
        ServerMsg,
        ClientMsg
    }
};

use world::World;

use crate::client::Client;

const CLIENT_TIMEOUT: f64 = 5.0; // segundos

pub enum Event {
    ClientConnected {
        ecs_entity: EcsEntity
    },

    ClientDisconnected {
        ecs_entity: EcsEntity
    },

    Chat {
        ecs_entity: EcsEntity,
        msg: String
    }
}

pub struct Server {
    state: State,
    world: World,

    postoffice: PostOffice<ServerMsg, ClientMsg>,
    clients: Vec<Client>
}

impl Server {
    /// cria um novo servidor
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            state: State::new(),
            world: World::new(),

            postoffice: PostOffice::new(SocketAddr::from(([0; 4], 59003)))?,
            clients: Vec::new()
        })
    }

    /// obtém uma referência do estado do jogo do servidor
    #[allow(dead_code)]
    pub fn state(&self) -> &State { &self.state }

    /// obtém uma referência mutável do estado do jogo do cliente
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut State { &mut self.state }

    /// obtém uma referência para o mundo do servidor
    #[allow(dead_code)]
    pub fn world(&self) -> &World { &self.world }

    /// obtém uma referência mutável do mundo do servidor
    #[allow(dead_code)]
    pub fn world_mut(&mut self) -> &mut World { &mut self.world }

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
        // 3) ir dentro de todas as comunicações recebidas da rede do client
        // 4) performar tick localstate único (ex: atualizar o mundo e suas entidades)
        // 5) ir dentro da atualização de terreno e aplicar todas as mudanças para o terreno
        // 6) enviar atualizações de estado relevantes para todos os clients
        // 7) finalizar o tick, passando controle para a thread principal e voltar para o frontend

        // constrói uma lista de eventos para esse frame, para ser passado para o frontend
        let mut frontend_events = Vec::new();

        // se estiver tendo problemas com networking, auxiliar eles
        if let Some(err) = self.postoffice.status() {
            return Err(err.into());
        }

        // auxiliar novas conexões do client (passo 2)
        frontend_events.append(&mut self.handle_new_connections()?);

        // auxiliar novas mensagens dos clients
        frontend_events.append(&mut self.handle_new_messages()?);

        // tick para o localstate do client (passo 3)
        self.state.tick(dt);

        // finalizar o tick, passar controle de volta para o frontend (passo 6)
        Ok(frontend_events)
    }

    /// limpar o servidor depois de tick
    #[allow(dead_code)]
    pub fn cleanup(&mut self) {
        // limpar o estado local
        self.state.cleanup();
    }

    /// auxiliar novas conexões do client
    fn handle_new_connections(&mut self) -> Result<Vec<Event>, Error> {
        let mut frontend_events = Vec::new();

        for postbox in self.postoffice.new_connections() {
            // todo: não utilizar esse método
            let ecs_entity = self.state.new_test_player();

            frontend_events.push(Event::ClientConnected {
                ecs_entity
            });

            self.clients.push(Client {
                ecs_entity,
                postbox,

                last_ping: self.state.get_time()
            });
        }

        Ok(frontend_events)
    }

    /// auxiliar novas mensagens do client
    fn handle_new_messages(&mut self) -> Result<Vec<Event>, Error> {
        let mut frontend_events = Vec::new();

        let state = &mut self.state;
        let mut new_chat_msgs = Vec::new();

        self.clients.drain_filter(|client| {
            let mut disconnected = false;
            let new_msgs = client.postbox.new_messages();

            // atualizar ping do client
            if new_msgs.len() > 0 {
                client.last_ping = state.get_time();

                // processar mensagens a caminho
                for msg in new_msgs {
                    match msg {
                        ClientMsg::Chat(msg) => new_chat_msgs.push((client.ecs_entity, msg)),
                        ClientMsg::Disconnect => disconnected = true
                    }
                }
            } else if
                state.get_time() - client.last_ping > CLIENT_TIMEOUT ||
                client.postbox.status().is_some()
            {
                disconnected = true;
            }

            if disconnected {
                state.delete_entity(client.ecs_entity);

                frontend_events.push(Event::ClientDisconnected {
                    ecs_entity: client.ecs_entity
                });

                true
            } else {
                false
            }
        });

        // auxiliar novas mensagens do chat
        for (ecs_entity, msg) in new_chat_msgs {
            for client in &mut self.clients {
                let _ = client.postbox.send(ServerMsg::Chat(msg.clone()));
            }

            frontend_events.push(Event::Chat {
                ecs_entity,
                msg
            });
        }

        Ok(frontend_events)
    }
}
