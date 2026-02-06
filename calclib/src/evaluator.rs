use crate::ast::Expression::{Grouped, Infix, Integer, Prefix};
use crate::parser::Parser;

pub struct EvaluationResult {
    pub float_value: Option<f64>,
    pub int_value: Option<i64>,
}

impl EvaluationResult {
    pub fn is_float(&self) -> bool {
        self.float_value.is_some()
    }

    pub fn is_int(&self) -> bool {
        self.int_value.is_some()
    }

    pub fn value(&self) -> String {
        if let Some(f) = self.float_value {
            format!("{}", f)
        } else if let Some(i) = self.int_value {
            format!("{}", i)
        } else {
            "No value".to_string()
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
                None => Err("No expression to evaluate".to_string()),
            }
        }
    }
}

fn evaluate_expression(expression: crate::ast::Expression) -> Result<EvaluationResult, String> {
    match expression {
        Integer { value } => Ok(EvaluationResult {
            float_value: None,
            int_value: Some(value),
        }),
        Infix {
            left,
            operator,
            right,
        } => {
            let left_val = evaluate_expression(*left)?;
            let right_val = evaluate_expression(*right)?;

            if left_val.is_int() && right_val.is_int() {
                let left_int = left_val.int_value.unwrap();
                let right_int = right_val.int_value.unwrap();

                match operator {
                    crate::token::Token::Plus => Ok(EvaluationResult {
                        float_value: None,
                        int_value: Some(left_int + right_int),
                    }),
                    crate::token::Token::Minus => Ok(EvaluationResult {
                        float_value: None,
                        int_value: Some(left_int - right_int),
                    }),
                    crate::token::Token::Multiply => Ok(EvaluationResult {
                        float_value: None,
                        int_value: Some(left_int * right_int),
                    }),
                    crate::token::Token::Divide => {
                        if right_int == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(EvaluationResult {
                                float_value: None,
                                int_value: Some(left_int / right_int),
                            })
                        }
                    }
                    _ => Err("Unsupported operator".to_string()),
                }
            } else {
                Err("Type mismatch: expected integers".to_string())
            }
        }
        Prefix { operator, right } => {
            let right_val = evaluate_expression(*right)?;

            if right_val.is_int() {
                let right_int = right_val.int_value.unwrap();

                match operator {
                    crate::token::Token::Minus => Ok(EvaluationResult {
                        float_value: None,
                        int_value: Some(-right_int),
                    }),
                    _ => Err("Unsupported operator".to_string()),
                }
            } else {
                Err("Type mismatch: expected integers".to_string())
            }
        }

        _ => Err("Unsupported expression type".to_string()),
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
        assert!(eval_result.is_int());
        assert_eq!(eval_result.int_value, Some(42));
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
            assert!(eval_result.is_int());
            assert_eq!(eval_result.int_value, Some(i.1));
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
            assert!(eval_result.is_int());
            assert_eq!(eval_result.int_value, Some(i.1));
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
        assert!(eval_result.is_int());
        assert_eq!(eval_result.int_value, Some(14));
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
