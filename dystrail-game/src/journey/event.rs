//! Structured simulation events emitted by the journey kernel.
//!
//! The engine is migrating from "log key as truth" to event-sourced outcomes.
//! During the transition, events can carry legacy i18n log keys as presentation
//! hints, but the `kind` remains a mechanical descriptor.

use crate::journey::DayTagSet;
use serde::{Deserialize, Serialize};

/// Stable, deterministic identifier for a single event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId {
    /// One-based day counter when the event occurred.
    pub day: u32,
    /// Per-day sequence number (0-based) within the emitted event stream.
    pub seq: u16,
}

impl EventId {
    #[must_use]
    pub const fn new(day: u32, seq: u16) -> Self {
        Self { day, seq }
    }
}

/// Mechanical event kind emitted by the simulation kernel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Transitional event mapping an existing i18n log key into the event stream.
    LegacyLogKey,
}

/// Severity tier for a simulation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

/// Hint for how the UI should surface an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSurfaceHint {
    Log,
    Toast,
    Modal,
}

/// Structured event emitted by the simulation kernel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    /// One-based day counter when the event occurred.
    pub day: u32,
    pub kind: EventKind,
    pub severity: EventSeverity,
    /// Stable tags describing the event (e.g., `repair`, `detour`, `crossing`).
    #[serde(default)]
    pub tags: DayTagSet,
    /// Optional UI guidance for surfacing the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_surface_hint: Option<UiSurfaceHint>,
    /// Optional i18n key for presentation-layer rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_key: Option<String>,
    /// Optional structured payload for debugging and downstream rendering.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub payload: serde_json::Value,
}

impl Event {
    #[must_use]
    pub fn legacy_log_key(id: EventId, day: u32, ui_key: impl Into<String>) -> Self {
        Self {
            id,
            day,
            kind: EventKind::LegacyLogKey,
            severity: EventSeverity::Info,
            tags: DayTagSet::new(),
            ui_surface_hint: Some(UiSurfaceHint::Log),
            ui_key: Some(ui_key.into()),
            payload: serde_json::Value::Null,
        }
    }
}

/// Explainability telemetry for random event selection.
///
/// This is populated by phases that select from weighted pools (events/encounters).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDecisionTrace {
    /// Identifier for the selection pool (e.g., `otdeluxe.random_events`).
    pub pool_id: String,
    /// Random draw used to select from the weighted pool.
    pub roll: RollValue,
    /// Candidate weights considered during selection.
    pub candidates: Vec<WeightedCandidate>,
    /// Identifier of the selected candidate.
    pub chosen_id: String,
}

/// Candidate weight telemetry captured during event selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightedCandidate {
    pub id: String,
    pub base_weight: f64,
    /// Multipliers applied in order.
    pub multipliers: Vec<WeightFactor>,
    pub final_weight: f64,
}

/// Random roll value used by weighted selection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RollValue {
    U32(u32),
    F32(f32),
}

/// Single multiplicative weight factor used in an event selection trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightFactor {
    pub label: String,
    pub value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_log_event_roundtrips_and_has_stable_id() {
        let id = EventId::new(7, 3);
        let event = Event::legacy_log_key(id, 7, "log.traveled");

        assert_eq!(event.id, id);
        assert_eq!(event.day, 7);
        assert_eq!(event.kind, EventKind::LegacyLogKey);
        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.ui_surface_hint, Some(UiSurfaceHint::Log));
        assert_eq!(event.ui_key.as_deref(), Some("log.traveled"));
        assert!(event.payload.is_null());

        let json = serde_json::to_string(&event).expect("serialize");
        let restored: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, event);
    }

    #[test]
    fn decision_trace_roundtrips() {
        let trace = EventDecisionTrace {
            pool_id: String::from("example.pool"),
            roll: RollValue::U32(7),
            candidates: vec![WeightedCandidate {
                id: String::from("candidate-a"),
                base_weight: 1.0,
                multipliers: vec![
                    WeightFactor {
                        label: String::from("region"),
                        value: 1.25,
                    },
                    WeightFactor {
                        label: String::from("weather"),
                        value: 0.8,
                    },
                ],
                final_weight: 1.0,
            }],
            chosen_id: String::from("candidate-a"),
        };

        let json = serde_json::to_string(&trace).expect("serialize");
        let restored: EventDecisionTrace = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, trace);
    }
}
