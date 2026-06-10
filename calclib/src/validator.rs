use crate::numformat::NumberFormat;

/// Validates if the input character is allowed in developer mode (binary, hex, or decimal),
/// which disallows the decimal point in addition to normal format restrictions.
pub fn validate_developer(input: &char, number_format: NumberFormat) -> bool {
    if *input == 'e' && number_format == NumberFormat::Decimal {
        return false;
    }

    if *input == '.' {
        return false;
    }

    validate(input, number_format)
}

/// Validates if the input character is one of the allowed mathematical symbols or digits.
pub fn validate(input: &char, number_format: NumberFormat) -> bool {
    let symbols_match = matches!(
        input,
        '+' | '-' | '*' | '/' | '(' | ')' | '^' | '=' | '!' | '×' | '÷' | '−'
    );

    match number_format {
        NumberFormat::Decimal => {
            symbols_match || input.is_ascii_digit() || matches!(input, '.' | 'e')
        }
        NumberFormat::Hexadecimal => symbols_match || input.is_ascii_hexdigit(),
        NumberFormat::Binary => symbols_match || matches!(input, '0' | '1'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_with_valid_bin_chars() {
        let valid_chars = vec![
            '0', '1', '+', '-', '*', '/', '(', ')', '^', '!', '=', '×', '÷', '−',
        ];

        for ch in valid_chars {
            assert!(
                validate(&ch, NumberFormat::Binary),
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
            ',', '.', '?', '\\', '|', '~', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
            'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A',
            'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
            'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '2', '3', '4', '5', '6', '7', '8', '9',
        ];

        for ch in invalid_chars {
            assert!(
                !validate(&ch, NumberFormat::Binary),
                "Character '{}' should be invalid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_valid_dec_chars() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '*', '/', '(', ')', '.',
            '^', '!', '=', '×', '÷', '−', 'e',
        ];

        for ch in valid_chars {
            assert!(
                validate(&ch, NumberFormat::Decimal),
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
            ',', '?', '\\', '|', '~', '`', 'a', 'b', 'c', 'd', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
            'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C',
            'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
            'U', 'V', 'W', 'X', 'Y', 'Z',
        ];

        for ch in invalid_chars {
            assert!(
                !validate(&ch, NumberFormat::Decimal),
                "Character '{}' should be invalid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_valid_hex_chars() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A',
            'B', 'C', 'D', 'E', 'F', '+', '-', '*', '/', '(', ')', '^', '!', '=', '×', '÷', '−',
        ];

        for ch in valid_chars {
            assert!(
                validate(&ch, NumberFormat::Hexadecimal),
                "Character '{}' should be valid",
                ch
            );
        }
    }

    #[test]
    fn test_validate_with_invalid_hex_chars() {
        // Invalid insert action
        let invalid_chars = vec![
            '.', '@', '#', '$', '%', '&', '_', '[', ']', '{', '}', ';', ':', '"', '\'', '<', '>',
            ',', '?', '\\', '|', '~', '`', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
            'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ',
        ];

        for ch in invalid_chars {
            assert!(
                !validate(&ch, NumberFormat::Hexadecimal),
                "Character '{}' should be invalid",
                ch
            );
        }
    }

    // --- Developer mode tests ---

    #[test]
    fn test_developer_decimal_blocks_period() {
        assert!(
            !validate_developer(&'.', NumberFormat::Decimal),
            "'.' should be blocked in developer decimal mode"
        );
    }

    #[test]
    fn test_developer_hex_blocks_period() {
        assert!(
            !validate_developer(&'.', NumberFormat::Hexadecimal),
            "'.' should be blocked in developer hex mode"
        );
    }

    #[test]
    fn test_developer_binary_blocks_period() {
        assert!(
            !validate_developer(&'.', NumberFormat::Binary),
            "'.' should be blocked in developer binary mode"
        );
    }

    #[test]
    fn test_developer_decimal_allows_digits_and_operators() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '*', '/', '(', ')', '^',
            '!', '=', '×', '÷', '−',
        ];
        for ch in valid_chars {
            assert!(
                validate_developer(&ch, NumberFormat::Decimal),
                "Character '{}' should be valid in developer decimal mode",
                ch
            );
        }
    }

    #[test]
    fn test_developer_hex_allows_hex_digits_and_operators() {
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A',
            'B', 'C', 'D', 'E', 'F', '+', '-', '*', '/', '(', ')', '^', '!', '=', '×', '÷', '−',
        ];
        for ch in valid_chars {
            assert!(
                validate_developer(&ch, NumberFormat::Hexadecimal),
                "Character '{}' should be valid in developer hex mode",
                ch
            );
        }
    }

    #[test]
    fn test_developer_hex_blocks_non_hex_letters() {
        let invalid_chars = vec!['g', 'h', 'z', 'G', 'H', 'Z'];
        for ch in invalid_chars {
            assert!(
                !validate_developer(&ch, NumberFormat::Hexadecimal),
                "Character '{}' should be invalid in developer hex mode",
                ch
            );
        }
    }

    #[test]
    fn test_developer_binary_allows_only_zero_one_and_operators() {
        let valid_chars = vec![
            '0', '1', '+', '-', '*', '/', '(', ')', '^', '!', '=', '×', '÷', '−',
        ];
        for ch in valid_chars {
            assert!(
                validate_developer(&ch, NumberFormat::Binary),
                "Character '{}' should be valid in developer binary mode",
                ch
            );
        }
    }

    #[test]
    fn test_developer_binary_blocks_digits_above_one() {
        let invalid_chars = vec!['2', '3', '4', '5', '6', '7', '8', '9'];
        for ch in invalid_chars {
            assert!(
                !validate_developer(&ch, NumberFormat::Binary),
                "Character '{}' should be invalid in developer binary mode",
                ch
            );
        }
    }
}
