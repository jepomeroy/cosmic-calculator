use crate::ast::Expression::{self, Function, Infix, Number, Prefix, Unary};
use crate::error::CalcLibError;
use crate::numformat::NumberFormat;
use crate::parser::Parser;
use crate::token::{FunctionType, Token};
use crate::utils::{change_sign, is_integer, is_negative};
use statrs::function::{factorial, gamma::gamma};

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
            }
        }
    }
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
        ];

        for (input, expected) in inputs {
            let result = evaluate(input, NumberFormat::Decimal);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert_eq!(eval_result, expected);
        }
    }
}
