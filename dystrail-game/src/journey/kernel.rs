//! Kernel orchestration for a single simulated day.

use std::sync::OnceLock;

use crate::day_accounting;
use crate::endgame::EndgameTravelCfg;
use crate::journey::{DayOutcome, Event, EventId, JourneyCfg, TravelDayKind, apply_daily_effect};
use crate::pacing::PacingConfig;
use crate::state::GameState;

pub(crate) struct DailyTickKernel<'a> {
    cfg: &'a JourneyCfg,
    endgame_cfg: &'a EndgameTravelCfg,
}

impl<'a> DailyTickKernel<'a> {
    pub(crate) const fn new(cfg: &'a JourneyCfg, endgame_cfg: &'a EndgameTravelCfg) -> Self {
        Self { cfg, endgame_cfg }
    }

    pub(crate) fn apply_daily_physics(&self, state: &mut GameState) {
        let starting_new_day = !state.day_state.lifecycle.day_initialized;
        state.start_of_day();
        if starting_new_day {
            state.run_daily_root_ticks();
            let _ = apply_daily_effect(&self.cfg.daily, state);
        }
    }

    pub(crate) fn tick_day(&self, state: &mut GameState) -> DayOutcome {
        self.apply_daily_physics(state);
        state.apply_pace_and_diet(default_pacing_config());

        let (ended, log_key, breakdown_started) = state.travel_next_leg(self.endgame_cfg);
        let day_consumed = state.day_state.lifecycle.did_end_of_day;
        let event_day = if day_consumed {
            state.day.saturating_sub(1)
        } else {
            state.day
        };
        let record = if day_consumed {
            state.day_records.last().cloned()
        } else {
            None
        };
        let events = vec![Event::legacy_log_key(
            EventId::new(event_day, 0),
            event_day,
            log_key.clone(),
        )];
        let decision_traces = std::mem::take(&mut state.decision_traces_today);
        DayOutcome {
            ended,
            log_key,
            breakdown_started,
            record,
            events,
            decision_traces,
        }
    }

    pub(crate) fn tick_non_travel_day(
        &self,
        state: &mut GameState,
        kind: TravelDayKind,
        miles: f32,
        reason_tag: &str,
    ) -> f32 {
        self.apply_daily_physics(state);
        let credited_miles = if matches!(kind, TravelDayKind::Partial) && miles <= 0.0 {
            day_accounting::partial_day_miles(state, miles)
        } else {
            miles
        };
        state.record_travel_day(kind, credited_miles, reason_tag);
        state.end_of_day();
        credited_miles
    }
}

fn default_pacing_config() -> &'static PacingConfig {
    static CONFIG: OnceLock<PacingConfig> = OnceLock::new();
    CONFIG.get_or_init(PacingConfig::default_config)
}
