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
    #[serde(default = "EndgamePolicyCfg::default_travel_bias")]
    pub travel_bias: f32,
    #[serde(default = "EndgamePolicyCfg::default_stop_cap_window")]
    pub stop_cap_window: u8,
    #[serde(default = "EndgamePolicyCfg::default_stop_cap_max_full")]
    pub stop_cap_max_full: u8,
    #[serde(default = "EndgamePolicyCfg::default_breakdown_scale")]
    pub breakdown_scale: f32,
    #[serde(default = "EndgamePolicyCfg::default_wear_shave_ratio")]
    pub wear_shave_ratio: f32,
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
            travel_bias: Self::default_travel_bias(),
            stop_cap_window: Self::default_stop_cap_window(),
            stop_cap_max_full: Self::default_stop_cap_max_full(),
            breakdown_scale: Self::default_breakdown_scale(),
            wear_shave_ratio: Self::default_wear_shave_ratio(),
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

    const fn default_travel_bias() -> f32 {
        1.06
    }

    const fn default_stop_cap_window() -> u8 {
        10
    }

    const fn default_stop_cap_max_full() -> u8 {
        2
    }

    const fn default_breakdown_scale() -> f32 {
        1.0
    }

    const fn default_wear_shave_ratio() -> f32 {
        0.7
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
    #[serde(default)]
    pub resource_priority: Vec<ResourceKind>,
    #[serde(default = "EndgameState::default_travel_bias")]
    pub travel_bias: f32,
    #[serde(default = "EndgameState::default_stop_cap_window")]
    pub stop_cap_window: u8,
    #[serde(default = "EndgameState::default_stop_cap_max_full")]
    pub stop_cap_max_full: u8,
    #[serde(default = "EndgameState::default_breakdown_scale")]
    pub breakdown_scale: f32,
    #[serde(default = "EndgameState::default_wear_shave_ratio")]
    pub wear_shave_ratio: f32,
    #[serde(default)]
    pub wear_reset_used: bool,
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
            resource_priority: Vec::new(),
            travel_bias: Self::default_travel_bias(),
            stop_cap_window: EndgamePolicyCfg::default_stop_cap_window(),
            stop_cap_max_full: EndgamePolicyCfg::default_stop_cap_max_full(),
            breakdown_scale: Self::default_breakdown_scale(),
            wear_shave_ratio: Self::default_wear_shave_ratio(),
            wear_reset_used: false,
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

    const fn default_travel_bias() -> f32 {
        1.0
    }

    const fn default_stop_cap_window() -> u8 {
        10
    }

    const fn default_stop_cap_max_full() -> u8 {
        2
    }

    const fn default_breakdown_scale() -> f32 {
        1.0
    }

    const fn default_wear_shave_ratio() -> f32 {
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
        self.resource_priority.clone_from(&policy.resource_priority);
        self.travel_bias = policy.travel_bias.max(1.0);
        self.stop_cap_window = policy.stop_cap_window.max(1);
        self.stop_cap_max_full = policy.stop_cap_max_full;
        self.breakdown_scale = policy.breakdown_scale.clamp(0.0, 1.0);
        self.wear_shave_ratio = policy.wear_shave_ratio.clamp(0.0, 1.0);
        self.wear_reset_used = false;
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

    let policy_key = policy_key_for_mode(state.policy);
    let Some(policy_cfg) = policy_key.and_then(|key| cfg.policy(key)) else {
        return;
    };

    if !state.endgame.active && state.miles_traveled_actual >= policy_cfg.mi_start {
        state
            .endgame
            .configure(policy_key.unwrap_or_default(), policy_cfg);
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

    if breakdown_started
        && !state.endgame.wear_reset_used
        && policy_cfg.wear_reset > 0.0
        && state.endgame.active
    {
        apply_vehicle_stabilizers(state, 0.0, policy_cfg.wear_reset);
        state.endgame.wear_reset_used = true;
        state.logs.push(String::from("log.endgame.wear_reset"));
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
        Some(PolicyKind::Conservative) => Some("deep_conservative"),
        Some(PolicyKind::ResourceManager) => Some("deep_resource_manager"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vehicle::Part;
    use crate::{Breakdown, GameMode, PolicyKind};

    #[test]
    fn config_defaults_load() {
        let cfg = EndgameTravelCfg::default_config();
        assert!(cfg.policies.contains_key("deep_balanced"));
        assert!(cfg.policies.contains_key("deep_aggressive"));
    }

    #[test]
    fn field_repair_consumes_resources() {
        #![allow(clippy::field_reassign_with_default)]
        let cfg = EndgamePolicyCfg {
            health_floor: 60.0,
            wear_reset: 5.0,
            cooldown_days: 2,
            partial_ratio: 0.5,
            wear_multiplier: 0.9,
            resource_priority: vec![ResourceKind::MatchingSpare, ResourceKind::Emergency],
            ..EndgamePolicyCfg::default()
        };
        let mut state = GameState::default();
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.last_breakdown_part = Some(Part::Tire);
        state.inventory.spares.tire = 1;
        state.budget_cents = 10_000;
        state.endgame.partial_ratio = 0.5;
        run_field_repair(&mut state, &cfg, 12.0);
        assert!(state.breakdown.is_none());
        assert!(state.endgame.field_repair_used);
        assert!(state.logs.iter().any(|log| log == LOG_ENDGAME_FIELD_REPAIR));
    }

    #[test]
    fn one_time_wear_reset_applies_once() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Balanced);
        state.miles_traveled_actual = 1_900.0;
        state.vehicle.set_wear(12.0);

        let mut cfg = EndgameTravelCfg::default_config();
        let entry = cfg
            .policies
            .get_mut("deep_balanced")
            .expect("policy exists");
        entry.wear_reset = 1.5;
        entry.breakdown_scale = 1.0;
        entry.wear_shave_ratio = 1.0;

        // Activate endgame
        run_endgame_controller(&mut state, 0.0, false, &cfg);
        assert!(state.endgame.active);

        // First breakdown triggers reset
        state.vehicle.set_wear(10.0);
        run_endgame_controller(&mut state, 0.0, true, &cfg);
        assert!(state.endgame.wear_reset_used);
        assert!(state.vehicle.wear <= 1.5);

        // Subsequent breakdowns do not reset again
        state.vehicle.set_wear(9.0);
        run_endgame_controller(&mut state, 0.0, true, &cfg);
        assert!(state.vehicle.wear >= 9.0 - f32::EPSILON);
    }

    #[test]
    fn policy_key_resolves() {
        assert_eq!(
            policy_key_for_mode(Some(PolicyKind::Balanced)),
            Some("deep_balanced")
        );
        assert_eq!(
            policy_key_for_mode(Some(PolicyKind::Aggressive)),
            Some("deep_aggressive")
        );
        assert!(policy_key_for_mode(None).is_none());
    }
}
