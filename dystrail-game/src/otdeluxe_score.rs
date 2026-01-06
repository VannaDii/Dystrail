//! Scoring helpers for Oregon Trail Deluxe parity.

use std::sync::OnceLock;

use crate::mechanics::otdeluxe90s::{
    OtDeluxe90sPolicy, OtDeluxeHealthLabelRanges, OtDeluxeOccupation,
    OtDeluxeScorePointsPerPersonByHealth,
};
use crate::numbers::round_f64_to_i32;
use crate::otdeluxe_state::OtDeluxeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtDeluxeScoreHealthLabel {
    Good,
    Fair,
    Poor,
    VeryPoor,
}

fn default_policy() -> &'static OtDeluxe90sPolicy {
    static POLICY: OnceLock<OtDeluxe90sPolicy> = OnceLock::new();
    POLICY.get_or_init(OtDeluxe90sPolicy::default)
}

const fn score_health_label(
    health_general: u16,
    ranges: OtDeluxeHealthLabelRanges,
) -> OtDeluxeScoreHealthLabel {
    if health_general <= ranges.good_max {
        OtDeluxeScoreHealthLabel::Good
    } else if health_general <= ranges.fair_max {
        OtDeluxeScoreHealthLabel::Fair
    } else if health_general <= ranges.poor_max {
        OtDeluxeScoreHealthLabel::Poor
    } else {
        OtDeluxeScoreHealthLabel::VeryPoor
    }
}

const fn points_per_person(
    label: OtDeluxeScoreHealthLabel,
    points: &OtDeluxeScorePointsPerPersonByHealth,
) -> u32 {
    match label {
        OtDeluxeScoreHealthLabel::Good => points.good,
        OtDeluxeScoreHealthLabel::Fair => points.fair,
        OtDeluxeScoreHealthLabel::Poor => points.poor,
        OtDeluxeScoreHealthLabel::VeryPoor => points.very_poor,
    }
}

fn occupation_bonus_multiplier(
    occupation: Option<OtDeluxeOccupation>,
    policy: &OtDeluxe90sPolicy,
) -> f32 {
    let Some(occupation) = occupation else {
        return 1.0;
    };
    policy
        .occupations
        .iter()
        .find(|spec| spec.occupation == occupation)
        .map_or(1.0, |spec| spec.final_bonus_mult)
}

/// Compute the `OTDeluxe` parity score for a finished run.
#[must_use]
pub fn compute_score(state: &OtDeluxeState) -> i32 {
    compute_score_with_policy(state, default_policy())
}

/// Compute the `OTDeluxe` parity score using an explicit policy.
#[must_use]
pub fn compute_score_with_policy(state: &OtDeluxeState, policy: &OtDeluxe90sPolicy) -> i32 {
    let health_label = score_health_label(state.health_general, policy.health.label_ranges);
    let points_people = points_per_person(health_label, &policy.score.points_per_person_by_health);
    let alive = u32::from(state.party.alive_count());
    let mut total: u32 = 0;
    total = total.saturating_add(points_people.saturating_mul(alive));
    total = total.saturating_add(policy.score.points_wagon);
    total = total.saturating_add(
        policy
            .score
            .points_ox
            .saturating_mul(u32::from(state.oxen.total())),
    );

    let spares = u32::from(state.inventory.spares_wheels)
        + u32::from(state.inventory.spares_axles)
        + u32::from(state.inventory.spares_tongues);
    total = total
        .saturating_add(policy.score.points_spare_part.saturating_mul(spares))
        .saturating_add(
            policy
                .score
                .points_clothes
                .saturating_mul(u32::from(state.inventory.clothes_sets)),
        );

    let bullets_points = u32::from(state.inventory.bullets) / policy.score.divisor_bullets.max(1);
    let food_points = u32::from(state.inventory.food_lbs) / policy.score.divisor_food_lbs.max(1);
    let cash_points = state.inventory.cash_cents / policy.score.divisor_cash_cents.max(1);
    total = total
        .saturating_add(bullets_points)
        .saturating_add(food_points)
        .saturating_add(cash_points);

    let multiplier = occupation_bonus_multiplier(state.mods.occupation, policy).max(0.0);
    let scaled = f64::from(total) * f64::from(multiplier);
    round_f64_to_i32(scaled).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::otdeluxe90s::OtDeluxeOccupation;
    use crate::otdeluxe_state::{
        OtDeluxeInventory, OtDeluxePartyState, OtDeluxeState, OtDeluxeTerrain, OtDeluxeWeatherState,
    };
    use crate::state::Season;

    #[test]
    fn score_matches_policy_formula() {
        let policy = OtDeluxe90sPolicy::default();
        let mut state = OtDeluxeState {
            party: OtDeluxePartyState::from_names(["A", "B", "C", "D", "E"]),
            health_general: 10,
            oxen: crate::otdeluxe_state::OtDeluxeOxenState {
                healthy: 6,
                sick: 0,
            },
            inventory: OtDeluxeInventory {
                food_lbs: 500,
                bullets: 120,
                clothes_sets: 10,
                cash_cents: 1000,
                spares_wheels: 1,
                spares_axles: 1,
                spares_tongues: 1,
            },
            terrain: OtDeluxeTerrain::Plains,
            season: Season::Spring,
            weather: OtDeluxeWeatherState::default(),
            ..OtDeluxeState::default()
        };
        state.mods.occupation = Some(OtDeluxeOccupation::Merchant);

        let score = compute_score_with_policy(&state, &policy);
        assert_eq!(score, 3936);
    }

    #[test]
    fn score_respects_health_label_points() {
        let policy = OtDeluxe90sPolicy::default();
        let mut state = OtDeluxeState {
            party: OtDeluxePartyState::from_names(["A", "B"]),
            health_general: 80,
            ..OtDeluxeState::default()
        };
        state.mods.occupation = Some(OtDeluxeOccupation::Banker);

        let score = compute_score_with_policy(&state, &policy);
        assert_eq!(score, 50);
    }
}
