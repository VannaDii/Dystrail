//! Boss fight system
use crate::state::{GameState, PolicyKind};
use serde::{Deserialize, Serialize};

const DEFAULT_BOSS_DATA: &str = include_str!("../../dystrail-web/static/assets/data/boss.json");

/// Canonical trail length in miles, sourced from `boss.json`.
pub const ROUTE_LEN_MILES: f32 = 2_100.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossOutcome {
    PassedCloture,
    SurvivedFlood,
    PantsEmergency,
    Exhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boss {
    pub name: String,
    pub hp: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BalancedBossBias {
    #[serde(default = "BalancedBossBias::default_classic_bonus")]
    pub classic_bonus: f32,
    #[serde(default = "BalancedBossBias::default_deep_multiplier")]
    pub deep_multiplier: f32,
    #[serde(default = "BalancedBossBias::default_deep_bonus")]
    pub deep_bonus: f32,
}

impl BalancedBossBias {
    const fn default_classic_bonus() -> f32 {
        0.30
    }

    const fn default_deep_multiplier() -> f32 {
        1.1
    }

    const fn default_deep_bonus() -> f32 {
        0.08
    }
}

impl Default for BalancedBossBias {
    fn default() -> Self {
        Self {
            classic_bonus: Self::default_classic_bonus(),
            deep_multiplier: Self::default_deep_multiplier(),
            deep_bonus: Self::default_deep_bonus(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BossConfig {
    pub distance_required: f32,
    pub rounds: u32,
    pub passes_required: u32,
    pub sanity_loss_per_round: i32,
    pub pants_gain_per_round: i32,
    pub base_victory_chance: f32,
    pub credibility_weight: f32,
    pub sanity_weight: f32,
    pub supplies_weight: f32,
    pub allies_weight: f32,
    pub pants_penalty_weight: f32,
    pub min_chance: f32,
    pub max_chance: f32,
    #[serde(default)]
    pub balanced: BalancedBossBias,
}

impl Default for BossConfig {
    fn default() -> Self {
        serde_json::from_str(DEFAULT_BOSS_DATA).unwrap_or_else(|_| Self {
            distance_required: ROUTE_LEN_MILES,
            rounds: 3,
            passes_required: 2,
            sanity_loss_per_round: 2,
            pants_gain_per_round: 3,
            base_victory_chance: 0.18,
            credibility_weight: 0.012,
            sanity_weight: 0.01,
            supplies_weight: 0.004,
            allies_weight: 0.015,
            pants_penalty_weight: 0.005,
            min_chance: 0.25,
            max_chance: 0.88,
            balanced: BalancedBossBias::default(),
        })
    }
}

impl BossConfig {
    #[must_use]
    pub fn load_from_static() -> Self {
        Self::default()
    }
}

pub fn run_boss_minigame(state: &mut GameState, cfg: &BossConfig) -> BossOutcome {
    state.boss_attempted = true;

    if state.mode.is_deep() && matches!(state.policy, Some(PolicyKind::Aggressive)) {
        let _ = state.apply_deep_aggressive_compose();
    }

    for _ in 0..cfg.rounds {
        if cfg.pants_gain_per_round > 0 {
            state.stats.pants += cfg.pants_gain_per_round;
        }
        if cfg.sanity_loss_per_round > 0 {
            state.stats.sanity -= cfg.sanity_loss_per_round;
        }
        state.stats.clamp();
        if state.stats.pants >= 100 {
            return BossOutcome::PantsEmergency;
        }
        if state.stats.sanity <= 0 {
            return BossOutcome::Exhausted;
        }
    }

    let distance_required =
        f64::from(cfg.distance_required).max(f64::from(state.mode.boss_threshold()));
    let threshold = distance_required.max(1.0);
    let score = state.journey_score().max(0);
    let win_ratio = (f64::from(score) / threshold).min(1.25);
    let mut win_prob = (win_ratio - 0.5).clamp(0.0, 1.0);
    let base = f64::from(cfg.base_victory_chance).clamp(0.0, 1.0);
    let min_cap = f64::from(cfg.min_chance).clamp(0.0, 1.0);
    let max_cap = f64::from(cfg.max_chance).clamp(min_cap, 1.0);
    win_prob = (win_prob + base).min(max_cap);
    win_prob = win_prob.max(min_cap);
    if matches!(state.policy, Some(PolicyKind::Balanced)) {
        let bias = cfg.balanced;
        if state.mode.is_deep() {
            let deep_mult = f64::from(bias.deep_multiplier).clamp(0.0, 2.0);
            win_prob *= deep_mult;
            win_prob += f64::from(bias.deep_bonus);
        } else {
            win_prob += f64::from(bias.classic_bonus);
        }
        win_prob = win_prob.clamp(min_cap, max_cap);
    }

    if state.mode.is_deep() && matches!(state.policy, Some(PolicyKind::Aggressive)) {
        win_prob = 1.0;
    }

    let roll = f64::from(state.next_pct()) / 100.0;
    if roll < win_prob {
        state.boss_victory = true;
        state.logs.push(String::from("log.boss.victory"));
        BossOutcome::PassedCloture
    } else {
        state.logs.push(String::from("log.boss.failure"));
        BossOutcome::SurvivedFlood
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::EncounterData;
    use crate::journey::RngBundle;
    use crate::state::{GameMode, PolicyKind};
    use std::rc::Rc;

    #[test]
    fn run_boss_probability_branches_cover_edges() {
        let data = EncounterData::empty();

        let mut fail_state = GameState::default().with_seed(0xFACE, GameMode::Deep, data.clone());
        fail_state.stats.supplies = 0;
        fail_state.stats.morale = 0;
        fail_state.stats.credibility = 0;
        fail_state.stats.allies = 0;
        fail_state.stats.sanity = 6;
        fail_state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(17)));
        let mut fail_cfg = BossConfig::load_from_static();
        fail_cfg.rounds = 0;
        fail_cfg.distance_required = 5_000.0;
        fail_cfg.base_victory_chance = 0.0;
        fail_cfg.min_chance = 0.0;
        fail_cfg.max_chance = 0.0;
        let fail_outcome = run_boss_minigame(&mut fail_state, &fail_cfg);
        assert!(matches!(fail_outcome, BossOutcome::SurvivedFlood));

        let mut win_state = GameState::default().with_seed(0xBEEF, GameMode::Deep, data);
        win_state.policy = Some(PolicyKind::Aggressive);
        win_state.stats.supplies = 30;
        win_state.stats.morale = 30;
        win_state.stats.credibility = 20;
        win_state.stats.allies = 10;
        win_state.stats.sanity = 10;
        win_state.encounters_resolved = 60;
        win_state
            .receipts
            .extend(["attestation".into(), "briefing".into()]);
        win_state.miles_traveled_actual = 2_200.0;
        win_state.detach_rng_bundle();
        let mut win_cfg = BossConfig::load_from_static();
        win_cfg.rounds = 0;
        win_cfg.base_victory_chance = 1.0;
        win_cfg.max_chance = 1.0;
        let win_outcome = run_boss_minigame(&mut win_state, &win_cfg);
        assert!(matches!(win_outcome, BossOutcome::PassedCloture));
    }

    #[test]
    fn balanced_biases_load_from_assets() {
        let cfg = BossConfig::load_from_static();
        assert!(
            (cfg.balanced.classic_bonus - 0.30).abs() < f32::EPSILON,
            "expected classic bonus from assets"
        );
        assert!(
            (cfg.balanced.deep_multiplier - 1.1).abs() < f32::EPSILON,
            "expected deep multiplier from assets"
        );
        assert!(
            (cfg.balanced.deep_bonus - 0.08).abs() < f32::EPSILON,
            "expected deep bonus from assets"
        );
    }

    #[test]
    fn balanced_defaults_match_const_fns() {
        let bias = BalancedBossBias::default();
        assert!(
            (bias.classic_bonus - BalancedBossBias::default_classic_bonus()).abs() < f32::EPSILON
        );
        assert!(
            (bias.deep_multiplier - BalancedBossBias::default_deep_multiplier()).abs()
                < f32::EPSILON
        );
        assert!((bias.deep_bonus - BalancedBossBias::default_deep_bonus()).abs() < f32::EPSILON);
    }

    #[test]
    fn default_config_has_minimum_rounds_and_passes() {
        let cfg = BossConfig::default();
        assert!(cfg.rounds >= 1);
        assert!(cfg.passes_required >= 1);
        assert_eq!(cfg.balanced, BalancedBossBias::default());
    }
}
