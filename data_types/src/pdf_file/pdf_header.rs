use crate::{
    next_eol,
    pdf_error::{PdfError, PdfErrorKind},
    strip_whitespace, Parsable,
};

use std::{char::REPLACEMENT_CHARACTER, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfHeader {
    version: PdfVersion,
    binary_marker: bool,
}
impl PdfHeader {
    pub fn with_verion(version: PdfVersion, binary_marker: bool) -> Result<Self, PdfError> {
        matches!((version.major, version.minor), (1, 0..=7) | (2, 0))
            .then_some(Self {
                version,
                binary_marker,
            })
            .ok_or_else(|| PdfError::with_kind(PdfErrorKind::ParseError))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct PdfVersion {
    minor: u8,
    major: u8,
}

impl Display for PdfVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
impl Display for PdfHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.binary_marker {
            writeln!(f, "%PDF-{}", self.version)?;
            writeln!(f, "%{}", REPLACEMENT_CHARACTER.to_string().repeat(2))
        } else {
            writeln!(f, "%PDF-{}", self.version)
        }
    }
}

impl Parsable for PdfHeader {
    fn from_bytes(mut bytes: &[u8]) -> Result<(Self, &[u8]), crate::pdf_error::PdfError> {
        if !bytes.starts_with(b"%PDF-") {
            return Err(PdfError::with_kind(PdfErrorKind::ParseError));
        };
        bytes = &bytes[5..];
        if bytes.len() < 3 {
            return Err(PdfError::with_kind(PdfErrorKind::ParseError));
        }
        let major = (bytes[0] as char)
            .to_digit(10)
            .map(TryInto::try_into)
            .map(Result::unwrap);
        let minor = (bytes[2] as char)
            .to_digit(10)
            .map(TryInto::try_into)
            .map(Result::unwrap);
        if major.and(minor).is_some() && bytes[1] == b'.' {
            let minor = minor.unwrap();
            let major = major.unwrap();
            bytes = &bytes[3..];
            bytes = strip_whitespace(bytes);
            let binary_marker = match bytes {
                [b'%', a, b, c, d, ..] if *a >= 128 && *b >= 128 && *c >= 128 && *d >= 128 => {
                    bytes = next_eol(bytes);
                    true
                }
                _ => false,
            };

            let version = PdfVersion { minor, major };
            let header = Self::with_verion(version, binary_marker)?;

            Ok((header, bytes))
        } else {
            Err(PdfError::with_kind(PdfErrorKind::ParseError))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parse,
        pdf_error::{PdfError, PdfErrorKind},
    };

    use super::{PdfHeader, PdfVersion};

    #[test]
    fn print_header() {
        let mut header = PdfHeader {
            version: PdfVersion { major: 1, minor: 5 },
            binary_marker: true,
        };
        assert_eq!(&header.to_string(), "%PDF-1.5\n%\u{FFFD}\u{FFFD}\n");
        header.binary_marker = false;
        assert_eq!(&header.to_string(), "%PDF-1.5\n");
    }

    #[test]
    fn parsing() {
        let examples = [
            "%PDF-1.0\n%\u{FFFD}\u{FFFD}\n",
            "%PDF-2.0\n%\u{FFFD}\u{FFFD}\n",
            "%PDF-1.0\n",
            "%PDF-1.9\n",
        ];
        let expected = [
            Ok(PdfHeader {
                version: PdfVersion { major: 1, minor: 0 },
                binary_marker: true,
            }),
            Ok(PdfHeader {
                version: PdfVersion { major: 2, minor: 0 },
                binary_marker: true,
            }),
            Ok(PdfHeader {
                version: PdfVersion { major: 1, minor: 0 },
                binary_marker: false,
            }),
            Err(PdfError::with_kind(PdfErrorKind::ParseError)),
        ];

        for (s, e) in examples.into_iter().zip(expected) {
            assert_eq!(parse(s.as_bytes()), e.map(|o| (o, &[] as &[u8])));
        }
    }
}
