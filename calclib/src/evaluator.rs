use crate::parser::Parser;

pub fn evaluate(input: String) -> Result<Option<f64>, String> {
    let mut parser = Parser::new();
    let _tokens = parser.parse(input)?;

    Ok(Some(0.0))
}
