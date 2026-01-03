//! Kernel orchestration for a single simulated day.

use std::sync::OnceLock;

use crate::constants::LOG_TRAVELED;
use crate::day_accounting;
use crate::endgame::{self, EndgameTravelCfg};
use crate::journey::daily::{
    apply_daily_health, apply_daily_supplies_sanity, finalize_daily_effects,
};
use crate::journey::{DayOutcome, Event, EventId, JourneyCfg, RngPhase, TravelDayKind};
use crate::pacing::PacingConfig;
use crate::state::GameState;
use crate::weather::WeatherConfig;

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
            Self::run_weather_tick(state);
            Self::run_exec_order_tick(state);
            self.run_supplies_tick(state);
            self.run_health_tick(state);
            finalize_daily_effects(state);
        }
    }

    pub(crate) fn tick_day(&self, state: &mut GameState) -> DayOutcome {
        self.tick_day_with_hook(state, |_| {})
    }

    pub(crate) fn tick_day_with_hook<F>(&self, state: &mut GameState, hook: F) -> DayOutcome
    where
        F: FnOnce(&mut GameState),
    {
        self.apply_daily_physics(state);
        state.apply_pace_and_diet(default_pacing_config());
        hook(state);

        let (ended, log_key, breakdown_started) = self.run_travel_flow(state);
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
        self.tick_non_travel_day_with_hook(state, kind, miles, reason_tag, |_| {})
    }

    pub(crate) fn tick_non_travel_day_with_hook<F>(
        &self,
        state: &mut GameState,
        kind: TravelDayKind,
        miles: f32,
        reason_tag: &str,
        hook: F,
    ) -> f32
    where
        F: FnOnce(&mut GameState),
    {
        self.apply_daily_physics(state);
        hook(state);
        let credited_miles = if state.current_day_kind.is_none() {
            let credited = if matches!(kind, TravelDayKind::Partial) && miles <= 0.0 {
                day_accounting::partial_day_miles(state, miles)
            } else {
                miles
            };
            state.record_travel_day(kind, credited, reason_tag);
            credited
        } else {
            state.current_day_miles
        };
        state.end_of_day();
        credited_miles
    }

    fn run_travel_flow(&self, state: &mut GameState) -> (bool, String, bool) {
        let rng_bundle = state.rng_bundle.as_ref().map(std::rc::Rc::clone);

        if let Some(result) = state.guard_boss_gate() {
            return result;
        }
        if let Some(result) = state.pre_travel_checks() {
            return result;
        }

        let breakdown_started = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::VehicleBreakdown));
            state.vehicle_roll()
        };
        state.resolve_breakdown();
        if let Some(result) = state.handle_vehicle_state(breakdown_started) {
            return result;
        }
        if let Some(result) = state.handle_travel_block(breakdown_started) {
            return result;
        }

        if let Some(result) = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::EncounterTick));
            state.process_encounter_flow(rng_bundle.as_ref(), breakdown_started)
        } {
            return result;
        }

        let computed_miles_today = state.distance_today.max(state.distance_today_raw);
        state.apply_travel_wear();
        endgame::run_endgame_controller(
            state,
            computed_miles_today,
            breakdown_started,
            self.endgame_cfg,
        );
        if let Some((ended, log)) = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
            state.handle_crossing_event(computed_miles_today)
        } {
            return (ended, log, breakdown_started);
        }

        let additional_miles = (state.distance_today - state.current_day_miles).max(0.0);
        state.record_travel_day(TravelDayKind::Travel, additional_miles, "");
        state.log_travel_debug();

        if let Some(log_key) = state.failure_log_key() {
            state.end_of_day();
            return (true, String::from(log_key), breakdown_started);
        }

        state.end_of_day();
        (false, String::from(LOG_TRAVELED), breakdown_started)
    }

    fn run_weather_tick(state: &mut GameState) {
        let weather_cfg = WeatherConfig::default_config();
        let rng_bundle = state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::WeatherTick));
        crate::weather::process_daily_weather(state, &weather_cfg, rng_bundle.as_deref());
    }

    fn run_exec_order_tick(state: &mut GameState) {
        let rng_bundle = state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::ExecOrders));
        state.tick_exec_order_state();
    }

    fn run_supplies_tick(&self, state: &mut GameState) {
        let rng_bundle = state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::DailyEffects));
        let _ = apply_daily_supplies_sanity(&self.cfg.daily, state);
    }

    fn run_health_tick(&self, state: &mut GameState) {
        state.apply_starvation_tick();
        let rng_bundle = state.rng_bundle.clone();
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
            state.roll_daily_illness();
        }
        state.apply_deep_aggressive_sanity_guard();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::DailyEffects));
        let _ = apply_daily_health(&self.cfg.daily, state);
    }
}

fn default_pacing_config() -> &'static PacingConfig {
    static CONFIG: OnceLock<PacingConfig> = OnceLock::new();
    CONFIG.get_or_init(PacingConfig::default_config)
}
