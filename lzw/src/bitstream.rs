use crate::codes::{LZWCodeEncoder, SizedCode};
use crate::{EncoderOptions, LZWError};

pub struct LZWEncoder<I> {
    input: LZWCodeEncoder<I>,
    buffer: usize,
    buffer_size: usize,
    done: bool,
}

impl<I> LZWEncoder<I> {
    pub fn new(input: I, options: EncoderOptions) -> Result<Self, LZWError> {
        let input = LZWCodeEncoder::new(input, options)?;
        Ok(Self {
            input,
            buffer: 0,
            buffer_size: 0,
            done: false,
        })
    }
}
impl<I: Iterator<Item = u8>> Iterator for LZWEncoder<I> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        };
        if self.buffer_size >= 8 {
            let output = (self.buffer & 0xFF) as u8;
            self.buffer >>= 8;
            self.buffer_size -= 8;
            return Some(output);
        };

        match self.input.next() {
            None => {
                self.done = true;
                if self.buffer_size == 0 {
                    None
                } else {
                    Some((self.buffer << (8 - self.buffer_size)) as u8)
                }
            }
            Some(SizedCode(mut code, code_size)) => {
                let extra_bits_len = code_size + self.buffer_size - 8;
                let extra_bits = code & ((1 << extra_bits_len) - 1);
                code >>= extra_bits_len;
                let output = ((self.buffer << (8 - self.buffer_size)) | code) as u8;
                self.buffer = extra_bits;
                self.buffer_size = extra_bits_len;
                Some(output)
            }
        }
    }
}
