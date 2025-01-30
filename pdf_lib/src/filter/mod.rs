use super::PdfName;
use crate::pdf_error::*;

macro_rules! filter_impl {
    ($Name:tt, $($Var:tt $raw:literal $($Param:literal)*),+) => {
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
