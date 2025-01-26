use std::str::FromStr;

use crate::pdf_error::{PdfError, PdfErrorKind};

#[derive(PartialEq, Debug)]
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
                    kind: PdfErrorKind::ParseError,
                })
            }
        } else {
            Err(PdfError {
                kind: PdfErrorKind::ParseError,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let err = Err(PdfError {
            kind: PdfErrorKind::ParseError,
        });
        let test_data: [(&str, PdfNumeric); 11] = [
            ("123", 123.into()),
            ("43445", 43445.into()),
            ("+17", 17.into()),
            ("-98", (-98).into()),
            ("0", 0.into()),
            ("34.5", 34.5.into()),
            ("-3.62", (-3.62).into()),
            ("+123.6", 123.6.into()),
            ("4.", 4.0.into()),
            ("-.002", (-0.002).into()),
            ("0.0", 0.0.into()),
        ];

        for (s, r) in test_data {
            assert_eq!(s.parse(), Ok(r));
        }
        let err_data = ["b", "a", "3.0e1", "16#FFFE"];
        for s in err_data {
            assert_eq!(s.parse::<PdfNumeric>(), err);
        }
    }
}
