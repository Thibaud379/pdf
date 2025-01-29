use crate::{
    Parsable, parse,
    pdf_error::{PdfError, PdfErrorKind},
};

use super::{PdfObject, strip_whitespace};

#[derive(Debug, PartialEq, Clone)]
pub struct PdfArray {
    data: Vec<PdfObject>,
}

impl Parsable for PdfArray {
    fn from_bytes(mut bytes: &[u8]) -> Result<(Self, &[u8]), crate::pdf_error::PdfError> {
        if !matches!(bytes.get(0), Some(b'[')) {
            return Err(PdfError::with_kind(PdfErrorKind::ParseError));
        }
        bytes = &bytes[1..];
        let mut data = Vec::new();
        while let Ok((obj, b)) = parse::<PdfObject>(bytes) {
            data.push(obj);
            bytes = strip_whitespace(b);
        }
        if !matches!(bytes.get(0), Some(b']')) {
            return Err(PdfError::with_kind(PdfErrorKind::ParseError));
        }
        bytes = &bytes[1..];
        Ok((Self { data }, bytes))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parse,
        types::{PdfName, PdfNumeric, PdfString},
    };

    use super::PdfArray;

    #[test]
    fn parsing() {
        let examples = ["[549 3.14 true (Ralph) /SomeName null 12 0 obj <FF> endobj]"];
        let expected = [PdfArray {
            data: vec![
                PdfNumeric::PdfInt(549).into(),
                PdfNumeric::PdfReal(3.14).into(),
                true.into(),
                PdfString::from_raw_bytes(b"Ralph").into(),
                PdfName::from_raw_bytes(b"SomeName").into(),
                None.into(),
                PdfString::from_raw_bytes(&[0xff]).as_indirect_raw(12, 0),
            ],
        }];

        for (e, o) in examples.into_iter().zip(expected) {
            let parsed = parse(e.as_bytes());
            assert_eq!(parsed, Ok((o, &[] as &[u8])))
        }
    }
}
