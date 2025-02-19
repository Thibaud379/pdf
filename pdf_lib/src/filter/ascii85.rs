use crate::pdf_error::*;

use super::{FilterData, FilterError, FilterIter};

pub struct EncodeASCII85<I> {
    inner: FilterData<I>,
    temp: Vec<u8>,
}
impl<I> EncodeASCII85<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self {
            inner,
            temp: Vec::new(),
        }
    }
}

impl<I: FilterIter> Iterator for EncodeASCII85<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        {
            if !self.temp.is_empty() {
                return match self.temp.pop() {
                    Some(0) => {
                        self.temp.push(0);
                        None
                    }
                    Some(b) => Some(Ok(b)),
                    None => unreachable!(),
                };
            }
        }
        let mut bytes = [
            self.inner.next(),
            self.inner.next(),
            self.inner.next(),
            self.inner.next(),
        ];
        let n = if bytes.iter().all(Option::is_some) {
            4
        } else {
            let p = bytes
                .iter()
                .position(Option::is_none)
                .expect("checked for none");
            bytes = bytes.map(|orb| orb.or(Some(Ok(0))));

            p
        };
        match bytes.iter().find(|e| matches!(e, Some(Err(_)))) {
            Some(e) => return e.clone(),
            None => (),
        };
        let bytes = bytes.map(Option::unwrap).map(Result::unwrap);
        let mut base_256 = u32::from_be_bytes(bytes);
        let mut out = [0; 5];
        let o = if n == 4 && base_256 == 0 {
            &[b'z']
        } else {
            for i in 0..5 {
                out[i] = (base_256 % 85) as u8 + b'!';
                base_256 /= 85;
            }
            &out[(4 - n)..]
        };
        if n != 4 {
            self.temp.push(0);
            self.temp.push(b'>');
            self.temp.push(b'~');
        }
        if n != 0 {
            self.temp.extend_from_slice(o);
        }
        self.temp.pop().map(Result::Ok)
    }
}

pub struct DecodeASCII85<I> {
    inner: FilterData<I>,
    temp: Vec<u8>,
    done: bool,
}
impl<I> DecodeASCII85<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        Self {
            inner,
            temp: Vec::new(),
            done: false,
        }
    }
}

impl<I: FilterIter> Iterator for DecodeASCII85<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let temp = &mut self.temp;
        if !temp.is_empty() {
            return temp.pop().map(Result::Ok);
        }
        if self.done {
            return None;
        };

        let next_byte = self.inner.next_non_whitespace();
        if let Some(Ok(b'z')) = next_byte {
            // handle z case
            temp.extend_from_slice(&[0; 4]);
            return temp.pop().map(Result::Ok);
        }
        let bytes = [
            next_byte,
            self.inner.next_non_whitespace(),
            self.inner.next_non_whitespace(),
            self.inner.next_non_whitespace(),
            self.inner.next_non_whitespace(),
        ];

        if let Some(Some(Err(e))) = bytes
            .iter()
            .find(|b| b.as_ref().is_some_and(|bb| bb.is_err()))
        {
            return Some(Err(e.clone()));
        };
        let mut bytes = bytes.map(|b| b.unwrap_or(Ok(b'u')).unwrap());
        let n = if let Some(tilde) = bytes.iter().position(|b| *b == b'~') {
            let nb = if tilde == 4 {
                let next = self.inner.next_non_whitespace();
                if next.is_none() {
                    return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                        FilterError::MissingEOD,
                    ))));
                }
                if next.as_ref().is_some_and(|r| r.is_err()) {
                    return next;
                };
                next.unwrap().unwrap()
            } else {
                bytes[tilde + 1]
            };
            if nb == b'>' {
                self.done = true;
                if tilde == 0 {
                    return None;
                }
                bytes = bytes.map(|b| if b"~>".contains(&b) { b'u' } else { b });
            } else {
                return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                    FilterError::MissingEOD,
                ))));
            }
            tilde
        } else {
            5
        };
        if bytes
            .iter()
            .any(|b| !(b'!'..=b'u').contains(b) && *b != b'z')
        {
            return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                FilterError::ASCII85Decode,
            ))));
        }
        let bytes = bytes.map(|b| (b - b'!') as u32);
        let total = (((bytes[0] * 85 + bytes[1]) * 85 + bytes[2]) * 85 + bytes[3]) * 85 + bytes[4];
        let bytes = total.to_be_bytes();
        for i in 0..(n - 1) {
            temp.push(bytes[n - 2 - i]);
        }
        temp.pop().map(Result::Ok)
    }
}

#[cfg(test)]
mod test {
    use crate::{PdfDict, filter::Filter, pdf_error::PdfResult};
    use core::str;

    #[test]
    fn encode_ascii85() {
        let examples: &[(&[u8], &[u8])] = &[
            (b"DCODE", b"6pja<70~>"),
            (
                b"testougfhAbIEf'pEJgf[IF[jgroho;iug;aoefh{Fjo;sirghl/sjrhgKGR",
                b"FCfN8Dfo])BL-*!7:mod77q3)>=h:gC1hR*BQ#tHF_<dEDe3NoHU_ag4*,+VB4u*7F)#`/B1k`m~>",
            ),
            (b"\x00\x00\x00\x00", b"z~>"),
            (b"a\x00\x00\x00\x00", b"@/p9-!!~>"),
            (b"a\x00\x00\x00\x00\x00", b"@/p9-!!!~>"),
            (b"a\x00\x00\x00\x00\x00\x00", b"@/p9-!!!!~>"),
        ];

        for (i, o) in examples {
            let encoded =
                Filter::ASCII85.encode(i.iter().copied().map(Result::Ok), PdfDict::empty());
            let res: PdfResult<Vec<_>> = encoded.collect();
            assert!(res.is_ok());
            unsafe {
                assert_eq!(
                    res.clone().unwrap().as_slice(),
                    *o,
                    "{} => {}",
                    str::from_utf8_unchecked(i),
                    str::from_utf8_unchecked(res.unwrap().as_slice())
                );
            }
        }
    }

    #[test]
    fn decode_ascii85() {
        let examples: &[(&[u8], &[u8])] = &[
            (b"DCODE", b"6pja<70~>"),
            (
                b"testougfhAbIEf'pEJgf[IF[jgroho;iug;aoefh{Fjo;sirghl/sjrhgKGR",
                b"FCfN8Dfo])BL-*!7:mod77q3)>=h:gC1hR*BQ#tHF_<dEDe3NoHU_ag4*,+VB4u*7F)#`/B1k`m~>",
            ),
            (b"\x00\x00\x00\x00", b"z~>"),
            (b"a\x00\x00\x00\x00", b"@/p9-!!~>"),
            (b"a\x00\x00\x00\x00\x00", b"@/p9-!!!~>"),
            (b"a\x00\x00\x00\x00\x00\x00", b"@/p9-!!!!~>"),
        ];

        for (i, o) in examples {
            let decoded =
                Filter::ASCII85.decode(o.iter().copied().map(Result::Ok), PdfDict::empty());
            let res: PdfResult<Vec<_>> = decoded.collect();
            assert!(res.is_ok());
            unsafe {
                assert_eq!(
                    res.clone().unwrap().as_slice(),
                    *i,
                    "{} => {}",
                    str::from_utf8_unchecked(res.unwrap().as_slice()),
                    str::from_utf8_unchecked(i),
                );
            }
        }
    }

    #[test]
    fn chain_ascii85() {
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
                Filter::ASCII85.encode(e.iter().copied().map(Result::Ok), PdfDict::empty());
            let decoded = Filter::ASCII85.decode(encoded, PdfDict::empty());
            let c = decoded.collect::<Result<Vec<_>, _>>();
            assert!(c.is_ok());
            assert_eq!(c.unwrap().as_slice(), e);
        }
    }
}
