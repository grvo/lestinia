pub mod phys;

// externo
use specs::{World as EcsWorld, Builder};

pub fn register_local_components(ecs_world: &mut EcsWorld) {
    ecs_world.register::<phys::Pos>();
    ecs_world.register::<phys::Vel>();
    ecs_world.register::<phys::Dir>();
}