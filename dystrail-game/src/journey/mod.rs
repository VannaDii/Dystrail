//! Journey domain primitives shared by the controller and state ledger.
use hmac::{Hmac, Mac};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use smallvec::SmallVec;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;
use thiserror::Error;

use crate::endgame::EndgameTravelCfg;
use crate::state::{DietId, GameMode, PaceId, PolicyKind};
use crate::vehicle::PartWeights;
use crate::weather::Weather;

pub mod daily;
pub mod session;
pub use daily::{DailyTickOutcome, apply_daily_effect};
pub use session::JourneySession;

/// Maximum tag capacity stored inline without additional allocations.
pub type DayTagSet = SmallVec<[DayTag; 4]>;

/// Tag describing why a particular day ended up in its recorded state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayTag(pub String);

impl DayTag {
    /// Construct a tag from a string slice, trimming whitespace.
    #[must_use]
    pub fn new(value: &str) -> Self {
        Self(value.trim().to_string())
    }

    /// Returns true when the tag has no visible characters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

/// Travel classification for a recorded day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TravelDayKind {
    /// Full travel day with complete mileage credit.
    Travel,
    /// Partial travel day (detours, repairs, shared travel).
    Partial,
    /// No travel, typically camps or blockers.
    NonTravel,
}

impl TravelDayKind {
    /// Whether this day counts toward the travel ratio.
    #[must_use]
    pub const fn counts_toward_ratio(self) -> bool {
        !matches!(self, Self::NonTravel)
    }
}

/// Immutable ledger entry representing a single simulated day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayRecord {
    /// Zero-based index of the day.
    pub day_index: u16,
    /// Classification for the day.
    pub kind: TravelDayKind,
    /// Miles credited for the day (already partial adjusted).
    pub miles: f32,
    /// Descriptive tags (camp, repair, detour, etc.).
    #[serde(default)]
    pub tags: DayTagSet,
}

impl DayRecord {
    /// Create a new record with no tags.
    #[must_use]
    pub fn new(day_index: u16, kind: TravelDayKind, miles: f32) -> Self {
        Self {
            day_index,
            kind,
            miles,
            tags: DayTagSet::new(),
        }
    }

    /// Adds a tag if it is not already present.
    pub fn push_tag(&mut self, tag: DayTag) {
        if tag.is_empty() || self.tags.iter().any(|existing| existing == &tag) {
            return;
        }
        self.tags.push(tag);
    }
}

/// High-level policy family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyId {
    Classic,
    Deep,
}

impl From<PolicyId> for PolicyKind {
    fn from(value: PolicyId) -> Self {
        match value {
            PolicyId::Classic => Self::Balanced,
            PolicyId::Deep => Self::Aggressive,
        }
    }
}

impl From<PolicyKind> for PolicyId {
    fn from(value: PolicyKind) -> Self {
        match value {
            PolicyKind::Balanced | PolicyKind::Conservative => Self::Classic,
            PolicyKind::Aggressive | PolicyKind::ResourceManager => Self::Deep,
        }
    }
}

impl From<GameMode> for PolicyId {
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Classic => Self::Classic,
            GameMode::Deep => Self::Deep,
        }
    }
}

/// Strategy overlay placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyId {
    Balanced,
    Aggressive,
    Conservative,
    ResourceManager,
}

impl From<StrategyId> for PolicyKind {
    fn from(value: StrategyId) -> Self {
        match value {
            StrategyId::Balanced => Self::Balanced,
            StrategyId::Aggressive => Self::Aggressive,
            StrategyId::Conservative => Self::Conservative,
            StrategyId::ResourceManager => Self::ResourceManager,
        }
    }
}

impl From<PolicyKind> for StrategyId {
    fn from(value: PolicyKind) -> Self {
        match value {
            PolicyKind::Balanced => Self::Balanced,
            PolicyKind::Aggressive => Self::Aggressive,
            PolicyKind::Conservative => Self::Conservative,
            PolicyKind::ResourceManager => Self::ResourceManager,
        }
    }
}

/// Minimal journey configuration scaffold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyCfg {
    #[serde(default)]
    pub travel: TravelConfig,
    #[serde(default = "JourneyCfg::default_partial_ratio")]
    pub partial_ratio: f32,
    #[serde(default = "JourneyCfg::default_victory_miles")]
    pub victory_miles: f32,
    #[serde(default)]
    pub wear: WearConfig,
    #[serde(default)]
    pub breakdown: BreakdownConfig,
    #[serde(default)]
    pub part_weights: PartWeights,
    #[serde(default)]
    pub crossing: CrossingPolicy,
    #[serde(default)]
    pub daily: DailyTickConfig,
    #[serde(default)]
    pub guards: AcceptanceGuards,
}

impl JourneyCfg {
    #[must_use]
    pub const fn default_partial_ratio() -> f32 {
        0.5
    }

    #[must_use]
    pub const fn default_victory_miles() -> f32 {
        crate::boss::ROUTE_LEN_MILES
    }

    /// Validate configuration invariants before sanitization.
    ///
    /// # Errors
    ///
    /// Returns `JourneyConfigError` when any field violates the documented bounds.
    pub fn validate(&self) -> Result<(), JourneyConfigError> {
        self.travel.validate()?;
        self.validate_partial_ratio()?;
        self.validate_victory_miles()?;
        self.wear.validate()?;
        self.breakdown.validate()?;
        self.crossing.validate()?;
        self.daily.validate()?;
        self.guards.validate()?;
        Ok(())
    }

    fn validate_victory_miles(&self) -> Result<(), JourneyConfigError> {
        if !(500.0..=10_000.0).contains(&self.victory_miles) {
            return Err(JourneyConfigError::RangeViolation {
                field: "victory_miles",
                min: 500.0,
                max: 10_000.0,
                value: self.victory_miles,
            });
        }
        Ok(())
    }

    fn validate_partial_ratio(&self) -> Result<(), JourneyConfigError> {
        const MIN_RATIO: f32 = 0.2;
        const MAX_RATIO: f32 = 0.95;
        if !(MIN_RATIO..=MAX_RATIO).contains(&self.partial_ratio) {
            return Err(JourneyConfigError::RangeViolation {
                field: "partial_ratio",
                min: MIN_RATIO,
                max: MAX_RATIO,
                value: self.partial_ratio,
            });
        }
        Ok(())
    }
}

impl Default for JourneyCfg {
    fn default() -> Self {
        Self {
            travel: TravelConfig::default(),
            partial_ratio: Self::default_partial_ratio(),
            victory_miles: Self::default_victory_miles(),
            wear: WearConfig::default(),
            breakdown: BreakdownConfig::default(),
            part_weights: PartWeights::default(),
            crossing: CrossingPolicy::default(),
            daily: DailyTickConfig::default(),
            guards: AcceptanceGuards::default(),
        }
    }
}

/// Acceptance guardrails communicated to the tester for aggregate validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceGuards {
    #[serde(default = "AcceptanceGuards::default_min_travel_ratio")]
    pub min_travel_ratio: f32,
    #[serde(default = "AcceptanceGuards::default_target_distance")]
    pub target_distance: f32,
    #[serde(default = "AcceptanceGuards::default_target_days_min")]
    pub target_days_min: u16,
    #[serde(default = "AcceptanceGuards::default_target_days_max")]
    pub target_days_max: u16,
}

impl AcceptanceGuards {
    const fn default_min_travel_ratio() -> f32 {
        0.9
    }

    const fn default_target_distance() -> f32 {
        2_000.0
    }

    const fn default_target_days_min() -> u16 {
        84
    }

    const fn default_target_days_max() -> u16 {
        180
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        if !(0.5..=1.0).contains(&self.min_travel_ratio) {
            return Err(JourneyConfigError::RangeViolation {
                field: "guards.min_travel_ratio",
                min: 0.5,
                max: 1.0,
                value: self.min_travel_ratio,
            });
        }
        if self.target_distance <= 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field: "guards.target_distance",
                min: 1.0,
                value: self.target_distance,
            });
        }
        if self.target_days_min == 0 {
            return Err(JourneyConfigError::MinViolation {
                field: "guards.target_days_min",
                min: 1.0,
                value: 0.0,
            });
        }
        if self.target_days_min > self.target_days_max {
            return Err(JourneyConfigError::GuardDaysRange {
                min: self.target_days_min,
                max: self.target_days_max,
            });
        }
        Ok(())
    }

    fn with_overlay(&self, overlay: &AcceptanceGuardsOverlay) -> Self {
        Self {
            min_travel_ratio: overlay.min_travel_ratio.unwrap_or(self.min_travel_ratio),
            target_distance: overlay.target_distance.unwrap_or(self.target_distance),
            target_days_min: overlay.target_days_min.unwrap_or(self.target_days_min),
            target_days_max: overlay.target_days_max.unwrap_or(self.target_days_max),
        }
    }
}

impl Default for AcceptanceGuards {
    fn default() -> Self {
        Self {
            min_travel_ratio: Self::default_min_travel_ratio(),
            target_distance: Self::default_target_distance(),
            target_days_min: Self::default_target_days_min(),
            target_days_max: Self::default_target_days_max(),
        }
    }
}

/// Errors raised when journey configuration invariants are violated.
#[derive(Debug, Error, PartialEq)]
pub enum JourneyConfigError {
    #[error("travel minimum {min:.2} exceeds maximum {max:.2}")]
    TravelMinExceedsMax { min: f32, max: f32 },
    #[error("{field} must be at least {min:.2} (got {value:.2})")]
    MinViolation {
        field: &'static str,
        min: f32,
        value: f32,
    },
    #[error("{field} must be between {min:.2} and {max:.2} (got {value:.2})")]
    RangeViolation {
        field: &'static str,
        min: f32,
        max: f32,
        value: f32,
    },
    #[error("target day window invalid (min {min} > max {max})")]
    GuardDaysRange { min: u16, max: u16 },
    #[error("crossing detour bounds invalid (min {min} > max {max})")]
    CrossingDetourBounds { min: u8, max: u8 },
    #[error(
        "crossing probabilities invalid: pass {pass:.2}, detour {detour:.2}, terminal {terminal:.2}"
    )]
    CrossingProbabilities {
        pass: f32,
        detour: f32,
        terminal: f32,
    },
}

/// Policy-driven travel pacing configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TravelConfig {
    #[serde(default = "TravelConfig::default_mpd_base")]
    pub mpd_base: f32,
    #[serde(default = "TravelConfig::default_mpd_min")]
    pub mpd_min: f32,
    #[serde(default = "TravelConfig::default_mpd_max")]
    pub mpd_max: f32,
    #[serde(default = "TravelConfig::default_pace_factor")]
    pub pace_factor: HashMap<PaceId, f32>,
    #[serde(default = "TravelConfig::default_weather_factor")]
    pub weather_factor: HashMap<Weather, f32>,
}

impl TravelConfig {
    const fn default_mpd_base() -> f32 {
        crate::constants::TRAVEL_V2_BASE_DISTANCE
    }

    const fn default_mpd_min() -> f32 {
        6.0
    }

    const fn default_mpd_max() -> f32 {
        24.0
    }

    fn default_pace_factor() -> HashMap<PaceId, f32> {
        HashMap::from([
            (PaceId::Steady, 1.0),
            (PaceId::Heated, 1.2),
            (PaceId::Blitz, 1.35),
        ])
    }

    fn default_weather_factor() -> HashMap<Weather, f32> {
        HashMap::from([
            (Weather::Clear, 1.0),
            (Weather::Storm, 0.85),
            (Weather::HeatWave, 0.8),
            (Weather::ColdSnap, 0.9),
            (Weather::Smoke, 0.88),
        ])
    }
}

impl TravelConfig {
    fn validate(&self) -> Result<(), JourneyConfigError> {
        let min_floor = crate::constants::TRAVEL_PARTIAL_MIN_DISTANCE;
        if self.mpd_min < min_floor {
            return Err(JourneyConfigError::MinViolation {
                field: "travel.mpd_min",
                min: min_floor,
                value: self.mpd_min,
            });
        }
        if self.mpd_min > self.mpd_max {
            return Err(JourneyConfigError::TravelMinExceedsMax {
                min: self.mpd_min,
                max: self.mpd_max,
            });
        }
        if self.mpd_base < self.mpd_min || self.mpd_base > self.mpd_max {
            return Err(JourneyConfigError::RangeViolation {
                field: "travel.mpd_base",
                min: self.mpd_min,
                max: self.mpd_max,
                value: self.mpd_base,
            });
        }
        let multiplier_floor = crate::constants::TRAVEL_CONFIG_MIN_MULTIPLIER;
        for &value in self.pace_factor.values() {
            if value < multiplier_floor {
                return Err(JourneyConfigError::MinViolation {
                    field: "travel.pace_factor",
                    min: multiplier_floor,
                    value,
                });
            }
        }
        for &value in self.weather_factor.values() {
            if value < multiplier_floor {
                return Err(JourneyConfigError::MinViolation {
                    field: "travel.weather_factor",
                    min: multiplier_floor,
                    value,
                });
            }
        }
        Ok(())
    }
}

impl Default for TravelConfig {
    fn default() -> Self {
        Self {
            mpd_base: Self::default_mpd_base(),
            mpd_min: Self::default_mpd_min(),
            mpd_max: Self::default_mpd_max(),
            pace_factor: Self::default_pace_factor(),
            weather_factor: Self::default_weather_factor(),
        }
    }
}

/// Partial overlay of wear parameters applied atop a resolved policy.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WearConfigOverlay {
    pub base: Option<f32>,
    pub fatigue_k: Option<f32>,
    pub comfort_miles: Option<f32>,
}

/// Partial overlay of breakdown parameters applied atop a resolved policy.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BreakdownConfigOverlay {
    pub base: Option<f32>,
    pub beta: Option<f32>,
    #[serde(default)]
    pub pace_factor: Option<HashMap<PaceId, f32>>,
    #[serde(default)]
    pub weather_factor: Option<HashMap<Weather, f32>>,
}

/// Overlay for part weights used in breakdown selection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PartWeightsOverlay {
    pub tire: Option<u32>,
    pub battery: Option<u32>,
    pub alt: Option<u32>,
    pub pump: Option<u32>,
}

/// Probability and behavior policy for river crossings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossingPolicy {
    #[serde(default = "CrossingPolicy::default_pass")]
    pub pass: f32,
    #[serde(default = "CrossingPolicy::default_detour")]
    pub detour: f32,
    #[serde(default = "CrossingPolicy::default_terminal")]
    pub terminal: f32,
    #[serde(default)]
    pub detour_days: DetourPolicy,
    #[serde(default)]
    pub bribe: BribePolicy,
    #[serde(default)]
    pub permit: PermitPolicy,
}

impl CrossingPolicy {
    const fn default_pass() -> f32 {
        0.7
    }

    const fn default_detour() -> f32 {
        0.2
    }

    const fn default_terminal() -> f32 {
        0.1
    }

    #[must_use]
    pub fn with_overlay(&self, overlay: &CrossingPolicyOverlay) -> Self {
        let mut merged = self.clone();
        if let Some(pass) = overlay.pass {
            merged.pass = pass;
        }
        if let Some(detour) = overlay.detour {
            merged.detour = detour;
        }
        if let Some(terminal) = overlay.terminal {
            merged.terminal = terminal;
        }
        if let Some(detour_days) = overlay.detour_days.as_ref() {
            merged.detour_days = detour_days.clone();
        }
        if let Some(bribe) = overlay.bribe.as_ref() {
            merged.bribe = bribe.clone();
        }
        if let Some(permit) = overlay.permit.as_ref() {
            merged.permit = permit.clone();
        }
        merged
    }

    pub fn sanitize(&mut self) {
        let mut pass = self.pass.max(0.0);
        let mut detour = self.detour.max(0.0);
        let mut terminal = self.terminal.max(0.0);
        let total = pass + detour + terminal;
        if total <= f32::EPSILON {
            pass = Self::default_pass();
            detour = Self::default_detour();
            terminal = Self::default_terminal();
        }
        let normalized_total = pass + detour + terminal;
        self.pass = (pass / normalized_total).clamp(0.0, 1.0);
        self.detour = (detour / normalized_total).clamp(0.0, 1.0);
        self.terminal = (terminal / normalized_total).clamp(0.0, 1.0);
        let renormalized = self.pass + self.detour + self.terminal;
        if (renormalized - 1.0).abs() > 1e-6 {
            self.pass /= renormalized;
            self.detour /= renormalized;
            self.terminal /= renormalized;
        }
        self.detour_days.sanitize();
        self.bribe.sanitize();
        self.permit.sanitize();
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        if self.detour_days.min > self.detour_days.max {
            return Err(JourneyConfigError::CrossingDetourBounds {
                min: self.detour_days.min,
                max: self.detour_days.max,
            });
        }
        if self.pass < 0.0 || self.detour < 0.0 || self.terminal < 0.0 {
            return Err(JourneyConfigError::CrossingProbabilities {
                pass: self.pass,
                detour: self.detour,
                terminal: self.terminal,
            });
        }
        let sum = self.pass + self.detour + self.terminal;
        if sum <= f32::EPSILON {
            return Err(JourneyConfigError::CrossingProbabilities {
                pass: self.pass,
                detour: self.detour,
                terminal: self.terminal,
            });
        }
        Ok(())
    }
}

/// Per-day stat adjustments driven by policy configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyTickConfig {
    #[serde(default)]
    pub supplies: DailyChannelConfig,
    #[serde(default)]
    pub sanity: DailyChannelConfig,
    #[serde(default)]
    pub health: HealthTickConfig,
}

impl Default for DailyTickConfig {
    fn default() -> Self {
        Self {
            supplies: DailyChannelConfig::new(0.0),
            sanity: DailyChannelConfig::new(0.0),
            health: HealthTickConfig::default(),
        }
    }
}

impl DailyTickConfig {
    pub fn sanitize(&mut self) {
        self.supplies.sanitize();
        self.sanity.sanitize();
        self.health.sanitize();
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        self.supplies.validate("daily.supplies")?;
        self.sanity.validate("daily.sanity")?;
        self.health.validate()?;
        Ok(())
    }
}

/// Channel multiplier set for supplies/sanity adjustments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyChannelConfig {
    #[serde(default = "DailyChannelConfig::default_base")]
    pub base: f32,
    #[serde(default)]
    pub pace: HashMap<PaceId, f32>,
    #[serde(default)]
    pub diet: HashMap<DietId, f32>,
    #[serde(default)]
    pub weather: HashMap<Weather, f32>,
    #[serde(default)]
    pub exec: HashMap<String, f32>,
}

impl DailyChannelConfig {
    const fn default_base() -> f32 {
        0.0
    }

    fn new(base: f32) -> Self {
        Self {
            base,
            pace: HashMap::new(),
            diet: HashMap::new(),
            weather: HashMap::new(),
            exec: HashMap::new(),
        }
    }

    fn sanitize(&mut self) {
        if !self.base.is_finite() || self.base < 0.0 {
            self.base = 0.0;
        }
        for value in self.pace.values_mut() {
            if !value.is_finite() || *value <= 0.0 {
                *value = 1.0;
            }
        }
        for value in self.diet.values_mut() {
            if !value.is_finite() || *value <= 0.0 {
                *value = 1.0;
            }
        }
        for value in self.weather.values_mut() {
            if !value.is_finite() || *value <= 0.0 {
                *value = 1.0;
            }
        }
        for value in self.exec.values_mut() {
            if !value.is_finite() || *value <= 0.0 {
                *value = 1.0;
            }
        }
    }

    fn validate(&self, field: &'static str) -> Result<(), JourneyConfigError> {
        if self.base < 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field,
                min: 0.0,
                value: self.base,
            });
        }
        Ok(())
    }
}

impl Default for DailyChannelConfig {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Health-specific daily tuning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthTickConfig {
    #[serde(default = "HealthTickConfig::default_decay")]
    pub decay: f32,
    #[serde(default = "HealthTickConfig::default_rest_heal")]
    pub rest_heal: f32,
    #[serde(default)]
    pub weather: HashMap<Weather, f32>,
    #[serde(default)]
    pub exec: HashMap<String, f32>,
}

impl Default for HealthTickConfig {
    fn default() -> Self {
        Self {
            decay: Self::default_decay(),
            rest_heal: Self::default_rest_heal(),
            weather: HashMap::new(),
            exec: HashMap::new(),
        }
    }
}

impl HealthTickConfig {
    const fn default_decay() -> f32 {
        0.0
    }

    const fn default_rest_heal() -> f32 {
        2.0
    }

    fn sanitize(&mut self) {
        if !self.decay.is_finite() || self.decay < 0.0 {
            self.decay = 0.0;
        }
        if !self.rest_heal.is_finite() || self.rest_heal < 0.0 {
            self.rest_heal = 0.0;
        }
        for value in self.weather.values_mut() {
            if !value.is_finite() || *value < 0.0 {
                *value = 0.0;
            }
        }
        for value in self.exec.values_mut() {
            if !value.is_finite() {
                *value = 0.0;
            }
        }
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        if self.decay < 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field: "daily.health.decay",
                min: 0.0,
                value: self.decay,
            });
        }
        if self.rest_heal < 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field: "daily.health.rest_heal",
                min: 0.0,
                value: self.rest_heal,
            });
        }
        Ok(())
    }
}

impl Default for CrossingPolicy {
    fn default() -> Self {
        Self {
            pass: Self::default_pass(),
            detour: Self::default_detour(),
            terminal: Self::default_terminal(),
            detour_days: DetourPolicy::default(),
            bribe: BribePolicy::default(),
            permit: PermitPolicy::default(),
        }
    }
}

/// Detour duration policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetourPolicy {
    #[serde(default = "DetourPolicy::default_min")]
    pub min: u8,
    #[serde(default = "DetourPolicy::default_max")]
    pub max: u8,
}

impl DetourPolicy {
    const fn default_min() -> u8 {
        1
    }

    const fn default_max() -> u8 {
        3
    }

    #[allow(clippy::missing_const_for_fn)]
    fn sanitize(&mut self) {
        if self.min == 0 {
            self.min = 1;
        }
        if self.max < self.min {
            self.max = self.min;
        }
    }
}

impl Default for DetourPolicy {
    fn default() -> Self {
        Self {
            min: Self::default_min(),
            max: Self::default_max(),
        }
    }
}

/// Bribe probability adjustments.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BribePolicy {
    #[serde(default)]
    pub pass_bonus: f32,
    #[serde(default)]
    pub detour_bonus: f32,
    #[serde(default)]
    pub terminal_penalty: f32,
    #[serde(default = "BribePolicy::default_diminishing")]
    pub diminishing_returns: f32,
}

impl BribePolicy {
    const fn default_diminishing() -> f32 {
        0.5
    }

    #[allow(clippy::missing_const_for_fn)]
    fn sanitize(&mut self) {
        self.pass_bonus = self.pass_bonus.clamp(-0.9, 0.9);
        self.detour_bonus = self.detour_bonus.clamp(-0.9, 0.9);
        self.terminal_penalty = self.terminal_penalty.clamp(-0.9, 0.9);
        self.diminishing_returns = self.diminishing_returns.clamp(0.0, 1.0);
    }
}

impl Default for BribePolicy {
    fn default() -> Self {
        Self {
            pass_bonus: 0.0,
            detour_bonus: 0.0,
            terminal_penalty: 0.0,
            diminishing_returns: Self::default_diminishing(),
        }
    }
}

/// Permit adjustments controlling terminal outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PermitPolicy {
    #[serde(default)]
    pub disable_terminal: bool,
    #[serde(default)]
    pub eligible: Vec<String>,
}

impl PermitPolicy {
    fn sanitize(&mut self) {
        self.eligible.sort();
        self.eligible.dedup();
    }
}

/// Overlay for crossing policy tweaks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CrossingPolicyOverlay {
    pub pass: Option<f32>,
    pub detour: Option<f32>,
    pub terminal: Option<f32>,
    #[serde(default)]
    pub detour_days: Option<DetourPolicy>,
    #[serde(default)]
    pub bribe: Option<BribePolicy>,
    #[serde(default)]
    pub permit: Option<PermitPolicy>,
}

/// Strategy overlay containing policy adjustments.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct JourneyOverlay {
    #[serde(default)]
    pub travel: Option<TravelConfigOverlay>,
    pub partial_ratio: Option<f32>,
    pub victory_miles: Option<f32>,
    #[serde(default)]
    pub wear: Option<WearConfigOverlay>,
    #[serde(default)]
    pub breakdown: Option<BreakdownConfigOverlay>,
    #[serde(default)]
    pub part_weights: Option<PartWeightsOverlay>,
    #[serde(default)]
    pub crossing: Option<CrossingPolicyOverlay>,
    #[serde(default)]
    pub guards: Option<AcceptanceGuardsOverlay>,
}

impl JourneyCfg {
    /// Apply a strategy overlay to this configuration, producing a merged set of parameters.
    #[must_use]
    pub fn merge_overlay(&self, overlay: &JourneyOverlay) -> Self {
        let mut merged = self.clone();
        if let Some(travel_overlay) = overlay.travel.as_ref() {
            merged.travel = merged.travel.with_overlay(travel_overlay);
        }
        if let Some(ratio) = overlay.partial_ratio {
            merged.partial_ratio = ratio;
        }
        if let Some(distance) = overlay.victory_miles {
            merged.victory_miles = distance;
        }
        if let Some(wear_overlay) = overlay.wear.as_ref() {
            merged.wear = merged.wear.with_overlay(wear_overlay);
        }
        if let Some(breakdown_overlay) = overlay.breakdown.as_ref() {
            merged.breakdown = merged.breakdown.with_overlay(breakdown_overlay);
        }
        if let Some(part_overlay) = overlay.part_weights.as_ref() {
            merged.part_weights = merged.part_weights.with_overlay(part_overlay);
        }
        if let Some(crossing_overlay) = overlay.crossing.as_ref() {
            merged.crossing = merged.crossing.with_overlay(crossing_overlay);
        }
        if let Some(guards_overlay) = overlay.guards.as_ref() {
            merged.guards = merged.guards.with_overlay(guards_overlay);
        }
        merged
    }
}

/// Overlay for acceptance guard adjustments.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AcceptanceGuardsOverlay {
    pub min_travel_ratio: Option<f32>,
    pub target_distance: Option<f32>,
    pub target_days_min: Option<u16>,
    pub target_days_max: Option<u16>,
}

/// Overlay of travel pacing parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TravelConfigOverlay {
    pub mpd_base: Option<f32>,
    pub mpd_min: Option<f32>,
    pub mpd_max: Option<f32>,
    #[serde(default)]
    pub pace_factor: Option<HashMap<PaceId, f32>>,
    #[serde(default)]
    pub weather_factor: Option<HashMap<Weather, f32>>,
}

impl TravelConfig {
    #[must_use]
    fn with_overlay(&self, overlay: &TravelConfigOverlay) -> Self {
        let mut merged = self.clone();
        if let Some(base) = overlay.mpd_base {
            merged.mpd_base = base;
        }
        if let Some(min) = overlay.mpd_min {
            merged.mpd_min = min;
        }
        if let Some(max) = overlay.mpd_max {
            merged.mpd_max = max;
        }
        if let Some(pace_map) = overlay.pace_factor.as_ref() {
            for (&pace, &value) in pace_map {
                merged.pace_factor.insert(pace, value);
            }
        }
        if let Some(weather_map) = overlay.weather_factor.as_ref() {
            for (&weather, &value) in weather_map {
                merged.weather_factor.insert(weather, value);
            }
        }
        merged
    }

    pub(crate) fn sanitize(&mut self) {
        self.mpd_min = self
            .mpd_min
            .max(crate::constants::TRAVEL_PARTIAL_MIN_DISTANCE);
        self.mpd_max = self.mpd_max.max(self.mpd_min);
        if self.mpd_base.is_nan() || self.mpd_base <= 0.0 {
            self.mpd_base = Self::default_mpd_base();
        }
        self.mpd_base = self.mpd_base.clamp(self.mpd_min, self.mpd_max);

        for pace in [PaceId::Steady, PaceId::Heated, PaceId::Blitz] {
            let default = Self::default_pace_factor()
                .get(&pace)
                .copied()
                .unwrap_or(1.0);
            let entry = self.pace_factor.entry(pace).or_insert(default);
            *entry = entry.max(crate::constants::TRAVEL_CONFIG_MIN_MULTIPLIER);
        }
        for value in self.pace_factor.values_mut() {
            *value = value.max(crate::constants::TRAVEL_CONFIG_MIN_MULTIPLIER);
        }

        for weather in [
            Weather::Clear,
            Weather::Storm,
            Weather::HeatWave,
            Weather::ColdSnap,
            Weather::Smoke,
        ] {
            let default = Self::default_weather_factor()
                .get(&weather)
                .copied()
                .unwrap_or(1.0);
            let entry = self.weather_factor.entry(weather).or_insert(default);
            *entry = entry.max(crate::constants::TRAVEL_CONFIG_MIN_MULTIPLIER);
        }
        for value in self.weather_factor.values_mut() {
            *value = value.max(crate::constants::TRAVEL_CONFIG_MIN_MULTIPLIER);
        }
    }
}

/// Wear configuration resolved from policy and overlays.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WearConfig {
    #[serde(default = "WearConfig::default_base")]
    pub base: f32,
    #[serde(default = "WearConfig::default_fatigue_k")]
    pub fatigue_k: f32,
    #[serde(default = "WearConfig::default_comfort_miles")]
    pub comfort_miles: f32,
}

impl WearConfig {
    const fn default_base() -> f32 {
        crate::constants::VEHICLE_DAILY_WEAR
    }

    const fn default_fatigue_k() -> f32 {
        0.0
    }

    const fn default_comfort_miles() -> f32 {
        1_200.0
    }
}

impl Default for WearConfig {
    fn default() -> Self {
        Self {
            base: Self::default_base(),
            fatigue_k: Self::default_fatigue_k(),
            comfort_miles: Self::default_comfort_miles(),
        }
    }
}

impl WearConfig {
    #[must_use]
    fn with_overlay(&self, overlay: &WearConfigOverlay) -> Self {
        Self {
            base: overlay.base.unwrap_or(self.base),
            fatigue_k: overlay.fatigue_k.unwrap_or(self.fatigue_k),
            comfort_miles: overlay.comfort_miles.unwrap_or(self.comfort_miles),
        }
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        for (field, value) in [
            ("wear.base", self.base),
            ("wear.fatigue_k", self.fatigue_k),
            ("wear.comfort_miles", self.comfort_miles),
        ] {
            if value < 0.0 {
                return Err(JourneyConfigError::MinViolation {
                    field,
                    min: 0.0,
                    value,
                });
            }
        }
        Ok(())
    }
}

/// Breakdown probability configuration bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakdownConfig {
    #[serde(default = "BreakdownConfig::default_base")]
    pub base: f32,
    #[serde(default = "BreakdownConfig::default_beta")]
    pub beta: f32,
    #[serde(default = "BreakdownConfig::default_pace_factor")]
    pub pace_factor: HashMap<PaceId, f32>,
    #[serde(default = "BreakdownConfig::default_weather_factor")]
    pub weather_factor: HashMap<Weather, f32>,
}

impl BreakdownConfig {
    const fn default_base() -> f32 {
        crate::constants::VEHICLE_BREAKDOWN_BASE_CHANCE
    }

    const fn default_beta() -> f32 {
        crate::constants::VEHICLE_BREAKDOWN_WEAR_COEFFICIENT
    }

    fn default_pace_factor() -> HashMap<PaceId, f32> {
        HashMap::from([
            (PaceId::Steady, crate::constants::PACE_BREAKDOWN_STEADY),
            (PaceId::Heated, crate::constants::PACE_BREAKDOWN_HEATED),
            (PaceId::Blitz, crate::constants::PACE_BREAKDOWN_BLITZ),
        ])
    }

    fn default_weather_factor() -> HashMap<Weather, f32> {
        HashMap::from([
            (Weather::Clear, 1.0),
            (Weather::Storm, 1.3),
            (Weather::HeatWave, 1.4),
            (Weather::ColdSnap, 1.1),
            (Weather::Smoke, 1.1),
        ])
    }
}

impl Default for BreakdownConfig {
    fn default() -> Self {
        Self {
            base: Self::default_base(),
            beta: Self::default_beta(),
            pace_factor: Self::default_pace_factor(),
            weather_factor: Self::default_weather_factor(),
        }
    }
}

impl BreakdownConfig {
    #[must_use]
    fn with_overlay(&self, overlay: &BreakdownConfigOverlay) -> Self {
        let mut merged = self.clone();
        if let Some(base) = overlay.base {
            merged.base = base;
        }
        if let Some(beta) = overlay.beta {
            merged.beta = beta;
        }
        if let Some(pace_map) = overlay.pace_factor.as_ref() {
            for (&pace, &value) in pace_map {
                merged.pace_factor.insert(pace, value);
            }
        }
        if let Some(weather_map) = overlay.weather_factor.as_ref() {
            for (&weather, &value) in weather_map {
                merged.weather_factor.insert(weather, value);
            }
        }
        merged
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        if !(0.0..=1.0).contains(&self.base) {
            return Err(JourneyConfigError::RangeViolation {
                field: "breakdown.base",
                min: 0.0,
                max: 1.0,
                value: self.base,
            });
        }
        if self.beta < 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field: "breakdown.beta",
                min: 0.0,
                value: self.beta,
            });
        }
        for &value in self.pace_factor.values() {
            if value < 0.0 {
                return Err(JourneyConfigError::MinViolation {
                    field: "breakdown.pace_factor",
                    min: 0.0,
                    value,
                });
            }
        }
        for &value in self.weather_factor.values() {
            if value < 0.0 {
                return Err(JourneyConfigError::MinViolation {
                    field: "breakdown.weather_factor",
                    min: 0.0,
                    value,
                });
            }
        }
        Ok(())
    }
}

impl PartWeights {
    #[must_use]
    fn with_overlay(&self, overlay: &PartWeightsOverlay) -> Self {
        Self {
            tire: overlay.tire.unwrap_or(self.tire),
            battery: overlay.battery.unwrap_or(self.battery),
            alt: overlay.alt.unwrap_or(self.alt),
            pump: overlay.pump.unwrap_or(self.pump),
        }
    }
}

/// Aggregates journey policies and strategy overlays.
#[derive(Debug, Clone, Default)]
pub struct PolicyCatalog {
    families: HashMap<PolicyId, JourneyCfg>,
    overlays: HashMap<StrategyId, JourneyOverlay>,
}

impl PolicyCatalog {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        families: HashMap<PolicyId, JourneyCfg>,
        overlays: HashMap<StrategyId, JourneyOverlay>,
    ) -> Self {
        Self { families, overlays }
    }

    #[must_use]
    ///
    /// # Panics
    ///
    /// Panics when the resolved configuration violates invariant checks.
    pub fn resolve(&self, policy: PolicyId, strategy: StrategyId) -> JourneyCfg {
        let base = self
            .families
            .get(&policy)
            .cloned()
            .unwrap_or_else(JourneyCfg::default);
        let overlay = self
            .overlays
            .get(&strategy)
            .or_else(|| self.overlays.get(&StrategyId::Balanced));
        let mut resolved = if let Some(overlay) = overlay {
            base.merge_overlay(overlay)
        } else {
            base
        };
        resolved.validate().unwrap_or_else(|err| {
            panic!("invalid journey config for {policy:?}/{strategy:?}: {err}");
        });
        resolved.travel.sanitize();
        resolved.crossing.sanitize();
        resolved
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn families(&self) -> &HashMap<PolicyId, JourneyCfg> {
        &self.families
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn overlays(&self) -> &HashMap<StrategyId, JourneyOverlay> {
        &self.overlays
    }
}

fn policy_catalog() -> &'static PolicyCatalog {
    static CATALOG: OnceLock<PolicyCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let classic_cfg: JourneyCfg = serde_json::from_str(include_str!(
            "../../../dystrail-web/static/assets/data/journey/classic.json"
        ))
        .expect("valid classic journey config");
        let deep_cfg: JourneyCfg = serde_json::from_str(include_str!(
            "../../../dystrail-web/static/assets/data/journey/deep.json"
        ))
        .expect("valid deep journey config");

        let mut families = HashMap::new();
        families.insert(PolicyId::Classic, classic_cfg);
        families.insert(PolicyId::Deep, deep_cfg);

        let mut overlays = HashMap::new();
        overlays.insert(
            StrategyId::Balanced,
            serde_json::from_str(include_str!(
                "../../../dystrail-web/static/assets/data/journey/overlays/balanced.json"
            ))
            .expect("valid balanced overlay"),
        );
        overlays.insert(
            StrategyId::Aggressive,
            serde_json::from_str(include_str!(
                "../../../dystrail-web/static/assets/data/journey/overlays/aggressive.json"
            ))
            .expect("valid aggressive overlay"),
        );
        overlays.insert(
            StrategyId::Conservative,
            serde_json::from_str(include_str!(
                "../../../dystrail-web/static/assets/data/journey/overlays/conservative.json"
            ))
            .expect("valid conservative overlay"),
        );
        overlays.insert(
            StrategyId::ResourceManager,
            serde_json::from_str(include_str!(
                "../../../dystrail-web/static/assets/data/journey/overlays/resource_manager.json"
            ))
            .expect("valid resource manager overlay"),
        );

        PolicyCatalog::new(families, overlays)
    })
}

/// Result returned by a journey tick.
#[derive(Debug, Clone)]
pub struct DayOutcome {
    pub ended: bool,
    pub log_key: String,
    pub breakdown_started: bool,
    pub record: Option<DayRecord>,
}

/// Deterministic bundle of RNG streams segregated by simulation domain.
#[derive(Debug, Clone)]
pub struct RngBundle {
    travel: RefCell<CountingRng<SmallRng>>,
    breakdown: RefCell<CountingRng<SmallRng>>,
    encounter: RefCell<CountingRng<SmallRng>>,
    crossing: RefCell<CountingRng<SmallRng>>,
}

impl RngBundle {
    /// Construct the bundle from a user-visible seed.
    #[must_use]
    pub fn from_user_seed(seed: u64) -> Self {
        let travel = CountingRng::new(derive_stream_seed(seed, b"travel"));
        let breakdown = CountingRng::new(derive_stream_seed(seed, b"breakdown"));
        let encounter = CountingRng::new(derive_stream_seed(seed, b"encounter"));
        let crossing = CountingRng::new(derive_stream_seed(seed, b"crossing"));
        Self {
            travel: RefCell::new(travel),
            breakdown: RefCell::new(breakdown),
            encounter: RefCell::new(encounter),
            crossing: RefCell::new(crossing),
        }
    }

    /// Access the travel RNG stream.
    #[must_use]
    pub fn travel(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.travel.borrow_mut()
    }

    /// Access the breakdown RNG stream.
    #[must_use]
    pub fn breakdown(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.breakdown.borrow_mut()
    }

    /// Access the encounter RNG stream.
    #[must_use]
    pub fn encounter(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.encounter.borrow_mut()
    }

    /// Access the crossing RNG stream.
    #[must_use]
    pub fn crossing(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.crossing.borrow_mut()
    }
}

/// Counting wrapper for RNG streams providing instrumentation.
#[derive(Debug, Clone)]
pub struct CountingRng<R> {
    rng: R,
    draws: u64,
}

impl CountingRng<SmallRng> {
    fn new(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
            draws: 0,
        }
    }
}

impl<R: rand::RngCore> CountingRng<R> {
    /// Number of draw calls performed against this stream.
    #[must_use]
    pub const fn draws(&self) -> u64 {
        self.draws
    }
}

impl<R: rand::RngCore> rand::RngCore for CountingRng<R> {
    fn next_u32(&mut self) -> u32 {
        self.draws = self.draws.saturating_add(1);
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.draws = self.draws.saturating_add(1);
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.draws = self.draws.saturating_add(1);
        self.rng.fill_bytes(dest);
    }
}

fn derive_stream_seed(user_seed: u64, domain_tag: &[u8]) -> u64 {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(&user_seed.to_le_bytes()).expect("64-bit seed is valid key");
    mac.update(domain_tag);
    let digest = mac.finalize().into_bytes();
    let seed_bytes: [u8; 8] = digest[..8].try_into().expect("digest slice length");
    u64::from_le_bytes(seed_bytes)
}

/// Shell journey controller; later phases will expand its responsibilities.
#[derive(Debug, Clone)]
pub struct JourneyController {
    policy: PolicyId,
    strategy: StrategyId,
    cfg: JourneyCfg,
    rng: Rc<RngBundle>,
    endgame_cfg: EndgameTravelCfg,
}

impl JourneyController {
    /// Create a new controller with default configuration.
    #[must_use]
    pub fn new(policy: PolicyId, strategy: StrategyId, seed: u64) -> Self {
        let cfg = policy_catalog().resolve(policy, strategy);
        Self::with_config(
            policy,
            strategy,
            cfg,
            seed,
            EndgameTravelCfg::default_config(),
        )
    }

    /// Create a new controller with explicit configuration and endgame settings.
    ///
    /// # Panics
    ///
    /// Panics when the supplied configuration violates validation rules.
    #[must_use]
    pub fn with_config(
        policy: PolicyId,
        strategy: StrategyId,
        cfg: JourneyCfg,
        seed: u64,
        endgame_cfg: EndgameTravelCfg,
    ) -> Self {
        cfg.validate().expect("valid journey config");
        let mut resolved_cfg = cfg;
        resolved_cfg.partial_ratio = resolved_cfg.partial_ratio.clamp(0.2, 0.95);
        resolved_cfg.travel.sanitize();
        resolved_cfg.wear.base = resolved_cfg.wear.base.max(0.0);
        resolved_cfg.wear.fatigue_k = resolved_cfg.wear.fatigue_k.max(0.0);
        resolved_cfg.wear.comfort_miles = resolved_cfg.wear.comfort_miles.max(0.0);
        resolved_cfg.breakdown.base = resolved_cfg.breakdown.base.clamp(0.0, 1.0);
        resolved_cfg.breakdown.beta = resolved_cfg.breakdown.beta.max(0.0);
        resolved_cfg.breakdown.pace_factor.insert(
            PaceId::Steady,
            resolved_cfg
                .breakdown
                .pace_factor
                .get(&PaceId::Steady)
                .copied()
                .unwrap_or(crate::constants::PACE_BREAKDOWN_STEADY),
        );
        resolved_cfg.breakdown.pace_factor.insert(
            PaceId::Heated,
            resolved_cfg
                .breakdown
                .pace_factor
                .get(&PaceId::Heated)
                .copied()
                .unwrap_or(crate::constants::PACE_BREAKDOWN_HEATED),
        );
        resolved_cfg.breakdown.pace_factor.insert(
            PaceId::Blitz,
            resolved_cfg
                .breakdown
                .pace_factor
                .get(&PaceId::Blitz)
                .copied()
                .unwrap_or(crate::constants::PACE_BREAKDOWN_BLITZ),
        );
        for weather in [
            Weather::Clear,
            Weather::Storm,
            Weather::HeatWave,
            Weather::ColdSnap,
            Weather::Smoke,
        ] {
            let default = BreakdownConfig::default_weather_factor()
                .get(&weather)
                .copied()
                .unwrap_or(1.0);
            let entry = resolved_cfg
                .breakdown
                .weather_factor
                .entry(weather)
                .or_insert(default);
            *entry = entry.max(0.0);
        }
        resolved_cfg.crossing.sanitize();
        resolved_cfg.daily.sanitize();
        Self {
            policy,
            strategy,
            cfg: resolved_cfg,
            rng: Rc::new(RngBundle::from_user_seed(seed)),
            endgame_cfg,
        }
    }

    #[must_use]
    pub const fn policy(&self) -> PolicyId {
        self.policy
    }

    #[must_use]
    pub const fn strategy(&self) -> StrategyId {
        self.strategy
    }

    #[must_use]
    pub const fn config(&self) -> &JourneyCfg {
        &self.cfg
    }

    /// Expose the shared RNG bundle for session initialization.
    #[must_use]
    pub fn rng_bundle(&self) -> Rc<RngBundle> {
        self.rng.clone()
    }

    /// Override the controller's endgame travel configuration.
    pub fn set_endgame_config(&mut self, cfg: EndgameTravelCfg) {
        self.endgame_cfg = cfg;
    }

    /// Deterministically reseed controller-owned RNGs.
    pub fn reseed(&mut self, seed: u64) {
        self.rng = Rc::new(RngBundle::from_user_seed(seed));
    }

    /// Perform a single day tick using the current game state.
    #[must_use]
    pub fn tick_day(&mut self, state: &mut crate::state::GameState) -> DayOutcome {
        state.attach_rng_bundle(self.rng.clone());
        state.policy = Some(self.strategy.into());
        state.journey_partial_ratio = self.cfg.partial_ratio;
        state.trail_distance = self.cfg.victory_miles.max(1.0);
        state.journey_travel = self.cfg.travel.clone();
        state.journey_wear = self.cfg.wear.clone();
        state.journey_breakdown = self.cfg.breakdown.clone();
        state.journey_part_weights = self.cfg.part_weights.clone();
        state.journey_crossing = self.cfg.crossing.clone();
        {
            let travel_rng = self.rng.travel();
            let _ = travel_rng.draws();
        }
        let (ended, log_key, breakdown_started) = state.travel_next_leg(&self.endgame_cfg);
        let record = state.day_records.last().cloned();
        DayOutcome {
            ended,
            log_key,
            breakdown_started,
            record,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameState;
    use crate::weather::Weather;
    use rand::RngCore;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn policy_catalog_resolves_family_and_overlay() {
        let catalog = policy_catalog();
        let classic_balanced = catalog.resolve(PolicyId::Classic, StrategyId::Balanced);
        let classic_aggressive = catalog.resolve(PolicyId::Classic, StrategyId::Aggressive);
        assert!(
            classic_aggressive.partial_ratio < classic_balanced.partial_ratio,
            "aggressive overlay should reduce partial ratio"
        );
        assert!(
            classic_aggressive.wear.base > classic_balanced.wear.base,
            "aggressive overlay should increase base wear"
        );
        assert!(
            classic_aggressive.travel.mpd_base > classic_balanced.travel.mpd_base,
            "aggressive overlay should increase base mpd"
        );
        assert!(
            classic_aggressive
                .travel
                .pace_factor
                .get(&PaceId::Blitz)
                .unwrap()
                > classic_balanced
                    .travel
                    .pace_factor
                    .get(&PaceId::Blitz)
                    .unwrap(),
            "aggressive overlay should bias blitz pace"
        );

        let deep_balanced = catalog.resolve(PolicyId::Deep, StrategyId::Balanced);
        let deep_conservative = catalog.resolve(PolicyId::Deep, StrategyId::Conservative);
        assert!(
            deep_conservative.breakdown.base < deep_balanced.breakdown.base,
            "conservative overlay should ease breakdown chance"
        );
        assert!(
            deep_conservative.travel.mpd_max < deep_balanced.travel.mpd_max,
            "conservative overlay should lower max mpd"
        );
    }

    #[test]
    fn resource_manager_overlay_adjusts_part_weights() {
        let catalog = policy_catalog();
        let baseline = catalog.resolve(PolicyId::Deep, StrategyId::Balanced);
        let resource = catalog.resolve(PolicyId::Deep, StrategyId::ResourceManager);
        assert!(
            resource.part_weights.pump > baseline.part_weights.pump,
            "resource manager should favor pump repairs"
        );
        assert!(
            resource.part_weights.tire < baseline.part_weights.tire,
            "resource manager should reduce tire breakdown weight"
        );
        assert!(
            resource.travel.weather_factor.get(&Weather::Storm).unwrap()
                > baseline.travel.weather_factor.get(&Weather::Storm).unwrap(),
            "resource manager should soften storm travel penalty"
        );
    }

    #[test]
    fn travel_day_kind_ratio_flag() {
        assert!(TravelDayKind::Travel.counts_toward_ratio());
        assert!(TravelDayKind::Partial.counts_toward_ratio());
        assert!(!TravelDayKind::NonTravel.counts_toward_ratio());
    }

    #[test]
    fn policy_conversion_roundtrip() {
        let classic = PolicyId::Classic;
        let deep = PolicyId::Deep;
        assert_eq!(PolicyKind::from(classic), PolicyKind::Balanced);
        assert_eq!(PolicyKind::from(deep), PolicyKind::Aggressive);
        assert_eq!(PolicyId::from(PolicyKind::Conservative), PolicyId::Classic);
        assert_eq!(PolicyId::from(PolicyKind::ResourceManager), PolicyId::Deep);
    }

    #[test]
    fn journey_config_partial_ratio_clamped() {
        let mut controller = JourneyController::with_config(
            PolicyId::Classic,
            StrategyId::Balanced,
            JourneyCfg {
                partial_ratio: 0.8,
                ..JourneyCfg::default()
            },
            42,
            EndgameTravelCfg::default_config(),
        );
        let mut state = GameState::default();
        let _ = controller.tick_day(&mut state);
        assert!((state.journey_partial_ratio - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn reseed_resets_rng_bundle() {
        let mut controller = JourneyController::new(PolicyId::Classic, StrategyId::Balanced, 1);
        controller.reseed(2);
        let mut state = GameState::default();
        let _ = controller.tick_day(&mut state);
    }

    #[test]
    fn rng_bundle_uses_domain_hmac() {
        let seed = 0xFEED_CAFE_u64;
        let bundle = RngBundle::from_user_seed(seed);

        let mut travel_rng = bundle.travel();
        let mut expected_travel = SmallRng::seed_from_u64(derive_stream_seed(seed, b"travel"));
        assert_eq!(travel_rng.next_u32(), expected_travel.next_u32());
        assert_eq!(travel_rng.draws(), 1);

        let mut breakdown_rng = bundle.breakdown();
        let mut expected_breakdown =
            SmallRng::seed_from_u64(derive_stream_seed(seed, b"breakdown"));
        assert_eq!(breakdown_rng.next_u64(), expected_breakdown.next_u64());

        assert_ne!(
            derive_stream_seed(seed, b"travel"),
            derive_stream_seed(seed, b"crossing"),
            "domain tags must derive distinct seeds"
        );
    }

    #[test]
    fn journey_config_rejects_invalid_ratio() {
        let cfg = JourneyCfg {
            partial_ratio: 1.2,
            ..JourneyCfg::default()
        };
        assert!(matches!(
            cfg.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "partial_ratio"
        ));
    }

    #[test]
    fn travel_bounds_validation_catches_min_above_max() {
        let cfg = JourneyCfg {
            travel: TravelConfig {
                mpd_min: 30.0,
                mpd_max: 10.0,
                ..TravelConfig::default()
            },
            ..JourneyCfg::default()
        };
        assert!(matches!(
            cfg.travel.validate(),
            Err(JourneyConfigError::TravelMinExceedsMax { .. })
        ));
    }

    #[test]
    fn journey_cfg_missing_fields_use_defaults() {
        let cfg: JourneyCfg = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(cfg, JourneyCfg::default());
        cfg.validate().expect("defaults are valid");
    }
}
