//! Kernel orchestration for a single simulated day.

use crate::day_accounting;
use crate::endgame::EndgameTravelCfg;
use crate::journey::phase::{
    BossPhase, ExecOrderPhase, HealthPhase, IntentPhase, PacingPhase, PendingPhase, SuppliesPhase,
    TravelPhase, WaitPhase, WeatherPhase,
};
use crate::journey::{
    DayEffects, DayInputs, DayOutcome, Event, EventId, JourneyCfg, MechanicalPolicyId, StatsDelta,
    TravelDayKind,
};
use crate::state::{DayIntent, DietId, GameMode, GameState, PaceId, Region, Season};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
struct StatsSnapshot {
    supplies: i32,
    hp: i32,
    sanity: i32,
    credibility: i32,
    morale: i32,
    allies: i32,
    pants: i32,
}

#[derive(Debug, Clone, Copy)]
struct DaySnapshot {
    day: u32,
    intent: DayIntent,
    pace: PaceId,
    diet: DietId,
    region: Region,
    season: Season,
    mode: GameMode,
    mechanical_policy: MechanicalPolicyId,
    stats: StatsSnapshot,
    budget: i32,
    budget_cents: i64,
    miles_traveled: f32,
    miles_traveled_actual: f32,
}

impl DaySnapshot {
    const fn capture(state: &GameState) -> Self {
        Self {
            day: state.day,
            intent: state.intent.pending,
            pace: state.pace,
            diet: state.diet,
            region: state.region,
            season: state.season,
            mode: state.mode,
            mechanical_policy: state.mechanical_policy,
            stats: StatsSnapshot {
                supplies: state.stats.supplies,
                hp: state.stats.hp,
                sanity: state.stats.sanity,
                credibility: state.stats.credibility,
                morale: state.stats.morale,
                allies: state.stats.allies,
                pants: state.stats.pants,
            },
            budget: state.budget,
            budget_cents: state.budget_cents,
            miles_traveled: state.miles_traveled,
            miles_traveled_actual: state.miles_traveled_actual,
        }
    }
}

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
            {
                let mut phase = WeatherPhase::new(state);
                phase.run();
            }
            {
                let mut phase = ExecOrderPhase::new(state);
                phase.run();
            }
            {
                let mut phase = SuppliesPhase::new(state);
                phase.run(&self.cfg.daily);
            }
            {
                let mut phase = HealthPhase::new(state);
                phase.run(&self.cfg.daily, &self.cfg.strain);
            }
        }
    }

    pub(crate) fn tick_day(&self, state: &mut GameState) -> DayOutcome {
        self.tick_day_with_hook(state, |_| {})
    }

    pub(crate) fn tick_day_with_hook<F>(&self, state: &mut GameState, hook: F) -> DayOutcome
    where
        F: FnOnce(&mut GameState),
    {
        let snapshot = DaySnapshot::capture(state);
        if let Some((ended, log_key, breakdown_started)) =
            PendingPhase::new(state).resolve_pending_route_prompt()
        {
            return Self::build_outcome(state, snapshot, ended, log_key, breakdown_started);
        }
        if let Some((ended, log_key, breakdown_started)) =
            PendingPhase::new(state).resolve_pending_crossing()
        {
            return Self::build_outcome(state, snapshot, ended, log_key, breakdown_started);
        }
        if let Some((ended, log_key, breakdown_started)) =
            PendingPhase::new(state).resolve_pending_store()
        {
            return Self::build_outcome(state, snapshot, ended, log_key, breakdown_started);
        }
        self.apply_daily_physics(state);
        {
            let mut phase = PacingPhase::new(state);
            phase.apply();
        }
        hook(state);

        if let Some((ended, log_key, breakdown_started)) = BossPhase::new(state).run() {
            return Self::build_outcome(state, snapshot, ended, log_key, breakdown_started);
        }

        if let Some((ended, log_key, breakdown_started)) = WaitPhase::new(state).run() {
            return Self::build_outcome(state, snapshot, ended, log_key, breakdown_started);
        }

        if let Some((ended, log_key, breakdown_started)) = IntentPhase::new(state).run() {
            return Self::build_outcome(state, snapshot, ended, log_key, breakdown_started);
        }

        let (ended, log_key, breakdown_started) = TravelPhase::new(state).run(self.endgame_cfg);
        Self::build_outcome(state, snapshot, ended, log_key, breakdown_started)
    }

    fn build_outcome(
        state: &mut GameState,
        snapshot: DaySnapshot,
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
        let mut seen_keys = HashSet::new();
        for event in &events {
            if let Some(key) = event.ui_key.as_deref() {
                seen_keys.insert(key.to_string());
            }
        }
        if !seen_keys.contains(&resolved_log_key) {
            events.push(Event::legacy_log_key(
                EventId::new(event_day, seq),
                event_day,
                resolved_log_key.clone(),
            ));
            seen_keys.insert(resolved_log_key.clone());
            seq = seq.saturating_add(1);
        }

        let log_start = usize::try_from(state.day_state.lifecycle.log_cursor).unwrap_or(0);
        let log_end = state.logs.len();
        let log_start = log_start.min(log_end);
        for log in &state.logs[log_start..log_end] {
            if log == &resolved_log_key || seen_keys.contains(log) {
                continue;
            }
            events.push(Event::legacy_log_key(
                EventId::new(event_day, seq),
                event_day,
                log.clone(),
            ));
            seen_keys.insert(log.clone());
            seq = seq.saturating_add(1);
        }
        state.day_state.lifecycle.log_cursor = u32::try_from(log_end).unwrap_or(u32::MAX);
        state.day_state.lifecycle.event_seq = seq;
        let decision_traces = std::mem::take(&mut state.decision_traces_today);
        let inputs = DayInputs {
            day: snapshot.day,
            intent: snapshot.intent,
            pace: snapshot.pace,
            diet: snapshot.diet,
            region: snapshot.region,
            season: snapshot.season,
            mode: snapshot.mode,
            mechanical_policy: snapshot.mechanical_policy,
            weather: state.weather_state.today,
        };
        let stats_delta = StatsDelta {
            supplies: state.stats.supplies - snapshot.stats.supplies,
            hp: state.stats.hp - snapshot.stats.hp,
            sanity: state.stats.sanity - snapshot.stats.sanity,
            credibility: state.stats.credibility - snapshot.stats.credibility,
            morale: state.stats.morale - snapshot.stats.morale,
            allies: state.stats.allies - snapshot.stats.allies,
            pants: state.stats.pants - snapshot.stats.pants,
        };
        let effects = DayEffects {
            stats: stats_delta,
            budget_delta: state.budget - snapshot.budget,
            budget_cents_delta: state.budget_cents - snapshot.budget_cents,
            miles_traveled_delta: state.miles_traveled - snapshot.miles_traveled,
            miles_traveled_actual_delta: state.miles_traveled_actual
                - snapshot.miles_traveled_actual,
        };
        DayOutcome {
            ended: resolved_ended,
            log_key: resolved_log_key,
            breakdown_started,
            day_consumed,
            inputs,
            effects,
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
    fn tick_day_assigns_sequential_event_ids() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            stats: Stats {
                supplies: 20,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(45)));
        state.push_log("log.test.pre");

        let outcome = kernel.tick_day_with_hook(&mut state, |state| {
            state.push_log("log.test.hook");
        });

        let mut expected = 0u16;
        for event in &outcome.events {
            assert_eq!(event.id.seq, expected);
            assert_eq!(event.id.day, event.day);
            expected = expected.saturating_add(1);
        }
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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_crossing();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("expected outcome");

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_crossing();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("expected outcome");

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_crossing();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("expected outcome");

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_crossing();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("expected outcome");

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

        let outcome = PendingPhase::new(&mut state).resolve_pending_route_prompt();

        assert!(outcome.is_none());
        assert!(state.pending_route_choice.is_none());
    }

    #[test]
    fn pending_route_prompt_clears_choice_for_non_otdeluxe() {
        let mut state = GameState {
            pending_route_choice: Some(OtDeluxeRouteDecision::SubletteCutoff),
            ..GameState::default()
        };

        let outcome = PendingPhase::new(&mut state).resolve_pending_route_prompt();

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

        let outcome = WaitPhase::new(&mut state).run();

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
        state.wait.drying_days_remaining = 1;

        let _ = WaitPhase::new(&mut state).run();

        let record = state.day_records.last().expect("expected day record");
        assert!(record.tags.iter().any(|tag| tag.0 == "wait_drying"));
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

        let outcome = PendingPhase::new(&mut state).resolve_pending_store();

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

        let snapshot = DaySnapshot::capture(&state);
        let outcome = DailyTickKernel::build_outcome(
            &mut state,
            snapshot,
            false,
            String::from("log.fallback"),
            false,
        );

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

        let snapshot = DaySnapshot::capture(&state);
        let outcome = DailyTickKernel::build_outcome(
            &mut state,
            snapshot,
            false,
            String::from(LOG_TRAVELED),
            false,
        );

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_store();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("outcome");

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_route_prompt();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("outcome");

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_crossing();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("outcome");

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

        let snapshot = DaySnapshot::capture(&state);
        let pending = PendingPhase::new(&mut state).resolve_pending_crossing();
        let outcome = pending
            .map(|(ended, log_key, breakdown_started)| {
                DailyTickKernel::build_outcome(
                    &mut state,
                    snapshot,
                    ended,
                    log_key,
                    breakdown_started,
                )
            })
            .expect("outcome");

        assert_eq!(outcome.log_key, LOG_TRAVEL_BLOCKED);
        assert!(!outcome.day_consumed);
    }
}
