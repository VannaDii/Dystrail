use crate::Stats;
use crate::exec_orders::ExecOrder;
use crate::journey::{DailyChannelConfig, DailyTickConfig, HealthTickConfig};
use crate::state::{DietId, GameState, PaceId};
use crate::weather::Weather;

/// Resulting stat deltas applied during the daily tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyTickOutcome {
    pub supplies_delta: i32,
    pub sanity_delta: i32,
    pub health_delta: i32,
}

impl DailyTickOutcome {
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            supplies_delta: 0,
            sanity_delta: 0,
            health_delta: 0,
        }
    }
}

/// Apply policy-driven daily effects to the provided game state.
pub fn apply_daily_effect(cfg: &DailyTickConfig, state: &mut GameState) -> DailyTickOutcome {
    let weather = state.weather_state.today;
    let exec_key = state.current_order.map(ExecOrder::key);
    let pace = state.pace;
    let diet = state.diet;

    let supplies_loss = channel_value(&cfg.supplies, pace, diet, weather, exec_key);
    let sanity_loss = channel_value(&cfg.sanity, pace, diet, weather, exec_key);
    let health_delta = health_change(&cfg.health, state, weather, exec_key);

    let supplies_delta = -rounded_i32(supplies_loss);
    let sanity_delta = -rounded_i32(sanity_loss);
    apply_supplies_delta(state, supplies_delta);
    apply_sanity_delta(state, sanity_delta);
    apply_health_delta(state, health_delta);

    state.stats.clamp();

    DailyTickOutcome {
        supplies_delta,
        sanity_delta,
        health_delta,
    }
}

fn channel_value(
    cfg: &DailyChannelConfig,
    pace: PaceId,
    diet: DietId,
    weather: Weather,
    exec_key: Option<&str>,
) -> f32 {
    if cfg.base <= f32::EPSILON {
        return 0.0;
    }
    let mut value = cfg.base;
    value *= cfg.pace.get(&pace).copied().unwrap_or(1.0);
    if !cfg.diet.is_empty() {
        value *= cfg.diet.get(&diet).copied().unwrap_or(1.0);
    }
    value *= cfg.weather.get(&weather).copied().unwrap_or(1.0);
    if let Some(exec) = exec_key
        && let Some(mult) = cfg.exec.get(exec)
    {
        value *= *mult;
    }
    value
}

fn health_change(
    cfg: &HealthTickConfig,
    state: &GameState,
    weather: Weather,
    exec_key: Option<&str>,
) -> i32 {
    let mut delta = 0.0_f32;
    if cfg.decay > 0.0 {
        let mut decay = cfg.decay;
        decay *= cfg.weather.get(&weather).copied().unwrap_or(1.0);
        if let Some(exec) = exec_key
            && let Some(mult) = cfg.exec.get(exec)
        {
            decay *= *mult;
        }
        delta -= decay;
    }

    if cfg.rest_heal > 0.0 && state.rest_requested {
        delta += cfg.rest_heal;
    }

    rounded_i32(delta)
}

fn apply_supplies_delta(state: &mut GameState, delta: i32) {
    if delta == 0 {
        return;
    }
    let supplies = state.stats.supplies + delta;
    state.stats.supplies = supplies.max(0);
}

fn apply_sanity_delta(state: &mut GameState, delta: i32) {
    if delta == 0 {
        return;
    }
    let max_sanity = Stats::default().sanity;
    let sanity = state.stats.sanity + delta;
    state.stats.sanity = sanity.clamp(0, max_sanity);
}

fn apply_health_delta(state: &mut GameState, delta: i32) {
    if delta == 0 {
        return;
    }
    let max_hp = Stats::default().hp;
    let hp = state.stats.hp + delta;
    state.stats.hp = hp.clamp(0, max_hp);
}

#[allow(clippy::missing_const_for_fn)]
fn rounded_i32(value: f32) -> i32 {
    #[allow(clippy::cast_possible_truncation)]
    {
        value.round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec_orders::ExecOrder;
    use crate::journey::DailyTickConfig;
    use crate::state::{DietId, GameMode};
    use std::collections::HashMap;

    fn populated_state() -> GameState {
        GameState {
            mode: GameMode::Classic,
            stats: Stats {
                supplies: 10,
                sanity: 12,
                hp: 8,
                ..Stats::default()
            },
            ..GameState::default()
        }
    }

    #[test]
    fn applies_pace_diet_weather_and_exec_multipliers() {
        let mut cfg = DailyTickConfig {
            supplies: DailyChannelConfig::new(2.0),
            sanity: DailyChannelConfig::new(1.0),
            ..DailyTickConfig::default()
        };
        cfg.supplies.pace.insert(PaceId::Blitz, 1.5);
        cfg.supplies.diet.insert(DietId::Doom, 1.2);
        cfg.supplies.weather.insert(Weather::Storm, 1.1);
        cfg.supplies
            .exec
            .insert(String::from(ExecOrder::TravelBanLite.key()), 2.0);

        cfg.sanity.pace.insert(PaceId::Blitz, 1.0);
        cfg.sanity.diet.insert(DietId::Doom, 1.0);
        cfg.sanity.weather.insert(Weather::Storm, 1.0);
        cfg.sanity
            .exec
            .insert(String::from(ExecOrder::TravelBanLite.key()), 1.0);

        let mut state = populated_state();
        state.pace = PaceId::Blitz;
        state.diet = DietId::Doom;
        state.weather_state.today = Weather::Storm;
        state.current_order = Some(ExecOrder::TravelBanLite);

        let exec_key = state.current_order.map(ExecOrder::key);
        let expected_supplies = channel_value(
            &cfg.supplies,
            state.pace,
            state.diet,
            state.weather_state.today,
            exec_key,
        );
        let expected_sanity = channel_value(
            &cfg.sanity,
            state.pace,
            state.diet,
            state.weather_state.today,
            exec_key,
        );

        let initial_supplies = state.stats.supplies;
        let initial_sanity = state.stats.sanity;

        let outcome = apply_daily_effect(&cfg, &mut state);
        assert_eq!(outcome.supplies_delta, -rounded_i32(expected_supplies));
        assert_eq!(outcome.sanity_delta, -rounded_i32(expected_sanity));
        assert_eq!(outcome.health_delta, 0);
        assert_eq!(
            state.stats.supplies,
            (initial_supplies + outcome.supplies_delta).max(0)
        );
        assert_eq!(
            state.stats.sanity,
            (initial_sanity + outcome.sanity_delta).clamp(0, Stats::default().sanity)
        );
    }

    #[test]
    fn health_decay_and_rest_heal_interplay() {
        let mut cfg = DailyTickConfig {
            health: HealthTickConfig {
                decay: 1.5,
                rest_heal: 2.5,
                ..HealthTickConfig::default()
            },
            ..DailyTickConfig::default()
        };
        cfg.health.weather.insert(Weather::HeatWave, 2.0);
        cfg.health
            .exec
            .insert(String::from(ExecOrder::WarDeptReorg.key()), 0.5);

        let mut state = populated_state();
        state.weather_state.today = Weather::HeatWave;
        state.current_order = Some(ExecOrder::WarDeptReorg);
        state.rest_requested = true;

        let exec_key = state.current_order.map(ExecOrder::key);
        let expected_delta =
            health_change(&cfg.health, &state, state.weather_state.today, exec_key);
        let initial_hp = state.stats.hp;
        let outcome = apply_daily_effect(&cfg, &mut state);
        assert_eq!(outcome.health_delta, expected_delta);
        assert_eq!(
            state.stats.hp,
            (initial_hp + outcome.health_delta).clamp(0, Stats::default().hp)
        );
    }

    #[test]
    fn sanitize_guards_invalid_entries() {
        let mut cfg = DailyTickConfig {
            supplies: DailyChannelConfig {
                base: -5.0,
                pace: HashMap::from([(PaceId::Steady, -0.5)]),
                diet: HashMap::from([(DietId::Mixed, -1.0)]),
                weather: HashMap::from([(Weather::Clear, -3.0)]),
                exec: HashMap::from([(String::from("bad_exec"), -2.0)]),
            },
            sanity: DailyChannelConfig::default(),
            health: HealthTickConfig {
                decay: -1.0,
                rest_heal: -10.0,
                weather: HashMap::from([(Weather::Smoke, -2.0)]),
                exec: HashMap::from([(String::from("neg"), f32::NAN)]),
            },
        };

        cfg.sanitize();

        approx_eq(cfg.supplies.base, 0.0);
        assert_eq!(cfg.supplies.pace.get(&PaceId::Steady), Some(&1.0));
        assert_eq!(cfg.supplies.diet.get(&DietId::Mixed), Some(&1.0));
        assert_eq!(cfg.supplies.weather.get(&Weather::Clear), Some(&1.0));
        assert_eq!(cfg.supplies.exec.get("bad_exec"), Some(&1.0));
        approx_eq(cfg.health.decay, 0.0);
        approx_eq(cfg.health.rest_heal, 0.0);
        assert_eq!(cfg.health.weather.get(&Weather::Smoke), Some(&0.0));
    }

    fn approx_eq(left: f32, right: f32) {
        const EPS: f32 = 1e-6;
        assert!((left - right).abs() <= EPS, "{left} != {right}");
    }
}
