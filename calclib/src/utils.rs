///
pub(crate) fn change_sign(num: f64, make_negative: bool) -> f64 {
    if make_negative { -num.abs() } else { num.abs() }
}

pub(crate) fn is_integer(num: Option<f64>) -> bool {
    if let Some(f) = num {
        return f.fract() == 0.0;
    }
    false
}

pub(crate) fn is_negative(num: Option<f64>) -> bool {
    if let Some(x) = num {
        return x < 0.0;
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_integer() {
        let inputs = vec![
            (Some(1.0), true),
            (Some(1.23), false),
            (Some(-1.0), true),
            (Some(-1.999), false),
            (Some(0.0), true),
            (None, false),
        ];

        for i in inputs {
            assert_eq!(is_integer(i.0), i.1);
        }
    }

    #[test]
    fn test_is_negative() {
        let inputs = vec![
            (Some(1.0), false),
            (Some(-1.0), true),
            (Some(0.0), false),
            (None, false),
        ];

        for i in inputs {
            assert_eq!(is_negative(i.0), i.1);
        }
    }

    #[test]
    fn test_change_sign() {
        let inputs = vec![
            (1.0, false, 1.0),
            (1.0, true, -1.0),
            (-1.0, true, -1.0),
            (-1.0, false, 1.0),
            (0.0, false, 0.0),
            (0.0, true, -0.0),
        ];

        for i in inputs {
            assert_eq!(change_sign(i.0, i.1), i.2);
        }
    }
}
