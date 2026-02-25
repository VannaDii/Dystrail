//! OTDeluxe-first kernel facade.
//!
//! This module provides a stable, neutral simulation API that callers can use
//! while `state.rs`/`journey` internals continue to be decomposed into phase and
//! system modules.

pub mod events;
pub mod phases;
pub mod session;
pub mod systems;
pub mod types;

pub use events::{
    KERNEL_EVENT_CODE_SCHEMA_VERSION, KERNEL_EVENT_CODES, KernelDecisionTrace, KernelEventCode,
    KernelEventPayload,
};
pub use session::{KernelSession, KernelSessionError};
pub use types::{KernelEvent, KernelTickInput, KernelTickOutput};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endgame::EndgameTravelCfg;
    use crate::journey::{
        DayEffects, DayInputs, DayOutcome, DayRecord, DayTagSet, Event, EventId, EventKind,
        EventSeverity, MechanicalPolicyId, PolicyId, StatsDelta, StrategyId, TravelDayKind,
    };
    use crate::mechanics::OtDeluxeOccupation;
    use crate::state::{DayIntent, DietId, GameMode, GameState, PaceId, Region, Season};
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
            EventKind::LegacyLogKey,
            EventKind::WeatherResolved,
            EventKind::DailyConsumptionApplied,
            EventKind::HealthTickApplied,
            EventKind::GeneralStrainComputed,
            EventKind::ExecOrderStarted,
            EventKind::ExecOrderEnded,
            EventKind::BreakdownStarted,
            EventKind::BreakdownResolved,
            EventKind::EncounterTriggered,
            EventKind::RandomEventResolved,
            EventKind::TradeResolved,
            EventKind::HuntResolved,
            EventKind::AfflictionTriggered,
            EventKind::NavigationEvent,
            EventKind::CrossingResolved,
            EventKind::TravelBlocked,
        ];
        assert_eq!(pairs.len(), KERNEL_EVENT_CODES.len());
        for (index, kind) in pairs.into_iter().enumerate() {
            let code = KernelEventCode::from(&kind);
            assert_eq!(code, KERNEL_EVENT_CODES[index]);
            assert!(!code.as_str().is_empty());
        }
    }

    #[test]
    fn kernel_event_schema_guardrails_are_stable() {
        assert_eq!(KERNEL_EVENT_CODE_SCHEMA_VERSION, 1);
        let mut seen = std::collections::BTreeSet::new();
        for code in KERNEL_EVENT_CODES {
            assert!(seen.insert(code.as_str()));
        }
        assert_eq!(seen.len(), KERNEL_EVENT_CODES.len());
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
