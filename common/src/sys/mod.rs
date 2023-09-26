pub mod phys;

// externo
use specs::DispatcherBuilder;

// nomes de sistemas
const MOVEMENT_SYS: &str = "movement_sys";

pub fn add_local_systems(dispatch_builder: &mut DispatcherBuilder) {
	dispatch_builder.add(phys::MovementSys, MOVEMENT_SYS, &[]);
}
