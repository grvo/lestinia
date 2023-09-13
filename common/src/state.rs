// padrão
use std::time::Duration;

// externo
use specs::World as EcsWorld;

// caixote
use Crate::{
    comp,

    terrain::TerrainMap
};

/// o quão rápido deve um dia do jogo ser comparado a um dia real
// todo: não forçar código disso
const DAY_CYCLE_FACTOR: f64 = 24.0 * 60.0;

/// recurso para armazenar tempo do dia
struct TimeOfDay(f64);

/// recurso para armazenar o tempo de tick (exemplo: físicas)
struct Tick(f64);

// tipagem utilizada para representar estado do jogo armazenado tanto no client quanto no servidor.
// isso inclui coisas como componentes, dados de terreno, estado global (ex: chuva), tempo do dia, etc.
pub struct State {
    ecs_world: EcsWorld,
    terrain_map: TerrainMap,
    time: f64
}

impl State {
    /// criar um novo `state`
    pub fn new() -> Self {
        let mut ecs_world = EcsWorld::new();

        // registrar recursos utilizados por ecs
        ecs_world.add_resource(TimeOfDay(0.0));
        ecs_world.add_resource(Tick(0.0));

        // registrar componentes comuns com o estado
        comp::register_local_components(&mut ecs_world);

        Self {
            ecs_world,

            terrain_map: TerrainMap::new(),
            time: 0.0
        }
    }

    /// obter o tempo do dia do jogo atual
    ///
    /// note que isso não deve ser utilizado por físicas, animações ou qualquer outros tempos localizados
    pub fn get_time_of_day(&self) -> f64 {
        self.ecs_world.read_resource::<TimeOfDay>().0
    }

    /// obter tempo de tick do jogo atual
    ///
    /// note que isso não deve corresponder com o tempo do dia
    pub fn get_tick(&self) -> f64 {
        self.ecs_world.read_resource::<Tick>().0
    }

    // executar tick individual, simulando estado de jogo pela duração recebida
    pub fn tick(&mut self, dt: Duration) {
        // mudar o tempo
        self.ecs_world.write_resource::<TimeOfDay>().0 += dt.as_float_secs() * DAY_CYCLE_FACTOR;

        self.ecs_world.write_resource::<Tick>().0 += dt.as_float_secs();
    }
}
