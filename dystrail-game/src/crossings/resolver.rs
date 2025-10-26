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
        outcome.result = resolve_detour_or_terminal(rng, 0.88);
        return outcome;
    }

    outcome.bribe_attempted = true;

    let mut p_bribe = 0.74_f32;
    if mode.is_deep() {
        p_bribe -= 0.02;
    }
    let _ = policy;

    let bribe_success_chance = p_bribe.clamp(0.70, 0.85);
    let roll: f32 = rng.random();
    if roll < bribe_success_chance {
        outcome.bribe_succeeded = true;
        outcome.result = CrossingResult::Pass;
        return outcome;
    }

    outcome.result = resolve_detour_or_terminal(rng, 0.85);
    outcome
}

fn resolve_detour_or_terminal<R: Rng + ?Sized>(rng: &mut R, detour_weight: f32) -> CrossingResult {
    let roll: f32 = rng.random();
    if roll < detour_weight {
        let detour_roll: f32 = rng.random();
        let days = match detour_roll {
            r if r < 0.40 => 2,
            r if r < 0.80 => 3,
            _ => 4,
        };
        CrossingResult::Detour(days)
    } else {
        CrossingResult::TerminalFail
    }
}
