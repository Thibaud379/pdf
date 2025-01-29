use crate::{Parsable, pdf_error::PdfError};

use super::{PdfDict, parse_indirect};

#[derive(Debug, PartialEq, Clone)]
pub struct PdfStream {
    info: PdfDict,
    length: usize,
    data: Vec<u8>,
}

impl PdfStream {
    pub fn with_len(info: PdfDict, length: usize, data: Vec<u8>) -> Self {
        PdfStream { info, length, data }
    }
    pub fn len(&self) -> usize {
        self.length
    }
}
impl Parsable for PdfStream {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        let (indirect, b) = parse_indirect(bytes)?;
        let stream = indirect.as_stream()?;
        Ok((stream, b))
    }
}
