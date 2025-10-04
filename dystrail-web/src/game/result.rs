use serde::{Deserialize, Serialize};

/// Configuration for the result screen and scoring system
///
/// Contains all the parameters needed to calculate scores, determine endings,
/// and generate result summaries based on game state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultConfig {
    pub score: ScoreCfg,
    pub multipliers: MultipliersCfg,
    pub endings: EndingCfg,
    pub limits: ResultLimits,
}

/// Scoring algorithm configuration
///
/// Defines the parameters for calculating the final score including
/// thresholds, penalties, and rounding behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreCfg {
    pub pants_threshold: i32,
    pub pants_penalty_per_point: i32,
    pub persona_rounding: Rounding,
    pub final_min: i32,
    pub final_max: i32,
}

/// Rounding behavior for score calculations
///
/// Determines how fractional scores should be rounded when applying multipliers.
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
///
/// Contains multipliers applied to base scores based on game mode and other factors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultipliersCfg {
    pub display_bonus_deep: f32,
}

/// Configuration for determining game endings
///
/// Defines the keys used for different ending types and their priority order.
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
///
/// Controls maximum lengths for generated text to prevent overflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultLimits {
    pub share_seed_maxlen: usize,
    pub share_persona_maxlen: usize,
}

/// Possible game ending types
///
/// Represents the different ways a game can end, used for determining
/// the appropriate epilogue and final scoring.
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
///
/// Contains all the information needed to render the result screen including
/// scores, statistics, ending type, and display strings.
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

use crate::game::state::GameState;

/// Select the ending based on strict priority order
/// Selects the appropriate ending based on game state and configuration.
/// Returns the ending key string for use in i18n lookups.
#[must_use]
pub fn select_ending(gs: &GameState, _cfg: &ResultConfig, boss_won: bool) -> Ending {
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

/// Compute base and final scores with persona multiplier
/// Computes base and final scores from game state and configuration.
/// Returns a tuple of (`base_score`, `final_score`).
#[must_use]
pub fn compute_scores(gs: &GameState, cfg: &ResultConfig) -> (i32, i32) {
    let pants_pen =
        i32::max(0, gs.stats.pants - cfg.score.pants_threshold) * cfg.score.pants_penalty_per_point;

    // Note: GameState needs a deaths field - for now use 0
    let deaths = 0; // TODO: Add deaths tracking to GameState

    let base = gs.stats.credibility * 10
        + i32::try_from(gs.receipts.len()).unwrap_or(0) * 25
        + gs.stats.allies * 15
        + gs.stats.supplies * 2
        - deaths * 50
        - pants_pen;

    // Get persona multiplier from GameState score_mult field
    let persona_mult = gs.score_mult;

    // Apply persona multiplier with rounding
    #[allow(clippy::cast_precision_loss)]
    let x = (base as f32) * persona_mult;
    #[allow(clippy::cast_possible_truncation)]
    let rounded = match cfg.score.persona_rounding {
        Rounding::Nearest => x.round(),
        Rounding::Down => x.floor(),
        Rounding::Up => x.ceil(),
    } as i32;

    let final_score = rounded.clamp(cfg.score.final_min, cfg.score.final_max);
    (base, final_score)
}

/// Build a localized result summary
///
/// # Errors
///
/// Returns an error string if persona name sanitization fails or if
/// encounter counting overflows integer conversion.
pub fn result_summary(
    gs: &GameState,
    rc: &ResultConfig,
    boss_won: bool,
) -> Result<ResultSummary, String> {
    let ending = select_ending(gs, rc, boss_won);

    // Get headline key based on ending
    let headline_key = match ending {
        Ending::Victory => &rc.endings.victory_key,
        Ending::BossLoss => &rc.endings.boss_loss_key,
        Ending::Pants => &rc.endings.pants_key,
        Ending::Sanity => &rc.endings.sanity_key,
        Ending::Collapse => &rc.endings.collapse_key,
    };

    // Use i18n system to get localized strings
    let headline = crate::i18n::t(headline_key);
    let epilogue_key = format!("result.epilogue.{ending}");
    let epilogue = crate::i18n::t(&epilogue_key);

    let (_, final_score) = compute_scores(gs, rc);

    // Get persona multiplier from score_mult field
    let persona_mult = gs.score_mult;
    let mult_str = format!("×{persona_mult:.1}");

    // Get persona name from persona_id
    let persona_name = gs.persona_id.clone().map_or_else(
        || "Unknown".to_string(),
        |id| sanitize(id, rc.limits.share_persona_maxlen),
    );

    // Get mode string
    let mode = match gs.mode {
        crate::game::state::GameMode::Classic => "CL",
        crate::game::state::GameMode::Deep => "DP",
    };

    // Generate friendly seed from the current seed and mode
    let is_deep = matches!(gs.mode, crate::game::state::GameMode::Deep);
    let friendly_seed = crate::game::seed::encode_friendly(is_deep, gs.seed);
    let seed = sanitize(friendly_seed, rc.limits.share_seed_maxlen);

    // Count encounters from logs
    let encounters = i32::try_from(
        gs.logs
            .iter()
            .filter(|log| log.contains("encounter"))
            .count(),
    )
    .map_err(|_| "Encounter count overflow".to_string())?;

    Ok(ResultSummary {
        ending,
        headline,
        epilogue,
        seed,
        persona_name,
        mult_str,
        mode: mode.to_string(),
        dp_badge: matches!(gs.mode, crate::game::state::GameMode::Deep),
        score: final_score,
        #[allow(clippy::cast_possible_wrap)]
        days: gs.day as i32,
        encounters,
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        receipts: i32::try_from(gs.receipts.len()).unwrap_or(0),
        allies: gs.stats.allies,
        supplies: gs.stats.supplies,
        credibility: gs.stats.credibility,
        pants_pct: gs.stats.pants,
    })
}

/// Sanitize strings to prevent layout breakage
#[must_use]
pub fn sanitize(mut s: String, max: usize) -> String {
    if s.len() > max {
        s.truncate(max.saturating_sub(1));
        s.push('…');
    }
    s
}

/// Format multiplier for display
#[must_use]
pub fn fmt_mult(mult: f32) -> String {
    format!("×{mult:.1}")
}

/// Load result configuration from JSON file
///
/// # Errors
///
/// Returns an error string if the file cannot be fetched, read, or parsed as JSON.
pub async fn load_result_config() -> Result<ResultConfig, String> {
    use gloo::net::http::Request;

    let response = Request::get("/assets/data/result.json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch result.json: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {status}", status = response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {e}"))?;

    serde_json::from_str(&text).map_err(|e| format!("Failed to parse result.json: {e}"))
}
