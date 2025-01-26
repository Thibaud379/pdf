use pdf_cross_ref_table::*;
use pdf_header::PdfHeader;

use crate::pdf_error::*;

mod pdf_cross_ref_table;
mod pdf_header;

pub(self) mod constants {
    use std::char::REPLACEMENT_CHARACTER;

    pub const BINARY_MARKER_CHAR: char = REPLACEMENT_CHARACTER;
    pub const BINARY_MARKER_LEN: usize = 4;
    pub const CROSS_REF_SECTION_KEYWORD: &str = "xref";
}
pub struct PdfFile {
    header: PdfHeader,
    body: PdfBody,
    ref_table: PdfCrossRefTable,
    trailer: PdfTrailer,
}

pub struct PdfBody {
    // "Sequence of indirect objects" 7.5.3
    // If Version >= 1.5, also contains object streams
}
pub struct PdfTrailer {}
