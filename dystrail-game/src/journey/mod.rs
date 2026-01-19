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
use std::time::Instant;
use thiserror::Error;

use crate::endgame::EndgameTravelCfg;
use crate::state::{DietId, GameMode, PaceId, PolicyKind};
use crate::vehicle::PartWeights;
use crate::weather::Weather;

pub mod daily;
pub mod event;
pub mod kernel;
pub mod session;
pub use daily::DailyTickOutcome;
pub use event::{
    Event, EventDecisionTrace, EventId, EventKind, EventSeverity, RollValue, UiSurfaceHint,
    WeightFactor, WeightedCandidate,
};
pub(crate) use kernel::DailyTickKernel;
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

/// Mechanical ruleset selection for the simulation kernel.
///
/// This is distinct from strategy/difficulty overlays (`StrategyId`) and from the
/// legacy campaign families (`PolicyId`). It must be explicit so parity-critical
/// rules cannot drift via piecemeal mixing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MechanicalPolicyId {
    #[default]
    DystrailLegacy,
    OtDeluxe90s,
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
    pub strain: StrainConfig,
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
        self.strain.validate()?;
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
            strain: StrainConfig::default(),
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WearConfigOverlay {
    pub base: Option<f32>,
    pub fatigue_k: Option<f32>,
    pub comfort_miles: Option<f32>,
}

/// Partial overlay of breakdown parameters applied atop a resolved policy.
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

/// Derived general strain configuration (Dystrail parity scalar).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrainConfig {
    #[serde(default)]
    pub weights: StrainWeights,
    #[serde(default = "StrainConfig::default_weather_severity")]
    pub weather_severity: HashMap<Weather, f32>,
    #[serde(default)]
    pub exec_order_bonus: HashMap<String, f32>,
    #[serde(default = "StrainConfig::default_vehicle_wear_norm_denom")]
    pub vehicle_wear_norm_denom: f32,
    #[serde(default = "StrainConfig::default_strain_norm_denom")]
    pub strain_norm_denom: f32,
    #[serde(default)]
    pub label_bounds: StrainLabelBounds,
}

impl Default for StrainConfig {
    fn default() -> Self {
        Self {
            weights: StrainWeights::default(),
            weather_severity: Self::default_weather_severity(),
            exec_order_bonus: HashMap::new(),
            vehicle_wear_norm_denom: Self::default_vehicle_wear_norm_denom(),
            strain_norm_denom: Self::default_strain_norm_denom(),
            label_bounds: StrainLabelBounds::default(),
        }
    }
}

impl StrainConfig {
    const fn default_vehicle_wear_norm_denom() -> f32 {
        100.0
    }

    const fn default_strain_norm_denom() -> f32 {
        4.0
    }

    fn default_weather_severity() -> HashMap<Weather, f32> {
        HashMap::from([
            (Weather::Clear, 0.0),
            (Weather::Storm, 1.0),
            (Weather::HeatWave, 1.0),
            (Weather::ColdSnap, 0.6),
            (Weather::Smoke, 0.8),
        ])
    }

    pub fn sanitize(&mut self) {
        self.weights.sanitize();
        for value in self.weather_severity.values_mut() {
            if !value.is_finite() || *value < 0.0 {
                *value = 0.0;
            }
        }
        for value in self.exec_order_bonus.values_mut() {
            if !value.is_finite() || *value < 0.0 {
                *value = 0.0;
            }
        }
        if !self.vehicle_wear_norm_denom.is_finite() || self.vehicle_wear_norm_denom <= 0.0 {
            self.vehicle_wear_norm_denom = Self::default_vehicle_wear_norm_denom();
        }
        if !self.strain_norm_denom.is_finite() || self.strain_norm_denom <= 0.0 {
            self.strain_norm_denom = Self::default_strain_norm_denom();
        }
        self.label_bounds.sanitize();
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        self.weights.validate()?;
        for &value in self.weather_severity.values() {
            if value < 0.0 {
                return Err(JourneyConfigError::MinViolation {
                    field: "strain.weather_severity",
                    min: 0.0,
                    value,
                });
            }
        }
        for &value in self.exec_order_bonus.values() {
            if value < 0.0 {
                return Err(JourneyConfigError::MinViolation {
                    field: "strain.exec_order_bonus",
                    min: 0.0,
                    value,
                });
            }
        }
        if self.vehicle_wear_norm_denom <= 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field: "strain.vehicle_wear_norm_denom",
                min: f32::EPSILON,
                value: self.vehicle_wear_norm_denom,
            });
        }
        if self.strain_norm_denom <= 0.0 {
            return Err(JourneyConfigError::MinViolation {
                field: "strain.strain_norm_denom",
                min: f32::EPSILON,
                value: self.strain_norm_denom,
            });
        }
        self.label_bounds.validate()?;
        Ok(())
    }
}

/// Per-axis weights for computing general strain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrainWeights {
    #[serde(default = "StrainWeights::default_hp")]
    pub hp: f32,
    #[serde(default = "StrainWeights::default_sanity")]
    pub sanity: f32,
    #[serde(default = "StrainWeights::default_pants")]
    pub pants: f32,
    #[serde(default = "StrainWeights::default_starvation")]
    pub starvation: f32,
    #[serde(default = "StrainWeights::default_vehicle")]
    pub vehicle: f32,
    #[serde(default = "StrainWeights::default_weather")]
    pub weather: f32,
    #[serde(default = "StrainWeights::default_exec")]
    pub exec: f32,
}

impl Default for StrainWeights {
    fn default() -> Self {
        Self {
            hp: Self::default_hp(),
            sanity: Self::default_sanity(),
            pants: Self::default_pants(),
            starvation: Self::default_starvation(),
            vehicle: Self::default_vehicle(),
            weather: Self::default_weather(),
            exec: Self::default_exec(),
        }
    }
}

impl StrainWeights {
    const fn default_hp() -> f32 {
        0.1
    }

    const fn default_sanity() -> f32 {
        0.1
    }

    const fn default_pants() -> f32 {
        0.01
    }

    const fn default_starvation() -> f32 {
        0.2
    }

    const fn default_vehicle() -> f32 {
        1.0
    }

    const fn default_weather() -> f32 {
        1.0
    }

    const fn default_exec() -> f32 {
        1.0
    }

    fn sanitize(&mut self) {
        for value in [
            &mut self.hp,
            &mut self.sanity,
            &mut self.pants,
            &mut self.starvation,
            &mut self.vehicle,
            &mut self.weather,
            &mut self.exec,
        ] {
            if !value.is_finite() || *value < 0.0 {
                *value = 0.0;
            }
        }
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        for (field, value) in [
            ("strain.weights.hp", self.hp),
            ("strain.weights.sanity", self.sanity),
            ("strain.weights.pants", self.pants),
            ("strain.weights.starvation", self.starvation),
            ("strain.weights.vehicle", self.vehicle),
            ("strain.weights.weather", self.weather),
            ("strain.weights.exec", self.exec),
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

/// Label thresholds for general strain normalization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrainLabelBounds {
    #[serde(default = "StrainLabelBounds::default_good_max")]
    pub good_max: f32,
    #[serde(default = "StrainLabelBounds::default_fair_max")]
    pub fair_max: f32,
    #[serde(default = "StrainLabelBounds::default_poor_max")]
    pub poor_max: f32,
}

impl Default for StrainLabelBounds {
    fn default() -> Self {
        Self {
            good_max: Self::default_good_max(),
            fair_max: Self::default_fair_max(),
            poor_max: Self::default_poor_max(),
        }
    }
}

impl StrainLabelBounds {
    const fn default_good_max() -> f32 {
        0.25
    }

    const fn default_fair_max() -> f32 {
        0.5
    }

    const fn default_poor_max() -> f32 {
        0.75
    }

    const fn sanitize(&mut self) {
        if !self.good_max.is_finite() {
            self.good_max = Self::default_good_max();
        }
        if !self.fair_max.is_finite() {
            self.fair_max = Self::default_fair_max();
        }
        if !self.poor_max.is_finite() {
            self.poor_max = Self::default_poor_max();
        }
        self.good_max = self.good_max.clamp(0.0, 1.0);
        self.fair_max = self.fair_max.clamp(self.good_max, 1.0);
        self.poor_max = self.poor_max.clamp(self.fair_max, 1.0);
    }

    fn validate(&self) -> Result<(), JourneyConfigError> {
        for (field, value) in [
            ("strain.label_bounds.good_max", self.good_max),
            ("strain.label_bounds.fair_max", self.fair_max),
            ("strain.label_bounds.poor_max", self.poor_max),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(JourneyConfigError::RangeViolation {
                    field,
                    min: 0.0,
                    max: 1.0,
                    value,
                });
            }
        }
        if self.good_max > self.fair_max || self.fair_max > self.poor_max {
            return Err(JourneyConfigError::RangeViolation {
                field: "strain.label_bounds",
                min: 0.0,
                max: 1.0,
                value: self.good_max.max(self.fair_max).max(self.poor_max),
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

    fn sanitize(&mut self) {
        let _ = Instant::now();
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

    fn sanitize(&mut self) {
        let _ = Instant::now();
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

impl Eq for WearConfigOverlay {}
impl Eq for BreakdownConfigOverlay {}
impl Eq for PartWeightsOverlay {}
impl Eq for BribePolicy {}
impl Eq for CrossingPolicyOverlay {}
impl Eq for JourneyOverlay {}
impl Eq for AcceptanceGuardsOverlay {}
impl Eq for TravelConfigOverlay {}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AcceptanceGuardsOverlay {
    pub min_travel_ratio: Option<f32>,
    pub target_distance: Option<f32>,
    pub target_days_min: Option<u16>,
    pub target_days_max: Option<u16>,
}

/// Overlay of travel pacing parameters.
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
    pub fn new(
        families: HashMap<PolicyId, JourneyCfg>,
        overlays: HashMap<StrategyId, JourneyOverlay>,
    ) -> Self {
        let _ = Instant::now();
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
        resolved.daily.sanitize();
        resolved.strain.sanitize();
        resolved
    }

    #[must_use]
    pub fn families(&self) -> &HashMap<PolicyId, JourneyCfg> {
        let _ = Instant::now();
        &self.families
    }

    #[must_use]
    pub fn overlays(&self) -> &HashMap<StrategyId, JourneyOverlay> {
        let _ = Instant::now();
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
    pub day_consumed: bool,
    pub record: Option<DayRecord>,
    pub events: Vec<Event>,
    pub decision_traces: Vec<EventDecisionTrace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RngStream {
    Weather,
    Health,
    Travel,
    Events,
    Breakdown,
    Encounter,
    Crossing,
    Boss,
    Trade,
    Hunt,
}

impl RngStream {
    const fn bit(self) -> u16 {
        match self {
            Self::Weather => 1 << 0,
            Self::Health => 1 << 1,
            Self::Travel => 1 << 2,
            Self::Events => 1 << 3,
            Self::Breakdown => 1 << 4,
            Self::Encounter => 1 << 5,
            Self::Crossing => 1 << 6,
            Self::Boss => 1 << 7,
            Self::Trade => 1 << 8,
            Self::Hunt => 1 << 9,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RngStreamMask(u16);

impl RngStreamMask {
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

    pub(crate) const fn single(stream: RngStream) -> Self {
        Self(stream.bit())
    }

    pub(crate) const fn pair(first: RngStream, second: RngStream) -> Self {
        Self(first.bit() | second.bit())
    }

    pub(crate) const fn contains(self, stream: RngStream) -> bool {
        (self.0 & stream.bit()) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RngPhase {
    DailyEffects,
    ExecOrders,
    HealthTick,
    WeatherTick,
    VehicleBreakdown,
    EncounterTick,
    TravelTick,
    CrossingTick,
    BossTick,
    TradeTick,
    HuntTick,
}

impl RngPhase {
    pub(crate) const fn allowed_streams(self) -> RngStreamMask {
        match self {
            Self::DailyEffects | Self::ExecOrders => RngStreamMask::single(RngStream::Events),
            Self::HealthTick => RngStreamMask::single(RngStream::Health),
            Self::WeatherTick => RngStreamMask::single(RngStream::Weather),
            Self::VehicleBreakdown => RngStreamMask::single(RngStream::Breakdown),
            Self::EncounterTick => RngStreamMask::single(RngStream::Encounter),
            Self::TravelTick => RngStreamMask::pair(RngStream::Travel, RngStream::Events),
            Self::CrossingTick => RngStreamMask::single(RngStream::Crossing),
            Self::BossTick => RngStreamMask::single(RngStream::Boss),
            Self::TradeTick => RngStreamMask::single(RngStream::Trade),
            Self::HuntTick => RngStreamMask::single(RngStream::Hunt),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RngPhaseGuardState {
    phase: Option<RngPhase>,
    allowed: RngStreamMask,
}

impl Default for RngPhaseGuardState {
    fn default() -> Self {
        Self {
            phase: None,
            allowed: RngStreamMask::empty(),
        }
    }
}

pub(crate) struct RngPhaseGuard<'a> {
    bundle: &'a RngBundle,
    prev: RngPhaseGuardState,
    enabled: bool,
}

impl Drop for RngPhaseGuard<'_> {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        let mut state = self.bundle.phase_guard_state.borrow_mut();
        *state = self.prev;
    }
}

/// Deterministic bundle of RNG streams segregated by simulation domain.
#[derive(Debug, Clone)]
pub struct RngBundle {
    weather: RefCell<CountingRng<SmallRng>>,
    health: RefCell<CountingRng<SmallRng>>,
    travel: RefCell<CountingRng<SmallRng>>,
    events: RefCell<CountingRng<SmallRng>>,
    breakdown: RefCell<CountingRng<SmallRng>>,
    encounter: RefCell<CountingRng<SmallRng>>,
    crossing: RefCell<CountingRng<SmallRng>>,
    boss: RefCell<CountingRng<SmallRng>>,
    trade: RefCell<CountingRng<SmallRng>>,
    hunt: RefCell<CountingRng<SmallRng>>,
    phase_guard_state: RefCell<RngPhaseGuardState>,
}

impl RngBundle {
    /// Construct the bundle from a user-visible seed.
    #[must_use]
    pub fn from_user_seed(seed: u64) -> Self {
        let weather = CountingRng::new(derive_stream_seed(seed, b"weather"));
        let health = CountingRng::new(derive_stream_seed(seed, b"health"));
        let travel = CountingRng::new(derive_stream_seed(seed, b"travel"));
        let events = CountingRng::new(derive_stream_seed(seed, b"events"));
        let breakdown = CountingRng::new(derive_stream_seed(seed, b"breakdown"));
        let encounter = CountingRng::new(derive_stream_seed(seed, b"encounter"));
        let crossing = CountingRng::new(derive_stream_seed(seed, b"crossing"));
        let boss = CountingRng::new(derive_stream_seed(seed, b"boss"));
        let trade = CountingRng::new(derive_stream_seed(seed, b"trade"));
        let hunt = CountingRng::new(derive_stream_seed(seed, b"hunt"));
        Self {
            weather: RefCell::new(weather),
            health: RefCell::new(health),
            travel: RefCell::new(travel),
            events: RefCell::new(events),
            breakdown: RefCell::new(breakdown),
            encounter: RefCell::new(encounter),
            crossing: RefCell::new(crossing),
            boss: RefCell::new(boss),
            trade: RefCell::new(trade),
            hunt: RefCell::new(hunt),
            phase_guard_state: RefCell::new(RngPhaseGuardState::default()),
        }
    }

    /// Access the weather RNG stream.
    #[must_use]
    pub fn weather(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Weather);
        self.weather.borrow_mut()
    }

    /// Access the health RNG stream.
    #[must_use]
    pub fn health(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Health);
        self.health.borrow_mut()
    }

    /// Access the travel RNG stream.
    #[must_use]
    pub fn travel(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Travel);
        self.travel.borrow_mut()
    }

    /// Access the events RNG stream.
    #[must_use]
    pub fn events(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Events);
        self.events.borrow_mut()
    }

    /// Access the breakdown RNG stream.
    #[must_use]
    pub fn breakdown(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Breakdown);
        self.breakdown.borrow_mut()
    }

    /// Access the vehicle RNG stream (alias for breakdown/vehicle incidents).
    #[must_use]
    pub fn vehicle(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Breakdown);
        self.breakdown.borrow_mut()
    }

    /// Access the encounter RNG stream.
    #[must_use]
    pub fn encounter(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Encounter);
        self.encounter.borrow_mut()
    }

    /// Access the crossing RNG stream.
    #[must_use]
    pub fn crossing(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Crossing);
        self.crossing.borrow_mut()
    }

    /// Access the boss RNG stream.
    #[must_use]
    pub fn boss(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Boss);
        self.boss.borrow_mut()
    }

    /// Access the trade RNG stream.
    #[must_use]
    pub fn trade(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Trade);
        self.trade.borrow_mut()
    }

    /// Access the hunt RNG stream.
    #[must_use]
    pub fn hunt(&self) -> RefMut<'_, CountingRng<SmallRng>> {
        self.assert_stream_allowed(RngStream::Hunt);
        self.hunt.borrow_mut()
    }

    pub(crate) fn phase_guard_for(&self, phase: RngPhase) -> RngPhaseGuard<'_> {
        if !phase_guard_enabled() {
            return RngPhaseGuard {
                bundle: self,
                prev: RngPhaseGuardState::default(),
                enabled: false,
            };
        }
        let mut state = self.phase_guard_state.borrow_mut();
        let prev = *state;
        *state = RngPhaseGuardState {
            phase: Some(phase),
            allowed: phase.allowed_streams(),
        };
        RngPhaseGuard {
            bundle: self,
            prev,
            enabled: true,
        }
    }

    fn assert_stream_allowed(&self, stream: RngStream) {
        if !phase_guard_enabled() {
            return;
        }
        let state = self.phase_guard_state.borrow();
        let Some(phase) = state.phase else {
            return;
        };
        assert!(
            state.allowed.contains(stream),
            "RNG stream {:?} used during {:?}; allowed {:?}",
            stream,
            phase,
            state.allowed
        );
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

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.draws = self.draws.saturating_add(1);
        self.rng.try_fill_bytes(dest)
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

fn phase_guard_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| matches!(std::env::var("DYSTRAIL_RNG_PHASE_GUARD"), Ok(val) if val != "0"))
}

/// Shell journey controller; later phases will expand its responsibilities.
#[derive(Debug, Clone)]
pub struct JourneyController {
    mechanics: MechanicalPolicyId,
    policy: PolicyId,
    strategy: StrategyId,
    cfg: JourneyCfg,
    rng: Rc<RngBundle>,
    endgame_cfg: EndgameTravelCfg,
}

impl JourneyController {
    /// Create a new controller with default configuration.
    #[must_use]
    pub fn new(
        mechanics: MechanicalPolicyId,
        policy: PolicyId,
        strategy: StrategyId,
        seed: u64,
    ) -> Self {
        let cfg = policy_catalog().resolve(policy, strategy);
        Self::with_config(
            mechanics,
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
        mechanics: MechanicalPolicyId,
        policy: PolicyId,
        strategy: StrategyId,
        cfg: JourneyCfg,
        seed: u64,
        endgame_cfg: EndgameTravelCfg,
    ) -> Self {
        let resolved_cfg = normalize_cfg(cfg);
        Self {
            mechanics,
            policy,
            strategy,
            cfg: resolved_cfg,
            rng: Rc::new(RngBundle::from_user_seed(seed)),
            endgame_cfg,
        }
    }

    #[must_use]
    pub const fn mechanics(&self) -> MechanicalPolicyId {
        self.mechanics
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

    /// Replace the controller RNG bundle to keep sessions deterministic across state rehydration.
    pub fn set_rng_bundle(&mut self, bundle: Rc<RngBundle>) {
        self.rng = bundle;
    }

    /// Apply controller configuration to a game state before ticking.
    pub fn configure_state(&self, state: &mut crate::state::GameState) {
        state.attach_rng_bundle(self.rng.clone());
        state.mechanical_policy = self.mechanics;
        state.policy = Some(self.strategy.into());
        if self.mechanics == MechanicalPolicyId::DystrailLegacy {
            state.journey_partial_ratio = self.cfg.partial_ratio;
            state.trail_distance = self.cfg.victory_miles.max(1.0);
            state.journey_travel = self.cfg.travel.clone();
            state.journey_wear = self.cfg.wear.clone();
            state.journey_breakdown = self.cfg.breakdown.clone();
            state.journey_part_weights = self.cfg.part_weights.clone();
            state.journey_crossing = self.cfg.crossing.clone();
        }
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
        let kernel = DailyTickKernel::new(&self.cfg, &self.endgame_cfg);
        kernel.tick_day(state)
    }
}

fn normalize_cfg(mut cfg: JourneyCfg) -> JourneyCfg {
    cfg.validate().expect("valid journey config");
    cfg.partial_ratio = cfg.partial_ratio.clamp(0.2, 0.95);
    cfg.travel.sanitize();
    cfg.wear.base = cfg.wear.base.max(0.0);
    cfg.wear.fatigue_k = cfg.wear.fatigue_k.max(0.0);
    cfg.wear.comfort_miles = cfg.wear.comfort_miles.max(0.0);
    cfg.breakdown.base = cfg.breakdown.base.clamp(0.0, 1.0);
    cfg.breakdown.beta = cfg.breakdown.beta.max(0.0);
    cfg.breakdown.pace_factor.insert(
        PaceId::Steady,
        cfg.breakdown
            .pace_factor
            .get(&PaceId::Steady)
            .copied()
            .unwrap_or(crate::constants::PACE_BREAKDOWN_STEADY),
    );
    cfg.breakdown.pace_factor.insert(
        PaceId::Heated,
        cfg.breakdown
            .pace_factor
            .get(&PaceId::Heated)
            .copied()
            .unwrap_or(crate::constants::PACE_BREAKDOWN_HEATED),
    );
    cfg.breakdown.pace_factor.insert(
        PaceId::Blitz,
        cfg.breakdown
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
        let entry = cfg
            .breakdown
            .weather_factor
            .entry(weather)
            .or_insert(default);
        *entry = entry.max(0.0);
    }
    cfg.crossing.sanitize();
    cfg.daily.sanitize();
    cfg.strain.sanitize();
    cfg
}

pub(crate) fn resolve_cfg_for_state(state: &crate::state::GameState) -> JourneyCfg {
    let strategy = state.policy.map_or(StrategyId::Balanced, StrategyId::from);
    let policy = PolicyId::from(state.mode);
    let cfg = policy_catalog().resolve(policy, strategy);
    normalize_cfg(cfg)
}

pub(crate) fn tick_non_travel_day_for_state(
    state: &mut crate::state::GameState,
    kind: TravelDayKind,
    miles: f32,
    reason_tag: &str,
) -> f32 {
    let cfg = resolve_cfg_for_state(state);
    let kernel = DailyTickKernel::new(&cfg, default_endgame_config());
    kernel.tick_non_travel_day(state, kind, miles, reason_tag)
}

fn default_endgame_config() -> &'static EndgameTravelCfg {
    static CONFIG: OnceLock<EndgameTravelCfg> = OnceLock::new();
    CONFIG.get_or_init(EndgameTravelCfg::default_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Encounter, EncounterData};
    use crate::state::GameState;
    use crate::state::{FeatureFlags, RecentEncounter, Region};
    use crate::weather::Weather;
    use rand::RngCore;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::collections::{HashMap, VecDeque};

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
            MechanicalPolicyId::DystrailLegacy,
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
        controller.configure_state(&mut state);
        let _ = controller.tick_day(&mut state);
        assert!((state.journey_partial_ratio - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn reseed_resets_rng_bundle() {
        let mut controller = JourneyController::new(
            MechanicalPolicyId::DystrailLegacy,
            PolicyId::Classic,
            StrategyId::Balanced,
            1,
        );
        controller.reseed(2);
        let mut state = GameState::default();
        controller.configure_state(&mut state);
        let _ = controller.tick_day(&mut state);
    }

    #[test]
    fn tick_day_emits_events_with_stable_day_id_when_day_is_consumed() {
        let mut state = GameState {
            // Ensure encounter selection cannot early-return into the encounter UI flow.
            data: None,
            ..GameState::default()
        };

        let mut controller = JourneyController::new(
            MechanicalPolicyId::DystrailLegacy,
            PolicyId::Classic,
            StrategyId::Balanced,
            77,
        );

        controller.configure_state(&mut state);
        let outcome = controller.tick_day(&mut state);

        assert!(
            state.day >= 2,
            "expected day counter to advance after a completed day"
        );
        let record = outcome
            .record
            .as_ref()
            .expect("expected a day record when the day is finalized");
        let log_event = outcome
            .events
            .iter()
            .find(|event| event.ui_key.as_deref() == Some(outcome.log_key.as_str()))
            .expect("expected legacy log event");

        assert_eq!(log_event.day, u32::from(record.day_index) + 1);
        assert_eq!(log_event.id.day, log_event.day);
        assert_eq!(log_event.kind, EventKind::LegacyLogKey);

        for (idx, event) in outcome.events.iter().enumerate() {
            let expected = u16::try_from(idx).unwrap_or(u16::MAX);
            assert_eq!(
                event.id.seq, expected,
                "event sequence should be contiguous and ordered"
            );
        }
    }

    #[test]
    fn tick_day_surfaces_and_drains_decision_traces() {
        let encounter_data = EncounterData::from_encounters(vec![
            Encounter {
                id: String::from("alpha"),
                name: String::from("Alpha"),
                desc: String::new(),
                weight: 5,
                regions: vec![String::from("heartland")],
                modes: vec![String::from("classic")],
                choices: Vec::new(),
                hard_stop: false,
                major_repair: false,
                chainable: false,
            },
            Encounter {
                id: String::from("beta"),
                name: String::from("Beta"),
                desc: String::new(),
                weight: 5,
                regions: vec![String::from("heartland")],
                modes: vec![String::from("classic")],
                choices: Vec::new(),
                hard_stop: false,
                major_repair: false,
                chainable: false,
            },
            Encounter {
                id: String::from("gamma"),
                name: String::from("Gamma"),
                desc: String::new(),
                weight: 5,
                regions: vec![String::from("heartland")],
                modes: vec![String::from("classic")],
                choices: Vec::new(),
                hard_stop: false,
                major_repair: false,
                chainable: false,
            },
        ]);

        let base_state = GameState {
            day: 20,
            data: Some(encounter_data),
            region: Region::Heartland,
            pace: PaceId::Blitz,
            encounter_chance_today: 1.0,
            vehicle: crate::vehicle::Vehicle {
                health: crate::constants::VEHICLE_CRITICAL_THRESHOLD,
                ..crate::vehicle::Vehicle::default()
            },
            features: FeatureFlags {
                encounter_diversity: false,
                ..FeatureFlags::default()
            },
            recent_encounters: VecDeque::from(vec![
                RecentEncounter::new(String::from("alpha"), 19, Region::Heartland),
                RecentEncounter::new(String::from("beta"), 19, Region::Heartland),
                RecentEncounter::new(String::from("gamma"), 19, Region::Heartland),
            ]),
            ..GameState::default()
        };
        let mut hit = None;
        for seed in 0_u64..128 {
            let mut state = base_state.clone();
            let mut controller = JourneyController::new(
                MechanicalPolicyId::DystrailLegacy,
                PolicyId::Classic,
                StrategyId::Balanced,
                seed,
            );
            controller.configure_state(&mut state);
            let outcome = controller.tick_day(&mut state);
            if outcome.log_key == "log.encounter" {
                hit = Some((outcome, state));
                break;
            }
        }
        let (outcome, state) = hit.expect("expected encounter within seed window");
        assert!(
            !outcome.decision_traces.is_empty(),
            "expected encounter selection trace to be surfaced in the day outcome"
        );
        assert!(
            state.decision_traces_today.is_empty(),
            "expected decision traces to be drained from state after ticking"
        );
    }

    #[test]
    fn rng_bundle_uses_domain_hmac() {
        let seed = 0xFEED_CAFE_u64;
        let bundle = RngBundle::from_user_seed(seed);

        let mut weather_rng = bundle.weather();
        let mut expected_weather = SmallRng::seed_from_u64(derive_stream_seed(seed, b"weather"));
        assert_eq!(weather_rng.next_u32(), expected_weather.next_u32());
        assert_eq!(weather_rng.draws(), 1);

        let mut health_rng = bundle.health();
        let mut expected_health = SmallRng::seed_from_u64(derive_stream_seed(seed, b"health"));
        assert_eq!(health_rng.next_u32(), expected_health.next_u32());
        assert_eq!(health_rng.draws(), 1);

        let mut travel_rng = bundle.travel();
        let mut expected_travel = SmallRng::seed_from_u64(derive_stream_seed(seed, b"travel"));
        assert_eq!(travel_rng.next_u32(), expected_travel.next_u32());
        assert_eq!(travel_rng.draws(), 1);

        let mut events_rng = bundle.events();
        let mut expected_events = SmallRng::seed_from_u64(derive_stream_seed(seed, b"events"));
        assert_eq!(events_rng.next_u32(), expected_events.next_u32());
        assert_eq!(events_rng.draws(), 1);

        let mut breakdown_rng = bundle.breakdown();
        let mut expected_breakdown =
            SmallRng::seed_from_u64(derive_stream_seed(seed, b"breakdown"));
        assert_eq!(breakdown_rng.next_u64(), expected_breakdown.next_u64());

        let mut trade_rng = bundle.trade();
        let mut expected_trade = SmallRng::seed_from_u64(derive_stream_seed(seed, b"trade"));
        assert_eq!(trade_rng.next_u32(), expected_trade.next_u32());
        assert_eq!(trade_rng.draws(), 1);

        let mut boss_rng = bundle.boss();
        let mut expected_boss = SmallRng::seed_from_u64(derive_stream_seed(seed, b"boss"));
        assert_eq!(boss_rng.next_u32(), expected_boss.next_u32());
        assert_eq!(boss_rng.draws(), 1);

        let mut hunt_rng = bundle.hunt();
        let mut expected_hunt = SmallRng::seed_from_u64(derive_stream_seed(seed, b"hunt"));
        assert_eq!(hunt_rng.next_u32(), expected_hunt.next_u32());
        assert_eq!(hunt_rng.draws(), 1);

        assert_ne!(
            derive_stream_seed(seed, b"travel"),
            derive_stream_seed(seed, b"crossing"),
            "domain tags must derive distinct seeds"
        );
    }

    #[test]
    fn rng_phase_contract_matches_expected_streams() {
        assert_eq!(
            RngPhase::DailyEffects.allowed_streams(),
            RngStreamMask::single(RngStream::Events)
        );
        assert_eq!(
            RngPhase::ExecOrders.allowed_streams(),
            RngStreamMask::single(RngStream::Events)
        );
        assert_eq!(
            RngPhase::HealthTick.allowed_streams(),
            RngStreamMask::single(RngStream::Health)
        );
        assert_eq!(
            RngPhase::WeatherTick.allowed_streams(),
            RngStreamMask::single(RngStream::Weather)
        );
        assert_eq!(
            RngPhase::VehicleBreakdown.allowed_streams(),
            RngStreamMask::single(RngStream::Breakdown)
        );
        assert_eq!(
            RngPhase::EncounterTick.allowed_streams(),
            RngStreamMask::single(RngStream::Encounter)
        );
        assert_eq!(
            RngPhase::TravelTick.allowed_streams(),
            RngStreamMask::pair(RngStream::Travel, RngStream::Events)
        );
        assert_eq!(
            RngPhase::CrossingTick.allowed_streams(),
            RngStreamMask::single(RngStream::Crossing)
        );
        assert_eq!(
            RngPhase::BossTick.allowed_streams(),
            RngStreamMask::single(RngStream::Boss)
        );
        assert_eq!(
            RngPhase::TradeTick.allowed_streams(),
            RngStreamMask::single(RngStream::Trade)
        );
        assert_eq!(
            RngPhase::HuntTick.allowed_streams(),
            RngStreamMask::single(RngStream::Hunt)
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

    #[test]
    fn day_tags_trim_and_ignore_empty() {
        let tag = DayTag::new("  camp  ");
        assert_eq!(tag.0, "camp");
        assert!(DayTag::new("   ").is_empty());
    }

    #[test]
    fn day_record_push_tag_ignores_duplicates() {
        let mut record = DayRecord::new(1, TravelDayKind::Travel, 12.0);
        record.push_tag(DayTag::new("camp"));
        record.push_tag(DayTag::new("camp"));
        record.push_tag(DayTag::new(" "));
        assert_eq!(record.tags.len(), 1);
        assert_eq!(record.tags[0].0, "camp");
    }

    #[test]
    fn journey_config_rejects_invalid_victory_miles() {
        let cfg = JourneyCfg {
            victory_miles: 200.0,
            ..JourneyCfg::default()
        };
        assert!(matches!(
            cfg.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "victory_miles"
        ));
    }

    #[test]
    fn normalize_cfg_clamps_breakdown_and_defaults() {
        let cfg = JourneyCfg {
            partial_ratio: 0.9,
            breakdown: BreakdownConfig {
                base: 0.2,
                beta: 0.3,
                pace_factor: HashMap::new(),
                weather_factor: HashMap::new(),
            },
            ..JourneyCfg::default()
        };
        let normalized = normalize_cfg(cfg);
        assert!(normalized.partial_ratio >= 0.2);
        assert!(normalized.partial_ratio <= 0.95);
        assert!(normalized.breakdown.base >= 0.0);
        assert!(
            normalized
                .breakdown
                .pace_factor
                .contains_key(&PaceId::Steady)
        );
        assert!(
            normalized
                .breakdown
                .weather_factor
                .contains_key(&Weather::Clear)
        );
    }

    #[test]
    fn crossing_policy_sanitize_normalizes_and_dedups() {
        let mut policy = CrossingPolicy {
            pass: -1.0,
            detour: -2.0,
            terminal: -3.0,
            detour_days: DetourPolicy { min: 0, max: 0 },
            bribe: BribePolicy {
                pass_bonus: 2.0,
                detour_bonus: -2.0,
                terminal_penalty: 2.0,
                diminishing_returns: 2.0,
            },
            permit: PermitPolicy {
                disable_terminal: true,
                eligible: vec!["checkpoint".to_string(), "checkpoint".to_string()],
            },
        };

        policy.sanitize();

        let total = policy.pass + policy.detour + policy.terminal;
        assert!((total - 1.0).abs() <= 1e-6);
        assert!(policy.pass > 0.0);
        assert_eq!(policy.detour_days.min, 1);
        assert_eq!(policy.detour_days.max, 1);
        assert!(policy.bribe.pass_bonus <= 0.9 + f32::EPSILON);
        assert!(policy.bribe.detour_bonus >= -0.9 - f32::EPSILON);
        assert!(policy.bribe.diminishing_returns <= 1.0);
        assert_eq!(policy.permit.eligible.len(), 1);
    }

    #[test]
    fn daily_channel_sanitize_resets_invalid_values() {
        let mut channel = DailyChannelConfig {
            base: -1.0,
            pace: HashMap::from([(PaceId::Steady, 0.0)]),
            diet: HashMap::from([(DietId::Mixed, f32::NAN)]),
            weather: HashMap::from([(Weather::Clear, -1.0)]),
            exec: HashMap::from([(String::from("order"), 0.0)]),
        };

        channel.sanitize();

        assert!((channel.base - 0.0).abs() <= f32::EPSILON);
        assert!((channel.pace[&PaceId::Steady] - 1.0).abs() <= f32::EPSILON);
        assert!((channel.diet[&DietId::Mixed] - 1.0).abs() <= f32::EPSILON);
        assert!((channel.weather[&Weather::Clear] - 1.0).abs() <= f32::EPSILON);
        assert!((channel.exec["order"] - 1.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn daily_channel_validation_rejects_negative_base() {
        let channel = DailyChannelConfig {
            base: -0.5,
            ..DailyChannelConfig::default()
        };
        assert!(matches!(
            channel.validate("daily.supplies"),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "daily.supplies"
        ));
    }

    #[test]
    fn health_tick_sanitize_clamps_values() {
        let mut cfg = HealthTickConfig {
            decay: -1.0,
            rest_heal: f32::NAN,
            weather: HashMap::from([(Weather::Storm, -2.0)]),
            exec: HashMap::from([(String::from("order"), f32::NAN)]),
        };

        cfg.sanitize();

        assert!((cfg.decay - 0.0).abs() <= f32::EPSILON);
        assert!((cfg.rest_heal - 0.0).abs() <= f32::EPSILON);
        assert!((cfg.weather[&Weather::Storm] - 0.0).abs() <= f32::EPSILON);
        assert!((cfg.exec["order"] - 0.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn strain_config_sanitize_clamps_values() {
        let mut cfg = StrainConfig {
            weights: StrainWeights {
                hp: -1.0,
                sanity: f32::NAN,
                pants: -2.0,
                starvation: 0.2,
                vehicle: f32::NAN,
                weather: -3.0,
                exec: 1.0,
            },
            weather_severity: HashMap::from([(Weather::Clear, -1.0)]),
            exec_order_bonus: HashMap::from([(String::from("order"), f32::NAN)]),
            vehicle_wear_norm_denom: 0.0,
            strain_norm_denom: -1.0,
            label_bounds: StrainLabelBounds {
                good_max: f32::NAN,
                fair_max: -1.0,
                poor_max: 2.0,
            },
        };

        cfg.sanitize();

        assert!(cfg.weights.hp >= 0.0);
        assert!(cfg.weights.pants >= 0.0);
        assert!(cfg.weather_severity[&Weather::Clear] >= 0.0);
        assert!(cfg.exec_order_bonus["order"] >= 0.0);
        assert!(cfg.vehicle_wear_norm_denom > 0.0);
        assert!(cfg.strain_norm_denom > 0.0);
        assert!(cfg.label_bounds.good_max <= cfg.label_bounds.fair_max);
        assert!(cfg.label_bounds.fair_max <= cfg.label_bounds.poor_max);
    }

    #[test]
    fn strain_config_validation_rejects_negative_weight() {
        let cfg = StrainConfig {
            weights: StrainWeights {
                hp: -0.1,
                ..StrainWeights::default()
            },
            ..StrainConfig::default()
        };
        assert!(matches!(
            cfg.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "strain.weights.hp"
        ));
    }

    #[test]
    fn acceptance_guards_validation_rejects_out_of_range() {
        let guards = AcceptanceGuards {
            min_travel_ratio: 0.3,
            ..AcceptanceGuards::default()
        };
        assert!(matches!(
            guards.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "guards.min_travel_ratio"
        ));
    }

    #[test]
    fn strain_label_bounds_validation_rejects_invalid_order() {
        let bounds = StrainLabelBounds {
            good_max: 0.8,
            fair_max: 0.5,
            poor_max: 0.6,
        };
        assert!(matches!(
            bounds.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "strain.label_bounds"
        ));
    }

    #[test]
    fn crossing_policy_validation_rejects_negative_weights() {
        let policy = CrossingPolicy {
            pass: -0.1,
            ..CrossingPolicy::default()
        };
        assert!(matches!(
            policy.validate(),
            Err(JourneyConfigError::CrossingProbabilities { .. })
        ));
    }

    #[test]
    fn travel_validation_rejects_base_out_of_range() {
        let config = TravelConfig {
            mpd_min: 10.0,
            mpd_max: 20.0,
            mpd_base: 30.0,
            ..TravelConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "travel.mpd_base"
        ));
    }

    #[test]
    fn travel_validation_rejects_pace_multiplier_below_floor() {
        let config = TravelConfig {
            pace_factor: HashMap::from([(PaceId::Steady, 0.0)]),
            ..TravelConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "travel.pace_factor"
        ));
    }

    #[test]
    fn travel_validation_rejects_weather_multiplier_below_floor() {
        let config = TravelConfig {
            weather_factor: HashMap::from([(Weather::Clear, 0.0)]),
            ..TravelConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "travel.weather_factor"
        ));
    }

    #[test]
    fn wear_validation_rejects_negative_values() {
        let config = WearConfig {
            base: -1.0,
            ..WearConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "wear.base"
        ));
    }

    #[test]
    fn breakdown_validation_rejects_base_out_of_range() {
        let config = BreakdownConfig {
            base: 2.0,
            ..BreakdownConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "breakdown.base"
        ));
    }

    #[test]
    fn breakdown_validation_rejects_negative_beta() {
        let config = BreakdownConfig {
            beta: -0.1,
            ..BreakdownConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "breakdown.beta"
        ));
    }

    #[test]
    fn breakdown_validation_rejects_negative_pace_factor() {
        let config = BreakdownConfig {
            pace_factor: HashMap::from([(PaceId::Steady, -1.0)]),
            ..BreakdownConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "breakdown.pace_factor"
        ));
    }

    #[test]
    fn breakdown_validation_rejects_negative_weather_factor() {
        let config = BreakdownConfig {
            weather_factor: HashMap::from([(Weather::Clear, -1.0)]),
            ..BreakdownConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "breakdown.weather_factor"
        ));
    }

    #[test]
    fn health_tick_validation_rejects_negative_decay() {
        let config = HealthTickConfig {
            decay: -0.5,
            ..HealthTickConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "daily.health.decay"
        ));
    }

    #[test]
    fn health_tick_validation_rejects_negative_rest_heal() {
        let config = HealthTickConfig {
            rest_heal: -1.0,
            ..HealthTickConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "daily.health.rest_heal"
        ));
    }

    #[test]
    fn strain_validation_rejects_negative_weather_severity() {
        let config = StrainConfig {
            weather_severity: HashMap::from([(Weather::Storm, -1.0)]),
            ..StrainConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "strain.weather_severity"
        ));
    }

    #[test]
    fn strain_validation_rejects_negative_exec_bonus() {
        let config = StrainConfig {
            exec_order_bonus: HashMap::from([(String::from("order"), -1.0)]),
            ..StrainConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "strain.exec_order_bonus"
        ));
    }

    #[test]
    fn strain_validation_rejects_non_positive_vehicle_norm() {
        let config = StrainConfig {
            vehicle_wear_norm_denom: 0.0,
            ..StrainConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "strain.vehicle_wear_norm_denom"
        ));
    }

    #[test]
    fn strain_validation_rejects_non_positive_strain_norm() {
        let config = StrainConfig {
            strain_norm_denom: 0.0,
            ..StrainConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "strain.strain_norm_denom"
        ));
    }

    #[test]
    fn acceptance_guards_validation_rejects_target_distance() {
        let guards = AcceptanceGuards {
            target_distance: 0.0,
            ..AcceptanceGuards::default()
        };
        assert!(matches!(
            guards.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "guards.target_distance"
        ));
    }

    #[test]
    fn acceptance_guards_validation_rejects_target_days_min() {
        let guards = AcceptanceGuards {
            target_days_min: 0,
            ..AcceptanceGuards::default()
        };
        assert!(matches!(
            guards.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "guards.target_days_min"
        ));
    }

    #[test]
    fn acceptance_guards_validation_rejects_days_range() {
        let guards = AcceptanceGuards {
            target_days_min: 10,
            target_days_max: 5,
            ..AcceptanceGuards::default()
        };
        assert!(matches!(
            guards.validate(),
            Err(JourneyConfigError::GuardDaysRange { .. })
        ));
    }

    #[test]
    fn crossing_policy_validation_rejects_detour_bounds() {
        let policy = CrossingPolicy {
            detour_days: DetourPolicy { min: 3, max: 1 },
            ..CrossingPolicy::default()
        };
        assert!(matches!(
            policy.validate(),
            Err(JourneyConfigError::CrossingDetourBounds { .. })
        ));
    }

    #[test]
    fn crossing_policy_validation_rejects_zero_sum() {
        let policy = CrossingPolicy {
            pass: 0.0,
            detour: 0.0,
            terminal: 0.0,
            ..CrossingPolicy::default()
        };
        assert!(matches!(
            policy.validate(),
            Err(JourneyConfigError::CrossingProbabilities { .. })
        ));
    }

    #[test]
    fn strategy_conversion_covers_all_variants() {
        assert_eq!(
            PolicyKind::from(StrategyId::Conservative),
            PolicyKind::Conservative
        );
        assert_eq!(
            PolicyKind::from(StrategyId::ResourceManager),
            PolicyKind::ResourceManager
        );
        assert_eq!(
            StrategyId::from(PolicyKind::Conservative),
            StrategyId::Conservative
        );
        assert_eq!(
            StrategyId::from(PolicyKind::ResourceManager),
            StrategyId::ResourceManager
        );
    }

    #[test]
    fn acceptance_guards_overlay_applies_fields() {
        let guards = AcceptanceGuards::default();
        let overlay = AcceptanceGuardsOverlay {
            min_travel_ratio: Some(0.95),
            target_distance: Some(1500.0),
            target_days_min: Some(90),
            target_days_max: Some(120),
        };

        let merged = guards.with_overlay(&overlay);

        assert!((merged.min_travel_ratio - 0.95).abs() <= f32::EPSILON);
        assert!((merged.target_distance - 1500.0).abs() <= f32::EPSILON);
        assert_eq!(merged.target_days_min, 90);
        assert_eq!(merged.target_days_max, 120);
    }

    #[test]
    fn travel_validation_rejects_min_below_floor() {
        let config = TravelConfig {
            mpd_min: 0.0,
            ..TravelConfig::default()
        };
        assert!(matches!(
            config.validate(),
            Err(JourneyConfigError::MinViolation { field, .. }) if field == "travel.mpd_min"
        ));
    }

    #[test]
    fn crossing_policy_overlay_applies_detour_days() {
        let policy = CrossingPolicy::default();
        let overlay = CrossingPolicyOverlay {
            detour_days: Some(DetourPolicy { min: 2, max: 4 }),
            ..CrossingPolicyOverlay::default()
        };

        let merged = policy.with_overlay(&overlay);

        assert_eq!(merged.detour_days.min, 2);
        assert_eq!(merged.detour_days.max, 4);
    }

    #[test]
    fn travel_config_sanitize_restores_invalid_base() {
        let mut config = TravelConfig {
            mpd_base: 0.0,
            mpd_min: 10.0,
            mpd_max: 20.0,
            ..TravelConfig::default()
        };

        config.sanitize();

        assert!(config.mpd_base >= config.mpd_min);
    }

    #[test]
    fn breakdown_overlay_applies_weather_map() {
        let base = BreakdownConfig::default();
        let overlay = BreakdownConfigOverlay {
            weather_factor: Some(HashMap::from([(Weather::Smoke, 1.9)])),
            ..BreakdownConfigOverlay::default()
        };

        let merged = base.with_overlay(&overlay);

        assert!((merged.weather_factor[&Weather::Smoke] - 1.9).abs() <= f32::EPSILON);
    }

    #[test]
    fn journey_cfg_merge_overlay_applies_guard_overrides() {
        let base = JourneyCfg::default();
        let overlay = JourneyOverlay {
            guards: Some(AcceptanceGuardsOverlay {
                min_travel_ratio: Some(0.95),
                target_distance: Some(1500.0),
                target_days_min: Some(90),
                target_days_max: Some(110),
            }),
            ..JourneyOverlay::default()
        };

        let merged = base.merge_overlay(&overlay);

        assert!((merged.guards.min_travel_ratio - 0.95).abs() <= f32::EPSILON);
        assert!((merged.guards.target_distance - 1500.0).abs() <= f32::EPSILON);
        assert_eq!(merged.guards.target_days_min, 90);
        assert_eq!(merged.guards.target_days_max, 110);
    }

    #[test]
    fn policy_catalog_resolve_falls_back_to_defaults() {
        let catalog = PolicyCatalog::new(HashMap::new(), HashMap::new());
        let resolved = catalog.resolve(PolicyId::Classic, StrategyId::Balanced);

        assert_eq!(resolved, JourneyCfg::default());
        assert!(catalog.families().is_empty());
        assert!(catalog.overlays().is_empty());
    }

    #[test]
    fn policy_catalog_resolve_panics_on_invalid_config() {
        let invalid = JourneyCfg {
            partial_ratio: 1.2,
            ..JourneyCfg::default()
        };
        let catalog = PolicyCatalog::new(
            HashMap::from([(PolicyId::Classic, invalid)]),
            HashMap::new(),
        );

        let result = std::panic::catch_unwind(|| {
            let _ = catalog.resolve(PolicyId::Classic, StrategyId::Balanced);
        });

        assert!(result.is_err());
    }

    #[test]
    fn rng_stream_mask_contains_bits() {
        let mask = RngStreamMask::pair(RngStream::Travel, RngStream::Events);
        assert!(mask.contains(RngStream::Travel));
        assert!(!mask.contains(RngStream::Health));
    }

    #[test]
    fn rng_bundle_vehicle_accessor_returns_breakdown_stream() {
        let bundle = RngBundle::from_user_seed(7);
        {
            let mut vehicle_rng = bundle.vehicle();
            let _ = vehicle_rng.next_u32();
        }
        let breakdown_draws = bundle.breakdown().draws();
        assert_eq!(breakdown_draws, 1);
    }

    #[test]
    fn counting_rng_tracks_fill_bytes_calls() {
        let bundle = RngBundle::from_user_seed(4242);
        let mut buffer = [0_u8; 8];
        {
            let mut rng = bundle.weather();
            let draws_before = rng.draws();
            rng.fill_bytes(&mut buffer);
            assert_eq!(rng.draws(), draws_before + 1);
        }
        let mut buffer = [0_u8; 4];
        {
            let mut rng = bundle.weather();
            let draws_before = rng.draws();
            rng.try_fill_bytes(&mut buffer).expect("fill bytes");
            assert_eq!(rng.draws(), draws_before + 1);
        }
    }

    #[test]
    fn strain_label_bounds_sanitize_replaces_non_finite() {
        let mut bounds = StrainLabelBounds {
            good_max: f32::NAN,
            fair_max: f32::NAN,
            poor_max: f32::NAN,
        };

        bounds.sanitize();

        assert!(bounds.good_max.is_finite());
        assert!(bounds.fair_max.is_finite());
        assert!(bounds.poor_max.is_finite());
    }

    #[test]
    fn strain_label_bounds_validation_rejects_out_of_range() {
        let bounds = StrainLabelBounds {
            good_max: 1.2,
            fair_max: 0.5,
            poor_max: 0.6,
        };

        assert!(matches!(
            bounds.validate(),
            Err(JourneyConfigError::RangeViolation { field, .. }) if field == "strain.label_bounds.good_max"
        ));
    }
}
