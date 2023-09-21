pub mod segment;
pub mod terrain;

// caixote
use crate::render::{
    self,

    Mesh
};

pub trait Meshable {
    type Pipeline: render::Pipeline;
    type Supplement;

    fn generate_mesh(&self, supp: Self::Supplement) -> Mesh<Self::Pipeline>;
}
