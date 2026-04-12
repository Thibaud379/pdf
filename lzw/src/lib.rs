#![feature(assert_matches)]

mod bitstream;
pub use bitstream::*;
mod codes;

pub struct EncoderOptions {
    pub early_change: bool,
    //Must be > log2(dict.len() + (clear_table_code exists) + (eof_code exists))
    pub min_bit_width: usize,
    //Must be < 2*sizeof(usize)
    pub max_bit_width: usize,
    pub dict: Vec<u8>,
    pub clear_table_code: usize,
    pub eof_code: Option<usize>,
}
impl Default for EncoderOptions {
    /// Default settings for PDF LZWDecode filter
    fn default() -> Self {
        EncoderOptions {
            early_change: true,
            min_bit_width: 9,
            max_bit_width: 12,
            dict: (0..=255).collect(),
            clear_table_code: 256,
            eof_code: Some(257),
        }
    }
}

#[derive(Debug)]
pub enum LZWErrorKind {
    InvalidOptions,
}
#[derive(Debug)]
pub struct LZWError {
    kind: LZWErrorKind,
    reason: String,
}

impl LZWError {
    pub fn with_kind_and_reason(kind: LZWErrorKind, reason: &str) -> LZWError {
        LZWError {
            kind,
            reason: String::from(reason),
        }
    }
}
