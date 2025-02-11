use core::str;
use std::str::FromStr;

use crate::{Parsable, pdf_error::*};

use super::{EOLS, WHITESPACES};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct PdfString {
    data: Vec<u8>,
}

impl PdfString {
    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.iter().copied().collect(),
        }
    }

    fn from_str_literal(s: &str) -> PdfResult<Self> {
        let mut pars: usize = 0;
        let mut data = Vec::new();
        let mut solidus = false;
        let mut code = String::with_capacity(3);
        let last = s.get((s.len() - 1)..);
        if !last.is_some_and(|s| s.starts_with(')')) {
            return Err(PdfError {
                kind: PdfErrorKind::Parse,
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
                            kind: PdfErrorKind::Parse,
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
                                kind: PdfErrorKind::Parse,
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
        if solidus && !code.is_empty() {
            let Ok(code_value) = u8::from_str_radix(&code, 8) else {
                return Err(PdfError {
                    kind: PdfErrorKind::Parse,
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
                kind: PdfErrorKind::Parse,
            })
        }
    }
    fn from_str_hexa(mut s: &str) -> PdfResult<Self> {
        let mut data = Vec::new();
        let last = s.get((s.len() - 1)..);
        if !last.is_some_and(|s| s.starts_with('>')) {
            return Err(PdfError {
                kind: PdfErrorKind::Parse,
            });
        }
        s = &s[1..(s.len() - 1)];
        let chars = s
            .chars()
            .filter(|b| !WHITESPACES.contains(&(*b as u8)))
            .collect::<Vec<_>>();
        for (a, b) in chars.iter().zip(chars.iter().skip(1)).step_by(2) {
            let s = String::from_iter([a, b]);
            data.push(u8::from_str_radix(&s, 16)?);
        }
        if chars.len() % 2 != 0 {
            let last = chars.last().unwrap();
            let s = String::from_iter([last, &'0']);
            data.push(u8::from_str_radix(&s, 16)?);
        }
        Ok(Self { data })
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn from_bytes_hexa(bytes: &[u8]) -> PdfResult<(PdfString, &[u8])> {
        let e = Err(PdfError::with_kind(PdfErrorKind::Parse));
        let Some(right_bracket) = bytes.iter().position(|b| *b == b'>') else {
            return e;
        };

        let first_token = &bytes[..=right_bracket];
        let rest = &bytes[(right_bracket + 1)..];
        let content = &first_token[1..first_token.len() - 1];

        let data =
            content
                .iter()
                .filter(|b| !WHITESPACES.contains(b))
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|p| {
                    let l = *p[0];
                    let r = *p.get(1).copied().unwrap_or(&b'0');
                    if l.is_ascii_hexdigit() && r.is_ascii_hexdigit() {
                        #[allow(clippy::cast_possible_truncation)]
                        Ok(((l as char).to_digit(16).unwrap() * 16
                            + (r as char).to_digit(16).unwrap()) as u8)
                    } else {
                        Err(PdfError::with_kind(PdfErrorKind::Parse))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;

        Ok((Self { data }, rest))
    }

    fn from_bytes_literal(mut bytes: &[u8]) -> PdfResult<(PdfString, &[u8])> {
        let mut data = Vec::new();
        let mut pars = 1;
        bytes = &bytes[1..];
        while let [byte, rest @ ..] = bytes {
            let mut rest = rest;
            let n = match *byte {
                b')' => {
                    pars -= 1;
                    if pars == 0 {
                        bytes = &bytes[1..];
                        break;
                    }
                    b')'
                }
                b'(' => {
                    pars += 1;
                    b'('
                }
                b if EOLS.contains(&b) => {
                    if b == b'\r' {
                        if let [b'\n', rrest @ ..] = rest {
                            rest = rrest;
                        }
                    }
                    b'\n'
                }
                b'\\' => {
                    if let [bbyte, rrest @ ..] = rest {
                        let mut rrest = rrest;
                        let n = match (*bbyte, rrest) {
                            (b'n', _) => b'\n',
                            (b't', _) => b'\t',
                            (b'r', _) => b'\r',
                            (b'b', _) => 0x08,
                            (b'f', _) => 0xff,
                            (b'\\', _) => b'\\',
                            (b')', _) => b')',
                            (b'(', _) => b'(',
                            (b, r) if EOLS.contains(&b) => {
                                bytes = if b == b'\r' {
                                    if let [b'\n', rr @ ..] = r { rr } else { r }
                                } else {
                                    r
                                };
                                continue;
                            }
                            (b'0'..=b'7', [b'0'..=b'7', b'0'..=b'7', r @ ..]) => {
                                let b = u8::from_str_radix(
                                    unsafe { str::from_utf8_unchecked(&rest[..3]) },
                                    8,
                                )?;
                                rrest = r;
                                b
                            }
                            (b'0'..=b'7', [b'0'..=b'7', r @ ..]) => {
                                let b = u8::from_str_radix(
                                    unsafe { str::from_utf8_unchecked(&rest[..2]) },
                                    8,
                                )?;
                                rrest = r;
                                b
                            }
                            (b'0'..=b'7', r) => {
                                let b = u8::from_str_radix(
                                    unsafe { str::from_utf8_unchecked(&rest[..1]) },
                                    8,
                                )?;
                                rrest = r;
                                b
                            }
                            _ => continue,
                        };
                        rest = rrest;
                        n
                    } else {
                        break;
                    }
                }
                b => b,
            };
            data.push(n);
            bytes = rest;
        }
        Ok((Self { data }, bytes))
    }
}

impl FromStr for PdfString {
    type Err = PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('<') => Self::from_str_hexa(s),
            Some('(') => Self::from_str_literal(s),
            _ => Err(PdfError {
                kind: PdfErrorKind::Parse,
            }),
        }
    }
}

impl Parsable for PdfString {
    fn from_bytes(b: &[u8]) -> PdfResult<(Self, &[u8])> {
        match b.first() {
            Some(b'<') => Self::from_bytes_hexa(b),
            Some(b'(') => Self::from_bytes_literal(b),
            _ => Err(PdfError {
                kind: PdfErrorKind::Parse,
            }),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{PdfString, parse};

    #[test]
    fn hexa() {
        let examples: [&str; 4] = ["<901FA3>", "<901FA>", "<901 FA>", "<90\n\t1FA>"];
        let expected = [
            PdfString::from_raw_bytes(&[0x90, 0x1F, 0xA3]),
            PdfString::from_raw_bytes(&[0x90, 0x1F, 0xA0]),
            PdfString::from_raw_bytes(&[0x90, 0x1F, 0xA0]),
            PdfString::from_raw_bytes(&[0x90, 0x1F, 0xA0]),
        ];

        for (r, s) in expected.into_iter().zip(examples) {
            let parsed_str = s.parse();
            let parsed_bytes = parse(s.as_bytes());
            assert_eq!(parsed_bytes, Ok((r.clone(), &[] as &[u8])), "B => {s:?}");
            assert_eq!(parsed_str, Ok(r), "S => {s:?}");
        }
    }
    #[test]
    fn literal() {
        let examples = [
            "(string)",
            "(new\nline)",
            "(p(a)r(s)c(a(n)b)e(used))",
            "(*!&}^%)",
            "()",
            r"(\))",
            "(a\\245a\\307)",
            "(\\0053)",
            "(\\053)",
            "(\\53)",
            "(\\53a)",
            "(\\5a)",
            "(a\nb)",
            "(a\rb)",
            "(a\r\nb)",
            "(a\\\nb)",
            "(a\\\r\nb)",
        ];
        let expected = [
            PdfString::from_raw_bytes(b"string"),
            PdfString::from_raw_bytes(b"new\nline"),
            PdfString::from_raw_bytes(b"p(a)r(s)c(a(n)b)e(used)"),
            PdfString::from_raw_bytes(b"*!&}^%"),
            PdfString::from_raw_bytes(b""),
            PdfString::from_raw_bytes(b")"),
            PdfString::from_raw_bytes(&[b'a', 0o245, b'a', 0o307]),
            PdfString::from_raw_bytes(&[0o5, b'3']),
            PdfString::from_raw_bytes(&[0o53]),
            PdfString::from_raw_bytes(&[0o53]),
            PdfString::from_raw_bytes(&[0o53, b'a']),
            PdfString::from_raw_bytes(&[0o5, b'a']),
            PdfString::from_raw_bytes(b"a\nb"),
            PdfString::from_raw_bytes(b"a\nb"),
            PdfString::from_raw_bytes(b"a\nb"),
            PdfString::from_raw_bytes(b"ab"),
            PdfString::from_raw_bytes(b"ab"),
        ];

        for (e, r) in examples.into_iter().zip(expected) {
            let parsed_str = e.parse();
            let parsed_bytes = parse(e.as_bytes());
            assert_eq!(parsed_bytes, Ok((r.clone(), &[] as &[u8])), "B => {e:?}");
            assert_eq!(parsed_str, Ok(r), "S => {e:?}");
        }
    }
}
