use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

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
pub fn resolve_crossing(
    policy: PolicyKind,
    mode: GameMode,
    has_permit: bool,
    bribe_intent: bool,
    crossing_ix: u32,
    day_ix: u32,
    seed: u64,
) -> CrossingOutcome {
    let mut rng = seeded_rng(seed, crossing_ix, day_ix);

    if has_permit {
        let mut outcome = CrossingOutcome::new(CrossingResult::Pass);
        outcome.used_permit = true;
        return outcome;
    }

    let mut outcome = CrossingOutcome::new(CrossingResult::TerminalFail);

    if !bribe_intent {
        outcome.result = resolve_detour_or_terminal(&mut rng, 0.88);
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

    outcome.result = resolve_detour_or_terminal(&mut rng, 0.85);
    outcome
}

fn seeded_rng(seed: u64, crossing_ix: u32, day_ix: u32) -> ChaCha20Rng {
    let mix = seed
        ^ (u64::from(crossing_ix)).wrapping_mul(1_146_707)
        ^ (u64::from(day_ix)).wrapping_mul(97);
    let hashed = fnv64(mix);
    ChaCha20Rng::seed_from_u64(hashed)
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

fn fnv64(value: u64) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}
