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
    pub fn encode<'a, I: IntoIterator<Item = Result<u8, PdfError>>>(
        &self,
        bytes: I,
        filter_params: &'a PdfDict,
    ) -> Filtered<'a, I::IntoIter> {
        let data = match self {
            PdfFilter::ASCIIHex => FilteredData::Temp(None),
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
            decoding: false,
            data,
            iter: bytes.into_iter(),
        }
    }

    pub fn decode<'a, I: IntoIterator<Item = Result<u8, PdfError>>>(
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

impl<'a, I: Iterator<Item = Result<u8, PdfError>>> Filtered<'a, I> {
    fn decode_next_ascii_hex(&mut self) -> Option<Result<u8, PdfError>> {
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

    fn _next_ascii_digit(&mut self) -> Option<Result<u8, PdfError>> {
        while let Some(b) = self.iter.next() {
            let Ok(b) = b else { return Some(b) };
            if b.is_ascii_digit() {
                return Some(Ok(b));
            }
        }
        None
    }
    fn next_non_whitespace(&mut self) -> Option<Result<u8, PdfError>> {
        while let Some(b) = self.iter.next() {
            let Ok(b) = b else { return Some(b) };
            if !WHITESPACES.contains(&b) {
                return Some(Ok(b));
            }
        }
        None
    }

    fn encode_next_ascii_hex(&mut self) -> Option<Result<u8, PdfError>> {
        let temp = self.data.as_temp_mut();
        if temp.is_some_and(|b| b != b'>') {
            return temp.take().map(Result::Ok);
        }
        match self.iter.next() {
            Some(Ok(b)) => {
                let s = format!("{b:02X}");
                let s = s.as_bytes();
                *temp = Some(s[1]);
                Some(Ok(s[0]))
            }
            Some(Err(e)) => return Some(Err(e)),
            None => match temp {
                Some(_) => None,
                None => {
                    *temp = Some(b'>');
                    Some(Ok(b'>'))
                }
            },
        }
    }
}

impl<'a, I: Iterator<Item = Result<u8, PdfError>>> Iterator for Filtered<'a, I> {
    type Item = Result<u8, PdfError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.decoding {
            match self.filter {
                PdfFilter::ASCIIHex => self.decode_next_ascii_hex(),
                PdfFilter::ASCII85 => todo!(),
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
                PdfFilter::ASCII85 => todo!(),
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
        #[derive(Debug, Clone, Copy)]
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

enum_cast!(FilteredData, EOD bool, Temp Option<u8>);

#[cfg(test)]
mod tests {
    use super::PdfFilter;
    use crate::PdfDict;

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
}
