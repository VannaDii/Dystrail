use crate::journey::{
    DayOutcome, DayTagSet, Event, EventId, EventSeverity, JourneyCfg, TravelDayKind,
};
use crate::state::DayIntent;

use super::events::{KernelDecisionTrace, KernelEventCode, KernelEventPayload};

/// Kernel-facing simulation configuration.
pub type KernelConfig = JourneyCfg;

/// Kernel-facing simulation state snapshot.
pub type KernelState = crate::GameState;

/// Input provided to the kernel for a single daily tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelTickInput {
    /// Player's day intent for this tick.
    pub intent: DayIntent,
}

/// A kernel event with stable code plus structured payload.
#[derive(Debug, Clone)]
pub struct KernelEvent {
    pub id: EventId,
    pub day: u32,
    pub code: KernelEventCode,
    pub severity: EventSeverity,
    pub tags: DayTagSet,
    pub ui_key: Option<String>,
    pub payload: KernelEventPayload,
}

impl KernelEvent {
    fn from_journey_event(event: Event) -> Self {
        Self {
            id: event.id,
            day: event.day,
            code: KernelEventCode::from(&event.kind),
            severity: event.severity,
            tags: event.tags,
            ui_key: event.ui_key,
            payload: event.payload.into(),
        }
    }
}

/// Result of a kernel daily tick.
#[derive(Debug, Clone)]
pub struct KernelTickOutput {
    pub ended: bool,
    pub day_consumed: bool,
    pub day_kind: Option<TravelDayKind>,
    pub log_key: String,
    pub events: Vec<KernelEvent>,
    pub decision_traces: Vec<KernelDecisionTrace>,
}

impl From<DayOutcome> for KernelTickOutput {
    fn from(outcome: DayOutcome) -> Self {
        let day_kind = outcome.record.as_ref().map(|record| record.kind);
        let events = outcome
            .events
            .into_iter()
            .map(KernelEvent::from_journey_event)
            .collect();
        Self {
            ended: outcome.ended,
            day_consumed: outcome.day_consumed,
            day_kind,
            log_key: outcome.log_key,
            events,
            decision_traces: outcome.decision_traces,
        }
    }
}
