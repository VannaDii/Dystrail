use rand::Rng;

use crate::journey::{EventDecisionTrace, RollValue, WeightedCandidate};
use crate::mechanics::otdeluxe90s::{OtDeluxeAfflictionPolicy, OtDeluxePolicyOverride};
use crate::otdeluxe_state::OtDeluxeAfflictionKind;

#[must_use]
pub fn otdeluxe_affliction_probability(
    health_general: u16,
    policy: &OtDeluxeAfflictionPolicy,
) -> f32 {
    let mut probability = policy.curve_pwl[0].probability;
    if health_general <= policy.curve_pwl[0].health {
        return probability.clamp(0.0, policy.probability_max);
    }
    for window in policy.curve_pwl.windows(2) {
        let start = window[0];
        let end = window[1];
        if health_general <= end.health {
            let span = f32::from(end.health.saturating_sub(start.health));
            if span > 0.0 {
                let offset = f32::from(health_general.saturating_sub(start.health));
                let t = (offset / span).clamp(0.0, 1.0);
                probability = start.probability.mul_add(1.0 - t, end.probability * t);
            }
            return probability.clamp(0.0, policy.probability_max);
        }
    }
    if let Some(last) = policy.curve_pwl.last() {
        probability = last.probability;
    }
    probability.clamp(0.0, policy.probability_max)
}

#[must_use]
pub const fn otdeluxe_affliction_duration(
    kind: OtDeluxeAfflictionKind,
    policy: &OtDeluxeAfflictionPolicy,
) -> u8 {
    let duration = match kind {
        OtDeluxeAfflictionKind::Illness => policy.illness_duration_days,
        OtDeluxeAfflictionKind::Injury => policy.injury_duration_days,
    };
    if duration == 0 { 1 } else { duration }
}

#[must_use]
pub fn roll_otdeluxe_affliction_kind_with_trace<R: Rng + ?Sized>(
    policy: &OtDeluxeAfflictionPolicy,
    overrides: &OtDeluxePolicyOverride,
    rng: &mut R,
) -> (OtDeluxeAfflictionKind, Option<EventDecisionTrace>) {
    let weights = &overrides.affliction_weights;
    let illness_weight = weights.illness.unwrap_or(policy.weight_illness);
    let injury_weight = weights.injury.unwrap_or(policy.weight_injury);
    let total = u32::from(illness_weight) + u32::from(injury_weight);
    if total == 0 {
        return (OtDeluxeAfflictionKind::Illness, None);
    }
    let roll = rng.gen_range(0..total);
    let kind = if roll < u32::from(illness_weight) {
        OtDeluxeAfflictionKind::Illness
    } else {
        OtDeluxeAfflictionKind::Injury
    };
    let candidates = vec![
        WeightedCandidate {
            id: String::from("illness"),
            base_weight: f64::from(illness_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(illness_weight),
        },
        WeightedCandidate {
            id: String::from("injury"),
            base_weight: f64::from(injury_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(injury_weight),
        },
    ];
    let chosen_id = match kind {
        OtDeluxeAfflictionKind::Illness => "illness",
        OtDeluxeAfflictionKind::Injury => "injury",
    };
    let trace = EventDecisionTrace {
        pool_id: String::from("otdeluxe.affliction_kind"),
        roll: RollValue::U32(roll),
        candidates,
        chosen_id: chosen_id.to_string(),
    };
    (kind, Some(trace))
}
