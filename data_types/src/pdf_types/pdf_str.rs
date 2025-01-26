use std::str::{self, FromStr};

use crate::{PdfError, PdfErrorKind};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct PdfString {
    data: Vec<u8>,
}

impl PdfString {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: Vec::from_iter(bytes.iter().cloned()),
        }
    }

    fn from_str_literal(s: &str) -> Result<Self, PdfError> {
        let mut pars: usize = 0;
        let mut data = Vec::new();
        let mut solidus = false;
        let mut code = String::with_capacity(3);
        let last = s.get((s.len() - 1)..);
        if !last.is_some_and(|s| s.starts_with(')')) {
            return Err(PdfError {
                kind: PdfErrorKind::ParseError,
            });
        }

        for b in s[1..(s.len() - 1)].bytes() {
            if !solidus {
                let n = match b {
                    b'(' => {
                        pars += 1;
                        b
                    }
                    b')' => {
                        pars = pars.checked_sub(1).ok_or(PdfError {
                            kind: PdfErrorKind::ParseError,
                        })?;
                        b
                    }
                    b'\\' => {
                        solidus = true;
                        continue;
                    }
                    b'\n' => {
                        if data.last() == Some(&b'\r') {
                            data.pop();
                        };
                        b
                    }
                    _ => b,
                };
                data.push(n);
            } else {
                match (code.len(), (b'0'..=b'7').contains(&b)) {
                    (0, false) => {
                        // Other rule
                        solidus = false;
                    }
                    (3, true) | (_, false) => {
                        let Ok(code_value) = u8::from_str_radix(&code, 8) else {
                            return Err(PdfError {
                                kind: PdfErrorKind::ParseError,
                            });
                        };
                        data.push(code_value);
                        code.clear();
                        solidus = false;
                    }
                    (_, true) => {
                        // add to code and continue
                        code.push(b as char);
                        continue;
                    }
                };
                match b {
                    b'n' => data.push(b'\n'),
                    b't' => data.push(b'\t'),
                    b'r' => data.push(b'\r'),
                    b'b' => data.push(0x08),
                    b'f' => data.push(0xff),
                    b'\r' => solidus = true,
                    b'\n' => {
                        if data.last() == Some(&b'\r') {
                            data.pop();
                        };
                        solidus = false;
                    }
                    _ => data.push(b),
                };
            }
        }
        if solidus && code.len() > 0 {
            let Ok(code_value) = u8::from_str_radix(&code, 8) else {
                return Err(PdfError {
                    kind: PdfErrorKind::ParseError,
                });
            };
            data.push(code_value);
        }

        data = data
            .into_iter()
            .map(|b| if b == b'\r' { b'\n' } else { b })
            .collect();
        if pars == 0 {
            Ok(Self { data })
        } else {
            Err(PdfError {
                kind: PdfErrorKind::ParseError,
            })
        }
    }
    fn from_str_hexa(mut s: &str) -> Result<Self, PdfError> {
        let mut data = Vec::new();
        let last = s.get((s.len() - 1)..);
        if !last.is_some_and(|s| s.starts_with('>')) {
            return Err(PdfError {
                kind: PdfErrorKind::ParseError,
            });
        }
        s = &s[1..(s.len() - 1)];
        for (a, b) in s.chars().zip(s.chars().skip(1)).step_by(2) {
            let s = String::from_iter([a, b]);
            data.push(u8::from_str_radix(&s, 16).map_err(|_e| {
                println!("{_e:?}");
                PdfError {
                    kind: PdfErrorKind::ParseError,
                }
            })?);
        }
        if s.len() % 2 != 0 {
            let last = s.chars().last().unwrap();
            let s = String::from_iter([last, '0']);
            data.push(u8::from_str_radix(&s, 16).map_err(|_e| {
                println!("{_e:?}");
                PdfError {
                    kind: PdfErrorKind::ParseError,
                }
            })?);
        }
        Ok(Self { data })
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl FromStr for PdfString {
    type Err = PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('<') => Self::from_str_hexa(s),
            Some('(') => Self::from_str_literal(s),
            _ => Err(PdfError {
                kind: PdfErrorKind::ParseError,
            }),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::PdfString;

    #[test]
    fn hexa_parsing() {
        let examples = ["<901FA3>", "<901FA>"];
        let expected = [
            PdfString::from_bytes(&[0x90, 0x1F, 0xA3]),
            PdfString::from_bytes(&[0x90, 0x1F, 0xA0]),
        ];

        for (r, s) in expected.into_iter().zip(examples) {
            let parsed = s.parse::<PdfString>();
            assert_eq!(parsed, Ok(r));
        }
    }
    #[test]
    fn literal_parsing() {
        let examples = [
            "(string)",
            "(new\nline)",
            "(p(a)r(s)c(a(n)b)e(used))",
            "(*!&}^%)",
            "()",
            r"(\))",
        ];
        let example_lens: [usize; 6] = [6, 8, 23, 6, 0, 1];

        for example in example_lens.into_iter().zip(examples) {
            let parsed = example.1.parse::<PdfString>();
            assert!(
                parsed.clone().is_ok_and(|s| s.len() == example.0),
                "`{}`({}) => {:?}({:?})",
                example.1,
                example.0,
                parsed,
                parsed.clone().map(|s| s.len())
            )
        }
    }

    #[test]
    fn literal_error() {
        let examples = [r"(()", r"())", r"((\))", r"(\())"];
        for s in examples {
            let parsed = s.parse::<PdfString>();
            assert!(parsed.is_err());
        }
    }

    #[test]
    fn literal_escaping() {
        let examples = [r"(a\245a\307)", r"(\0053)", r"(\053)", r"(\53)"];
        let expected = [
            PdfString::from_bytes(&[b'a', 0o245, b'a', 0o307]),
            PdfString::from_bytes(&[0o5, b'3']),
            PdfString::from_bytes(&[0o53]),
            PdfString::from_bytes(&[0o53]),
        ];

        for (s, r) in examples.iter().zip(expected) {
            let parsed = s.parse();
            assert_eq!(parsed, Ok(r));
        }
    }

    #[test]
    fn literal_eol() {
        let examples = ["(a\nb)", "(a\rb)", "(a\r\nb)"];
        let expected = PdfString::from_bytes(vec![b'a', b'\n', b'b'].as_slice());

        for s in examples {
            let parsed = s.parse();
            assert_eq!(parsed, Ok(expected.clone()), "{s}");
        }
    }
    #[test]
    fn literal_line_split() {
        let examples = ["(a\\\nb)", "(a\\\r\nb)"];
        let expected = PdfString::from_bytes(vec![b'a', b'b'].as_slice());

        for s in examples {
            let parsed = s.parse();
            assert_eq!(parsed, Ok(expected.clone()), "{s}");
        }
    }
}
