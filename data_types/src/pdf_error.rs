#[allow(dead_code)]
pub type PdfResult<T> = std::result::Result<T, PdfError>;
#[derive(PartialEq, Debug)]
pub enum PdfErrorKind {
    ParseError,
}
#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub struct PdfError {
    pub(crate) kind: PdfErrorKind,
}
