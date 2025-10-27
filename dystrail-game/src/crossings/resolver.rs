use rand::Rng;

use crate::state::{GameMode, PolicyKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossingResult {
    Pass,
    Detour(u8),
    TerminalFail,
}

#[derive(Debug, Clone, Copy)]
pub struct CrossingOutcome {
    pub result: CrossingResult,
    pub used_permit: bool,
    pub bribe_attempted: bool,
    pub bribe_succeeded: bool,
}

impl CrossingOutcome {
    #[must_use]
    const fn new(result: CrossingResult) -> Self {
        Self {
            result,
            used_permit: false,
            bribe_attempted: false,
            bribe_succeeded: false,
        }
    }
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn resolve_crossing<R: Rng + ?Sized>(
    policy: PolicyKind,
    mode: GameMode,
    has_permit: bool,
    bribe_intent: bool,
    _crossing_ix: u32,
    _day_ix: u32,
    rng: &mut R,
) -> CrossingOutcome {
    if has_permit {
        let mut outcome = CrossingOutcome::new(CrossingResult::Pass);
        outcome.used_permit = true;
        return outcome;
    }

    let mut outcome = CrossingOutcome::new(CrossingResult::TerminalFail);

    if !bribe_intent {
        outcome.result = resolve_detour_or_terminal(
            policy,
            mode,
            rng,
            baseline_detour_probability(policy, mode),
        );
        return outcome;
    }

    outcome.bribe_attempted = true;
    if rng.random::<f32>() < bribe_success_probability(policy, mode) {
        outcome.bribe_succeeded = true;
        outcome.result = CrossingResult::Pass;
        return outcome;
    }

    let detour_weight = detour_probability_after_bribe_failure(policy, mode);
    outcome.result = resolve_detour_or_terminal(policy, mode, rng, detour_weight);
    outcome
}

#[allow(clippy::missing_const_for_fn)]
fn baseline_detour_probability(policy: PolicyKind, mode: GameMode) -> f32 {
    if mode.is_deep() {
        match policy {
            PolicyKind::Aggressive => 0.80,
            PolicyKind::ResourceManager | PolicyKind::MonteCarlo => 0.86,
            PolicyKind::Conservative => 0.88,
            PolicyKind::Balanced => 0.84,
        }
    } else {
        match policy {
            PolicyKind::Conservative => 0.92,
            PolicyKind::Aggressive => 0.86,
            PolicyKind::ResourceManager | PolicyKind::MonteCarlo => 0.91,
            PolicyKind::Balanced => 0.90,
        }
    }
}

fn detour_probability_after_bribe_failure(policy: PolicyKind, mode: GameMode) -> f32 {
    let penalty = if mode.is_deep() { 0.03 } else { 0.02 };
    (baseline_detour_probability(policy, mode) - penalty).clamp(0.6, 0.98)
}

#[allow(clippy::missing_const_for_fn)]
fn bribe_success_probability(policy: PolicyKind, mode: GameMode) -> f32 {
    if mode.is_deep() {
        match policy {
            PolicyKind::Aggressive => 0.78,
            PolicyKind::ResourceManager | PolicyKind::MonteCarlo => 0.82,
            PolicyKind::Conservative | PolicyKind::Balanced => 0.80,
        }
    } else {
        match policy {
            PolicyKind::Conservative => 0.86,
            PolicyKind::Aggressive => 0.76,
            PolicyKind::ResourceManager | PolicyKind::MonteCarlo => 0.84,
            PolicyKind::Balanced => 0.82,
        }
    }
}

fn resolve_detour_or_terminal<R: Rng + ?Sized>(
    policy: PolicyKind,
    mode: GameMode,
    rng: &mut R,
    detour_weight: f32,
) -> CrossingResult {
    if rng.random::<f32>() < detour_weight {
        CrossingResult::Detour(sample_detour_days(policy, mode, rng))
    } else {
        CrossingResult::TerminalFail
    }
}

fn sample_detour_days<R: Rng + ?Sized>(policy: PolicyKind, mode: GameMode, rng: &mut R) -> u8 {
    let mut detour_roll: f32 = rng.random();
    if matches!(policy, PolicyKind::ResourceManager) && !mode.is_deep() {
        detour_roll *= 0.9;
    }
    match detour_roll {
        r if r < 0.35 => 2,
        r if r < 0.75 => 3,
        _ => {
            if mode.is_deep() {
                if matches!(policy, PolicyKind::Aggressive) || detour_roll > 0.9 {
                    4
                } else {
                    3
                }
            } else {
                3
            }
        }
    }
}
