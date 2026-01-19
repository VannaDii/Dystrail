//! Camping and rest system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::endgame::EndgameTravelCfg;
use crate::journey::{DailyTickKernel, resolve_cfg_for_state};
use crate::{Stats, TravelDayKind, numbers::round_f64_to_i32};

const DEFAULT_CAMP_DATA: &str = include_str!("../../dystrail-web/static/assets/data/camp.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CampState {
    pub rest_cooldown: u32,
    pub forage_cooldown: u32,
    pub repair_cooldown: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CampConfig {
    #[serde(default)]
    pub rest: RestConfig,
    #[serde(default)]
    pub forage: ForageConfig,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RestConfig {
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub supplies: i32,
    #[serde(default)]
    pub pants: i32,
    #[serde(default)]
    pub day: u32,
    #[serde(default)]
    pub cooldown_days: u32,
    #[serde(default)]
    pub recovery_day: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ForageConfig {
    #[serde(default)]
    pub supplies: i32,
    #[serde(default)]
    pub day: u32,
    #[serde(default)]
    pub cooldown_days: u32,
    #[serde(default)]
    pub region_multipliers: HashMap<String, f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampOutcome {
    pub message: String,
    pub rested: bool,
    pub supplies_delta: i32,
}

impl CampConfig {
    #[must_use]
    pub fn load_from_static() -> Self {
        serde_json::from_str(DEFAULT_CAMP_DATA).unwrap_or_default()
    }

    #[must_use]
    pub fn default_config() -> Self {
        Self::load_from_static()
    }
}

pub fn camp_rest(gs: &mut crate::GameState, cfg: &CampConfig) -> CampOutcome {
    let endgame_cfg = EndgameTravelCfg::default_config();
    camp_rest_with_endgame(gs, cfg, &endgame_cfg)
}

pub fn camp_rest_with_endgame(
    gs: &mut crate::GameState,
    cfg: &CampConfig,
    endgame_cfg: &EndgameTravelCfg,
) -> CampOutcome {
    let rest_cfg = &cfg.rest;
    if rest_cfg.day == 0 {
        return CampOutcome {
            message: String::from("log.camp.rest.disabled"),
            rested: false,
            supplies_delta: 0,
        };
    }

    if gs.camp.rest_cooldown > 0 {
        return CampOutcome {
            message: String::from("log.camp.rest.cooldown"),
            rested: false,
            supplies_delta: 0,
        };
    }

    let mut supplies_delta = 0;
    let rest_days = rest_cfg.day.max(1);
    let rest_supplies = rest_cfg.supplies;
    let rest_hp = rest_cfg.hp;
    let rest_sanity = rest_cfg.sanity;
    let rest_pants = rest_cfg.pants;
    let rest_recovery_day = rest_cfg.recovery_day;
    let max_hp = Stats::default().hp;
    let max_sanity = Stats::default().sanity;
    let journey_cfg = resolve_cfg_for_state(gs);
    let kernel = DailyTickKernel::new(&journey_cfg, endgame_cfg);
    for day_idx in 0..rest_days {
        let apply_effects = day_idx == 0;
        kernel.tick_non_travel_day_with_hook(gs, TravelDayKind::NonTravel, 0.0, "camp", |state| {
            if apply_effects {
                if rest_supplies < 0 {
                    let cost = rest_supplies.abs();
                    let available = state.stats.supplies.max(0);
                    let actual_cost = cost.min(available);
                    state.stats.supplies -= actual_cost;
                    supplies_delta = -actual_cost;
                } else if rest_supplies > 0 {
                    state.stats.supplies += rest_supplies;
                    supplies_delta = rest_supplies;
                }

                if rest_hp != 0 {
                    state.stats.hp = (state.stats.hp + rest_hp).clamp(0, max_hp);
                }
                if rest_sanity != 0 {
                    state.stats.sanity = (state.stats.sanity + rest_sanity).clamp(0, max_sanity);
                }
                if rest_pants != 0 {
                    state.stats.pants = (state.stats.pants + rest_pants).clamp(0, 100);
                }
            }

            if !rest_recovery_day {
                state.apply_rest_travel_credit();
            }
        });
    }
    gs.camp.rest_cooldown = rest_cfg.cooldown_days;
    gs.clear_illness_penalty();
    gs.day_state.rest.rest_requested = false;
    gs.logs.push(String::from("log.camp.rest"));
    CampOutcome {
        message: String::from("log.camp.rest"),
        rested: true,
        supplies_delta,
    }
}

pub fn camp_forage(gs: &mut crate::GameState, cfg: &CampConfig) -> CampOutcome {
    let endgame_cfg = EndgameTravelCfg::default_config();
    camp_forage_with_endgame(gs, cfg, &endgame_cfg)
}

pub fn camp_forage_with_endgame(
    gs: &mut crate::GameState,
    cfg: &CampConfig,
    endgame_cfg: &EndgameTravelCfg,
) -> CampOutcome {
    let forage_cfg = &cfg.forage;
    if forage_cfg.day == 0 || forage_cfg.supplies == 0 {
        return CampOutcome {
            message: String::from("log.camp.forage.disabled"),
            rested: false,
            supplies_delta: 0,
        };
    }

    if gs.camp.forage_cooldown > 0 {
        return CampOutcome {
            message: String::from("log.camp.forage.cooldown"),
            rested: false,
            supplies_delta: 0,
        };
    }

    let mut supplies_delta = forage_cfg.supplies;
    if supplies_delta != 0 && !forage_cfg.region_multipliers.is_empty() {
        let region_key = gs.region.asset_key();
        if let Some(multiplier) = forage_cfg.region_multipliers.get(region_key) {
            let scaled = f64::from(supplies_delta) * f64::from(*multiplier);
            let clamped = scaled.clamp(f64::from(i32::MIN), f64::from(i32::MAX));
            let adjusted_i32 = round_f64_to_i32(clamped);
            supplies_delta = if supplies_delta > 0 {
                adjusted_i32.max(1)
            } else {
                adjusted_i32.min(-1)
            };
        }
    }

    let forage_days = forage_cfg.day.max(1);
    let journey_cfg = resolve_cfg_for_state(gs);
    let kernel = DailyTickKernel::new(&journey_cfg, endgame_cfg);
    for day_idx in 0..forage_days {
        let apply_forage = day_idx == 0 && supplies_delta != 0;
        kernel.tick_non_travel_day_with_hook(gs, TravelDayKind::NonTravel, 0.0, "camp", |state| {
            if apply_forage {
                state.stats.supplies += supplies_delta;
                state.stats.clamp();
            }
        });
    }
    gs.camp.forage_cooldown = forage_cfg.cooldown_days;
    gs.logs.push(String::from("log.camp.forage"));

    CampOutcome {
        message: String::from("log.camp.forage"),
        rested: false,
        supplies_delta,
    }
}

pub fn camp_therapy(_gs: &mut crate::GameState, _cfg: &CampConfig) -> CampOutcome {
    CampOutcome {
        message: String::from("log.camp.therapy"),
        rested: false,
        supplies_delta: 0,
    }
}

pub fn camp_repair_spare(
    _gs: &mut crate::GameState,
    _cfg: &CampConfig,
    _part: crate::vehicle::Part,
) -> CampOutcome {
    CampOutcome {
        message: String::from("log.camp.repair"),
        rested: false,
        supplies_delta: 0,
    }
}

pub fn camp_repair_hack(_gs: &mut crate::GameState, _cfg: &CampConfig) -> CampOutcome {
    CampOutcome {
        message: String::from("log.camp.repair.hack"),
        rested: false,
        supplies_delta: 0,
    }
}

#[must_use]
pub const fn can_repair(gs: &crate::GameState, _cfg: &CampConfig) -> bool {
    // Check if there's a breakdown to repair
    gs.breakdown.is_some()
}

#[must_use]
pub const fn can_therapy(_gs: &crate::GameState, _cfg: &CampConfig) -> bool {
    // Placeholder - could check sanity levels, etc.
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Region, Stats};

    #[test]
    fn forage_negative_supplies_apply_region_multiplier() {
        let mut cfg = CampConfig::default();
        cfg.forage.day = 1;
        cfg.forage.supplies = -3;
        cfg.forage
            .region_multipliers
            .insert(String::from("Heartland"), 0.5);

        let mut state = crate::GameState {
            region: Region::Heartland,
            stats: Stats {
                supplies: 10,
                ..Stats::default()
            },
            ..crate::GameState::default()
        };

        let outcome = camp_forage(&mut state, &cfg);

        assert!(outcome.supplies_delta < 0);
        assert!(state.stats.supplies < 10);
        assert_eq!(outcome.message, "log.camp.forage");
    }
}
