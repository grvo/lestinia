// padrão
use std::time::Duration;

// biblioteca
use shred::{
    Fetch,
    FetchMut
};

use specs::{
    Builder,
    Component,
    DispatcherBuilder,
    
    Entity as EcsEntity,
    World as EcsWorld,

    storage::{
        Storage as EcsStorage,
        MaskedStorage as EcsMaskedStorage
    }
};

use vek::*;

// caixote
use Crate::{
    comp,
    sys,

    terrain::TerrainMap
};

/// o quão rápido deve um dia do jogo ser comparado a um dia real
// todo: não forçar código disso
const DAY_CYCLE_FACTOR: f64 = 24.0 * 60.0;

/// recurso para armazenar tempo do dia
struct TimeOfDay(f64);

/// recurso para armazenar o tempo de tick (exemplo: físicas)
struct Time(f64);

/// recurso utilizado para armazenar o tempo desde o último tick
#[derive(Default)]
pub struct DeltaTime(pub f64);

pub struct Changes {
    pub new_chunks: Vec<Vec3<i32>>,
    pub changed_chunks: Vec<Vec3<i32>>,
    pub removed_chunks: Vec<Vec3<i32>>
}

impl Changes {
    pub fn default() -> Self {
        Self {
            new_chunks: vec![],
            changed_chunks: vec![],
            removed_chunks: vec![]
        }
    }

    pub fn cleanup(&mut self) {
        self.new_chunks.clear();
        self.changed_chunks.clear();
        self.removed_chunks.clear();
    }
}

// tipagem utilizada para representar estado do jogo armazenado tanto no client quanto no servidor.
// isso inclui coisas como componentes, dados de terreno, estado global (ex: chuva), tempo do dia, etc.
pub struct State {
    ecs_world: EcsWorld,
    changes: Changes
}

impl State {
    /// criar um novo `state`
    pub fn new() -> Self {
        let mut ecs_world = EcsWorld::new();

        // registrar recursos utilizados por ecs
        ecs_world.add_resource(TimeOfDay(0.0));
        ecs_world.add_resource(Time(0.0));
        ecs_world.add_resource(DeltaTime(0.0));
        ecs_world.add_resource(TerrainMap::new());

        // registrar componentes comuns com o estado
        comp::register_local_components(&mut ecs_world);

        Self {
            ecs_world,
            
            changes: Changes::default()
        }
    }

    /// registrar um componente com o ecs do estado
    pub fn with_component<T: Component>(mut self) -> Self
        where <T as Component>::Storage: Default
    {
        self.ecs_world.register::<T>();

        self
    }

    /// deleta uma entidade do ecs do estado, caso exista
    pub fn delete_entity(&mut self, entity: EcsEntity) {
        let _ = self.ecs_world.delete_entity(entity);
    }

    // todo: obter rid disso
    pub fn new_test_player(&mut self) -> EcsEntity {
        self.ecs_world
            .create_entity()

            .with(comp::phys::Pos(Vec3::default()))
            .with(comp::phys::Vel(Vec3::default()))
            .with(comp::phys::Dir(Vec3::default()))
            
            .build()
    }

    /// escreve um componente atribuído a uma entidade em particular
    pub fn write_component<C: Component>(&mut self, e: EcsEntity, comp: C) {
        let _ = self.ecs_world.write_storage().insert(entity, comp);
    }

    /// lê uma clonagem de um componente atribuído a uma entidade em particular
    pub fn read_component<C: Component + Clone>(&self, entity: EcsEntity) -> Option<C> {
        self.ecs_world.read_storage::<C>().get(entity).cloned()
    }

    /// obtém uma referência (apenas leitura) para o armazenamento de um tipo de componente em particular
    pub fn read_storage<C: Component>(&self) -> EcsStorage<C, Fetch<EcsMaskedStorage<C>>> {
        self.ecs_world.read_storage::<C>()
    }

    /// obtém uma referência para o mundo ecs interno
    pub fn ecs_world(&self) -> &EcsWorld {
        &self.ecs_world
    }

    /// obtém uma referência mutável para o mundo ecs interno
    pub fn ecs_world_mut(&mut self) -> &mut EcsWorld {
        &mut self.ecs_world
    }

    /// obtém uma referência para a estrutura changes
    ///
    /// isso contém informação sobre o estado que foi alterado assim que ocorreu um tick no jogo
    pub fn changes(&self) -> &Changes {
        &self.changes
    }

    // todo: obter rid disso quando não for necessário
    pub fn changes_mut(&mut self) -> &mut Changes {
        &mut self.changes
    }

    /// obtém o tempo do dia do jogo atual
    ///
    /// note que isso não deve ser utilizado por físicas, animações ou qualquer outros tempos localizados
    pub fn get_time_of_day(&self) -> f64 {
        self.ecs_world.read_resource::<TimeOfDay>().0
    }

    /// obtém o tempo atual do jogo
    ///
    /// note que isso não deve corresponder com o tempo do dia
    pub fn get_time(&self) -> f64 {
        self.ecs_world.read_resource::<Time>().0
    }

    /// obtém uma referência para esse terreno do estado
    pub fn terrain(&self) -> Fetch<TerrainMap> {
        self.ecs_world.read_resource::<TerrainMap>()
    }

    // todo: obter rid disso quando não for necessário
    pub fn terrain_mut(&mut self) -> FetchMut<TerrainMap> {
        self.ecs_world.write_resource::<TerrainMap>()
    }

    // executar tick individual, simulando estado de jogo pela duração recebida
    pub fn tick(&mut self, dt: Duration) {
        // mudar o tempo
        self.ecs_world.write_resource::<TimeOfDay>().0 += dt.as_float_secs() * DAY_CYCLE_FACTOR;

        self.ecs_world.write_resource::<Time>().0 += dt.as_float_secs();

        // rodar sistemas para atualizar o mundo
        self.ecs_world.write_resource::<DeltaTime>().0 = dt.as_float_secs();

        // cria e roda um dispatcher para os sistemas ecs
        let mut dispatch_builder = DispatcherBuilder::new();

        sys::add_local_systems(&mut dispatch_builder);

        // isso dispatcha todos os sistemas em paralelo
        dispatch_builder.build().dispatch(&self.ecs_world.res);

        self.ecs_world.maintain();
    }

    /// limpar o estado depois de tick
    pub fn cleanup(&mut self) {
        // limpar estruturas de dados do último tick
        self.changes.cleanup();
    }
}
