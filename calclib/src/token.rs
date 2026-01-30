#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(u32),
    Plus,
    Minus,
    Multiply,
    Divide,
    LParen,
    RParen,
    Percent,
    Period,
    Caret,
    Equal,
    Exclamation,
}

pub fn lookup_token(ch: char) -> Result<Option<Token>, String> {
    match ch {
        '=' => Ok(Some(Token::Equal)),
        '(' => Ok(Some(Token::LParen)),
        ')' => Ok(Some(Token::RParen)),
        '+' => Ok(Some(Token::Plus)),
        '-' => Ok(Some(Token::Minus)),
        '*' => Ok(Some(Token::Multiply)),
        '/' => Ok(Some(Token::Divide)),
        '×' => Ok(Some(Token::Multiply)),
        '÷' => Ok(Some(Token::Divide)),
        '^' => Ok(Some(Token::Caret)),
        '%' => Ok(Some(Token::Percent)),
        '.' => Ok(Some(Token::Period)),
        '!' => Ok(Some(Token::Exclamation)),
        '0'..='9' => {
            if let Some(n) = ch.to_digit(10) {
                return Ok(Some(Token::Number(n)));
            }

            Err(format!("Error parsing number: {}", ch))
        }

        _ => Err(format!("Unknown type: {}", ch)),
    }
}
