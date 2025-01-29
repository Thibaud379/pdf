use crate::{pdf_error::PdfError, Parsable};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PdfStream {}
impl Parsable for PdfStream {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        todo!("{bytes:?}")
    }
}
