use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CalcLibError {
    #[error("Conversion Error: {0}")]
    FloatConversionError(#[from] std::num::ParseFloatError),
    #[error("Conversion Error: {0}")]
    HexConversionError(String),
    #[error("Conversion Error: {0}")]
    BinConversionError(String),
    #[error("Syntax Error: {0}")]
    SyntaxError(String),
    #[error("Invalid Expression: {0}")]
    InvalidExpression(String),
    #[error("Division by zero")]
    DivisionByZero(),
    #[error("Failed to compute factorial")]
    FactorialComputeError(),
    #[error("Unsupported Operator")]
    UnsupportedOperator(),
}
