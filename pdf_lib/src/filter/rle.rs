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

enum RLEState {
    Copy(u8),
    Run(u8, u8),
    Done,
}
pub struct DecodeRLE<I> {
    inner: FilterData<I>,
    state: RLEState,
}

impl<I> DecodeRLE<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self {
            inner,
            state: RLEState::Copy(0),
        }
    }
}

macro_rules! expect_next {
    ($var:expr) => {
        match $var {
            None => {
                return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(MissingEOD))));
            }
            Some(Err(e)) => {
                return Some(Err(e));
            }
            Some(Ok(val)) => val,
        }
    };
}
impl<I: FilterIter> Iterator for DecodeRLE<I> {
    type Item = PdfResult<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            RLEState::Done => None,
            RLEState::Copy(ref mut val) if *val > 0 => {
                let output = expect_next!(self.inner.next());
                *val -= 1;
                Some(Ok(output))
            }
            RLEState::Run(ref mut val, byte) if *val > 0 => {
                *val -= 1;
                Some(Ok(byte))
            }
            _ => {
                match expect_next!(self.inner.next()) {
                    128 => self.state = RLEState::Done,
                    val if val <= 127 => {
                        self.state = RLEState::Copy(val+1);
                    }
                    val => {
                        let b = expect_next!(self.inner.next());
                        self.state = RLEState::Run(128 - (val - 129), b);
                    }
                };
                self.next()
            }
        }
    }
}
