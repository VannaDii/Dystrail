use crate::Stats;
use crate::exec_orders::ExecOrder;
use crate::journey::{DailyChannelConfig, DailyTickConfig, HealthTickConfig};
use crate::state::{DamageCause, DietId, GameState, PaceId};
use crate::weather::Weather;
use serde::{Deserialize, Serialize};

/// Unapplied millionths of a stat point; integer storage survives JSON round trips exactly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyRemainders {
    pub supplies: i32,
    pub sanity: i32,
    pub health: i32,
}

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
    let health = health_change(&cfg.health, state, weather, exec_key);
    let remainder = &mut state.daily_remainders;
    let supplies_delta = carried_delta(-supplies_loss, &mut remainder.supplies);
    let sanity_delta = carried_delta(-sanity_loss, &mut remainder.sanity);
    let health_delta = carried_delta(health, &mut remainder.health);
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
) -> f32 {
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

    if cfg.rest_heal > 0.0 && state.day_state.rest.rest_requested {
        delta += cfg.rest_heal;
    }

    delta
}

fn carried_delta(value: f32, remainder: &mut i32) -> i32 {
    const SCALE: i64 = 1_000_000;
    let scaled = crate::numbers::round_f64_to_i32(f64::from(value) * 1_000_000.0);
    let total = i64::from(scaled) + i64::from(*remainder);
    let delta = total / SCALE
        + if (total % SCALE).abs() >= SCALE / 2 {
            total.signum()
        } else {
            0
        };
    *remainder = i32::try_from(total - delta * SCALE).unwrap_or(0);
    i32::try_from(delta).unwrap_or(0)
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
    let previous_hp = state.stats.hp;
    let hp = previous_hp + delta;
    state.stats.hp = hp.clamp(0, max_hp);
    if delta < 0 && state.stats.hp < previous_hp {
        // Ordinary daily attrition uses the burnout ending.
        state.mark_damage(DamageCause::Breakdown);
    }
}

#[cfg(test)]
fn rounded_i32(value: f32) -> i32 {
    crate::numbers::round_f32_to_i32(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endgame::EndgameTravelCfg;
    use crate::exec_orders::ExecOrder;
    use crate::journey::DailyTickConfig;
    use crate::repairs::RepairChoice;
    use crate::state::{
        CollapseCause, DietId, Ending, ExposureKind, GameMode, VehicleFailureCause,
    };
    use crate::vehicle::{Breakdown, Part};
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
        state.day_state.rest.rest_requested = true;

        let exec_key = state.current_order.map(ExecOrder::key);
        let expected_delta =
            health_change(&cfg.health, &state, state.weather_state.today, exec_key);
        let initial_hp = state.stats.hp;
        let outcome = apply_daily_effect(&cfg, &mut state);
        assert_eq!(outcome.health_delta, rounded_i32(expected_delta));
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

    #[test]
    fn fractional_daily_costs_accumulate_and_survive_a_saved_journey() {
        let cfg = DailyTickConfig {
            supplies: DailyChannelConfig::new(0.24),
            sanity: DailyChannelConfig::new(0.04),
            health: HealthTickConfig {
                decay: 0.12,
                ..HealthTickConfig::default()
            },
        };
        let mut state = GameState {
            last_damage: Some(DamageCause::Vehicle),
            ..GameState::default()
        };
        state.stats.supplies = 20;
        for day in 0..50 {
            apply_daily_effect(&cfg, &mut state);
            if day < 4 {
                assert_eq!(state.stats.hp, 10);
                assert_eq!(state.last_damage, Some(DamageCause::Vehicle));
            } else {
                assert_eq!(state.last_damage, Some(DamageCause::Breakdown));
            }
            if day == 22 {
                state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
                assert_eq!(state.last_damage, Some(DamageCause::Breakdown));
            }
        }
        assert_eq!(state.stats.supplies, 8);
        assert_eq!(state.stats.sanity, 8);
        assert_eq!(state.stats.hp, 4);
    }

    #[test]
    fn daily_health_collapse_does_not_blame_an_explicitly_repaired_vehicle() {
        let mut state = GameState {
            last_damage: Some(DamageCause::Vehicle),
            breakdown: Some(Breakdown {
                part: Part::Tire,
                day_started: 1,
            }),
            ..GameState::default()
        };
        state.stats.hp = 1;
        state.vehicle.health = 84.0;
        state.inventory.spares.tire = 1;
        assert!(state.choose_repair(RepairChoice::Onboard));
        assert!(state.breakdown.is_none());
        assert_eq!(state.last_damage, Some(DamageCause::Vehicle));
        let repaired_health = state.vehicle.health;
        let cfg = DailyTickConfig {
            health: HealthTickConfig {
                decay: 1.0,
                ..HealthTickConfig::default()
            },
            ..DailyTickConfig::default()
        };

        let outcome = apply_daily_effect(&cfg, &mut state);
        assert_eq!(outcome.health_delta, -1);
        assert_eq!(state.stats.hp, 0);
        assert_eq!(state.last_damage, Some(DamageCause::Breakdown));
        state.day_state.lifecycle.day_initialized = true;
        let (ended, _, _) = state.travel_next_leg(&EndgameTravelCfg::default());
        assert!(ended);
        assert_eq!(
            state.ending,
            Some(Ending::Collapse {
                cause: CollapseCause::Breakdown,
            })
        );
        approx_eq(state.vehicle.health, repaired_health);
    }

    #[test]
    fn zero_ticks_and_rest_healing_keep_the_cause_until_new_health_loss() {
        let mut state = populated_state();
        state.last_damage = Some(DamageCause::ExposureCold);
        let outcome = apply_daily_effect(&DailyTickConfig::default(), &mut state);
        assert_eq!(outcome.health_delta, 0);
        assert_eq!(state.stats.hp, 8);
        assert_eq!(state.last_damage, Some(DamageCause::ExposureCold));

        let cfg = DailyTickConfig {
            health: HealthTickConfig {
                decay: 1.0,
                rest_heal: 2.0,
                ..HealthTickConfig::default()
            },
            ..DailyTickConfig::default()
        };
        state.day_state.rest.rest_requested = true;
        for expected_hp in [9, 10, 10] {
            let outcome = apply_daily_effect(&cfg, &mut state);
            assert_eq!(outcome.health_delta, 1);
            assert_eq!(state.stats.hp, expected_hp);
            assert_eq!(state.last_damage, Some(DamageCause::ExposureCold));
        }
        state.day_state.rest.rest_requested = false;
        let outcome = apply_daily_effect(&cfg, &mut state);
        assert_eq!(outcome.health_delta, -1);
        assert_eq!(state.stats.hp, 9);
        assert_eq!(state.last_damage, Some(DamageCause::Breakdown));
    }

    #[test]
    fn daily_decay_preserves_the_cause_when_health_is_already_depleted() {
        let cfg = DailyTickConfig {
            health: HealthTickConfig {
                decay: 1.0,
                ..HealthTickConfig::default()
            },
            ..DailyTickConfig::default()
        };
        for (cause, ending) in [
            (
                DamageCause::Disease,
                Ending::Collapse {
                    cause: CollapseCause::Disease,
                },
            ),
            (
                DamageCause::Starvation,
                Ending::Collapse {
                    cause: CollapseCause::Hunger,
                },
            ),
            (
                DamageCause::ExposureCold,
                Ending::Exposure {
                    kind: ExposureKind::Cold,
                },
            ),
            (
                DamageCause::ExposureHeat,
                Ending::Exposure {
                    kind: ExposureKind::Heat,
                },
            ),
        ] {
            let mut state = GameState {
                last_damage: Some(cause),
                ..GameState::default()
            };
            state.stats.hp = 0;
            state.day_state.lifecycle.day_initialized = true;
            let outcome = apply_daily_effect(&cfg, &mut state);
            assert_eq!(outcome.health_delta, -1);
            assert_eq!(state.stats.hp, 0);
            assert_eq!(state.last_damage, Some(cause));
            let (ended, _, _) = state.travel_next_leg(&EndgameTravelCfg::default());
            assert!(ended);
            assert_eq!(state.ending, Some(ending));
            apply_daily_effect(&cfg, &mut state);
            assert_eq!(state.ending, Some(ending));
            assert_eq!(state.last_damage, Some(cause));
        }
    }

    #[test]
    fn destroyed_vehicle_keeps_terminal_priority_over_daily_health_collapse() {
        let mut state = GameState::default();
        state.stats.hp = 1;
        state.vehicle.health = 0.0;
        state.continuity.interactive_repairs = true;
        state.day_state.lifecycle.day_initialized = true;
        let cfg = DailyTickConfig {
            health: HealthTickConfig {
                decay: 1.0,
                ..HealthTickConfig::default()
            },
            ..DailyTickConfig::default()
        };
        let outcome = apply_daily_effect(&cfg, &mut state);
        assert_eq!(outcome.health_delta, -1);
        assert_eq!(state.stats.hp, 0);
        assert_eq!(state.last_damage, Some(DamageCause::Breakdown));
        let (ended, _, _) = state.travel_next_leg(&EndgameTravelCfg::default());
        assert!(ended);
        assert_eq!(
            state.ending,
            Some(Ending::VehicleFailure {
                cause: VehicleFailureCause::Destroyed,
            })
        );
        approx_eq(state.vehicle.health, 0.0);
    }

    fn approx_eq(left: f32, right: f32) {
        const EPS: f32 = 1e-6;
        assert!((left - right).abs() <= EPS, "{left} != {right}");
    }
}
