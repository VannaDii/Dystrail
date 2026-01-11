//! Kernel orchestration for a single simulated day.

use std::sync::OnceLock;

use crate::constants::{LOG_TRAVEL_BLOCKED, LOG_TRAVELED};
use crate::day_accounting;
use crate::endgame::{self, EndgameTravelCfg};
use crate::journey::daily::{apply_daily_health, apply_daily_supplies_sanity};
use crate::journey::{
    DayOutcome, Event, EventId, JourneyCfg, MechanicalPolicyId, RngPhase, TravelDayKind,
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
        self.apply_daily_physics(state);
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            state.apply_otdeluxe_pace_and_rations();
        } else {
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
        let events = vec![Event::legacy_log_key(
            EventId::new(event_day, 0),
            event_day,
            resolved_log_key.clone(),
        )];
        let decision_traces = std::mem::take(&mut state.decision_traces_today);
        DayOutcome {
            ended: resolved_ended,
            log_key: resolved_log_key,
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

        if let Some(result) = {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::EncounterTick));
            state.process_encounter_flow(rng_bundle.as_ref(), breakdown_started)
        } {
            return result;
        }
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::TravelTick));
            if state.apply_otdeluxe_navigation_event() {
                return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
            }
        }

        let computed_miles_today = if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            state.distance_today
        } else {
            state.distance_today.max(state.distance_today_raw)
        };
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

        state.end_of_day();
        (false, String::from(LOG_TRAVELED), breakdown_started)
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
                if let Some(bundle) = rng_bundle.as_ref() {
                    let _guard = bundle.phase_guard_for(RngPhase::TradeTick);
                    let mut rng = bundle.trade();
                    let _ = trade::resolve_trade_with_rng(state, &mut *rng);
                } else {
                    let _ = trade::resolve_trade(state);
                }
                Self::record_intent_day(state, "intent_trade");
                Some((false, String::from(LOG_TRAVELED), false))
            }
            DayIntent::Hunt => {
                state.intent.pending = DayIntent::Continue;
                state.intent.rest_days_remaining = 0;
                let rng_bundle = state.rng_bundle.clone();
                if let Some(bundle) = rng_bundle.as_ref() {
                    let _guard = bundle.phase_guard_for(RngPhase::HuntTick);
                    let mut rng = bundle.hunt();
                    let _ = hunt::resolve_hunt_with_rng(state, &mut *rng);
                } else {
                    let _ = hunt::resolve_hunt(state);
                }
                Self::record_intent_day(state, "intent_hunt");
                Some((false, String::from(LOG_TRAVELED), false))
            }
        }
    }

    fn record_intent_day(state: &mut GameState, reason_tag: &str) {
        Self::record_gate_day(state, reason_tag);
    }

    fn record_gate_day(state: &mut GameState, reason_tag: &str) {
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
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            state.apply_otdeluxe_consumption();
            return;
        }
        let _ = apply_daily_supplies_sanity(&self.cfg.daily, state);
    }

    fn run_health_tick(&self, state: &mut GameState) {
        if state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            Self::run_otdeluxe_health_tick(state);
            return;
        }
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
        state.update_general_strain(&self.cfg.strain);
        {
            let _guard = rng_bundle
                .as_ref()
                .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
            state.tick_ally_attrition();
        }
        state.stats.clamp();
    }

    fn run_otdeluxe_health_tick(state: &mut GameState) {
        let rng_bundle = state.rng_bundle.clone();
        let _guard = rng_bundle
            .as_ref()
            .map(|bundle| bundle.phase_guard_for(RngPhase::HealthTick));
        state.apply_otdeluxe_health_update();
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
    use crate::constants::{LOG_BOSS_AWAIT, LOG_TRAVEL_BLOCKED, LOG_TRAVELED};
    use crate::exec_orders::ExecOrder;
    use crate::journey::{
        DailyChannelConfig, DailyTickConfig, HealthTickConfig, JourneyCfg, MechanicalPolicyId,
        RngBundle,
    };
    use crate::numbers::round_f32_to_i32;
    use crate::otdeluxe_state::OtDeluxeWagonState;
    use crate::state::{GameState, Region, Stats};
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
    fn wait_gate_consumes_non_travel_day_and_decrements_counter() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState::default();
        state.wait.ferry_wait_days_remaining = 1;

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(state.wait.ferry_wait_days_remaining, 0);
        assert_eq!(state.day, 2);
        let record = outcome.record.expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "wait_ferry"));
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
        let record = outcome.record.expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Blocked
        ));
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
        let record = outcome.record.expect("expected day record");
        assert!(matches!(
            record.kind,
            TravelDayKind::Travel | TravelDayKind::Partial
        ));
        assert!(state.distance_today > 0.0);
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
        assert!(matches!(state.intent.pending, DayIntent::Rest));
        assert_eq!(state.intent.rest_days_remaining, 1);
        let first_record = first.record.expect("expected first record");
        assert!(matches!(first_record.kind, TravelDayKind::NonTravel));
        assert!(first_record.tags.iter().any(|tag| tag.0 == "intent_rest"));

        let second = kernel.tick_day(&mut state);
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
        let record = outcome.record.expect("expected trade record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "intent_trade"));
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
        let record = outcome.record.expect("expected hunt record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "intent_hunt"));
    }
}
