use calclib::{error::CalcLibError, evaluator::evaluate, numformat::NumberFormat};

pub enum EvalResult {
    Success { expression: String, value: f64 },
    Failure(CalcLibError),
}

/// Evaluate the current input and update the result and history
pub(crate) fn evaluate_input(input: &str, numformat: NumberFormat) -> EvalResult {
    let expression = input
        .replace('×', "*")
        .replace('÷', "/")
        .replace('−', "-")
        .replace('π', format!("({})", &std::f64::consts::PI).as_str())
        .replace('e', format!("({})", std::f64::consts::E).as_str())
        .replace('²', "^2")
        .replace('³', "^3")
        .replace('√', "sqrt")
        .replace('∛', "cbrt")
        .replace("log₂", "logtwo");

    match evaluate(&expression, numformat) {
        Ok(result) => EvalResult::Success {
            expression: input.to_string(),
            value: result,
        },
        Err(e) => EvalResult::Failure(e),
    }
}

/// Substitute certain characters with their calc lib equivalents
pub(crate) fn substitute(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '*' => '×',
            '/' => '÷',
            '-' => '−',
            _ => c,
        })
        .collect()
}

pub(crate) fn get_paren_count(input: &str) -> i32 {
    let mut opening = 0;
    let mut closing = 0;

    for c in input.chars() {
        match c {
            '(' => opening += 1,
            ')' => closing += 1,
            _ => (),
        };
    }

    opening - closing
}

/// Inserts `text` into `input` at `cursor_pos` (char index), or appends if None.
/// Returns the new cursor position (after the inserted text).
pub(crate) fn insert_at_cursor(input: &mut String, text: &str, cursor_pos: Option<usize>) -> usize {
    match cursor_pos {
        Some(pos) => {
            let byte_pos = input
                .char_indices()
                .nth(pos)
                .map(|(i, _)| i)
                .unwrap_or(input.len());
            input.insert_str(byte_pos, text);
            pos + text.chars().count()
        }
        None => {
            input.push_str(text);
            input.chars().count()
        }
    }
}

/// Format an f64 result for display, respecting the active number format.
pub(crate) fn format_f64(value: f64, format: NumberFormat) -> String {
    if value.fract() == 0.0 && value.is_finite() && value.abs() <= i64::MAX as f64 {
        let n = value as i64;
        match format {
            NumberFormat::Decimal => format!("{}", n),
            NumberFormat::Hexadecimal => {
                if n < 0 {
                    format!("-{:X}", n.unsigned_abs())
                } else {
                    format!("{:X}", n)
                }
            }
            NumberFormat::Binary => {
                if n < 0 {
                    format!("-{:b}", n.unsigned_abs())
                } else {
                    format!("{:b}", n)
                }
            }
        }
    } else {
        format!("{}", value)
    }
}
