//! End game result calculation
use crate::seed::encode_friendly;
use crate::state::{CollapseCause, Ending, GameMode, GameState};
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreCfg {
    pub pants_threshold: i32,
    pub pants_penalty_per_point: i32,
    pub persona_rounding: Rounding,
    pub final_min: i32,
    pub final_max: i32,
}

/// Rounding behavior for score calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndingCfg {
    pub priority: Vec<String>,
    pub victory_key: String,
    pub boss_loss_key: String,
    pub pants_key: String,
    pub sanity_key: String,
    pub collapse_key: String,
}

/// Limits for text generation in results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultLimits {
    pub share_seed_maxlen: usize,
    pub share_persona_maxlen: usize,
}

/// Complete summary of a game run for display on the result screen
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultSummary {
    pub ending: Ending,
    pub headline_key: String,
    pub epilogue_key: String,
    pub ending_cause: Option<String>,
    pub seed: String,
    pub persona_name: String,
    pub mult_str: String,
    pub mode: String,
    pub dp_badge: bool,
    pub score: i32,
    pub score_threshold: i32,
    pub passed_threshold: bool,
    pub days: i32,
    pub encounters: i32,
    pub receipts: i32,
    pub allies: i32,
    pub supplies: i32,
    pub credibility: i32,
    pub pants_pct: i32,
    pub vehicle_breakdowns: i32,
    pub miles_traveled: f32,
    pub malnutrition_days: u32,
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
pub fn result_summary(gs: &GameState, cfg: &ResultConfig) -> Result<ResultSummary, String> {
    let score = compute_score(gs, &cfg.score, &cfg.multipliers);
    let threshold = success_threshold(gs.mode);
    let passed_threshold = score >= threshold;

    let final_ending = determine_final_ending(gs, passed_threshold);
    let ending_cause_token = ending_cause_token(final_ending);
    let headline_key = headline_key_for(final_ending, &cfg.endings, ending_cause_token.as_deref());
    let epilogue_key = epilogue_key_for(&headline_key);

    let seed = encode_friendly(gs.mode.is_deep(), gs.seed);
    let persona_name = resolve_persona_name(gs);
    let mult_val = total_multiplier(gs, &cfg.multipliers);
    let mult_str = format!("{mult_val:.2}×");
    let mode_label = mode_display(gs.mode);

    let days = i32::try_from(gs.day.saturating_sub(1)).unwrap_or(0);
    let encounters = i32::try_from(gs.encounters_resolved).unwrap_or(0);
    let receipts = i32::try_from(gs.receipts.len()).unwrap_or(0);

    Ok(ResultSummary {
        ending: final_ending,
        headline_key,
        epilogue_key,
        seed,
        persona_name,
        mult_str,
        mode: mode_label,
        dp_badge: gs.mode.is_deep(),
        score,
        score_threshold: threshold,
        passed_threshold,
        days,
        encounters,
        receipts,
        allies: gs.stats.allies,
        supplies: gs.stats.supplies,
        credibility: gs.stats.credibility,
        pants_pct: gs.stats.pants,
        vehicle_breakdowns: gs.vehicle_breakdowns,
        miles_traveled: gs.miles_traveled_actual,
        malnutrition_days: gs.starvation_days,
        ending_cause: ending_cause_token,
    })
}

#[must_use]
pub fn select_ending(gs: &GameState, cfg: &ResultConfig, boss_won: bool) -> Ending {
    if let Some(existing) = gs.ending {
        return existing;
    }
    let score = compute_score(gs, &cfg.score, &cfg.multipliers);
    determine_final_ending(gs, boss_won || score >= success_threshold(gs.mode))
}

fn compute_score(gs: &GameState, cfg: &ScoreCfg, mult_cfg: &MultipliersCfg) -> i32 {
    let mut base = gs.journey_score();
    let pants = gs.stats.pants.max(0);

    if pants > cfg.pants_threshold {
        base -= (pants - cfg.pants_threshold) * cfg.pants_penalty_per_point;
    }

    base = base.clamp(cfg.final_min, cfg.final_max);
    let multiplier = total_multiplier(gs, mult_cfg);
    let scaled = apply_rounding(f64::from(base) * multiplier, cfg.persona_rounding);
    scaled.clamp(cfg.final_min, cfg.final_max)
}

fn apply_rounding(value: f64, rounding: Rounding) -> i32 {
    match rounding {
        Rounding::Nearest => to_i32(value.round()),
        Rounding::Down => to_i32(value.floor()),
        Rounding::Up => to_i32(value.ceil()),
    }
}

fn to_i32(value: f64) -> i32 {
    let clamped = value.clamp(f64::from(i32::MIN), f64::from(i32::MAX));
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    {
        clamped as i32
    }
}

fn total_multiplier(gs: &GameState, mult_cfg: &MultipliersCfg) -> f64 {
    let deep_bonus = if gs.mode.is_deep() {
        mult_cfg.display_bonus_deep
    } else {
        0.0
    };
    f64::from((gs.score_mult + deep_bonus).max(0.1))
}

const fn determine_final_ending(gs: &GameState, passed_threshold: bool) -> Ending {
    if let Some(existing) = gs.ending {
        return existing;
    }
    if passed_threshold {
        Ending::BossVictory
    } else {
        Ending::BossVoteFailed
    }
}

fn headline_key_for(ending: Ending, cfg: &EndingCfg, cause_token: Option<&str>) -> String {
    if let Some(token) = cause_token {
        return format!("result.headline.{token}");
    }

    match ending {
        Ending::BossVictory => cfg.victory_key.clone(),
        Ending::BossVoteFailed => cfg.boss_loss_key.clone(),
        Ending::SanityLoss => cfg.sanity_key.clone(),
        Ending::VehicleFailure { .. } => "result.headline.vehicle_failure".to_string(),
        Ending::Exposure { kind } => format!("result.headline.exposure_{}", kind.key()),
        Ending::Collapse { cause } => collapse_headline_key(cfg, cause),
    }
}

fn collapse_headline_key(cfg: &EndingCfg, cause: CollapseCause) -> String {
    match cause {
        CollapseCause::Panic => cfg.pants_key.clone(),
        CollapseCause::Hunger => format!("{}_{}", cfg.collapse_key, CollapseCause::Hunger.key()),
        CollapseCause::Vehicle => format!("{}_{}", cfg.collapse_key, CollapseCause::Vehicle.key()),
        CollapseCause::Weather => format!("{}_{}", cfg.collapse_key, CollapseCause::Weather.key()),
        CollapseCause::Breakdown => {
            format!("{}_{}", cfg.collapse_key, CollapseCause::Breakdown.key())
        }
        CollapseCause::Disease | CollapseCause::Crossing => {
            format!("{}_{}", cfg.collapse_key, cause.key())
        }
    }
}

fn ending_cause_token(ending: Ending) -> Option<String> {
    match ending {
        Ending::Collapse { cause } => Some(format!("collapse_{}", cause.key())),
        Ending::Exposure { kind } => Some(format!("exposure_{}", kind.key())),
        Ending::VehicleFailure { cause } => Some(format!("vehicle_failure_{}", cause.key())),
        _ => None,
    }
}

fn epilogue_key_for(headline_key: &str) -> String {
    headline_key.strip_prefix("result.headline").map_or_else(
        || "result.epilogue.generic".to_string(),
        |stripped| format!("result.epilogue{stripped}"),
    )
}

fn resolve_persona_name(gs: &GameState) -> String {
    gs.persona_id
        .clone()
        .filter(|id| !id.is_empty())
        .or_else(|| {
            if gs.party.leader.is_empty() {
                None
            } else {
                Some(gs.party.leader.clone())
            }
        })
        .unwrap_or_else(|| "Traveler".to_string())
}

fn mode_display(mode: GameMode) -> String {
    match mode {
        GameMode::Classic => "Classic".to_string(),
        GameMode::Deep => "The Deep End".to_string(),
    }
}

const fn success_threshold(mode: GameMode) -> i32 {
    mode.boss_threshold()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::VehicleFailureCause;

    #[test]
    fn high_score_triggers_victory() {
        let cfg = ResultConfig::default();
        let mut state = GameState::default();
        state.stats.supplies = 20;
        state.stats.hp = 10;
        state.stats.morale = 10;
        state.stats.credibility = 20;
        state.stats.allies = 5;
        state.day = 45;
        state.encounters_resolved = 8;

        let summary = result_summary(&state, &cfg).unwrap();
        assert!(summary.passed_threshold);
        assert!(matches!(summary.ending, Ending::BossVictory));
        assert!(summary.ending_cause.is_none());
    }

    #[test]
    fn hunger_collapse_maps_to_hunger_keys() {
        #![allow(clippy::field_reassign_with_default)]
        let cfg = ResultConfig::default();
        let mut state = GameState::default();
        state.ending = Some(Ending::Collapse {
            cause: CollapseCause::Hunger,
        });

        let summary = result_summary(&state, &cfg).unwrap();
        assert_eq!(summary.headline_key, "result.headline.collapse_hunger");
        assert_eq!(summary.epilogue_key, "result.epilogue.collapse_hunger");
        assert_eq!(summary.ending_cause.as_deref(), Some("collapse_hunger"));
    }

    #[test]
    fn vehicle_failure_uses_detailed_keys() {
        #![allow(clippy::field_reassign_with_default)]
        let cfg = ResultConfig::default();
        let mut state = GameState::default();
        state.ending = Some(Ending::VehicleFailure {
            cause: VehicleFailureCause::Destroyed,
        });

        let summary = result_summary(&state, &cfg).unwrap();
        assert_eq!(
            summary.headline_key,
            "result.headline.vehicle_failure_vehicle_destroyed"
        );
        assert_eq!(
            summary.epilogue_key,
            "result.epilogue.vehicle_failure_vehicle_destroyed"
        );
        assert_eq!(
            summary.ending_cause.as_deref(),
            Some("vehicle_failure_vehicle_destroyed")
        );
    }
}
