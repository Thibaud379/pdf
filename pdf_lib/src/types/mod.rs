mod parsable;
mod pdf_array;
mod pdf_dict;
mod pdf_name;
mod pdf_null;
mod pdf_num;
mod pdf_str;
mod pdf_stream;
use crate::{PdfError, PdfErrorKind};
pub use parsable::*;
use paste::paste;
pub use pdf_array::*;
pub use pdf_dict::*;
pub use pdf_name::*;
pub use pdf_null::*;
pub use pdf_num::*;
pub use pdf_str::*;
pub use pdf_stream::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndirectData {
    object: usize,
    generation: usize,
}
#[derive(Debug, PartialEq, Clone)]
pub struct PdfObject {
    kind: PdfObjectKind,
    indirect: Option<IndirectData>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PdfObjectKind {
    Boolean(bool),
    Numeric(PdfNumeric),
    String(PdfString),
    Name(PdfName),
    Array(PdfArray),
    Dict(PdfDict),
    Stream(PdfStream),
    Null(PdfNull),
    Ref,
}

macro_rules! from_impl {
    ($F:ty,$Var:tt) => {
        impl From<$F> for PdfObjectKind {
            fn from(value: $F) -> Self {
                Self::$Var(value)
            }
        }
        impl From<$F> for PdfObject {
            fn from(value: $F) -> Self {
                Self {
                    kind: value.into(),
                    indirect: None,
                }
            }
        }
    };
}

from_impl!(bool, Boolean); //
from_impl!(PdfNumeric, Numeric);
from_impl!(PdfString, String); //
from_impl!(PdfName, Name); //
from_impl!(PdfArray, Array); //
from_impl!(PdfDict, Dict); //
from_impl!(PdfStream, Stream); //
from_impl!(PdfNull, Null); //

macro_rules! as_indirect_impl {
    ($($F:ty)+) => {$(
        impl $F {
            pub fn as_indirect_raw(&self, object: usize, generation: usize) -> PdfObject {
                let mut o: PdfObject = self.clone().into();
                o.indirect = Some(IndirectData { object, generation });
                o
            }
            pub fn as_indirect(&self, indirect: IndirectData) -> PdfObject {
                let mut o: PdfObject = self.clone().into();
                o.indirect = Some(indirect);
                o
            }
        }
    )+    };
}
as_indirect_impl!(
    PdfNumeric PdfString PdfName PdfArray PdfDict PdfStream PdfNull PdfObject
);

macro_rules! as_kind_impl {
    ($($K:ty)+) => {
        impl PdfObject {$(
            paste!{
                pub fn [<as_ $K:lower>](self) -> Result<[<Pdf $K>],PdfError> {
                    match self.kind{
                        PdfObjectKind::$K(inner) => Ok(inner),
                        _=>Err(PdfError::with_kind(PdfErrorKind::WrongType)),
                    }
                }
                pub fn [<as_ $K:lower _ref>](&self) -> Result<&[<Pdf $K>],PdfError> {
                    match &self.kind{
                        PdfObjectKind::$K(inner) => Ok(inner),
                        _=>Err(PdfError::with_kind(PdfErrorKind::WrongType)),
                    }
                }
            }
        )+}
    };
}
as_kind_impl!(
    Numeric String Name Array Dict Stream Null
);

pub(crate) const WHITESPACES: [u8; 6] = *b"\x00\t\n\x0c\r ";
pub(crate) const EOLS: [u8; 2] = [b'\n', b'\r'];
pub(crate) const DELIMITERS: [u8; 10] = *b"()<>[]{}/%";
pub(crate) fn is_regular(byte: u8) -> bool {
    !WHITESPACES.contains(&byte) && !DELIMITERS.contains(&byte)
}
