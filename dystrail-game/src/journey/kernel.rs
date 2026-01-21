//! Kernel orchestration for a single simulated day.

use std::sync::OnceLock;

use crate::constants::{LOG_HUNT, LOG_STORE, LOG_TRADE, LOG_TRAVEL_BLOCKED, LOG_TRAVELED};
use crate::day_accounting;
use crate::endgame::{self, EndgameTravelCfg};
use crate::journey::daily::{apply_daily_health, apply_daily_supplies_sanity};
use crate::journey::{
    DayOutcome, DayTagSet, Event, EventId, EventKind, EventSeverity, JourneyCfg,
    MechanicalPolicyId, RngPhase, TravelDayKind,
};
use crate::pacing::PacingConfig;
use crate::state::{DayIntent, GameState};
use crate::weather::WeatherConfig;
use crate::{hunt, trade};

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
        }
    }

    pub(crate) fn tick_day(&self, state: &mut GameState) -> DayOutcome {
        self.tick_day_with_hook(state, |_| {})
    }

    pub(crate) fn tick_day_with_hook<F>(&self, state: &mut GameState, hook: F) -> DayOutcome
    where
        F: FnOnce(&mut GameState),
    {
        if let Some(outcome) = Self::resolve_pending_route_prompt(state) {
            return outcome;
        }
        if let Some(outcome) = Self::resolve_pending_crossing(state) {
            return outcome;
        }
        if let Some(outcome) = Self::resolve_pending_store(state) {
            return outcome;
        }
        self.apply_daily_physics(state);
        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            state.apply_pace_and_diet(default_pacing_config());
        }
        hook(state);

        if let Some((ended, log_key, breakdown_started)) = Self::run_boss_gate(state) {
            return Self::build_outcome(state, ended, log_key, breakdown_started);
        }

        if let Some((ended, log_key, breakdown_started)) = Self::run_wait_gate(state) {
            return Self::build_outcome(state, ended, log_key, breakdown_started);
        }

        if let Some((ended, log_key, breakdown_started)) = Self::run_intent_gate(state) {
            return Self::build_outcome(state, ended, log_key, breakdown_started);
        }

        let (ended, log_key, breakdown_started) = self.run_travel_flow(state);
        Self::build_outcome(state, ended, log_key, breakdown_started)
    }

    fn resolve_pending_crossing(state: &mut GameState) -> Option<DayOutcome> {
        let rng_bundle = state.rng_bundle.clone();
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            if !state.ot_deluxe.crossing.choice_pending {
                return None;
            }
            if let Some(method) = state.ot_deluxe.crossing.chosen_method.take() {
                state.start_of_day();
                let _guard = rng_bundle
                    .as_ref()
                    .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
                if let Some((ended, log_key)) =
                    state.resolve_pending_otdeluxe_crossing_choice(method)
                {
                    return Some(Self::build_outcome(state, ended, log_key, false));
                }
            }
            return Some(Self::build_outcome(
                state,
                false,
                String::from(LOG_TRAVEL_BLOCKED),
                false,
            ));
        }

        state.pending_crossing?;
        if let Some(choice) = state.pending_crossing_choice.take() {
            state.start_of_day();
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
            if let Some((ended, log_key)) = state.resolve_pending_crossing_choice(choice) {
                return Some(Self::build_outcome(state, ended, log_key, false));
            }
        }
        Some(Self::build_outcome(
            state,
            false,
            String::from(LOG_TRAVEL_BLOCKED),
            false,
        ))
    }

    fn resolve_pending_store(state: &mut GameState) -> Option<DayOutcome> {
        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            state.ot_deluxe.store.pending_node = None;
            state.ot_deluxe.store.pending_purchase = None;
            return None;
        }
        let pending_node = state.ot_deluxe.store.pending_node?;
        if let Some(lines) = state.ot_deluxe.store.pending_purchase.take() {
            if state
                .apply_otdeluxe_store_purchase(pending_node, &lines)
                .is_ok()
            {
                state.clear_otdeluxe_store_pending();
            }
            return Some(Self::build_outcome(
                state,
                false,
                String::from(LOG_STORE),
                false,
            ));
        }
        Some(Self::build_outcome(
            state,
            false,
            String::from(LOG_STORE),
            false,
        ))
    }

    fn resolve_pending_route_prompt(state: &mut GameState) -> Option<DayOutcome> {
        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            state.pending_route_choice = None;
            return None;
        }
        if state.ot_deluxe.route.pending_prompt.is_none() {
            state.pending_route_choice = None;
            return None;
        }
        if let Some(choice) = state.pending_route_choice.take() {
            let _ = state.resolve_otdeluxe_route_prompt(choice);
        }
        Some(Self::build_outcome(
            state,
            false,
            String::from(LOG_TRAVEL_BLOCKED),
            false,
        ))
    }

    fn build_outcome(
        state: &mut GameState,
        ended: bool,
        log_key: String,
        breakdown_started: bool,
    ) -> DayOutcome {
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
        let terminal_log_key = state.terminal_log_key.take();
        let resolved_log_key = terminal_log_key.unwrap_or(log_key);
        let resolved_ended = ended || state.ending.is_some();
        let mut events = std::mem::take(&mut state.events_today);
        let mut seq = state.day_state.lifecycle.event_seq;
        events.push(Event::legacy_log_key(
            EventId::new(event_day, seq),
            event_day,
            resolved_log_key.clone(),
        ));
        seq = seq.saturating_add(1);

        let log_start = usize::try_from(state.day_state.lifecycle.log_cursor).unwrap_or(0);
        let log_end = state.logs.len();
        let log_start = log_start.min(log_end);
        for log in &state.logs[log_start..log_end] {
            if log == &resolved_log_key {
                continue;
            }
            events.push(Event::legacy_log_key(
                EventId::new(event_day, seq),
                event_day,
                log.clone(),
            ));
            seq = seq.saturating_add(1);
        }
        state.day_state.lifecycle.log_cursor = u32::try_from(log_end).unwrap_or(u32::MAX);
        state.day_state.lifecycle.event_seq = seq;
        let decision_traces = std::mem::take(&mut state.decision_traces_today);
        DayOutcome {
            ended: resolved_ended,
            log_key: resolved_log_key,
            breakdown_started,
            day_consumed,
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
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            state.clear_today_travel_distance();
        }
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
        if state.consume_otdeluxe_navigation_delay_day() {
            return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
        }

        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s
            && let Some(result) = {
                let _guard = rng_bundle
                    .as_ref()
                    .map(|bundle| bundle.phase_guard_for(RngPhase::EncounterTick));
                state.process_encounter_flow(rng_bundle.as_ref(), breakdown_started)
            }
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            state.compute_travel_distance_today(default_pacing_config());
            state.apply_encounter_partial_travel();
            return result;
        }
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            state.apply_otdeluxe_pace_and_rations();
            state.compute_otdeluxe_travel_distance_today();
            if state.apply_otdeluxe_navigation_event() {
                return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
            }
        } else {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            state.compute_travel_distance_today(default_pacing_config());
        }

        let computed_miles_today = if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            state.distance_today
        } else {
            state.distance_today.max(state.distance_today_raw)
        };
        self.run_endgame_tick(state, computed_miles_today, breakdown_started);
        if let Some((ended, log)) = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
            state.handle_crossing_event(computed_miles_today)
        } {
            return (ended, log, breakdown_started);
        }

        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::RandomEventTick));
            state.apply_otdeluxe_random_event();
        }

        let additional_miles = (state.distance_today - state.current_day_miles).max(0.0);
        state.record_travel_day(TravelDayKind::Travel, additional_miles, "");
        state.apply_travel_wear_for_day(computed_miles_today);
        state.log_travel_debug();
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            state.queue_otdeluxe_store_if_available();
        }

        state.end_of_day();
        (false, String::from(LOG_TRAVELED), breakdown_started)
    }

    fn run_endgame_tick(
        &self,
        state: &mut GameState,
        computed_miles_today: f32,
        breakdown_started: bool,
    ) {
        endgame::run_endgame_controller(
            state,
            computed_miles_today,
            breakdown_started,
            self.endgame_cfg,
        );
    }

    fn run_wait_gate(state: &mut GameState) -> Option<(bool, String, bool)> {
        let reason_tag = if state.wait.ferry_wait_days_remaining > 0 {
            state.wait.ferry_wait_days_remaining =
                state.wait.ferry_wait_days_remaining.saturating_sub(1);
            Some("wait_ferry")
        } else if state.wait.drying_days_remaining > 0 {
            state.wait.drying_days_remaining = state.wait.drying_days_remaining.saturating_sub(1);
            Some("wait_drying")
        } else {
            None
        };

        let tag = reason_tag?;

        Self::record_gate_day(state, tag);
        Some((false, String::from(LOG_TRAVELED), false))
    }

    fn run_intent_gate(state: &mut GameState) -> Option<(bool, String, bool)> {
        match state.intent.pending {
            DayIntent::Continue => None,
            DayIntent::CrossingChoicePending => Some((false, String::from(LOG_TRAVELED), false)),
            DayIntent::Rest => {
                let remaining = state.intent.rest_days_remaining.clamp(1, 9);
                let updated = remaining.saturating_sub(1);
                state.intent.rest_days_remaining = updated;
                if updated == 0 {
                    state.intent.pending = DayIntent::Continue;
                }
                Self::record_intent_day(state, "intent_rest");
                Some((false, String::from(LOG_TRAVELED), false))
            }
            DayIntent::Trade => {
                state.intent.pending = DayIntent::Continue;
                state.intent.rest_days_remaining = 0;
                let rng_bundle = state.rng_bundle.clone();
                let outcome = if let Some(bundle) = rng_bundle.as_ref() {
                    let _guard = bundle.phase_guard_for(RngPhase::TradeTick);
                    let mut rng = bundle.trade();
                    trade::resolve_trade_with_rng(state, &mut *rng)
                } else {
                    trade::resolve_trade(state)
                };
                state.push_event(
                    EventKind::TradeResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null),
                );
                Self::record_intent_day(state, "intent_trade");
                Some((false, String::from(LOG_TRADE), false))
            }
            DayIntent::Hunt => {
                state.intent.pending = DayIntent::Continue;
                state.intent.rest_days_remaining = 0;
                let rng_bundle = state.rng_bundle.clone();
                let outcome = if let Some(bundle) = rng_bundle.as_ref() {
                    let _guard = bundle.phase_guard_for(RngPhase::HuntTick);
                    let mut rng = bundle.hunt();
                    hunt::resolve_hunt_with_rng(state, &mut *rng)
                } else {
                    hunt::resolve_hunt(state)
                };
                state.push_event(
                    EventKind::HuntResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null),
                );
                Self::record_intent_day(state, "intent_hunt");
                Some((false, String::from(LOG_HUNT), false))
            }
        }
    }

    fn record_intent_day(state: &mut GameState, reason_tag: &str) {
        Self::record_gate_day(state, reason_tag);
    }

    fn record_gate_day(state: &mut GameState, reason_tag: &str) {
        state.start_of_day();
        if state.current_day_kind.is_none() {
            state.day_state.lifecycle.suppress_stop_ratio = true;
            state.record_travel_day(TravelDayKind::NonTravel, 0.0, reason_tag);
        } else {
            state.add_day_reason_tag(reason_tag);
        }
        state.end_of_day();
    }

    fn run_boss_gate(state: &mut GameState) -> Option<(bool, String, bool)> {
        let (ended, log_key, breakdown_started) = state.guard_boss_gate()?;
        Self::record_gate_day(state, "boss_gate");
        Some((ended, log_key, breakdown_started))
    }

    fn run_weather_tick(state: &mut GameState) {
        let weather_cfg = WeatherConfig::default_config();
        let previous = state.weather_state.today;
        let stats_before = state.stats.clone();
        let rng_bundle = state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::WeatherTick));
        crate::weather::process_daily_weather(state, &weather_cfg, rng_bundle.as_deref());
        let stats_after = state.stats.clone();
        let supplies_delta = stats_after.supplies - stats_before.supplies;
        let sanity_delta = stats_after.sanity - stats_before.sanity;
        let pants_delta = stats_after.pants - stats_before.pants;
        let hp_delta = stats_after.hp - stats_before.hp;
        let weather = state.weather_state.today;
        let effects = state.weather_effects;
        state.push_event(
            EventKind::WeatherResolved,
            EventSeverity::Info,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "previous": previous,
                "weather": weather,
                "stats_delta": {
                    "supplies": supplies_delta,
                    "sanity": sanity_delta,
                    "pants": pants_delta,
                    "hp": hp_delta
                },
                "travel_mult": effects.travel_mult,
                "encounter_delta": effects.encounter_delta,
                "encounter_cap": effects.encounter_cap,
                "breakdown_mult": effects.breakdown_mult,
                "rain_accum": state.weather_state.rain_accum,
                "snow_depth": state.weather_state.snow_depth
            }),
        );
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
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let food_before = state.ot_deluxe.inventory.food_lbs;
            let consumed = state.apply_otdeluxe_consumption();
            let food_after = state.ot_deluxe.inventory.food_lbs;
            state.push_event(
                EventKind::DailyConsumptionApplied,
                EventSeverity::Info,
                DayTagSet::new(),
                None,
                None,
                serde_json::json!({
                    "policy": "otdeluxe90s",
                    "food_before_lbs": food_before,
                    "food_after_lbs": food_after,
                    "consumed_lbs": consumed,
                    "alive_party": state.otdeluxe_alive_party_count(),
                    "pace": state.ot_deluxe.pace,
                    "rations": state.ot_deluxe.rations
                }),
            );
            return;
        }
        let stats_before = state.stats.clone();
        let _ = apply_daily_supplies_sanity(&self.cfg.daily, state);
        let stats_after = state.stats.clone();
        state.push_event(
            EventKind::DailyConsumptionApplied,
            EventSeverity::Info,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "policy": "dystrail",
                "supplies_before": stats_before.supplies,
                "supplies_after": stats_after.supplies,
                "supplies_delta": stats_after.supplies - stats_before.supplies,
                "sanity_before": stats_before.sanity,
                "sanity_after": stats_after.sanity,
                "sanity_delta": stats_after.sanity - stats_before.sanity
            }),
        );
    }

    fn run_health_tick(&self, state: &mut GameState) {
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            Self::run_otdeluxe_health_tick(state);
            return;
        }
        let stats_before = state.stats.clone();
        state.apply_starvation_tick();
        let rng_bundle = state.rng_bundle.clone();
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
            state.roll_daily_illness();
        }
        state.apply_deep_aggressive_sanity_guard();
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::DailyEffects));
            let _ = apply_daily_health(&self.cfg.daily, state);
        }
        let strain = state.update_general_strain(&self.cfg.strain);
        state.push_event(
            EventKind::GeneralStrainComputed,
            EventSeverity::Info,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({ "value": strain }),
        );
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
            state.tick_ally_attrition();
        }
        state.stats.clamp();
        let stats_after = state.stats.clone();
        state.push_event(
            EventKind::HealthTickApplied,
            EventSeverity::Info,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "policy": "dystrail",
                "hp_before": stats_before.hp,
                "hp_after": stats_after.hp,
                "hp_delta": stats_after.hp - stats_before.hp,
                "sanity_before": stats_before.sanity,
                "sanity_after": stats_after.sanity,
                "sanity_delta": stats_after.sanity - stats_before.sanity,
                "supplies_before": stats_before.supplies,
                "supplies_after": stats_after.supplies,
                "supplies_delta": stats_after.supplies - stats_before.supplies,
                "pants_before": stats_before.pants,
                "pants_after": stats_after.pants,
                "pants_delta": stats_after.pants - stats_before.pants
            }),
        );
    }

    fn run_otdeluxe_health_tick(state: &mut GameState) {
        let rng_bundle = state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
        let health_before = state.ot_deluxe.health_general;
        let delta = state.apply_otdeluxe_health_update();
        let health_after = state.ot_deluxe.health_general;
        state.push_event(
            EventKind::HealthTickApplied,
            EventSeverity::Info,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "policy": "otdeluxe90s",
                "health_general_before": health_before,
                "health_general_after": health_after,
                "health_delta": delta
            }),
        );
        let _ = state.tick_otdeluxe_afflictions();
    }
}

fn default_pacing_config() -> &'static PacingConfig {
    static CONFIG: OnceLock<PacingConfig> = OnceLock::new();
    CONFIG.get_or_init(PacingConfig::default_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        CROSSING_MILESTONES, LOG_BOSS_AWAIT, LOG_STORE, LOG_TRAVEL_BLOCKED, LOG_TRAVELED,
    };
    use crate::crossings::{CrossingChoice, CrossingKind};
    use crate::exec_orders::ExecOrder;
    use crate::journey::{
        DailyChannelConfig, DailyTickConfig, EventKind, HealthTickConfig, JourneyCfg,
        MechanicalPolicyId, RngBundle,
    };
    use crate::numbers::round_f32_to_i32;
    use crate::otdeluxe_state::{
        OtDeluxeCrossingMethod, OtDeluxePartyMember, OtDeluxeRiver, OtDeluxeRiverBed,
        OtDeluxeRiverState, OtDeluxeRouteDecision, OtDeluxeRoutePrompt, OtDeluxeWagonState,
    };
    use crate::otdeluxe_store::{OtDeluxeStoreItem, OtDeluxeStoreLineItem};
    use crate::state::{DayIntent, GameState, PendingCrossing, Region, Spares, Stats};
    use crate::vehicle::{Breakdown, Part};
    use crate::weather::{Weather, WeatherConfig, WeatherState, select_weather_for_today};
    use std::rc::Rc;

    fn seed_with_non_clear_weather(cfg: &WeatherConfig) -> (u64, Weather) {
        let base_state = GameState {
            region: Region::Heartland,
            ..GameState::default()
        };
        for seed in 1_u64..10_000 {
            let mut probe_state = base_state.clone();
            let rng_bundle = RngBundle::from_user_seed(seed);
            if let Ok(weather) = select_weather_for_today(&mut probe_state, cfg, &rng_bundle)
                && weather != Weather::Clear
            {
                return (seed, weather);
            }
        }
        panic!("unable to find seed that produces non-clear weather");
    }

    #[test]
    fn daily_physics_applies_weather_before_supplies() {
        let weather_cfg = WeatherConfig::default_config();
        let (seed, expected_weather) = seed_with_non_clear_weather(&weather_cfg);

        let mut supplies = DailyChannelConfig::new(1.0);
        supplies.weather.insert(Weather::Clear, 1.0);
        supplies.weather.insert(Weather::Storm, 5.0);
        supplies.weather.insert(Weather::HeatWave, 3.0);
        supplies.weather.insert(Weather::ColdSnap, 2.0);
        supplies.weather.insert(Weather::Smoke, 4.0);

        let daily = DailyTickConfig {
            supplies,
            sanity: DailyChannelConfig::new(0.0),
            health: HealthTickConfig {
                decay: 0.0,
                rest_heal: 0.0,
                ..HealthTickConfig::default()
            },
        };

        let cfg = JourneyCfg {
            daily,
            ..JourneyCfg::default()
        };
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            region: Region::Heartland,
            weather_state: WeatherState {
                today: Weather::Clear,
                ..WeatherState::default()
            },
            current_order: Some(ExecOrder::WarDeptReorg),
            exec_order_days_remaining: 2,
            stats: Stats {
                supplies: 20,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));

        kernel.apply_daily_physics(&mut state);

        let weather_effect = weather_cfg
            .effects
            .get(&expected_weather)
            .map_or(0, |effect| effect.supplies);
        let weather_multiplier = cfg
            .daily
            .supplies
            .weather
            .get(&expected_weather)
            .copied()
            .unwrap_or(1.0);
        let daily_supplies_delta = -round_f32_to_i32(cfg.daily.supplies.base * weather_multiplier);
        let expected_supplies = 20 + weather_effect + daily_supplies_delta;

        assert_eq!(state.stats.supplies, expected_supplies);
    }

    #[test]
    fn daily_physics_burns_supplies_before_starvation_tick() {
        let daily = DailyTickConfig {
            supplies: DailyChannelConfig::new(2.0),
            sanity: DailyChannelConfig::new(0.0),
            health: HealthTickConfig {
                decay: 0.0,
                rest_heal: 0.0,
                ..HealthTickConfig::default()
            },
        };

        let cfg = JourneyCfg {
            daily,
            ..JourneyCfg::default()
        };
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            current_order: Some(ExecOrder::WarDeptReorg),
            exec_order_days_remaining: 2,
            stats: Stats {
                supplies: 1,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(11)));

        kernel.apply_daily_physics(&mut state);

        assert_eq!(state.stats.supplies, 0);
        assert_eq!(state.starvation_days, 1);
    }

    #[test]
    fn daily_physics_runs_exec_order_tick_before_supplies() {
        let weather_cfg = WeatherConfig::default_config();
        let seed = 7_u64;
        let mut probe_state = GameState {
            region: Region::Heartland,
            ..GameState::default()
        };
        let expected_weather = select_weather_for_today(
            &mut probe_state,
            &weather_cfg,
            &RngBundle::from_user_seed(seed),
        )
        .expect("weather selection");

        let mut supplies = DailyChannelConfig::new(2.0);
        supplies
            .exec
            .insert(String::from(ExecOrder::TravelBanLite.key()), 3.0);

        let daily = DailyTickConfig {
            supplies,
            sanity: DailyChannelConfig::new(0.0),
            health: HealthTickConfig {
                decay: 0.0,
                rest_heal: 0.0,
                ..HealthTickConfig::default()
            },
        };

        let cfg = JourneyCfg {
            daily,
            ..JourneyCfg::default()
        };
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            region: Region::Heartland,
            current_order: Some(ExecOrder::TravelBanLite),
            exec_order_days_remaining: 1,
            stats: Stats {
                supplies: 10,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));

        kernel.apply_daily_physics(&mut state);

        let weather_effect = weather_cfg
            .effects
            .get(&expected_weather)
            .map_or(0, |effect| effect.supplies);
        let weather_multiplier = cfg
            .daily
            .supplies
            .weather
            .get(&expected_weather)
            .copied()
            .unwrap_or(1.0);
        let daily_supplies_delta = -round_f32_to_i32(cfg.daily.supplies.base * weather_multiplier);
        let mut expected_stats = Stats {
            supplies: 10 + weather_effect + daily_supplies_delta,
            ..Stats::default()
        };
        expected_stats.clamp();

        assert_eq!(state.stats.supplies, expected_stats.supplies);
        assert!(state.current_order.is_none());
    }

    #[test]
    fn daily_physics_uses_only_weather_rng_when_health_and_events_gated() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            current_order: Some(ExecOrder::WarDeptReorg),
            exec_order_days_remaining: 2,
            illness_days_remaining: 1,
            stats: Stats {
                allies: 0,
                ..Stats::default()
            },
            ..GameState::default()
        };

        let bundle = Rc::new(RngBundle::from_user_seed(123));
        state.attach_rng_bundle(bundle.clone());

        kernel.apply_daily_physics(&mut state);

        assert!(bundle.weather().draws() > 0);
        assert_eq!(bundle.health().draws(), 0);
        assert_eq!(bundle.events().draws(), 0);
        assert_eq!(bundle.travel().draws(), 0);
        assert_eq!(bundle.breakdown().draws(), 0);
        assert_eq!(bundle.encounter().draws(), 0);
        assert_eq!(bundle.crossing().draws(), 0);
        assert_eq!(bundle.boss().draws(), 0);
        assert_eq!(bundle.trade().draws(), 0);
        assert_eq!(bundle.hunt().draws(), 0);
    }

    #[test]
    fn daily_physics_runs_once_per_day() {
        let daily = DailyTickConfig {
            supplies: DailyChannelConfig::new(2.0),
            sanity: DailyChannelConfig::new(0.0),
            health: HealthTickConfig {
                decay: 0.0,
                rest_heal: 0.0,
                ..HealthTickConfig::default()
            },
        };
        let cfg = JourneyCfg {
            daily,
            ..JourneyCfg::default()
        };
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            weather_state: WeatherState {
                today: Weather::Clear,
                ..WeatherState::default()
            },
            stats: Stats {
                supplies: 10,
                ..Stats::default()
            },
            ..GameState::default()
        };

        kernel.apply_daily_physics(&mut state);
        let supplies_after_first = state.stats.supplies;
        let starvation_after_first = state.starvation_days;

        kernel.apply_daily_physics(&mut state);

        assert_eq!(state.stats.supplies, supplies_after_first);
        assert_eq!(state.starvation_days, starvation_after_first);
    }

    #[test]
    fn daily_physics_runs_otdeluxe_supplies_and_health() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.food_lbs = 200;
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        state.ot_deluxe.oxen.healthy = 4;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(33)));

        kernel.apply_daily_physics(&mut state);

        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::DailyConsumptionApplied)
        );
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::HealthTickApplied)
        );
    }

    #[test]
    fn tick_day_emits_new_logs_as_events() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            data: None,
            stats: Stats {
                supplies: 20,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.logs.push(String::from("log.previous"));
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(44)));

        let outcome = kernel.tick_day_with_hook(&mut state, |state| {
            state.logs.push(String::from("log.hook"));
        });

        let keys: Vec<&str> = outcome
            .events
            .iter()
            .filter_map(|event| event.ui_key.as_deref())
            .collect();
        assert!(keys.contains(&outcome.log_key.as_str()));
        assert!(keys.contains(&"log.hook"));
        assert!(!keys.contains(&"log.previous"));
    }

    #[test]
    fn tick_day_phase_order_emits_core_events_in_sequence() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            encounter_chance_today: 0.0,
            exec_order_cooldown: 1,
            stats: Stats {
                supplies: 20,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(11)));

        let outcome = kernel.tick_day(&mut state);

        let find_kind = |kind| {
            outcome
                .events
                .iter()
                .position(|event| event.kind == kind)
                .unwrap_or_else(|| panic!("missing event {kind:?}"))
        };
        let weather_idx = find_kind(EventKind::WeatherResolved);
        let supplies_idx = find_kind(EventKind::DailyConsumptionApplied);
        let strain_idx = find_kind(EventKind::GeneralStrainComputed);
        let health_idx = find_kind(EventKind::HealthTickApplied);

        assert!(weather_idx < supplies_idx);
        assert!(supplies_idx < strain_idx);
        assert!(strain_idx < health_idx);
        assert!(outcome.day_consumed);
    }

    #[test]
    fn pending_crossing_blocks_without_consuming_day() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 0.0,
            }),
            ..GameState::default()
        };

        let outcome = kernel.tick_day(&mut state);

        assert!(!outcome.day_consumed);
        assert!(outcome.record.is_none());
        assert_eq!(state.day, 1);
        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
    }

    #[test]
    fn pending_otdeluxe_crossing_choice_resolves_with_method() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.inventory.cash_cents = 10_000;
        state.ot_deluxe.party.members = vec![
            OtDeluxePartyMember::new("Ada"),
            OtDeluxePartyMember::new("Bea"),
        ];
        state.ot_deluxe.crossing.choice_pending = true;
        state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Ferry);
        state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 200.0,
            depth_ft: 2.5,
            swiftness: 1.2,
            bed: OtDeluxeRiverBed::Muddy,
        });
        state.ot_deluxe.crossing.computed_miles_today = 6.0;

        let outcome =
            DailyTickKernel::resolve_pending_crossing(&mut state).expect("expected outcome");

        assert!(outcome.day_consumed);
        assert!(!state.ot_deluxe.crossing.choice_pending);
    }

    #[test]
    fn pending_otdeluxe_crossing_without_method_blocks() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.crossing.choice_pending = true;

        let outcome =
            DailyTickKernel::resolve_pending_crossing(&mut state).expect("expected outcome");

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(!outcome.day_consumed);
    }

    #[test]
    fn pending_crossing_choice_invalid_blocks() {
        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 0.0,
            }),
            pending_crossing_choice: Some(crate::crossings::CrossingChoice::Permit),
            ..GameState::default()
        };

        let outcome =
            DailyTickKernel::resolve_pending_crossing(&mut state).expect("expected outcome");

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(!outcome.day_consumed);
    }

    #[test]
    fn pending_crossing_choice_pass_resolves() {
        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 5.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Detour),
            ..GameState::default()
        };

        let outcome =
            DailyTickKernel::resolve_pending_crossing(&mut state).expect("expected outcome");

        assert!(outcome.day_consumed);
        assert!(state.pending_crossing.is_none());
        assert!(state.pending_crossing_choice.is_none());
    }

    #[test]
    fn pending_route_prompt_blocks_without_consuming_day() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);

        let outcome = kernel.tick_day(&mut state);

        assert!(!outcome.day_consumed);
        assert!(outcome.record.is_none());
        assert_eq!(state.day, 1);
        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
    }

    #[test]
    fn pending_route_prompt_clears_choice_when_prompt_missing() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.pending_route_choice = Some(OtDeluxeRouteDecision::SubletteCutoff);
        state.ot_deluxe.route.pending_prompt = None;

        let outcome = DailyTickKernel::resolve_pending_route_prompt(&mut state);

        assert!(outcome.is_none());
        assert!(state.pending_route_choice.is_none());
    }

    #[test]
    fn pending_route_prompt_clears_choice_for_non_otdeluxe() {
        let mut state = GameState {
            pending_route_choice: Some(OtDeluxeRouteDecision::SubletteCutoff),
            ..GameState::default()
        };

        let outcome = DailyTickKernel::resolve_pending_route_prompt(&mut state);

        assert!(outcome.is_none());
        assert!(state.pending_route_choice.is_none());
    }

    #[test]
    fn wait_gate_consumes_non_travel_day_and_decrements_counter() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.wait.ferry_wait_days_remaining = 1;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(state.wait.ferry_wait_days_remaining, 0);
        assert_eq!(state.day, 2);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "wait_ferry"));
    }

    #[test]
    fn wait_gate_tracks_drying_days() {
        let mut state = GameState::default();
        state.wait.drying_days_remaining = 1;

        let outcome = DailyTickKernel::run_wait_gate(&mut state);

        assert!(outcome.is_some());
        assert_eq!(state.wait.drying_days_remaining, 0);
        let record = state.day_records.last().expect("expected day record");
        assert!(record.tags.iter().any(|tag| tag.0 == "wait_drying"));
    }

    #[test]
    fn record_gate_day_appends_reason_when_day_recorded() {
        let mut state = GameState::default();
        state.start_of_day();
        state.record_travel_day(TravelDayKind::Travel, 8.0, "");

        DailyTickKernel::record_gate_day(&mut state, "gate_reason");

        let record = state.day_records.last().expect("expected day record");
        assert!(record.tags.iter().any(|tag| tag.0 == "gate_reason"));
    }

    #[test]
    fn boss_gate_records_non_travel_day_and_blocks_waits() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.boss.readiness.ready = true;
        state.wait.ferry_wait_days_remaining = 2;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_BOSS_AWAIT);
        assert_eq!(state.wait.ferry_wait_days_remaining, 2);
        assert_eq!(state.day, 2);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected boss gate record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "boss_gate"));
    }

    #[test]
    fn boss_gate_skips_otdeluxe_runs() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.boss.readiness.ready = true;
        state.ot_deluxe.oxen.healthy = 4;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(88)));

        let outcome = kernel.tick_day(&mut state);

        assert_ne!(outcome.log_key, LOG_BOSS_AWAIT);
        assert!(
            !outcome
                .record
                .as_ref()
                .is_some_and(|record| record.tags.iter().any(|tag| tag.0 == "boss_gate"))
        );
    }

    #[test]
    fn otdeluxe_store_purchase_resolves_without_consuming_day() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.cash_cents = 10_000;
        state.ot_deluxe.store.pending_node = Some(0);
        state.ot_deluxe.store.pending_purchase = Some(vec![OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::Oxen,
            quantity: 2,
        }]);

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_STORE);
        assert!(!outcome.day_consumed);
        assert_eq!(state.day, 1);
        assert_eq!(state.ot_deluxe.oxen.healthy, 2);
        assert!(state.ot_deluxe.store.pending_node.is_none());
        assert_eq!(state.ot_deluxe.store.last_node, Some(0));
    }

    #[test]
    fn otdeluxe_store_purchase_error_keeps_pending_node() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.cash_cents = 0;
        state.ot_deluxe.store.pending_node = Some(0);
        state.ot_deluxe.store.pending_purchase = Some(vec![OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::Oxen,
            quantity: 1,
        }]);

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_STORE);
        assert!(!outcome.day_consumed);
        assert_eq!(state.ot_deluxe.store.pending_node, Some(0));
        assert!(state.ot_deluxe.store.pending_purchase.is_none());
    }

    #[test]
    fn pending_store_is_cleared_for_non_otdeluxe() {
        let mut state = GameState::default();
        state.ot_deluxe.store.pending_node = Some(2);
        state.ot_deluxe.store.pending_purchase = Some(Vec::new());

        let outcome = DailyTickKernel::resolve_pending_store(&mut state);

        assert!(outcome.is_none());
        assert!(state.ot_deluxe.store.pending_node.is_none());
        assert!(state.ot_deluxe.store.pending_purchase.is_none());
    }

    #[test]
    fn otdeluxe_no_oxen_blocks_travel() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.sick = 1;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(state.day, 2);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Blocked
        ));
    }

    #[test]
    fn tick_non_travel_day_with_existing_record_preserves_miles() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            trail_distance: 500.0,
            ..GameState::default()
        };

        let credited = kernel.tick_non_travel_day_with_hook(
            &mut state,
            TravelDayKind::NonTravel,
            0.0,
            "gate",
            |state| {
                state.record_travel_day(TravelDayKind::Travel, 12.0, "");
            },
        );

        assert!((credited - 12.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn tick_non_travel_day_clears_distance_for_otdeluxe() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.distance_today = 12.0;
        state.distance_today_raw = 12.0;

        kernel.tick_non_travel_day(&mut state, TravelDayKind::NonTravel, 0.0, "gate");

        assert!(state.distance_today.abs() <= f32::EPSILON);
        assert!(state.distance_today_raw.abs() <= f32::EPSILON);
    }

    #[test]
    fn otdeluxe_navigation_delay_blocks_travel() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.travel.delay_days_remaining = 1;
        state.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Delayed;
        state.vehicle.breakdown_cooldown = 2;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "otdeluxe.nav_delay"));
        assert_eq!(state.ot_deluxe.travel.delay_days_remaining, 0);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Moving
        ));
    }

    #[test]
    fn otdeluxe_navigation_blocked_days_consume_day() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.travel.blocked_days_remaining = 1;
        state.vehicle.breakdown_cooldown = 2;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected day record");
        assert!(
            record
                .tags
                .iter()
                .any(|tag| tag.0 == "otdeluxe.nav_blocked")
        );
        assert_eq!(state.ot_deluxe.travel.blocked_days_remaining, 0);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Moving
        ));
    }

    #[test]
    fn otdeluxe_travel_flow_reaches_navigation_roll() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            encounter_chance_today: 0.0,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.healthy = 4;
        state.vehicle.breakdown_cooldown = 2;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(101)));

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVELED);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected day record");
        assert!(matches!(
            record.kind,
            TravelDayKind::Travel | TravelDayKind::Partial
        ));
        assert!(state.distance_today > 0.0);
    }

    #[test]
    fn travel_flow_blocks_on_crossing_event() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            miles_traveled_actual: CROSSING_MILESTONES[0],
            encounter_chance_today: 0.0,
            ..GameState::default()
        };
        state.vehicle.breakdown_cooldown = 2;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(42)));

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(!outcome.day_consumed);
        assert!(state.pending_crossing.is_some());
    }

    #[test]
    fn intent_gate_crossing_choice_pending_blocks_day() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.intent.pending = DayIntent::CrossingChoicePending;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVELED);
        assert!(!outcome.day_consumed);
    }

    #[test]
    fn rest_intent_counts_down_and_records_non_travel_days() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.intent.pending = DayIntent::Rest;
        state.intent.rest_days_remaining = 2;

        let first = kernel.tick_day(&mut state);
        assert!(first.day_consumed);
        assert!(matches!(state.intent.pending, DayIntent::Rest));
        assert_eq!(state.intent.rest_days_remaining, 1);
        let first_record = first.record.expect("expected first record");
        assert!(matches!(first_record.kind, TravelDayKind::NonTravel));
        assert!(first_record.tags.iter().any(|tag| tag.0 == "intent_rest"));

        let second = kernel.tick_day(&mut state);
        assert!(second.day_consumed);
        assert!(matches!(state.intent.pending, DayIntent::Continue));
        assert_eq!(state.intent.rest_days_remaining, 0);
        let second_record = second.record.expect("expected second record");
        assert!(matches!(second_record.kind, TravelDayKind::NonTravel));
        assert!(second_record.tags.iter().any(|tag| tag.0 == "intent_rest"));
        assert_eq!(state.day_records.len(), 2);
    }

    #[test]
    fn trade_intent_consumes_day_and_uses_trade_rng() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.intent.pending = DayIntent::Trade;
        state.ot_deluxe.inventory.cash_cents = 2_500;
        let bundle = Rc::new(RngBundle::from_user_seed(77));
        state.attach_rng_bundle(bundle.clone());

        let outcome = kernel.tick_day(&mut state);

        assert!(matches!(state.intent.pending, DayIntent::Continue));
        assert!(bundle.trade().draws() > 0);
        assert!(outcome.day_consumed);
        assert!(
            outcome
                .events
                .iter()
                .any(|event| event.kind == EventKind::TradeResolved),
            "expected trade resolved event"
        );
        let record = outcome.record.expect("expected trade record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "intent_trade"));
    }

    #[test]
    fn trade_intent_consumes_day_without_rng() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.intent.pending = DayIntent::Trade;

        let outcome = kernel.tick_day(&mut state);

        assert!(outcome.day_consumed);
        assert!(
            outcome
                .events
                .iter()
                .any(|event| event.kind == EventKind::TradeResolved),
            "expected trade resolved event"
        );
    }

    #[test]
    fn hunt_intent_consumes_day_and_spends_bullets() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.intent.pending = DayIntent::Hunt;
        state.ot_deluxe.inventory.bullets = 10;
        state.ot_deluxe.party.members = vec![crate::otdeluxe_state::OtDeluxePartyMember::new("A")];
        let bundle = Rc::new(RngBundle::from_user_seed(93));
        state.attach_rng_bundle(bundle.clone());

        let outcome = kernel.tick_day(&mut state);

        assert!(matches!(state.intent.pending, DayIntent::Continue));
        assert!(bundle.hunt().draws() > 0);
        assert!(state.ot_deluxe.inventory.bullets < 10);
        assert!(
            outcome
                .events
                .iter()
                .any(|event| event.kind == EventKind::HuntResolved),
            "expected hunt resolved event"
        );
        let record = outcome.record.expect("expected hunt record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "intent_hunt"));
    }

    #[test]
    fn hunt_intent_consumes_day_without_rng() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.intent.pending = DayIntent::Hunt;
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        state.ot_deluxe.inventory.bullets = 5;

        let outcome = kernel.tick_day(&mut state);

        assert!(outcome.day_consumed);
        assert!(
            outcome
                .events
                .iter()
                .any(|event| event.kind == EventKind::HuntResolved),
            "expected hunt resolved event"
        );
    }
    #[test]
    fn route_prompt_choice_resolves_and_blocks_day() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);
        state.pending_route_choice = Some(OtDeluxeRouteDecision::SubletteCutoff);

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(!outcome.day_consumed);
        assert!(state.ot_deluxe.route.pending_prompt.is_none());
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Moving
        ));
    }

    #[test]
    fn pending_otdeluxe_crossing_choice_resolves() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("A")];
        state.ot_deluxe.crossing.choice_pending = true;
        state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Ford);
        state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState::default());
        state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        state.ot_deluxe.crossing.computed_miles_today = 5.0;
        state.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Stopped;

        let outcome = kernel.tick_day(&mut state);

        assert!(outcome.day_consumed);
        assert!(!state.ot_deluxe.crossing.choice_pending);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Moving | OtDeluxeWagonState::Delayed
        ));
    }

    #[test]
    fn pending_store_without_purchase_blocks_until_confirmed() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.store.pending_node = Some(0);
        state.ot_deluxe.store.pending_purchase = None;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_STORE);
        assert!(!outcome.day_consumed);
        assert!(state.ot_deluxe.store.pending_node.is_some());
    }

    #[test]
    fn travel_flow_blocks_on_breakdown() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: i32::try_from(state.day).unwrap_or(0),
        });
        state.budget_cents = 0;
        state.inventory.spares = Spares::default();
        state.vehicle.breakdown_cooldown = 0;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(13)));

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(outcome.day_consumed);
        let record = outcome.record.expect("expected record");
        assert!(record.tags.iter().any(|tag| tag.0 == "repair"));
    }

    #[test]
    fn non_travel_partial_uses_partial_day_miles() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        let credited = kernel.tick_non_travel_day_with_hook(
            &mut state,
            TravelDayKind::Partial,
            0.0,
            "pause",
            |state| {
                state.distance_today = 10.0;
            },
        );

        assert!(credited > 0.0);
        let record = state.day_records.last().expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::Partial));
    }

    #[test]
    fn build_outcome_prefers_terminal_log_key() {
        let mut state = GameState {
            terminal_log_key: Some(String::from("log.terminal")),
            ..GameState::default()
        };
        state.logs.push(String::from("log.extra"));

        let outcome =
            DailyTickKernel::build_outcome(&mut state, false, String::from("log.fallback"), false);

        assert_eq!(outcome.log_key, "log.terminal");
        assert!(!outcome.day_consumed);
        assert!(outcome.record.is_none());
        assert!(
            outcome
                .events
                .iter()
                .any(|event| event.ui_key.as_deref() == Some("log.terminal"))
        );
    }

    #[test]
    fn build_outcome_skips_duplicate_log_entries() {
        let mut state = GameState::default();
        state.logs.push(String::from(LOG_TRAVELED));

        let outcome =
            DailyTickKernel::build_outcome(&mut state, false, String::from(LOG_TRAVELED), false);

        let logged = outcome
            .events
            .iter()
            .filter(|event| event.ui_key.as_deref() == Some(LOG_TRAVELED))
            .count();
        assert_eq!(logged, 1);
    }

    #[test]
    fn travel_flow_records_travel_day_without_crossing() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.vehicle.breakdown_cooldown = 2;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(5)));

        let outcome = kernel.tick_day_with_hook(&mut state, |state| {
            state.encounter_chance_today = 0.0;
        });

        assert_eq!(outcome.log_key, LOG_TRAVELED);
        assert!(outcome.day_consumed);
        assert!(state.current_day_miles >= 0.0);
        assert!(state.day_records.last().is_some());
    }

    #[test]
    fn resolve_pending_store_applies_purchase_and_clears_pending() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.store.pending_node = Some(0);
        state.ot_deluxe.inventory.cash_cents = 10_000;
        state.ot_deluxe.store.pending_purchase = Some(vec![OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::FoodLb,
            quantity: 10,
        }]);

        let outcome = DailyTickKernel::resolve_pending_store(&mut state).expect("outcome");

        assert_eq!(outcome.log_key, LOG_STORE);
        assert!(state.ot_deluxe.store.pending_node.is_none());
        assert!(state.ot_deluxe.store.pending_purchase.is_none());
    }

    #[test]
    fn resolve_pending_route_prompt_consumes_choice() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);
        state.pending_route_choice = Some(OtDeluxeRouteDecision::SubletteCutoff);

        let outcome = DailyTickKernel::resolve_pending_route_prompt(&mut state).expect("outcome");

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(state.pending_route_choice.is_none());
    }

    #[test]
    fn resolve_pending_crossing_detour_applies_outcome() {
        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 5.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Detour),
            ..GameState::default()
        };

        let outcome = DailyTickKernel::resolve_pending_crossing(&mut state).expect("outcome");

        assert_eq!(outcome.log_key, crate::constants::LOG_CROSSING_DETOUR);
        assert!(state.pending_crossing.is_none());
    }

    #[test]
    fn resolve_pending_otdeluxe_crossing_blocks_without_context() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.crossing.choice_pending = true;
        state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Ford);

        let outcome = DailyTickKernel::resolve_pending_crossing(&mut state).expect("outcome");

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(!outcome.day_consumed);
    }
}
