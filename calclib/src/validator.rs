/// Validates if the input character is one of the allowed mathematical symbols or digits.
pub fn validate(input: &char) -> bool {
    matches!(
        input,
        '0'..='9'
            | '+'
            | '-'
            | '*'
            | '/'
            | '('
            | ')'
            | '%'
            | '^'
            | '.'
            | '='
            | '!'
            | '×'
            | '÷'
            | '−'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_with_valid_chars() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '*', '/', '(', ')', '.',
            '^', '%', '!', '=', '×', '÷', '−',
        ];

        for ch in valid_chars {
            assert!(validate(&ch), "Character '{}' should be valid", ch);
        }
    }

    #[test]
    fn test_validate_with_invalid_chars() {
        // Invalid insert action
        let invalid_chars = vec![
            'a', 'b', 'c', ' ', '@', '#', '$', '&', '_', '[', ']', '{', '}', ';', ':', '"', '\'',
            '<', '>', ',', '?', '\\', '|', '~', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
            'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ',
        ];

        for ch in invalid_chars {
            assert!(!validate(&ch), "Character '{}' should be invalid", ch);
        }
    }
}
