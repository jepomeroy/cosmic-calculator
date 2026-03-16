use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CalcLibError {
    #[error("Conversion Error: {0}")]
    FloatCoversionError(#[from] std::num::ParseFloatError),
    #[error("Conversion Error: {0}")]
    HexCoversionError(String),
    #[error("Conversion Error: {0}")]
    BinCoversionError(String),
    #[error("Syntax Error: {0}")]
    SyntaxError(String),
}
