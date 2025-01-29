use core::str;

use crate::{
    pdf_error::{PdfError, PdfErrorKind},
    Parsable,
};

use super::is_regular;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PdfName {
    data: Vec<u8>,
}

impl PdfName {
    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        Self { data: bytes.into() }
    }
}

impl Parsable for PdfName {
    fn from_bytes(mut bytes: &[u8]) -> Result<(Self, &[u8]), crate::pdf_error::PdfError> {
        let err = Err(PdfError::with_kind(PdfErrorKind::ParseError));
        if !matches!(bytes.get(0), Some(b'/')) {
            return err;
        }
        bytes = &bytes[1..];
        let mut data = Vec::new();
        while let [byte, rest @ ..] = bytes {
            let mut rest = rest;
            let n = match *byte {
                0 => {
                    return Err(PdfError::with_kind(PdfErrorKind::ParseError));
                }
                b'#' => {
                    if let [l, r, rrest @ ..] = rest {
                        if l.is_ascii_hexdigit() && r.is_ascii_hexdigit() {
                            rest = rrest;
                            u8::from_str_radix(unsafe { str::from_utf8_unchecked(&[*l, *r]) }, 16)?
                        } else {
                            return err;
                        }
                    } else {
                        return err;
                    }
                }
                b if is_regular(b) => b,
                _ => break,
            };
            data.push(n);
            bytes = rest;
        }
        Ok((Self { data }, bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, pdf_types::PdfName};

    #[test]
    fn parsing() {
        let examples = [
            "/Name1",
            "/ASomeWhatLongerName",
            "/A;Name_With-Various***Characters?",
            "/1.2",
            "/$$",
            "/@pattern",
            "/.notdef",
            "/Lime#20Green",
            "/paired#28#29parentheses",
            "/The_Key_of_F#23_Minor",
            "/A#42",
        ];
        let expected = [
            "Name1",
            "ASomeWhatLongerName",
            "A;Name_With-Various***Characters?",
            "1.2",
            "$$",
            "@pattern",
            ".notdef",
            "Lime Green",
            "paired()parentheses",
            "The_Key_of_F#_Minor",
            "AB",
        ];

        for e in examples.into_iter().zip(expected.map(|s| PdfName {
            data: Vec::from_iter(s.as_bytes().iter().copied()),
        })) {
            let parsed = parse(e.0.as_bytes());
            assert_eq!(parsed, Ok((e.1, &[] as &[u8])))
        }
    }
    #[test]
    fn delimiters() {
        let parsed = parse("/Name]".as_bytes());
        assert_eq!(
            parsed,
            Ok((PdfName::from_raw_bytes("Name".as_bytes()), &[b']'] as &[u8]))
        )
    }
}
