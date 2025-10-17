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
}

impl Default for BossConfig {
    fn default() -> Self {
        serde_json::from_str(DEFAULT_BOSS_DATA).unwrap_or(BossConfig {
            distance_required: ROUTE_LEN_MILES,
            rounds: 3,
            passes_required: 2,
            sanity_loss_per_round: 2,
            pants_gain_per_round: 4,
            base_victory_chance: 0.11,
            credibility_weight: 0.012,
            sanity_weight: 0.01,
            supplies_weight: 0.004,
            allies_weight: 0.015,
            pants_penalty_weight: 0.005,
            min_chance: 0.08,
            max_chance: 0.65,
        })
    }
}

impl BossConfig {
    #[must_use]
    pub fn load_from_static() -> Self {
        BossConfig::default()
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
    if win_prob <= 0.0 {
        win_prob = 0.05;
    }
    if state.mode.is_deep() && matches!(state.policy, Some(PolicyKind::Aggressive)) {
        let boosted = win_prob + 0.02;
        let cap = f64::from(cfg.max_chance).max(win_prob);
        win_prob = boosted.min(cap);
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
