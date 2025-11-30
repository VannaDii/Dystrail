//! Numeric conversion helpers centralizing clippy-allowed casts.
#![allow(clippy::cast_possible_truncation)]

/// Clamp a f64 to the f32 range and downcast, returning 0.0 for non-finite values.
#[must_use]
pub fn clamp_f64_to_f32(value: f64) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    {
        value.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32
    }
}

/// Round a f64 and clamp it to the i32 range, returning 0 for NaN values.
#[must_use]
pub fn round_f64_to_i32(value: f64) -> i32 {
    if value.is_nan() {
        return 0;
    }
    value
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX))
        .round() as i32
}

/// Round a f32 and clamp it to the i32 range, returning 0 for NaN values.
#[must_use]
pub fn round_f32_to_i32(value: f32) -> i32 {
    round_f64_to_i32(f64::from(value))
}

/// Ceil a f64 and clamp it to the i64 range, returning 0 for non-finite values.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn ceil_f64_to_i64(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    #[allow(clippy::cast_precision_loss)]
    {
        value
            .clamp(i64_to_f64(i64::MIN), i64_to_f64(i64::MAX))
            .ceil() as i64
    }
}

/// Convert i64 to f64 while allowing precision loss in a single location.
#[must_use]
pub const fn i64_to_f64(value: i64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    {
        value as f64
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn clamp_handles_non_finite() {
        assert_eq!(clamp_f64_to_f32(f64::NAN), 0.0);
        assert_eq!(clamp_f64_to_f32(f64::from(f32::MAX) * 2.0), f32::MAX);
    }

    #[test]
    fn rounders_cover_ranges() {
        assert_eq!(round_f64_to_i32(1.6), 2);
        assert_eq!(round_f32_to_i32(f32::NAN), 0);
        assert_eq!(round_f64_to_i32(f64::from(i32::MAX) * 2.0), i32::MAX);
    }

    #[test]
    fn ceil_clamps_and_handles_nan() {
        assert_eq!(ceil_f64_to_i64(1.2), 2);
        assert_eq!(ceil_f64_to_i64(f64::NAN), 0);
    }
}
