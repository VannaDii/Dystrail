//! Camping and rest system
use serde::{Deserialize, Serialize};

use crate::{Stats, TravelDayKind, activities::Activity};

const DEFAULT_CAMP_DATA: &str = include_str!("../../dystrail-web/static/assets/data/camp.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CampState {
    pub rest_cooldown: u32,
    pub repair_cooldown: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CampConfig {
    #[serde(default)]
    pub rest: RestConfig,
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
    pub day: u32,
    #[serde(default)]
    pub cooldown_days: u32,
    #[serde(default)]
    pub recovery_day: bool,
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
    let rest_cfg = &cfg.rest;
    if gs.breakdown.is_some() || rest_cfg.day == 0 {
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

    gs.start_of_day();
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

    let rest_days = rest_cfg.day.max(1);
    for day_idx in 0..rest_days {
        if day_idx > 0 {
            gs.start_of_day();
        }
        if rest_cfg.recovery_day {
            gs.day_state.lifecycle.suppress_stop_ratio = true;
            gs.record_travel_day(TravelDayKind::NonTravel, 0.0, "camp");
        } else {
            gs.apply_rest_travel_credit();
        }
        gs.end_of_day();
    }
    gs.camp.rest_cooldown = rest_cfg.cooldown_days;
    gs.clear_illness_penalty();
    // A full rest buffers the crew against immediately falling ill again.
    gs.disease_cooldown = gs.disease_cooldown.max(10);
    gs.recover_crew();
    gs.day_state.rest.rest_requested = false;
    gs.logs.push(String::from("log.camp.rest"));
    CampOutcome {
        message: String::from("log.camp.rest"),
        rested: true,
        supplies_delta,
    }
}

/// Camp and roadside gathering use the same costs, clock and cooldown.
pub fn camp_forage(gs: &mut crate::GameState, _cfg: &CampConfig) -> CampOutcome {
    let supplies_before = gs.stats.supplies;
    if !gs.perform_activity(Activity::Forage) {
        return CampOutcome {
            message: String::from(if gs.forage_cooldown_days() > 0 {
                "log.camp.forage.cooldown"
            } else {
                "log.camp.forage.disabled"
            }),
            rested: false,
            supplies_delta: 0,
        };
    }
    gs.logs.push(String::from("log.camp.forage"));
    CampOutcome {
        message: String::from("log.camp.forage"),
        rested: false,
        supplies_delta: gs.stats.supplies - supplies_before,
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
mod explicit_breakdown_tests {
    #[test]
    fn camping_cannot_spend_time_resources_or_consume_a_spare_during_breakdown() {
        let mut gs = crate::GameState::default();
        gs.breakdown = Some(crate::vehicle::Breakdown {
            part: crate::vehicle::Part::Battery,
            day_started: i32::try_from(gs.day).unwrap(),
        });
        gs.inventory.spares.battery = 1;
        let before = serde_json::to_string(&gs).unwrap();
        let cfg = super::CampConfig::default_config();
        assert!(!super::camp_rest(&mut gs, &cfg).rested);
        assert_eq!(super::camp_forage(&mut gs, &cfg).supplies_delta, 0);
        assert_eq!(serde_json::to_string(&gs).unwrap(), before);
    }

    #[test]
    fn camp_and_roadside_forage_share_costs_and_cooldown() {
        let mut camp = crate::GameState::default();
        camp.stats.supplies = 5;
        camp.stats.sanity = 5;
        let mut roadside = camp.clone();
        let cfg = super::CampConfig::default_config();

        let outcome = super::camp_forage(&mut camp, &cfg);
        assert_eq!(outcome.supplies_delta, 2);
        assert!(roadside.perform_activity(crate::activities::Activity::Forage));
        assert_eq!(camp.stats, roadside.stats);
        assert_eq!(camp.stats.supplies, 7);
        assert_eq!(camp.stats.sanity, 6);
        assert_eq!(camp.day, 1);
        assert_eq!(
            camp.continuity.clock_minutes,
            crate::journal::morning() + 120
        );
        assert_eq!(
            camp.continuity.clock_minutes,
            roadside.continuity.clock_minutes
        );
        assert_eq!(
            camp.forage_cooldown_days(),
            crate::activities::FORAGE_COOLDOWN_DAYS
        );
        assert_eq!(camp.miles_traveled_actual.to_bits(), 0.0_f32.to_bits());
        assert_eq!(camp.continuity.driving_minutes_total, 0);

        let mut restored: crate::GameState =
            serde_json::from_str(&serde_json::to_string(&camp).unwrap()).unwrap();
        let before = serde_json::to_string(&restored).unwrap();
        assert!(!restored.perform_activity(crate::activities::Activity::Glean));
        assert_eq!(super::camp_forage(&mut restored, &cfg).supplies_delta, 0);
        assert_eq!(serde_json::to_string(&restored).unwrap(), before);
        restored.day += crate::activities::FORAGE_COOLDOWN_DAYS - 1;
        assert!(!restored.can_activity(crate::activities::Activity::Forage));
        restored.day += 1;
        assert!(restored.can_activity(crate::activities::Activity::Glean));
    }
}
