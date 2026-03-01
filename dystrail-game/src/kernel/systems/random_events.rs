#[must_use]
pub fn sanitize_event_weight_mult(weight_mult: f32) -> f32 {
    if weight_mult.is_finite() && weight_mult >= 0.0 {
        weight_mult
    } else {
        1.0
    }
}
