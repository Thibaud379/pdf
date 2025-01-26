use pdf_types::*;

mod pdf_error;
mod pdf_file;
mod pdf_types;

pub use pdf_error::{PdfError, PdfErrorKind};
pub use pdf_file::PdfFile;

#[allow(dead_code)]
mod pdf_constants {
    const LINE_WIDTH: usize = 255;
    const EOL: &'static str = "\r\n";
}

pub enum PdfObject {
    Boolean(PdfBoolean),
    Numeric(PdfNumeric),
    String(PdfString),
    Name(PdfName),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_test() {
        println!("PdfObject({}B)", std::mem::size_of::<PdfObject>());
        println!("PdfBoolean({}B)", std::mem::size_of::<PdfBoolean>());
        println!("PdfNumeric({}B)", std::mem::size_of::<PdfNumeric>());
        println!("PdfString({}B)", std::mem::size_of::<PdfString>());
    }
}
