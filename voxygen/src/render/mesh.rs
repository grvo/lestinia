// local
use super::Pipeline;

/// utilizado para armazenar dado de vertex na cpu
pub struct Mesh<P: Pipeline> {
    verts: Vec<P::Vertex>
}

impl<P: Pipeline> Mesh<P> {
    pub fn empty() -> Self {
        Self { verts: vec![] }
    }

    pub fn verts(&self) -> &[P::Vertex] {
        &self.verts
    }

    pub fn push(&mut self, vert: P::Vertex) {
        self.verts.push(vert);
    }
}