use crate::filter::FilterError::MissingEOD;
use crate::filter::{FilterData, FilterIter};
use crate::pdf_error::{PdfError, PdfErrorKind, PdfResult};

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

pub struct EncodeRLE<I> {
    inner: FilterData<I>,
    buffer: Vec<u8>,
    extra_char: Option<u8>,
    done: bool,
}
impl<I> EncodeRLE<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self {
            inner,
            buffer: Vec::with_capacity(128),
            extra_char: None,
            done: false,
        }
    }
}

impl<I: FilterIter> Iterator for EncodeRLE<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        println!("In next");
        if self.done {
            return None;
        }
        if !self.buffer.is_empty() {
            return self.buffer.pop().map(Ok);
        };
        let mut same_length = 1;
        let mut same_char: u8 = match self.extra_char.take() {
            None => match self.inner.next() {
                None => {
                    self.done = true;
                    return None;
                }
                Some(Ok(val)) => val,
                val => return val,
            },
            Some(c) => c,
        };
        self.buffer.push(same_char);
        for i in 1..128u8 {
            let n = match self.inner.next() {
                None => break,
                Some(Ok(val)) => val,
                val => return val,
            };
            if n == same_char {
                same_length += 1;
                self.buffer.push(same_char);
            } else if same_length > 3 {
                // add an encoded run
                for _ in 0..same_length {
                    self.buffer.pop();
                }
                self.buffer.push(129 + (128 - same_length));
                self.buffer.push(same_char);
                self.buffer.reverse();
                self.extra_char = Some(n);
                return Some(Ok(i - same_length));
            } else {
                // continue current run
                same_char = n;
                self.buffer.push(same_char);
                same_length = 1;
            }
        }
        let mut i = 0;
        if same_length > 3 {
            for _ in 0..same_length {
                self.buffer.pop();
            }
            i = self.buffer.len() as u8;
            self.buffer.push(129 + (128 - same_length));
            self.buffer.push(same_char);
            self.buffer.push(128);
            self.buffer.reverse();
        } else {
            self.buffer.push(128);
            self.buffer.reverse()
        };
        if i > 0 {
            Some(Ok(i))
        } else {
            Some(Ok(self.buffer.pop().unwrap()))
        }
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
                        self.state = RLEState::Copy(val);
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

#[cfg(test)]
mod test {
    use crate::PdfDict;
    use crate::filter::Filter;

    #[test]
    fn test_encode() {
        let input = b"abbbcdefgggggghiiii";
        let expected = b"\x08abbbcdef\xFBg\x01h\xFDi\x80";
        let encoded: Result<Vec<_>, _> = Filter::RunLength
            .encode(input.iter().copied().map(Result::Ok), PdfDict::empty())
            .collect();
        assert!(encoded.is_ok());
        assert_eq!(encoded.unwrap().as_slice(), expected);
    }
    #[test]
    fn test_decode() {
        let input = b"\x08abbbcdef\xFBg\x01h\xFDi\x80";
        let expected = b"abbbcdefgggggghiiii";
        let decoded: Result<Vec<_>, _> = Filter::RunLength
            .decode(input.iter().copied().map(Result::Ok), PdfDict::empty())
            .collect();
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().as_slice(), expected);
    }
}
