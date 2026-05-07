use core::fmt;
use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use dioxus::{dioxus_core::AttributeValue, prelude::IntoAttributeValue};

#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Default)]
pub struct ImStr {
    pub(crate) inner: Rc<str>,
}

impl ImStr {
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn as_rc(&self) -> Rc<str> {
        self.inner.clone()
    }
}

impl From<String> for ImStr {
    fn from(s: String) -> Self {
        Self {
            inner: Rc::<str>::from(s),
        }
    }
}

impl From<&str> for ImStr {
    fn from(s: &str) -> Self {
        Self {
            inner: Rc::<str>::from(s),
        }
    }
}

impl From<Rc<str>> for ImStr {
    fn from(s: Rc<str>) -> Self {
        Self { inner: s }
    }
}

impl IntoAttributeValue for ImStr {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Text(self.inner.to_string())
    }
}

impl Display for ImStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Formats a floating-point hours value with the given decimal precision,
/// normalising negative zero (and tiny negatives that round to zero) so they
/// render as `0.00` (or the precision-appropriate string) instead of `-0.00`.
///
/// Without this helper, `-0.0049_f32` formatted as `{:.2}` becomes `-0.00`.
/// Same with literal `-0.0_f32`. Both are visually noisy in HR views where
/// "the balance is zero" is the meaningful state. We round to the target
/// precision first, then clamp `±0.0` to `+0.0` before formatting.
pub fn format_hours(value: f32, decimals: usize) -> String {
    let factor = 10f32.powi(decimals as i32);
    let rounded = (value * factor).round() / factor;
    // After rounding, treat ±0.0 as +0.0 so the sign disappears.
    let normalized = if rounded == 0.0 { 0.0_f32 } else { rounded };
    format!("{:.*}", decimals, normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_hours_zero_renders_without_sign() {
        assert_eq!(format_hours(0.0_f32, 2), "0.00");
    }

    #[test]
    fn format_hours_negative_zero_renders_without_sign() {
        assert_eq!(format_hours(-0.0_f32, 2), "0.00");
    }

    #[test]
    fn format_hours_tiny_negative_rounds_to_positive_zero() {
        assert_eq!(format_hours(-0.0049_f32, 2), "0.00");
        assert_eq!(format_hours(-0.0001_f32, 2), "0.00");
    }

    #[test]
    fn format_hours_tiny_positive_rounds_to_zero() {
        assert_eq!(format_hours(0.0049_f32, 2), "0.00");
    }

    #[test]
    fn format_hours_preserves_real_negatives() {
        assert_eq!(format_hours(-0.5_f32, 2), "-0.50");
        assert_eq!(format_hours(-12.5_f32, 2), "-12.50");
    }

    #[test]
    fn format_hours_preserves_real_positives() {
        assert_eq!(format_hours(1.5_f32, 2), "1.50");
        assert_eq!(format_hours(12.345_f32, 2), "12.35");
    }

    #[test]
    fn format_hours_one_decimal_handles_negative_zero() {
        assert_eq!(format_hours(-0.04_f32, 1), "0.0");
        assert_eq!(format_hours(-0.0_f32, 1), "0.0");
    }

    #[test]
    fn format_hours_zero_decimals_handles_negative_zero() {
        assert_eq!(format_hours(-0.4_f32, 0), "0");
        assert_eq!(format_hours(-0.0_f32, 0), "0");
    }
}
