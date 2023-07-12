// biblioteca
use gfx::{
    self,

    traits::FactoryExt
};

// local
use super::{
    mesh::Mesh,
    Pipeline,
    gfx_backend
};

// representa o mesh que foi enviado para a cpu
pub struct Model<P: Pipeline> {
    pub vbuf: gfx::handle::Buffer<gfx_backend::Resources, P::Vertex>,

    pub slice: gfx::Slice<gfx_backend::Resources>
}

impl<P: Pipeline> Model<P> {
    pub fn new(
        factory: &mut gfx_backend::Factory,
        mesh: &Mesh<P>
    ) -> Self {
        Self {
            vbuf: factory.create_vertex_buffer(mesh.vertices()),

            slice: gfx::Slice {
                start: 0,
                end: mesh.vertices().len() as u32,
                base_vertex: 0,
                instances: None,
                buffer: gfx::IndexBuffer::Auto
            }
        }
    }
}