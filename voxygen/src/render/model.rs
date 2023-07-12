// padrão
use std::marker::PhantomData;

// local
use super::Pipeline;

// representa o mesh que foi enviado para a cpu
pub struct Model<P: Pipeline> {
    phantom: PhantomData<P>
}

impl<P: Pipeline> Model<P> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData
        }
    }
}