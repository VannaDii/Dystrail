//! Boss fight system
use crate::state::GameState;
use serde::{Deserialize, Serialize};

const DEFAULT_BOSS_DATA: &str = include_str!("../../dystrail-web/static/assets/data/boss.json");

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
            distance_required: 2_100.0,
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

    let mut successes: u32 = 0;
    let required_successes = cfg.passes_required.max(1);

    for _ in 0..cfg.rounds {
        let mut chance = f64::from(cfg.base_victory_chance);
        chance += f64::from(state.stats.credibility) * f64::from(cfg.credibility_weight);
        chance += f64::from(state.stats.sanity) * f64::from(cfg.sanity_weight);
        chance += f64::from(state.stats.supplies) * f64::from(cfg.supplies_weight);
        chance += f64::from(state.stats.allies) * f64::from(cfg.allies_weight);
        chance -= f64::from(state.stats.pants) * f64::from(cfg.pants_penalty_weight);
        chance = chance.clamp(f64::from(cfg.min_chance), f64::from(cfg.max_chance));

        let roll = f64::from(state.next_pct()) / 100.0;
        if roll < chance {
            successes += 1;
        }

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

    if successes >= required_successes {
        state.boss_victory = true;
        state.logs.push(String::from("log.boss.victory"));
        BossOutcome::PassedCloture
    } else {
        state.logs.push(String::from("log.boss.failure"));
        BossOutcome::SurvivedFlood
    }
}
