use std::collections::HashMap;

pub struct LZWEncoder<I> {
    table: HashMap<Vec<u8>, u16>,
    input: I,
    peeked: Option<u8>,
}

impl<I> LZWEncoder<I> {
    pub fn new(input: I) -> Self {
        // Default encoding for all possible bytes value
        let table = (0u8..=255u8).map(|b| (vec![b], b as u16)).collect();
        Self {
            input,
            table,
            peeked: None,
        }
    }

    pub fn seed_table<E>(mut self, entries: E) -> Self
    where
        E: IntoIterator<Item = (Vec<u8>, u16)>,
    {
        self.table.extend(entries);
        self
    }
}
// pub struct LZWDecoder<I> {
//     table: HashMap<u16, Vec<u8>>,
//     input: I,
//     peeked: Option<u8>,
// }

impl<I: Iterator<Item = u8>> Iterator for LZWEncoder<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
