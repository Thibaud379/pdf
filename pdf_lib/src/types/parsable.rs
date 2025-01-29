use core::str;

use crate::pdf_error::{PdfError, PdfErrorKind};

use super::*;

pub trait Parsable: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError>;
}

pub fn parse<T>(bytes: &[u8]) -> Result<(T, &[u8]), PdfError>
where
    T: Parsable,
{
    T::from_bytes(bytes)
}

impl Parsable for PdfObject {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        match bytes {
            [b'<', b'<', ..] => parse::<PdfDict>(bytes).map(|(o, b)| (o.into(), b)),
            [b'[', ..] => parse::<PdfArray>(bytes).map(|(o, b)| (o.into(), b)),
            [b'(' | b'<', ..] => parse::<PdfString>(bytes).map(|(o, b)| (o.into(), b)),
            [b'/', ..] => parse::<PdfName>(bytes).map(|(o, b)| (o.into(), b)),
            [b'f' | b't', ..] => parse::<bool>(bytes).map(|(o, b)| (o.into(), b)),
            [b'n', ..] => parse::<PdfNull>(bytes).map(|(o, b)| (o.into(), b)),
            _ => {
                // Handle Number and indirect object
                let indirect = parse_indirect(bytes);
                let indirect_or_num =
                    indirect.or_else(|_e| parse::<PdfNumeric>(bytes).map(|(o, b)| (o.into(), b)));
                indirect_or_num
            }
        }
    }
}

impl Parsable for bool {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        match bytes.len() {
            ..4 => None,
            4 => (bytes == b"true").then_some((true, &[] as &[u8])),
            5.. => (&bytes[..5] == b"false")
                .then_some((false, &bytes[5..]))
                .or_else(|| (&bytes[..4] == b"true").then_some((true, &bytes[4..]))),
        }
        .and_then(|r| match r.1.len() {
            0 => Some(r),
            _ => WHITESPACES.contains(&r.1[0]).then_some(r),
        })
        .ok_or(PdfError {
            kind: PdfErrorKind::ParseError,
        })
    }
}
#[derive(Clone)]
struct Whitespace {
    _bytes: Vec<u8>,
}
impl Parsable for Whitespace {
    fn from_bytes(mut bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        let mut data = Vec::new();
        while bytes.first().is_some_and(|b| WHITESPACES.contains(b)) {
            data.push(bytes[0]);
            bytes = &bytes[1..];
        }
        Ok((Self { _bytes: data }, bytes))
    }
}

pub(crate) fn parse_indirect(mut bytes: &[u8]) -> Result<(PdfObject, &[u8]), PdfError> {
    let e = Err(PdfError::with_kind(PdfErrorKind::ParseError));
    let Some(first_space) = bytes.iter().position(|b| WHITESPACES.contains(b)) else {
        e?
    };
    let o = usize::from_str_radix(str::from_utf8(&bytes[..first_space])?, 10)?;
    bytes = strip_whitespace(&bytes[first_space..]);
    let Some(second_space) = bytes.iter().position(|b| WHITESPACES.contains(b)) else {
        e?
    };
    let g = usize::from_str_radix(str::from_utf8(&bytes[..second_space])?, 10)?;
    let indirect = IndirectData {
        object: o,
        generation: g,
    };
    bytes = strip_whitespace(&bytes[first_space..]);
    if !bytes.starts_with(b"obj") {
        e?
    }
    bytes = strip_whitespace(&bytes[3..]);
    let (mut object, rest) = parse::<PdfObject>(bytes)?;
    bytes = strip_whitespace(rest);
    if bytes.starts_with(b"endobj") {
        bytes = &bytes[6..];
        object.indirect = Some(indirect);
        Ok((object, bytes))
    } else if bytes.starts_with(b"stream") {
        bytes = &bytes[6..];
        bytes = match bytes {
            [b'\n', ..] => &bytes[1..],
            [b'\r', b'\n', ..] => &bytes[2..],
            _ => e?,
        };
        let dict = object.as_dict()?;
        let stream_length: i32 = dict
            .get_or_null(&PdfName::from_raw_bytes(b"Length"))
            .as_numeric_ref()?
            .into();
        let len = stream_length as usize;
        if bytes.len() < len {
            e?
        };
        let mut data = Vec::with_capacity(stream_length as usize);
        data.extend_from_slice(&bytes[..len]);
        bytes = strip_whitespace(&bytes[len..]);
        if !bytes.starts_with(b"endstream") {
            e?
        }
        bytes = strip_whitespace(&bytes[9..]);
        if !bytes.starts_with(b"endobj") {
            e?
        }
        bytes = &bytes[6..];

        Ok((
            PdfStream::with_len(dict, len, data).as_indirect(indirect),
            bytes,
        ))
    } else {
        e?
    }
}

pub(crate) fn strip_whitespace(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(|b| WHITESPACES.contains(b)) {
        bytes = &bytes[1..];
    }
    bytes
}

pub(crate) fn next_eol(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(|b| !EOLS.contains(b)) {
        bytes = &bytes[1..];
    }
    match bytes {
        [b'\r', b'\n', ..] => &bytes[2..],
        [_b, ..] => &bytes[1..],
        [] => bytes,
    }
}
