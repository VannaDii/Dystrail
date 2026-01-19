use rand::RngCore;
use std::convert::TryFrom;

use super::CrossingKind;
use crate::journey::{
    BribePolicy, CrossingPolicy, EventDecisionTrace, RollValue, WeightedCandidate,
};
use crate::numbers::clamp_f64_to_f32;

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

/// Context passed to the deterministic crossing resolver.
#[derive(Debug, Clone, Copy)]
pub struct CrossingContext<'a> {
    pub policy: &'a CrossingPolicy,
    pub kind: CrossingKind,
    pub has_permit: bool,
    pub bribe_intent: bool,
    /// Number of bribes attempted prior to this crossing (0-based).
    pub prior_bribe_attempts: u32,
}

#[must_use]
pub fn resolve_crossing<R: RngCore>(ctx: CrossingContext<'_>, rng: &mut R) -> CrossingOutcome {
    let (outcome, _) = resolve_crossing_with_trace(ctx, rng);
    outcome
}

#[must_use]
pub fn resolve_crossing_with_trace<R: RngCore>(
    ctx: CrossingContext<'_>,
    rng: &mut R,
) -> (CrossingOutcome, Option<EventDecisionTrace>) {
    let sample = rng.next_u32();
    let draw = safe_sample_ratio(sample);

    let (pass_weight, detour_weight, terminal_weight, permit_used) = effective_weights(&ctx);
    let detour_threshold = pass_weight + detour_weight;

    let result = if draw < pass_weight {
        CrossingResult::Pass
    } else if draw < detour_threshold {
        let detour_days = detour_days_for_sample(ctx.policy, sample);
        CrossingResult::Detour(detour_days)
    } else {
        CrossingResult::TerminalFail
    };

    let mut outcome = CrossingOutcome::new(result);
    outcome.used_permit = permit_used;
    outcome.bribe_attempted = ctx.bribe_intent;
    outcome.bribe_succeeded = ctx.bribe_intent && matches!(result, CrossingResult::Pass);

    let pool_id = match ctx.kind {
        CrossingKind::Checkpoint => "crossing.checkpoint",
        CrossingKind::BridgeOut => "crossing.bridge_out",
    };
    let candidates = vec![
        WeightedCandidate {
            id: String::from("pass"),
            base_weight: f64::from(pass_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(pass_weight),
        },
        WeightedCandidate {
            id: String::from("detour"),
            base_weight: f64::from(detour_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(detour_weight),
        },
        WeightedCandidate {
            id: String::from("terminal"),
            base_weight: f64::from(terminal_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(terminal_weight),
        },
    ];
    let chosen_id = match outcome.result {
        CrossingResult::Pass => "pass",
        CrossingResult::Detour(_) => "detour",
        CrossingResult::TerminalFail => "terminal",
    };
    let trace = EventDecisionTrace {
        pool_id: pool_id.to_string(),
        roll: RollValue::F32(draw),
        candidates,
        chosen_id: chosen_id.to_string(),
    };
    (outcome, Some(trace))
}

fn effective_weights(ctx: &CrossingContext<'_>) -> (f32, f32, f32, bool) {
    let policy = ctx.policy;
    let mut pass = policy.pass.max(0.0);
    let mut detour = policy.detour.max(0.0);
    let mut terminal = policy.terminal.max(0.0);

    let permit_applicable = ctx.has_permit && permit_allows(policy, ctx.kind);
    if permit_applicable && policy.permit.disable_terminal {
        terminal = 0.0;
    }

    if ctx.bribe_intent {
        let factor = bribe_multiplier(&policy.bribe, ctx.prior_bribe_attempts);
        if policy.bribe.pass_bonus != 0.0 {
            pass = policy.bribe.pass_bonus.mul_add(factor, pass).max(0.0);
        }
        if policy.bribe.detour_bonus != 0.0 {
            detour = policy.bribe.detour_bonus.mul_add(factor, detour).max(0.0);
        }
        if policy.bribe.terminal_penalty != 0.0 {
            terminal = (-policy.bribe.terminal_penalty)
                .mul_add(factor, terminal)
                .max(0.0);
        }
    }

    let total = pass + detour + terminal;
    if total <= f32::EPSILON {
        return (1.0, 0.0, 0.0, permit_applicable);
    }

    let norm = 1.0 / total;
    (
        pass * norm,
        detour * norm,
        terminal * norm,
        permit_applicable,
    )
}

fn detour_days_for_sample(policy: &CrossingPolicy, sample: u32) -> u8 {
    let min = policy.detour_days.min;
    let max = policy.detour_days.max;
    if min >= max {
        return min;
    }
    let span = u32::from(max.saturating_sub(min)) + 1;
    let offset = sample % span;
    let offset_u8 = u8::try_from(offset).unwrap_or(u8::MAX);
    min.saturating_add(offset_u8)
}

fn bribe_multiplier(policy: &BribePolicy, attempt_index: u32) -> f32 {
    if policy.diminishing_returns <= f32::EPSILON {
        return 1.0;
    }
    let returns = f64::from(policy.diminishing_returns.max(0.0));
    let attempts = f64::from(attempt_index);
    let denom = returns.mul_add(attempts, 1.0);
    clamp_f64_to_f32(1.0 / denom)
}

fn permit_allows(policy: &CrossingPolicy, kind: CrossingKind) -> bool {
    if policy.permit.eligible.is_empty() {
        return policy.permit.disable_terminal;
    }
    let kind_token = match kind {
        CrossingKind::Checkpoint => "checkpoint",
        CrossingKind::BridgeOut => "bridge_out",
    };
    policy
        .permit
        .eligible
        .iter()
        .any(|entry| entry == kind_token)
}

fn safe_sample_ratio(sample: u32) -> f32 {
    let denom = f64::from(u32::MAX) + 1.0;
    let ratio = (f64::from(sample) + 0.5) / denom;
    clamp_f64_to_f32(ratio.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DetourPolicy, PermitPolicy};
    use rand::RngCore;

    struct StubRng {
        value: u32,
        calls: u32,
    }

    impl StubRng {
        fn new(value: u32) -> Self {
            Self { value, calls: 0 }
        }
    }

    impl RngCore for StubRng {
        fn next_u32(&mut self) -> u32 {
            self.calls = self.calls.saturating_add(1);
            self.value
        }

        fn next_u64(&mut self) -> u64 {
            u64::from(self.next_u32())
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let value = self.next_u32().to_le_bytes();
            for (idx, byte) in dest.iter_mut().enumerate() {
                *byte = value[idx % value.len()];
            }
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    #[test]
    fn crossing_consumes_single_draw() {
        let policy = CrossingPolicy {
            pass: 0.6,
            detour: 0.25,
            terminal: 0.15,
            ..CrossingPolicy::default()
        };
        let mut rng = StubRng::new(0);
        let ctx = CrossingContext {
            policy: &policy,
            kind: CrossingKind::Checkpoint,
            has_permit: false,
            bribe_intent: false,
            prior_bribe_attempts: 0,
        };
        let _ = resolve_crossing(ctx, &mut rng);
        assert_eq!(rng.calls, 1, "resolver must draw exactly once");
    }

    #[test]
    fn permit_disables_terminal() {
        let policy = CrossingPolicy {
            pass: 0.6,
            detour: 0.25,
            terminal: 0.15,
            permit: PermitPolicy {
                disable_terminal: true,
                eligible: vec!["checkpoint".to_string()],
            },
            ..CrossingPolicy::default()
        };

        let ctx = CrossingContext {
            policy: &policy,
            kind: CrossingKind::Checkpoint,
            has_permit: true,
            bribe_intent: false,
            prior_bribe_attempts: 0,
        };

        let mut rng = StubRng::new(0);
        let outcome = resolve_crossing(ctx, &mut rng);
        assert!(
            matches!(
                outcome.result,
                CrossingResult::Detour(_) | CrossingResult::Pass
            ),
            "permit should prevent terminal failures"
        );
        assert!(outcome.used_permit);
    }

    #[test]
    fn diminishing_returns_apply_to_bribes() {
        let policy = CrossingPolicy {
            pass: 0.6,
            detour: 0.25,
            terminal: 0.15,
            bribe: BribePolicy {
                pass_bonus: 0.3,
                detour_bonus: 0.0,
                terminal_penalty: 0.3,
                diminishing_returns: 0.5,
            },
            ..CrossingPolicy::default()
        };

        let ctx_first = CrossingContext {
            policy: &policy,
            kind: CrossingKind::Checkpoint,
            has_permit: false,
            bribe_intent: true,
            prior_bribe_attempts: 0,
        };
        let ctx_second = CrossingContext {
            prior_bribe_attempts: 3,
            ..ctx_first
        };

        let mut rng = StubRng::new(u32::MAX / 2);
        let first = resolve_crossing(ctx_first, &mut rng);

        let mut rng_second = StubRng::new(u32::MAX / 2);
        let second = resolve_crossing(ctx_second, &mut rng_second);

        let pass_first = matches!(first.result, CrossingResult::Pass);
        let pass_second = matches!(second.result, CrossingResult::Pass);

        assert!(
            pass_first || !pass_second,
            "later bribe attempt should not be more favorable than the first"
        );
    }

    #[test]
    fn detour_days_cover_span_using_single_sample() {
        let policy = CrossingPolicy {
            pass: 0.0,
            detour: 1.0,
            terminal: 0.0,
            detour_days: DetourPolicy { min: 2, max: 5 },
            ..CrossingPolicy::default()
        };

        let ctx = CrossingContext {
            policy: &policy,
            kind: CrossingKind::BridgeOut,
            has_permit: false,
            bribe_intent: false,
            prior_bribe_attempts: 0,
        };

        let mut rng_min = StubRng::new(0);
        let outcome_min = resolve_crossing(ctx, &mut rng_min);
        assert!(matches!(outcome_min.result, CrossingResult::Detour(2)));

        let mut rng_max = StubRng::new(3);
        let outcome_max = resolve_crossing(ctx, &mut rng_max);
        assert!(matches!(outcome_max.result, CrossingResult::Detour(5)));
    }

    #[test]
    fn resolve_crossing_defaults_when_weights_zero() {
        let policy = CrossingPolicy {
            pass: 0.0,
            detour: 0.0,
            terminal: 0.0,
            ..CrossingPolicy::default()
        };
        let ctx = CrossingContext {
            policy: &policy,
            kind: CrossingKind::Checkpoint,
            has_permit: false,
            bribe_intent: false,
            prior_bribe_attempts: 0,
        };
        let mut rng = StubRng::new(0);
        let outcome = resolve_crossing(ctx, &mut rng);
        assert!(matches!(outcome.result, CrossingResult::Pass));
    }

    #[test]
    fn detour_days_return_min_when_range_collapses() {
        let policy = CrossingPolicy {
            detour_days: DetourPolicy { min: 4, max: 2 },
            ..CrossingPolicy::default()
        };
        let days = detour_days_for_sample(&policy, 99);
        assert_eq!(days, 4);
    }

    #[test]
    fn permit_allows_bridge_out_when_eligible() {
        let policy = CrossingPolicy {
            permit: PermitPolicy {
                disable_terminal: false,
                eligible: vec![String::from("bridge_out")],
            },
            ..CrossingPolicy::default()
        };
        assert!(permit_allows(&policy, CrossingKind::BridgeOut));
    }
}
