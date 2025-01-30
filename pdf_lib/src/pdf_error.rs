use std::error::Error;

#[allow(dead_code)]
pub type PdfResult<T> = std::result::Result<T, PdfError>;
#[derive(PartialEq, Debug, Clone)]
pub enum PdfErrorKind {
    ParseError,
    WrongType,
    InvalidData,
    External(String),
    MissingStreamLength,
}
#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub struct PdfError {
    pub(crate) kind: PdfErrorKind,
}
impl PdfError {
    pub(crate) fn with_kind(kind: PdfErrorKind) -> Self {
        Self { kind }
    }
}

impl<E> From<E> for PdfError
where
    E: Error,
{
    fn from(e: E) -> Self {
        Self {
            kind: PdfErrorKind::External(e.to_string()),
        }
    }
}
