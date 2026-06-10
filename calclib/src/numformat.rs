use std::{fmt::Display, str::FromStr};

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

    // pub fn from_str(s: &str) -> Option<NumberFormat> {}
}

impl FromStr for NumberFormat {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "decimal" => Ok(NumberFormat::Decimal),
            "hexadecimal" => Ok(NumberFormat::Hexadecimal),
            "binary" => Ok(NumberFormat::Binary),
            _ => Err(std::fmt::Error),
        }
    }
}
