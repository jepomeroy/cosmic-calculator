use crate::ast::Expression::{Infix, Number, Prefix, Unary};
use crate::parser::Parser;
use crate::utils::{change_sign, is_integer, is_negative};
use statrs::function::{factorial, gamma::gamma};

pub struct EvaluationResult {
    value: Option<f64>,
}

impl EvaluationResult {
    pub fn int_value(&self) -> Option<i64> {
        if is_integer(self.value) && self.value.map_or(false, |f| f.abs() <= i64::MAX as f64) {
            return self.value.map(|f| f.trunc() as i64);
        }
        None
    }

    pub fn value(&self) -> String {
        if let Some(f) = self.value {
            if is_integer(self.value) {
                if f.abs() <= i64::MAX as f64 {
                    return format!("{}", f.trunc() as i64);
                } else {
                    return format!("{:e}", f);
                }
            } else {
                return format!("{}", f);
            }
        } else {
            "NaN".to_string()
        }
    }
}

pub fn evaluate(input: String) -> Result<EvaluationResult, String> {
    let mut parser = Parser::new();
    let parse_val = parser.parse(input);

    match parse_val {
        Err(e) => Err(e),
        Ok(v) => {
            // println!("Parser output: {:?}", v);
            match v {
                Some(ex) => evaluate_expression(ex),
                None => Err("Invalid expression".to_string()),
            }
        }
    }
}

fn evaluate_expression(expression: crate::ast::Expression) -> Result<EvaluationResult, String> {
    match expression {
        Number { value } => Ok(EvaluationResult { value: Some(value) }),
        Infix {
            left,
            operator,
            right,
        } => {
            let left_val = evaluate_expression(*left)?;
            let right_val = evaluate_expression(*right)?;

            let left_num = left_val.value.unwrap();
            let right_num = right_val.value.unwrap();

            match operator {
                crate::token::Token::Plus => Ok(EvaluationResult {
                    value: Some(left_num + right_num),
                }),
                crate::token::Token::Minus => Ok(EvaluationResult {
                    value: Some(left_num - right_num),
                }),
                crate::token::Token::Multiply => Ok(EvaluationResult {
                    value: Some(left_num * right_num),
                }),
                crate::token::Token::Divide => {
                    if right_num == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(EvaluationResult {
                            value: Some(left_num / right_num),
                        })
                    }
                }
                _ => Err("Unsupported operator".to_string()),
            }
        }
        Prefix { operator, right } => {
            let right_val = evaluate_expression(*right)?;

            let right_num = right_val.value.unwrap();

            match operator {
                crate::token::Token::Minus => Ok(EvaluationResult {
                    value: Some(-right_num),
                }),
                _ => Err("Unsupported operator".to_string()),
            }
        }
        Unary {
            operator,
            expression,
        } => {
            let expr_val = evaluate_expression(*expression)?;

            let expr_num = expr_val.value;

            match operator {
                crate::token::Token::Exclamation => match calc_factorial(expr_num) {
                    Ok(result) => Ok(EvaluationResult {
                        value: Some(result),
                    }),
                    Err(_) => Err("Failed to compute factorial".to_string()),
                },
                _ => Err("Unsupported operator".to_string()),
            }
        }
    }
}

/// Computes the factorial of a non-negative integer n.
fn calc_factorial(n: Option<f64>) -> Result<f64, ()> {
    if n.is_none() {
        return Err(());
    }

    // if it is negative, I want to flip the sign, compute the factorial, and then flip the sign
    // back at the end. This is because the factorial of a negative number is not defined, but we
    // can use the gamma function to compute it for negative numbers as well.
    let neg = is_negative(n);
    let num = n.unwrap().abs();

    if is_integer(n) {
        let integer: u64 = num as u64;

        let result = factorial::factorial(integer);

        Ok(change_sign(result, neg))
    } else {
        Ok(change_sign(gamma(num + 1.0), neg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_int_expression() {
        let result = evaluate("42".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result.value));
        assert_eq!(eval_result.int_value(), Some(42));
    }

    #[test]
    fn test_evaluate_simple_expression() {
        let input = vec![
            ("2+3".to_string(), 5),
            ("10-4".to_string(), 6),
            ("6*7".to_string(), 42),
            ("20/5".to_string(), 4),
        ];

        for i in input {
            let result = evaluate(i.0);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert!(is_integer(eval_result.value));
            assert_eq!(eval_result.int_value(), Some(i.1));
        }
    }

    #[test]
    fn test_evaluate_expression_with_prefix() {
        let input = vec![
            ("-5".to_string(), -5),
            ("-(-3)".to_string(), 3),
            ("-(2+3)".to_string(), -5),
            ("-4+7".to_string(), 3),
        ];

        for i in input {
            let result = evaluate(i.0);
            assert!(result.is_ok());
            let eval_result = result.unwrap();
            assert!(is_integer(eval_result.value));
            assert_eq!(eval_result.int_value(), Some(i.1));
        }
    }

    #[test]
    fn test_evaluate_division_by_zero() {
        let result = evaluate("10/0".to_string());
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Division by zero".to_string());
    }

    #[test]
    fn test_evaluate_nested_expression() {
        let result = evaluate("2*(3+4)".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result.value));
        assert_eq!(eval_result.int_value(), Some(14));
    }

    #[test]
    fn test_evaluate_factoriacl_expressions() {
        let result = evaluate("5!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result.value));
        assert_eq!(eval_result.int_value(), Some(120));
    }

    #[test]
    fn test_evaluate_factoriacl_with_negative_expressions() {
        let result = evaluate("-5!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result.value));
        assert_eq!(eval_result.int_value(), Some(-120));
    }

    #[test]
    fn test_evaluate_factoriacl_float_expressions() {
        let result = evaluate("2.3!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result.value));
        assert_eq!(eval_result.value, Some(2.6834373819557666));
    }

    #[test]
    fn test_evaluate_factoriacl_negative_float_expressions() {
        let result = evaluate("-2.3!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result.value));
        assert_eq!(eval_result.value, Some(-2.6834373819557666));
    }

    #[test]
    fn test_evaluate_factoriacl_limit_expressions() {
        let result = evaluate("170!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result.value));
        assert_eq!(eval_result.int_value(), None);
        assert_eq!(eval_result.value(), "7.257415615307994e306");
    }

    #[test]
    fn test_evaluate_factoriacl_overflow_expressions() {
        let result = evaluate("171!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result.value));
        assert_eq!(eval_result.value, Some(f64::INFINITY));
    }

    #[test]
    fn test_evaluate_factoriacl_negative_limit_expressions() {
        let result = evaluate("-170!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(is_integer(eval_result.value));
        assert_eq!(eval_result.int_value(), None);
        assert_eq!(eval_result.value(), "-7.257415615307994e306");
    }

    #[test]
    fn test_evaluate_factoriacl_negative_overflow_expressions() {
        let result = evaluate("-171!".to_string());
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert!(!is_integer(eval_result.value));
        assert_eq!(eval_result.value, Some(f64::NEG_INFINITY));
    }

    #[test]
    fn test_evaluate_factorial_function_of_zero() {
        let result = calc_factorial(Some(0.0));
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert_eq!(eval_result, 1.0);
    }

    #[test]
    #[ignore = "Not implemented yet"]
    fn test_evaluate_type_mismatch() {
        let result = evaluate("2+3.5".to_string());
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Type mismatch: expected integers".to_string()
        );
    }

    #[test]
    fn test_evaluate_unsupported_operator() {
        let result = evaluate("2^3".to_string());
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Unsupported operator".to_string());
    }
}
