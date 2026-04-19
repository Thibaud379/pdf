use crate::pdf_error::{PdfError, PdfErrorKind};

mod filter;
mod pdf_error;
mod pdf_file;
mod types;

pub use pdf_file::PdfFile;
pub use types::*;

#[allow(dead_code)]
mod pdf_constants {
    const LINE_WIDTH: usize = 255;
    const EOL: &str = "\r\n";
}
