use crate::ast::Expression::{self, Function, Infix, Number, Prefix, Unary};
use crate::error::CalcLibError;
use crate::numformat::NumberFormat;
use crate::parser::Parser;
use crate::token::{FunctionType, Token};
use crate::utils::{change_sign, is_integer, is_negative};
use statrs::function::{factorial, gamma::gamma};

enum BitOps {
    AND,
    OR,
    NAND,
    NOR,
    XOR,
    XNOR,
    LSHIFT,
    RSHIFT,
    MOD,
}

pub fn evaluate(input: &str, number_format: NumberFormat) -> Result<f64, CalcLibError> {
    let mut parser = Parser::new();
    let parse_val = parser.parse(input, number_format);

    match parse_val {
        Ok(v) => match v {
            Some(ex) => evaluate_expression(ex),
            None => Err(CalcLibError::InvalidExpression(input.to_string())),
        },
        Err(e) => Err(e),
    }
}

fn evaluate_expression(expression: Expression) -> Result<f64, CalcLibError> {
    match expression {
        Number { value } => Ok(value),
        Infix {
            left,
            operator,
            right,
        } => {
            let left_num = evaluate_expression(*left)?;
            let right_num = evaluate_expression(*right)?;

            match operator {
                Token::And => bitwise_operation(left_num, right_num, BitOps::AND),
                Token::Or => bitwise_operation(left_num, right_num, BitOps::OR),
                Token::Nand => bitwise_operation(left_num, right_num, BitOps::NAND),
                Token::Nor => bitwise_operation(left_num, right_num, BitOps::NOR),
                Token::Xor => bitwise_operation(left_num, right_num, BitOps::XOR),
                Token::Xnor => bitwise_operation(left_num, right_num, BitOps::XNOR),
                Token::Lshift => bitwise_operation(left_num, right_num, BitOps::LSHIFT),
                Token::Rshift => bitwise_operation(left_num, right_num, BitOps::RSHIFT),
                Token::Mod => bitwise_operation(left_num, right_num, BitOps::MOD),
                Token::Plus => Ok(left_num + right_num),
                Token::Minus => Ok(left_num - right_num),
                Token::Multiply => Ok(left_num * right_num),
                Token::Divide => {
                    if right_num == 0.0 {
                        Err(CalcLibError::DivisionByZero())
                    } else {
                        Ok(left_num / right_num)
                    }
                }
                Token::Caret => Ok(left_num.powf(right_num)),
                _ => Err(CalcLibError::UnsupportedOperator()),
            }
        }
        Prefix { operator, right } => {
            let right_num = evaluate_expression(*right)?;

            match operator {
                Token::Minus => Ok(-right_num),
                _ => Err(CalcLibError::UnsupportedOperator()),
            }
        }
        Unary {
            operator,
            expression,
        } => {
            let expr_num = evaluate_expression(*expression)?;

            match operator {
                Token::Exclamation => Ok(calc_factorial(expr_num)),
                _ => Err(CalcLibError::UnsupportedOperator()),
            }
        }

        Function { function, argument } => {
            let expr_num = evaluate_expression(*argument)?;

            match function {
                FunctionType::Log => Ok(expr_num.log10()),
                FunctionType::Ln => Ok(expr_num.ln()),
                FunctionType::LogTwo => Ok(expr_num.log2()),
                FunctionType::SqRt => Ok(expr_num.sqrt()),
                FunctionType::CbRt => Ok(expr_num.cbrt()),
                FunctionType::Abs => Ok(expr_num.abs()),
                FunctionType::Not => Ok((!(expr_num as i64)) as f64),
            }
        }
    }
}

fn bitwise_operation(left: f64, right: f64, op: BitOps) -> Result<f64, CalcLibError> {
    let left = left as i64;
    let right = right as i64;

    let result = match op {
        BitOps::AND => left & right,
        BitOps::OR => left | right,
        BitOps::NAND => !(left & right),
        BitOps::NOR => !(left | right),
        BitOps::XOR => left ^ right,
        BitOps::XNOR => !(left ^ right),
        BitOps::LSHIFT | BitOps::RSHIFT => {
            if right < 0 || right >= 64 {
                return Err(CalcLibError::ShiftOverflow());
            }
            if matches!(op, BitOps::LSHIFT) {
                left << right
            } else {
                left >> right
            }
        }
        BitOps::MOD => left % right,
    };

    Ok(result as f64)
}

/// Computes the factorial of a non-negative integer n.
fn calc_factorial(n: f64) -> f64 {
    // if it is negative, I want to flip the sign, compute the factorial, and then flip the sign
    // back at the end. This is because the factorial of a negative number is not defined, but we
    // can use the gamma function to compute it for negative numbers as well.
    let neg = is_negative(n);
    let num = n.abs();

    if is_integer(n) {
        let integer: u64 = num as u64;

        let result = factorial::factorial(integer);

        change_sign(result, neg)
    } else {
        change_sign(gamma(num + 1.0), neg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int_value(num: f64) -> Option<i64> {
        (is_integer(num) && num >= i64::MIN as f64 && num <= i64::MAX as f64)
            .then(|| num.trunc() as i64)
    }

    #[test]
    fn test_evaluate_int_expression() {
        let result = evaluate("42", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result));
        assert_eq!(int_value(eval_result), Some(42));
    }

    #[test]
    fn test_evaluate_simple_expression() {
        let input = vec![("2+3", 5), ("10-4", 6), ("6*7", 42), ("20/5", 4)];

        for i in input {
            let result = evaluate(i.0, NumberFormat::Decimal);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert!(is_integer(eval_result));
            assert_eq!(int_value(eval_result), Some(i.1));
        }
    }

    #[test]
    fn test_evaluate_expression_with_prefix() {
        let input = vec![("-5", -5), ("-(-3)", 3), ("-(2+3)", -5), ("-4+7", 3)];

        for i in input {
            let result = evaluate(i.0, NumberFormat::Decimal);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert!(is_integer(eval_result));
            assert_eq!(int_value(eval_result), Some(i.1));
        }
    }

    #[test]
    fn test_evaluate_division_by_zero() {
        let result = evaluate("10/0", NumberFormat::Decimal);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), CalcLibError::DivisionByZero());
    }

    #[test]
    fn test_evaluate_nested_expression() {
        let result = evaluate("2*(3+4)", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result));
        assert_eq!(int_value(eval_result), Some(14));
    }

    #[test]
    fn test_evaluate_factorial_expressions() {
        let result = evaluate("5!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result));
        assert_eq!(int_value(eval_result), Some(120));
    }

    #[test]
    fn test_evaluate_factorial_with_negative_expressions() {
        let result = evaluate("-5!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result));
        assert_eq!(int_value(eval_result), Some(-120));
    }

    #[test]
    fn test_evaluate_factorial_float_expressions() {
        let result = evaluate("2.3!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result));
        assert_eq!(eval_result, 2.6834373819557666);
    }

    #[test]
    fn test_evaluate_factorial_negative_float_expressions() {
        let result = evaluate("-2.3!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result));
        assert_eq!(eval_result, -2.6834373819557666);
    }

    #[test]
    fn test_evaluate_factorial_limit_expressions() {
        let result = evaluate("170!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result));
        assert_eq!(int_value(eval_result), None);
        assert_eq!(eval_result, 7.257415615307994e306);
    }

    #[test]
    fn test_evaluate_factorial_overflow_expressions() {
        let result = evaluate("171!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result));
        assert_eq!(eval_result, f64::INFINITY);
    }

    #[test]
    fn test_evaluate_factorial_negative_limit_expressions() {
        let result = evaluate("-170!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result));
        assert_eq!(int_value(eval_result), None);
        assert_eq!(eval_result, -7.257415615307994e306);
    }

    #[test]
    fn test_evaluate_factorial_negative_overflow_expressions() {
        let result = evaluate("-171!", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result));
        assert_eq!(eval_result, f64::NEG_INFINITY);
    }

    #[test]
    fn test_evaluate_factorial_function_of_zero() {
        let result = calc_factorial(0.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_evaluate_type_mismatch() {
        let result = evaluate("2+3.5", NumberFormat::Decimal);
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert_eq!(eval_result, 5.5);
    }

    #[test]
    fn test_evaluate_powers() {
        let inputs = vec![
            ("3^2", 9.0),
            ("2^3", 8.0),
            ("2^8", 256.0),
            ("2.391^2", 5.716881),
        ];

        for (input, expected) in inputs {
            let result = evaluate(input, NumberFormat::Decimal);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert_eq!(eval_result, expected);
        }
    }

    // --- Bitwise operations ---
    // Reference values (all formats reduce to the same i64 operands):
    //   decimal 12 = hex C = binary 1100
    //   decimal 10 = hex A = binary 1010
    //
    //   AND:   12 & 10  = 8   (0b1000)
    //   OR:    12 | 10  = 14  (0b1110)
    //   NAND: !(12 & 10) = -9
    //   NOR:  !(12 | 10) = -15
    //   XOR:   12 ^ 10  = 6   (0b0110)
    //   XNOR: !(12 ^ 10) = -7
    //   LSHIFT: 1 << 3  = 8
    //   RSHIFT: 8 >> 3  = 1
    //   MOD: 12 % 5 = 2

    #[test]
    fn test_bitwise_and() {
        assert_eq!(evaluate("12 AND 10", NumberFormat::Decimal).unwrap(), 8.0);
        assert_eq!(evaluate("C AND A", NumberFormat::Hexadecimal).unwrap(), 8.0);
        assert_eq!(
            evaluate("1100 AND 1010", NumberFormat::Binary).unwrap(),
            8.0
        );
    }

    #[test]
    fn test_bitwise_or() {
        assert_eq!(evaluate("12 OR 10", NumberFormat::Decimal).unwrap(), 14.0);
        assert_eq!(evaluate("C OR A", NumberFormat::Hexadecimal).unwrap(), 14.0);
        assert_eq!(
            evaluate("1100 OR 1010", NumberFormat::Binary).unwrap(),
            14.0
        );
    }

    #[test]
    fn test_bitwise_nand() {
        assert_eq!(evaluate("12 NAND 10", NumberFormat::Decimal).unwrap(), -9.0);
        assert_eq!(
            evaluate("C NAND A", NumberFormat::Hexadecimal).unwrap(),
            -9.0
        );
        assert_eq!(
            evaluate("1100 NAND 1010", NumberFormat::Binary).unwrap(),
            -9.0
        );
    }

    #[test]
    fn test_bitwise_nor() {
        assert_eq!(evaluate("12 NOR 10", NumberFormat::Decimal).unwrap(), -15.0);
        assert_eq!(
            evaluate("C NOR A", NumberFormat::Hexadecimal).unwrap(),
            -15.0
        );
        assert_eq!(
            evaluate("1100 NOR 1010", NumberFormat::Binary).unwrap(),
            -15.0
        );
    }

    #[test]
    fn test_bitwise_xor() {
        assert_eq!(evaluate("12 XOR 10", NumberFormat::Decimal).unwrap(), 6.0);
        assert_eq!(evaluate("C XOR A", NumberFormat::Hexadecimal).unwrap(), 6.0);
        assert_eq!(
            evaluate("1100 XOR 1010", NumberFormat::Binary).unwrap(),
            6.0
        );
    }

    #[test]
    fn test_bitwise_xnor() {
        assert_eq!(evaluate("12 XNOR 10", NumberFormat::Decimal).unwrap(), -7.0);
        assert_eq!(
            evaluate("C XNOR A", NumberFormat::Hexadecimal).unwrap(),
            -7.0
        );
        assert_eq!(
            evaluate("1100 XNOR 1010", NumberFormat::Binary).unwrap(),
            -7.0
        );
    }

    #[test]
    fn test_bitwise_lshift() {
        assert_eq!(evaluate("1 « 3", NumberFormat::Decimal).unwrap(), 8.0);
        assert_eq!(evaluate("1 « 3", NumberFormat::Hexadecimal).unwrap(), 8.0);
        assert_eq!(evaluate("1 « 11", NumberFormat::Binary).unwrap(), 8.0);
    }

    #[test]
    fn test_bitwise_rshift() {
        assert_eq!(evaluate("8 » 3", NumberFormat::Decimal).unwrap(), 1.0);
        assert_eq!(evaluate("8 » 3", NumberFormat::Hexadecimal).unwrap(), 1.0);
        assert_eq!(evaluate("1000 » 11", NumberFormat::Binary).unwrap(), 1.0);
    }

    #[test]
    fn test_bitwise_and_with_zero() {
        assert_eq!(evaluate("255 AND 0", NumberFormat::Decimal).unwrap(), 0.0);
        assert_eq!(evaluate("0 AND 255", NumberFormat::Decimal).unwrap(), 0.0);
    }

    #[test]
    fn test_bitwise_or_with_zero() {
        assert_eq!(evaluate("255 OR 0", NumberFormat::Decimal).unwrap(), 255.0);
        assert_eq!(evaluate("0 OR 0", NumberFormat::Decimal).unwrap(), 0.0);
    }

    #[test]
    fn test_bitwise_xor_with_self() {
        // Any value XOR itself is 0
        assert_eq!(evaluate("42 XOR 42", NumberFormat::Decimal).unwrap(), 0.0);
    }

    #[test]
    fn test_bitwise_and_with_self() {
        // Any value AND itself is itself
        assert_eq!(evaluate("42 AND 42", NumberFormat::Decimal).unwrap(), 42.0);
    }

    #[test]
    fn test_bitwise_lshift_by_zero() {
        assert_eq!(evaluate("5 « 0", NumberFormat::Decimal).unwrap(), 5.0);
    }

    #[test]
    fn test_bitwise_rshift_by_zero() {
        assert_eq!(evaluate("5 » 0", NumberFormat::Decimal).unwrap(), 5.0);
    }

    #[test]
    fn test_bitwise_nand_with_zero() {
        // !(0 & 0) = !0 = -1
        assert_eq!(evaluate("0 NAND 0", NumberFormat::Decimal).unwrap(), -1.0);
        // !(x & 0) = !0 = -1 for any x
        assert_eq!(evaluate("255 NAND 0", NumberFormat::Decimal).unwrap(), -1.0);
    }

    #[test]
    fn test_bitwise_nor_with_zero() {
        // !(0 | 0) = !0 = -1
        assert_eq!(evaluate("0 NOR 0", NumberFormat::Decimal).unwrap(), -1.0);
    }

    #[test]
    fn test_bitwise_xnor_with_self() {
        // !(x ^ x) = !0 = -1
        assert_eq!(evaluate("42 XNOR 42", NumberFormat::Decimal).unwrap(), -1.0);
    }

    #[test]
    fn test_bitwise_float_operands_are_truncated() {
        // 5.9 truncates to 5 (0b0101), 3.9 truncates to 3 (0b0011)
        assert_eq!(evaluate("5.9 AND 3.9", NumberFormat::Decimal).unwrap(), 1.0); // 5 & 3 = 1, not 6 & 4 = 4
        assert_eq!(evaluate("5.9 OR 3.9", NumberFormat::Decimal).unwrap(), 7.0); // 5 | 3 = 7
        assert_eq!(evaluate("5.9 XOR 3.9", NumberFormat::Decimal).unwrap(), 6.0);
        // 5 ^ 3 = 6
    }

    #[test]
    fn test_bitwise_negative_operands() {
        // -1 in two's complement is all 1s, so:
        // -1 AND x = x
        assert_eq!(evaluate("-1 AND 5", NumberFormat::Decimal).unwrap(), 5.0);
        // -1 OR x = -1
        assert_eq!(evaluate("-1 OR 5", NumberFormat::Decimal).unwrap(), -1.0);
        // -1 XOR x = ~x (bitwise complement)
        assert_eq!(evaluate("-1 XOR 5", NumberFormat::Decimal).unwrap(), -6.0);
    }

    #[test]
    fn test_bitwise_rshift_negative_number() {
        // Rust uses arithmetic (sign-extending) right shift for i64
        assert_eq!(evaluate("-8 » 1", NumberFormat::Decimal).unwrap(), -4.0);
        assert_eq!(evaluate("-1 » 1", NumberFormat::Decimal).unwrap(), -1.0); // sign extends forever
    }

    #[test]
    fn test_bitwise_lshift_overflow_returns_error() {
        let result = evaluate("1 « 64", NumberFormat::Decimal);
        assert_eq!(result, Err(CalcLibError::ShiftOverflow()));
    }

    #[test]
    fn test_bitwise_rshift_overflow_returns_error() {
        let result = evaluate("1 » 64", NumberFormat::Decimal);
        assert_eq!(result, Err(CalcLibError::ShiftOverflow()));
    }

    #[test]
    fn test_bitwise_shift_negative_amount_returns_error() {
        assert_eq!(
            evaluate("1 « -1", NumberFormat::Decimal),
            Err(CalcLibError::ShiftOverflow())
        );
        assert_eq!(
            evaluate("1 » -1", NumberFormat::Decimal),
            Err(CalcLibError::ShiftOverflow())
        );
    }

    #[test]
    fn test_mod() {
        assert_eq!(evaluate("12 MOD 5", NumberFormat::Decimal).unwrap(), 2.0); // 12 % 5 = 2
        assert_eq!(
            evaluate("12 MOD 5", NumberFormat::Hexadecimal).unwrap(),
            3.0
        ); // 18 % 5 = 3
        assert_eq!(evaluate("1100 MOD 101", NumberFormat::Binary).unwrap(), 2.0);
        // 12 % 5 = 2
    }

    #[test]
    fn test_bitwise_shift_max_valid_amount() {
        // 63 is the largest valid shift amount
        assert_eq!(
            evaluate("1 « 63", NumberFormat::Decimal).unwrap(),
            i64::MIN as f64
        );
        assert_eq!(evaluate("1 » 63", NumberFormat::Decimal).unwrap(), 0.0);
    }

    #[test]
    fn test_bitwise_precedence_lower_than_arithmetic() {
        // BIT < ADD, so: 4+4 AND 6 = (4+4) AND 6 = 8 AND 6 = 0
        assert_eq!(evaluate("4+4 AND 6", NumberFormat::Decimal).unwrap(), 0.0);
        // 4+4 OR 3 = 8 OR 3 = 11
        assert_eq!(evaluate("4+4 OR 3", NumberFormat::Decimal).unwrap(), 11.0);
    }

    #[test]
    fn test_math_functions() {
        let inputs = vec![
            ("log(100)", 2.0),
            ("ln(12)", 2.4849066497880004),
            ("logtwo(8)", 3.0),
            ("cbrt(27)", 3.0),
            ("cbrt(8)", 2.0),
            ("abs(-5)", 5.0),
            ("abs(3)", 3.0),
            ("NOT(4)", -5.0),
        ];

        for (input, expected) in inputs {
            let result = evaluate(input, NumberFormat::Decimal);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert_eq!(eval_result, expected);
        }
    }
}
