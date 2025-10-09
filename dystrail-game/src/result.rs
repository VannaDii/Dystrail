//! End game result calculation
use serde::{Deserialize, Serialize};

/// Configuration for the result screen and scoring system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultConfig {
    pub score: ScoreCfg,
    pub multipliers: MultipliersCfg,
    pub endings: EndingCfg,
    pub limits: ResultLimits,
}

/// Scoring algorithm configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreCfg {
    pub pants_threshold: i32,
    pub pants_penalty_per_point: i32,
    pub persona_rounding: Rounding,
    pub final_min: i32,
    pub final_max: i32,
}

/// Rounding behavior for score calculations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rounding {
    /// Round to the nearest integer
    Nearest,
    /// Always round down (floor)
    Down,
    /// Always round up (ceiling)
    Up,
}

/// Multiplier configuration for score bonuses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultipliersCfg {
    pub display_bonus_deep: f32,
}

/// Configuration for determining game endings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndingCfg {
    pub priority: Vec<String>,
    pub victory_key: String,
    pub boss_loss_key: String,
    pub pants_key: String,
    pub sanity_key: String,
    pub collapse_key: String,
}

/// Limits for text generation in results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultLimits {
    pub share_seed_maxlen: usize,
    pub share_persona_maxlen: usize,
}

/// Possible game ending types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ending {
    /// Game ended due to pants meter reaching 100%
    Pants,
    /// Game ended due to sanity reaching 0 or below
    Sanity,
    /// Game ended due to supplies or HP reaching 0 or below
    Collapse,
    /// Game ended due to failing the filibuster boss fight
    BossLoss,
    /// Game completed successfully
    Victory,
}

impl std::fmt::Display for Ending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ending::Pants => write!(f, "pants"),
            Ending::Sanity => write!(f, "sanity"),
            Ending::Collapse => write!(f, "collapse"),
            Ending::BossLoss => write!(f, "boss_loss"),
            Ending::Victory => write!(f, "victory"),
        }
    }
}

/// Complete summary of a game run for display on the result screen
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultSummary {
    pub ending: Ending,
    pub headline: String,
    pub epilogue: String,
    pub seed: String,
    pub persona_name: String,
    pub mult_str: String,
    pub mode: String,
    pub dp_badge: bool,
    pub score: i32,
    pub days: i32,
    pub encounters: i32,
    pub receipts: i32,
    pub allies: i32,
    pub supplies: i32,
    pub credibility: i32,
    pub pants_pct: i32,
}

impl Default for ResultConfig {
    fn default() -> Self {
        Self {
            score: ScoreCfg {
                pants_threshold: 70,
                pants_penalty_per_point: 2,
                persona_rounding: Rounding::Nearest,
                final_min: 0,
                final_max: 999_999,
            },
            multipliers: MultipliersCfg {
                display_bonus_deep: 0.05,
            },
            endings: EndingCfg {
                priority: vec![
                    "pants".to_string(),
                    "sanity".to_string(),
                    "collapse".to_string(),
                    "boss_loss".to_string(),
                    "victory".to_string(),
                ],
                victory_key: "result.headline.victory".to_string(),
                boss_loss_key: "result.headline.boss_loss".to_string(),
                pants_key: "result.headline.pants".to_string(),
                sanity_key: "result.headline.sanity".to_string(),
                collapse_key: "result.headline.collapse".to_string(),
            },
            limits: ResultLimits {
                share_seed_maxlen: 32,
                share_persona_maxlen: 24,
            },
        }
    }
}

/// Load result configuration from static assets (function for web compatibility)
/// This is a placeholder that returns default data - web implementation should override this
///
/// # Errors
///
/// This function currently does not return errors but may in future implementations.
pub fn load_result_config() -> Result<ResultConfig, Box<dyn std::error::Error>> {
    Ok(ResultConfig::default())
}

/// Generate result summary from game state
///
/// # Errors
///
/// Returns an error if the result summary cannot be generated (currently never fails).
pub fn result_summary(
    _gs: &crate::GameState,
    _cfg: &ResultConfig,
) -> Result<ResultSummary, String> {
    // Placeholder implementation
    Ok(ResultSummary {
        ending: Ending::Victory,
        headline: "Game Complete".to_string(),
        epilogue: "You completed the journey.".to_string(),
        seed: "CL-PLACEHOLDER00".to_string(),
        persona_name: "Player".to_string(),
        mult_str: "1.0x".to_string(),
        mode: "Classic".to_string(),
        dp_badge: false,
        score: 1000,
        days: 30,
        encounters: 10,
        receipts: 5,
        allies: 3,
        supplies: 10,
        credibility: 50,
        pants_pct: 25,
    })
}

/// Select the ending based on strict priority order
#[must_use]
pub fn select_ending(gs: &crate::GameState, _cfg: &ResultConfig, boss_won: bool) -> Ending {
    if gs.stats.pants >= 100 {
        return Ending::Pants;
    }
    if gs.stats.sanity <= 0 {
        return Ending::Sanity;
    }
    if gs.stats.hp <= 0 || gs.stats.supplies <= 0 {
        return Ending::Collapse;
    }
    if !boss_won {
        return Ending::BossLoss;
    }
    Ending::Victory
}
