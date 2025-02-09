use core::str;
use std::str::FromStr;

use crate::{
    Parsable,
    pdf_error::{PdfError, PdfErrorKind},
};

use super::WHITESPACES;

#[derive(PartialEq, Debug, Clone)]
pub enum PdfNumeric {
    PdfInt(i32),
    PdfReal(f32),
}

impl From<PdfNumeric> for i32 {
    fn from(value: PdfNumeric) -> Self {
        match value {
            PdfNumeric::PdfInt(v) => v,
            PdfNumeric::PdfReal(f) => f as i32,
        }
    }
}
impl From<&PdfNumeric> for i32 {
    fn from(value: &PdfNumeric) -> Self {
        match value {
            PdfNumeric::PdfInt(v) => *v,
            PdfNumeric::PdfReal(f) => *f as i32,
        }
    }
}
impl From<i32> for PdfNumeric {
    fn from(value: i32) -> Self {
        Self::PdfInt(value)
    }
}
impl From<f32> for PdfNumeric {
    fn from(value: f32) -> Self {
        Self::PdfReal(value)
    }
}
impl FromStr for PdfNumeric {
    type Err = PdfError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(p) = value.parse::<i32>() {
            Ok(Self::PdfInt(p))
        } else if let Ok(p) = value.parse::<f32>() {
            // No PostScript and no exponential notation
            if !value.contains('e') && !value.contains('#') {
                Ok(Self::PdfReal(p))
            } else {
                Err(PdfError {
                    kind: PdfErrorKind::Parse,
                })
            }
        } else {
            Err(PdfError {
                kind: PdfErrorKind::Parse,
            })
        }
    }
}

impl Parsable for PdfNumeric {
    fn from_bytes(b: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        let first_token = b
            .split(|b| WHITESPACES.contains(b))
            .next()
            .ok_or_else(|| PdfError::with_kind(PdfErrorKind::Parse))?;
        let parsed = str::from_utf8(first_token)?.parse()?;
        Ok((parsed, &b[first_token.len()..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn parsing_str() {
        let err = Err(PdfError {
            kind: PdfErrorKind::Parse,
        });
        let test_data: [(&str, PdfNumeric); 13] = [
            ("123", 123.into()),
            ("43445", 43445.into()),
            ("+17", 17.into()),
            ("-98", (-98).into()),
            ("0", 0.into()),
            ("00987", 987.into()),
            ("34.5", 34.5.into()),
            ("-3.62", (-3.62).into()),
            ("+123.6", 123.6.into()),
            ("4.", 4.0.into()),
            ("-.002", (-0.002).into()),
            ("0.0", 0.0.into()),
            ("009.87", 9.87.into()),
        ];

        for (s, r) in test_data {
            assert_eq!(s.parse(), Ok(r));
        }
        let err_data = ["b", "a", "3.0e1", "16#FFFE"];
        for s in err_data {
            assert_eq!(s.parse::<PdfNumeric>(), err);
        }
    }
    #[test]
    fn parsing_bytes() {
        let err = Err(PdfError {
            kind: PdfErrorKind::Parse,
        });
        let test_data: [(&[u8], PdfNumeric); 13] = [
            (b"123", 123.into()),
            (b"43445", 43445.into()),
            (b"+17", 17.into()),
            (b"-98", (-98).into()),
            (b"0", 0.into()),
            (b"00987", 987.into()),
            (b"34.5", 34.5.into()),
            (b"-3.62", (-3.62).into()),
            (b"+123.6", 123.6.into()),
            (b"4.", 4.0.into()),
            (b"-.002", (-0.002).into()),
            (b"0.0", 0.0.into()),
            (b"009.87", 9.87.into()),
        ];

        for (s, r) in test_data {
            assert_eq!(parse(s), Ok((r, &[] as &[u8])));
        }
        let err_data: [&[u8]; 4] = [b"b", b"a", b"3.0e1", b"16#FFFE"];
        for s in err_data {
            assert_eq!(parse::<PdfNumeric>(s), err);
        }
    }
    #[test]
    fn parsing_rest() {
        assert_eq!(
            parse::<PdfNumeric>(b"123 d"),
            Ok((123.into(), b" d" as &[u8]))
        );
        assert_eq!(
            parse::<PdfNumeric>(b"123\n"),
            Ok((123.into(), b"\n" as &[u8]))
        );
        assert_eq!(
            parse::<PdfNumeric>(b"123 "),
            Ok((123.into(), b" " as &[u8]))
        );
        assert_eq!(
            parse::<PdfNumeric>(b"0.4\n/"),
            Ok((0.4.into(), b"\n/" as &[u8]))
        );
    }
}
