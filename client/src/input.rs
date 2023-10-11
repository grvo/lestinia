use vek::*;

pub struct Input {
	// todo: utilizar esse tipo para gerenciar input do client
	pub move_dir: Vec2<f32>
}

impl Default for Input {
	fn default() -> Self {
		Input {
			move_dir: Vec2::zero()
		}
	}
}
