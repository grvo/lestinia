// biblioteca
use gfx::{
    self,

    traits::FactoryExt
};

// local
use super::{
    RenderError,

    gfx_backend
};

/// um identificador para uma série de constantes na gpu. isso é usado para armazenar informações usadas em o processo de renderização que não muda em uma única passagem de renderização
#[derive(Clone)]
pub struct Consts<T: Copy + gfx::traits::Pod> {
    pub buf: gfx::handle::Buffer<gfx_backend::Resources, T>
}

impl<T: Copy + gfx::traits::Pod> Consts<T> {
    /// criar um novo `const<t>`
    pub fn new(factory: &mut gfx_backend::Factory, len: usize) -> Self {
        Self {
            buf: factory.create_constant_buffer(len)
        }
    }

    /// atualiza o valor do lado da gpu representado por esse handle constante
    pub fn update(
        &mut self,
        encoder: &mut gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,
        vals: &[T]
    ) -> Result<(), RenderError> {
        encoder.update_buffer(&self.buf, vals, 0)
            .map_err(|err| RenderError::UpdateError(err))
    }
}
