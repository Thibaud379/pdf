use crate::pdf_error::*;

use super::{FilterData, FilterError, FilterIter};

pub struct EncodeASCIIHex<I> {
    inner: FilterData<I>,
    temp: Option<u8>,
}
impl<I> EncodeASCIIHex<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self { inner, temp: None }
    }
}

impl<I: FilterIter> Iterator for EncodeASCIIHex<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.temp.is_some_and(|b| b != b'>') {
            return self.temp.take().map(Result::Ok);
        }
        match self.inner.next() {
            Some(Ok(b)) => {
                let s = format!("{b:02X}");
                let s = s.as_bytes();
                self.temp = Some(s[1]);
                Some(Ok(s[0]))
            }
            Some(Err(e)) => return Some(Err(e)),
            None => match self.temp {
                Some(_) => None,
                None => {
                    self.temp = Some(b'>');
                    Some(Ok(b'>'))
                }
            },
        }
    }
}
pub struct DecodeASCIIHex<I> {
    inner: FilterData<I>,
    eod: bool,
}
impl<I> DecodeASCIIHex<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self { inner, eod: false }
    }
}

impl<I: Iterator<Item = PdfResult<u8>>> Iterator for DecodeASCIIHex<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eod {
            return None;
        }
        let first_byte = match self.inner.next_non_whitespace() {
            Some(Ok(b)) if b.is_ascii_hexdigit() => b,
            Some(Ok(b)) if b == b'>' => {
                self.eod = true;
                return None;
            }
            Some(Ok(b)) => {
                return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                    FilterError::ASCIIHexDecode(b),
                ))));
            }
            None => {
                if !self.eod {
                    return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                        FilterError::MissingEOD,
                    ))));
                } else {
                    return None;
                }
            }
            o => return o,
        };

        let second_byte = match self.inner.next_non_whitespace() {
            Some(Ok(b)) if b.is_ascii_hexdigit() => Some(b),
            Some(Ok(b)) if b == b'>' => {
                self.eod = true;
                None
            }
            Some(Ok(b)) => {
                return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                    FilterError::ASCIIHexDecode(b),
                ))));
            }
            None => return None,
            o => return o,
        };
        let r = match second_byte {
            None => (first_byte as char).to_digit(16)? * 16,
            Some(second) => {
                (first_byte as char).to_digit(16)? * 16 + (second as char).to_digit(16)?
            }
        } as u8;
        Some(Ok(r))
    }
}

#[cfg(test)]
mod tests {
    use crate::PdfDict;

    use super::super::Filter;

    #[test]
    fn encode_ascii_hex() {
        let examples: [&[u8]; 3] = [b"test", b"\x12\xFF", b" \n0\x00"];
        let expected: [&[u8]; 3] = [b"74657374>", b"12FF>", b"200A3000>"];
        for (example, exp) in examples.into_iter().zip(expected) {
            let encoded: Result<Vec<_>, _> = Filter::ASCIIHex
                .encode(example.iter().copied().map(Result::Ok), PdfDict::empty())
                .collect();
            assert!(encoded.is_ok());
            assert_eq!(encoded.unwrap().as_slice(), exp);
        }
    }
    #[test]
    fn decode_ascii_hex() {
        let examples: [&[u8]; 3] = [b"74657374>", b"12FF>", b"200A3000>"];
        let expected: [&[u8]; 3] = [b"test", b"\x12\xFF", b" \n0\x00"];
        for (example, exp) in examples.into_iter().zip(expected) {
            let decoded: Result<Vec<_>, _> = Filter::ASCIIHex
                .decode(example.iter().copied().map(Result::Ok), PdfDict::empty())
                .collect();
            assert!(decoded.is_ok());
            assert_eq!(decoded.unwrap().as_slice(), exp);
        }
    }

    #[test]
    fn chain_ascii_hex() {
        let examples: [&[u8]; 6] = [
            b"74657374>",
            b"12FF>",
            b"test",
            b"\x12\xFF",
            b" \n0\x00",
            b"200A3000>",
        ];
        for e in examples {
            let encoded =
                Filter::ASCIIHex.encode(e.iter().copied().map(Result::Ok), PdfDict::empty());
            let decoded = Filter::ASCIIHex.decode(encoded, PdfDict::empty());
            let c = decoded.collect::<Result<Vec<_>, _>>();
            assert!(c.is_ok());
            assert_eq!(c.unwrap().as_slice(), e);
        }
    }
}
