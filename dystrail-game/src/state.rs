use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::str::FromStr;

use crate::camp::CampState;
use crate::data::{Encounter, EncounterData};
use crate::encounters::pick_encounter;
use crate::exec_orders::ExecOrder;
use crate::personas::{Persona, PersonaMods};
use crate::vehicle::{Breakdown, Part, Vehicle};
use crate::weather::{Weather, WeatherConfig, WeatherState};

const DEBUG_ENV_VAR: &str = "DYSTRAIL_DEBUG_LOGS";
const LOG_PANTS_EMERGENCY: &str = "log.pants-emergency";
const LOG_HEALTH_COLLAPSE: &str = "log.health-collapse";
const LOG_SANITY_COLLAPSE: &str = "log.sanity-collapse";
const LOG_TRAVEL_BLOCKED: &str = "log.travel-blocked";
const LOG_TRAVELED: &str = "log.traveled";
const DEFAULT_SUPPLY_COST: i32 = 1;
const BLITZ_SUPPLY_COST: i32 = 2;
const VEHICLE_BREAKDOWN_DAMAGE: f32 = 8.0;
const VEHICLE_DAILY_WEAR: f32 = 0.2;
const VEHICLE_CRITICAL_THRESHOLD: f32 = 20.0;
const VEHICLE_HEALTH_MAX: f32 = 100.0;
const VEHICLE_BREAKDOWN_WEAR: f32 = 8.0;
const VEHICLE_EMERGENCY_HEAL: f32 = 6.0;
const VEHICLE_JURY_RIG_HEAL: f32 = 4.0;
const VEHICLE_BREAKDOWN_TOLERANCE_FLOOR: i32 = 5;
const STARVATION_BASE_HP_LOSS: i32 = 1;
const STARVATION_SANITY_LOSS: i32 = 1;
const STARVATION_PANTS_GAIN: i32 = 1;
const STARVATION_MAX_STACK: u32 = 5;
const ALLY_ATTRITION_CHANCE: f32 = 0.02;
const EMERGENCY_REPAIR_COST: i64 = 1_000;
const ENCOUNTER_BASE_DEFAULT: f32 = 0.22;
const ENCOUNTER_COOLDOWN_DAYS: u8 = 1;
const ENCOUNTER_SOFT_CAP_THRESHOLD: u32 = 5;
const ENCOUNTER_HISTORY_WINDOW: usize = 10;
const MAX_ENCOUNTERS_PER_DAY: u8 = 2;
const EXEC_ORDER_DAILY_CHANCE: f32 = 0.08;
const EXEC_ORDER_MIN_DURATION: u8 = 3;
const EXEC_ORDER_MAX_DURATION: u8 = 6;
const EXEC_ORDER_MIN_COOLDOWN: u8 = 5;
const EXEC_ORDER_MAX_COOLDOWN: u8 = 10;
const LOG_EXEC_START_PREFIX: &str = "exec.start.";
const LOG_EXEC_END_PREFIX: &str = "exec.end.";
const LOG_STARVATION_TICK: &str = "log.starvation.tick";
const LOG_STARVATION_RELIEF: &str = "log.starvation.relief";
const LOG_ALLY_LOST: &str = "log.ally.lost";
const LOG_ALLIES_GONE: &str = "log.allies.gone";
const LOG_VEHICLE_FAILURE: &str = "log.vehicle.failure";
const LOG_VEHICLE_REPAIR_EMERGENCY: &str = "log.vehicle.repair.emergency";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PaceId {
    #[default]
    Steady,
    Heated,
    Blitz,
}

impl PaceId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steady => "steady",
            Self::Heated => "heated",
            Self::Blitz => "blitz",
        }
    }
}

impl fmt::Display for PaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PaceId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "steady" => Ok(Self::Steady),
            "heated" => Ok(Self::Heated),
            "blitz" => Ok(Self::Blitz),
            _ => Err(()),
        }
    }
}

impl From<PaceId> for String {
    fn from(value: PaceId) -> Self {
        value.as_str().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DietId {
    #[default]
    Mixed,
    Quiet,
    Doom,
}

impl DietId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::Quiet => "quiet",
            Self::Doom => "doom",
        }
    }
}

impl fmt::Display for DietId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DietId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mixed" => Ok(Self::Mixed),
            "quiet" => Ok(Self::Quiet),
            "doom" => Ok(Self::Doom),
            _ => Err(()),
        }
    }
}

impl From<DietId> for String {
    fn from(value: DietId) -> Self {
        value.as_str().to_string()
    }
}

#[cfg(debug_assertions)]
fn debug_log_enabled() -> bool {
    matches!(std::env::var(DEBUG_ENV_VAR), Ok(val) if val != "0")
}

#[cfg(not(debug_assertions))]
const fn debug_log_enabled() -> bool {
    false
}

/// Default pace setting
fn default_pace() -> PaceId {
    PaceId::Steady
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Choice, Effects, Encounter};
    use crate::weather::Weather;

    #[test]
    fn breakdown_consumes_spare_and_clears_block() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.inventory.spares.tire = 1;
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.travel_blocked = true;
        state.rng = Some(ChaCha20Rng::seed_from_u64(1));
        state.data = Some(EncounterData::empty());

        let (_ended, _msg, _started) = state.travel_next_leg();

        assert_eq!(state.inventory.spares.tire, 0);
        assert!(!state.travel_blocked);
        assert!(state.breakdown.is_none());
    }

    #[test]
    fn breakdown_without_spare_resolves_after_stall() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.breakdown = Some(Breakdown {
            part: Part::Battery,
            day_started: 1,
        });
        state.travel_blocked = true;
        state.rng = Some(ChaCha20Rng::seed_from_u64(2));
        state.data = Some(EncounterData::empty());

        let (_ended_first, msg_first, _started_first) = state.travel_next_leg();
        assert_eq!(msg_first, "log.traveled");
        assert!(!state.travel_blocked);
        assert!(state.breakdown.is_none());
        assert!(
            state
                .logs
                .iter()
                .any(|entry| entry == LOG_VEHICLE_REPAIR_EMERGENCY)
        );
        assert_eq!(state.budget_cents, 9_000);
        assert_eq!(state.budget, 90);
        assert_eq!(state.repairs_spent_cents, EMERGENCY_REPAIR_COST);
    }

    #[test]
    fn exec_order_drain_clamped_to_zero() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.stats.supplies = 0;
        state.stats.sanity = 0;
        state.rng = Some(ChaCha20Rng::seed_from_u64(3));
        state.encounter_chance_today = 0.0;
        state.data = Some(EncounterData::empty());

        let (_ended, _msg, _started) = state.travel_next_leg();

        assert!(state.stats.supplies >= 0, "supplies went negative");
        assert!(state.stats.sanity >= 0, "sanity went negative");
    }

    #[test]
    fn exec_order_expires_and_sets_cooldown() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.current_order = Some(ExecOrder::Shutdown);
        state.exec_order_days_remaining = 1;
        state.exec_order_cooldown = 0;
        state.rng = None;
        let supplies_before = state.stats.supplies;
        let morale_before = state.stats.morale;

        state.start_of_day();

        assert!(state.current_order.is_none());
        assert_eq!(state.exec_order_cooldown, EXEC_ORDER_MIN_COOLDOWN);
        let end_log = format!("{}{}", LOG_EXEC_END_PREFIX, ExecOrder::Shutdown.key());
        assert!(state.logs.iter().any(|entry| entry == &end_log));
        assert!(state.stats.supplies < supplies_before);
        assert!(state.stats.morale < morale_before);
    }

    #[test]
    fn starvation_stacks_damage() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.stats.supplies = 0;

        state.apply_starvation_tick();
        assert_eq!(state.stats.hp, 8);
        assert_eq!(state.malnutrition_level, 1);

        state.apply_starvation_tick();
        assert_eq!(state.stats.hp, 5);
        assert_eq!(state.malnutrition_level, 2);
        assert!(state.logs.iter().any(|entry| entry == LOG_STARVATION_TICK));
    }

    #[test]
    fn vehicle_terminal_sets_ending() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.vehicle_breakdowns = 10;
        state.vehicle.health = 0.0;
        state.inventory.spares = Spares::default();
        state.budget_cents = 0;
        assert!(state.check_vehicle_terminal_state());
        assert!(matches!(
            state.ending,
            Some(Ending::VehicleFailure {
                cause: VehicleFailureCause::Destroyed
            })
        ));
    }

    #[test]
    fn starvation_sets_hunger_collapse() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.stats.supplies = 0;
        state.stats.hp = 1;
        state.start_of_day();
        state.failure_log_key();
        assert!(matches!(
            state.ending,
            Some(Ending::Collapse {
                cause: CollapseCause::Hunger
            })
        ));
    }

    #[test]
    fn exposure_sets_kind() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.stats.supplies = 10;
        state.stats.hp = 1;
        state.weather_state.today = Weather::ColdSnap;
        state.weather_state.yesterday = Weather::Clear;
        state.start_of_day();
        state.failure_log_key();
        assert!(matches!(
            state.ending,
            Some(Ending::Exposure {
                kind: ExposureKind::Cold
            })
        ));
    }

    #[test]
    fn steady_clear_progress_is_sane() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.rng = None;
        state.pace = PaceId::Steady;
        let pacing = crate::pacing::PacingConfig::default_config();
        for _ in 0..30 {
            state.start_of_day();
            state.weather_state.today = Weather::Clear;
            state.weather_state.yesterday = Weather::Clear;
            state.apply_pace_and_diet(&pacing);
            state.encounter_chance_today = 0.0;
            let (ended, _, _) = state.travel_next_leg();
            assert!(!ended, "run ended prematurely");
        }
        assert!(state.travel_days >= 30);
        let travel_days = state.travel_days.max(1);
        let avg_mpd = if state.travel_days > 0 {
            f64::from(state.distance_traveled_actual) / f64::from(travel_days)
        } else {
            0.0
        };
        assert!(avg_mpd >= 12.0, "average miles per day {avg_mpd:.2}");
    }

    #[test]
    fn no_miles_on_camp() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.rng = None;
        for _ in 0..5 {
            state.advance_days(1);
        }
        assert!(state.distance_traveled_actual.abs() <= f32::EPSILON);
        assert_eq!(state.travel_days, 0);
        assert_eq!(state.non_travel_days, 5);
    }

    #[test]
    fn encounter_soft_cap_reduces_chance() {
        #![allow(clippy::field_reassign_with_default)]
        let cfg = crate::pacing::PacingConfig::default_config();

        let mut base_state = GameState::default();
        base_state.rng = None;
        base_state.apply_pace_and_diet(&cfg);
        let base = base_state.encounter_chance_today;
        assert!((f64::from(base) - f64::from(ENCOUNTER_BASE_DEFAULT)).abs() < 1e-6);

        let mut capped_state = GameState::default();
        capped_state.rng = None;
        capped_state.encounter_history = VecDeque::from(vec![2, 1, 1, 1, 0, 0, 0, 0, 0]);
        capped_state.apply_pace_and_diet(&cfg);
        let capped = capped_state.encounter_chance_today;
        assert!(
            (f64::from(capped) - f64::from(base) * 0.5).abs() < 1e-6,
            "expected soft cap to halve encounter chance (base {base}, capped {capped})"
        );
    }

    #[test]
    fn max_two_encounters_per_day() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.rng = Some(ChaCha20Rng::seed_from_u64(42));
        let encounter = Encounter {
            id: "test".to_string(),
            name: "Test".to_string(),
            desc: "desc".to_string(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: "Do it".to_string(),
                effects: Effects::default(),
            }],
        };
        state.data = Some(EncounterData::from_encounters(vec![encounter]));
        let cfg = crate::pacing::PacingConfig::default_config();
        state.apply_pace_and_diet(&cfg);
        state.encounters_today = MAX_ENCOUNTERS_PER_DAY;
        if let Some(back) = state.encounter_history.back_mut() {
            *back = state.encounters_today;
        }
        state.encounter_cooldown = 0;
        state.encounter_chance_today = 1.0;
        state.encounter_occurred_today = false;
        state.current_encounter = None;

        let (ended, message, _) = state.travel_next_leg();
        assert!(!ended);
        assert_eq!(message, LOG_TRAVELED);
        assert!(state.current_encounter.is_none());
    }

    #[test]
    fn allows_two_encounters_before_cooldown() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.rng = Some(ChaCha20Rng::seed_from_u64(99));
        let encounter = Encounter {
            id: "test".to_string(),
            name: "Test".to_string(),
            desc: "desc".to_string(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: "Do it".to_string(),
                effects: Effects::default(),
            }],
        };
        state.data = Some(EncounterData::from_encounters(vec![encounter]));
        let cfg = crate::pacing::PacingConfig::default_config();

        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        let (_ended_first, msg_first, _) = state.travel_next_leg();
        assert_eq!(msg_first, "log.encounter");
        assert_eq!(state.encounters_today, 1);
        state.apply_choice(0);
        assert!(!state.encounter_occurred_today);

        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        let (_ended_second, msg_second, _) = state.travel_next_leg();
        assert_eq!(msg_second, "log.encounter");
        assert_eq!(state.encounters_today, 2);
        state.apply_choice(0);
        assert!(state.encounter_occurred_today);

        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        let (_ended_third, msg_third, _) = state.travel_next_leg();
        assert_eq!(msg_third, LOG_TRAVELED);
        assert_eq!(
            state.encounter_history.back(),
            Some(&MAX_ENCOUNTERS_PER_DAY)
        );
    }
}

/// Default diet setting
fn default_diet() -> DietId {
    DietId::Mixed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Deep,
}

impl GameMode {
    #[must_use]
    pub fn is_deep(self) -> bool {
        matches!(self, GameMode::Deep)
    }

    #[must_use]
    pub const fn boss_threshold(self) -> i32 {
        match self {
            GameMode::Classic => 1_000,
            GameMode::Deep => 1_200,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    Heartland,
    RustBelt,
    Beltway,
}

impl Region {
    #[must_use]
    pub fn asset_key(self) -> &'static str {
        match self {
            Region::Heartland => "Heartland",
            Region::RustBelt => "RustBelt",
            Region::Beltway => "Beltway",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    #[must_use]
    pub fn from_day(day: u32) -> Self {
        let season_len = 45;
        let idx = day.saturating_sub(1) / season_len;
        match idx % 4 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Fall,
            _ => Season::Winter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollapseCause {
    Hunger,
    Vehicle,
    Weather,
    Breakdown,
    Panic,
}

impl CollapseCause {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            CollapseCause::Hunger => "hunger",
            CollapseCause::Vehicle => "vehicle",
            CollapseCause::Weather => "weather",
            CollapseCause::Breakdown => "breakdown",
            CollapseCause::Panic => "panic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExposureKind {
    Cold,
    Heat,
}

impl ExposureKind {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ExposureKind::Cold => "cold",
            ExposureKind::Heat => "heat",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleFailureCause {
    Destroyed,
}

impl VehicleFailureCause {
    #[must_use]
    pub const fn key(self) -> &'static str {
        "vehicle_destroyed"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ending {
    Collapse { cause: CollapseCause },
    SanityLoss,
    VehicleFailure { cause: VehicleFailureCause },
    Exposure { kind: ExposureKind },
    BossVoteFailed,
    BossVictory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageCause {
    Starvation,
    ExposureCold,
    ExposureHeat,
    Vehicle,
    Breakdown,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub supplies: i32,
    pub hp: i32,
    pub sanity: i32,
    pub credibility: i32,
    pub morale: i32,
    pub allies: i32,
    pub pants: i32, // 0..100
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            supplies: 10,
            hp: 10,
            sanity: 10,
            credibility: 5,
            morale: 5,
            allies: 0,
            pants: 0,
        }
    }
}

impl Stats {
    pub fn clamp(&mut self) {
        self.hp = self.hp.clamp(0, 10);
        self.sanity = self.sanity.clamp(0, 10);
        self.credibility = self.credibility.clamp(0, 20);
        self.morale = self.morale.clamp(0, 10);
        self.supplies = self.supplies.clamp(0, 20);
        self.allies = self.allies.clamp(0, 50);
        self.pants = self.pants.clamp(0, 100);
    }
}

/// Player inventory including spares and tags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Inventory {
    #[serde(default)]
    pub spares: Spares,
    #[serde(default)]
    pub tags: HashSet<String>,
}

impl Inventory {
    #[must_use]
    pub fn total_spares(&self) -> i32 {
        self.spares.tire + self.spares.battery + self.spares.alt + self.spares.pump
    }

    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

/// Vehicle and equipment spares
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Spares {
    #[serde(default)]
    pub tire: i32,
    #[serde(default)]
    pub battery: i32,
    #[serde(default)]
    pub alt: i32, // alternator
    #[serde(default)]
    pub pump: i32, // fuel pump
}

/// Party configuration (leader plus four companions)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Party {
    #[serde(default)]
    pub leader: String,
    #[serde(default)]
    pub companions: Vec<String>,
}

fn default_rest_threshold() -> i32 {
    4
}

fn default_trail_distance() -> f32 {
    2_100.0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Boot,
    Persona,
    Menu,
    Travel,
    Encounter,
    Boss,
    Result,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub mode: GameMode,
    pub seed: u64,
    pub day: u32,
    pub region: Region,
    #[serde(default)]
    pub season: Season,
    pub stats: Stats,
    #[serde(default)]
    pub budget: i32,
    /// Budget in cents for precise calculations
    #[serde(default)]
    pub budget_cents: i64,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default)]
    pub persona_id: Option<String>,
    #[serde(default)]
    pub score_mult: f32,
    #[serde(default)]
    pub mods: PersonaMods,
    #[serde(default)]
    pub party: Party,
    #[serde(default)]
    pub auto_camp_rest: bool,
    #[serde(default = "default_rest_threshold")]
    pub rest_threshold: i32,
    #[serde(default)]
    pub rest_requested: bool,
    #[serde(default = "default_trail_distance")]
    pub trail_distance: f32,
    #[serde(default)]
    pub distance_traveled: f32,
    #[serde(default)]
    pub distance_traveled_actual: f32,
    #[serde(default)]
    pub vehicle_breakdowns: i32,
    #[serde(default)]
    pub starvation_days: u32,
    #[serde(default)]
    pub malnutrition_level: u32,
    #[serde(default)]
    pub boss_ready: bool,
    #[serde(default)]
    pub boss_attempted: bool,
    #[serde(default)]
    pub boss_victory: bool,
    #[serde(default)]
    pub ending: Option<Ending>,
    /// Current pace setting
    #[serde(default = "default_pace")]
    pub pace: PaceId,
    /// Current info diet setting
    #[serde(default = "default_diet")]
    pub diet: DietId,
    /// Calculated receipt finding bonus percentage for this tick
    #[serde(default)]
    pub receipt_bonus_pct: i32,
    /// Base encounter chance for today after pace modifiers
    #[serde(default)]
    pub encounter_chance_today: f32,
    /// Whether an encounter has already occurred on the current day
    #[serde(default)]
    pub encounter_occurred_today: bool,
    /// Distance multiplier for today
    #[serde(default)]
    pub distance_today: f32,
    pub logs: Vec<String>,
    pub receipts: Vec<String>,
    #[serde(default)]
    pub encounters_resolved: u32,
    #[serde(default)]
    pub prev_distance_traveled: f32,
    #[serde(default)]
    pub travel_days: u32,
    #[serde(default)]
    pub non_travel_days: u32,
    #[serde(default)]
    pub traveled_today: bool,
    #[serde(default)]
    pub day_initialized: bool,
    #[serde(default)]
    pub did_end_of_day: bool,
    #[serde(default)]
    pub encounters_today: u8,
    #[serde(default)]
    pub encounter_history: VecDeque<u8>,
    #[serde(default)]
    pub encounter_cooldown: u8,
    #[serde(default)]
    pub repairs_spent_cents: i64,
    #[serde(default)]
    pub bribes_spent_cents: i64,
    #[serde(default)]
    pub current_order: Option<ExecOrder>,
    #[serde(default)]
    pub exec_order_days_remaining: u8,
    #[serde(default)]
    pub exec_order_cooldown: u8,
    #[serde(default)]
    pub exec_travel_multiplier: f32,
    #[serde(default)]
    pub exec_breakdown_bonus: f32,
    #[serde(default)]
    pub current_encounter: Option<Encounter>,
    /// Vehicle state and spares
    #[serde(default)]
    pub vehicle: Vehicle,
    /// Active breakdown blocking travel
    #[serde(default)]
    pub breakdown: Option<Breakdown>,
    /// Whether travel is blocked due to breakdown
    #[serde(default)]
    pub travel_blocked: bool,
    /// Weather state and history for streak tracking
    #[serde(default)]
    pub weather_state: WeatherState,
    /// Camp state and cooldowns
    #[serde(default)]
    pub camp: CampState,
    #[serde(skip)]
    pub rng: Option<ChaCha20Rng>,
    #[serde(skip)]
    pub data: Option<EncounterData>,
    #[serde(skip)]
    pub last_damage: Option<DamageCause>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            mode: GameMode::Classic,
            seed: 0,
            day: 1,
            region: Region::Heartland,
            season: Season::default(),
            stats: Stats::default(),
            budget: 100,
            budget_cents: 10_000, // $100.00 in cents
            inventory: Inventory::default(),
            persona_id: None,
            score_mult: 1.0,
            mods: PersonaMods::default(),
            party: Party::default(),
            auto_camp_rest: false,
            rest_threshold: default_rest_threshold(),
            rest_requested: false,
            trail_distance: default_trail_distance(),
            distance_traveled: 0.0,
            distance_traveled_actual: 0.0,
            vehicle_breakdowns: 0,
            starvation_days: 0,
            malnutrition_level: 0,
            boss_ready: false,
            boss_attempted: false,
            boss_victory: false,
            ending: None,
            pace: default_pace(),
            diet: default_diet(),
            receipt_bonus_pct: 0,
            encounter_chance_today: ENCOUNTER_BASE_DEFAULT,
            encounter_occurred_today: false,
            distance_today: 1.0,
            logs: vec![String::from("log.booting")],
            receipts: vec![],
            encounters_resolved: 0,
            prev_distance_traveled: 0.0,
            travel_days: 0,
            non_travel_days: 0,
            traveled_today: false,
            day_initialized: false,
            did_end_of_day: false,
            encounters_today: 0,
            encounter_history: VecDeque::with_capacity(ENCOUNTER_HISTORY_WINDOW + 2),
            encounter_cooldown: 0,
            repairs_spent_cents: 0,
            bribes_spent_cents: 0,
            current_encounter: None,
            current_order: None,
            exec_order_days_remaining: 0,
            exec_order_cooldown: 0,
            exec_travel_multiplier: 1.0,
            exec_breakdown_bonus: 0.0,
            vehicle: Vehicle::default(),
            breakdown: None,
            travel_blocked: false,
            weather_state: WeatherState::default(),
            camp: CampState::default(),
            rng: None,
            data: None,
            last_damage: None,
        }
    }
}

impl GameState {
    pub(crate) fn start_of_day(&mut self) {
        if self.day_initialized {
            return;
        }
        self.day_initialized = true;
        self.did_end_of_day = false;
        self.traveled_today = false;
        self.encounters_today = 0;
        self.encounter_occurred_today = false;
        self.prev_distance_traveled = self.distance_traveled_actual;
        self.exec_travel_multiplier = 1.0;
        self.exec_breakdown_bonus = 0.0;

        if self.encounter_history.len() >= ENCOUNTER_HISTORY_WINDOW {
            self.encounter_history.pop_front();
        }
        self.encounter_history.push_back(0);

        if self.encounter_cooldown > 0 {
            self.encounter_cooldown -= 1;
        }

        self.tick_exec_order_state();

        self.apply_starvation_tick();
        let weather_cfg = WeatherConfig::default_config();
        crate::weather::process_daily_weather(self, &weather_cfg);
        self.stats.clamp();

        self.vehicle.wear = (self.vehicle.wear + VEHICLE_DAILY_WEAR).min(VEHICLE_HEALTH_MAX);
        self.vehicle.apply_damage(VEHICLE_DAILY_WEAR);
    }

    fn tick_exec_order_state(&mut self) {
        if let Some(order) = self.current_order {
            self.apply_exec_order_effects(order);
            if self.exec_order_days_remaining > 0 {
                self.exec_order_days_remaining -= 1;
            }
            if self.exec_order_days_remaining == 0 {
                self.logs
                    .push(format!("{}{}", LOG_EXEC_END_PREFIX, order.key()));
                self.current_order = None;
                if let Some(rng) = self.rng.as_mut() {
                    self.exec_order_cooldown =
                        rng.random_range(EXEC_ORDER_MIN_COOLDOWN..=EXEC_ORDER_MAX_COOLDOWN);
                } else {
                    self.exec_order_cooldown = EXEC_ORDER_MIN_COOLDOWN;
                }
            }
            return;
        }

        if self.exec_order_cooldown > 0 {
            self.exec_order_cooldown -= 1;
            return;
        }

        if let Some(rng) = self.rng.as_mut()
            && rng.random::<f32>() < EXEC_ORDER_DAILY_CHANCE
        {
            let idx = rng.random_range(0..ExecOrder::ALL.len());
            let order = ExecOrder::ALL[idx];
            self.current_order = Some(order);
            self.exec_order_days_remaining =
                rng.random_range(EXEC_ORDER_MIN_DURATION..=EXEC_ORDER_MAX_DURATION);
            self.logs
                .push(format!("{}{}", LOG_EXEC_START_PREFIX, order.key()));
            self.apply_exec_order_effects(order);
            if self.exec_order_days_remaining > 0 {
                self.exec_order_days_remaining -= 1;
            }
        }
    }

    fn apply_exec_order_effects(&mut self, order: ExecOrder) {
        match order {
            ExecOrder::Shutdown => {
                self.stats.morale -= 1;
                self.stats.supplies -= 1;
            }
            ExecOrder::TravelBanLite => {
                self.stats.sanity -= 1;
                self.exec_travel_multiplier *= 0.8;
            }
            ExecOrder::BookPanic => {
                let mut penalty = 2;
                if self.stats.morale >= 7 {
                    penalty = 1;
                }
                self.stats.sanity -= penalty;
            }
            ExecOrder::TariffTsunami => {
                let penalty = if self.inventory.has_tag("legal_fund") {
                    -1
                } else {
                    -2
                };
                self.stats.supplies += penalty;
            }
            ExecOrder::DoEEliminated => {
                self.stats.morale -= 1;
                self.stats.sanity -= 1;
            }
            ExecOrder::WarDeptReorg => {
                self.exec_breakdown_bonus += 0.15;
            }
        }
        self.stats.clamp();
    }

    pub(crate) fn end_of_day(&mut self) {
        if self.did_end_of_day {
            return;
        }
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
        if self.traveled_today {
            self.travel_days = self.travel_days.saturating_add(1);
        } else {
            self.non_travel_days = self.non_travel_days.saturating_add(1);
        }
        self.did_end_of_day = true;
        self.day = self.day.saturating_add(1);
        self.region = Self::region_by_day(self.day);
        self.season = Season::from_day(self.day);
        self.day_initialized = false;
        self.encounters_today = 0;
        self.encounter_occurred_today = false;
        self.traveled_today = false;
    }

    fn record_encounter(&mut self) {
        self.encounters_today = self.encounters_today.saturating_add(1);
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
        self.encounter_cooldown = ENCOUNTER_COOLDOWN_DAYS.saturating_add(1);
    }

    fn seed_bytes(s: u64) -> [u8; 32] {
        #[inline]
        fn b(x: u64, shift: u8, xorv: u8) -> u8 {
            (((x >> shift) & 0xFF) as u8) ^ xorv
        }
        [
            b(s, 56, 0x00),
            b(s, 48, 0x00),
            b(s, 40, 0x00),
            b(s, 32, 0x00),
            b(s, 24, 0x00),
            b(s, 16, 0x00),
            b(s, 8, 0x00),
            b(s, 0, 0x00),
            b(s, 56, 0xAA),
            b(s, 48, 0x55),
            b(s, 40, 0xAA),
            b(s, 32, 0x55),
            b(s, 24, 0xAA),
            b(s, 16, 0x55),
            b(s, 8, 0xAA),
            b(s, 0, 0x55),
            b(s, 56, 0x11),
            b(s, 48, 0x22),
            b(s, 40, 0x33),
            b(s, 32, 0x44),
            b(s, 24, 0x55),
            b(s, 16, 0x66),
            b(s, 8, 0x77),
            b(s, 0, 0x88),
            b(s, 56, 0x99),
            b(s, 48, 0xAA),
            b(s, 40, 0xBB),
            b(s, 32, 0xCC),
            b(s, 24, 0xDD),
            b(s, 16, 0xEE),
            b(s, 8, 0xFF),
            b(s, 0, 0x10),
        ]
    }

    fn set_ending(&mut self, ending: Ending) {
        if self.ending.is_none() {
            self.ending = Some(ending);
        }
    }

    pub(crate) fn mark_damage(&mut self, cause: DamageCause) {
        self.last_damage = Some(cause);
    }

    #[must_use]
    pub fn vehicle_health(&self) -> f32 {
        self.vehicle.health
    }

    #[must_use]
    pub fn journey_score(&self) -> i32 {
        let stats = &self.stats;
        let supplies = stats.supplies.max(0);
        let hp = stats.hp.max(0);
        let morale = stats.morale.max(0);
        let credibility = stats.credibility.max(0);
        let allies = stats.allies.max(0);
        let days = i32::try_from(self.day.saturating_sub(1)).unwrap_or(0);
        let encounters = i32::try_from(self.encounters_resolved).unwrap_or(0);
        let receipts = i32::try_from(self.receipts.len()).unwrap_or(0);
        let breakdown_penalty = (self.vehicle_breakdowns * 12).min(600);

        supplies * 10
            + hp * 50
            + morale * 25
            + credibility * 15
            + allies * 5
            + days * 4
            + encounters * 6
            + receipts * 8
            - breakdown_penalty
    }

    #[must_use]
    fn total_spares(&self) -> i32 {
        self.inventory.total_spares()
    }

    fn apply_starvation_tick(&mut self) {
        if self.stats.supplies > 0 {
            if self.starvation_days > 0 {
                self.logs.push(String::from(LOG_STARVATION_RELIEF));
            }
            self.starvation_days = 0;
            self.malnutrition_level = 0;
            return;
        }

        self.starvation_days = self.starvation_days.saturating_add(1);
        self.malnutrition_level = (self.malnutrition_level + 1).min(STARVATION_MAX_STACK);

        let malnutrition_penalty = i32::try_from(self.malnutrition_level).unwrap_or(0);
        let hp_loss = STARVATION_BASE_HP_LOSS + malnutrition_penalty.min(3);

        self.stats.hp -= hp_loss;
        self.stats.sanity -= STARVATION_SANITY_LOSS + malnutrition_penalty.min(2);
        self.stats.pants = (self.stats.pants + STARVATION_PANTS_GAIN).clamp(0, 100);
        self.mark_damage(DamageCause::Starvation);
        self.logs.push(String::from(LOG_STARVATION_TICK));
        if self.stats.hp <= 0 {
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Hunger,
            });
        }
    }

    fn tick_ally_attrition(&mut self) {
        if self.stats.allies <= 0 {
            return;
        }
        let Some(rng) = self.rng.as_mut() else {
            return;
        };
        if rng.random::<f32>() <= ALLY_ATTRITION_CHANCE {
            self.stats.allies -= 1;
            self.stats.morale -= 1;
            self.logs.push(String::from(LOG_ALLY_LOST));
            if self.stats.allies == 0 {
                self.stats.sanity -= 2;
                self.logs.push(String::from(LOG_ALLIES_GONE));
            }
        }
    }

    #[must_use]
    fn current_weather_speed_penalty(&self) -> f32 {
        match self.weather_state.today {
            Weather::Storm => 0.75,
            Weather::ColdSnap => 0.80,
            Weather::Smoke | Weather::HeatWave => 0.90,
            Weather::Clear => 1.0,
        }
    }

    #[must_use]
    fn compute_miles_for_today(
        &self,
        pace_cfg: &crate::pacing::PaceCfg,
        limits: &crate::pacing::PacingLimits,
    ) -> f32 {
        let base_distance = if pace_cfg.distance > 0.0 {
            pace_cfg.distance
        } else if limits.distance_base > 0.0 {
            limits.distance_base
        } else {
            12.0
        };
        let config_scalar = pace_cfg.dist_mult.max(0.1);
        let pace_scalar = match self.pace {
            PaceId::Steady => 1.0,
            PaceId::Heated => 1.15,
            PaceId::Blitz => 1.30,
        } * config_scalar;
        let weather_scalar = self.current_weather_speed_penalty();
        let mut distance = base_distance * (weather_scalar * pace_scalar).max(0.6);

        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            distance *= 0.5;
        }

        if self.malnutrition_level > 0 {
            #[allow(clippy::cast_precision_loss)]
            let malnutrition = self.malnutrition_level as f32;
            let starvation_penalty = 1.0 - (malnutrition * 0.05);
            distance *= starvation_penalty.max(0.3);
        }

        (distance * self.exec_travel_multiplier).max(1.0)
    }

    fn check_vehicle_terminal_state(&mut self) -> bool {
        let spare_guard = self.total_spares();
        let tolerance = VEHICLE_BREAKDOWN_TOLERANCE_FLOOR.max(spare_guard * 3);
        let health_depleted = self.vehicle.health <= 0.0;
        let out_of_options = spare_guard == 0 && self.budget_cents < EMERGENCY_REPAIR_COST;
        if health_depleted && self.vehicle_breakdowns >= tolerance && out_of_options {
            self.vehicle.health = 0.0;
            self.mark_damage(DamageCause::Vehicle);
            self.set_ending(Ending::VehicleFailure {
                cause: VehicleFailureCause::Destroyed,
            });
            self.logs.push(String::from(LOG_VEHICLE_FAILURE));
            return true;
        }
        false
    }

    #[must_use]
    pub fn with_seed(mut self, seed: u64, mode: GameMode, data: EncounterData) -> Self {
        let bytes = Self::seed_bytes(seed);
        self.mode = mode;
        self.seed = seed;
        self.rng = Some(ChaCha20Rng::from_seed(bytes));
        self.logs.push(String::from("log.seed-set"));
        self.data = Some(data);
        self
    }

    #[must_use]
    pub fn rehydrate(mut self, data: EncounterData) -> Self {
        let bytes = Self::seed_bytes(self.seed);
        self.rng = Some(ChaCha20Rng::from_seed(bytes));
        self.data = Some(data);
        self
    }

    #[must_use]
    pub fn region_by_day(day: u32) -> Region {
        match day {
            0..=4 => Region::Heartland,
            5..=9 => Region::RustBelt,
            _ => Region::Beltway,
        }
    }

    pub fn travel_next_leg(&mut self) -> (bool, String, bool) {
        self.start_of_day();

        if self.boss_ready && !self.boss_attempted {
            return (false, String::from("log.boss.await"), false);
        }

        self.tick_ally_attrition();
        self.stats.clamp();
        if let Some(log_key) = self.failure_log_key() {
            self.end_of_day();
            return (true, String::from(log_key), false);
        }

        let breakdown_started = self.vehicle_roll();
        self.resolve_breakdown();
        if self.check_vehicle_terminal_state() {
            self.end_of_day();
            return (true, String::from(LOG_VEHICLE_FAILURE), breakdown_started);
        }

        if self.travel_blocked {
            self.end_of_day();
            return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
        }

        let mut trigger_encounter = false;
        if !self.encounter_occurred_today
            && let Some(rng) = self.rng.as_mut()
        {
            let roll: f32 = rng.random();
            if roll < self.encounter_chance_today {
                trigger_encounter = true;
            }
        }

        if self.encounters_today >= MAX_ENCOUNTERS_PER_DAY {
            trigger_encounter = false;
        }

        if trigger_encounter
            && let (Some(rng), Some(data)) = (self.rng.as_mut(), self.data.as_ref())
        {
            let encounter = pick_encounter(
                self.region,
                self.mode.is_deep(),
                self.malnutrition_level,
                self.stats.supplies <= 0,
                data,
                rng,
            );
            if let Some(enc) = encounter {
                self.current_encounter = Some(enc);
                self.encounter_occurred_today = true;
                self.record_encounter();
                return (false, String::from("log.encounter"), breakdown_started);
            }
        }

        let travel_distance = self.distance_today;
        let before = self.distance_traveled_actual;
        self.distance_traveled_actual += travel_distance;
        self.distance_traveled =
            (self.distance_traveled + travel_distance).min(self.trail_distance);
        self.traveled_today = self.distance_traveled_actual > before;
        if self.distance_traveled_actual >= self.trail_distance {
            self.boss_ready = true;
        }

        if debug_log_enabled() {
            println!(
                "Day {}: distance {:.1}/{:.1} (actual {:.1}), boss_ready {}, HP {}, Sanity {}",
                self.day,
                self.distance_traveled,
                self.trail_distance,
                self.distance_traveled_actual,
                self.boss_ready,
                self.stats.hp,
                self.stats.sanity
            );
        }

        if let Some(log_key) = self.failure_log_key() {
            self.end_of_day();
            return (true, String::from(log_key), breakdown_started);
        }

        self.end_of_day();
        (false, String::from(LOG_TRAVELED), breakdown_started)
    }

    /// Apply vehicle breakdown logic
    fn vehicle_roll(&mut self) -> bool {
        let mut breakdown_started = false;
        if let Some(rng) = self.rng.as_mut()
            && self.breakdown.is_none()
        {
            let mut breakdown_chance = 0.08 + self.exec_breakdown_bonus;
            breakdown_chance += (self.vehicle.wear / VEHICLE_HEALTH_MAX) * 0.2;
            breakdown_chance += match self.pace {
                PaceId::Steady => 0.0,
                PaceId::Heated => 0.03,
                PaceId::Blitz => 0.06,
            };
            if self.weather_state.today.is_extreme() {
                breakdown_chance += 0.04;
            }
            if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
                breakdown_chance += 0.05;
            }
            let roll: f32 = rng.random();
            if roll < breakdown_chance {
                use crate::vehicle::Part;
                let parts = [Part::Tire, Part::Battery, Part::Alternator, Part::FuelPump];
                let part_idx: usize = rng.random_range(0..parts.len());
                self.breakdown = Some(crate::vehicle::Breakdown {
                    part: parts[part_idx],
                    day_started: i32::try_from(self.day).unwrap_or(0),
                });
                self.travel_blocked = true;
                self.vehicle_breakdowns += 1;
                self.vehicle.apply_damage(VEHICLE_BREAKDOWN_DAMAGE);
                self.vehicle.wear =
                    (self.vehicle.wear + VEHICLE_BREAKDOWN_WEAR).min(VEHICLE_HEALTH_MAX);
                self.mark_damage(DamageCause::Vehicle);
                breakdown_started = true;
                if debug_log_enabled() {
                    println!(
                        "🚗 Breakdown started: {:?} | health {} | roll {:.3} chance {:.3}",
                        parts[part_idx], self.vehicle.health, roll, breakdown_chance
                    );
                }
            }
        }
        breakdown_started
    }

    pub fn apply_choice(&mut self, idx: usize) {
        if let Some(enc) = self.current_encounter.clone()
            && let Some(choice) = enc.choices.get(idx)
        {
            #[cfg(debug_assertions)]
            let (hp_before, sanity_before) = (self.stats.hp, self.stats.sanity);

            let eff = &choice.effects;
            self.stats.hp += eff.hp;
            self.stats.sanity += eff.sanity;
            self.stats.credibility += eff.credibility;
            self.stats.supplies += eff.supplies;
            self.stats.morale += eff.morale;
            self.stats.allies += eff.allies;
            self.stats.pants += eff.pants;
            if eff.hp < 0 {
                self.mark_damage(DamageCause::Breakdown);
            }
            if let Some(r) = &eff.add_receipt {
                self.receipts.push(r.clone());
            }
            if eff.use_receipt {
                let _ = self.receipts.pop();
            }
            if let Some(log) = &eff.log {
                self.logs.push(log.clone());
            }

            #[cfg(debug_assertions)]
            if debug_log_enabled() && (eff.hp != 0 || eff.sanity != 0) {
                println!(
                    "Encounter '{}' applied HP {} -> {}, Sanity {} -> {}",
                    enc.name, hp_before, self.stats.hp, sanity_before, self.stats.sanity
                );
            }

            self.stats.clamp();
        }
        self.current_encounter = None;
        self.encounters_resolved = self.encounters_resolved.saturating_add(1);
        if self.encounters_today < MAX_ENCOUNTERS_PER_DAY {
            self.encounter_occurred_today = false;
        }
    }

    fn resolve_breakdown(&mut self) {
        if let Some(breakdown) = self.breakdown.clone() {
            if self.consume_spare_for_part(breakdown.part) {
                self.vehicle.repair(VEHICLE_JURY_RIG_HEAL);
                self.breakdown = None;
                self.travel_blocked = false;
                self.repairs_spent_cents += 0;
                self.logs.push(String::from("log.breakdown-repaired"));
                return;
            }

            if self.total_spares() == 0 && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.budget_cents -= EMERGENCY_REPAIR_COST;
                self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
                self.repairs_spent_cents += EMERGENCY_REPAIR_COST;
                self.vehicle
                    .repair(VEHICLE_EMERGENCY_HEAL + VEHICLE_JURY_RIG_HEAL);
                self.breakdown = None;
                self.travel_blocked = false;
                self.logs.push(String::from(LOG_VEHICLE_REPAIR_EMERGENCY));
                return;
            }

            let day_started = u32::try_from(breakdown.day_started).unwrap_or(0);
            if self.day.saturating_sub(day_started) >= 1 {
                self.vehicle.apply_damage(VEHICLE_BREAKDOWN_DAMAGE / 2.0);
                self.mark_damage(DamageCause::Vehicle);
                self.breakdown = None;
                self.travel_blocked = false;
                self.logs.push(String::from("log.breakdown-jury-rigged"));
            } else {
                self.travel_blocked = true;
            }
        } else {
            self.travel_blocked = false;
        }
    }

    fn consume_spare_for_part(&mut self, part: Part) -> bool {
        let spares = &mut self.inventory.spares;
        match part {
            Part::Tire if spares.tire > 0 => {
                spares.tire -= 1;
                true
            }
            Part::Battery if spares.battery > 0 => {
                spares.battery -= 1;
                true
            }
            Part::Alternator if spares.alt > 0 => {
                spares.alt -= 1;
                true
            }
            Part::FuelPump if spares.pump > 0 => {
                spares.pump -= 1;
                true
            }
            _ => false,
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        if let Some(rng) = self.rng.as_mut() {
            rng.random()
        } else {
            0
        }
    }

    pub fn next_pct(&mut self) -> u8 {
        (self.next_u32() % 100) as u8
    }

    /// Clamp all stats to valid ranges
    pub fn clamp_stats(&mut self) {
        self.stats.clamp();
    }

    /// Apply pace and diet configuration (placeholder)
    pub fn apply_pace_and_diet(&mut self, cfg: &crate::pacing::PacingConfig) {
        self.start_of_day();
        let pace_cfg = cfg.get_pace_safe(self.pace.as_str());
        let diet_cfg = cfg.get_diet_safe(self.diet.as_str());
        let limits = &cfg.limits;

        let encounter_base = if limits.encounter_base == 0.0 {
            ENCOUNTER_BASE_DEFAULT
        } else {
            limits.encounter_base
        };
        let encounter_floor = limits.encounter_floor;
        let encounter_ceiling = if limits.encounter_ceiling == 0.0 {
            1.0
        } else {
            limits.encounter_ceiling
        };
        let mut encounter = encounter_base + pace_cfg.encounter_chance_delta;

        let distance = self.compute_miles_for_today(&pace_cfg, limits);

        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            encounter = (encounter + 0.12).clamp(encounter_floor, encounter_ceiling);
        }

        let encounters_last_window: u32 =
            self.encounter_history.iter().copied().map(u32::from).sum();
        if encounters_last_window >= ENCOUNTER_SOFT_CAP_THRESHOLD {
            encounter *= 0.5;
        }

        if self.encounters_today >= MAX_ENCOUNTERS_PER_DAY
            || (self.encounter_cooldown > 0 && self.encounters_today == 0)
        {
            encounter = 0.0;
        }

        self.encounter_chance_today = encounter.clamp(encounter_floor, encounter_ceiling).max(0.0);
        self.distance_today = distance.max(1.0);

        let pants_floor = limits.pants_floor;
        let pants_ceiling = limits.pants_ceiling;
        let mut pants_value = self.stats.pants;

        if limits.passive_relief != 0 && pants_value >= limits.passive_relief_threshold {
            pants_value = (pants_value + limits.passive_relief).clamp(pants_floor, pants_ceiling);
        }

        if self.mods.pants_relief != 0 && pants_value >= self.mods.pants_relief_threshold {
            pants_value = (pants_value + self.mods.pants_relief).clamp(pants_floor, pants_ceiling);
        }

        let boss_stage = self.boss_ready || self.distance_traveled >= self.trail_distance;
        if boss_stage && limits.boss_passive_relief != 0 {
            pants_value =
                (pants_value + limits.boss_passive_relief).clamp(pants_floor, pants_ceiling);
        }

        let mut pants_delta = pace_cfg.pants + diet_cfg.pants;
        if boss_stage && limits.boss_pants_cap > 0 && pants_delta > limits.boss_pants_cap {
            pants_delta = limits.boss_pants_cap;
        }

        pants_value = (pants_value + pants_delta).clamp(pants_floor, pants_ceiling);
        self.stats.pants = pants_value;

        self.receipt_bonus_pct += diet_cfg.receipt_find_pct_delta;
        self.receipt_bonus_pct = self.receipt_bonus_pct.clamp(-100, 100);
    }

    /// Save game state (placeholder - platform specific)
    pub fn save(&self) {
        // Placeholder - web implementation will handle this
    }

    /// Load game state (placeholder - platform specific)
    #[must_use]
    pub fn load() -> Option<Self> {
        // Placeholder - web implementation will handle this
        None
    }

    /// Apply persona effects (placeholder)
    pub fn apply_persona(&mut self, persona: &Persona) {
        self.persona_id = Some(persona.id.clone());
        self.score_mult = persona.score_mult;
        self.mods = persona.mods.clone();

        if persona.start.supplies > 0 {
            self.stats.supplies = persona.start.supplies;
        }
        if persona.start.credibility > 0 {
            self.stats.credibility = persona.start.credibility;
        }
        if persona.start.sanity > 0 {
            self.stats.sanity = persona.start.sanity;
        }
        if persona.start.morale > 0 {
            self.stats.morale = persona.start.morale;
        }
        if persona.start.allies > 0 {
            self.stats.allies = persona.start.allies;
        }

        if persona.start.budget > 0 {
            self.budget = persona.start.budget;
            self.budget_cents = i64::from(persona.start.budget) * 100;
        }

        self.stats.clamp();
        self.logs
            .push(format!("log.persona.selected.{}", persona.id));
    }

    pub fn set_party<I, S>(&mut self, leader: S, companions: I)
    where
        I: IntoIterator,
        I::Item: Into<String>,
        S: Into<String>,
    {
        self.party.leader = leader.into();
        self.party.companions = companions.into_iter().map(Into::into).take(4).collect();
        while self.party.companions.len() < 4 {
            let idx = self.party.companions.len() + 2;
            self.party.companions.push(format!("Traveler {idx}"));
        }
        self.logs.push(String::from("log.party.updated"));
    }

    pub fn request_rest(&mut self) {
        self.rest_requested = true;
    }

    fn failure_log_key(&mut self) -> Option<&'static str> {
        if self.vehicle.health <= 0.0 {
            self.set_ending(Ending::VehicleFailure {
                cause: VehicleFailureCause::Destroyed,
            });
            return Some(LOG_VEHICLE_FAILURE);
        }
        if self.stats.pants >= 100 {
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Panic,
            });
            return Some(LOG_PANTS_EMERGENCY);
        }
        if self.stats.hp <= 0 {
            if self.ending.is_none() {
                match self.last_damage.unwrap_or(DamageCause::Unknown) {
                    DamageCause::ExposureCold | DamageCause::ExposureHeat => {
                        let kind = if matches!(self.last_damage, Some(DamageCause::ExposureCold)) {
                            ExposureKind::Cold
                        } else {
                            ExposureKind::Heat
                        };
                        self.set_ending(Ending::Exposure { kind });
                    }
                    DamageCause::Starvation => {
                        self.set_ending(Ending::Collapse {
                            cause: CollapseCause::Hunger,
                        });
                    }
                    DamageCause::Vehicle => {
                        self.set_ending(Ending::VehicleFailure {
                            cause: VehicleFailureCause::Destroyed,
                        });
                    }
                    DamageCause::Breakdown | DamageCause::Unknown => {
                        self.set_ending(Ending::Collapse {
                            cause: CollapseCause::Breakdown,
                        });
                    }
                }
            }
            return Some(LOG_HEALTH_COLLAPSE);
        }
        if self.stats.sanity <= 0 {
            self.set_ending(Ending::SanityLoss);
            return Some(LOG_SANITY_COLLAPSE);
        }
        None
    }

    pub fn consume_daily_effects(&mut self, sanity_delta: i32, supplies_delta: i32) {
        let pace_sup_cost = match self.pace {
            PaceId::Blitz => BLITZ_SUPPLY_COST,
            _ => DEFAULT_SUPPLY_COST,
        };
        if sanity_delta != 0 {
            let max_sanity = Stats::default().sanity;
            self.stats.sanity = (self.stats.sanity + sanity_delta).clamp(0, max_sanity);
        }
        let net_supplies = supplies_delta - pace_sup_cost;
        let old_supplies = self.stats.supplies;
        self.stats.supplies = (old_supplies + net_supplies).max(0);
        if debug_log_enabled() && net_supplies != 0 {
            println!(
                "Daily supplies effect: {} -> {} (delta {})",
                old_supplies, self.stats.supplies, net_supplies
            );
        }
        self.stats.clamp();
    }

    pub fn advance_days(&mut self, days: u32) {
        if days == 0 {
            return;
        }
        for _ in 0..days {
            self.start_of_day();
            self.end_of_day();
        }
    }

    pub fn tick_camp_cooldowns(&mut self) {
        if self.camp.rest_cooldown > 0 {
            self.camp.rest_cooldown -= 1;
        }
        if self.camp.forage_cooldown > 0 {
            self.camp.forage_cooldown -= 1;
        }
        if self.camp.repair_cooldown > 0 {
            self.camp.repair_cooldown -= 1;
        }
    }

    #[must_use]
    pub fn should_auto_rest(&self) -> bool {
        self.auto_camp_rest
            && self.stats.sanity <= self.rest_threshold
            && self.camp.rest_cooldown == 0
    }

    pub fn refresh_exec_order(&mut self) {
        self.start_of_day();
    }

    /// Apply store purchase effects
    pub fn apply_store_purchase(
        &mut self,
        cost_cents: i64,
        grants: &crate::store::Grants,
        tags: &[String],
    ) {
        let budget_before = self.budget_cents;

        // Subtract cost from budget
        self.budget_cents = (self.budget_cents - cost_cents).max(0);

        if debug_log_enabled() {
            println!(
                "Budget change: {} -> {} (cost {})",
                budget_before, self.budget_cents, cost_cents
            );
        }

        // Apply grants
        self.stats.supplies += grants.supplies;
        self.stats.credibility += grants.credibility;
        self.inventory.spares.tire += grants.spare_tire;
        self.inventory.spares.battery += grants.spare_battery;
        self.inventory.spares.alt += grants.spare_alt;
        self.inventory.spares.pump += grants.spare_pump;

        // Add tags
        for tag in tags {
            self.inventory.tags.insert(tag.clone());
        }

        // Clamp stats to valid ranges
        self.clamp_stats();
    }
}
