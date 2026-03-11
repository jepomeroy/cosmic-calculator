use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharSet {
    #[default]
    Decimal,
    Hexadecimal,
    Binary,
}

impl Display for CharSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharSet::Decimal => write!(f, "Decimal"),
            CharSet::Hexadecimal => write!(f, "Hexadecimal"),
            CharSet::Binary => write!(f, "Binary"),
        }
    }
}

impl CharSet {
    pub fn as_str(&self) -> &str {
        match self {
            CharSet::Decimal => "decimal",
            CharSet::Hexadecimal => "hexadecimal",
            CharSet::Binary => "binary",
        }
    }

    pub fn from_str(s: &str) -> Option<CharSet> {
        match s {
            "decimal" => Some(CharSet::Decimal),
            "hexadecimal" => Some(CharSet::Hexadecimal),
            "binary" => Some(CharSet::Binary),
            _ => None,
        }
    }
}

/// Validates if the input character is one of the allowed mathematical symbols or digits.
pub fn validate(input: &char, char_set: CharSet) -> bool {
    let symbols_match = matches!(
        input,
        '+' | '-' | '*' | '/' | '(' | ')' | '^' | '.' | '=' | '!' | '×' | '÷' | '−'
    );

    match char_set {
        CharSet::Decimal => symbols_match || matches!(input, '0'..='9'),
        CharSet::Hexadecimal => symbols_match || matches!(input, '0'..='9' | 'a'..='f' | 'A'..='F'),
        CharSet::Binary => symbols_match || matches!(input, '0' | '1'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_with_valid_bin_chars() {
        let valid_chars = vec![
            '0', '1', '+', '-', '*', '/', '(', ')', '.', '^', '!', '=', '×', '÷', '−',
        ];

        for ch in valid_chars {
            assert!(
                validate(&ch, CharSet::Binary),
                "Character '{}' should be valid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_invalid_bin_chars() {
        // Invalid insert action
        let invalid_chars = vec![
            ' ', '@', '#', '$', '%', '&', '_', '[', ']', '{', '}', ';', ':', '"', '\'', '<', '>',
            ',', '?', '\\', '|', '~', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
            'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
            'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
            'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '2', '3', '4', '5', '6', '7', '8', '9',
        ];

        for ch in invalid_chars {
            assert!(
                !validate(&ch, CharSet::Binary),
                "Character '{}' should be invalid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_valid_dec_chars() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '*', '/', '(', ')', '.',
            '^', '!', '=', '×', '÷', '−',
        ];

        for ch in valid_chars {
            assert!(
                validate(&ch, CharSet::Decimal),
                "Character '{}' should be valid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_invalid_dec_chars() {
        // Invalid insert action
        let invalid_chars = vec![
            ' ', '@', '#', '$', '%', '&', '_', '[', ']', '{', '}', ';', ':', '"', '\'', '<', '>',
            ',', '?', '\\', '|', '~', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
            'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
            'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
            'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];

        for ch in invalid_chars {
            assert!(
                !validate(&ch, CharSet::Decimal),
                "Character '{}' should be invalid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_valid_hex_chars() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A',
            'B', 'C', 'D', 'E', 'F', '+', '-', '*', '/', '(', ')', '.', '^', '!', '=', '×', '÷',
            '−',
        ];

        for ch in valid_chars {
            assert!(
                validate(&ch, CharSet::Hexadecimal),
                "Character '{}' should be valid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_invalid_hex_chars() {
        // Invalid insert action
        let invalid_chars = vec![
            ' ', '@', '#', '$', '%', '&', '_', '[', ']', '{', '}', ';', ':', '"', '\'', '<', '>',
            ',', '?', '\\', '|', '~', '`', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
            'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ',
        ];

        for ch in invalid_chars {
            assert!(
                !validate(&ch, CharSet::Hexadecimal),
                "Character '{}' should be invalid",
                ch
            );
        }
    }
}
