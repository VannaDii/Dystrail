//! Versioned hearing record. Randomness is committed once; presentation only reads it.
use crate::state::Stats;
use serde::{Deserialize, Serialize};

pub const HEARING_RULES_VERSION: u8 = 1;
pub const CONTINUATION_PERCENT: u8 = 50;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Draw {
    Influence,
    Continuation,
    Vote,
}
impl Draw {
    pub(super) const fn bound(self) -> u32 {
        match self {
            Self::Influence => 101,
            _ => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HearingPhase {
    #[default]
    Arrival,
    Preparation,
    RoundRolling(u8),
    RoundResult(u8),
    Committee(u8),
    CommitteeResult(u8),
    Closed,
    VoteRolling,
    Verdict,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HearingOutcome {
    Passed,
    Failed,
    Secured,
    Exhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HearingRound {
    /// A multiplier contribution, not a probability of winning.
    pub influence: u16,
    pub sanity_before: i32,
    pub sanity_after: i32,
    /// Only rolled after a survived round with another round available.
    pub continuation_roll: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HearingReport {
    pub rules_version: u8,
    pub starting_stats: Stats,
    pub day: u32,
    pub minute: u16,
    pub base_chance: f64,
    pub policy_guarantee: bool,
    pub rounds: Vec<HearingRound>,
    /// Uncapped adjusted chance. None when exhaustion prevented a vote.
    pub adjusted_chance: Option<f64>,
    pub vote_roll: Option<u8>,
    pub outcome: HearingOutcome,
}

impl HearingReport {
    #[must_use]
    pub fn average_through(&self, count: usize) -> f64 {
        let count = count.min(self.rounds.len());
        if count == 0 {
            return 100.0;
        }
        self.rounds
            .iter()
            .take(count)
            .map(|r| f64::from(r.influence))
            .sum::<f64>()
            / f64::from(u32::try_from(count).unwrap_or(3))
    }

    #[must_use]
    pub fn adjusted_through(&self, count: usize) -> f64 {
        if self.policy_guarantee {
            1.0
        } else {
            self.base_chance * self.average_through(count) / 100.0
        }
    }

    #[must_use]
    pub fn revealed_rounds(&self, phase: HearingPhase) -> usize {
        let count = match phase {
            HearingPhase::Arrival | HearingPhase::Preparation => 0,
            HearingPhase::RoundRolling(i) => usize::from(i),
            HearingPhase::RoundResult(i)
            | HearingPhase::Committee(i)
            | HearingPhase::CommitteeResult(i) => usize::from(i) + 1,
            _ => self.rounds.len(),
        };
        count.min(self.rounds.len())
    }

    /// A transition has no game-state cost and consumes no random draws.
    #[must_use]
    pub fn next_phase(&self, phase: HearingPhase) -> HearingPhase {
        match phase {
            HearingPhase::RoundRolling(i) => {
                if self
                    .rounds
                    .get(usize::from(i))
                    .is_some_and(|r| r.sanity_after <= 0)
                {
                    HearingPhase::Verdict
                } else {
                    HearingPhase::RoundResult(i)
                }
            }
            HearingPhase::RoundResult(i) => {
                if self
                    .rounds
                    .get(usize::from(i))
                    .is_some_and(|r| r.sanity_after <= 0)
                {
                    HearingPhase::Verdict
                } else if self
                    .rounds
                    .get(usize::from(i))
                    .is_some_and(|r| r.continuation_roll.is_some())
                {
                    HearingPhase::Committee(i)
                } else {
                    self.closed_phase()
                }
            }
            HearingPhase::Committee(i) => HearingPhase::CommitteeResult(i),
            HearingPhase::CommitteeResult(i) => {
                if usize::from(i) + 1 < self.rounds.len() {
                    HearingPhase::RoundRolling(i + 1)
                } else {
                    self.closed_phase()
                }
            }
            HearingPhase::Closed => HearingPhase::VoteRolling,
            HearingPhase::VoteRolling => HearingPhase::Verdict,
            HearingPhase::Verdict => HearingPhase::Complete,
            other => other,
        }
    }

    const fn closed_phase(&self) -> HearingPhase {
        if matches!(
            self.outcome,
            HearingOutcome::Secured | HearingOutcome::Exhausted
        ) {
            HearingPhase::Verdict
        } else {
            HearingPhase::Closed
        }
    }
}

/// Uses a supplied bounded draw source so exact roll boundaries and consumption are testable.
pub(super) fn resolve(
    mut report: HearingReport,
    max_rounds: u32,
    sanity_cost: i32,
    mut draw: impl FnMut(Draw) -> u32,
) -> HearingReport {
    let mut sanity = report.starting_stats.sanity;
    if sanity <= 0 {
        return report;
    }
    for index in 0..max_rounds.clamp(1, 3) {
        let influence = u16::try_from(50 + draw(Draw::Influence)).unwrap_or(100);
        let before = sanity;
        sanity = sanity.saturating_sub(sanity_cost.max(0)).max(0);
        let continuation_roll = if sanity > 0 && index + 1 < max_rounds.clamp(1, 3) {
            Some(u8::try_from(draw(Draw::Continuation)).unwrap_or(99))
        } else {
            None
        };
        report.rounds.push(HearingRound {
            influence,
            sanity_before: before,
            sanity_after: sanity,
            continuation_roll,
        });
        if sanity <= 0 {
            return report;
        }
        if continuation_roll.is_none_or(|roll| roll >= CONTINUATION_PERCENT) {
            break;
        }
    }
    let adjusted = report.adjusted_through(report.rounds.len());
    report.adjusted_chance = Some(adjusted);
    if adjusted >= 1.0 {
        report.outcome = HearingOutcome::Secured;
    } else {
        let roll = u8::try_from(draw(Draw::Vote)).unwrap_or(99);
        report.vote_roll = Some(roll);
        report.outcome = if f64::from(roll) < adjusted * 100.0 {
            HearingOutcome::Passed
        } else {
            HearingOutcome::Failed
        };
    }
    report
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HearingForecast {
    pub base_chance: f64,
    pub survival_chance: f64,
    pub policy_guarantee: bool,
    /// Existing policy preparation is previewed without spending these resources.
    pub entry_sanity: i32,
    pub preparation_supplies: i32,
    pub preparation_cents: i64,
}
