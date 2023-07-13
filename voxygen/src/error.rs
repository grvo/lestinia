// padrão
use std::any;

// caixote
use crate::render::RenderError;

/// representa qualquer erro que podem ser avisados pelo voxygen
#[derive(Debug)]
pub enum Error {
    /// erro diverso relacionado a uma dependência de backend
    BackendError(Box<any::Any>),

    /// erro relacionado ao subsistema de renderização
    RenderError(RenderError),

    /// erro variado com uma fonte desconhecida ou não especificada
    Other(failure::Error)
}

impl From<RenderError> for Error {
    fn from(err: RenderError) -> Self {
        Error::RenderError(err)
    }
}