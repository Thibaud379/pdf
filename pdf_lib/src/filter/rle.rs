use crate::filter::FilterError::MissingEOD;
use crate::filter::asciihex::EncodeASCIIHex;
use crate::filter::{FilterData, FilterError, FilterIter};
use crate::pdf_error::{PdfError, PdfErrorKind, PdfResult};
use std::io::Repeat;
use std::iter::repeat;

pub struct EncodeRLE<I> {
    inner: FilterData<I>,
}
impl<I> EncodeRLE<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self { inner }
    }
}

impl<I: FilterIter> Iterator for EncodeRLE<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
const RLE_BUFFER_SIZE: usize = 128;

pub struct DecodeRLE<I> {
    inner: FilterData<I>,
    buffer: [u8; RLE_BUFFER_SIZE],
    buffer_index: usize,
}

impl<I> DecodeRLE<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self {
            inner,
            buffer: [0; RLE_BUFFER_SIZE],
            buffer_index: 0,
        }
    }
}

impl<I: FilterIter> Iterator for DecodeRLE<I> {
    type Item = PdfResult<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_index > 0 {
            let output = self.buffer[self.buffer_index - 1];
            self.buffer_index -= 1;
            return Some(Ok(output));
        };
        let marker = self.inner.next();
        if marker.is_none() || marker.as_ref().is_some_and(|t| t.is_err()) {
            return marker;
        };
        let marker = marker.unwrap().unwrap() as usize;
        if marker < 128 {
            let next_bytes: PdfResult<Vec<_>> = self.inner.by_ref().take(marker + 1).collect();
            match next_bytes {
                Err(e) => return Some(Err(e)),
                Ok(mut b) => {
                    if b.len() != marker + 1 {
                        return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(MissingEOD))));
                    }
                    b.extend(repeat(0).take(RLE_BUFFER_SIZE - 1 - marker));
                    self.buffer = b.try_into().expect("we padded with zeros");
                    self.buffer_index = marker;
                }
            };
        } else if marker > 128 {
            let byte_to_copy = self.inner.next();
            if byte_to_copy.is_none() {
                return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(MissingEOD))));
            };
            if byte_to_copy.as_ref().is_some_and(|b| b.is_ok()) {
                return byte_to_copy;
            }
            let byte = byte_to_copy.unwrap().unwrap();
            self.buffer.fill(byte);
            self.buffer_index = 257 - marker;
        };
        self.next()
    }
}
