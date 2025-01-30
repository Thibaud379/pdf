use crate::{
    Parsable,
    pdf_error::{PdfError, PdfErrorKind},
};

use super::{PdfDict, PdfName, PdfNumeric, parse_indirect};

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
    pub fn with_data(info: PdfDict, data: Vec<u8>) -> Result<Self, PdfError> {
        let &PdfNumeric::PdfInt(dict_len) = info
            .get(&PdfName::from_raw_bytes(b"Length"))
            .ok_or(PdfError::with_kind(PdfErrorKind::MissingStreamLength))?
            .as_numeric_ref()?
        else {
            Err(PdfError::with_kind(PdfErrorKind::WrongType))?
        };
        if dict_len as usize != data.len() {
            Err(PdfError::with_kind(PdfErrorKind::InvalidData))?
        }
        Ok(PdfStream {
            info,
            length: data.len(),
            data,
        })
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
