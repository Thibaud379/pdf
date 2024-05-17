use crate::pdf_error::{PdfError, PdfErrorKind};
use std::str::FromStr;

pub enum PdfBoolean {
    PdfTrue,
    PdfFalse,
}

impl From<&PdfBoolean> for bool {
    fn from(value: &PdfBoolean) -> Self {
        match value {
            PdfBoolean::PdfTrue => true,
            PdfBoolean::PdfFalse => false,
        }
    }
}
impl From<PdfBoolean> for bool {
    fn from(value: PdfBoolean) -> Self {
        match value {
            PdfBoolean::PdfTrue => true,
            PdfBoolean::PdfFalse => false,
        }
    }
}
impl From<bool> for PdfBoolean {
    fn from(value: bool) -> Self {
        if value {
            Self::PdfTrue
        } else {
            Self::PdfFalse
        }
    }
}
impl FromStr for PdfBoolean {
    type Err = PdfError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "true" {
            Ok(Self::PdfTrue)
        } else if value == "false" {
            Ok(Self::PdfFalse)
        } else {
            Err(PdfError {
                kind: PdfErrorKind::ParseError,
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::pdf_error::PdfResult;

    use super::*;

    #[test]
    fn parsing() {
        let p = |a: &str| {
            let c: PdfResult<PdfBoolean> = a.parse();
            let b = c.map(Into::<bool>::into);
            b
        };
        let err = Err(PdfError {
            kind: PdfErrorKind::ParseError,
        });
        assert_eq!(p("true"), Ok(true));
        assert_eq!(p("false"), Ok(false));
        assert_eq!(p("tru e"), err);
        assert_eq!(p("True"), err);
    }
}
