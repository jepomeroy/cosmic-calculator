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
        Err(e) => return Err(e),
        Ok(v) => {
            println!("Parser output: {:?}", v);
            match v {
                Some(ex) => {
                    // Dummy evaluation logic for UI testing
                    println!("Parsed expression: {:?}", ex);
                    Ok(EvaluationResult {
                        float_value: None,
                        int_value: Some(12),
                        // float_value: Some(42.3),
                        // int_value: None,
                    })
                }
                None => return Err("No expression to evaluate".to_string()),
            }
        }
    }
}
