// biblioteca
use specs::{
	Join,
	Read,
	ReadStorage,
	System,
	WriteStorage
};

// caixote
use crate::{
	comp::phys::{
		Pos,
		Vel
	},

	state::DeltaTime
};

// sistema de física ecs básico
pub struct MovementSys;

impl<'a> System<'a> for MovementSys {
	type SystemData = (
		WriteStorage<'a, Pos>,

		ReadStorage<'a, Vel>,
		Read<'a, DeltaTime>
	);

	fn run(&mut self, (mut positions, velocities, dt): Self::SystemData) {
		(&mut positions, &velocities)
			.join() // isso pode ser em paralelo com par_join()
			.for_each(|(pos, vel)| pos.0 += vel.0 * dt.0 as f32);
	}
}
