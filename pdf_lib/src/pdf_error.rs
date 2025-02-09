use std::{error::Error, fmt::Display, num::ParseIntError, str::Utf8Error};

use crate::filter::FilterError;

#[allow(dead_code)]
pub type PdfResult<T> = std::result::Result<T, PdfError>;
#[derive(PartialEq, Debug, Clone)]
pub enum PdfErrorKind {
    Parse,
    WrongType,
    InvalidData,
    External(String),
    MissingStreamLength,
    Filter(FilterError),
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

impl Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for PdfError {}

macro_rules! impl_from {
    ($($E:ty),+) => {
        $(
            impl From<$E> for PdfError {
                fn from(e: $E) -> Self {
                    Self {
                        kind: PdfErrorKind::External(e.to_string()),
                    }
                }
            }
        )+
    };
}

impl_from!(ParseIntError, Utf8Error);
