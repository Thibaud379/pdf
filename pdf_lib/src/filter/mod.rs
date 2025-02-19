use core::str;

use super::{PdfDict, PdfName, WHITESPACES};
use crate::pdf_error::*;

use ascii85::*;
use asciihex::*;
mod ascii85;
mod asciihex;

#[derive(Clone, PartialEq, Debug)]
pub enum FilterError {
    ASCIIHexDecode(u8),
    MissingEOD,
    ASCII85Decode,
}

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
filter_impl!(Filter,
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

pub struct FilterData<I> {
    iter: I,
    #[allow(dead_code)]
    params: PdfDict,
}

impl<I: Iterator<Item = PdfResult<u8>>> Iterator for FilterData<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
pub enum Encode<I> {
    ASCIIHex(EncodeASCIIHex<I>),
    ASCII85(EncodeASCII85<I>),
}

impl<I: FilterIter> Iterator for Encode<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Encode::ASCIIHex(inner) => inner.next(),
            Encode::ASCII85(inner) => inner.next(),
        }
    }
}
pub enum Decode<I> {
    ASCIIHex(DecodeASCIIHex<I>),
    ASCII85(DecodeASCII85<I>),
}
impl<I: FilterIter> Iterator for Decode<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Decode::ASCIIHex(inner) => inner.next(),
            Decode::ASCII85(inner) => inner.next(),
        }
    }
}

impl Filter {
    pub fn encode<I: IntoIterator<Item = PdfResult<u8>>>(
        &self,
        bytes: I,
        params: PdfDict,
    ) -> Encode<I::IntoIter> {
        let inner = FilterData {
            iter: bytes.into_iter(),
            params,
        };
        match self {
            Filter::ASCIIHex => Encode::ASCIIHex(EncodeASCIIHex::new(inner)),
            Filter::ASCII85 => Encode::ASCII85(EncodeASCII85::new(inner)),
            Filter::LZW => todo!(),
            Filter::Flate => todo!(),
            Filter::RunLength => todo!(),
            Filter::CCITTFax => todo!(),
            Filter::JBIG2 => todo!(),
            Filter::DCT => todo!(),
            Filter::JPX => todo!(),
            Filter::Crypt => todo!(),
        }
    }

    pub fn decode<I: IntoIterator<Item = PdfResult<u8>>>(
        &self,
        bytes: I,
        params: PdfDict,
    ) -> Decode<I::IntoIter> {
        let inner = FilterData {
            iter: bytes.into_iter(),
            params,
        };
        match self {
            Filter::ASCIIHex => Decode::ASCIIHex(DecodeASCIIHex::new(inner)),
            Filter::ASCII85 => Decode::ASCII85(DecodeASCII85::new(inner)),
            Filter::LZW => todo!(),
            Filter::Flate => todo!(),
            Filter::RunLength => todo!(),
            Filter::CCITTFax => todo!(),
            Filter::JBIG2 => todo!(),
            Filter::DCT => todo!(),
            Filter::JPX => todo!(),
            Filter::Crypt => todo!(),
        }
    }
}

trait FilterIter: Iterator<Item = PdfResult<u8>> {
    fn next_non_whitespace(&mut self) -> Option<PdfResult<u8>>;
}

impl<I> FilterIter for I
where
    I: Iterator<Item = PdfResult<u8>>,
{
    fn next_non_whitespace(&mut self) -> Option<PdfResult<u8>> {
        while let Some(b) = self.next() {
            let Ok(b) = b else { return Some(b) };
            if !WHITESPACES.contains(&b) {
                return Some(Ok(b));
            }
        }
        None
    }
}
