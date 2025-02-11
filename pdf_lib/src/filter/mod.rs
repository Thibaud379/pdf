use core::str;
use paste::paste;

use super::{PdfDict, PdfName, WHITESPACES};
use crate::pdf_error::*;

macro_rules! filter_impl {
    ($Name:tt, $($Var:tt $raw:literal $($Param:literal)*),+) => {
        #[derive(Debug, Clone, Copy)]
        pub enum $Name{
            $($Var),+
        }

        impl $Name{
            pub fn valid_params(&self)->Vec<PdfName>{
                match self{
                    $(Self::$Var=>vec![$(PdfName::from_raw_bytes($Param),)*],)+
                }
            }
        }

        impl TryFrom<&str> for $Name{
            type Error = PdfError;
            fn try_from(value:&str)->Result<Self,Self::Error>{
                match value.as_bytes(){
                    $($raw => Ok(Self::$Var),)+
                    _=> Err(PdfError::with_kind(PdfErrorKind::InvalidData))
                }
            }
        }
        impl TryFrom<&PdfName> for $Name{
            type Error = PdfError;
            fn try_from(value:&PdfName)->Result<Self,Self::Error>{
                match value.bytes(){
                    $(
                        $raw => Ok(Self::$Var),
                    )+
                    _=> Err(PdfError::with_kind(PdfErrorKind::InvalidData))
                }
            }
        }

        impl From<&$Name> for PdfName{
            fn from(value:&$Name)->Self{
                match value{
                    $($Name::$Var => PdfName::from_raw_bytes($raw),)+
                }
            }
        }
    };
}
filter_impl!(PdfFilter,
    ASCIIHex b"ASCIIHexDecode",
    ASCII85 b"ASCII85Decode",
    LZW b"LZWDecode" b"Predictor" b"Colors" b"BitsPerComponent" b"Columns" b"EarlyChange",
    Flate b"FlateDecode" b"Predictor" b"Colors" b"BitsPerComponent" b"Columns",
    RunLength b"RunLengthDecode",
    CCITTFax b"CCITTFaxDecode" b"K" b"EndOfLine" b"EncodeByteAlign" b"Columns" b"Rows" b"EndOfBlock" b"BlackIs1" b"DamagedRowsBeforeError",
    JBIG2 b"JBIG2Decode" b"JBIG2Globals",
    DCT b"DCTDecode" b"ColorTransform",
    JPX b"JPXDecode",
    Crypt b"CryptDecode" b"Type" b"Name"
);

impl PdfFilter {
    pub fn encode<'a, I: IntoIterator<Item = PdfResult<u8>>>(
        &self,
        bytes: I,
        filter_params: &'a PdfDict,
    ) -> Filtered<'a, I::IntoIter> {
        let data = match self {
            PdfFilter::ASCIIHex => FilteredData::Temp(Vec::new()),
            PdfFilter::ASCII85 => FilteredData::Temp(Vec::new()),
            PdfFilter::LZW => todo!(),
            PdfFilter::Flate => todo!(),
            PdfFilter::RunLength => todo!(),
            PdfFilter::CCITTFax => todo!(),
            PdfFilter::JBIG2 => todo!(),
            PdfFilter::DCT => todo!(),
            PdfFilter::JPX => todo!(),
            PdfFilter::Crypt => todo!(),
        };
        Filtered {
            filter: self.clone(),
            params: filter_params,
            decoding: false,
            data,
            iter: bytes.into_iter(),
        }
    }

    pub fn decode<'a, I: IntoIterator<Item = PdfResult<u8>>>(
        &self,
        bytes: I,
        filter_params: &'a PdfDict,
    ) -> Filtered<'a, I::IntoIter> {
        let data = match self {
            PdfFilter::ASCIIHex => FilteredData::EOD(false),
            PdfFilter::ASCII85 => todo!(),
            PdfFilter::LZW => todo!(),
            PdfFilter::Flate => todo!(),
            PdfFilter::RunLength => todo!(),
            PdfFilter::CCITTFax => todo!(),
            PdfFilter::JBIG2 => todo!(),
            PdfFilter::DCT => todo!(),
            PdfFilter::JPX => todo!(),
            PdfFilter::Crypt => todo!(),
        };
        Filtered {
            filter: self.clone(),
            params: filter_params,
            decoding: true,
            data,
            iter: bytes.into_iter(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum FilterError {
    ASCIIHexDecode(u8),
    MissingEOD,
}

pub struct Filtered<'a, I> {
    filter: PdfFilter,
    iter: I,
    #[allow(dead_code)]
    params: &'a PdfDict,
    decoding: bool,
    data: FilteredData,
}

impl<'a, I: Iterator<Item = PdfResult<u8>>> Filtered<'a, I> {
    fn decode_next_ascii_hex(&mut self) -> Option<PdfResult<u8>> {
        if *self.data.as_eod() {
            return None;
        }
        let first_byte = match self.next_non_whitespace() {
            Some(Ok(b)) if b.is_ascii_hexdigit() => b,
            Some(Ok(b)) if b == b'>' => {
                *self.data.as_eod_mut() = true;
                return None;
            }
            Some(Ok(b)) => {
                return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                    FilterError::ASCIIHexDecode(b),
                ))));
            }
            None => {
                if !self.data.as_eod() {
                    return Some(Err(PdfError::with_kind(PdfErrorKind::Filter(
                        FilterError::MissingEOD,
                    ))));
                } else {
                    return None;
                }
            }
            o => return o,
        };

        let second_byte = match self.next_non_whitespace() {
            Some(Ok(b)) if b.is_ascii_hexdigit() => Some(b),
            Some(Ok(b)) if b == b'>' => {
                *self.data.as_eod_mut() = true;
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

    fn _next_ascii_digit(&mut self) -> Option<PdfResult<u8>> {
        while let Some(b) = self.iter.next() {
            let Ok(b) = b else { return Some(b) };
            if b.is_ascii_digit() {
                return Some(Ok(b));
            }
        }
        None
    }
    fn next_non_whitespace(&mut self) -> Option<PdfResult<u8>> {
        while let Some(b) = self.iter.next() {
            let Ok(b) = b else { return Some(b) };
            if !WHITESPACES.contains(&b) {
                return Some(Ok(b));
            }
        }
        None
    }
    fn next_85(&mut self) -> Option<PdfResult<u8>> {
        while let Some(b) = self.iter.next() {
            let Ok(b) = b else { return Some(b) };
            if (b'!'..b'u').contains(&b) || b"~>z".contains(&b) {
                return Some(Ok(b));
            }
        }
        None
    }

    fn encode_next_ascii_hex(&mut self) -> Option<PdfResult<u8>> {
        let temp = self.data.as_temp_mut();
        if temp.first().is_some_and(|b| *b != b'>') {
            return temp.pop().map(Result::Ok);
        }
        match self.iter.next() {
            Some(Ok(b)) => {
                let s = format!("{b:02X}");
                let s = s.as_bytes();
                temp.push(s[1]);
                Some(Ok(s[0]))
            }
            Some(Err(e)) => return Some(Err(e)),
            None => match temp[..] {
                [_, ..] => None,
                _ => {
                    temp.push(b'>');
                    Some(Ok(b'>'))
                }
            },
        }
    }

    fn decode_next_ascii85(&mut self) -> Option<PdfResult<u8>> {
        let mut bytes = [
            self.next_non_whitespace(),
            self.next_non_whitespace(),
            self.next_non_whitespace(),
            self.next_non_whitespace(),
            self.next_non_whitespace(),
        ];

        // if bytes.iter().all(Option::is_some) {}else {

        // }
        todo!("DECODE")
    }

    fn encode_next_ascii85(&mut self) -> Option<PdfResult<u8>> {
        {
            let temp = self.data.as_temp_mut();
            if !temp.is_empty() {
                return match temp.pop() {
                    Some(0) => {
                        temp.push(0);
                        None
                    }
                    Some(b) => Some(Ok(b)),
                    None => unreachable!(),
                };
            }
        }
        let mut bytes = [
            self.iter.next(),
            self.iter.next(),
            self.iter.next(),
            self.iter.next(),
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
            self.data.as_temp_mut().push(0);
            self.data.as_temp_mut().push(b'>');
            self.data.as_temp_mut().push(b'~');
        }
        if n != 0 {
            self.data.as_temp_mut().extend_from_slice(o);
        }
        self.data.as_temp_mut().pop().map(Result::Ok)
    }
}

impl<'a, I: Iterator<Item = PdfResult<u8>>> Iterator for Filtered<'a, I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.decoding {
            match self.filter {
                PdfFilter::ASCIIHex => self.decode_next_ascii_hex(),
                PdfFilter::ASCII85 => self.decode_next_ascii85(),
                PdfFilter::LZW => todo!(),
                PdfFilter::Flate => todo!(),
                PdfFilter::RunLength => todo!(),
                PdfFilter::CCITTFax => todo!(),
                PdfFilter::JBIG2 => todo!(),
                PdfFilter::DCT => todo!(),
                PdfFilter::JPX => todo!(),
                PdfFilter::Crypt => todo!(),
            }
        } else {
            match self.filter {
                PdfFilter::ASCIIHex => self.encode_next_ascii_hex(),
                PdfFilter::ASCII85 => self.encode_next_ascii85(),
                PdfFilter::LZW => todo!(),
                PdfFilter::Flate => todo!(),
                PdfFilter::RunLength => todo!(),
                PdfFilter::CCITTFax => todo!(),
                PdfFilter::JBIG2 => todo!(),
                PdfFilter::DCT => todo!(),
                PdfFilter::JPX => todo!(),
                PdfFilter::Crypt => todo!(),
            }
        }
    }
}

macro_rules! enum_cast {
    ($Name:ident, $($Var:ident $Inner:ty),+) => {
        #[derive(Debug, Clone)]
        #[allow(dead_code)]
        pub enum $Name{
            NoData,
            $($Var($Inner)),+
        }

        #[allow(dead_code)]
        impl $Name{
            $(
            paste!{
                    pub fn [<as_ $Var:lower _mut>](&mut self)->&mut $Inner{
                        match self{
                            $Name::$Var(i) => i,
                            _=>panic!()
                        }
                    }
                    pub fn [<as_ $Var:lower>](&self)->&$Inner{
                        match self{
                            $Name::$Var(i) => i,
                            _=>panic!()
                        }
                    }
                }
            )+
        }
    };
}

enum_cast!(FilteredData, EOD bool, Temp Vec<u8>);

#[cfg(test)]
mod tests {
    use core::str;

    use super::PdfFilter;
    use crate::{PdfDict, pdf_error::PdfResult};

    #[test]
    fn encode_ascii_hex() {
        let examples: [&[u8]; 3] = [b"test", b"\x12\xFF", b" \n0\x00"];
        let expected: [&[u8]; 3] = [b"74657374>", b"12FF>", b"200A3000>"];
        for (example, exp) in examples.into_iter().zip(expected) {
            let encoded: Result<Vec<_>, _> = PdfFilter::ASCIIHex
                .encode(example.iter().copied().map(Result::Ok), &PdfDict::empty())
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
            let decoded: Result<Vec<_>, _> = PdfFilter::ASCIIHex
                .decode(example.iter().copied().map(Result::Ok), &PdfDict::empty())
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
        let d = PdfDict::empty();
        for e in examples {
            let encoded = PdfFilter::ASCIIHex.encode(e.iter().copied().map(Result::Ok), &d);
            let decoded = PdfFilter::ASCIIHex.decode(encoded, &d);
            let c = decoded.collect::<Result<Vec<_>, _>>();
            assert!(c.is_ok());
            assert_eq!(c.unwrap().as_slice(), e);
        }
    }

    #[test]
    fn encode_ascii85() {
        let d = PdfDict::empty();
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
            let encoded = PdfFilter::ASCII85.encode(i.iter().copied().map(Result::Ok), &d);
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
}
