use rand::Rng;
use rand::seq::SliceRandom;

use crate::journey::EventSeverity;
use crate::mechanics::otdeluxe90s::OtDeluxeDallesOutcomeWeights;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtDeluxeDallesOutcome {
    Safe,
    Loss,
    Drown,
}

#[must_use]
pub const fn otdeluxe_dalles_outcome_id(outcome: OtDeluxeDallesOutcome) -> &'static str {
    match outcome {
        OtDeluxeDallesOutcome::Safe => "safe",
        OtDeluxeDallesOutcome::Loss => "loss",
        OtDeluxeDallesOutcome::Drown => "drown",
    }
}

#[must_use]
pub const fn otdeluxe_dalles_severity(outcome: OtDeluxeDallesOutcome) -> EventSeverity {
    match outcome {
        OtDeluxeDallesOutcome::Safe => EventSeverity::Info,
        OtDeluxeDallesOutcome::Loss => EventSeverity::Warning,
        OtDeluxeDallesOutcome::Drown => EventSeverity::Critical,
    }
}

#[must_use]
pub fn roll_otdeluxe_dalles_outcome<R: Rng>(
    weights: &OtDeluxeDallesOutcomeWeights,
    rng: &mut R,
) -> OtDeluxeDallesOutcome {
    let safe = weights.safe.max(0.0);
    let loss = weights.loss.max(0.0);
    let drown = weights.drown.max(0.0);
    let total = safe + loss + drown;
    if total <= f32::EPSILON {
        return OtDeluxeDallesOutcome::Safe;
    }
    let roll = rng.r#gen::<f32>() * total;
    if roll < safe {
        OtDeluxeDallesOutcome::Safe
    } else if roll < safe + loss {
        OtDeluxeDallesOutcome::Loss
    } else {
        OtDeluxeDallesOutcome::Drown
    }
}

#[must_use]
pub fn sample_otdeluxe_dalles_outcome_with_rng<R: Rng>(
    weights: &OtDeluxeDallesOutcomeWeights,
    drownings_min: u8,
    drownings_max: u8,
    alive_indices: &[usize],
    rng: &mut R,
) -> (OtDeluxeDallesOutcome, Vec<usize>) {
    let outcome = roll_otdeluxe_dalles_outcome(weights, rng);
    let drowned_indices = if matches!(outcome, OtDeluxeDallesOutcome::Drown) {
        let min = drownings_min.min(drownings_max);
        let max = drownings_max.max(drownings_min);
        let drown_count = if max == 0 {
            0
        } else {
            rng.gen_range(min..=max)
        };
        select_drowning_indices(rng, alive_indices, drown_count)
    } else {
        Vec::new()
    };
    (outcome, drowned_indices)
}

fn select_drowning_indices<R: Rng>(
    rng: &mut R,
    alive_indices: &[usize],
    drownings: u8,
) -> Vec<usize> {
    let count = usize::from(drownings);
    if count == 0 || alive_indices.is_empty() {
        return Vec::new();
    }
    let mut indices = alive_indices.to_vec();
    indices.shuffle(rng);
    indices.truncate(count.min(indices.len()));
    indices
}
