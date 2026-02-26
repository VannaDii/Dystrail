use crate::mechanics::otdeluxe90s::OtDeluxeTravelPolicy;

#[must_use]
pub fn otdeluxe_snow_speed_mult(snow_depth: f32, policy: &OtDeluxeTravelPolicy) -> f32 {
    if !snow_depth.is_finite() {
        return 1.0;
    }
    let penalty_per_in = policy.snow_speed_penalty_per_in.max(0.0);
    if penalty_per_in <= 0.0 {
        return 1.0;
    }
    let floor = policy.snow_speed_floor.clamp(0.0, 1.0);
    let depth = snow_depth.max(0.0);
    let mult = 1.0 - depth * penalty_per_in;
    mult.clamp(floor, 1.0)
}
