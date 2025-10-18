//! Camping and rest system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Stats, state::TravelDayKind};

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
    gs.start_of_day();
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
    if rest_cfg.supplies < 0 {
        let cost = rest_cfg.supplies.abs();
        let available = gs.stats.supplies.max(0);
        let actual_cost = cost.min(available);
        gs.stats.supplies -= actual_cost;
        supplies_delta -= actual_cost;
    } else if rest_cfg.supplies > 0 {
        gs.stats.supplies += rest_cfg.supplies;
        supplies_delta += rest_cfg.supplies;
    }

    let max_hp = Stats::default().hp;
    let max_sanity = Stats::default().sanity;
    if rest_cfg.hp != 0 {
        gs.stats.hp = (gs.stats.hp + rest_cfg.hp).clamp(0, max_hp);
    }
    if rest_cfg.sanity != 0 {
        gs.stats.sanity = (gs.stats.sanity + rest_cfg.sanity).clamp(0, max_sanity);
    }
    if rest_cfg.pants != 0 {
        gs.stats.pants = (gs.stats.pants + rest_cfg.pants).clamp(0, 100);
    }

    let rest_days = rest_cfg.day.max(1);
    for day_idx in 0..rest_days {
        if day_idx > 0 {
            gs.start_of_day();
        }
        if rest_cfg.recovery_day {
            gs.record_travel_day(TravelDayKind::None, 0.0, "camp");
        } else {
            gs.apply_rest_travel_credit();
        }
        gs.end_of_day();
    }
    gs.camp.rest_cooldown = rest_cfg.cooldown_days;
    gs.clear_illness_penalty();
    gs.rest_requested = false;
    gs.logs.push(String::from("log.camp.rest"));
    CampOutcome {
        message: String::from("log.camp.rest"),
        rested: true,
        supplies_delta,
    }
}

pub fn camp_forage(gs: &mut crate::GameState, cfg: &CampConfig) -> CampOutcome {
    gs.start_of_day();
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
            #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
            let adjusted = (f64::from(supplies_delta) * f64::from(*multiplier)).round() as i32;
            supplies_delta = if supplies_delta > 0 {
                adjusted.max(1)
            } else {
                adjusted.min(-1)
            };
        }
    }

    gs.stats.supplies += supplies_delta;
    gs.stats.clamp();
    let forage_days = forage_cfg.day.max(1);
    gs.advance_days_with_reason(forage_days, "camp");
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
pub fn can_repair(gs: &crate::GameState, _cfg: &CampConfig) -> bool {
    // Check if there's a breakdown to repair
    gs.breakdown.is_some()
}

#[must_use]
pub fn can_therapy(_gs: &crate::GameState, _cfg: &CampConfig) -> bool {
    // Placeholder - could check sanity levels, etc.
    true
}
