mod pdf_name;
mod pdf_num;
mod pdf_str;

pub use pdf_name::*;
pub use pdf_num::*;
pub use pdf_str::*;

pub(crate) const WHITESPACES: [u8; 6] = *b"\x00\t\n\x0c\r ";
pub(crate) const EOLS: [u8; 2] = [b'\n', b'\r'];
pub(crate) const DELIMITERS: [u8; 10] = *b"()<>[]{}/%";
pub(crate) fn is_regular(byte: u8) -> bool {
    !WHITESPACES.contains(&byte) && !DELIMITERS.contains(&byte)
}
