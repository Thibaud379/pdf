use super::constants::*;
use std::fmt::Display;
pub struct PdfHeader {
    version: PdfVersion,
    binary_marker: bool,
}
pub struct PdfVersion {
    minor: u8,
    major: u8,
}

impl Display for PdfVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
impl Display for PdfHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.binary_marker {
            writeln!(f, "%PDF-{}", self.version)?;
            writeln!(
                f,
                "%{}",
                BINARY_MARKER_CHAR.to_string().repeat(BINARY_MARKER_LEN)
            )
        } else {
            writeln!(f, "%PDF-{}", self.version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PdfHeader, PdfVersion};

    #[test]
    fn print_header() {
        let mut header = PdfHeader {
            version: PdfVersion { major: 1, minor: 5 },
            binary_marker: true,
        };
        assert_eq!(
            &header.to_string(),
            "%PDF-1.5\n%\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\n"
        );
        header.binary_marker = false;
        assert_eq!(&header.to_string(), "%PDF-1.5\n");
    }
}
