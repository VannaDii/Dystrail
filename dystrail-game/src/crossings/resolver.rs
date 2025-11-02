use rand::RngCore;

use super::CrossingKind;
use crate::journey::{BribePolicy, CrossingPolicy};

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
    let sample = rng.next_u32();
    let draw = ((sample as f64 + 0.5) / ((u32::MAX as f64) + 1.0)) as f32;

    let (pass_weight, detour_weight, _terminal_weight, permit_used) = effective_weights(&ctx);
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
    outcome
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
            pass = (pass + policy.bribe.pass_bonus * factor).max(0.0);
        }
        if policy.bribe.detour_bonus != 0.0 {
            detour = (detour + policy.bribe.detour_bonus * factor).max(0.0);
        }
        if policy.bribe.terminal_penalty != 0.0 {
            terminal = (terminal - policy.bribe.terminal_penalty * factor).max(0.0);
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
    min.saturating_add(offset as u8)
}

fn bribe_multiplier(policy: &BribePolicy, attempt_index: u32) -> f32 {
    if policy.diminishing_returns <= f32::EPSILON {
        return 1.0;
    }
    let denom = 1.0 + (attempt_index as f32) * policy.diminishing_returns.max(0.0);
    denom.recip()
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

#[cfg(test)]
mod tests {
    use super::*;
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
            self.next_u32() as u64
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let value = self.next_u32().to_le_bytes();
            for (idx, byte) in dest.iter_mut().enumerate() {
                *byte = value[idx % value.len()];
            }
        }
    }

    #[test]
    fn crossing_consumes_single_draw() {
        let policy = {
            let mut p = CrossingPolicy::default();
            p.pass = 0.6;
            p.detour = 0.25;
            p.terminal = 0.15;
            p
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
        let mut policy = CrossingPolicy::default();
        policy.pass = 0.6;
        policy.detour = 0.25;
        policy.terminal = 0.15;
        policy.permit.disable_terminal = true;
        policy.permit.eligible = vec!["checkpoint".to_string()];

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
        let mut policy = CrossingPolicy::default();
        policy.pass = 0.6;
        policy.detour = 0.25;
        policy.terminal = 0.15;
        policy.bribe.pass_bonus = 0.3;
        policy.bribe.terminal_penalty = 0.3;
        policy.bribe.diminishing_returns = 0.5;

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
        let mut policy = CrossingPolicy::default();
        policy.pass = 0.0;
        policy.detour = 1.0;
        policy.terminal = 0.0;
        policy.detour_days.min = 2;
        policy.detour_days.max = 5;

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
}
