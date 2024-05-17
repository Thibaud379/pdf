use pdf_bool::PdfBoolean;
use pdf_num::PdfNumeric;
use pdf_str::PdfString;

mod pdf_bool;
mod pdf_error;
mod pdf_num;
mod pdf_str;

pub enum PdfObject {
    Boolean(PdfBoolean),
    Numeric(PdfNumeric),
    String(PdfString),
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
