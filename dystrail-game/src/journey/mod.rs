//! Journey domain primitives shared by the controller and state ledger.
use rand::SeedableRng;
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::Hasher;
use std::rc::Rc;
use twox_hash::XxHash64;

use crate::endgame::EndgameTravelCfg;
use crate::state::{PaceId, PolicyKind};
use crate::weather::Weather;

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
            PolicyKind::Aggressive | PolicyKind::ResourceManager | PolicyKind::MonteCarlo => {
                Self::Deep
            }
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
    MonteCarlo,
}

/// Minimal journey configuration scaffold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyCfg {
    #[serde(default = "JourneyCfg::default_partial_ratio")]
    pub partial_ratio: f32,
    #[serde(default)]
    pub wear: WearConfig,
    #[serde(default)]
    pub breakdown: BreakdownConfig,
    #[serde(default)]
    pub part_weights: crate::vehicle::PartWeights,
}

impl JourneyCfg {
    #[must_use]
    pub const fn default_partial_ratio() -> f32 {
        0.5
    }
}

impl Default for JourneyCfg {
    fn default() -> Self {
        Self {
            partial_ratio: Self::default_partial_ratio(),
            wear: WearConfig::default(),
            breakdown: BreakdownConfig::default(),
            part_weights: crate::vehicle::PartWeights::default(),
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
        let root = hash64(seed.to_le_bytes(), 0);
        let travel = CountingRng::new(seed_domain(root, b"travel"));
        let breakdown = CountingRng::new(seed_domain(root, b"breakdown"));
        let encounter = CountingRng::new(seed_domain(root, b"encounter"));
        let crossing = CountingRng::new(seed_domain(root, b"crossing"));
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

fn hash64(bytes: [u8; 8], seed: u64) -> u64 {
    let mut hasher = XxHash64::with_seed(seed);
    hasher.write(&bytes);
    hasher.finish()
}

fn seed_domain(root: u64, tag: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(root);
    hasher.write(tag);
    hasher.finish()
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
        Self::with_config(
            policy,
            strategy,
            JourneyCfg::default(),
            seed,
            EndgameTravelCfg::default_config(),
        )
    }

    /// Create a new controller with explicit configuration and endgame settings.
    #[must_use]
    pub fn with_config(
        policy: PolicyId,
        strategy: StrategyId,
        cfg: JourneyCfg,
        seed: u64,
        endgame_cfg: EndgameTravelCfg,
    ) -> Self {
        let mut resolved_cfg = cfg;
        resolved_cfg.partial_ratio = resolved_cfg.partial_ratio.clamp(0.2, 0.95);
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

    /// Deterministically reseed controller-owned RNGs.
    pub fn reseed(&mut self, seed: u64) {
        self.rng = Rc::new(RngBundle::from_user_seed(seed));
    }

    /// Perform a single day tick using the current game state.
    #[must_use]
    pub fn tick_day(&mut self, state: &mut crate::state::GameState) -> DayOutcome {
        state.attach_rng_bundle(self.rng.clone());
        state.policy = Some(self.policy.into());
        state.journey_partial_ratio = self.cfg.partial_ratio;
        state.journey_wear = self.cfg.wear.clone();
        state.journey_breakdown = self.cfg.breakdown.clone();
        state.journey_part_weights = self.cfg.part_weights.clone();
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
                partial_ratio: 1.2,
                ..JourneyCfg::default()
            },
            42,
            EndgameTravelCfg::default_config(),
        );
        let mut state = GameState::default();
        let _ = controller.tick_day(&mut state);
        assert!((state.journey_partial_ratio - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn reseed_resets_rng_bundle() {
        let mut controller = JourneyController::new(PolicyId::Classic, StrategyId::Balanced, 1);
        controller.reseed(2);
        let mut state = GameState::default();
        let _ = controller.tick_day(&mut state);
    }
}
