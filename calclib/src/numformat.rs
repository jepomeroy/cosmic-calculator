use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberFormat {
    #[default]
    Decimal,
    Hexadecimal,
    Binary,
}

impl Display for NumberFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberFormat::Decimal => write!(f, "Decimal"),
            NumberFormat::Hexadecimal => write!(f, "Hexadecimal"),
            NumberFormat::Binary => write!(f, "Binary"),
        }
    }
}

impl NumberFormat {
    pub fn as_str(&self) -> &str {
        match self {
            NumberFormat::Decimal => "decimal",
            NumberFormat::Hexadecimal => "hexadecimal",
            NumberFormat::Binary => "binary",
        }
    }

    pub fn from_str(s: &str) -> Option<NumberFormat> {
        match s {
            "decimal" => Some(NumberFormat::Decimal),
            "hexadecimal" => Some(NumberFormat::Hexadecimal),
            "binary" => Some(NumberFormat::Binary),
            _ => None,
        }
    }
}
