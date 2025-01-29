use pdf_types::{PdfName, PdfNumeric, PdfString, EOLS, WHITESPACES};

use crate::pdf_error::{PdfError, PdfErrorKind};

mod pdf_error;
mod pdf_file;
mod pdf_types;

pub use pdf_file::PdfFile;

#[allow(dead_code)]
mod pdf_constants {
    const LINE_WIDTH: usize = 255;
    const EOL: &str = "\r\n";
}

pub trait Parsable: Sized {
    fn from_bytes(b: &[u8]) -> Result<(Self, &[u8]), PdfError>;
}

pub fn parse<T>(bytes: &[u8]) -> Result<(T, &[u8]), PdfError>
where
    T: Parsable,
{
    T::from_bytes(bytes)
}

impl Parsable for bool {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        match bytes.len() {
            ..4 => None,
            4 => (bytes == b"true").then_some((true, &bytes[4..4])),
            5.. => (&bytes[..5] == b"false")
                .then_some((false, &bytes[5..]))
                .or_else(|| (&bytes[..4] == b"true").then_some((true, &bytes[3..]))),
        }
        .and_then(|r| match r.1.len() {
            0 => Some(r),
            _ => WHITESPACES.contains(&r.1[0]).then_some(r),
        })
        .ok_or_else(|| PdfError {
            kind: PdfErrorKind::ParseError,
        })
    }
}

struct Whitespace {
    _bytes: Vec<u8>,
}
impl Parsable for Whitespace {
    fn from_bytes(mut bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        let mut data = Vec::new();
        while bytes.get(0).is_some_and(|b| WHITESPACES.contains(b)) {
            data.push(bytes[0]);
            bytes = &bytes[1..];
        }
        Ok((Self { _bytes: data }, bytes))
    }
}

fn strip_whitespace(bytes: &[u8]) -> &[u8] {
    parse::<Whitespace>(bytes).unwrap().1
}

fn next_eol(mut bytes: &[u8]) -> &[u8] {
    while bytes.get(0).is_some_and(|b| !EOLS.contains(b)) {
        bytes = &bytes[1..];
    }
    match bytes {
        [b'\r', b'\n', ..] => &bytes[2..],
        [_b, ..] => &bytes[1..],
        [] => bytes,
    }
}

pub enum PdfObject {
    Boolean(bool),
    Numeric(PdfNumeric),
    String(PdfString),
    Name(PdfName),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn parsing() {
        let valid: [&[u8]; 3] = [b"true", b"false", b"false "];
        let expected = [(true, 4), (false, 5), (false, 5)];
        for (bytes, (res, rest)) in valid.into_iter().zip(expected) {
            let parsed = parse(bytes);
            assert_eq!(parsed, Ok((res, &bytes[rest..])))
        }

        let invalid: [&[u8]; 3] = [b"True", b"fal se", b"false\\"];
        for bytes in invalid {
            assert!(parse::<bool>(bytes).is_err())
        }
    }
    #[test]
    fn size_test() {
        println!("PdfObject({}B)", std::mem::size_of::<PdfObject>());
        println!("bool({}B)", std::mem::size_of::<bool>());
        println!("PdfNumeric({}B)", std::mem::size_of::<PdfNumeric>());
        println!("PdfString({}B)", std::mem::size_of::<PdfString>());
        println!("PdfName({}B)", std::mem::size_of::<PdfName>());
    }
}
