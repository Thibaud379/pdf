use crate::filter::FilterData;
use crate::pdf_error::{PdfError, PdfResult};
use crate::{PdfName, PdfNumeric};
use lzw::{EncoderOptions, LZWEncoder};
use std::cell::RefCell;
use std::iter::Map;

thread_local! {
    static ERROR: RefCell<Option<PdfError>> = RefCell::new(None);
}
fn adapt(v: PdfResult<u8>) -> u8 {
    v.unwrap_or_else(|e| {
        ERROR.replace(Some(e));
        0
    })
}
pub struct EncodeLZW<I> {
    errored: bool,
    encoder: LZWEncoder<Map<I, fn(PdfResult<u8>) -> u8>>,
}

impl<I: Iterator<Item = PdfResult<u8>>> EncodeLZW<I> {
    pub fn new(inner: FilterData<I>) -> Self {
        let options = inner.params;
        let early_change = options
            .get(&PdfName::from_raw_bytes(b"EarlyChange"))
            .map(|obj| obj.clone().as_numeric())
            .unwrap_or(Ok(PdfNumeric::PdfInt(1)))
            .expect("Param EarlyChange to be a number");
        let early_change = i32::from(early_change) == 1;
        let adapter: Map<I, fn(PdfResult<u8>) -> u8> = inner.iter.map(adapt);
        let encoder = LZWEncoder::new(
            adapter,
            EncoderOptions {
                early_change,
                ..Default::default()
            },
        )
        .expect("Default options should be valid");
        Self {
            errored: false,
            encoder,
        }
    }
}

impl<I: Iterator<Item = PdfResult<u8>>> Iterator for EncodeLZW<I> {
    type Item = PdfResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.errored) {
            return None;
        };
        if ERROR.with_borrow(|e| e.is_some()) {
            self.errored = true;
            let error = ERROR.replace(None);
            return error.map(Result::Err);
        };
        self.encoder.next().map(Result::Ok)
    }
}
