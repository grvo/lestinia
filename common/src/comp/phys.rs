// biblioteca
use specs::{Component, VecStorage};
use vek::*;

// pos

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Pos(pub Vec3<f32>);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

// vel

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vel(pub Vec3<f32>);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

// dir

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Dir(pub Vec3<f32>);

impl Component for Dir {
    type Storage = VecStorage<Self>;
}
