use std::collections::HashMap;

use crate::{
    parse,
    pdf_error::{PdfError, PdfErrorKind},
    strip_whitespace, Parsable,
};

use super::{PdfName, PdfNull, PdfObject};

#[derive(Debug, PartialEq, Clone)]
pub struct PdfDict {
    data: HashMap<PdfName, PdfObject>,
}

impl PdfDict {
    pub const NULL: PdfObject = PdfObject {
        kind: super::PdfObjectKind::Null(PdfNull {}),
        indirect: None,
    };
    pub fn get(&self, key: &PdfName) -> Option<&PdfObject> {
        self.data.get(key)
    }

    pub fn get_or_null(&self, key: &PdfName) -> &PdfObject {
        self.data.get(key).unwrap_or(&Self::NULL)
    }
}

impl Parsable for PdfDict {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), PdfError> {
        let [b'<', b'<', bytes @ ..] = bytes else {
            return Err(PdfError::with_kind(PdfErrorKind::ParseError));
        };
        let mut bytes = strip_whitespace(bytes);
        let mut data = HashMap::new();
        while !bytes.is_empty() && !matches!(bytes, [b'>', b'>', ..]) {
            let (key, key_bytes) = parse(bytes)?;
            let key_bytes = strip_whitespace(key_bytes);
            let (value, value_bytes) = parse(key_bytes)?;
            let value_bytes = strip_whitespace(value_bytes);

            data.insert(key, value);
            bytes = value_bytes;
        }

        if !matches!(bytes, [b'>', b'>', ..]) {
            return Err(PdfError::with_kind(PdfErrorKind::ParseError));
        }

        bytes = strip_whitespace(&bytes[2..]);

        Ok((Self { data }, bytes))
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::{
        parse,
        types::{PdfName, PdfNumeric, PdfString},
    };

    use super::PdfDict;

    #[test]
    fn parsing() {
        let example = "<</Type /Example
                                /Subtype /DictionaryExample
                                /Version 0.01
                                /IntegerItem 12
                                /StringItem (a string)
                                /Subdictionary <<
                                    /Item1 0.4
                                    /Item2 true
                                    /LastItem (not !)
                                    /VeryLastItem (OK)
                                >>
                            >>";
        let expected = PdfDict {
            data: HashMap::from_iter([
                (
                    PdfName::from_raw_bytes(b"Type"),
                    PdfName::from_raw_bytes(b"Example").into(),
                ),
                (
                    PdfName::from_raw_bytes(b"Subtype"),
                    PdfName::from_raw_bytes(b"DictionaryExample").into(),
                ),
                (
                    PdfName::from_raw_bytes(b"Version"),
                    PdfNumeric::PdfReal(0.01).into(),
                ),
                (
                    PdfName::from_raw_bytes(b"IntegerItem"),
                    PdfNumeric::PdfInt(12).into(),
                ),
                (
                    PdfName::from_raw_bytes(b"StringItem"),
                    PdfString::from_raw_bytes(b"a string").into(),
                ),
                (
                    PdfName::from_raw_bytes(b"Subdictionary"),
                    PdfDict {
                        data: HashMap::from_iter([
                            (
                                PdfName::from_raw_bytes(b"Item1"),
                                PdfNumeric::PdfReal(0.4).into(),
                            ),
                            (PdfName::from_raw_bytes(b"Item2"), true.into()),
                            (
                                PdfName::from_raw_bytes(b"LastItem"),
                                PdfString::from_raw_bytes(b"not !").into(),
                            ),
                            (
                                PdfName::from_raw_bytes(b"VeryLastItem"),
                                PdfString::from_raw_bytes(b"OK").into(),
                            ),
                        ]),
                    }
                    .into(),
                ),
            ]),
        };
        let parsed = parse::<PdfDict>(example.as_bytes());

        assert_eq!(parsed, Ok((expected, &[] as &[u8])))
    }
}
