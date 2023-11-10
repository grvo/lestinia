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

use specs::{
    Entity as EcsEntity,
    world::EntityBuilder as EcsEntityBuilder,
    Builder,
    join::Join,
    saveload::MarkedBuilder
};

use vek::*;

use common::{
    comp,
    
    state::State,
    net::PostOffice,

    msg::{
        ServerMsg,
        ClientMsg
    }
};

use world::World;

use crate::client::{
    Client,
    Clients
};

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
    clients: Clients
}

impl Server {
    /// cria um novo servidor
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            state: State::new(),
            world: World::new(),

            postoffice: PostOffice::bind(SocketAddr::from(([0; 4], 59003)))?,
            clients: Clients::empty()
        })
    }

    /// obtém uma referência do estado do jogo do servidor
    #[allow(dead_code)]
    pub fn state(&self) -> &State { &self.state }

    /// obtém uma referência mutável do estado do jogo do cliente
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut State { &mut self.state }

    /// constrói um novo jogador com um uid gerado
    pub fn build_player(&mut self) -> EcsEntityBuilder {
        self.state.build_uid_entity()
            .with(comp::phys::Pos(Vec3::zero()))
            .with(comp::phys::Vel(Vec3::zero()))
            .with(comp::phys::Dir(Vec3::unit_y()))
            .with(comp::phys::UpdateKind::Passive)
    }

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
        if let Some(err) = self.postoffice.error() {
            return Err(err.into());
        }

        // auxiliar novas conexões do client (passo 2)
        frontend_events.append(&mut self.handle_new_connections()?);

        // auxiliar novas mensagens dos clients
        frontend_events.append(&mut self.handle_new_messages()?);

        // tick para o localstate do client (passo 3)
        self.state.tick(dt);

        // sincroniza os clients com o novo estado do mundo
        self.sync_clients();

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

        for mut postbox in self.postoffice.new_connections() {
            let ecs_entity = self.build_player()
                // quando o jogador for criado primeiramente, forçar notificação de física para todos
                // incluindo eles mesmos
                .with(comp::phys::UpdateKind::Force)
                .build();
            
            let uid = self.state.read_storage().get(ecs_entity).cloned().unwrap();
            
            postbox.send(ServerMsg::SetPlayerEntity(uid));

            self.clients.add(Client {
                ecs_entity,
                postbox,

                last_ping: self.state.get_time()
            });

            frontend_events.push(Event::ClientConnected {
                ecs_entity
            });
        }

        Ok(frontend_events)
    }

    /// auxiliar novas mensagens do client
    fn handle_new_messages(&mut self) -> Result<Vec<Event>, Error> {
        let mut frontend_events = Vec::new();

        let state = &mut self.state;
        
        let mut new_chat_msgs = Vec::new();
        let mut disconnected_clients = Vec::new();

        self.clients.remove_if(|client| {
            let mut disconnected = false;
            let new_msgs = client.postbox.new_messages();

            // atualizar ping do client
            if new_msgs.len() > 0 {
                client.last_ping = state.get_time();

                // processar mensagens a caminho
                for msg in new_msgs {
                    match msg {
                        ClientMsg::Ping => client.postbox.send(ServerMsg::Pong),
                        ClientMsg::Pong => {},

                        ClientMsg::Chat(msg) => new_chat_msgs.push((client.ecs_entity, msg)),

                        ClientMsg::PlayerPhysics { pos, vel, dir } => {
                            state.write_component(client.ecs_entity, pos);
                            state.write_component(client.ecs_entity, vel;
                            state.write_component(client.ecs_entity, dir);
                        },

                        ClientMsg::Disconnect => disconnected = true
                    }
                }
            } else if
                state.get_time() - client.last_ping > CLIENT_TIMEOUT || // timeout
                client.postbox.error().is_some() // erro de postbox
            {
                disconnected = true;
            } else if state.get_time() - client.last_ping > CLIENT_TIMEOUT * 0.5 {
				// tentar pingar o client caso o timeout esteja próximo

				client.postbox.send(ServerMsg::Ping);
			}

            if disconnected {
                disconnected_clients.push(client.ecs_entity);
                
                true
            } else {
                false
            }
        });

        // auxiliar novas mensagens do chat
        for (ecs_entity, msg) in new_chat_msgs {
            self.clients.notify_all(ServerMsg::Chat(msg.clone()));

            frontend_events.push(Event::Chat {
                ecs_entity,
                msg
            });
        }

        // auxiliar desconexões do cliente
        for ecs_entity in disconnected_clients {
            self.clients.notify_all(ServerMsg::EntityDeleted(state.read_storage().get(ecs_entity).cloned().unwrap()));

            frontend_events.push(Event::ClientDisconnected {
                ecs_entity
            });

            state.ecs_world_mut().delete_entity(ecs_entity);
        }

        Ok(frontend_events)
    }

    /// sincroniza os estados do client com informações atualizadas
    fn sync_clients(&mut self) {
        for (entity, &uid, &pos, &vel, &dir, update_kind) in (
            &self.state.esc_world().entities(),
            
            &self.state.ecs_world().read_storage::<comp::Uid>(),
            &self.state.ecs_world().read_storage::<comp::phys::Pos>(),
            &self.state.ecs_world().read_storage::<comp::phys::Vel>(),
            &self.state.ecs_world().read_storage::<comp::phys::Dir>(),

            &mut self.state.ecs_world().write_storage::<comp::phys::UpdateKind>()
        ).join() {
            let msg = ServerMsg::EntityPhysics {
                uid,
                pos,
                vel,
                dir
            };

            // algumas vezes é necessário forçar atualização
            match update_kind {
                comp::phys::UpdateKind::Force => self.clients.notify_all(msg),
                comp::phys::UpdateKind::Passive => self.clients.notify_all_except(entity, msg)
            }

            // com a atualização ocorrida, padrão é uma atualização passiva
            *update_kind = comp::phys::UpdateKind::Passive;
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.clients.notify_all(ServerMsg::Shutdown)
    }
}
