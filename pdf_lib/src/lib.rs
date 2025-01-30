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

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn parsing() {
        let valid: [&[u8]; 3] = [b"true ", b"false", b"false "];
        let expected = [(true, 4), (false, 5), (false, 5)];
        for (bytes, (res, rest)) in valid.into_iter().zip(expected) {
            let parsed = parse(bytes);
            assert_eq!(parsed, Ok((res, &bytes[rest..])));
        }

        let invalid: [&[u8]; 3] = [b"True", b"fal se", b"false\\"];
        for bytes in invalid {
            assert!(parse::<bool>(bytes).is_err());
        }
    }
}
