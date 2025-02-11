use crate::pdf_error::*;

use super::{Parsable, PdfObject, is_regular, strip_whitespace};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PdfNull {}

impl Parsable for PdfNull {
    fn from_bytes(bytes: &[u8]) -> PdfResult<(Self, &[u8])> {
        if bytes.starts_with(b"null") && bytes.get(4).is_none_or(|b| !is_regular(*b)) {
            let bytes = strip_whitespace(&bytes[4..]);
            Ok((Self {}, bytes))
        } else {
            Err(PdfError::with_kind(PdfErrorKind::Parse))
        }
    }
}
impl From<Option<PdfObject>> for PdfObject {
    fn from(value: Option<PdfObject>) -> Self {
        match value {
            Some(o) => o,
            None => PdfNull {}.into(),
        }
    }
}
