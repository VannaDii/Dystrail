//! Numeric conversion helpers centralizing safe numeric casts.

use num_traits::cast::cast;

/// Clamp a f64 to the f32 range and downcast, returning 0.0 for non-finite values.
#[must_use]
pub fn clamp_f64_to_f32(value: f64) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    let min = cast::<f32, f64>(f32::MIN).unwrap_or(f64::MIN);
    let max = cast::<f32, f64>(f32::MAX).unwrap_or(f64::MAX);
    let clamped = value.clamp(min, max);
    cast::<f64, f32>(clamped).unwrap_or(0.0)
}

/// Round a f64 and clamp it to the i32 range, returning 0 for NaN values.
#[must_use]
pub fn round_f64_to_i32(value: f64) -> i32 {
    if value.is_nan() {
        return 0;
    }
    let min = cast::<i32, f64>(i32::MIN).unwrap_or(f64::MIN);
    let max = cast::<i32, f64>(i32::MAX).unwrap_or(f64::MAX);
    let clamped = value.clamp(min, max).round();
    cast::<f64, i32>(clamped).unwrap_or(0)
}

/// Round a f32 and clamp it to the i32 range, returning 0 for NaN values.
#[must_use]
pub fn round_f32_to_i32(value: f32) -> i32 {
    round_f64_to_i32(f64::from(value))
}

/// Ceil a f64 and clamp it to the i64 range, returning 0 for non-finite values.
#[must_use]
pub fn ceil_f64_to_i64(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    let min = cast::<i64, f64>(i64::MIN).unwrap_or(f64::MIN);
    let max = cast::<i64, f64>(i64::MAX).unwrap_or(f64::MAX);
    let clamped = value.clamp(min, max).ceil();
    cast::<f64, i64>(clamped).unwrap_or(0)
}

/// Convert i64 to f64 while allowing precision loss in a single location.
#[must_use]
pub fn i64_to_f64(value: i64) -> f64 {
    cast::<i64, f64>(value).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_handles_non_finite() {
        assert!((clamp_f64_to_f32(f64::NAN) - 0.0).abs() < f32::EPSILON);
        assert!((clamp_f64_to_f32(f64::from(f32::MAX) * 2.0) - f32::MAX).abs() < f32::EPSILON);
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
