pub mod comp;

// padrão
use std::time::Duration;

// externo
use specs::{World as EcsWorld, Builder};

// tipagem utilizada para representar estado do jogo armazenado tanto no client quanto no servidor.
// isso inclui coisas como componentes, dados de terreno, estado global (ex: chuva), tempo do dia, etc.
pub struct LocalState {
    ecs_world: EcsWorld
}

impl LocalState {
    pub fn new() -> Self {
        let mut ecs_world = EcsWorld::new();

        comp::register_local_components(&mut ecs_world);

        Self {
            ecs_world
        }
    }

    // executar tick individual, simulando estado de jogo pela duração recebida
    pub fn tick(&mut self, dt: Duration) {
        println!("tickado!");
    }
}