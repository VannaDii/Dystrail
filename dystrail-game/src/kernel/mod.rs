//! OTDeluxe-first kernel facade.
//!
//! This module provides a stable, neutral simulation API that callers can use
//! while `state.rs`/`journey` internals continue to be decomposed into phase and
//! system modules.

pub mod events;
pub mod types;

use crate::endgame::EndgameTravelCfg;
use crate::journey::{JourneySession, MechanicalPolicyId, PolicyId, StrategyId};
use crate::mechanics::OtDeluxeOccupation;
use crate::state::GameMode;
use thiserror::Error;

pub use events::{KernelDecisionTrace, KernelEventCode, KernelEventPayload};
pub use types::{KernelEvent, KernelTickInput, KernelTickOutput};

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
    use crate::journey::{
        DayEffects, DayInputs, DayOutcome, DayRecord, DayTagSet, Event, EventId, EventKind,
        EventSeverity, StatsDelta, TravelDayKind,
    };
    use crate::state::{DayIntent, DietId, GameState, PaceId, Region, Season};
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
        assert_eq!(event.payload.as_value()["key"], "value");
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
