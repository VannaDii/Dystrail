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
use crate::mechanics::otdeluxe90s::OtDeluxeNavigationPolicy;
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
        let bundle = self.state.rng_bundle.clone();
        let _guard = phase_guard(bundle.as_deref(), RngPhase::WeatherTick);
        crate::weather::process_daily_weather(self.state, &weather_model, bundle.as_deref());
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            apply_otdeluxe_weather_overrides(self.state);
        }
        let stats_after = self.state.stats.clone();
        let supplies_delta = stats_after.supplies - stats_before.supplies;
        let sanity_delta = stats_after.sanity - stats_before.sanity;
        let pants_delta = stats_after.pants - stats_before.pants;
        let hp_delta = stats_after.hp - stats_before.hp;
        let weather = self.state.weather_state.today;
        let effects = self.state.weather_effects;
        let mut stats_delta = serde_json::Map::new();
        stats_delta.insert(String::from("supplies"), serde_json::json!(supplies_delta));
        stats_delta.insert(String::from("sanity"), serde_json::json!(sanity_delta));
        stats_delta.insert(String::from("pants"), serde_json::json!(pants_delta));
        stats_delta.insert(String::from("hp"), serde_json::json!(hp_delta));
        let mut payload = serde_json::Map::new();
        payload.insert(String::from("previous"), serde_json::json!(previous));
        payload.insert(String::from("weather"), serde_json::json!(weather));
        payload.insert(
            String::from("stats_delta"),
            serde_json::Value::Object(stats_delta),
        );
        payload.insert(
            String::from("travel_mult"),
            serde_json::json!(effects.travel_mult),
        );
        payload.insert(
            String::from("encounter_delta"),
            serde_json::json!(effects.encounter_delta),
        );
        payload.insert(
            String::from("encounter_cap"),
            serde_json::json!(effects.encounter_cap),
        );
        payload.insert(
            String::from("breakdown_mult"),
            serde_json::json!(effects.breakdown_mult),
        );
        payload.insert(
            String::from("rain_accum"),
            serde_json::json!(self.state.weather_state.rain_accum),
        );
        payload.insert(
            String::from("snow_depth"),
            serde_json::json!(self.state.weather_state.snow_depth),
        );
        let kind = EventKind::WeatherResolved;
        let severity = EventSeverity::Info;
        let tags = DayTagSet::new();
        let payload_value = serde_json::Value::Object(payload);
        emit_event(self.state, kind, severity, tags, payload_value);
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
        let bundle = self.state.rng_bundle.clone();
        let _guard = phase_guard(bundle.as_deref(), RngPhase::ExecOrders);
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
        if self.state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            self.state.apply_pace_and_diet(default_pacing_config());
        }
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
        let bundle = self.state.rng_bundle.clone();
        let _guard = phase_guard(bundle.as_deref(), RngPhase::DailyEffects);
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let food_before = self.state.ot_deluxe.inventory.food_lbs;
            let consumed = self.state.apply_otdeluxe_consumption();
            let food_after = self.state.ot_deluxe.inventory.food_lbs;
            let mut payload = serde_json::Map::new();
            payload.insert(String::from("policy"), serde_json::json!("otdeluxe90s"));
            payload.insert(
                String::from("food_before_lbs"),
                serde_json::json!(food_before),
            );
            payload.insert(
                String::from("food_after_lbs"),
                serde_json::json!(food_after),
            );
            payload.insert(String::from("consumed_lbs"), serde_json::json!(consumed));
            payload.insert(
                String::from("alive_party"),
                serde_json::json!(self.state.otdeluxe_alive_party_count()),
            );
            payload.insert(
                String::from("pace"),
                serde_json::json!(self.state.ot_deluxe.pace),
            );
            payload.insert(
                String::from("rations"),
                serde_json::json!(self.state.ot_deluxe.rations),
            );
            let kind = EventKind::DailyConsumptionApplied;
            let severity = EventSeverity::Info;
            let tags = DayTagSet::new();
            let payload_value = serde_json::Value::Object(payload);
            emit_event(self.state, kind, severity, tags, payload_value);
        } else {
            let stats_before = self.state.stats.clone();
            let _ = apply_daily_supplies_sanity(cfg, self.state);
            let stats_after = self.state.stats.clone();
            let mut payload = serde_json::Map::new();
            payload.insert(String::from("policy"), serde_json::json!("dystrail"));
            payload.insert(
                String::from("supplies_before"),
                serde_json::json!(stats_before.supplies),
            );
            payload.insert(
                String::from("supplies_after"),
                serde_json::json!(stats_after.supplies),
            );
            payload.insert(
                String::from("supplies_delta"),
                serde_json::json!(stats_after.supplies - stats_before.supplies),
            );
            payload.insert(
                String::from("sanity_before"),
                serde_json::json!(stats_before.sanity),
            );
            payload.insert(
                String::from("sanity_after"),
                serde_json::json!(stats_after.sanity),
            );
            payload.insert(
                String::from("sanity_delta"),
                serde_json::json!(stats_after.sanity - stats_before.sanity),
            );
            let kind = EventKind::DailyConsumptionApplied;
            let severity = EventSeverity::Info;
            let tags = DayTagSet::new();
            let payload_value = serde_json::Value::Object(payload);
            emit_event(self.state, kind, severity, tags, payload_value);
        }
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
        } else {
            self.run_dystrail(cfg, strain_cfg);
        }
    }

    fn run_dystrail(
        &mut self,
        cfg: &crate::journey::DailyTickConfig,
        strain_cfg: &crate::journey::StrainConfig,
    ) {
        let stats_before = self.state.stats.clone();
        let rng_bundle = self.state.rng_bundle.clone();
        self.apply_dystrail_health_ticks(cfg, rng_bundle.as_deref());
        let strain = self.state.update_general_strain(strain_cfg);
        self.push_strain_event(strain);
        self.apply_dystrail_ally_attrition(rng_bundle.as_deref());
        self.state.stats.clamp();
        let stats_after = self.state.stats.clone();
        self.push_health_event(&stats_before, &stats_after);
    }

    fn apply_dystrail_health_ticks(
        &mut self,
        cfg: &crate::journey::DailyTickConfig,
        rng_bundle: Option<&crate::journey::RngBundle>,
    ) {
        self.state.apply_starvation_tick();
        {
            let _guard = phase_guard(rng_bundle, RngPhase::HealthTick);
            self.state.roll_daily_illness();
        }
        self.state.apply_deep_aggressive_sanity_guard();
        {
            let _guard = phase_guard(rng_bundle, RngPhase::DailyEffects);
            let _ = apply_daily_health(cfg, self.state);
        }
    }

    fn apply_dystrail_ally_attrition(&mut self, rng_bundle: Option<&crate::journey::RngBundle>) {
        let _guard = phase_guard(rng_bundle, RngPhase::HealthTick);
        self.state.tick_ally_attrition();
    }

    fn push_strain_event(&mut self, strain: f32) {
        let mut strain_payload = serde_json::Map::new();
        strain_payload.insert(String::from("value"), serde_json::json!(strain));
        let strain_value = serde_json::Value::Object(strain_payload);
        emit_strain_event(self.state, strain_value);
    }

    fn push_health_event(
        &mut self,
        stats_before: &crate::state::Stats,
        stats_after: &crate::state::Stats,
    ) {
        let mut health_payload = serde_json::Map::new();
        health_payload.insert(String::from("policy"), serde_json::json!("dystrail"));
        health_payload.insert(
            String::from("hp_before"),
            serde_json::json!(stats_before.hp),
        );
        health_payload.insert(String::from("hp_after"), serde_json::json!(stats_after.hp));
        health_payload.insert(
            String::from("hp_delta"),
            serde_json::json!(stats_after.hp - stats_before.hp),
        );
        health_payload.insert(
            String::from("sanity_before"),
            serde_json::json!(stats_before.sanity),
        );
        health_payload.insert(
            String::from("sanity_after"),
            serde_json::json!(stats_after.sanity),
        );
        health_payload.insert(
            String::from("sanity_delta"),
            serde_json::json!(stats_after.sanity - stats_before.sanity),
        );
        health_payload.insert(
            String::from("supplies_before"),
            serde_json::json!(stats_before.supplies),
        );
        health_payload.insert(
            String::from("supplies_after"),
            serde_json::json!(stats_after.supplies),
        );
        health_payload.insert(
            String::from("supplies_delta"),
            serde_json::json!(stats_after.supplies - stats_before.supplies),
        );
        health_payload.insert(
            String::from("pants_before"),
            serde_json::json!(stats_before.pants),
        );
        health_payload.insert(
            String::from("pants_after"),
            serde_json::json!(stats_after.pants),
        );
        health_payload.insert(
            String::from("pants_delta"),
            serde_json::json!(stats_after.pants - stats_before.pants),
        );
        let health_value = serde_json::Value::Object(health_payload);
        emit_health_tick_event(self.state, health_value);
    }

    fn run_otdeluxe(&mut self) {
        let bundle = self.state.rng_bundle.clone();
        let _guard = phase_guard(bundle.as_deref(), RngPhase::HealthTick);
        let health_before = self.state.ot_deluxe.health_general;
        let delta = self.state.apply_otdeluxe_health_update();
        let health_after = self.state.ot_deluxe.health_general;
        let mut payload = serde_json::Map::new();
        payload.insert(String::from("policy"), serde_json::json!("otdeluxe90s"));
        payload.insert(
            String::from("health_general_before"),
            serde_json::json!(health_before),
        );
        payload.insert(
            String::from("health_general_after"),
            serde_json::json!(health_after),
        );
        payload.insert(String::from("health_delta"), serde_json::json!(delta));
        let kind = EventKind::HealthTickApplied;
        let severity = EventSeverity::Info;
        let tags = DayTagSet::new();
        let payload_value = serde_json::Value::Object(payload);
        emit_event(self.state, kind, severity, tags, payload_value);
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
            DayIntent::Rest => Some(self.handle_rest_intent()),
            DayIntent::Trade => Some(self.handle_trade_intent()),
            DayIntent::Hunt => Some(self.handle_hunt_intent()),
        }
    }

    fn handle_rest_intent(&mut self) -> (bool, String, bool) {
        let remaining = self.state.intent.rest_days_remaining.clamp(1, 9);
        let updated = remaining.saturating_sub(1);
        self.state.intent.rest_days_remaining = updated;
        if updated == 0 {
            self.state.intent.pending = DayIntent::Continue;
        }
        record_gate_day(self.state, "intent_rest");
        (false, String::from(LOG_TRAVELED), false)
    }

    fn handle_trade_intent(&mut self) -> (bool, String, bool) {
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
        let kind = EventKind::TradeResolved;
        let severity = EventSeverity::Info;
        let tags = DayTagSet::new();
        let payload = serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null);
        emit_event(self.state, kind, severity, tags, payload);
        record_gate_day(self.state, "intent_trade");
        (false, String::from(LOG_TRADE), false)
    }

    fn handle_hunt_intent(&mut self) -> (bool, String, bool) {
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
        let kind = EventKind::HuntResolved;
        let severity = EventSeverity::Info;
        let tags = DayTagSet::new();
        let payload = serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null);
        emit_event(self.state, kind, severity, tags, payload);
        record_gate_day(self.state, "intent_hunt");
        (false, String::from(LOG_HUNT), false)
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
            let applied = apply_pending_store_purchase(self.state, pending_node, &lines);
            if applied {
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
                let _guard = phase_guard(rng_bundle.as_deref(), RngPhase::CrossingTick);
                let result = self.state.resolve_pending_otdeluxe_crossing_choice(method);
                if let Some((ended, log_key)) = result {
                    return Some((ended, log_key, false));
                }
            }
            return Some((false, String::from(LOG_TRAVEL_BLOCKED), false));
        }

        self.state.pending_crossing?;
        if let Some(choice) = self.state.pending_crossing_choice.take() {
            self.state.start_of_day();
            let _guard = phase_guard(rng_bundle.as_deref(), RngPhase::CrossingTick);
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
        self.run_with_navigation_policy(endgame_cfg, None)
    }

    fn run_with_navigation_policy(
        &mut self,
        endgame_cfg: &EndgameTravelCfg,
        navigation_policy: Option<&OtDeluxeNavigationPolicy>,
    ) -> (bool, String, bool) {
        let rng_bundle = self.state.rng_bundle.as_ref().map(std::rc::Rc::clone);
        let bundle_ref = rng_bundle.as_ref();
        let guard_bundle = rng_bundle.as_deref();

        if let Some(result) = self.state.pre_travel_checks() {
            return result;
        }

        let breakdown_started = with_phase_guard(guard_bundle, RngPhase::VehicleBreakdown, || {
            self.state.vehicle_roll()
        });
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

        if self.state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            let encounter_result = with_phase_guard(guard_bundle, RngPhase::EncounterTick, || {
                self.state
                    .process_encounter_flow(bundle_ref, breakdown_started)
            });
            if let Some(result) = encounter_result {
                with_phase_guard(guard_bundle, RngPhase::TravelTick, || {
                    let pacing = default_pacing_config();
                    self.state.compute_travel_distance_today(pacing);
                    self.state.apply_encounter_partial_travel();
                });
                return result;
            }
        }
        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = phase_guard(guard_bundle, RngPhase::TravelTick);
            self.state.apply_otdeluxe_pace_and_rations();
            self.state.compute_otdeluxe_travel_distance_today();
            let navigation_applied = if let Some(policy) = navigation_policy {
                self.state
                    .apply_otdeluxe_navigation_event_with_policy(policy)
            } else {
                self.state.apply_otdeluxe_navigation_event()
            };
            if navigation_applied {
                return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
            }
        } else {
            let _guard = phase_guard(guard_bundle, RngPhase::TravelTick);
            let pacing = default_pacing_config();
            self.state.compute_travel_distance_today(pacing);
        }

        let policy = self.state.mechanical_policy;
        let distance = self.state.distance_today;
        let raw = self.state.distance_today_raw;
        let is_otdeluxe = policy == MechanicalPolicyId::OtDeluxe90s;
        let miles_today = if is_otdeluxe {
            distance
        } else {
            distance.max(raw)
        };
        endgame::run_endgame_controller(self.state, miles_today, breakdown_started, endgame_cfg);
        let crossing_result = with_phase_guard(guard_bundle, RngPhase::CrossingTick, || {
            self.state.handle_crossing_event(miles_today)
        });
        if let Some((ended, log)) = crossing_result {
            return (ended, log, breakdown_started);
        }

        if self.state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = phase_guard(guard_bundle, RngPhase::RandomEventTick);
            self.state.apply_otdeluxe_random_event();
        }

        let additional_miles = (self.state.distance_today - self.state.current_day_miles).max(0.0);
        self.state
            .record_travel_day(TravelDayKind::Travel, additional_miles, "");
        self.state.apply_travel_wear_for_day(miles_today);
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

fn apply_otdeluxe_weather_overrides(state: &mut GameState) {
    let overrides = state.otdeluxe_policy_overrides();
    overrides.weather_effects.apply(&mut state.weather_effects);
}

fn phase_guard(
    bundle: Option<&crate::journey::RngBundle>,
    phase: RngPhase,
) -> Option<crate::journey::RngPhaseGuard<'_>> {
    bundle.map(|bundle| bundle.phase_guard_for(phase))
}

fn with_phase_guard<T, F>(
    bundle: Option<&crate::journey::RngBundle>,
    phase: RngPhase,
    action: F,
) -> T
where
    F: FnOnce() -> T,
{
    let _guard = phase_guard(bundle, phase);
    action()
}

fn emit_event(
    state: &mut GameState,
    kind: EventKind,
    severity: EventSeverity,
    tags: DayTagSet,
    payload: serde_json::Value,
) {
    state.push_event(kind, severity, tags, None, None, payload);
}

#[rustfmt::skip]
fn emit_strain_event(state: &mut GameState, payload: serde_json::Value) { let tags = DayTagSet::new(); emit_event(state, EventKind::GeneralStrainComputed, EventSeverity::Info, tags, payload); }

#[rustfmt::skip]
fn emit_health_tick_event(state: &mut GameState, payload: serde_json::Value) { let tags = DayTagSet::new(); emit_event(state, EventKind::HealthTickApplied, EventSeverity::Info, tags, payload); }

#[rustfmt::skip]
fn apply_pending_store_purchase(state: &mut GameState, pending_node: u8, lines: &[crate::otdeluxe_store::OtDeluxeStoreLineItem]) -> bool { state.apply_otdeluxe_store_purchase(pending_node, lines).is_ok() }

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
    use crate::constants::{LOG_TRAVEL_BLOCKED, LOG_TRAVELED, LOG_VEHICLE_FAILURE};
    use crate::crossings::{CrossingChoice, CrossingKind};
    use crate::data::EncounterData;
    use crate::journey::{
        DayRecord, DayTag, EventKind, JourneyCfg, MechanicalPolicyId, RngBundle, TravelDayKind,
    };
    use crate::mechanics::otdeluxe90s::OtDeluxeNavigationPolicy;
    use crate::otdeluxe_state::{
        OtDeluxeCrossingMethod, OtDeluxeInventory, OtDeluxePartyMember, OtDeluxeRiver,
        OtDeluxeRiverBed, OtDeluxeRiverState, OtDeluxeState,
    };
    use crate::otdeluxe_store::{OtDeluxeStoreItem, OtDeluxeStoreLineItem};
    use crate::state::{
        DayIntent, DayState, GameMode, IntentState, LifecycleState, PendingCrossing, PolicyKind,
        Spares,
    };
    use std::rc::Rc;

    fn state_with_rng(seed: u64) -> GameState {
        let mut state = GameState::default();
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));
        state
    }

    fn sample_encounter_data() -> EncounterData {
        EncounterData::from_encounters(vec![crate::data::Encounter {
            id: String::from("encounter"),
            name: String::from("Encounter"),
            desc: String::new(),
            weight: 1,
            regions: vec![String::from("heartland")],
            modes: vec![String::from("classic")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }])
    }

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

    #[test]
    fn weather_phase_emits_event_for_otdeluxe_with_rng() {
        let mut state = state_with_rng(1);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        WeatherPhase::new(&mut state).run();
        assert!(
            state
                .events_today
                .iter()
                .any(|event| matches!(event.kind, EventKind::WeatherResolved))
        );
    }

    #[test]
    fn pacing_phase_handles_otdeluxe_and_dystrail() {
        let mut otdeluxe_state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        let stats_before = otdeluxe_state.stats.clone();
        PacingPhase::new(&mut otdeluxe_state).apply();
        assert_eq!(otdeluxe_state.stats, stats_before);

        let mut dystrail_state = GameState::default();
        PacingPhase::new(&mut dystrail_state).apply();
        assert!((0.0..=1.0).contains(&dystrail_state.encounter_chance_today));
    }

    #[test]
    fn supplies_phase_covers_otdeluxe_and_dystrail() {
        let cfg = crate::journey::DailyTickConfig::default();
        let mut otdeluxe_state = state_with_rng(2);
        otdeluxe_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        otdeluxe_state.ot_deluxe.inventory.food_lbs = 100;
        SuppliesPhase::new(&mut otdeluxe_state).run(&cfg);
        assert!(
            otdeluxe_state
                .events_today
                .iter()
                .any(|event| matches!(event.kind, EventKind::DailyConsumptionApplied))
        );

        let mut dystrail_state = state_with_rng(3);
        dystrail_state.mechanical_policy = MechanicalPolicyId::DystrailLegacy;
        SuppliesPhase::new(&mut dystrail_state).run(&cfg);
        assert!(
            dystrail_state
                .events_today
                .iter()
                .any(|event| matches!(event.kind, EventKind::DailyConsumptionApplied))
        );
    }

    #[test]
    fn health_phase_covers_otdeluxe_and_dystrail() {
        let cfg = crate::journey::DailyTickConfig::default();
        let strain_cfg = crate::journey::StrainConfig::default();

        let mut otdeluxe_state = state_with_rng(4);
        otdeluxe_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        HealthPhase::new(&mut otdeluxe_state).run(&cfg, &strain_cfg);
        assert!(
            otdeluxe_state
                .events_today
                .iter()
                .any(|event| matches!(event.kind, EventKind::HealthTickApplied))
        );

        let mut dystrail_state = state_with_rng(5);
        dystrail_state.mechanical_policy = MechanicalPolicyId::DystrailLegacy;
        HealthPhase::new(&mut dystrail_state).run(&cfg, &strain_cfg);
        let kinds: Vec<_> = dystrail_state
            .events_today
            .iter()
            .map(|event| event.kind.clone())
            .collect();
        assert!(kinds.contains(&EventKind::GeneralStrainComputed));
        assert!(kinds.contains(&EventKind::HealthTickApplied));
    }

    #[test]
    fn intent_phase_handles_rest_trade_hunt() {
        let mut rest_state = GameState {
            intent: IntentState {
                pending: DayIntent::Rest,
                rest_days_remaining: 1,
            },
            ..GameState::default()
        };
        let rest_outcome = IntentPhase::new(&mut rest_state).run();
        assert!(rest_outcome.is_some());
        assert!(rest_state.intent.pending == DayIntent::Continue);
        let rest_tag = rest_state
            .day_records
            .last()
            .is_some_and(|record| record.tags.contains(&DayTag::new("intent_rest")));
        assert!(rest_tag);

        let mut trade_state = GameState {
            intent: IntentState {
                pending: DayIntent::Trade,
                rest_days_remaining: 0,
            },
            ..state_with_rng(6)
        };
        let trade_outcome = IntentPhase::new(&mut trade_state).run();
        assert!(trade_outcome.is_some());
        let trade_tag = trade_state
            .day_records
            .last()
            .is_some_and(|record| record.tags.contains(&DayTag::new("intent_trade")));
        assert!(trade_tag);

        let mut hunt_state = GameState {
            intent: IntentState {
                pending: DayIntent::Hunt,
                rest_days_remaining: 0,
            },
            ot_deluxe: OtDeluxeState {
                inventory: OtDeluxeInventory {
                    bullets: 40,
                    ..OtDeluxeInventory::default()
                },
                ..OtDeluxeState::default()
            },
            ..state_with_rng(7)
        };
        let hunt_outcome = IntentPhase::new(&mut hunt_state).run();
        assert!(hunt_outcome.is_some());
        let hunt_tag = hunt_state
            .day_records
            .last()
            .is_some_and(|record| record.tags.contains(&DayTag::new("intent_hunt")));
        assert!(hunt_tag);
    }

    #[test]
    fn rest_intent_clamps_days_to_parity_range() {
        let mut over_state = GameState {
            intent: IntentState {
                pending: DayIntent::Rest,
                rest_days_remaining: 12,
            },
            ..GameState::default()
        };
        IntentPhase::new(&mut over_state).run();
        assert!(matches!(over_state.intent.pending, DayIntent::Rest));
        assert_eq!(over_state.intent.rest_days_remaining, 8);

        let mut zero_state = GameState {
            intent: IntentState {
                pending: DayIntent::Rest,
                rest_days_remaining: 0,
            },
            ..GameState::default()
        };
        IntentPhase::new(&mut zero_state).run();
        assert!(matches!(zero_state.intent.pending, DayIntent::Continue));
        assert_eq!(zero_state.intent.rest_days_remaining, 0);
    }

    #[test]
    fn intent_phase_handles_crossing_choice_pending() {
        let mut state = state_with_rng(19);
        state.intent.pending = DayIntent::CrossingChoicePending;
        let outcome = IntentPhase::new(&mut state).run();
        assert!(outcome.is_some());
        assert_eq!(state.intent.pending, DayIntent::CrossingChoicePending);
    }

    #[test]
    fn pending_phase_resolves_store_and_crossings() {
        let mut store_state = state_with_rng(8);
        store_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        store_state.ot_deluxe.inventory.cash_cents = 10_000;
        store_state.ot_deluxe.store.pending_node = Some(0);
        store_state.ot_deluxe.store.pending_purchase = Some(vec![OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::FoodLb,
            quantity: 1,
        }]);
        let store_outcome = PendingPhase::new(&mut store_state).resolve_pending_store();
        assert!(store_outcome.is_some());
        assert!(store_state.ot_deluxe.store.pending_node.is_none());

        let mut dystrail_state = state_with_rng(9);
        dystrail_state.pending_crossing = Some(PendingCrossing {
            kind: CrossingKind::BridgeOut,
            computed_miles_today: 12.0,
        });
        dystrail_state.pending_crossing_choice = Some(CrossingChoice::Detour);
        let dystrail_outcome = PendingPhase::new(&mut dystrail_state).resolve_pending_crossing();
        assert!(dystrail_outcome.is_some());
        assert!(dystrail_state.pending_crossing_choice.is_none());

        let mut otdeluxe_state = state_with_rng(10);
        otdeluxe_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        otdeluxe_state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        otdeluxe_state.ot_deluxe.crossing.choice_pending = true;
        otdeluxe_state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        otdeluxe_state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 120.0,
            depth_ft: 2.0,
            swiftness: 0.2,
            bed: OtDeluxeRiverBed::Muddy,
        });
        otdeluxe_state.ot_deluxe.crossing.computed_miles_today = 12.0;
        otdeluxe_state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Ford);
        let otdeluxe_outcome = PendingPhase::new(&mut otdeluxe_state).resolve_pending_crossing();
        assert!(otdeluxe_outcome.is_some());
    }

    fn otdeluxe_state_for_travel(seed: u64) -> GameState {
        let mut state = state_with_rng(seed);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.mode = GameMode::Classic;
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        state.ot_deluxe.inventory.food_lbs = 200;
        state.journey_breakdown.base = 0.0;
        state.start_of_day();
        state
    }

    #[test]
    fn travel_phase_handles_encounter_flow() {
        let mut state = state_with_rng(11);
        state.mode = GameMode::Classic;
        state.encounter_chance_today = 1.0;
        state.data = Some(sample_encounter_data());
        state.start_of_day();
        let outcome = TravelPhase::new(&mut state).run(&EndgameTravelCfg::default_config());
        assert!(
            outcome.1 == LOG_TRAVEL_BLOCKED
                || outcome.1 == LOG_TRAVELED
                || outcome.1 == "log.encounter"
        );
        assert!(state.current_encounter.is_some());
    }

    #[test]
    fn travel_phase_otdeluxe_reaches_random_event_tick() {
        let endgame_cfg = EndgameTravelCfg::default_config();
        let mut found = false;
        for seed in 1_u64..5_000 {
            let mut state = otdeluxe_state_for_travel(seed);
            let outcome = TravelPhase::new(&mut state).run(&endgame_cfg);
            let navigation_blocked = state.ot_deluxe.travel.delay_days_remaining > 0
                || state.ot_deluxe.travel.blocked_days_remaining > 0;
            if !navigation_blocked && outcome.1 == LOG_TRAVELED {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn travel_phase_otdeluxe_navigation_event_blocks() {
        let endgame_cfg = EndgameTravelCfg::default_config();
        let mut state = otdeluxe_state_for_travel(2024);
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 1,
            wrong_weight: 0,
            impassable_weight: 0,
            snowbound_weight: 0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let outcome =
            TravelPhase::new(&mut state).run_with_navigation_policy(&endgame_cfg, Some(&policy));
        assert_eq!(outcome.1, LOG_TRAVEL_BLOCKED);
        let blocked = state.ot_deluxe.travel.blocked_days_remaining
            + state.ot_deluxe.travel.delay_days_remaining;
        assert!(blocked > 0);
    }

    #[test]
    fn travel_phase_vehicle_failure_short_circuits() {
        let endgame_cfg = EndgameTravelCfg::default_config();
        let mut state = state_with_rng(42);
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.vehicle.health = 0.0;
        state.vehicle_breakdowns = 999;
        state.budget_cents = 0;
        state.inventory.spares = Spares::default();
        state.start_of_day();
        let outcome = TravelPhase::new(&mut state).run(&endgame_cfg);
        assert_eq!(outcome.1, LOG_VEHICLE_FAILURE);
    }

    #[test]
    fn weather_phase_emits_event_for_otdeluxe() {
        let mut state = state_with_rng(1);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.start_of_day();
        WeatherPhase::new(&mut state).run();
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::WeatherResolved)
        );
    }

    #[test]
    fn exec_order_phase_runs_with_rng_guard() {
        let mut state = state_with_rng(2);
        ExecOrderPhase::new(&mut state).run();
        assert!(state.exec_effects.travel_multiplier >= 0.0);
    }

    #[test]
    fn supplies_phase_emits_event_for_dystrail() {
        let cfg = JourneyCfg::default();
        let mut state = state_with_rng(3);
        state.stats.supplies = 5;
        SuppliesPhase::new(&mut state).run(&cfg.daily);
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::DailyConsumptionApplied)
        );
    }

    #[test]
    fn supplies_phase_emits_event_for_otdeluxe() {
        let cfg = JourneyCfg::default();
        let mut state = state_with_rng(4);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.inventory.food_lbs = 100;
        SuppliesPhase::new(&mut state).run(&cfg.daily);
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::DailyConsumptionApplied)
        );
    }

    #[test]
    fn health_phase_emits_event_for_dystrail() {
        let cfg = JourneyCfg::default();
        let mut state = state_with_rng(5);
        HealthPhase::new(&mut state).run(&cfg.daily, &cfg.strain);
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::HealthTickApplied)
        );
    }

    #[test]
    fn health_phase_emits_event_for_otdeluxe() {
        let cfg = JourneyCfg::default();
        let mut state = state_with_rng(6);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        HealthPhase::new(&mut state).run(&cfg.daily, &cfg.strain);
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::HealthTickApplied)
        );
    }

    #[test]
    fn intent_phase_rest_branch_records_day() {
        let mut state = state_with_rng(7);
        state.intent.pending = DayIntent::Rest;
        state.intent.rest_days_remaining = 2;
        let outcome = IntentPhase::new(&mut state).run();
        assert!(outcome.is_some());
        let record = state.day_records.last().expect("day record");
        assert!(record.tags.contains(&DayTag::new("intent_rest")));
    }

    #[test]
    fn intent_phase_trade_branch_emits_event() {
        let mut state = state_with_rng(8);
        state.intent.pending = DayIntent::Trade;
        state.start_of_day();
        let outcome = IntentPhase::new(&mut state).run();
        assert!(outcome.is_some());
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::TradeResolved)
        );
    }

    #[test]
    fn intent_phase_hunt_branch_emits_event() {
        let mut state = state_with_rng(9);
        state.intent.pending = DayIntent::Hunt;
        state.ot_deluxe.inventory.bullets = 40;
        state.start_of_day();
        let outcome = IntentPhase::new(&mut state).run();
        assert!(outcome.is_some());
        assert!(
            state
                .events_today
                .iter()
                .any(|event| event.kind == EventKind::HuntResolved)
        );
    }

    #[test]
    fn pending_phase_resolves_store_purchase() {
        let mut state = state_with_rng(10);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.inventory.cash_cents = 10_000;
        state.ot_deluxe.store.pending_node = Some(0);
        state.ot_deluxe.store.pending_purchase = Some(vec![OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::FoodLb,
            quantity: 1,
        }]);
        let outcome = PendingPhase::new(&mut state).resolve_pending_store();
        assert!(outcome.is_some());
        assert!(state.ot_deluxe.store.pending_node.is_none());
    }

    #[test]
    fn pending_phase_resolves_otdeluxe_crossing_choice() {
        let mut state = state_with_rng(11);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        state.ot_deluxe.crossing.choice_pending = true;
        state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Ford);
        state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        state.ot_deluxe.crossing.computed_miles_today = 7.0;
        state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 120.0,
            depth_ft: 2.0,
            swiftness: 0.2,
            bed: OtDeluxeRiverBed::Muddy,
        });
        let outcome = PendingPhase::new(&mut state).resolve_pending_crossing();
        assert!(outcome.is_some());
    }

    #[test]
    fn pending_phase_resolves_dystrail_crossing_choice() {
        let mut state = state_with_rng(12);
        state.pending_crossing = Some(PendingCrossing {
            kind: CrossingKind::BridgeOut,
            computed_miles_today: 12.0,
        });
        state.pending_crossing_choice = Some(CrossingChoice::Detour);
        let outcome = PendingPhase::new(&mut state).resolve_pending_crossing();
        assert!(outcome.is_some());
        assert!(state.pending_crossing_choice.is_none());
    }

    #[test]
    fn travel_phase_encounter_branch_returns_log() {
        let mut state = state_with_rng(13);
        state.encounter_chance_today = 1.0;
        state.data = Some(sample_encounter_data());
        state.journey_breakdown.base = 0.0;
        state.start_of_day();
        let outcome = TravelPhase::new(&mut state).run(&EndgameTravelCfg::default_config());
        assert_eq!(outcome.1, "log.encounter");
    }

    #[test]
    fn travel_phase_dystrail_full_run_travels() {
        let mut state = state_with_rng(14);
        state.encounter_chance_today = 0.0;
        state.journey_breakdown.base = 0.0;
        state.start_of_day();
        let outcome = TravelPhase::new(&mut state).run(&EndgameTravelCfg::default_config());
        assert_eq!(outcome.1, LOG_TRAVELED);
    }

    #[test]
    fn travel_phase_otdeluxe_navigation_policy_allows_travel() {
        let endgame_cfg = EndgameTravelCfg::default_config();
        let mut state = otdeluxe_state_for_travel(15);
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 0.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let outcome =
            TravelPhase::new(&mut state).run_with_navigation_policy(&endgame_cfg, Some(&policy));
        assert_eq!(outcome.1, LOG_TRAVELED);
    }
}
