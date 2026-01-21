//! Phase-scoped state accessors for the kernel tick pipeline.
//!
//! These wrappers prevent phase logic from directly grabbing `&mut GameState` so
//! each tick only touches the slices it owns.

use std::sync::OnceLock;

use crate::constants::{LOG_HUNT, LOG_STORE, LOG_TRADE, LOG_TRAVEL_BLOCKED, LOG_TRAVELED};
use crate::endgame::{self, EndgameTravelCfg};
use crate::journey::daily::{apply_daily_health, apply_daily_supplies_sanity};
use crate::journey::{
    DayTagSet, EventKind, EventSeverity, MechanicalPolicyId, RngPhase, TravelDayKind,
};
use crate::pacing::PacingConfig;
use crate::state::{DayIntent, GameState};
use crate::weather::DystrailRegionalWeather;
use crate::{hunt, trade};

pub(super) struct WeatherPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> WeatherPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self) {
        let weather_model = DystrailRegionalWeather::default();
        let previous = self.state.weather_state.today;
        let stats_before = self.state.stats.clone();
        let rng_bundle = self.state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::WeatherTick));
        crate::weather::process_daily_weather(self.state, &weather_model, rng_bundle.as_deref());
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let overrides = self.state.otdeluxe_policy_overrides();
            overrides
                .weather_effects
                .apply(&mut self.state.weather_effects);
        }
        let stats_after = self.state.stats.clone();
        let supplies_delta = stats_after.supplies - stats_before.supplies;
        let sanity_delta = stats_after.sanity - stats_before.sanity;
        let pants_delta = stats_after.pants - stats_before.pants;
        let hp_delta = stats_after.hp - stats_before.hp;
        let weather = self.state.weather_state.today;
        let effects = self.state.weather_effects;
        self.state.push_event(
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
                "rain_accum": self.state.weather_state.rain_accum,
                "snow_depth": self.state.weather_state.snow_depth
            }),
        );
    }
}

pub(super) struct ExecOrderPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> ExecOrderPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self) {
        let rng_bundle = self.state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::ExecOrders));
        self.state.tick_exec_order_state();
    }
}

pub(super) struct PacingPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> PacingPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn apply(&mut self) {
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        self.state.apply_pace_and_diet(default_pacing_config());
    }
}

pub(super) struct SuppliesPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> SuppliesPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self, cfg: &crate::journey::DailyTickConfig) {
        let rng_bundle = self.state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::DailyEffects));
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let food_before = self.state.ot_deluxe.inventory.food_lbs;
            let consumed = self.state.apply_otdeluxe_consumption();
            let food_after = self.state.ot_deluxe.inventory.food_lbs;
            self.state.push_event(
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
                    "alive_party": self.state.otdeluxe_alive_party_count(),
                    "pace": self.state.ot_deluxe.pace,
                    "rations": self.state.ot_deluxe.rations
                }),
            );
            return;
        }
        let stats_before = self.state.stats.clone();
        let _ = apply_daily_supplies_sanity(cfg, self.state);
        let stats_after = self.state.stats.clone();
        self.state.push_event(
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
}

pub(super) struct HealthPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> HealthPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(
        &mut self,
        cfg: &crate::journey::DailyTickConfig,
        strain_cfg: &crate::journey::StrainConfig,
    ) {
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            self.run_otdeluxe();
            return;
        }
        let stats_before = self.state.stats.clone();
        self.state.apply_starvation_tick();
        let rng_bundle = self.state.rng_bundle.clone();
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
            self.state.roll_daily_illness();
        }
        self.state.apply_deep_aggressive_sanity_guard();
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::DailyEffects));
            let _ = apply_daily_health(cfg, self.state);
        }
        let strain = self.state.update_general_strain(strain_cfg);
        self.state.push_event(
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
            self.state.tick_ally_attrition();
        }
        self.state.stats.clamp();
        let stats_after = self.state.stats.clone();
        self.state.push_event(
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

    fn run_otdeluxe(&mut self) {
        let rng_bundle = self.state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
        let health_before = self.state.ot_deluxe.health_general;
        let delta = self.state.apply_otdeluxe_health_update();
        let health_after = self.state.ot_deluxe.health_general;
        self.state.push_event(
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
        let _ = self.state.tick_otdeluxe_afflictions();
    }
}

pub(super) struct BossPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> BossPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self) -> Option<(bool, String, bool)> {
        let (ended, log_key, breakdown_started) = self.state.guard_boss_gate()?;
        record_gate_day(self.state, "boss_gate");
        Some((ended, log_key, breakdown_started))
    }
}

pub(super) struct WaitPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> WaitPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self) -> Option<(bool, String, bool)> {
        let reason_tag = if self.state.wait.ferry_wait_days_remaining > 0 {
            self.state.wait.ferry_wait_days_remaining =
                self.state.wait.ferry_wait_days_remaining.saturating_sub(1);
            Some("wait_ferry")
        } else if self.state.wait.drying_days_remaining > 0 {
            self.state.wait.drying_days_remaining =
                self.state.wait.drying_days_remaining.saturating_sub(1);
            Some("wait_drying")
        } else {
            None
        };

        let tag = reason_tag?;

        record_gate_day(self.state, tag);
        Some((false, String::from(LOG_TRAVELED), false))
    }
}

pub(super) struct IntentPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> IntentPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self) -> Option<(bool, String, bool)> {
        match self.state.intent.pending {
            DayIntent::Continue => None,
            DayIntent::CrossingChoicePending => Some((false, String::from(LOG_TRAVELED), false)),
            DayIntent::Rest => {
                let remaining = self.state.intent.rest_days_remaining.clamp(1, 9);
                let updated = remaining.saturating_sub(1);
                self.state.intent.rest_days_remaining = updated;
                if updated == 0 {
                    self.state.intent.pending = DayIntent::Continue;
                }
                record_gate_day(self.state, "intent_rest");
                Some((false, String::from(LOG_TRAVELED), false))
            }
            DayIntent::Trade => {
                self.state.intent.pending = DayIntent::Continue;
                self.state.intent.rest_days_remaining = 0;
                let rng_bundle = self.state.rng_bundle.clone();
                let outcome = if let Some(bundle) = rng_bundle.as_ref() {
                    let _guard = bundle.phase_guard_for(RngPhase::TradeTick);
                    let mut rng = bundle.trade();
                    trade::resolve_trade_with_rng(self.state, &mut *rng)
                } else {
                    trade::resolve_trade(self.state)
                };
                self.state.push_event(
                    EventKind::TradeResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null),
                );
                record_gate_day(self.state, "intent_trade");
                Some((false, String::from(LOG_TRADE), false))
            }
            DayIntent::Hunt => {
                self.state.intent.pending = DayIntent::Continue;
                self.state.intent.rest_days_remaining = 0;
                let rng_bundle = self.state.rng_bundle.clone();
                let outcome = if let Some(bundle) = rng_bundle.as_ref() {
                    let _guard = bundle.phase_guard_for(RngPhase::HuntTick);
                    let mut rng = bundle.hunt();
                    hunt::resolve_hunt_with_rng(self.state, &mut *rng)
                } else {
                    hunt::resolve_hunt(self.state)
                };
                self.state.push_event(
                    EventKind::HuntResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null),
                );
                record_gate_day(self.state, "intent_hunt");
                Some((false, String::from(LOG_HUNT), false))
            }
        }
    }
}

pub(super) struct PendingPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> PendingPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn resolve_pending_route_prompt(&mut self) -> Option<(bool, String, bool)> {
        if self.state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            self.state.pending_route_choice = None;
            return None;
        }
        if self.state.ot_deluxe.route.pending_prompt.is_none() {
            self.state.pending_route_choice = None;
            return None;
        }
        if let Some(choice) = self.state.pending_route_choice.take() {
            let _ = self.state.resolve_otdeluxe_route_prompt(choice);
        }
        Some((false, String::from(LOG_TRAVEL_BLOCKED), false))
    }

    pub(super) fn resolve_pending_store(&mut self) -> Option<(bool, String, bool)> {
        if self.state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            self.state.ot_deluxe.store.pending_node = None;
            self.state.ot_deluxe.store.pending_purchase = None;
            return None;
        }
        let pending_node = self.state.ot_deluxe.store.pending_node?;
        if let Some(lines) = self.state.ot_deluxe.store.pending_purchase.take() {
            if self
                .state
                .apply_otdeluxe_store_purchase(pending_node, &lines)
                .is_ok()
            {
                self.state.clear_otdeluxe_store_pending();
            }
            return Some((false, String::from(LOG_STORE), false));
        }
        Some((false, String::from(LOG_STORE), false))
    }

    pub(super) fn resolve_pending_crossing(&mut self) -> Option<(bool, String, bool)> {
        let rng_bundle = self.state.rng_bundle.clone();
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            if !self.state.ot_deluxe.crossing.choice_pending {
                return None;
            }
            if let Some(method) = self.state.ot_deluxe.crossing.chosen_method.take() {
                self.state.start_of_day();
                let _guard = rng_bundle
                    .as_ref()
                    .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
                if let Some((ended, log_key)) =
                    self.state.resolve_pending_otdeluxe_crossing_choice(method)
                {
                    return Some((ended, log_key, false));
                }
            }
            return Some((false, String::from(LOG_TRAVEL_BLOCKED), false));
        }

        self.state.pending_crossing?;
        if let Some(choice) = self.state.pending_crossing_choice.take() {
            self.state.start_of_day();
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
            if let Some((ended, log_key)) = self.state.resolve_pending_crossing_choice(choice) {
                return Some((ended, log_key, false));
            }
        }
        Some((false, String::from(LOG_TRAVEL_BLOCKED), false))
    }
}

pub(super) struct TravelPhase<'a> {
    state: &'a mut GameState,
}

impl<'a> TravelPhase<'a> {
    pub(super) const fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    pub(super) fn run(&mut self, endgame_cfg: &EndgameTravelCfg) -> (bool, String, bool) {
        let rng_bundle = self.state.rng_bundle.as_ref().map(std::rc::Rc::clone);

        if let Some(result) = self.state.pre_travel_checks() {
            return result;
        }

        let breakdown_started = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::VehicleBreakdown));
            self.state.vehicle_roll()
        };
        self.state.resolve_breakdown();
        if let Some(result) = self.state.handle_vehicle_state(breakdown_started) {
            return result;
        }
        if let Some(result) = self.state.handle_travel_block(breakdown_started) {
            return result;
        }
        if self.state.consume_otdeluxe_navigation_delay_day() {
            return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
        }

        if self.state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s
            && let Some(result) = {
                let _guard = rng_bundle
                    .as_ref()
                    .map(|bundle| bundle.phase_guard_for(RngPhase::EncounterTick));
                self.state
                    .process_encounter_flow(rng_bundle.as_ref(), breakdown_started)
            }
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            self.state
                .compute_travel_distance_today(default_pacing_config());
            self.state.apply_encounter_partial_travel();
            return result;
        }
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            self.state.apply_otdeluxe_pace_and_rations();
            self.state.compute_otdeluxe_travel_distance_today();
            if self.state.apply_otdeluxe_navigation_event() {
                return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
            }
        } else {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            self.state
                .compute_travel_distance_today(default_pacing_config());
        }

        let computed_miles_today =
            if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
                self.state.distance_today
            } else {
                self.state.distance_today.max(self.state.distance_today_raw)
            };
        endgame::run_endgame_controller(
            self.state,
            computed_miles_today,
            breakdown_started,
            endgame_cfg,
        );
        if let Some((ended, log)) = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::CrossingTick));
            self.state.handle_crossing_event(computed_miles_today)
        } {
            return (ended, log, breakdown_started);
        }

        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::RandomEventTick));
            self.state.apply_otdeluxe_random_event();
        }

        let additional_miles = (self.state.distance_today - self.state.current_day_miles).max(0.0);
        self.state
            .record_travel_day(TravelDayKind::Travel, additional_miles, "");
        self.state.apply_travel_wear_for_day(computed_miles_today);
        self.state.log_travel_debug();
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            self.state.queue_otdeluxe_store_if_available();
        }

        self.state.end_of_day();
        (false, String::from(LOG_TRAVELED), breakdown_started)
    }
}

fn default_pacing_config() -> &'static PacingConfig {
    static CONFIG: OnceLock<PacingConfig> = OnceLock::new();
    CONFIG.get_or_init(PacingConfig::default_config)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journey::{DayRecord, DayTag};
    use crate::state::{DayState, LifecycleState};

    #[test]
    fn record_gate_day_records_reason_when_day_is_empty() {
        let mut state = GameState {
            day_state: DayState {
                lifecycle: LifecycleState {
                    day_initialized: false,
                    did_end_of_day: false,
                    suppress_stop_ratio: false,
                    log_cursor: 0,
                    event_seq: 0,
                },
                ..DayState::default()
            },
            ..GameState::default()
        };
        record_gate_day(&mut state, "gate_reason");
        assert!(state.day_state.lifecycle.did_end_of_day);
        let last = state
            .day_records
            .last()
            .cloned()
            .unwrap_or_else(|| DayRecord::new(0, TravelDayKind::NonTravel, 0.0));
        assert!(last.tags.contains(&DayTag::new("gate_reason")));
    }
}
