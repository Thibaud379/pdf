use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::Read;
use std::iter::Peekable;

const MIN_BITS_WIDTH: usize = 9;
const MAX_BITS_WIDTH: usize = 12;
enum STATE {
    NotStarted,
    Running,
    HandleEOF,
    Padding,
    Done,
}
pub struct LZWEncoder<I> {
    table: HashMap<Vec<u8>, u16>,
    next_code: u16,
    input: I,
    buffer: Vec<u8>,
    buffer_out: u8,
    buffer_out_bit_size: usize,
    current_bit_size: usize,
    state: STATE,
}

impl<I: Debug> LZWEncoder<I> {
    pub(crate) fn padding(&mut self) -> Option<u8> {
        self.state = STATE::Done;
        let output = self.buffer_out << (8 - self.buffer_out_bit_size);
        Some(output)
    }

    pub(crate) fn handle_eof(&mut self) -> Option<u8> {
        if self.buffer_out_bit_size == 8 {
            self.buffer_out_bit_size = 0;
            self.buffer_out = 0;
            return Some(self.buffer_out);
        }
        let mut code = if self.buffer.is_empty() {
            self.state = STATE::Padding;
            257
        } else {
            *self.table.get(&self.buffer).expect("known sequence")
        };
        self.buffer.clear();
        let extra_bits_len = self.current_bit_size + self.buffer_out_bit_size - 8;
        let extra_bits = code & ((1 << extra_bits_len) - 1);
        assert!(extra_bits < 256);
        code >>= extra_bits_len;
        assert!(code < 256);
        let output = (self.buffer_out) << ((8 - self.buffer_out_bit_size) & 7) | code as u8;
        self.buffer_out = extra_bits as u8;
        self.buffer_out_bit_size = extra_bits_len;
        if self.buffer_out_bit_size == 0 {
            self.state = STATE::Done;
        };
        Some(output)
    }
}

impl<I: Debug> Debug for LZWEncoder<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LZWEncoder")
            .field("current_bit_size", &self.current_bit_size)
            .field("next_code", &self.next_code)
            .field("buffer", &self.buffer)
            .field("buffer_out", &self.buffer_out)
            .field("buffer_out_bit_size", &self.buffer_out_bit_size)
            .field(
                "table",
                &self
                    .table
                    .iter()
                    .filter(|(k, code)| **code > 255)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
impl<I> LZWEncoder<I> {
    pub fn new(input: I) -> Self {
        // Default PDF encoding for all possible bytes value
        let table = (0u8..=255u8).map(|b| (vec![b], b as u16)).collect();

        Self {
            input,
            table,
            next_code: 258u16,
            buffer: Vec::new(),
            buffer_out: 0,
            buffer_out_bit_size: 0,
            current_bit_size: 9,
            state: STATE::NotStarted,
        }
    }

    pub fn seed_table<E>(mut self, entries: E) -> Self
    where
        E: IntoIterator<Item = (Vec<u8>, u16)>,
    {
        self.table.extend(entries);
        self
    }

    fn clear_table(&mut self) {
        self.table = (0u8..=255u8).map(|b| (vec![b], b as u16)).collect();
        self.next_code = 258u16;
    }
}
impl<I> Iterator for LZWEncoder<I>
where
    I: Iterator<Item = u8> + Debug,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            STATE::NotStarted => {
                self.state = STATE::Running;
                self.buffer_out_bit_size = 1;
                self.buffer_out = 0;
                return Some(0x80);
            }
            STATE::Done => {
                return None;
            }
            STATE::HandleEOF => return self.handle_eof(),
            STATE::Padding => return self.padding(),
            STATE::Running => (),
        };

        if self.buffer_out_bit_size == 8 {
            // dbg!(&self);
            self.buffer_out_bit_size = 0;
            let output = self.buffer_out;
            self.buffer_out = 0;
            return Some(output);
        };

        loop {
            match self.input.next() {
                None => {
                    self.state = STATE::HandleEOF;
                    return self.handle_eof();
                }
                Some(b) => self.buffer.push(b),
            }
            if !self.table.contains_key(&self.buffer) {
                break;
            }
        }
        if self.buffer.len() > 2 {
            println!("{:?}", self.buffer)
        };
        //buffer <- [...existing_sequence, b]
        // 1. Add buffer to table
        self.table.insert(self.buffer.clone(), self.next_code);
        self.next_code += 1;
        // 2. get corresponding code and clear buffer
        let last_byte = self.buffer.pop().expect("buffer to have at least one byte");
        let mut code = *self.table.get(&self.buffer).expect("table has sequence");
        self.buffer.clear();
        self.buffer.push(last_byte);
        // 3. emit code with buffer_out bits added and store excess bits in buffer_out
        let extra_bits_len = self.current_bit_size + self.buffer_out_bit_size - 8;
        let extra_bits = code & ((1 << extra_bits_len) - 1);
        assert!(extra_bits < 256);
        code >>= extra_bits_len;
        assert!(code < 256);
        let output = (self.buffer_out) << ((8 - self.buffer_out_bit_size) & 7) | code as u8;
        self.buffer_out = extra_bits as u8;
        self.buffer_out_bit_size = extra_bits_len;
        // dbg!(output, code, extra_bits_len, extra_bits, last_byte);
        // dbg!(&self);
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let input: Vec<u8> = vec![45, 45, 45, 45, 45, 65, 45, 45, 45, 66];
        let expected_out = vec![0x80, 0x0B, 0x60, 0x50, 0x22, 0x0C, 0x0C, 0x85, 0x01];
        let encoder = LZWEncoder::new(input.iter().copied());

        let output: Vec<_> = encoder.collect();
        assert_eq!(output, expected_out);
    }

    #[test]
    fn example2() {
        //Provoke a code-length switch
        let input: Vec<u8> = (0..=255).chain((0..=255).rev()).collect();
        let encoder = LZWEncoder::new(input.iter().copied());
        let mut output: Vec<_> = encoder.collect();
    }
}
