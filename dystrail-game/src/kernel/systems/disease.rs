use crate::disease::DiseaseEffects;
use crate::otdeluxe_state::OtDeluxeInventory;

#[must_use]
pub const fn sanitize_disease_multiplier(mult: f32) -> f32 {
    if mult.is_finite() { mult.max(0.0) } else { 1.0 }
}

#[must_use]
pub fn apply_otdeluxe_disease_effects(
    health_general: &mut u16,
    inventory: &mut OtDeluxeInventory,
    effects: &DiseaseEffects,
) -> f32 {
    if effects.health_general_delta != 0 {
        let current = i32::from(*health_general);
        let next = (current + effects.health_general_delta).max(0);
        *health_general = u16::try_from(next).unwrap_or(u16::MAX);
    }
    if effects.food_lbs_delta != 0 {
        let current = i32::from(inventory.food_lbs);
        let next = (current + effects.food_lbs_delta).max(0);
        inventory.food_lbs = u16::try_from(next).unwrap_or(u16::MAX);
    }
    sanitize_disease_multiplier(effects.travel_speed_mult)
}
