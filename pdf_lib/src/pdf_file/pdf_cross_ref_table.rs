use std::str::FromStr;

use super::{PdfError, PdfErrorKind};

#[derive(PartialEq, Debug, Clone)]
pub struct PdfCrossRefTable {
    sections: Vec<PdfCrossRefTableSection>,
}
#[derive(PartialEq, Debug, Clone)]
pub struct PdfCrossRefTableSection {
    subsections: Vec<PdfCrossRefTableSubsection>,
}
#[derive(PartialEq, Debug, Clone)]
pub struct PdfCrossRefTableSubsection {
    header: PdfCrossRefTableSubsectionHeader,
    entries: Vec<PdfCrossRefTableEntry>,
}
#[derive(PartialEq, Debug, Clone)]
pub struct PdfCrossRefTableSubsectionHeader {
    first_object: usize,
    len: usize,
}
#[derive(PartialEq, Debug, Clone)]
pub struct PdfCrossRefTableEntry {
    // Byte offset from beginning of file
    offset: u64,
    gen_number: u16,
    free: bool,
}

impl FromStr for PdfCrossRefTable {
    type Err = PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = s
            .split(super::constants::CROSS_REF_SECTION_KEYWORD)
            .skip(1)
            .map(|s| format!("{}{s}", super::constants::CROSS_REF_SECTION_KEYWORD))
            .map(|s| s.parse::<PdfCrossRefTableSection>())
            .collect::<Result<_, _>>()?;
        Ok(Self { sections })
    }
}

impl FromStr for PdfCrossRefTableSection {
    type Err = PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut subsections = Vec::new();
        let mut subsection = None;
        let mut lines = s.lines();
        if lines
            .next()
            .is_none_or(|k| k != super::constants::CROSS_REF_SECTION_KEYWORD)
        {
            return Err(PdfError {
                kind: PdfErrorKind::Parse,
            });
        }
        for line in lines {
            if let Ok(header) = line.parse() {
                if let Some(ss) = subsection.take() {
                    subsections.push(ss);
                }
                subsection = Some(PdfCrossRefTableSubsection {
                    header,
                    entries: Vec::new(),
                });
                continue;
            }
            let Some(ss) = &mut subsection else {
                Err(PdfError {
                    kind: PdfErrorKind::Parse,
                })?
            };
            let Ok(entry) = line.parse() else {
                Err(PdfError {
                    kind: PdfErrorKind::Parse,
                })?
            };
            ss.entries.push(entry);
        }
        if let Some(ss) = subsection.take() {
            subsections.push(ss);
        }
        Ok(Self { subsections })
    }
}

impl FromStr for PdfCrossRefTableSubsectionHeader {
    type Err = PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items: Vec<usize> = s.split(' ').map(str::parse).collect::<Result<_, _>>()?;
        if items.len() != 2 {
            return Err(PdfError {
                kind: PdfErrorKind::Parse,
            });
        }

        Ok(PdfCrossRefTableSubsectionHeader {
            first_object: items[0],
            len: items[1],
        })
    }
}

impl FromStr for PdfCrossRefTableEntry {
    type Err = PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 18 {
            return Err(PdfError {
                kind: PdfErrorKind::Parse,
            });
        }
        let items: Vec<_> = [s.get(..10), s.get(11..16), s.get(17..18)]
            .into_iter()
            .flatten()
            .collect();
        if items.len() != 3 {
            return Err(PdfError {
                kind: PdfErrorKind::Parse,
            });
        }
        let (offset, gen_number, free) =
            (items[0].parse()?, items[1].parse()?, match items.get(2) {
                Some(&"f") => Ok(true),
                Some(&"n") => Ok(false),
                _ => Err(PdfError {
                    kind: PdfErrorKind::Parse,
                }),
            }?);

        Ok(PdfCrossRefTableEntry {
            offset,
            gen_number,
            free,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod examples {
        use super::*;
        const EX_2: &str = "xref
0 6
0000000003 65535 f
0000000017 00000 n
0000000081 00000 n
0000000000 00007 f
0000000331 00000 n
0000000409 00000 n";

        const EX_3: &str = "xref
0 1
0000000000 65535 f
3 1
0000025325 00000 n
23 2
0000025518 00002 n
0000025635 00000 n
30 1
0000025777 00000 n";

        fn ex_2_builder() -> PdfCrossRefTableSection {
            PdfCrossRefTableSection {
                subsections: vec![PdfCrossRefTableSubsection {
                    header: PdfCrossRefTableSubsectionHeader {
                        first_object: 0,
                        len: 6,
                    },
                    entries: vec![
                        PdfCrossRefTableEntry {
                            offset: 3,
                            gen_number: 65535,
                            free: true,
                        },
                        PdfCrossRefTableEntry {
                            offset: 17,
                            gen_number: 0,
                            free: false,
                        },
                        PdfCrossRefTableEntry {
                            offset: 81,
                            gen_number: 0,
                            free: false,
                        },
                        PdfCrossRefTableEntry {
                            offset: 0,
                            gen_number: 7,
                            free: true,
                        },
                        PdfCrossRefTableEntry {
                            offset: 331,
                            gen_number: 0,
                            free: false,
                        },
                        PdfCrossRefTableEntry {
                            offset: 409,
                            gen_number: 0,
                            free: false,
                        },
                    ],
                }],
            }
        }

        fn ex_3_builder() -> PdfCrossRefTableSection {
            PdfCrossRefTableSection {
                subsections: vec![
                    PdfCrossRefTableSubsection {
                        header: PdfCrossRefTableSubsectionHeader {
                            first_object: 0,
                            len: 1,
                        },
                        entries: vec![PdfCrossRefTableEntry {
                            offset: 0,
                            gen_number: 65535,
                            free: true,
                        }],
                    },
                    PdfCrossRefTableSubsection {
                        header: PdfCrossRefTableSubsectionHeader {
                            first_object: 3,
                            len: 1,
                        },
                        entries: vec![PdfCrossRefTableEntry {
                            offset: 25325,
                            gen_number: 0,
                            free: false,
                        }],
                    },
                    PdfCrossRefTableSubsection {
                        header: PdfCrossRefTableSubsectionHeader {
                            first_object: 23,
                            len: 2,
                        },
                        entries: vec![
                            PdfCrossRefTableEntry {
                                offset: 25518,
                                gen_number: 2,
                                free: false,
                            },
                            PdfCrossRefTableEntry {
                                offset: 25635,
                                gen_number: 0,
                                free: false,
                            },
                        ],
                    },
                    PdfCrossRefTableSubsection {
                        header: PdfCrossRefTableSubsectionHeader {
                            first_object: 30,
                            len: 1,
                        },
                        entries: vec![PdfCrossRefTableEntry {
                            offset: 25777,
                            gen_number: 0,
                            free: false,
                        }],
                    },
                ],
            }
        }

        pub(super) fn examples() -> [(&'static str, PdfCrossRefTableSection); 2] {
            [(EX_2, ex_2_builder()), (EX_3, ex_3_builder())]
        }
    }

    #[test]
    fn parse_cross_table_section() {
        for example in examples::examples() {
            let parsed = example.0.parse();
            assert_eq!(parsed, Ok(example.1));
        }
    }
    #[test]
    fn parse_cross_table() {
        for example in examples::examples() {
            let parsed = example
                .0
                .parse()
                .map(|r: PdfCrossRefTable| r.sections[0].clone());
            assert_eq!(parsed, Ok(example.1));
        }
        let combined = examples::examples()
            .iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(v.0);
                acc
            });
        let combined_sections = PdfCrossRefTable {
            sections: examples::examples().iter().fold(Vec::new(), |mut acc, v| {
                acc.push(v.1.clone());
                acc
            }),
        };

        assert_eq!(combined.parse(), Ok(combined_sections));
    }

    #[test]
    fn parse_entry_errors() {
        let examples = [
            " ",
            "0 1 f",
            "0000000000 1 f",
            "0000000000 10000 a",
            "00000a0000 99999 f",
            "0000000    00000 f",
            "0000000      000 f",
            "0000000    0xada f",
        ];

        for example in examples {
            assert!(
                example.parse::<PdfCrossRefTableEntry>().is_err(),
                "{example}"
            );
        }
    }
}
