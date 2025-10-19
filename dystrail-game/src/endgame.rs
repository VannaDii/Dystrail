//! Endgame travel controller tuning and runtime logic.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter;

use crate::constants::{
    EMERGENCY_REPAIR_COST, LOG_ENDGAME_ACTIVATE, LOG_ENDGAME_FAILURE_GUARD,
    LOG_ENDGAME_FIELD_REPAIR, TRAVEL_PARTIAL_MIN_DISTANCE, TRAVEL_PARTIAL_RATIO,
};
use crate::{
    TravelDayKind,
    state::{GameState, PolicyKind},
};

const DEFAULT_ENDGAME_DATA: &str =
    include_str!("../../dystrail-web/static/assets/data/endgame.json");

/// Configuration bundle for the endgame travel controller.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndgameTravelCfg {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub policies: HashMap<String, EndgamePolicyCfg>,
}

impl EndgameTravelCfg {
    #[must_use]
    pub fn load_from_static() -> Self {
        serde_json::from_str(DEFAULT_ENDGAME_DATA).unwrap_or_default()
    }

    #[must_use]
    pub fn default_config() -> Self {
        Self::load_from_static()
    }

    #[must_use]
    pub fn policy(&self, key: &str) -> Option<&EndgamePolicyCfg> {
        self.policies.get(key)
    }
}

/// Resource order for automatic field repairs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    MatchingSpare,
    AnySpare,
    Emergency,
}

/// Policy-specific tuning for endgame behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndgamePolicyCfg {
    #[serde(default)]
    pub mi_start: f32,
    #[serde(default = "EndgamePolicyCfg::default_failure_guard_miles")]
    pub failure_guard_miles: f32,
    #[serde(default)]
    pub health_floor: f32,
    #[serde(default)]
    pub wear_reset: f32,
    #[serde(default)]
    pub cooldown_days: u32,
    #[serde(default = "EndgamePolicyCfg::default_partial_ratio")]
    pub partial_ratio: f32,
    #[serde(default = "EndgamePolicyCfg::default_wear_multiplier")]
    pub wear_multiplier: f32,
    #[serde(default = "EndgamePolicyCfg::default_resource_priority")]
    pub resource_priority: Vec<ResourceKind>,
}

impl Default for EndgamePolicyCfg {
    fn default() -> Self {
        Self {
            mi_start: 1_850.0,
            failure_guard_miles: Self::default_failure_guard_miles(),
            health_floor: 40.0,
            wear_reset: 0.0,
            cooldown_days: 3,
            partial_ratio: Self::default_partial_ratio(),
            wear_multiplier: Self::default_wear_multiplier(),
            resource_priority: Self::default_resource_priority(),
        }
    }
}

impl EndgamePolicyCfg {
    const fn default_failure_guard_miles() -> f32 {
        1_950.0
    }

    const fn default_partial_ratio() -> f32 {
        TRAVEL_PARTIAL_RATIO
    }

    const fn default_wear_multiplier() -> f32 {
        1.0
    }

    fn default_resource_priority() -> Vec<ResourceKind> {
        vec![
            ResourceKind::MatchingSpare,
            ResourceKind::AnySpare,
            ResourceKind::Emergency,
        ]
    }
}

/// Runtime guard tracking for endgame features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndgameState {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub field_repair_used: bool,
    #[serde(default)]
    pub last_limp_mile: f32,
    #[serde(default = "EndgameState::default_failure_guard_miles")]
    pub failure_guard_miles: f32,
    #[serde(default)]
    pub health_floor: f32,
    #[serde(default)]
    pub wear_reset: f32,
    #[serde(default)]
    pub cooldown_days: u32,
    #[serde(default = "EndgameState::default_partial_ratio")]
    pub partial_ratio: f32,
    #[serde(default = "EndgameState::default_wear_multiplier")]
    pub wear_multiplier: f32,
    #[serde(default)]
    pub policy_key: String,
}

impl Default for EndgameState {
    fn default() -> Self {
        Self {
            active: false,
            field_repair_used: false,
            last_limp_mile: 0.0,
            failure_guard_miles: Self::default_failure_guard_miles(),
            health_floor: 0.0,
            wear_reset: 0.0,
            cooldown_days: 0,
            partial_ratio: Self::default_partial_ratio(),
            wear_multiplier: Self::default_wear_multiplier(),
            policy_key: String::new(),
        }
    }
}

impl EndgameState {
    const fn default_failure_guard_miles() -> f32 {
        1_950.0
    }

    const fn default_partial_ratio() -> f32 {
        0.5
    }

    const fn default_wear_multiplier() -> f32 {
        1.0
    }

    pub fn configure(&mut self, key: &str, policy: &EndgamePolicyCfg) {
        self.active = true;
        self.field_repair_used = false;
        self.last_limp_mile = 0.0;
        self.failure_guard_miles = policy.failure_guard_miles;
        self.health_floor = policy.health_floor;
        self.wear_reset = policy.wear_reset;
        self.cooldown_days = policy.cooldown_days;
        self.partial_ratio = policy.partial_ratio;
        self.wear_multiplier = policy.wear_multiplier;
        self.policy_key = key.to_string();
    }
}

/// Primary endgame controller invoked during the day loop.
pub fn run_endgame_controller(
    state: &mut GameState,
    computed_miles_today: f32,
    breakdown_started: bool,
    cfg: &EndgameTravelCfg,
) {
    if !cfg.enabled || !state.mode.is_deep() {
        return;
    }

    let policy_key = match state.policy {
        Some(PolicyKind::Balanced) => "deep_balanced",
        Some(PolicyKind::Aggressive) => "deep_aggressive",
        _ => return,
    };
    let Some(policy_cfg) = cfg.policy(policy_key) else {
        return;
    };

    if !state.endgame.active && state.miles_traveled_actual >= policy_cfg.mi_start {
        state.endgame.configure(policy_key, policy_cfg);
        state.add_day_reason_tag("endgame_activate");
        state.logs.push(String::from(LOG_ENDGAME_ACTIVATE));
    }

    if !state.endgame.active {
        return;
    }

    state.add_day_reason_tag("endgame_active");

    if state.vehicle.breakdown_suppressed() {
        state.add_day_reason_tag("endgame_cooldown");
    }

    if breakdown_started && !state.endgame.field_repair_used {
        run_field_repair(state, policy_cfg, computed_miles_today);
    }
}

/// Prevent terminal vehicle failures before the configured mileage guard.
pub fn enforce_failure_guard(state: &mut GameState) -> bool {
    if !state.endgame.active {
        return false;
    }
    if state.miles_traveled_actual >= state.endgame.failure_guard_miles {
        return false;
    }
    if state.vehicle.health > 0.0 {
        return false;
    }

    apply_vehicle_stabilizers(state, state.endgame.health_floor, state.endgame.wear_reset);
    if state.endgame.cooldown_days > 0 {
        state
            .vehicle
            .set_breakdown_cooldown(state.endgame.cooldown_days);
    }
    if state.endgame.wear_multiplier >= 0.0 {
        state
            .vehicle
            .set_wear_multiplier(state.endgame.wear_multiplier);
    }
    state.logs.push(String::from(LOG_ENDGAME_FAILURE_GUARD));
    state.add_day_reason_tag("endgame_guard");
    state.rest_requested = true;
    true
}

fn run_field_repair(
    state: &mut GameState,
    policy_cfg: &EndgamePolicyCfg,
    computed_miles_today: f32,
) {
    let last_part = state.last_breakdown_part;
    let priority_iter = if policy_cfg.resource_priority.is_empty() {
        EndgamePolicyCfg::default_resource_priority()
    } else {
        policy_cfg.resource_priority.clone()
    };
    for resource in priority_iter
        .into_iter()
        .chain(iter::once(ResourceKind::Emergency))
    {
        match resource {
            ResourceKind::MatchingSpare => {
                if let Some(part) = last_part
                    && state.consume_spare_for_part(part)
                {
                    break;
                }
            }
            ResourceKind::AnySpare => {
                if state.consume_any_spare_for_emergency() {
                    break;
                }
            }
            ResourceKind::Emergency => {
                if state.budget_cents >= EMERGENCY_REPAIR_COST {
                    state.spend_emergency_repair(LOG_ENDGAME_FIELD_REPAIR);
                    break;
                }
            }
        }
    }

    apply_vehicle_stabilizers(state, policy_cfg.health_floor, policy_cfg.wear_reset);
    if policy_cfg.cooldown_days > 0 {
        state
            .vehicle
            .set_breakdown_cooldown(policy_cfg.cooldown_days);
    }
    if policy_cfg.wear_multiplier >= 0.0 {
        state
            .vehicle
            .set_wear_multiplier(policy_cfg.wear_multiplier);
    }

    state.breakdown = None;
    state.travel_blocked = false;
    state.last_breakdown_part = None;
    state.endgame.field_repair_used = true;
    state.add_day_reason_tag("field_repair");
    state.logs.push(String::from(LOG_ENDGAME_FIELD_REPAIR));

    let ratio = state.endgame.partial_ratio.clamp(0.0, 1.0);
    let mut partial = (computed_miles_today * ratio).clamp(0.0, computed_miles_today);
    partial = partial.max(TRAVEL_PARTIAL_MIN_DISTANCE.min(computed_miles_today.max(0.0)));

    state.reset_today_progress();
    state.record_travel_day(TravelDayKind::Partial, partial, "field_repair");
    state.distance_today = partial;
    state.distance_today_raw = partial;
    state.partial_distance_today = partial;
    state.current_day_miles = partial;
    state.partial_traveled_today = true;
    state.traveled_today = false;
    state.stats.clamp();
}

fn apply_vehicle_stabilizers(state: &mut GameState, health_floor: f32, wear_reset: f32) {
    if health_floor > 0.0 {
        state.vehicle.ensure_health_floor(health_floor);
    }
    if wear_reset <= 0.0 {
        state.vehicle.reset_wear();
    } else {
        state.vehicle.set_wear(wear_reset);
    }
}

/// Helper mapping policy kinds to config keys.
#[must_use]
pub const fn policy_key_for_mode(policy: Option<PolicyKind>) -> Option<&'static str> {
    match policy {
        Some(PolicyKind::Balanced) => Some("deep_balanced"),
        Some(PolicyKind::Aggressive) => Some("deep_aggressive"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_load() {
        let cfg = EndgameTravelCfg::default_config();
        assert!(cfg.policies.contains_key("deep_balanced"));
        assert!(cfg.policies.contains_key("deep_aggressive"));
    }
}
