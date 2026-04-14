use crate::{EncoderOptions, LZWError, LZWErrorKind};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub(crate) struct EncoderOptionsChecked {
    early_change: bool,
    //Must be > log2(dict.max() + (clear_table_code exists) + (eof_code exists))
    min_bit_width: usize,
    //Must be <= 2*sizeof(usize)
    max_bit_width: usize,
    /// Assume no gap, they will not be used elsewise
    dict: Vec<u8>,
    clear_table_code: usize,
    eof_code: Option<usize>,
    starting_code: usize,
}

impl EncoderOptionsChecked {
    pub fn empty_table(&self) -> HashMap<Vec<u8>, usize> {
        self.dict
            .iter()
            .copied()
            .map(|b| (vec![b], b as usize))
            .collect()
    }
}

impl<I: Debug> Debug for LZWCodeEncoder<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LZWCodeEncoder")
            .field("options", &self.options)
            .field("buffer", &self.buffer)
            .field(
                "table",
                &self
                    .table
                    .iter()
                    .filter(|(k, code)| {
                        if **code < 256 {
                            self.options.dict.contains(&(**code as u8))
                        } else {
                            true
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
impl EncoderOptions {
    pub(crate) fn check(self) -> Result<EncoderOptionsChecked, LZWError> {
        (self.min_bit_width <= self.max_bit_width)
            .then(|| 0)
            .ok_or(LZWError::with_kind_and_reason(
                LZWErrorKind::InvalidOptions,
                "min_bit_width > max_bit_width",
            ))?;
        (self.max_bit_width <= 2 * size_of::<usize>())
            .then(|| 0)
            .ok_or(LZWError::with_kind_and_reason(
                LZWErrorKind::InvalidOptions,
                "max_bit_width > 2*size_of::<usize>",
            ))?;
        let max_dict_code = self.dict.iter().copied().max().unwrap_or_default();
        let mut first_code = (max_dict_code as usize).max(self.clear_table_code);
        if self.eof_code.is_some() {
            first_code = first_code.max(self.eof_code.unwrap());
        }
        first_code += 1;
        let min_bits = first_code.ilog2();
        (self.min_bit_width >= min_bits as usize).then(|| 0).ok_or(
            LZWError::with_kind_and_reason(
                LZWErrorKind::InvalidOptions,
                "min_bit_width is too small for the provided dict",
            ),
        )?;
        Ok(EncoderOptionsChecked {
            early_change: self.early_change,
            min_bit_width: self.min_bit_width,
            max_bit_width: self.max_bit_width,
            dict: self.dict,
            clear_table_code: self.clear_table_code,
            eof_code: self.eof_code,
            starting_code: first_code,
        })
    }
}

#[derive(Default)]
enum EncoderState {
    #[default]
    NotStarted,
    Running,
    Ending,
    Done,
}
#[derive(Debug, Default)]
struct EncoderStatistics {
    bytes_read: usize,
    bits_written: usize,
}
pub(crate) struct LZWCodeEncoder<I> {
    options: EncoderOptionsChecked,
    input: I,
    table: HashMap<Vec<u8>, usize>,
    next_code: usize,
    buffer: Vec<u8>,
    current_bit_size: usize,
    state: EncoderState,
    statistics: EncoderStatistics,
}

impl<I> LZWCodeEncoder<I> {
    pub(crate) fn new(input: I, options: EncoderOptions) -> Result<Self, LZWError> {
        let options = options.check()?;
        let current_bit_size = options.min_bit_width;
        let table = options.empty_table();
        let next_code = options.starting_code;
        Ok(Self {
            options,
            input,
            table,
            next_code,
            current_bit_size,
            buffer: vec![],
            state: EncoderState::NotStarted,
            statistics: EncoderStatistics::default(),
        })
    }
    pub fn compression_ratio(&self) -> f64 {
        self.statistics.bytes_read as f64 / (self.statistics.bits_written / 8) as f64
    }

    fn should_increment_code_width(&self) -> bool {
        self.next_code == ((1 << self.current_bit_size) - 1 - self.options.early_change as usize)
    }

    fn clear_table_code(&mut self) -> Option<SizedCode> {
        self.statistics.bits_written += self.current_bit_size;
        Some(SizedCode(
            self.options.clear_table_code,
            self.current_bit_size,
        ))
    }
    fn eof_code(&mut self) -> Option<SizedCode> {
        self.options.eof_code.map(|code| {
            self.statistics.bits_written += self.current_bit_size;
            SizedCode(code, self.current_bit_size)
        })
    }
    fn buffer_code(&mut self) -> SizedCode {
        self.statistics.bits_written += self.current_bit_size;
        SizedCode(
            *self
                .table
                .get(&self.buffer)
                .expect("buffer to be a known sequence"),
            self.current_bit_size,
        )
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub(crate) struct SizedCode(
    /// code
    pub usize,
    /// code width in bits
    pub usize,
);

impl<I: Iterator<Item = u8>> Iterator for LZWCodeEncoder<I> {
    type Item = SizedCode;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            EncoderState::NotStarted => {
                self.state = EncoderState::Running;
                return self.clear_table_code();
            }
            EncoderState::Done => {
                return None;
            }
            EncoderState::Ending => {
                self.state = EncoderState::Done;
                return self.eof_code();
            }
            _ => (),
        };
        if self.should_increment_code_width() {
            if self.current_bit_size == self.options.max_bit_width {
                self.current_bit_size = self.options.min_bit_width;
                self.next_code = self.options.starting_code;
                return self.clear_table_code();
            } else {
                self.current_bit_size += 1;
            };
        };

        loop {
            match self.input.next() {
                None => {
                    self.state = EncoderState::Ending;
                    return Some(self.buffer_code());
                }
                Some(b) => {
                    self.statistics.bytes_read += 1;
                    self.buffer.push(b)
                }
            }
            if !self.table.contains_key(&self.buffer) {
                break;
            }
        }

        self.table.insert(self.buffer.clone(), self.next_code);
        self.next_code += 1;

        let last_byte = self.buffer.pop().expect("buffer not to be empty");
        let code = self.buffer_code();
        self.buffer.clear();
        self.buffer.push(last_byte);
        Some(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn example_pdf() {
        let input: Vec<u8> = vec![45, 45, 45, 45, 45, 65, 45, 45, 45, 66];
        let expected_out: Vec<_> = vec![256, 45, 258, 258, 65, 259, 66, 257]
            .into_iter()
            .map(|code| SizedCode(code, 9))
            .collect();
        let encoder = LZWCodeEncoder::new(input.iter().copied(), EncoderOptions::default());
        assert_matches!(encoder, Ok(_));
        let output: Vec<_> = encoder.unwrap().collect();
        assert_eq!(output, expected_out);
    }
    #[test]
    fn example_wiki_fr_1() {
        let input: Vec<u8> = b"TOBEORNOTTOBEORTOBEORNOT".into_iter().copied().collect();
        let expected_out: Vec<_> = [256]
            .into_iter()
            .chain(b"TOBEORNOT".into_iter().map(|b| *b as u16))
            .chain([257, 259, 261, 266, 260, 262, 264])
            .map(|code| SizedCode(code as usize, 9))
            .collect();
        let encoder = LZWCodeEncoder::new(
            input.into_iter(),
            EncoderOptions {
                eof_code: None,
                ..Default::default()
            },
        );
        assert_matches!(encoder, Ok(_));
        let output: Vec<_> = encoder.unwrap().collect();
        assert_eq!(output, expected_out);
    }
    #[test]
    fn example_wiki_fr_2() {
        let input: Vec<u8> = b"ababcbababaaaaaaa".into_iter().copied().collect();
        let expected_out: Vec<_> = b"\0abdcehajka"
            .map(|code| SizedCode(code as usize, 9))
            .to_vec();
        let encoder = LZWCodeEncoder::new(
            input.into_iter(),
            EncoderOptions {
                eof_code: None,
                dict: b"abc".into(),
                clear_table_code: 0,
                ..Default::default()
            },
        );
        assert_matches!(encoder, Ok(_));
        let output: Vec<_> = encoder.unwrap().collect();
        assert_eq!(output, expected_out);
    }
}
