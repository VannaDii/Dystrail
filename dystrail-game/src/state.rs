use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use crate::camp::CampState;
use crate::crossings::{self, CrossingConfig, CrossingKind, CrossingTypeCfg, ThresholdEntry};
use crate::data::{Encounter, EncounterData};
use crate::encounters::{EncounterRequest, pick_encounter};
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
const VEHICLE_BREAKDOWN_DAMAGE: f32 = 6.0;
const VEHICLE_DAILY_WEAR: f32 = 0.2;
const VEHICLE_CRITICAL_THRESHOLD: f32 = 20.0;
const VEHICLE_HEALTH_MAX: f32 = 100.0;
const VEHICLE_BREAKDOWN_WEAR: f32 = 6.0;
const VEHICLE_BREAKDOWN_WEAR_CLASSIC: f32 = 5.0;
const VEHICLE_EMERGENCY_HEAL: f32 = 10.0;
const VEHICLE_EMERGENCY_DELAY_DAYS: u32 = 1;
const VEHICLE_JURY_RIG_HEAL: f32 = 4.0;
const STARVATION_BASE_HP_LOSS: i32 = 1;
const STARVATION_SANITY_LOSS: i32 = 1;
const STARVATION_PANTS_GAIN: i32 = 1;
const STARVATION_MAX_STACK: u32 = 5;
const STARVATION_GRACE_DAYS: u32 = 1;
const ALLY_ATTRITION_CHANCE: f32 = 0.02;
const EMERGENCY_REPAIR_COST: i64 = 1_000;
const ENCOUNTER_BASE_DEFAULT: f32 = 0.27;
const ENCOUNTER_COOLDOWN_DAYS: u8 = 1;
const ENCOUNTER_SOFT_CAP_THRESHOLD: u32 = 5;
const ENCOUNTER_HISTORY_WINDOW: usize = 10;
const MAX_ENCOUNTERS_PER_DAY: u8 = 2;
const ENCOUNTER_RECENT_MEMORY: usize = 8;
pub(crate) const ENCOUNTER_REPEAT_WINDOW_DAYS: u32 = 6;
pub(crate) const ENCOUNTER_EXTENDED_MEMORY_DAYS: u32 = ENCOUNTER_REPEAT_WINDOW_DAYS * 2;
const ENCOUNTER_REROLL_PENALTY: f32 = 0.8;
const EXEC_ORDER_DAILY_CHANCE: f32 = 0.06;
const EXEC_ORDER_MIN_DURATION: u8 = 2;
const EXEC_ORDER_MAX_DURATION: u8 = 4;
const EXEC_ORDER_MIN_COOLDOWN: u8 = 6;
const EXEC_ORDER_MAX_COOLDOWN: u8 = 9;
const LOG_EXEC_START_PREFIX: &str = "exec.start.";
const LOG_EXEC_END_PREFIX: &str = "exec.end.";
const LOG_STARVATION_TICK: &str = "log.starvation.tick";
const LOG_STARVATION_RELIEF: &str = "log.starvation.relief";
const LOG_ALLY_LOST: &str = "log.ally.lost";
const LOG_ALLIES_GONE: &str = "log.allies.gone";
const LOG_VEHICLE_FAILURE: &str = "log.vehicle.failure";
const LOG_VEHICLE_REPAIR_EMERGENCY: &str = "log.vehicle.repair.emergency";
const LOG_EMERGENCY_REPAIR_FORCED: &str = "log.vehicle.repair.forced";
const LOG_VEHICLE_REPAIR_SPARE: &str = "log.vehicle.repair.spare";
const LOG_CROSSING_DETOUR: &str = "log.crossing.detour";
const LOG_CROSSING_PASSED: &str = "log.crossing.passed";
const LOG_CROSSING_FAILURE: &str = "log.crossing.failure";
const LOG_CROSSING_DECISION_BRIBE: &str = "log.crossing.decision.bribe";
const LOG_CROSSING_DECISION_PERMIT: &str = "log.crossing.decision.permit";
const LOG_CROSSING_DECISION_DETOUR: &str = "log.crossing.decision.detour";
const LOG_TRAVEL_PARTIAL: &str = "log.travel.partial";
const LOG_TRAVEL_REST_CREDIT: &str = "log.travel.rest-credit";
const LOG_TRAVEL_DELAY_CREDIT: &str = "log.travel.delay-credit";
const LOG_TRAVEL_CROSSING_CREDIT: &str = "log.travel.crossing-credit";
const LOG_ENCOUNTER_ROTATION: &str = "log.encounter.rotation";
const LOG_TRAVEL_BONUS: &str = "log.travel.bonus";
const LOG_DISEASE_HIT: &str = "log.disease.hit";
const LOG_DISEASE_TICK: &str = "log.disease.tick";
const LOG_DISEASE_RECOVER: &str = "log.disease.recover";
const LOG_STARVATION_BACKSTOP: &str = "log.starvation.backstop";
const LOG_REST_REQUESTED_ENCOUNTER: &str = "log.encounter.rest-requested";

const CROSSING_MILESTONES: [f32; 3] = [650.0, 1_250.0, 1_900.0];
const CROSSING_FAILURE_BASE: f32 = 0.09;
const CROSSING_FAILURE_DEEP_BONUS: f32 = 0.03;
const ROTATION_FORCE_INTERVAL: u32 = 5;
pub(crate) const ROTATION_LOOKBACK_DAYS: u32 = 5;
const DETOUR_DAY_RANGE: (u32, u32) = (2, 4);
const DETOUR_SUPPLY_LOSS_RANGE: (i32, i32) = (2, 4);
const REST_TRAVEL_CREDIT_MILES: f32 = 12.0;
const DELAY_TRAVEL_CREDIT_MILES: f32 = 9.0;
const CROSSING_SUCCESS_CREDIT_MILES: f32 = 16.0;
const TRAVEL_HISTORY_WINDOW: usize = 10;

const DISEASE_DAILY_CHANCE: f32 = 0.012;
const DISEASE_COOLDOWN_DAYS: u32 = 5;
const DISEASE_SANITY_PENALTY: i32 = 1;
const DISEASE_HP_PENALTY: i32 = 1;
const DISEASE_SUPPLY_PENALTY: i32 = 1;
const ILLNESS_TRAVEL_PENALTY: f32 = 0.85;
const DISEASE_DURATION_RANGE: (u32, u32) = (2, 4);
const DISEASE_SUPPLIES_BONUS: f32 = 0.02;
const DISEASE_STARVATION_BONUS: f32 = 0.015;
const DISEASE_LOW_HP_BONUS: f32 = 0.01;
const DISEASE_MAX_DAILY_CHANCE: f32 = 0.18;
const DISEASE_TICK_HP_LOSS: i32 = 1;
const DISEASE_TICK_SANITY_LOSS: i32 = 1;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PolicyKind {
    #[default]
    Balanced,
    Conservative,
    Aggressive,
    ResourceManager,
    MonteCarlo,
}

impl PolicyKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Conservative => "conservative",
            Self::Aggressive => "aggressive",
            Self::ResourceManager => "resource_manager",
            Self::MonteCarlo => "monte_carlo",
        }
    }
}

impl fmt::Display for PolicyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PolicyKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "balanced" => Ok(Self::Balanced),
            "conservative" => Ok(Self::Conservative),
            "aggressive" => Ok(Self::Aggressive),
            "resource_manager" => Ok(Self::ResourceManager),
            "monte_carlo" => Ok(Self::MonteCarlo),
            _ => Err(()),
        }
    }
}

impl From<PolicyKind> for String {
    fn from(value: PolicyKind) -> Self {
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
        assert_eq!(state.stats.hp, 10, "first starvation day is a grace period");
        assert_eq!(state.malnutrition_level, 0);

        state.apply_starvation_tick();
        assert_eq!(state.stats.hp, 9);
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
        for _ in 0..=(STARVATION_GRACE_DAYS + 1) {
            state.apply_starvation_tick();
        }
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
        state.stats.hp = 0;
        state.last_damage = Some(DamageCause::ExposureCold);
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
        assert!(
            state.travel_days + state.partial_travel_days >= 30,
            "expected at least 30 days with travel credit"
        );
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
            hard_stop: false,
            major_repair: false,
            chainable: false,
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
            hard_stop: false,
            major_repair: false,
            chainable: false,
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
    Disease,
    Crossing,
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
            CollapseCause::Disease => "disease",
            CollapseCause::Crossing => "crossing",
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
    Disease,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub travel_v2: bool,
    pub encounter_diversity: bool,
    pub exposure_streaks: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            travel_v2: true,
            encounter_diversity: true,
            exposure_streaks: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentEncounter {
    pub id: String,
    pub day: u32,
    #[serde(default)]
    pub region: Option<Region>,
}

impl RecentEncounter {
    #[must_use]
    pub fn new(id: String, day: u32, region: Region) -> Self {
        Self {
            id,
            day,
            region: Some(region),
        }
    }
}

fn default_rest_threshold() -> i32 {
    4
}

fn default_trail_distance() -> f32 {
    crate::boss::ROUTE_LEN_MILES
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
    pub features: FeatureFlags,
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
    pub crossings_completed: u32,
    #[serde(default)]
    pub crossing_detours: u32,
    #[serde(default)]
    pub crossing_failures: u32,
    #[serde(default)]
    pub crossing_events: Vec<CrossingTelemetry>,
    #[serde(default)]
    pub pending_delay_days: u32,
    #[serde(default)]
    pub starvation_days: u32,
    #[serde(default)]
    pub malnutrition_level: u32,
    #[serde(default)]
    pub starvation_backstop_used: bool,
    #[serde(default)]
    pub exposure_streak_heat: u32,
    #[serde(default)]
    pub exposure_streak_cold: u32,
    #[serde(default)]
    pub exposure_damage_lockout: bool,
    #[serde(default)]
    pub disease_cooldown: u32,
    #[serde(default)]
    pub boss_ready: bool,
    #[serde(default)]
    pub boss_reached: bool,
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
    #[serde(default)]
    pub distance_today_raw: f32,
    #[serde(default)]
    pub partial_distance_today: f32,
    pub logs: Vec<String>,
    pub receipts: Vec<String>,
    #[serde(default)]
    pub encounters_resolved: u32,
    #[serde(default)]
    pub prev_distance_traveled: f32,
    #[serde(default)]
    pub travel_days: u32,
    #[serde(default)]
    pub partial_travel_days: u32,
    #[serde(default)]
    pub non_travel_days: u32,
    #[serde(default)]
    pub days_with_camp: u32,
    #[serde(default)]
    pub days_with_repair: u32,
    #[serde(default)]
    pub traveled_today: bool,
    #[serde(default)]
    pub partial_traveled_today: bool,
    #[serde(default)]
    pub day_initialized: bool,
    #[serde(default)]
    pub did_end_of_day: bool,
    #[serde(default)]
    pub encounters_today: u8,
    #[serde(default)]
    pub encounter_history: VecDeque<u8>,
    #[serde(default)]
    pub recent_encounters: VecDeque<RecentEncounter>,
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
    pub weather_travel_multiplier: f32,
    #[serde(default)]
    pub illness_travel_penalty: f32,
    #[serde(default)]
    pub illness_days_remaining: u32,
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
    #[serde(default)]
    pub rotation_travel_days: u32,
    #[serde(default)]
    pub force_rotation_pending: bool,
    #[serde(default)]
    pub delay_partial_days: u32,
    #[serde(default)]
    pub policy: Option<PolicyKind>,
    #[serde(default)]
    pub recent_travel_days: VecDeque<TravelDayKind>,
    #[serde(skip)]
    pub rotation_backlog: VecDeque<String>,
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
            features: FeatureFlags::default(),
            party: Party::default(),
            auto_camp_rest: false,
            rest_threshold: default_rest_threshold(),
            rest_requested: false,
            trail_distance: default_trail_distance(),
            distance_traveled: 0.0,
            distance_traveled_actual: 0.0,
            vehicle_breakdowns: 0,
            crossings_completed: 0,
            crossing_detours: 0,
            crossing_failures: 0,
            crossing_events: Vec::new(),
            pending_delay_days: 0,
            starvation_days: 0,
            malnutrition_level: 0,
            starvation_backstop_used: false,
            exposure_streak_heat: 0,
            exposure_streak_cold: 0,
            exposure_damage_lockout: false,
            disease_cooldown: 0,
            boss_ready: false,
            boss_reached: false,
            boss_attempted: false,
            boss_victory: false,
            ending: None,
            pace: default_pace(),
            diet: default_diet(),
            receipt_bonus_pct: 0,
            encounter_chance_today: ENCOUNTER_BASE_DEFAULT,
            encounter_occurred_today: false,
            distance_today: 0.0,
            distance_today_raw: 0.0,
            partial_distance_today: 0.0,
            logs: vec![String::from("log.booting")],
            receipts: vec![],
            encounters_resolved: 0,
            prev_distance_traveled: 0.0,
            travel_days: 0,
            partial_travel_days: 0,
            non_travel_days: 0,
            days_with_camp: 0,
            days_with_repair: 0,
            traveled_today: false,
            partial_traveled_today: false,
            day_initialized: false,
            did_end_of_day: false,
            encounters_today: 0,
            encounter_history: VecDeque::with_capacity(ENCOUNTER_HISTORY_WINDOW + 2),
            recent_encounters: VecDeque::with_capacity(ENCOUNTER_RECENT_MEMORY),
            encounter_cooldown: 0,
            repairs_spent_cents: 0,
            bribes_spent_cents: 0,
            current_encounter: None,
            current_order: None,
            exec_order_days_remaining: 0,
            exec_order_cooldown: 0,
            exec_travel_multiplier: 1.0,
            exec_breakdown_bonus: 0.0,
            weather_travel_multiplier: 1.0,
            illness_travel_penalty: 1.0,
            illness_days_remaining: 0,
            vehicle: Vehicle::default(),
            breakdown: None,
            travel_blocked: false,
            weather_state: WeatherState::default(),
            camp: CampState::default(),
            rotation_travel_days: 0,
            force_rotation_pending: false,
            delay_partial_days: 0,
            policy: None,
            recent_travel_days: VecDeque::with_capacity(TRAVEL_HISTORY_WINDOW),
            rotation_backlog: VecDeque::new(),
            rng: None,
            data: None,
            last_damage: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TravelProgressKind {
    Full,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TravelDayKind {
    Full,
    Partial,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrossingPressure {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossingOutcomeTelemetry {
    Passed,
    Detoured,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossingTelemetry {
    pub day: u32,
    pub region: Region,
    pub season: Season,
    pub kind: CrossingKind,
    pub permit_used: bool,
    pub bribe_attempted: bool,
    pub bribe_success: Option<bool>,
    pub bribe_cost_cents: i64,
    pub bribe_chance: Option<f32>,
    pub bribe_roll: Option<f32>,
    pub detour_taken: bool,
    pub detour_days: Option<u32>,
    pub detour_base_supplies_delta: Option<i32>,
    pub detour_extra_supplies_loss: Option<i32>,
    pub detour_pants_delta: Option<i32>,
    pub terminal_threshold: f32,
    pub terminal_roll: Option<f32>,
    pub outcome: CrossingOutcomeTelemetry,
}

impl CrossingTelemetry {
    fn new(day: u32, region: Region, season: Season, kind: CrossingKind) -> Self {
        Self {
            day,
            region,
            season,
            kind,
            permit_used: false,
            bribe_attempted: false,
            bribe_success: None,
            bribe_cost_cents: 0,
            bribe_chance: None,
            bribe_roll: None,
            detour_taken: false,
            detour_days: None,
            detour_base_supplies_delta: None,
            detour_extra_supplies_loss: None,
            detour_pants_delta: None,
            terminal_threshold: 0.0,
            terminal_roll: None,
            outcome: CrossingOutcomeTelemetry::Detoured,
        }
    }
}

#[derive(Debug, Default)]
struct CrossingBribeOutcome {
    attempted: bool,
    success: bool,
    cost_cents: i64,
    chance: Option<f32>,
    roll: Option<f32>,
}

#[derive(Debug)]
struct CrossingDetourResolution {
    log_key: Option<&'static str>,
    detour_days: u32,
    base_supplies_delta: i32,
    extra_supply_loss: i32,
    pants_delta: i32,
    terminal_roll: f32,
    failed: bool,
}

impl GameState {
    pub(crate) fn start_of_day(&mut self) {
        if self.day_initialized {
            return;
        }
        self.day_initialized = true;
        self.did_end_of_day = false;
        self.traveled_today = false;
        self.partial_traveled_today = false;
        self.encounters_today = 0;
        self.encounter_occurred_today = false;
        self.prev_distance_traveled = self.distance_traveled_actual;
        self.exec_travel_multiplier = 1.0;
        self.exec_breakdown_bonus = 0.0;
        self.weather_travel_multiplier = 1.0;
        self.distance_today = 0.0;
        self.distance_today_raw = 0.0;
        self.partial_distance_today = 0.0;
        if self.illness_days_remaining == 0 {
            self.illness_travel_penalty = 1.0;
        }

        if self.encounter_history.len() >= ENCOUNTER_HISTORY_WINDOW {
            self.encounter_history.pop_front();
        }
        self.encounter_history.push_back(0);

        if self.encounter_cooldown > 0 {
            self.encounter_cooldown -= 1;
        }

        self.tick_exec_order_state();

        self.apply_starvation_tick();
        self.roll_daily_illness();
        let weather_cfg = WeatherConfig::default_config();
        crate::weather::process_daily_weather(self, &weather_cfg);
        self.stats.clamp();

        if !self.features.travel_v2 {
            self.vehicle.wear = (self.vehicle.wear + VEHICLE_DAILY_WEAR).min(VEHICLE_HEALTH_MAX);
            self.vehicle.apply_damage(VEHICLE_DAILY_WEAR);
        }
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

        if let Some(rng) = self.rng.as_mut() {
            let should_issue_order = rng.random::<f32>() < EXEC_ORDER_DAILY_CHANCE;
            if should_issue_order {
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
    }

    fn apply_exec_order_effects(&mut self, order: ExecOrder) {
        match order {
            ExecOrder::Shutdown => {
                self.stats.morale -= 1;
                self.stats.supplies = (self.stats.supplies - 1).max(0);
            }
            ExecOrder::TravelBanLite => {
                self.stats.sanity -= 1;
                self.exec_travel_multiplier *= 0.88;
            }
            ExecOrder::BookPanic => {
                if self.stats.morale < 7 {
                    self.stats.sanity -= 1;
                }
            }
            ExecOrder::TariffTsunami => {
                if !self.inventory.has_tag("legal_fund") {
                    self.stats.supplies = (self.stats.supplies - 1).max(0);
                }
            }
            ExecOrder::DoEEliminated => {
                self.stats.morale -= 1;
            }
            ExecOrder::WarDeptReorg => {
                self.exec_breakdown_bonus += 0.10;
            }
        }
        self.cap_exec_order_effects();
        self.stats.clamp();
    }

    fn cap_exec_order_effects(&mut self) {
        self.exec_travel_multiplier = self.exec_travel_multiplier.clamp(0.72, 1.0);
        self.exec_breakdown_bonus = self.exec_breakdown_bonus.clamp(0.0, 0.2);
    }

    pub(crate) fn end_of_day(&mut self) {
        if self.did_end_of_day {
            return;
        }
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
        self.enforce_aggressive_delay_cap();
        if !self.traveled_today && !self.partial_traveled_today {
            let delta = (self.distance_traveled_actual - self.prev_distance_traveled).abs();
            assert!(
                delta <= 0.01,
                "distance advanced on non-travel day (delta {delta:.2})"
            );
        }
        if self.partial_traveled_today {
            debug_assert!(
                (self.distance_traveled_actual - self.prev_distance_traveled) > 0.0,
                "partial travel day without distance gain"
            );
        }
        if self.traveled_today {
            self.travel_days = self.travel_days.saturating_add(1);
            self.rotation_travel_days = self.rotation_travel_days.saturating_add(1);
        } else if self.partial_traveled_today {
            self.partial_travel_days = self.partial_travel_days.saturating_add(1);
            self.rotation_travel_days = self.rotation_travel_days.saturating_add(1);
        } else {
            self.non_travel_days = self.non_travel_days.saturating_add(1);
        }
        if self.rotation_travel_days >= self.rotation_force_interval() {
            self.force_rotation_pending = true;
            self.rotation_travel_days = 0;
        }
        let day_kind = if self.traveled_today {
            TravelDayKind::Full
        } else if self.partial_traveled_today {
            TravelDayKind::Partial
        } else {
            TravelDayKind::None
        };
        if self.recent_travel_days.len() >= TRAVEL_HISTORY_WINDOW {
            self.recent_travel_days.pop_front();
        }
        self.recent_travel_days.push_back(day_kind);
        self.did_end_of_day = true;
        self.day = self.day.saturating_add(1);
        self.region = Self::region_by_day(self.day);
        self.season = Season::from_day(self.day);
        self.day_initialized = false;
        self.encounters_today = 0;
        self.encounter_occurred_today = false;
        self.traveled_today = false;
        self.partial_traveled_today = false;
    }

    fn record_encounter(&mut self, encounter_id: &str) {
        self.encounters_today = self.encounters_today.saturating_add(1);
        debug_assert!(
            self.encounters_today <= MAX_ENCOUNTERS_PER_DAY,
            "Encounter limit exceeded"
        );
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
        self.encounter_cooldown = ENCOUNTER_COOLDOWN_DAYS.saturating_add(1);
        let day = self.day;
        while self.recent_encounters.len() >= ENCOUNTER_RECENT_MEMORY {
            self.recent_encounters.pop_front();
        }
        self.recent_encounters
            .retain(|entry| day.saturating_sub(entry.day) <= ENCOUNTER_EXTENDED_MEMORY_DAYS);
        self.recent_encounters.push_back(RecentEncounter::new(
            encounter_id.to_string(),
            day,
            self.region,
        ));
    }

    fn finalize_encounter(&mut self) {
        self.current_encounter = None;
        self.encounters_resolved = self.encounters_resolved.saturating_add(1);
        if self.encounters_today < MAX_ENCOUNTERS_PER_DAY {
            self.encounter_occurred_today = false;
        }
    }

    fn apply_travel_wear_scaled(&mut self, scale: f32) {
        let wear = (VEHICLE_DAILY_WEAR * scale).max(0.0);
        self.vehicle.wear = (self.vehicle.wear + wear).min(VEHICLE_HEALTH_MAX);
        self.vehicle.apply_damage(wear);
    }

    fn apply_travel_wear(&mut self) {
        self.apply_travel_wear_scaled(1.0);
    }

    fn apply_travel_progress(&mut self, distance: f32, kind: TravelProgressKind) {
        if distance <= 0.0 {
            return;
        }
        let before = self.distance_traveled_actual;
        self.distance_traveled_actual += distance;
        self.distance_traveled = (self.distance_traveled + distance).min(self.trail_distance);
        let advanced = self.distance_traveled_actual > before;
        if advanced {
            match kind {
                TravelProgressKind::Full => self.traveled_today = true,
                TravelProgressKind::Partial => self.partial_traveled_today = true,
            }
            if self.ending.is_none() && self.distance_traveled_actual >= self.trail_distance {
                self.boss_ready = true;
                self.boss_reached = true;
            }
        }
    }

    fn reset_today_progress(&mut self) {
        let day_progress = (self.distance_traveled_actual - self.prev_distance_traveled).max(0.0);
        if day_progress > 0.0 {
            self.distance_traveled_actual -= day_progress;
            self.distance_traveled = self.distance_traveled_actual.min(self.trail_distance);
            if self.distance_traveled_actual < self.trail_distance {
                self.boss_ready = false;
                self.boss_reached = false;
            }
        }
        self.distance_today = 0.0;
        self.distance_today_raw = 0.0;
        self.partial_distance_today = 0.0;
        self.traveled_today = false;
        self.partial_traveled_today = false;
    }

    fn rotation_force_interval(&self) -> u32 {
        let mut interval = ROTATION_FORCE_INTERVAL;
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Conservative)) {
            interval = interval.saturating_sub(2).max(1);
        }
        interval
    }

    fn enforce_aggressive_delay_cap(&mut self) {
        if self.traveled_today || self.partial_traveled_today {
            return;
        }
        if !self.mode.is_deep() || !matches!(self.policy, Some(PolicyKind::Aggressive)) {
            return;
        }
        let full_stops = self
            .recent_travel_days
            .iter()
            .filter(|kind| matches!(kind, TravelDayKind::None))
            .count();
        if full_stops < 2 {
            return;
        }
        self.apply_delay_travel_credit();
    }

    #[must_use]
    pub fn travel_ratio_recent(&self, window: usize) -> f32 {
        if window == 0 {
            return 1.0;
        }
        let mut traveled = 0usize;
        let mut total = 0usize;
        for kind in self.recent_travel_days.iter().rev().take(window) {
            total += 1;
            if matches!(kind, TravelDayKind::Full | TravelDayKind::Partial) {
                traveled += 1;
            }
        }
        if total == 0 {
            return 1.0;
        }
        let traveled_u16 = u16::try_from(traveled).unwrap_or(u16::MAX);
        let total_u16 = u16::try_from(total).unwrap_or(u16::MAX);
        if total_u16 == 0 {
            1.0
        } else {
            f32::from(traveled_u16) / f32::from(total_u16)
        }
    }

    fn apply_partial_travel_credit(&mut self, distance: f32, log_key: &'static str) {
        if distance <= 0.0 {
            return;
        }
        if self.traveled_today && !self.partial_traveled_today {
            self.reset_today_progress();
        }
        self.distance_today += distance;
        self.distance_today_raw += distance;
        self.partial_distance_today = self.partial_distance_today.max(distance);
        self.apply_travel_progress(distance, TravelProgressKind::Partial);
        self.logs.push(String::from(log_key));
    }

    pub(crate) fn apply_rest_travel_credit(&mut self) {
        self.apply_partial_travel_credit(REST_TRAVEL_CREDIT_MILES, LOG_TRAVEL_REST_CREDIT);
    }

    fn apply_delay_travel_credit(&mut self) {
        self.apply_partial_travel_credit(DELAY_TRAVEL_CREDIT_MILES, LOG_TRAVEL_DELAY_CREDIT);
    }

    fn apply_crossing_success_credit(&mut self) {
        if CROSSING_SUCCESS_CREDIT_MILES <= 0.0 {
            return;
        }
        self.distance_today += CROSSING_SUCCESS_CREDIT_MILES;
        self.distance_today_raw += CROSSING_SUCCESS_CREDIT_MILES;
        self.distance_traveled_actual += CROSSING_SUCCESS_CREDIT_MILES;
        self.distance_traveled = self.distance_traveled_actual.min(self.trail_distance);
        if self.distance_traveled_actual >= self.trail_distance {
            self.boss_ready = true;
            self.boss_reached = true;
        }
        self.logs.push(String::from(LOG_TRAVEL_CROSSING_CREDIT));
    }

    fn should_discourage_encounter(&self, encounter_id: &str) -> bool {
        if !self.features.encounter_diversity {
            return false;
        }
        let current_day = self.day;
        self.recent_encounters
            .iter()
            .rev()
            .find(|entry| entry.id == encounter_id)
            .is_some_and(|entry| {
                current_day.saturating_sub(entry.day) < ENCOUNTER_REPEAT_WINDOW_DAYS
            })
    }

    #[must_use]
    fn encounter_reroll_penalty(&self) -> f32 {
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Conservative)) {
            0.9_f32.min(1.0)
        } else {
            ENCOUNTER_REROLL_PENALTY
        }
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
            self.starvation_backstop_used = false;
            return;
        }

        self.starvation_days = self.starvation_days.saturating_add(1);
        if self.starvation_days <= STARVATION_GRACE_DAYS {
            self.malnutrition_level = 0;
            return;
        }

        self.malnutrition_level = self.starvation_days.min(STARVATION_MAX_STACK);

        self.stats.hp -= STARVATION_BASE_HP_LOSS;
        self.stats.sanity -= STARVATION_SANITY_LOSS;
        self.stats.pants = (self.stats.pants + STARVATION_PANTS_GAIN).clamp(0, 100);
        self.mark_damage(DamageCause::Starvation);
        self.logs.push(String::from(LOG_STARVATION_TICK));
        if self.stats.hp <= 0 {
            if !self.starvation_backstop_used {
                self.starvation_backstop_used = true;
                self.stats.hp = 1;
                self.rest_requested = true;
                self.logs.push(String::from(LOG_STARVATION_BACKSTOP));
                return;
            }
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Hunger,
            });
        }
    }

    fn roll_daily_illness(&mut self) {
        if self.disease_cooldown > 0 {
            self.disease_cooldown -= 1;
        }

        if self.illness_days_remaining > 0 {
            self.illness_travel_penalty = ILLNESS_TRAVEL_PENALTY;
            self.stats.hp -= DISEASE_TICK_HP_LOSS;
            self.stats.sanity -= DISEASE_TICK_SANITY_LOSS;
            self.stats.supplies = (self.stats.supplies - DISEASE_SUPPLY_PENALTY).max(0);
            self.rest_requested = true;
            self.mark_damage(DamageCause::Disease);
            self.logs.push(String::from(LOG_DISEASE_TICK));
            let recovering = self.illness_days_remaining <= 1;
            self.illness_days_remaining = self.illness_days_remaining.saturating_sub(1);
            if recovering {
                self.clear_illness_penalty();
                self.disease_cooldown = DISEASE_COOLDOWN_DAYS;
            }
            return;
        }

        let Some(rng) = self.rng.as_mut() else {
            return;
        };

        if self.disease_cooldown > 0 {
            return;
        }

        let mut chance = DISEASE_DAILY_CHANCE;
        if self.stats.supplies <= 0 {
            chance += DISEASE_SUPPLIES_BONUS;
        }
        if self.starvation_days > 0 {
            chance += DISEASE_STARVATION_BONUS;
        }
        if self.stats.hp <= 4 {
            chance += DISEASE_LOW_HP_BONUS;
        }
        chance = chance.clamp(0.0, DISEASE_MAX_DAILY_CHANCE);

        if rng.random::<f32>() >= chance {
            return;
        }

        let duration = rng.random_range(DISEASE_DURATION_RANGE.0..=DISEASE_DURATION_RANGE.1);
        self.illness_days_remaining = duration;
        self.stats.hp -= DISEASE_HP_PENALTY;
        self.stats.sanity -= DISEASE_SANITY_PENALTY;
        self.stats.supplies = (self.stats.supplies - DISEASE_SUPPLY_PENALTY).max(0);
        self.disease_cooldown = DISEASE_COOLDOWN_DAYS;
        self.rest_requested = true;
        self.illness_travel_penalty = ILLNESS_TRAVEL_PENALTY;
        self.mark_damage(DamageCause::Disease);
        self.logs.push(String::from(LOG_DISEASE_HIT));
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
            Weather::ColdSnap => 0.98,
            Weather::Storm | Weather::Smoke => 0.99,
            Weather::HeatWave => 0.97,
            Weather::Clear => 1.0,
        }
    }

    #[must_use]
    fn behind_schedule_multiplier(&self) -> f32 {
        if self.day >= 140 && self.distance_traveled_actual < 1_850.0 {
            1.10
        } else if self.day >= 120 && self.distance_traveled_actual < 1_600.0 {
            1.075
        } else if self.day >= 90 && self.distance_traveled_actual < 1_200.0 {
            1.05
        } else {
            1.0
        }
    }

    #[must_use]
    fn compute_miles_for_today(
        &mut self,
        pace_cfg: &crate::pacing::PaceCfg,
        limits: &crate::pacing::PacingLimits,
    ) -> f32 {
        let travel_v2 = self.features.travel_v2;
        let mut base_distance = if pace_cfg.distance > 0.0 {
            pace_cfg.distance
        } else if limits.distance_base > 0.0 {
            limits.distance_base
        } else if travel_v2 {
            13.5
        } else {
            12.0
        };
        if base_distance <= 0.0 {
            base_distance = if travel_v2 { 13.5 } else { 12.0 };
        }

        let pace_scalar = if travel_v2 {
            let cfg_mult = if pace_cfg.dist_mult > 0.0 {
                pace_cfg.dist_mult
            } else {
                match self.pace {
                    PaceId::Steady => 1.0,
                    PaceId::Heated => 1.15,
                    PaceId::Blitz => 1.30,
                }
            };
            cfg_mult.max(0.1)
        } else {
            let config_scalar = pace_cfg.dist_mult.max(0.1);
            (match self.pace {
                PaceId::Steady => 1.0,
                PaceId::Heated => 1.15,
                PaceId::Blitz => 1.30,
            }) * config_scalar
        };

        let weather_scalar = if travel_v2 {
            self.weather_travel_multiplier.max(0.1)
        } else {
            self.current_weather_speed_penalty()
        };

        let penalty_floor = if travel_v2 {
            if limits.distance_penalty_floor > 0.0 {
                limits.distance_penalty_floor
            } else {
                0.7
            }
        } else {
            0.6
        };

        let mut pace_weather = weather_scalar * pace_scalar;
        if matches!(
            self.policy,
            Some(PolicyKind::Conservative | PolicyKind::Aggressive)
        ) {
            let booster = self.behind_schedule_multiplier();
            pace_weather *= booster;
        }
        let raw_distance = base_distance * pace_weather;
        let floored_multiplier = pace_weather.max(penalty_floor);
        let mut distance = base_distance * floored_multiplier;
        let mut partial_distance = (raw_distance * 0.5).max(0.0);

        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            distance *= 0.5;
            partial_distance *= 0.5;
        }

        if self.malnutrition_level > 0 {
            #[allow(clippy::cast_precision_loss)]
            let malnutrition = self.malnutrition_level as f32;
            let starvation_penalty = 1.0 - (malnutrition * 0.05);
            let starvation_penalty = starvation_penalty.max(0.3);
            distance *= starvation_penalty;
            partial_distance *= starvation_penalty;
        }

        distance *= self.exec_travel_multiplier;
        partial_distance *= self.exec_travel_multiplier;

        distance *= self.illness_travel_penalty.max(0.0);
        partial_distance *= self.illness_travel_penalty.max(0.0);

        self.distance_today_raw = raw_distance.max(0.0);
        self.distance_today = distance.max(1.0);
        self.partial_distance_today = partial_distance.max(0.0).min(self.distance_today);
        self.distance_today
    }

    fn check_vehicle_terminal_state(&mut self) -> bool {
        let spare_guard = self.total_spares();
        let base_tolerance = if self.mode.is_deep() { 4 } else { 5 };
        let tolerance = base_tolerance.max(spare_guard * 3);

        if self.vehicle.health <= 0.0 {
            let mut recovered = false;
            if spare_guard > 0 {
                recovered = self.consume_any_spare_for_emergency();
            }
            if !recovered && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.spend_emergency_repair(LOG_EMERGENCY_REPAIR_FORCED);
                recovered = true;
            }
            if !recovered && self.vehicle_breakdowns < tolerance {
                // Limp along by burning time; the vehicle barely holds together.
                self.vehicle.health = self.vehicle.health.max(VEHICLE_JURY_RIG_HEAL);
                self.pending_delay_days = self
                    .pending_delay_days
                    .saturating_add(VEHICLE_EMERGENCY_DELAY_DAYS);
                recovered = true;
            }
            if recovered {
                self.mark_damage(DamageCause::Vehicle);
            }
        }

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

    fn crossing_pressure(&self) -> CrossingPressure {
        match (self.region, self.season) {
            (Region::Heartland, Season::Winter)
            | (Region::RustBelt, Season::Fall | Season::Winter)
            | (Region::Beltway, Season::Summer | Season::Fall | Season::Winter) => {
                CrossingPressure::High
            }
            (Region::Heartland, Season::Summer | Season::Fall)
            | (Region::RustBelt, _)
            | (Region::Beltway, Season::Spring) => CrossingPressure::Medium,
            (Region::Heartland, _) => CrossingPressure::Low,
        }
    }

    fn crossing_threshold_entry(&self, cfg: &CrossingConfig) -> ThresholdEntry {
        cfg.thresholds.lookup(self.region, self.season)
    }

    fn select_crossing_kind(&mut self, next_idx: usize) -> CrossingKind {
        let forced_bridge = next_idx + 1 >= CROSSING_MILESTONES.len()
            || (self.mode.is_deep()
                && self.rng.as_mut().map_or(1.0, rand::Rng::random::<f32>) < 0.55);

        if forced_bridge {
            CrossingKind::BridgeOut
        } else {
            CrossingKind::Checkpoint
        }
    }

    fn try_crossing_permit(&mut self, cfg: &CrossingConfig, kind: CrossingKind) -> bool {
        if !crossings::can_use_permit(self, &kind) {
            return false;
        }

        let permit_log = crossings::apply_permit(self, cfg, kind);
        self.logs.push(String::from(LOG_CROSSING_DECISION_PERMIT));
        self.logs.push(permit_log);
        self.logs.push(String::from(LOG_CROSSING_PASSED));
        self.crossings_completed = self.crossings_completed.saturating_add(1);
        self.stats.clamp();
        self.apply_crossing_success_credit();
        true
    }

    fn attempt_crossing_bribe(
        &mut self,
        type_cfg: &CrossingTypeCfg,
        cost_multiplier: u32,
        success_adjust: f32,
        terminal_threshold: &mut f32,
    ) -> CrossingBribeOutcome {
        let base_bribe_cost = crossings::calculate_bribe_cost(
            type_cfg.bribe.base_cost_cents,
            self.mods.bribe_discount_pct,
        );
        let adjusted_bribe_cost = (base_bribe_cost * i64::from(cost_multiplier) + 50) / 100;

        if self.budget_cents < adjusted_bribe_cost {
            return CrossingBribeOutcome::default();
        }

        self.budget_cents -= adjusted_bribe_cost;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.bribes_spent_cents += adjusted_bribe_cost;
        self.logs.push(format!(
            "{LOG_CROSSING_DECISION_BRIBE}.cents{adjusted_bribe_cost}"
        ));

        let mut success_chance = (type_cfg.bribe.success_chance + success_adjust).clamp(0.05, 0.95);
        if self.stats.credibility >= 7 {
            success_chance += 0.05;
        }
        if self.stats.pants >= 60 {
            success_chance -= 0.05;
        }
        success_chance = success_chance.clamp(0.05, 0.95);

        let roll = self.rng.as_mut().map(rand::Rng::random::<f32>);
        if roll.unwrap_or(1.0) <= success_chance {
            return CrossingBribeOutcome {
                attempted: true,
                success: true,
                cost_cents: adjusted_bribe_cost,
                chance: Some(success_chance),
                roll,
            };
        }

        *terminal_threshold = (*terminal_threshold - 0.03).max(0.02);
        if let Ok(delay) = u32::try_from(type_cfg.bribe.on_fail.days.max(0))
            && delay > 0
        {
            self.pending_delay_days = self.pending_delay_days.saturating_add(delay);
            self.delay_partial_days = self
                .delay_partial_days
                .saturating_add(delay.saturating_sub(1));
            self.apply_delay_travel_credit();
        }
        if type_cfg.bribe.on_fail.pants != 0 {
            self.stats.pants += type_cfg.bribe.on_fail.pants;
        }

        CrossingBribeOutcome {
            attempted: true,
            success: false,
            cost_cents: adjusted_bribe_cost,
            chance: Some(success_chance),
            roll,
        }
    }

    fn apply_crossing_detour(
        &mut self,
        cfg: &CrossingConfig,
        kind: CrossingKind,
        pressure: CrossingPressure,
        bribe_attempted: bool,
        terminal_threshold: &mut f32,
    ) -> CrossingDetourResolution {
        self.logs.push(String::from(LOG_CROSSING_DECISION_DETOUR));
        let supplies_before = self.stats.supplies;
        let pants_before = self.stats.pants;
        let detour_log = crossings::apply_detour(self, cfg, kind);
        self.logs.push(detour_log);
        self.crossing_detours = self.crossing_detours.saturating_add(1);

        let base_supplies_delta = self.stats.supplies - supplies_before;
        let pants_delta = self.stats.pants - pants_before;

        let detour_days = if let Some(rng) = self.rng.as_mut() {
            rng.random_range(DETOUR_DAY_RANGE.0..=DETOUR_DAY_RANGE.1)
        } else {
            u32::midpoint(DETOUR_DAY_RANGE.0, DETOUR_DAY_RANGE.1)
        };
        self.pending_delay_days = self.pending_delay_days.saturating_add(detour_days);
        self.delay_partial_days = self
            .delay_partial_days
            .saturating_add(detour_days.saturating_sub(1));
        self.apply_delay_travel_credit();

        let extra_supply_loss = if let Some(rng) = self.rng.as_mut() {
            rng.random_range(DETOUR_SUPPLY_LOSS_RANGE.0..=DETOUR_SUPPLY_LOSS_RANGE.1)
        } else {
            i32::midpoint(DETOUR_SUPPLY_LOSS_RANGE.0, DETOUR_SUPPLY_LOSS_RANGE.1)
        };
        let supplies_before_extra = self.stats.supplies;
        self.stats.supplies = (self.stats.supplies - extra_supply_loss).max(0);
        let actual_extra_loss = supplies_before_extra - self.stats.supplies;

        self.logs.push(String::from(LOG_CROSSING_DETOUR));
        self.logs.push(format!(
            "{LOG_CROSSING_DETOUR}.days{detour_days}.supplies{actual_extra_loss}"
        ));

        if !bribe_attempted && matches!(pressure, CrossingPressure::High) {
            *terminal_threshold = (*terminal_threshold + 0.02).min(0.15);
        } else {
            *terminal_threshold = (*terminal_threshold).min(0.15);
        }

        let terminal_roll = self
            .rng
            .as_mut()
            .map_or(1.0, rand::Rng::random::<f32>);

        if terminal_roll < *terminal_threshold {
            self.crossing_failures = self.crossing_failures.saturating_add(1);
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Crossing,
            });
            self.logs.push(String::from(LOG_CROSSING_FAILURE));
            return CrossingDetourResolution {
                log_key: Some(LOG_CROSSING_FAILURE),
                detour_days,
                base_supplies_delta,
                extra_supply_loss: actual_extra_loss,
                pants_delta,
                terminal_roll,
                failed: true,
            };
        }

        self.crossings_completed = self.crossings_completed.saturating_add(1);
        self.stats.clamp();
        CrossingDetourResolution {
            log_key: None,
            detour_days,
            base_supplies_delta,
            extra_supply_loss: actual_extra_loss,
            pants_delta,
            terminal_roll,
            failed: false,
        }
    }

    fn maybe_trigger_crossing_event(&mut self) -> Option<&'static str> {
        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let &milestone = CROSSING_MILESTONES.get(next_idx)?;
        if self.distance_traveled_actual < milestone {
            return None;
        }

        let cfg = CrossingConfig::default();
        let kind = self.select_crossing_kind(next_idx);
        let type_cfg = cfg.types.get(&kind)?;

        let pressure = self.crossing_pressure();
        let thresholds = self.crossing_threshold_entry(&cfg);

        let mut terminal_threshold =
            (CROSSING_FAILURE_BASE + thresholds.failure_adjust).clamp(0.0, 0.15);
        if self.mode.is_deep() {
            terminal_threshold = (terminal_threshold + CROSSING_FAILURE_DEEP_BONUS).min(0.15);
        }

        let mut telemetry = CrossingTelemetry::new(self.day, self.region, self.season, kind);
        telemetry.terminal_threshold = terminal_threshold;

        if self.try_crossing_permit(&cfg, kind) {
            telemetry.permit_used = true;
            telemetry.outcome = CrossingOutcomeTelemetry::Passed;
            self.crossing_events.push(telemetry);
            return None;
        }

        let bribe_outcome = self.attempt_crossing_bribe(
            type_cfg,
            thresholds.cost_multiplier,
            thresholds.success_adjust,
            &mut terminal_threshold,
        );

        telemetry.bribe_attempted = bribe_outcome.attempted;
        telemetry.bribe_cost_cents = bribe_outcome.cost_cents;
        telemetry.bribe_chance = bribe_outcome.chance;
        telemetry.bribe_roll = bribe_outcome.roll;
        if bribe_outcome.attempted {
            telemetry.bribe_success = Some(bribe_outcome.success);
        }

        if bribe_outcome.success {
            self.logs.push(String::from(LOG_CROSSING_PASSED));
            self.crossings_completed = self.crossings_completed.saturating_add(1);
            self.stats.clamp();
            self.apply_crossing_success_credit();
            telemetry.outcome = CrossingOutcomeTelemetry::Passed;
            telemetry.terminal_threshold = terminal_threshold.min(0.15);
            self.crossing_events.push(telemetry);
            return None;
        }

        let detour = self.apply_crossing_detour(
            &cfg,
            kind,
            pressure,
            bribe_outcome.attempted,
            &mut terminal_threshold,
        );

        telemetry.detour_taken = true;
        telemetry.detour_days = Some(detour.detour_days);
        telemetry.detour_base_supplies_delta = Some(detour.base_supplies_delta);
        telemetry.detour_extra_supplies_loss = Some(detour.extra_supply_loss);
        telemetry.detour_pants_delta = Some(detour.pants_delta);
        telemetry.terminal_roll = Some(detour.terminal_roll);
        telemetry.terminal_threshold = terminal_threshold.min(0.15);
        telemetry.outcome = if detour.failed {
            CrossingOutcomeTelemetry::Failed
        } else {
            CrossingOutcomeTelemetry::Detoured
        };
        self.crossing_events.push(telemetry);

        detour.log_key
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

    #[allow(clippy::too_many_lines)]
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
            self.days_with_repair = self.days_with_repair.saturating_add(1);
            if !self.partial_traveled_today {
                self.apply_delay_travel_credit();
            }
            self.end_of_day();
            return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
        }

        let mut trigger_encounter = false;
        if self.encounter_occurred_today {
            // Already had an encounter; keep trigger false.
        } else if let Some(rng) = self.rng.as_mut() {
            let roll: f32 = rng.random();
            if roll < self.encounter_chance_today {
                trigger_encounter = true;
            }
        }

        if self.encounters_today >= MAX_ENCOUNTERS_PER_DAY {
            trigger_encounter = false;
        }

        if trigger_encounter {
            let recent_snapshot: Vec<RecentEncounter> =
                self.recent_encounters.iter().cloned().collect();
            let mut encounter = if let (Some(rng), Some(data)) =
                (self.rng.as_mut(), self.data.as_ref())
            {
                let forced = self.force_rotation_pending;
                let request = EncounterRequest {
                    region: self.region,
                    is_deep: self.mode.is_deep(),
                    malnutrition_level: self.malnutrition_level,
                    starving: self.stats.supplies <= 0,
                    data,
                    recent: &recent_snapshot,
                    current_day: self.day,
                    policy: self.policy,
                    force_rotation: forced,
                };
                let (pick, satisfied) = pick_encounter(&request, &mut self.rotation_backlog, rng);
                if forced {
                    if satisfied {
                        self.logs.push(String::from(LOG_ENCOUNTER_ROTATION));
                    }
                    self.force_rotation_pending = !self.rotation_backlog.is_empty();
                }
                pick
            } else {
                None
            };

            let should_reroll = encounter.as_ref().is_some_and(|enc| {
                self.features.encounter_diversity && self.should_discourage_encounter(&enc.id)
            });

            if should_reroll {
                let reroll_penalty = self.encounter_reroll_penalty();
                if let (Some(rng), Some(data)) = (self.rng.as_mut(), self.data.as_ref()) {
                    let reroll_trigger = rng.random::<f32>() < reroll_penalty;
                    if reroll_trigger {
                        let request = EncounterRequest {
                            region: self.region,
                            is_deep: self.mode.is_deep(),
                            malnutrition_level: self.malnutrition_level,
                            starving: self.stats.supplies <= 0,
                            data,
                            recent: &recent_snapshot,
                            current_day: self.day,
                            policy: self.policy,
                            force_rotation: false,
                        };
                        let (replacement, satisfied) =
                            pick_encounter(&request, &mut self.rotation_backlog, rng);
                        if satisfied {
                            self.force_rotation_pending = false;
                        }
                        encounter = replacement;
                    }
                }
            }

            if let Some(enc) = encounter {
                let is_hard_stop = enc.hard_stop;
                let is_major_repair = enc.major_repair;
                if self.features.travel_v2
                    && self.distance_today > 0.0
                    && !(is_hard_stop || is_major_repair)
                {
                    let mut partial = if self.partial_distance_today > 0.0 {
                        self.partial_distance_today
                    } else {
                        self.distance_today * 0.92
                    };
                    partial = partial.min(self.distance_today);
                    let wear_scale = if self.distance_today > 0.0 {
                        (partial / self.distance_today).clamp(0.55, 0.99)
                    } else {
                        0.85
                    };
                    self.apply_travel_progress(partial, TravelProgressKind::Partial);
                    self.apply_travel_wear_scaled(wear_scale);
                    self.logs.push(String::from(LOG_TRAVEL_PARTIAL));
                }
                if is_major_repair {
                    self.days_with_repair = self.days_with_repair.saturating_add(1);
                }
                let encounter_id = enc.id.clone();
                self.current_encounter = Some(enc);
                self.encounter_occurred_today = true;
                self.record_encounter(&encounter_id);
                return (false, String::from("log.encounter"), breakdown_started);
            }
        }

        if self.features.travel_v2 {
            self.apply_travel_wear();
        }

        self.apply_travel_progress(self.distance_today, TravelProgressKind::Full);
        if let Some(crossing_log) = self.maybe_trigger_crossing_event() {
            self.end_of_day();
            return (true, String::from(crossing_log), breakdown_started);
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
        if self.pending_delay_days > 0 {
            let extra = self.pending_delay_days;
            self.pending_delay_days = 0;
            if extra > 0 {
                self.advance_days(extra);
            }
        }
        (false, String::from(LOG_TRAVELED), breakdown_started)
    }

    /// Apply vehicle breakdown logic
    fn vehicle_roll(&mut self) -> bool {
        if self.breakdown.is_some() {
            return false;
        }

        let Some(rng) = self.rng.as_mut() else {
            return false;
        };

        let mut breakdown_chance = 0.04 + self.exec_breakdown_bonus;
        breakdown_chance += (self.vehicle.wear / VEHICLE_HEALTH_MAX) * 0.2;
        if self.weather_state.today.is_extreme() {
            breakdown_chance += 0.04;
        }
        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            breakdown_chance += 0.05;
        }

        let pace_factor = match self.pace {
            PaceId::Steady => 0.95,
            PaceId::Heated => 1.0,
            PaceId::Blitz => 1.10,
        };
        breakdown_chance *= pace_factor;
        if matches!(self.policy, Some(PolicyKind::Conservative)) {
            breakdown_chance *= 0.90;
        }
        breakdown_chance = breakdown_chance.clamp(0.0, 1.0);

        let roll: f32 = rng.random();
        if roll >= breakdown_chance {
            return false;
        }

        let parts = [Part::Tire, Part::Battery, Part::Alternator, Part::FuelPump];
        let part_idx: usize = rng.random_range(0..parts.len());
        self.breakdown = Some(crate::vehicle::Breakdown {
            part: parts[part_idx],
            day_started: i32::try_from(self.day).unwrap_or(0),
        });
        self.travel_blocked = true;
        self.vehicle_breakdowns += 1;
        self.vehicle.apply_damage(VEHICLE_BREAKDOWN_DAMAGE);
        let breakdown_wear = if self.mode.is_deep() {
            VEHICLE_BREAKDOWN_WEAR
        } else {
            VEHICLE_BREAKDOWN_WEAR_CLASSIC
        };
        self.vehicle.wear = (self.vehicle.wear + breakdown_wear).min(VEHICLE_HEALTH_MAX);
        self.mark_damage(DamageCause::Vehicle);
        if debug_log_enabled() {
            println!(
                "🚗 Breakdown started: {:?} | health {} | roll {:.3} chance {:.3}",
                parts[part_idx], self.vehicle.health, roll, breakdown_chance
            );
        }
        true
    }

    pub fn apply_choice(&mut self, idx: usize) {
        let Some(enc) = self.current_encounter.clone() else {
            self.finalize_encounter();
            return;
        };

        if let Some(choice) = enc.choices.get(idx) {
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

            if eff.travel_bonus_ratio > 0.0 {
                let baseline = if self.distance_today > 0.0 {
                    self.distance_today
                } else if self.distance_today_raw > 0.0 {
                    self.distance_today_raw
                } else if self.features.travel_v2 {
                    13.5
                } else {
                    12.0
                };
                let bonus = (baseline * eff.travel_bonus_ratio).max(0.0);
                if bonus > 0.0 {
                    self.apply_partial_travel_credit(bonus, LOG_TRAVEL_BONUS);
                }
            }
            if eff.rest {
                if !self.rest_requested {
                    self.logs.push(String::from(LOG_REST_REQUESTED_ENCOUNTER));
                }
                self.request_rest();
            }
        }

        self.finalize_encounter();
    }

    fn resolve_breakdown(&mut self) {
        if let Some(breakdown) = self.breakdown.clone() {
            if self.consume_spare_for_part(breakdown.part) {
                self.vehicle.repair(VEHICLE_JURY_RIG_HEAL);
                self.breakdown = None;
                self.travel_blocked = false;
                self.logs.push(String::from("log.breakdown-repaired"));
                return;
            }

            if self.total_spares() == 0 && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.spend_emergency_repair(LOG_VEHICLE_REPAIR_EMERGENCY);
                self.breakdown = None;
                self.travel_blocked = false;
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

    fn consume_any_spare_for_emergency(&mut self) -> bool {
        let spares = &mut self.inventory.spares;
        let used = if spares.tire > 0 {
            spares.tire -= 1;
            true
        } else if spares.battery > 0 {
            spares.battery -= 1;
            true
        } else if spares.alt > 0 {
            spares.alt -= 1;
            true
        } else if spares.pump > 0 {
            spares.pump -= 1;
            true
        } else {
            false
        };
        if !used {
            return false;
        }
        self.vehicle.repair(VEHICLE_JURY_RIG_HEAL);
        self.exec_travel_multiplier = (self.exec_travel_multiplier * 0.85).max(0.7);
        self.logs.push(String::from(LOG_VEHICLE_REPAIR_SPARE));
        true
    }

    fn spend_emergency_repair(&mut self, log_key: &'static str) {
        self.budget_cents = (self.budget_cents - EMERGENCY_REPAIR_COST).max(0);
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += EMERGENCY_REPAIR_COST;
        self.vehicle.repair(VEHICLE_EMERGENCY_HEAL);
        self.exec_travel_multiplier = (self.exec_travel_multiplier * 0.85).max(0.7);
        self.logs.push(String::from(log_key));
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

    pub fn clear_illness_penalty(&mut self) {
        let was_ill = self.illness_days_remaining > 0 || self.illness_travel_penalty < 1.0;
        self.illness_days_remaining = 0;
        self.illness_travel_penalty = 1.0;
        if was_ill {
            self.logs.push(String::from(LOG_DISEASE_RECOVER));
            self.disease_cooldown = self.disease_cooldown.max(DISEASE_COOLDOWN_DAYS);
        }
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

        let _ = self.compute_miles_for_today(&pace_cfg, limits);

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
                    DamageCause::Disease => {
                        self.set_ending(Ending::Collapse {
                            cause: CollapseCause::Disease,
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
            if self.delay_partial_days > 0 {
                self.delay_partial_days -= 1;
                self.apply_delay_travel_credit();
            }
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
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);

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
