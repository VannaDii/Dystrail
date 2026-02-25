//! OTDeluxe-first kernel facade.
//!
//! This module provides a stable, neutral simulation API that callers can use
//! while `state.rs`/`journey` internals continue to be decomposed into phase and
//! system modules.

use crate::endgame::EndgameTravelCfg;
use crate::journey::{
    DayOutcome, DayTagSet, Event, EventDecisionTrace, EventId, EventKind, EventSeverity,
    JourneySession, MechanicalPolicyId, PolicyId, StrategyId, TravelDayKind,
};
use crate::mechanics::OtDeluxeOccupation;
use crate::state::{DayIntent, GameMode};
use thiserror::Error;

/// Input provided to the kernel for a single daily tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelTickInput {
    /// Player's day intent for this tick.
    pub intent: DayIntent,
}

/// Deterministic event code for i18n and UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelEventCode {
    LegacyLog,
    WeatherResolved,
    DailyConsumptionApplied,
    HealthTickApplied,
    GeneralStrainComputed,
    ExecOrderStarted,
    ExecOrderEnded,
    BreakdownStarted,
    BreakdownResolved,
    EncounterTriggered,
    RandomEventResolved,
    TradeResolved,
    HuntResolved,
    AfflictionTriggered,
    NavigationEvent,
    CrossingResolved,
    TravelBlocked,
}

impl KernelEventCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LegacyLog => "event.legacy.log",
            Self::WeatherResolved => "event.weather.resolved",
            Self::DailyConsumptionApplied => "event.supplies.daily_consumption_applied",
            Self::HealthTickApplied => "event.health.tick_applied",
            Self::GeneralStrainComputed => "event.health.general_strain_computed",
            Self::ExecOrderStarted => "event.exec_order.started",
            Self::ExecOrderEnded => "event.exec_order.ended",
            Self::BreakdownStarted => "event.vehicle.breakdown_started",
            Self::BreakdownResolved => "event.vehicle.breakdown_resolved",
            Self::EncounterTriggered => "event.encounter.triggered",
            Self::RandomEventResolved => "event.random.resolved",
            Self::TradeResolved => "event.trade.resolved",
            Self::HuntResolved => "event.hunt.resolved",
            Self::AfflictionTriggered => "event.affliction.triggered",
            Self::NavigationEvent => "event.navigation.resolved",
            Self::CrossingResolved => "event.crossing.resolved",
            Self::TravelBlocked => "event.travel.blocked",
        }
    }
}

impl From<&EventKind> for KernelEventCode {
    fn from(value: &EventKind) -> Self {
        match value {
            EventKind::LegacyLogKey => Self::LegacyLog,
            EventKind::WeatherResolved => Self::WeatherResolved,
            EventKind::DailyConsumptionApplied => Self::DailyConsumptionApplied,
            EventKind::HealthTickApplied => Self::HealthTickApplied,
            EventKind::GeneralStrainComputed => Self::GeneralStrainComputed,
            EventKind::ExecOrderStarted => Self::ExecOrderStarted,
            EventKind::ExecOrderEnded => Self::ExecOrderEnded,
            EventKind::BreakdownStarted => Self::BreakdownStarted,
            EventKind::BreakdownResolved => Self::BreakdownResolved,
            EventKind::EncounterTriggered => Self::EncounterTriggered,
            EventKind::RandomEventResolved => Self::RandomEventResolved,
            EventKind::TradeResolved => Self::TradeResolved,
            EventKind::HuntResolved => Self::HuntResolved,
            EventKind::AfflictionTriggered => Self::AfflictionTriggered,
            EventKind::NavigationEvent => Self::NavigationEvent,
            EventKind::CrossingResolved => Self::CrossingResolved,
            EventKind::TravelBlocked => Self::TravelBlocked,
        }
    }
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
    pub payload: serde_json::Value,
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
            payload: event.payload,
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
    pub decision_traces: Vec<EventDecisionTrace>,
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

/// Errors constructing or running the `OTDeluxe` kernel facade.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum KernelSessionError {
    #[error("OTDeluxe kernel session requires OTDeluxe mechanical policy")]
    NonOtDeluxePolicy,
}

/// `OTDeluxe` parity kernel session facade.
#[derive(Debug, Clone)]
pub struct KernelSession {
    inner: JourneySession,
}

impl KernelSession {
    /// Creates a new `OTDeluxe` kernel session.
    #[must_use]
    pub fn new(
        mode: GameMode,
        strategy: StrategyId,
        seed: u64,
        data: crate::EncounterData,
        endgame_cfg: &EndgameTravelCfg,
        occupation: Option<OtDeluxeOccupation>,
    ) -> Self {
        let inner = JourneySession::new_with_mechanics(
            MechanicalPolicyId::OtDeluxe90s,
            mode,
            strategy,
            seed,
            data,
            endgame_cfg,
            occupation,
        );
        Self { inner }
    }

    /// Creates a kernel session from an existing state.
    ///
    /// # Errors
    ///
    /// Returns an error when the provided state is not using `OTDeluxe90s` mechanics.
    pub fn from_state(
        state: crate::GameState,
        strategy: StrategyId,
        endgame_cfg: &EndgameTravelCfg,
    ) -> Result<Self, KernelSessionError> {
        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return Err(KernelSessionError::NonOtDeluxePolicy);
        }
        Ok(Self {
            inner: JourneySession::from_state(state, strategy, endgame_cfg),
        })
    }

    /// Advances the simulation one day under the provided intent.
    pub fn tick_day(&mut self, input: KernelTickInput) -> KernelTickOutput {
        self.inner.state_mut().intent.pending = input.intent;
        self.inner.tick_day().into()
    }

    /// Returns the immutable game state.
    #[must_use]
    pub const fn state(&self) -> &crate::GameState {
        self.inner.state()
    }

    /// Consumes the kernel session and returns the inner game state.
    #[must_use]
    pub fn into_state(self) -> crate::GameState {
        self.inner.into_state()
    }

    /// Mechanical policy used by this session.
    #[must_use]
    pub const fn mechanics(&self) -> MechanicalPolicyId {
        self.inner.state().mechanical_policy
    }

    /// Policy family currently configured by the underlying controller.
    #[must_use]
    pub const fn policy(&self) -> PolicyId {
        self.inner.policy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journey::{DayEffects, DayInputs, DayRecord, DayTagSet, Event, EventId, StatsDelta};
    use crate::state::{DietId, GameState, PaceId, Region, Season};
    use crate::weather::Weather;

    fn sample_outcome(kind: EventKind, with_record: bool) -> DayOutcome {
        let record = with_record.then(|| DayRecord::new(1, TravelDayKind::Travel, 12.0));
        DayOutcome {
            ended: false,
            log_key: String::from("log.sample"),
            breakdown_started: false,
            day_consumed: true,
            inputs: DayInputs {
                day: 2,
                intent: DayIntent::Continue,
                pace: PaceId::Steady,
                diet: DietId::Mixed,
                region: Region::Heartland,
                season: Season::Spring,
                mode: GameMode::Classic,
                mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
                weather: Weather::Clear,
            },
            effects: DayEffects {
                stats: StatsDelta::default(),
                budget_delta: 0,
                budget_cents_delta: 0,
                miles_traveled_delta: 1.0,
                miles_traveled_actual_delta: 1.0,
            },
            record,
            events: vec![Event {
                id: EventId::new(2, 0),
                day: 2,
                kind,
                severity: EventSeverity::Info,
                tags: DayTagSet::new(),
                ui_surface_hint: None,
                ui_key: Some(String::from("ui.sample")),
                payload: serde_json::json!({"key":"value"}),
            }],
            decision_traces: Vec::new(),
        }
    }

    #[test]
    fn kernel_event_code_mapping_is_stable() {
        assert_eq!(
            KernelEventCode::from(&EventKind::WeatherResolved).as_str(),
            "event.weather.resolved"
        );
        assert_eq!(
            KernelEventCode::from(&EventKind::TravelBlocked).as_str(),
            "event.travel.blocked"
        );
    }

    #[test]
    fn kernel_event_code_maps_every_event_kind() {
        let pairs = [
            (EventKind::LegacyLogKey, "event.legacy.log"),
            (EventKind::WeatherResolved, "event.weather.resolved"),
            (
                EventKind::DailyConsumptionApplied,
                "event.supplies.daily_consumption_applied",
            ),
            (EventKind::HealthTickApplied, "event.health.tick_applied"),
            (
                EventKind::GeneralStrainComputed,
                "event.health.general_strain_computed",
            ),
            (EventKind::ExecOrderStarted, "event.exec_order.started"),
            (EventKind::ExecOrderEnded, "event.exec_order.ended"),
            (
                EventKind::BreakdownStarted,
                "event.vehicle.breakdown_started",
            ),
            (
                EventKind::BreakdownResolved,
                "event.vehicle.breakdown_resolved",
            ),
            (EventKind::EncounterTriggered, "event.encounter.triggered"),
            (EventKind::RandomEventResolved, "event.random.resolved"),
            (EventKind::TradeResolved, "event.trade.resolved"),
            (EventKind::HuntResolved, "event.hunt.resolved"),
            (EventKind::AfflictionTriggered, "event.affliction.triggered"),
            (EventKind::NavigationEvent, "event.navigation.resolved"),
            (EventKind::CrossingResolved, "event.crossing.resolved"),
            (EventKind::TravelBlocked, "event.travel.blocked"),
        ];
        for (kind, expected) in pairs {
            assert_eq!(KernelEventCode::from(&kind).as_str(), expected);
        }
    }

    #[test]
    fn kernel_tick_output_conversion_maps_event_payloads() {
        let output: KernelTickOutput = sample_outcome(EventKind::EncounterTriggered, true).into();
        assert_eq!(output.day_kind, Some(TravelDayKind::Travel));
        assert_eq!(output.log_key, "log.sample");
        assert_eq!(output.events.len(), 1);
        let event = &output.events[0];
        assert_eq!(event.id, EventId::new(2, 0));
        assert_eq!(event.day, 2);
        assert_eq!(event.code, KernelEventCode::EncounterTriggered);
        assert_eq!(event.ui_key.as_deref(), Some("ui.sample"));
        assert_eq!(event.payload["key"], "value");
    }

    #[test]
    fn kernel_tick_output_handles_missing_record_kind() {
        let output: KernelTickOutput = sample_outcome(EventKind::TravelBlocked, false).into();
        assert_eq!(output.day_kind, None);
        assert_eq!(output.events[0].code, KernelEventCode::TravelBlocked);
    }

    #[test]
    fn kernel_session_new_forces_otdeluxe_mechanics() {
        let data = crate::EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let session = KernelSession::new(
            GameMode::Classic,
            StrategyId::Balanced,
            99,
            data,
            &endgame,
            Some(OtDeluxeOccupation::Banker),
        );
        assert_eq!(session.mechanics(), MechanicalPolicyId::OtDeluxe90s);
    }

    #[test]
    fn kernel_session_rejects_non_otdeluxe_state() {
        let data = crate::EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let state = GameState::default().with_seed(12, GameMode::Classic, data);
        let result = KernelSession::from_state(state, StrategyId::Balanced, &endgame);
        assert!(matches!(result, Err(KernelSessionError::NonOtDeluxePolicy)));
    }

    #[test]
    fn kernel_session_from_state_accepts_otdeluxe_and_exposes_policy() {
        let data = crate::EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let mut state = GameState::default().with_seed(33, GameMode::Classic, data);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        let session =
            KernelSession::from_state(state, StrategyId::Balanced, &endgame).expect("otdeluxe");
        assert_eq!(session.mechanics(), MechanicalPolicyId::OtDeluxe90s);
        assert_eq!(session.policy(), PolicyId::Classic);
    }

    #[test]
    fn kernel_session_tick_and_into_state_roundtrip() {
        let data = crate::EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let mut session = KernelSession::new(
            GameMode::Classic,
            StrategyId::Balanced,
            41,
            data,
            &endgame,
            Some(OtDeluxeOccupation::Banker),
        );
        let output = session.tick_day(KernelTickInput {
            intent: DayIntent::Continue,
        });
        assert!(!output.log_key.is_empty());
        let state = session.into_state();
        assert_eq!(state.mechanical_policy, MechanicalPolicyId::OtDeluxe90s);
    }
}
