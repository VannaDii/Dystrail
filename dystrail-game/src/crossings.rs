//! Crossing and checkpoint system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::constants::PERMIT_REQUIRED_TAGS;
use crate::state::{Region, Season};

mod resolver;

pub use resolver::{
    CrossingContext, CrossingOutcome, CrossingResult, resolve_crossing, resolve_crossing_with_trace,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossingKind {
    #[serde(rename = "checkpoint")]
    Checkpoint,
    #[serde(rename = "bridge_out")]
    BridgeOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossingChoice {
    Detour,
    Bribe,
    Permit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetourCfg {
    pub days: i32,
    pub supplies: i32,
    pub pants: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BribeCfg {
    pub base_cost_cents: i64,
    pub success_chance: f32,
    pub on_fail: FailCfg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermitCfg {
    pub cred_gain: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailCfg {
    pub days: i32,
    pub pants: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossingTypeCfg {
    pub detour: DetourCfg,
    pub bribe: BribeCfg,
    pub permit: PermitCfg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialDetour {
    #[serde(default)]
    pub days: Option<i32>,
    #[serde(default)]
    pub pants: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherDetourMod {
    pub detour: PartialDetour,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecBribeMod {
    pub bribe_success_chance: f32,
    pub on_fail: FailCfg,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalMods {
    #[serde(default)]
    pub weather: HashMap<crate::weather::Weather, WeatherDetourMod>,
    #[serde(default)]
    pub exec_orders: HashMap<String, ExecBribeMod>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyCfg {
    pub currency: String,
    pub allow_negative_budget: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThresholdEntry {
    #[serde(default)]
    pub cost_multiplier: u32,
    #[serde(default)]
    pub success_adjust: f32,
    #[serde(default)]
    pub failure_adjust: f32,
}

impl Default for ThresholdEntry {
    fn default() -> Self {
        Self {
            cost_multiplier: 100,
            success_adjust: 0.0,
            failure_adjust: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SeasonalThresholds {
    #[serde(default)]
    pub spring: ThresholdEntry,
    #[serde(default)]
    pub summer: ThresholdEntry,
    #[serde(default)]
    pub fall: ThresholdEntry,
    #[serde(default)]
    pub winter: ThresholdEntry,
}

impl SeasonalThresholds {
    const fn get(&self, season: Season) -> ThresholdEntry {
        match season {
            Season::Spring => self.spring,
            Season::Summer => self.summer,
            Season::Fall => self.fall,
            Season::Winter => self.winter,
        }
    }

    const fn set(&mut self, season: Season, entry: ThresholdEntry) {
        match season {
            Season::Spring => self.spring = entry,
            Season::Summer => self.summer = entry,
            Season::Fall => self.fall = entry,
            Season::Winter => self.winter = entry,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ThresholdTable {
    #[serde(default)]
    pub regions: HashMap<String, SeasonalThresholds>,
}

impl ThresholdTable {
    #[must_use]
    pub fn with_defaults() -> Self {
        const fn entry(
            cost_multiplier: u32,
            success_adjust: f32,
            failure_adjust: f32,
        ) -> ThresholdEntry {
            ThresholdEntry {
                cost_multiplier,
                success_adjust,
                failure_adjust,
            }
        }

        const DEFAULTS: &[(Region, Season, ThresholdEntry)] = &[
            (Region::Heartland, Season::Spring, entry(103, 0.0, 0.005)),
            (Region::Heartland, Season::Summer, entry(113, -0.035, 0.04)),
            (Region::Heartland, Season::Fall, entry(115, -0.04, 0.045)),
            (Region::Heartland, Season::Winter, entry(128, -0.07, 0.07)),
            (Region::RustBelt, Season::Spring, entry(114, -0.04, 0.045)),
            (Region::RustBelt, Season::Summer, entry(116, -0.045, 0.05)),
            (Region::RustBelt, Season::Fall, entry(130, -0.08, 0.075)),
            (Region::RustBelt, Season::Winter, entry(133, -0.08, 0.08)),
            (Region::Beltway, Season::Spring, entry(118, -0.045, 0.05)),
            (Region::Beltway, Season::Summer, entry(133, -0.08, 0.08)),
            (Region::Beltway, Season::Fall, entry(136, -0.08, 0.08)),
            (Region::Beltway, Season::Winter, entry(138, -0.08, 0.08)),
        ];

        let mut table = Self {
            regions: HashMap::new(),
        };

        for &(region, season, entry) in DEFAULTS {
            table
                .regions
                .entry(region.asset_key().to_string())
                .or_default()
                .set(season, entry);
        }

        table
    }

    #[must_use]
    pub fn lookup(&self, region: Region, season: Season) -> ThresholdEntry {
        self.regions
            .get(region.asset_key())
            .map(|seasonal| seasonal.get(season))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossingConfig {
    pub types: HashMap<CrossingKind, CrossingTypeCfg>,
    pub global_mods: GlobalMods,
    pub money: MoneyCfg,
    #[serde(default)]
    pub thresholds: ThresholdTable,
}

impl Default for CrossingConfig {
    fn default() -> Self {
        let mut types = HashMap::new();

        // Default checkpoint configuration
        types.insert(
            CrossingKind::Checkpoint,
            CrossingTypeCfg {
                detour: DetourCfg {
                    days: 2,
                    supplies: -2,
                    pants: 1,
                },
                bribe: BribeCfg {
                    base_cost_cents: 1000, // $10.00
                    success_chance: 1.0,
                    on_fail: FailCfg { days: 0, pants: 0 },
                },
                permit: PermitCfg { cred_gain: 1 },
            },
        );

        // Default bridge out configuration
        types.insert(
            CrossingKind::BridgeOut,
            CrossingTypeCfg {
                detour: DetourCfg {
                    days: 3,
                    supplies: -3,
                    pants: 2,
                },
                bribe: BribeCfg {
                    base_cost_cents: 1500, // $15.00
                    success_chance: 1.0,
                    on_fail: FailCfg { days: 0, pants: 0 },
                },
                permit: PermitCfg { cred_gain: 1 },
            },
        );

        // Default global modifiers
        let mut weather_mods = HashMap::new();
        weather_mods.insert(
            crate::weather::Weather::Storm,
            WeatherDetourMod {
                detour: PartialDetour {
                    days: Some(1),
                    pants: Some(1),
                },
            },
        );

        let mut exec_mods = HashMap::new();
        exec_mods.insert(
            "Shutdown".to_string(),
            ExecBribeMod {
                bribe_success_chance: 0.5,
                on_fail: FailCfg { days: 1, pants: 3 },
            },
        );

        Self {
            types,
            global_mods: GlobalMods {
                weather: weather_mods,
                exec_orders: exec_mods,
            },
            money: MoneyCfg {
                currency: "USD".to_string(),
                allow_negative_budget: false,
            },
            thresholds: ThresholdTable::with_defaults(),
        }
    }
}

/// Apply bribe option to crossing
///
/// # Panics
///
/// Panics if the crossing kind is not found in the configuration.
pub fn apply_bribe(gs: &mut crate::GameState, cfg: &CrossingConfig, kind: CrossingKind) -> String {
    let type_cfg = cfg.types.get(&kind).unwrap();
    let bribe_cost =
        calculate_bribe_cost(type_cfg.bribe.base_cost_cents, gs.mods.bribe_discount_pct);

    if gs.budget_cents >= bribe_cost {
        gs.budget_cents -= bribe_cost;
        gs.budget = i32::try_from(gs.budget_cents / 100).unwrap_or(0);
        gs.bribes_spent_cents += bribe_cost;
        "crossing.result.bribe.success".to_string()
    } else {
        "crossing.result.bribe.fail".to_string()
    }
}

/// Apply detour option to crossing
///
/// # Panics
///
/// Panics if the crossing kind is not found in the configuration.
pub fn apply_detour(gs: &mut crate::GameState, cfg: &CrossingConfig, kind: CrossingKind) -> String {
    let type_cfg = cfg.types.get(&kind).unwrap();
    gs.stats.supplies += type_cfg.detour.supplies; // Can be negative (cost)
    gs.stats.pants += type_cfg.detour.pants;
    let detour_days = type_cfg.detour.days.max(1);
    let partial = crate::journey::tick_non_travel_day_for_state(
        gs,
        crate::journey::TravelDayKind::Partial,
        0.0,
        "detour",
    );
    if detour_days > 1 {
        let extra = u32::try_from(detour_days - 1).unwrap_or(0);
        gs.advance_days_with_credit(
            extra,
            crate::journey::TravelDayKind::Partial,
            partial,
            "detour",
        );
    }
    "crossing.result.detour.success".to_string()
}

/// Apply permit option to crossing
///
/// # Panics
///
/// Panics if the crossing kind is not found in the configuration.
pub fn apply_permit(gs: &mut crate::GameState, cfg: &CrossingConfig, kind: CrossingKind) -> String {
    let type_cfg = cfg.types.get(&kind).unwrap();
    // For now, assume player always has permits available
    gs.stats.credibility += type_cfg.permit.cred_gain;
    "crossing.result.permit.success".to_string()
}

/// Calculate bribe cost based on base cost and discount
#[must_use]
pub fn calculate_bribe_cost(base_cost: i64, discount_pct: i32) -> i64 {
    if discount_pct <= 0 {
        return base_cost;
    }
    let clamped_pct = discount_pct.clamp(0, 100);
    let numerator = base_cost.saturating_mul(i64::from(100 - clamped_pct));
    let (quot, rem) = (numerator.div_euclid(100), numerator.rem_euclid(100));
    if rem == 0 {
        quot
    } else {
        quot.saturating_add(1)
    }
}

/// Check if player can afford bribe
#[must_use]
pub fn can_afford_bribe(gs: &crate::GameState, cfg: &CrossingConfig, kind: CrossingKind) -> bool {
    let Some(type_cfg) = cfg.types.get(&kind) else {
        return false;
    };
    let bribe_cost =
        calculate_bribe_cost(type_cfg.bribe.base_cost_cents, gs.mods.bribe_discount_pct);
    gs.budget_cents >= bribe_cost
}

/// Check if player can use permit
#[must_use]
pub fn can_use_permit(gs: &crate::GameState, _kind: &CrossingKind) -> bool {
    if PERMIT_REQUIRED_TAGS
        .iter()
        .any(|tag| gs.inventory.has_tag(tag))
    {
        return true;
    }

    gs.receipts
        .iter()
        .any(|receipt| PERMIT_REQUIRED_TAGS.iter().any(|tag| receipt.contains(tag)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journey::{RngBundle, TravelDayKind};
    use crate::state::{GameState, Region, Season};
    use std::rc::Rc;

    #[test]
    fn apply_detour_records_partial_day() {
        let mut state = GameState::default();
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(5)));
        let cfg = CrossingConfig::default();
        let baseline_records = state.day_records.len();

        let result = apply_detour(&mut state, &cfg, CrossingKind::BridgeOut);

        assert_eq!(result, "crossing.result.detour.success");
        let added = state.day_records.len().saturating_sub(baseline_records);
        assert!(
            added >= 1,
            "expected detour to add at least one recorded day, got {added}"
        );
        let last_days = &state.day_records[(state.day_records.len() - added)..];
        assert!(
            last_days
                .iter()
                .all(|rec| rec.kind == TravelDayKind::Partial)
        );
        assert!(last_days.iter().all(|rec| rec.miles > 0.0));
    }

    #[test]
    fn calculate_bribe_cost_rounds_up() {
        assert_eq!(calculate_bribe_cost(1000, 0), 1000);
        assert_eq!(calculate_bribe_cost(999, 10), 900);
        assert_eq!(calculate_bribe_cost(1000, 150), 0);
    }

    #[test]
    fn apply_bribe_spends_budget_when_possible() {
        let mut state = GameState {
            budget_cents: 1500,
            budget: 15,
            ..GameState::default()
        };
        let cfg = CrossingConfig::default();
        let bribe_cost = calculate_bribe_cost(
            cfg.types[&CrossingKind::Checkpoint].bribe.base_cost_cents,
            0,
        );

        let result = apply_bribe(&mut state, &cfg, CrossingKind::Checkpoint);

        assert_eq!(result, "crossing.result.bribe.success");
        assert_eq!(state.bribes_spent_cents, bribe_cost);
        assert_eq!(state.budget_cents, 1500 - bribe_cost);
    }

    #[test]
    fn apply_bribe_fails_without_budget() {
        let mut state = GameState::default();
        let cfg = CrossingConfig::default();
        let bribe_cost =
            calculate_bribe_cost(cfg.types[&CrossingKind::BridgeOut].bribe.base_cost_cents, 0);
        state.budget_cents = bribe_cost.saturating_sub(1);
        state.budget = i32::try_from(state.budget_cents / 100).unwrap_or(0);

        let result = apply_bribe(&mut state, &cfg, CrossingKind::BridgeOut);

        assert_eq!(result, "crossing.result.bribe.fail");
        assert_eq!(state.bribes_spent_cents, 0);
    }

    #[test]
    fn apply_permit_increases_credibility() {
        let mut state = GameState::default();
        let cfg = CrossingConfig::default();
        let starting = state.stats.credibility;

        let result = apply_permit(&mut state, &cfg, CrossingKind::Checkpoint);

        assert_eq!(result, "crossing.result.permit.success");
        assert_eq!(
            state.stats.credibility,
            starting + cfg.types[&CrossingKind::Checkpoint].permit.cred_gain
        );
    }

    #[test]
    fn can_afford_bribe_respects_discount() {
        let mut state = GameState::default();
        let cfg = CrossingConfig::default();
        state.mods.bribe_discount_pct = 50;
        let cost = calculate_bribe_cost(
            cfg.types[&CrossingKind::Checkpoint].bribe.base_cost_cents,
            state.mods.bribe_discount_pct,
        );
        state.budget_cents = cost.saturating_sub(1);
        assert!(!can_afford_bribe(&state, &cfg, CrossingKind::Checkpoint));
        state.budget_cents = cost;
        assert!(can_afford_bribe(&state, &cfg, CrossingKind::Checkpoint));
    }

    #[test]
    fn can_use_permit_detects_tags_and_receipts() {
        let mut state = GameState::default();
        assert!(!can_use_permit(&state, &CrossingKind::Checkpoint));

        state.inventory.tags.insert(String::from("permit"));
        assert!(can_use_permit(&state, &CrossingKind::Checkpoint));

        state.inventory.tags.clear();
        state.receipts.push(String::from("press_pass"));
        assert!(can_use_permit(&state, &CrossingKind::Checkpoint));
    }

    #[test]
    fn threshold_table_defaults_lookup_values() {
        let table = ThresholdTable::with_defaults();
        let entry = table.lookup(Region::Heartland, Season::Summer);
        assert_eq!(entry.cost_multiplier, 113);
        assert!((entry.success_adjust + 0.035).abs() <= f32::EPSILON);
    }
}
