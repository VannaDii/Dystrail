use crate::mechanics::otdeluxe90s::{OtDeluxe90sPolicy, OtDeluxeOccupation};
use crate::otdeluxe_state::OtDeluxeInventory;
use crate::vehicle::Part;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OtDeluxeSparePart {
    Wheel,
    Axle,
    Tongue,
}

#[must_use]
pub(crate) const fn otdeluxe_spare_for_breakdown(part: Part) -> OtDeluxeSparePart {
    match part {
        Part::Battery => OtDeluxeSparePart::Axle,
        Part::Alternator => OtDeluxeSparePart::Tongue,
        Part::Tire | Part::FuelPump => OtDeluxeSparePart::Wheel,
    }
}

#[must_use]
pub const fn consume_otdeluxe_spare_for_breakdown(
    inventory: &mut OtDeluxeInventory,
    part: Part,
) -> bool {
    match otdeluxe_spare_for_breakdown(part) {
        OtDeluxeSparePart::Wheel if inventory.spares_wheels > 0 => {
            inventory.spares_wheels -= 1;
            true
        }
        OtDeluxeSparePart::Axle if inventory.spares_axles > 0 => {
            inventory.spares_axles -= 1;
            true
        }
        OtDeluxeSparePart::Tongue if inventory.spares_tongues > 0 => {
            inventory.spares_tongues -= 1;
            true
        }
        _ => false,
    }
}

#[must_use]
pub const fn otdeluxe_mobility_failure_mult(
    occupation: Option<OtDeluxeOccupation>,
    policy: &OtDeluxe90sPolicy,
) -> f32 {
    if matches!(occupation, Some(OtDeluxeOccupation::Farmer)) {
        sanitize_multiplier(policy.occupation_advantages.mobility_failure_mult)
    } else {
        1.0
    }
}

const fn sanitize_multiplier(mult: f32) -> f32 {
    if mult.is_finite() { mult.max(0.0) } else { 1.0 }
}
