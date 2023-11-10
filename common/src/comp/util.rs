// biblioteca
use specs::{
    Component,

    NullStorage
};

use vek::*;

// pos
#[derive(Copy, Clone, Debug, Default)]
pub struct New;

impl Component for New {
    type Storage = NullStorage<Self>;
}
