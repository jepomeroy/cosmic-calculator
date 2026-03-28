/// Forces `num` to be negative or positive depending on `make_negative`.
///
/// If `make_negative` is `true`, returns the negated absolute value of `num`.
/// If `make_negative` is `false`, returns the absolute value of `num`.
pub(crate) fn change_sign(num: f64, make_negative: bool) -> f64 {
    if make_negative { -num.abs() } else { num.abs() }
}

/// Returns `true` if `num` is a value whose fractional part is zero.
///
/// Returns `false` for any value with a non-zero fractional part.
pub(crate) fn is_integer(num: f64) -> bool {
    num.fract() == 0.0
}

/// Returns `true` if `num` is a value that is strictly less than zero.
///
/// Returns `false` for zero or any positive value.
pub(crate) fn is_negative(num: f64) -> bool {
    num < 0.0
}

/// Converts a binary string to an `f64` by parsing it as an unsigned integer.
///
/// For example, `"1010"` maps to `10.0` and `"11111111"` maps to `255.0`.
///
/// Returns `None` if the string contains characters other than `'0'` and `'1'`,
/// if the string is empty, or if the value exceeds `u64::MAX`.
pub(crate) fn bin_to_dec(bin: &str) -> Option<f64> {
    base_to_dec(bin, 2)
}

/// Converts a hexadecimal string to an `f64` by parsing it as an unsigned integer.
///
/// Both uppercase and lowercase hex digits are accepted. For example, `"f"` and
/// `"F"` both map to `15.0`, and `"ff"` maps to `255.0`.
///
/// Returns `None` if the string contains invalid hex characters, is empty, or
/// if the value exceeds `u64::MAX`.
pub(crate) fn hex_to_dec(hex: &str) -> Option<f64> {
    base_to_dec(hex, 16)
}

/// Parses `num_str` as an unsigned integer in the given `base` and casts it to `f64`.
///
/// Returns `None` if `num_str` is empty, contains characters invalid for `base`,
/// or overflows `u64`.
fn base_to_dec(num_str: &str, base: u32) -> Option<f64> {
    let n = u64::from_str_radix(num_str, base).ok()?;
    Some(n as f64)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_integer() {
        let inputs = vec![
            (1.0, true),
            (1.23, false),
            (-1.0, true),
            (-1.999, false),
            (0.0, true),
            // Large exact integer
            (1_000_000_000_000.0, true),
            // Infinity has no fractional part but is not a meaningful integer
            (f64::INFINITY, false),
            (f64::NEG_INFINITY, false),
            // NaN comparisons always return false
            (f64::NAN, false),
        ];

        for i in inputs {
            assert_eq!(is_integer(i.0), i.1);
        }
    }

    #[test]
    fn test_is_negative() {
        let inputs = vec![
            (1.0, false),
            (-1.0, true),
            (0.0, false),
            // Negative zero is not less than zero in IEEE 754
            (-0.0, false),
            (f64::NEG_INFINITY, true),
            (f64::INFINITY, false),
            // NaN comparisons always return false
            (f64::NAN, false),
        ];

        for i in inputs {
            assert_eq!(is_negative(i.0), i.1);
        }
    }

    #[test]
    fn test_change_sign() {
        let inputs: Vec<(f64, bool, f64)> = vec![
            (1.0, false, 1.0),
            (1.0, true, -1.0),
            (-1.0, true, -1.0),
            (-1.0, false, 1.0),
            (0.0, false, 0.0),
            (0.0, true, -0.0),
            (f64::INFINITY, false, f64::INFINITY),
            (f64::INFINITY, true, f64::NEG_INFINITY),
            (f64::NEG_INFINITY, false, f64::INFINITY),
            (f64::NEG_INFINITY, true, f64::NEG_INFINITY),
        ];

        for i in inputs {
            assert_eq!(change_sign(i.0, i.1), i.2);
        }

        // NaN stays NaN regardless of sign flag
        assert!(change_sign(f64::NAN, false).is_nan());
        assert!(change_sign(f64::NAN, true).is_nan());
    }

    #[test]
    fn test_hex_to_dec() {
        assert_eq!(hex_to_dec("0"), Some(0.0_f64));
        assert_eq!(hex_to_dec("f"), Some(15.0_f64));
        assert_eq!(hex_to_dec("F"), Some(15.0_f64));
        assert_eq!(hex_to_dec("ff"), Some(255.0_f64));
        assert_eq!(hex_to_dec("10"), Some(16.0_f64));
        assert_eq!(hex_to_dec("1a"), Some(26.0_f64));
        assert_eq!(hex_to_dec("deadbeef"), Some(3735928559.0_f64));
        // Leading zeros are valid
        assert_eq!(hex_to_dec("00ff"), Some(255.0_f64));
        // u64::MAX is the largest accepted value
        assert_eq!(hex_to_dec("FFFFFFFFFFFFFFFF"), Some(u64::MAX as f64));
        // One digit beyond u64::MAX overflows
        assert_eq!(hex_to_dec("10000000000000000"), None);

        // Invalid input returns None.
        assert_eq!(hex_to_dec("ZZZZZZZZZZZZZZZZ"), None);
        assert_eq!(hex_to_dec(""), None);
    }

    #[test]
    fn test_bin_to_dec() {
        assert_eq!(bin_to_dec("0"), Some(0.0_f64));
        assert_eq!(bin_to_dec("1"), Some(1.0_f64));
        assert_eq!(bin_to_dec("101"), Some(5.0_f64));
        assert_eq!(bin_to_dec("1010"), Some(10.0_f64));
        assert_eq!(bin_to_dec("11111111"), Some(255.0_f64));
        // Leading zeros are valid
        assert_eq!(bin_to_dec("00001010"), Some(10.0_f64));
        // 64 ones = u64::MAX
        assert_eq!(
            bin_to_dec("1111111111111111111111111111111111111111111111111111111111111111"),
            Some(u64::MAX as f64)
        );
        // 65 ones overflows u64
        assert_eq!(
            bin_to_dec("11111111111111111111111111111111111111111111111111111111111111111"),
            None
        );

        // Invalid input returns None.
        assert_eq!(bin_to_dec("not_binary"), None);
        assert_eq!(bin_to_dec(""), None);
    }

    #[test]
    fn test_base_to_dec_roundtrip() {
        // Integer values should round-trip through hex and binary.
        let values: &[u64] = &[0, 1, 15, 255, 1000, 9999999];
        for &v in values {
            let hex = format!("{:X}", v);
            assert_eq!(hex_to_dec(&hex), Some(v as f64));

            let bin = format!("{:b}", v);
            assert_eq!(bin_to_dec(&bin), Some(v as f64));
        }
    }
}
