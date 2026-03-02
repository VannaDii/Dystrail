use crate::journey::{EventDecisionTrace, RollValue, WeightedCandidate};
use crate::mechanics::otdeluxe90s::{OtDeluxe90sPolicy, OtDeluxeOccupation};
use crate::otdeluxe_state::OtDeluxeInventory;
use crate::vehicle::{Part, PartWeights};
use rand::Rng;

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

#[must_use]
pub fn sanitize_breakdown_max_chance(max_chance: f32) -> f32 {
    if max_chance.is_finite() && max_chance > 0.0 {
        max_chance
    } else {
        1.0
    }
}

#[must_use]
pub fn select_breakdown_part_with_trace<R: Rng + ?Sized>(
    rng: &mut R,
    weights: &PartWeights,
) -> (Part, Option<EventDecisionTrace>) {
    let choices = [
        (Part::Tire, weights.tire),
        (Part::Battery, weights.battery),
        (Part::Alternator, weights.alt),
        (Part::FuelPump, weights.pump),
    ];
    let mut total = 0_u32;
    for (_, weight) in &choices {
        total = total.saturating_add(*weight);
    }
    if total == 0 {
        return (Part::Tire, None);
    }

    let roll = rng.gen_range(0..total);
    let mut current = 0_u32;
    let mut selected = None;
    for (part, weight) in &choices {
        current = current.saturating_add(*weight);
        if selected.is_none() && roll < current {
            selected = Some(*part);
        }
    }

    let selected = selected.unwrap_or(Part::Tire);
    let candidates = choices
        .iter()
        .map(|(part, weight)| WeightedCandidate {
            id: part.key().to_string(),
            base_weight: f64::from(*weight),
            multipliers: Vec::new(),
            final_weight: f64::from(*weight),
        })
        .collect();
    let trace = EventDecisionTrace {
        pool_id: String::from("dystrail.breakdown_part"),
        roll: RollValue::U32(roll),
        candidates,
        chosen_id: selected.key().to_string(),
    };
    (selected, Some(trace))
}

const fn sanitize_multiplier(mult: f32) -> f32 {
    if mult.is_finite() { mult.max(0.0) } else { 1.0 }
}
