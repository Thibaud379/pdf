use pdf_cross_ref_table::PdfCrossRefTable;
use pdf_header::PdfHeader;

use crate::pdf_error::{PdfError, PdfErrorKind};

mod pdf_cross_ref_table;
mod pdf_header;

mod constants {
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
