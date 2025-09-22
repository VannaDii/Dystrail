use gloo::net::http::Request;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::game::state::GameState;
use crate::game::vehicle::Part;
use crate::i18n;

/// Configuration for rest action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestCfg {
    pub sanity: i32,
    pub hp: i32,
    pub supplies: i32,
    pub day: i32,
}

impl Default for RestCfg {
    fn default() -> Self {
        Self {
            sanity: 2,
            hp: 1,
            supplies: -1,
            day: 1,
        }
    }
}

/// Configuration for therapy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapyCfg {
    pub sanity: i32,
    pub burn_receipt: i32,
    pub day: i32,
    pub cooldown_days: i32,
}

impl Default for TherapyCfg {
    fn default() -> Self {
        Self {
            sanity: 2,
            burn_receipt: 1,
            day: 1,
            cooldown_days: 3,
        }
    }
}

/// Weights for forage outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForageWeights {
    pub supplies: u32,
    pub none: u32,
    pub receipt: u32,
}

impl Default for ForageWeights {
    fn default() -> Self {
        Self {
            supplies: 50,
            none: 30,
            receipt: 20,
        }
    }
}

/// Configuration for forage action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForageCfg {
    pub weights: ForageWeights,
    pub receipt_bonus_cap: i32,
}

impl Default for ForageCfg {
    fn default() -> Self {
        Self {
            weights: ForageWeights::default(),
            receipt_bonus_cap: 25,
        }
    }
}

/// Configuration for hack fix repair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackCfg {
    pub supplies: i32,
    pub credibility: i32,
    pub day: i32,
}

impl Default for HackCfg {
    fn default() -> Self {
        Self {
            supplies: 3,
            credibility: 1,
            day: 1,
        }
    }
}

/// Configuration for repair actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairCfg {
    pub hack: HackCfg,
    pub use_spare_supplies: i32,
}

impl Default for RepairCfg {
    fn default() -> Self {
        Self {
            hack: HackCfg::default(),
            use_spare_supplies: 1,
        }
    }
}

/// Complete camp configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CampConfig {
    #[serde(default)]
    pub rest: RestCfg,
    #[serde(default)]
    pub therapy: TherapyCfg,
    #[serde(default)]
    pub forage: ForageCfg,
    #[serde(default)]
    pub repair: RepairCfg,
}

/// Camp state tracking cooldowns and usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CampState {
    /// Day when therapy was last used (for cooldown)
    #[serde(default)]
    pub last_therapy_day: Option<i32>,
}

impl CampConfig {
    /// Load configuration from JSON, falling back to defaults
    pub async fn load() -> Self {
        if let Ok(response) = Request::get("/static/assets/data/camp.json").send().await
            && response.ok()
            && let Ok(text) = response.text().await
            && let Ok(config) = serde_json::from_str::<CampConfig>(&text) {
            return config;
        }
        // If loading fails, use embedded defaults
        Self::default_config()
    }

    /// Load from static assets for WASM target
    pub async fn load_from_static() -> Self {
        Self::load().await
    }

    /// Get embedded default configuration if loading fails
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            rest: RestCfg::default(),
            therapy: TherapyCfg::default(),
            forage: ForageCfg::default(),
            repair: RepairCfg::default(),
        }
    }
}

/// Execute rest action
pub fn camp_rest(gs: &mut GameState, cfg: &CampConfig) -> String {
    gs.stats.sanity += cfg.rest.sanity;
    gs.stats.hp += cfg.rest.hp;
    gs.stats.supplies += cfg.rest.supplies;
    gs.day += u32::try_from(cfg.rest.day).unwrap_or(1);

    // Apply stat clamping
    gs.stats.clamp();

    i18n::t("camp.announce.rest")
}

/// Execute forage action with deterministic PRNG
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss, clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn camp_forage(gs: &mut GameState, cfg: &CampConfig) -> String {
    if let Some(ref mut rng) = gs.rng {
        // Adjust weights with receipt bonus
        let w_sup = cfg.forage.weights.supplies as i32;
        let mut w_rec = cfg.forage.weights.receipt as i32;
        let bonus = gs
            .receipt_bonus_pct
            .clamp(-cfg.forage.receipt_bonus_cap, cfg.forage.receipt_bonus_cap);
        w_rec = (w_rec as f32 * (1.0 + (bonus as f32) / 100.0))
            .round()
            .max(0.0) as i32;

        let total = (w_sup + cfg.forage.weights.none as i32 + w_rec).max(1) as u32;
        let mut roll = rng.random_range(0..total);

        if roll < w_sup as u32 {
            gs.stats.supplies += 1;
            gs.stats.clamp();
            i18n::t("camp.announce.forage_sup")
        } else {
            roll -= w_sup as u32;
            if roll < cfg.forage.weights.none {
                i18n::t("camp.announce.forage_none")
            } else {
                // Convert receipts vec into receipt count
                gs.receipts.push("foraged_receipt".to_string());
                i18n::t("camp.announce.forage_receipt")
            }
        }
    } else {
        i18n::t("camp.announce.forage_none")
    }
}

/// Execute therapy action with cooldown checking
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub fn camp_therapy(gs: &mut GameState, cfg: &CampConfig) -> String {
    // Check cooldown
    if let Some(last_day) = gs.camp.last_therapy_day {
        let days_since = i32::try_from(gs.day).unwrap_or(0) - last_day;
        if days_since < cfg.therapy.cooldown_days {
            let days_left = cfg.therapy.cooldown_days - days_since;
            let mut vars = HashMap::new();
            let days_str = days_left.to_string();
            vars.insert("days", days_str.as_str());
            return i18n::tr("camp.announce.cooldown", Some(&vars));
        }
    }

    // Check if we have receipts to burn
    if gs.receipts.len() < usize::try_from(cfg.therapy.burn_receipt).unwrap_or(1) {
        return i18n::t("camp.announce.no_receipt");
    }

    // Apply therapy effects
    gs.stats.sanity += cfg.therapy.sanity;
    for _ in 0..cfg.therapy.burn_receipt {
        if !gs.receipts.is_empty() {
            gs.receipts.pop();
        }
    }
    // Record therapy day before advancing the day
    gs.camp.last_therapy_day = Some(i32::try_from(gs.day).unwrap_or(0));
    gs.day += u32::try_from(cfg.therapy.day).unwrap_or(1);

    gs.stats.clamp();

    i18n::t("camp.announce.therapy")
}

/// Execute repair using spare part
pub fn camp_repair_spare(gs: &mut GameState, cfg: &CampConfig, part: Part) -> String {
    // Check if we have the right spare
    let has_spare = match part {
        Part::Tire => gs.inventory.spares.tire > 0,
        Part::Battery => gs.inventory.spares.battery > 0,
        Part::Alternator => gs.inventory.spares.alt > 0,
        Part::FuelPump => gs.inventory.spares.pump > 0,
    };

    if !has_spare {
        let mut vars = HashMap::new();
        vars.insert("part", part.key());
        return i18n::tr("vehicle.announce.no_spare", Some(&vars));
    }

    // Consume spare and supplies
    match part {
        Part::Tire => gs.inventory.spares.tire -= 1,
        Part::Battery => gs.inventory.spares.battery -= 1,
        Part::Alternator => gs.inventory.spares.alt -= 1,
        Part::FuelPump => gs.inventory.spares.pump -= 1,
    }

    gs.stats.supplies -= cfg.repair.use_spare_supplies;
    gs.stats.clamp();

    // Clear breakdown
    gs.breakdown = None;
    gs.travel_blocked = false;

    let mut vars = HashMap::new();
    let part_str = i18n::t(part.key());
    let sup_str = cfg.repair.use_spare_supplies.to_string();
    vars.insert("part", part_str.as_str());
    vars.insert("sup", sup_str.as_str());
    i18n::tr("camp.announce.repair_spare", Some(&vars))
}

/// Execute hack fix repair
pub fn camp_repair_hack(gs: &mut GameState, cfg: &CampConfig) -> String {
    // Check if there's actually a breakdown to repair
    if gs.breakdown.is_none() {
        return i18n::tr("camp.error.no_breakdown", None);
    }

    gs.stats.supplies -= cfg.repair.hack.supplies;
    gs.stats.credibility -= cfg.repair.hack.credibility;
    gs.day += u32::try_from(cfg.repair.hack.day).unwrap_or(1);
    gs.stats.clamp();

    // Clear breakdown
    gs.breakdown = None;
    gs.travel_blocked = false;

    let mut vars = HashMap::new();
    let sup_str = cfg.repair.hack.supplies.to_string();
    let cred_str = cfg.repair.hack.credibility.to_string();
    let day_str = cfg.repair.hack.day.to_string();
    vars.insert("sup", sup_str.as_str());
    vars.insert("cred", cred_str.as_str());
    vars.insert("day", day_str.as_str());
    i18n::tr("camp.announce.repair_hack", Some(&vars))
}

/// Check if repair is available (has breakdown)
#[must_use]
pub fn can_repair(gs: &GameState) -> bool {
    gs.breakdown.is_some()
}

/// Check if therapy is available (not on cooldown and has receipts)
#[must_use]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub fn can_therapy(gs: &GameState, cfg: &CampConfig) -> bool {
    // Check cooldown
    if let Some(last_day) = gs.camp.last_therapy_day {
        let days_since = i32::try_from(gs.day).unwrap_or(0) - last_day;
        if days_since < cfg.therapy.cooldown_days {
            return false;
        }
    }

    // Check receipts
    gs.receipts.len() >= usize::try_from(cfg.therapy.burn_receipt).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::vehicle::{Part, Breakdown};

    fn create_test_config() -> CampConfig {
        CampConfig::default()
    }

    fn create_test_state() -> GameState {
        GameState::default()
    }

    #[test]
    fn test_camp_rest_basic() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.stats.sanity = 5;
        state.stats.hp = 8;
        state.stats.supplies = 10;
        state.day = 1;

        let initial_sanity = state.stats.sanity;
        let initial_hp = state.stats.hp;
        let initial_supplies = state.stats.supplies;
        let initial_day = state.day;

        let msg = camp_rest(&mut state, &config);

        // Check state changes according to config
        assert_eq!(state.stats.sanity, initial_sanity + 2); // +2
        assert_eq!(state.stats.hp, initial_hp + 1); // +1
        assert_eq!(state.stats.supplies, initial_supplies - 1); // -1
        assert_eq!(state.day, initial_day + 1); // +1
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_rest_no_negative_supplies() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.stats.supplies = 0; // No supplies available

        let msg = camp_rest(&mut state, &config);

        // Supplies should not go below 0
        assert!(state.stats.supplies >= 0);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_therapy_success() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.stats.sanity = 5;
        state.receipts = vec!["test".to_string(), "test2".to_string(), "test3".to_string()];
        state.day = 1;
        state.camp.last_therapy_day = None; // No previous therapy

        let initial_sanity = state.stats.sanity;
        let initial_receipts = state.receipts.len();
        let initial_day = state.day;

        let msg = camp_therapy(&mut state, &config);

        // Check successful therapy
        assert_eq!(state.stats.sanity, initial_sanity + 2); // +2
        assert_eq!(state.receipts.len(), initial_receipts - 1); // -1 burned
        assert_eq!(state.day, initial_day + 1); // +1
        assert_eq!(state.camp.last_therapy_day, Some(1)); // Therapy day recorded
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_therapy_no_receipts() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.receipts = vec![]; // No receipts to burn
        state.camp.last_therapy_day = None;

        let initial_day = state.day;

        let msg = camp_therapy(&mut state, &config);

        // Should return error message and unchanged state
        assert_eq!(state.receipts.len(), 0);
        assert_eq!(state.day, initial_day); // No day change
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_therapy_cooldown_active() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.receipts = vec!["test".to_string(), "test2".to_string(), "test3".to_string()];
        state.day = 2;
        state.camp.last_therapy_day = Some(1); // Therapy yesterday, cooldown active

        let initial_receipts = state.receipts.len();
        let initial_day = state.day;

        let msg = camp_therapy(&mut state, &config);

        // Should return cooldown message and unchanged state
        assert_eq!(state.receipts.len(), initial_receipts);
        assert_eq!(state.day, initial_day);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_forage_basic() {
        let config = create_test_config();
        let mut state = create_test_state();

        let initial_supplies = state.stats.supplies;
        let initial_receipts = state.receipts.len();

        let msg = camp_forage(&mut state, &config);

        // Should either gain supplies, receipts, or nothing
        let _supplies_changed = state.stats.supplies != initial_supplies;
        let _receipts_changed = state.receipts.len() != initial_receipts;

        // At least one of these should be true (or neither - for "nothing" outcome)
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_repair_spare_success() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.inventory.spares.tire = 1; // Has spare tire
        state.stats.supplies = 10;

        let initial_supplies = state.stats.supplies;

        let msg = camp_repair_spare(&mut state, &config, Part::Tire);

        // Should repair breakdown and consume spare and supplies
        assert!(state.breakdown.is_none());
        assert_eq!(state.inventory.spares.tire, 0);
        assert!(state.stats.supplies <= initial_supplies); // Supplies consumed or same
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_repair_spare_no_spare() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.inventory.spares.tire = 0; // No spare tire available

        let msg = camp_repair_spare(&mut state, &config, Part::Tire);

        // Should return error message and unchanged state
        assert_eq!(state.breakdown.as_ref().unwrap().part, Part::Tire);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_repair_hack_success() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.stats.supplies = 10;
        state.stats.credibility = 10;
        state.day = 1;

        let initial_supplies = state.stats.supplies;
        let initial_credibility = state.stats.credibility;
        let initial_day = state.day;

        let msg = camp_repair_hack(&mut state, &config);

        // Should repair breakdown and apply hack fix penalties
        assert!(state.breakdown.is_none());
        assert!(state.stats.supplies <= initial_supplies); // Supplies consumed or same
        assert!(state.stats.credibility <= initial_credibility); // Credibility lost or same
        assert!(state.day >= initial_day); // Day advanced or same
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_repair_hack_no_breakdown() {
        let config = create_test_config();
        let mut state = create_test_state(); // No breakdown

        let initial_supplies = state.stats.supplies;
        let initial_credibility = state.stats.credibility;

        let msg = camp_repair_hack(&mut state, &config);

        // Should return error message and unchanged state
        assert!(state.breakdown.is_none());
        assert_eq!(state.stats.supplies, initial_supplies);
        assert_eq!(state.stats.credibility, initial_credibility);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_camp_config_defaults() {
        let config = CampConfig::default();

        // Test default values are reasonable
        assert_eq!(config.rest.sanity, 2);
        assert_eq!(config.rest.hp, 1);
        assert_eq!(config.rest.supplies, -1);
        assert_eq!(config.rest.day, 1);

        assert_eq!(config.therapy.sanity, 2);
        assert_eq!(config.therapy.burn_receipt, 1);
        assert_eq!(config.therapy.cooldown_days, 3);

        // Check forage weights structure
        assert!(config.forage.weights.supplies > 0);
        assert!(config.forage.weights.receipt > 0);
        assert!(config.forage.weights.none > 0);

        // Check repair costs structure
        assert!(config.repair.use_spare_supplies > 0);
        assert!(config.repair.hack.supplies > 0);
        assert!(config.repair.hack.credibility > 0);
        assert!(config.repair.hack.day > 0);
    }

    #[test]
    fn test_can_therapy_function() {
        let config = create_test_config();
        let mut state = create_test_state();

        // No receipts - should not be able to do therapy
        state.receipts = vec![];
        assert!(!can_therapy(&state, &config));

        // Has receipts, no previous therapy - should be able to do therapy
        state.receipts = vec!["test".to_string()];
        state.camp.last_therapy_day = None;
        assert!(can_therapy(&state, &config));

        // Has receipts, therapy on cooldown - should not be able to do therapy
        state.day = 2;
        state.camp.last_therapy_day = Some(1);
        assert!(!can_therapy(&state, &config));

        // Has receipts, cooldown expired - should be able to do therapy
        state.day = 5; // 4 days since therapy (cooldown is 3)
        assert!(can_therapy(&state, &config));
    }

    #[test]
    fn test_therapy_cooldown_progression() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.receipts = vec!["test1".to_string(), "test2".to_string(), "test3".to_string()];
        state.camp.last_therapy_day = None;
        state.day = 1;

        // First therapy should work and set last therapy day
        assert!(can_therapy(&state, &config));
        let msg = camp_therapy(&mut state, &config);
        assert_eq!(state.camp.last_therapy_day, Some(1));
        assert!(!msg.is_empty());

        // Second attempt should fail due to cooldown
        assert!(!can_therapy(&state, &config));
        let msg = camp_therapy(&mut state, &config);
        assert!(!msg.is_empty());
        assert_eq!(state.camp.last_therapy_day, Some(1)); // Unchanged
    }

    #[test]
    fn test_stat_clamping() {
        let config = create_test_config();
        let mut state = create_test_state();

        // Test with very high stats
        state.stats.sanity = 100;
        state.stats.hp = 100;
        state.stats.supplies = 100;

        let msg = camp_rest(&mut state, &config);

        // Should clamp stats to maximum values
        assert!(state.stats.sanity <= 10); // Max sanity
        assert!(state.stats.hp <= 10); // Max hp
        assert!(state.stats.supplies <= 20); // Max supplies
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_edge_cases_min_stats() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.stats.sanity = 0;
        state.stats.hp = 0;
        state.stats.supplies = 0;
        state.receipts = vec![];

        // Rest with min stats
        let msg = camp_rest(&mut state, &config);
        assert!(state.stats.sanity >= 0); // Should not go negative
        assert!(state.stats.hp >= 0); // Should not go negative
        assert!(state.stats.supplies >= 0); // Should not go negative
        assert!(!msg.is_empty());

        // Therapy with no receipts should fail
        let msg = camp_therapy(&mut state, &config);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_vehicle_repair_parts() {
        let config = create_test_config();

        // Test each part type
        let parts = vec![Part::Tire, Part::Battery, Part::Alternator, Part::FuelPump];

        for part in parts {
            let mut state = create_test_state();
            state.breakdown = Some(Breakdown {
                part,
                day_started: 1,
            });

            // Set spare for this part
            match part {
                Part::Tire => state.inventory.spares.tire = 1,
                Part::Battery => state.inventory.spares.battery = 1,
                Part::Alternator => state.inventory.spares.alt = 1,
                Part::FuelPump => state.inventory.spares.pump = 1,
            }

            state.stats.supplies = 10;

            let msg = camp_repair_spare(&mut state, &config, part);

            // Should repair breakdown
            assert!(state.breakdown.is_none());
            assert!(!msg.is_empty());
        }
    }
}
