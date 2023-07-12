// padrão
use std::time::Duration;

// externo
use specs::World as EcsWorld;

// caixote
use Crate::{
    comp,

    terrain::TerrainMap,
    vol::VolSize
};

// tipagem utilizada para representar estado do jogo armazenado tanto no client quanto no servidor.
// isso inclui coisas como componentes, dados de terreno, estado global (ex: chuva), tempo do dia, etc.
pub struct State {
    ecs_world: EcsWorld,
    terrain_map: TerrainMap,
    time: f64
}

impl State {
    pub fn new() -> Self {
        let mut ecs_world = EcsWorld::new();

        comp::register_local_components(&mut ecs_world);

        Self {
            ecs_world,

            terrain_map: TerrainMap::new(),
            time: 0.0
        }
    }

    // executar tick individual, simulando estado de jogo pela duração recebida
    pub fn tick(&mut self, dt: Duration) {
        println!("tickado!");
    }
}