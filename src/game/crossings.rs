use crate::game::state::GameState;
use crate::game::weather::Weather;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossingKind {
    #[serde(rename = "checkpoint")]
    Checkpoint,
    #[serde(rename = "bridge_out")]
    BridgeOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermitCfg {
    pub cred_gain: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialDetour {
    #[serde(default)]
    pub days: Option<i32>,
    #[serde(default)]
    pub pants: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub weather: HashMap<Weather, WeatherDetourMod>,
    #[serde(default)]
    pub exec_orders: HashMap<String, ExecBribeMod>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoneyCfg {
    pub currency: String,
    pub allow_negative_budget: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossingConfig {
    pub types: HashMap<CrossingKind, CrossingTypeCfg>,
    pub global_mods: GlobalMods,
    pub money: MoneyCfg,
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
            Weather::Storm,
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
                allow_negative_budget: true,
            },
        }
    }
}

impl CrossingConfig {
    pub async fn load_from_static() -> Self {
        let url = "/static/assets/data/crossings.json";
        if let Ok(resp) = gloo_net::http::Request::get(url).send().await
            && resp.status() == 200
            && let Ok(config) = resp.json::<CrossingConfig>().await
        {
            return config;
        }

        // Log warning and return defaults
        gloo::console::warn!("Failed to load crossings.json, using embedded defaults");
        Self::default()
    }
}

/// Calculate effective bribe cost with persona discount, rounded up
#[must_use]
pub fn calculate_bribe_cost(base_cost_cents: i64, discount_pct: i32) -> i64 {
    if discount_pct <= 0 {
        return base_cost_cents;
    }

    // Use integer arithmetic to avoid precision loss
    let discount_amount = (base_cost_cents * i64::from(discount_pct)) / 100;
    base_cost_cents - discount_amount
}

/// Get active exec order bribe modifier if any
fn get_active_exec_bribe_mod<'a>(
    gs: &GameState,
    mods: &'a HashMap<String, ExecBribeMod>,
) -> Option<&'a ExecBribeMod> {
    let order_name = format!("{:?}", gs.current_order);
    mods.get(&order_name)
}

/// Apply detour option with weather modifiers
pub fn apply_detour(gs: &mut GameState, cfg: &CrossingConfig, kind: CrossingKind) -> String {
    let base = match cfg.types.get(&kind) {
        Some(type_cfg) => &type_cfg.detour,
        None => return format!("Configuration error: Unknown crossing type {kind:?}"),
    };
    let mut days = base.days;
    let mut pants = base.pants;

    // Apply weather modifiers if any
    if let Some(weather_mod) = cfg.global_mods.weather.get(&gs.weather_state.today) {
        if let Some(extra_days) = weather_mod.detour.days {
            days += extra_days;
        }
        if let Some(extra_pants) = weather_mod.detour.pants {
            pants += extra_pants;
        }
    }

    // Apply effects to game state
    if days > 0 {
        #[allow(clippy::cast_sign_loss)]
        {
            gs.day += days as u32;
        }
    }
    gs.stats.supplies += base.supplies;
    gs.stats.pants = (gs.stats.pants + pants).clamp(0, 100);

    // Clamp stats
    gs.stats.clamp();

    // Return formatted announcement
    let days_str = if days >= 0 {
        format!("+{days}")
    } else {
        days.to_string()
    };
    let supplies_str = if base.supplies >= 0 {
        format!("+{}", base.supplies)
    } else {
        base.supplies.to_string()
    };
    let pants_str = if pants >= 0 {
        format!("+{pants}")
    } else {
        pants.to_string()
    };

    let mut args = std::collections::HashMap::new();
    args.insert("days", days_str.as_str());
    args.insert("supplies", supplies_str.as_str());
    args.insert("pants", pants_str.as_str());
    crate::i18n::tr("cross.announce.detour_applied", Some(&args))
}

/// Apply bribe option with persona discount and exec order modifiers
pub fn apply_bribe(gs: &mut GameState, cfg: &CrossingConfig, kind: CrossingKind) -> String {
    let bribe_cfg = match cfg.types.get(&kind) {
        Some(type_cfg) => &type_cfg.bribe,
        None => return format!("Configuration error: Unknown crossing type {kind:?}"),
    };
    let discount_pct = gs.mods.bribe_discount_pct;
    let cost_cents = calculate_bribe_cost(bribe_cfg.base_cost_cents, discount_pct);

    // Check budget if configured to do so
    if !cfg.money.allow_negative_budget && cost_cents > gs.budget_cents {
        return crate::i18n::tr("cross.announce.insufficient_funds", None);
    }

    // Deduct cost
    gs.budget_cents -= cost_cents;
    #[allow(clippy::cast_possible_truncation)]
    {
        gs.budget = (gs.budget_cents / 100) as i32;
    }

    // Determine success chance
    let mut success_chance = bribe_cfg.success_chance;
    let fail_cfg =
        if let Some(exec_mod) = get_active_exec_bribe_mod(gs, &cfg.global_mods.exec_orders) {
            success_chance = exec_mod.bribe_success_chance;
            &exec_mod.on_fail
        } else {
            &bribe_cfg.on_fail
        };

    // Roll for success
    let success = if let Some(ref mut rng) = gs.rng {
        let roll: f32 = rng.random();
        roll < success_chance
    } else {
        // Fallback if no RNG (shouldn't happen in normal gameplay)
        success_chance >= 1.0
    };

    // Format cost for display
    #[allow(clippy::cast_precision_loss)]
    let cost_display = format!("${:.2}", cost_cents as f64 / 100.0);
    let mut args = std::collections::HashMap::new();
    args.insert("cost", cost_display.as_str());

    if success {
        crate::i18n::tr("cross.announce.bribe_paid_passed", Some(&args))
    } else {
        // Apply failure penalties
        if fail_cfg.days > 0 {
            #[allow(clippy::cast_sign_loss)]
            {
                gs.day += fail_cfg.days as u32;
            }
        }
        gs.stats.pants = (gs.stats.pants + fail_cfg.pants).clamp(0, 100);
        gs.stats.clamp();

        let days_str = if fail_cfg.days >= 0 {
            format!("+{}", fail_cfg.days)
        } else {
            fail_cfg.days.to_string()
        };
        let pants_str = if fail_cfg.pants >= 0 {
            format!("+{}", fail_cfg.pants)
        } else {
            fail_cfg.pants.to_string()
        };

        args.insert("days", days_str.as_str());
        args.insert("pants", pants_str.as_str());
        crate::i18n::tr("cross.announce.bribe_paid_failed", Some(&args))
    }
}

/// Apply permit option with exec order considerations
pub fn apply_permit(gs: &mut GameState, cfg: &CrossingConfig, kind: CrossingKind) -> String {
    let permit_cfg = match cfg.types.get(&kind) {
        Some(type_cfg) => &type_cfg.permit,
        None => return format!("Configuration error: Unknown crossing type {kind:?}"),
    };

    // For permits, we just gain credibility and assume success
    gs.stats.credibility += permit_cfg.cred_gain;

    let mut args = std::collections::HashMap::new();
    let cred_str = permit_cfg.cred_gain.to_string();
    args.insert("cred", cred_str.as_str());

    crate::i18n::tr("cross.announce.permit_success", Some(&args))
}

/// Check if permit option is available
#[must_use]
pub fn can_use_permit(gs: &GameState) -> bool {
    !gs.receipts.is_empty()
        || gs.inventory.tags.contains("permit")
        || gs.inventory.tags.contains("press_pass")
}

/// Check if bribe option is available (based on budget rules)
#[must_use]
pub fn can_afford_bribe(gs: &GameState, cfg: &CrossingConfig, kind: CrossingKind) -> bool {
    if cfg.money.allow_negative_budget {
        return true;
    }

    let bribe_cfg = match cfg.types.get(&kind) {
        Some(type_cfg) => &type_cfg.bribe,
        None => return false, // Can't afford if configuration is missing
    };
    let cost_cents = calculate_bribe_cost(bribe_cfg.base_cost_cents, gs.mods.bribe_discount_pct);
    gs.budget_cents >= cost_cents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bribe_cost() {
        // No discount
        assert_eq!(calculate_bribe_cost(1000, 0), 1000);

        // 10% discount on $10.00 = $9.00
        assert_eq!(calculate_bribe_cost(1000, 10), 900);

        // 10% discount on $10.01 should round up to $9.01
        assert_eq!(calculate_bribe_cost(1001, 10), 901);

        // 20% discount on $25.00 = $20.00
        assert_eq!(calculate_bribe_cost(2500, 20), 2000);
    }

    #[test]
    fn test_default_config() {
        let cfg = CrossingConfig::default();

        // Should have both crossing types
        assert!(cfg.types.contains_key(&CrossingKind::Checkpoint));
        assert!(cfg.types.contains_key(&CrossingKind::BridgeOut));

        // Should allow negative budget by default
        assert!(cfg.money.allow_negative_budget);

        // Should have weather and exec order modifiers
        assert!(cfg.global_mods.weather.contains_key(&Weather::Storm));
        assert!(cfg.global_mods.exec_orders.contains_key("Shutdown"));
    }

    #[test]
    fn test_bribe_cost_calculation() {
        let cfg = CrossingConfig::default();
        let gs = GameState::default();

        // Test basic cost calculation with no discount
        assert!(can_afford_bribe(&gs, &cfg, CrossingKind::Checkpoint));

        // Test with insufficient funds
        let mut poor_gs = gs.clone();
        poor_gs.budget_cents = 500; // Less than $10 bribe cost

        let mut strict_cfg = cfg.clone();
        strict_cfg.money.allow_negative_budget = false;
        assert!(!can_afford_bribe(
            &poor_gs,
            &strict_cfg,
            CrossingKind::Checkpoint
        ));
    }

    #[test]
    fn test_permit_priority() {
        let cfg = CrossingConfig::default();
        let mut gs = GameState::default();

        // Add both a receipt and a permit tag
        gs.receipts.push("test_receipt".to_string());
        gs.inventory.tags.insert("press_pass".to_string());

        let initial_receipts = gs.receipts.len();

        // Should use tag and not consume receipt
        let result = apply_permit(&mut gs, &cfg, CrossingKind::Checkpoint);
        assert!(!result.is_empty());
        assert_eq!(gs.receipts.len(), initial_receipts);
        assert_eq!(gs.stats.credibility, 6); // Default 5 + 1 from permit
    }

    #[test]
    fn test_detour_weather_modifier() {
        let cfg = CrossingConfig::default();
        let mut gs = GameState::default();

        // Set storm weather
        gs.weather_state.today = Weather::Storm;

        let initial_day = gs.day;
        let initial_pants = gs.stats.pants;

        // Call apply_detour and just check that it returns a string (i18n might not work in tests)
        let result = apply_detour(&mut gs, &cfg, CrossingKind::Checkpoint);
        assert!(!result.is_empty());

        // Should have base detour (2 days, 1 pants) + storm modifier (1 day, 1 pants)
        assert_eq!(gs.day, initial_day + 3); // 2 + 1 from storm
        assert_eq!(gs.stats.pants, initial_pants + 2); // 1 + 1 from storm
    }
}
