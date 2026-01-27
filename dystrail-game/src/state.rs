use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::cell::RefMut;
use std::collections::{HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::OnceLock;

use crate::camp::CampState;
#[cfg(debug_assertions)]
use crate::constants::DEBUG_ENV_VAR;
use crate::constants::{
    AGGRESSIVE_STOP_CAP, AGGRESSIVE_STOP_WINDOW_DAYS, ALLY_ATTRITION_CHANCE,
    BEHIND_SCHEDULE_MILES_PER_DAY, BOSS_COMPOSE_FUNDS_COST, BOSS_COMPOSE_FUNDS_PANTS,
    BOSS_COMPOSE_PANTS_SUPPLY, BOSS_COMPOSE_SUPPLY_COST, CLASSIC_BALANCED_FAILURE_GUARD_MILES,
    CLASSIC_BALANCED_TRAVEL_NUDGE, CLASSIC_FIELD_REPAIR_COST_CENTS,
    CLASSIC_FIELD_REPAIR_WEAR_REDUCTION, CROSSING_MILESTONES, DEEP_AGGRESSIVE_BOOSTS,
    DEEP_AGGRESSIVE_BOSS_BIAS_MILES, DEEP_AGGRESSIVE_SANITY_COST, DEEP_AGGRESSIVE_SANITY_DAY,
    DEEP_AGGRESSIVE_SANITY_MILES, DEEP_AGGRESSIVE_SANITY_PANTS_PENALTY,
    DEEP_AGGRESSIVE_TOLERANCE_THRESHOLDS, DEEP_BALANCED_FAILSAFE_DISTANCE,
    DEEP_BALANCED_TOLERANCE_THRESHOLDS, DEEP_BALANCED_TRAVEL_NUDGE, DEEP_CONSERVATIVE_BOOSTS,
    DEEP_EMERGENCY_REPAIR_THRESHOLD, DELAY_TRAVEL_CREDIT_MILES, DISEASE_COOLDOWN_DAYS,
    DISEASE_DAILY_CHANCE, DISEASE_DURATION_RANGE, DISEASE_HP_PENALTY, DISEASE_LOW_HP_BONUS,
    DISEASE_MAX_DAILY_CHANCE, DISEASE_SANITY_PENALTY, DISEASE_STARVATION_BONUS,
    DISEASE_SUPPLIES_BONUS, DISEASE_SUPPLY_PENALTY, DISEASE_TICK_HP_LOSS, DISEASE_TICK_SANITY_LOSS,
    EMERGENCY_LIMP_MILE_WINDOW, EMERGENCY_LIMP_REPAIR_COST_CENTS, EMERGENCY_LIMP_WEAR_REDUCTION,
    EMERGENCY_REPAIR_COST, ENCOUNTER_BASE_DEFAULT, ENCOUNTER_COOLDOWN_DAYS,
    ENCOUNTER_CRITICAL_VEHICLE_BONUS, ENCOUNTER_EXTENDED_MEMORY_DAYS, ENCOUNTER_HISTORY_WINDOW,
    ENCOUNTER_RECENT_MEMORY, ENCOUNTER_REPEAT_WINDOW_DAYS, ENCOUNTER_REROLL_PENALTY,
    ENCOUNTER_SOFT_CAP_FACTOR, ENCOUNTER_SOFT_CAP_THRESHOLD, EXEC_BREAKDOWN_BONUS_CLAMP_MAX,
    EXEC_ORDER_BREAKDOWN_BONUS, EXEC_ORDER_DAILY_CHANCE, EXEC_ORDER_MAX_COOLDOWN,
    EXEC_ORDER_MAX_DURATION, EXEC_ORDER_MIN_COOLDOWN, EXEC_ORDER_MIN_DURATION,
    EXEC_ORDER_SPEED_BONUS, EXEC_TRAVEL_MULTIPLIER_CLAMP_MIN, ILLNESS_TRAVEL_PENALTY,
    LOG_ALLIES_GONE, LOG_ALLY_LOST, LOG_BOSS_AWAIT, LOG_BOSS_COMPOSE, LOG_BOSS_COMPOSE_FUNDS,
    LOG_BOSS_COMPOSE_SUPPLIES, LOG_CROSSING_DECISION_BRIBE, LOG_CROSSING_DECISION_PERMIT,
    LOG_CROSSING_DETOUR, LOG_CROSSING_FAILURE, LOG_CROSSING_PASSED,
    LOG_DEEP_AGGRESSIVE_FIELD_REPAIR, LOG_DISEASE_HIT, LOG_DISEASE_RECOVER, LOG_DISEASE_TICK,
    LOG_EMERGENCY_REPAIR_FORCED, LOG_ENCOUNTER_ROTATION, LOG_EXEC_END_PREFIX,
    LOG_EXEC_START_PREFIX, LOG_HEALTH_COLLAPSE, LOG_OT_CROSSING_DROWNED, LOG_OT_CROSSING_SAFE,
    LOG_OT_CROSSING_SANK, LOG_OT_CROSSING_STUCK, LOG_OT_CROSSING_TIPPED, LOG_OT_CROSSING_WET,
    LOG_PANTS_EMERGENCY, LOG_REST_REQUESTED_ENCOUNTER, LOG_SANITY_COLLAPSE,
    LOG_STARVATION_BACKSTOP, LOG_STARVATION_RELIEF, LOG_STARVATION_TICK, LOG_TRAVEL_BLOCKED,
    LOG_TRAVEL_BONUS, LOG_TRAVEL_DELAY_CREDIT, LOG_TRAVEL_PARTIAL, LOG_TRAVEL_REST_CREDIT,
    LOG_VEHICLE_EMERGENCY_LIMP, LOG_VEHICLE_FAILURE, LOG_VEHICLE_FIELD_REPAIR_GUARD,
    LOG_VEHICLE_REPAIR_EMERGENCY, LOG_VEHICLE_REPAIR_SPARE, MAX_ENCOUNTERS_PER_DAY,
    PROBABILITY_FLOOR, PROBABILITY_MAX, REST_TRAVEL_CREDIT_MILES, ROTATION_FORCE_INTERVAL,
    SANITY_POINT_REWARD, STARVATION_BASE_HP_LOSS, STARVATION_GRACE_DAYS, STARVATION_MAX_STACK,
    STARVATION_PANTS_GAIN, STARVATION_SANITY_LOSS, TRAVEL_CLASSIC_BASE_DISTANCE,
    TRAVEL_CLASSIC_PENALTY_FLOOR, TRAVEL_CONFIG_MIN_MULTIPLIER, TRAVEL_HISTORY_WINDOW,
    TRAVEL_PARTIAL_CLAMP_HIGH, TRAVEL_PARTIAL_CLAMP_LOW, TRAVEL_PARTIAL_MIN_DISTANCE,
    TRAVEL_PARTIAL_RATIO, TRAVEL_PARTIAL_RECOVERY_RATIO, TRAVEL_RATIO_DEFAULT,
    TRAVEL_V2_BASE_DISTANCE, TRAVEL_V2_PENALTY_FLOOR, VEHICLE_BASE_TOLERANCE_CLASSIC,
    VEHICLE_BASE_TOLERANCE_DEEP, VEHICLE_BREAKDOWN_DAMAGE, VEHICLE_BREAKDOWN_PARTIAL_FACTOR,
    VEHICLE_BREAKDOWN_WEAR, VEHICLE_BREAKDOWN_WEAR_CLASSIC, VEHICLE_CRITICAL_SPEED_FACTOR,
    VEHICLE_CRITICAL_THRESHOLD, VEHICLE_DEEP_EMERGENCY_HEAL_AGGRESSIVE,
    VEHICLE_DEEP_EMERGENCY_HEAL_BALANCED, VEHICLE_EMERGENCY_HEAL, VEHICLE_EXEC_MULTIPLIER_DECAY,
    VEHICLE_EXEC_MULTIPLIER_FLOOR, VEHICLE_HEALTH_MAX, VEHICLE_JURY_RIG_HEAL,
    VEHICLE_MALNUTRITION_MIN_FACTOR, VEHICLE_MALNUTRITION_PENALTY_PER_STACK,
    VEHICLE_SPARE_GUARD_SCALE, WEATHER_COLD_SNAP_SPEED, WEATHER_DEFAULT_SPEED,
    WEATHER_HEAT_WAVE_SPEED, WEATHER_PACE_MULTIPLIER_FLOOR, WEATHER_STORM_SMOKE_SPEED,
};
#[cfg(test)]
use crate::constants::{ASSERT_MIN_AVG_MPD, FLOAT_EPSILON};
use crate::crossings::{self, CrossingChoice, CrossingConfig, CrossingContext, CrossingKind};
use crate::data::{Encounter, EncounterData};
use crate::day_accounting::{self, DayLedgerMetrics};
use crate::disease::{
    DiseaseCatalog, DiseaseDef, DiseaseEffects, DiseaseKind, FatalityModel, FatalityModifier,
};
use crate::encounters::{EncounterRequest, pick_encounter};
use crate::endgame::{self, EndgameState};
use crate::exec_orders::{ExecOrder, ExecOrderEffects};
use crate::journey::{
    BreakdownConfig, CountingRng, CrossingPolicy, DayRecord, DayTag, DayTagSet, Event,
    EventDecisionTrace, EventId, EventKind, EventSeverity, JourneyCfg, MechanicalPolicyId,
    RngBundle, RollValue, StrainConfig, TravelConfig, TravelDayKind, UiSurfaceHint, WearConfig,
    WeightedCandidate,
};
use crate::mechanics::otdeluxe90s::{
    OtDeluxe90sPolicy, OtDeluxeAfflictionPolicy, OtDeluxeHealthPolicy, OtDeluxeNavigationDelay,
    OtDeluxeNavigationPolicy, OtDeluxeOccupation, OtDeluxePace, OtDeluxePaceHealthPolicy,
    OtDeluxePolicyOverride, OtDeluxeRations, OtDeluxeRationsPolicy, OtDeluxeTrailVariant,
    OtDeluxeTravelPolicy,
};
use crate::numbers::round_f32_to_i32;
use crate::otdeluxe_crossings;
use crate::otdeluxe_random_events::{
    self, OtDeluxeRandomEventContext, OtDeluxeRandomEventSelection,
};
#[cfg(test)]
use crate::otdeluxe_state::OtDeluxeTravelState;
use crate::otdeluxe_state::{
    OtDeluxeAfflictionKind, OtDeluxeAfflictionOutcome, OtDeluxeCalendar, OtDeluxeCrossingMethod,
    OtDeluxeDallesChoice, OtDeluxeInventory, OtDeluxePartyState, OtDeluxeRiver, OtDeluxeRiverState,
    OtDeluxeRouteDecision, OtDeluxeRoutePrompt, OtDeluxeState, OtDeluxeTerrain, OtDeluxeWagonState,
};
use crate::otdeluxe_store::{OtDeluxeStoreError, OtDeluxeStoreLineItem, OtDeluxeStoreReceipt};
use crate::otdeluxe_trail;
use crate::pacing::PacingLimits;
use crate::personas::{Persona, PersonaMods};
use crate::vehicle::{Breakdown, Part, PartWeights, Vehicle};
use crate::weather::{Weather, WeatherEffects, WeatherState};

const ENCOUNTER_UNIQUE_WINDOW: u32 = 20;
const ENCOUNTER_UNIQUE_RATIO_FLOOR: f32 = 0.075;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
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
}

impl PolicyKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Conservative => "conservative",
            Self::Aggressive => "aggressive",
            Self::ResourceManager => "resource_manager",
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
const fn default_pace() -> PaceId {
    PaceId::Steady
}

fn default_otdeluxe_policy() -> &'static OtDeluxe90sPolicy {
    static POLICY: OnceLock<OtDeluxe90sPolicy> = OnceLock::new();
    POLICY.get_or_init(OtDeluxe90sPolicy::default)
}

fn otdeluxe_starting_cash_cents(occupation: OtDeluxeOccupation, policy: &OtDeluxe90sPolicy) -> u32 {
    let dollars = policy
        .occupations
        .iter()
        .find(|spec| spec.occupation == occupation)
        .map_or(0, |spec| spec.starting_cash_dollars);
    u32::from(dollars).saturating_mul(100)
}

fn otdeluxe_affliction_probability(health_general: u16, policy: &OtDeluxeAfflictionPolicy) -> f32 {
    let mut probability = policy.curve_pwl[0].probability;
    if health_general <= policy.curve_pwl[0].health {
        return probability.clamp(0.0, policy.probability_max);
    }
    for window in policy.curve_pwl.windows(2) {
        let start = window[0];
        let end = window[1];
        if health_general <= end.health {
            let span = f32::from(end.health.saturating_sub(start.health));
            if span > 0.0 {
                let offset = f32::from(health_general.saturating_sub(start.health));
                let t = (offset / span).clamp(0.0, 1.0);
                probability = start.probability.mul_add(1.0 - t, end.probability * t);
            }
            return probability.clamp(0.0, policy.probability_max);
        }
    }
    if let Some(last) = policy.curve_pwl.last() {
        probability = last.probability;
    }
    probability.clamp(0.0, policy.probability_max)
}

fn roll_otdeluxe_affliction_kind<R>(
    policy: &OtDeluxeAfflictionPolicy,
    overrides: &OtDeluxePolicyOverride,
    rng: &mut R,
) -> (OtDeluxeAfflictionKind, Option<EventDecisionTrace>)
where
    R: rand::Rng + ?Sized,
{
    let weights = &overrides.affliction_weights;
    let illness_weight = weights.illness.unwrap_or(policy.weight_illness);
    let injury_weight = weights.injury.unwrap_or(policy.weight_injury);
    let total = u32::from(illness_weight) + u32::from(injury_weight);
    if total == 0 {
        return (OtDeluxeAfflictionKind::Illness, None);
    }
    let roll = rng.gen_range(0..total);
    let kind = if roll < u32::from(illness_weight) {
        OtDeluxeAfflictionKind::Illness
    } else {
        OtDeluxeAfflictionKind::Injury
    };
    let candidates = vec![
        WeightedCandidate {
            id: String::from("illness"),
            base_weight: f64::from(illness_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(illness_weight),
        },
        WeightedCandidate {
            id: String::from("injury"),
            base_weight: f64::from(injury_weight),
            multipliers: Vec::new(),
            final_weight: f64::from(injury_weight),
        },
    ];
    let chosen_id = match kind {
        OtDeluxeAfflictionKind::Illness => "illness",
        OtDeluxeAfflictionKind::Injury => "injury",
    };
    let trace = EventDecisionTrace {
        pool_id: String::from("otdeluxe.affliction_kind"),
        roll: RollValue::U32(roll),
        candidates,
        chosen_id: chosen_id.to_string(),
    };
    (kind, Some(trace))
}

const fn otdeluxe_pace_health_penalty(
    pace: OtDeluxePace,
    policy: &OtDeluxePaceHealthPolicy,
) -> i32 {
    match pace {
        OtDeluxePace::Steady => policy.steady,
        OtDeluxePace::Strenuous => policy.strenuous,
        OtDeluxePace::Grueling => policy.grueling,
    }
}

const fn otdeluxe_pace_food_multiplier(pace: OtDeluxePace, policy: &OtDeluxe90sPolicy) -> f32 {
    let mult = match pace {
        OtDeluxePace::Steady => policy.pace_mult_steady,
        OtDeluxePace::Strenuous => policy.pace_mult_strenuous,
        OtDeluxePace::Grueling => policy.pace_mult_grueling,
    };
    if mult < 0.0 { 0.0 } else { mult }
}

const fn otdeluxe_rations_food_per_person(
    rations: OtDeluxeRations,
    policy: &OtDeluxeRationsPolicy,
) -> u16 {
    match rations {
        OtDeluxeRations::Filling => policy.food_lbs_per_person[0],
        OtDeluxeRations::Meager => policy.food_lbs_per_person[1],
        OtDeluxeRations::BareBones => policy.food_lbs_per_person[2],
    }
}

const fn otdeluxe_rations_health_penalty(
    rations: OtDeluxeRations,
    policy: &OtDeluxeRationsPolicy,
) -> i32 {
    match rations {
        OtDeluxeRations::Filling => policy.health_penalty[0],
        OtDeluxeRations::Meager => policy.health_penalty[1],
        OtDeluxeRations::BareBones => policy.health_penalty[2],
    }
}

fn otdeluxe_rations_food_per_person_scaled(
    rations: OtDeluxeRations,
    pace: OtDeluxePace,
    policy: &OtDeluxe90sPolicy,
) -> u16 {
    let per_person = otdeluxe_rations_food_per_person(rations, &policy.rations);
    let scaled = f32::from(per_person) * otdeluxe_pace_food_multiplier(pace, policy);
    let rounded = round_f32_to_i32(scaled).max(0);
    u16::try_from(rounded).unwrap_or(u16::MAX)
}

fn otdeluxe_weather_health_penalty(weather: Weather, policy: &OtDeluxeHealthPolicy) -> i32 {
    *policy.weather_penalty.get(&weather).unwrap_or(&0)
}

fn otdeluxe_snow_speed_mult(snow_depth: f32, policy: &OtDeluxeTravelPolicy) -> f32 {
    if !snow_depth.is_finite() {
        return 1.0;
    }
    let penalty_per_in = policy.snow_speed_penalty_per_in.max(0.0);
    if penalty_per_in <= 0.0 {
        return 1.0;
    }
    let floor = policy.snow_speed_floor.clamp(0.0, 1.0);
    let depth = snow_depth.max(0.0);
    let mult = 1.0 - depth * penalty_per_in;
    mult.clamp(floor, 1.0)
}

fn otdeluxe_clothing_health_penalty(
    season: Season,
    inventory: &OtDeluxeInventory,
    alive: u16,
    policy: &OtDeluxeHealthPolicy,
) -> i32 {
    if season != Season::Winter {
        return 0;
    }
    if policy.clothing_penalty_winter == 0 || policy.clothing_sets_per_person == 0 {
        return 0;
    }
    let needed = u32::from(policy.clothing_sets_per_person).saturating_mul(u32::from(alive));
    if u32::from(inventory.clothes_sets) < needed {
        policy.clothing_penalty_winter
    } else {
        0
    }
}

fn otdeluxe_affliction_health_penalty(
    party: &OtDeluxePartyState,
    policy: &OtDeluxeHealthPolicy,
) -> i32 {
    let sick = i64::from(party.sick_count());
    let injured = i64::from(party.injured_count());
    let illness_penalty = i64::from(policy.affliction_illness_penalty);
    let injury_penalty = i64::from(policy.affliction_injury_penalty);
    let total = sick.saturating_mul(illness_penalty) + injured.saturating_mul(injury_penalty);
    i32::try_from(total.clamp(i64::from(i32::MIN), i64::from(i32::MAX))).unwrap_or(0)
}

fn otdeluxe_drought_health_penalty(rain_accum: f32, policy: &OtDeluxeHealthPolicy) -> i32 {
    if !rain_accum.is_finite() || policy.drought_threshold <= 0.0 {
        return 0;
    }
    if rain_accum <= policy.drought_threshold {
        policy.drought_penalty
    } else {
        0
    }
}

fn otdeluxe_health_delta(state: &GameState, policy: &OtDeluxe90sPolicy) -> i32 {
    let pace_penalty =
        otdeluxe_pace_health_penalty(state.ot_deluxe.pace, &policy.pace_health_penalty);
    let rations_penalty = otdeluxe_rations_health_penalty(state.ot_deluxe.rations, &policy.rations);
    let weather_penalty =
        otdeluxe_weather_health_penalty(state.weather_state.today, &policy.health);
    let alive = state.otdeluxe_alive_party_count();
    let clothing_penalty = otdeluxe_clothing_health_penalty(
        state.ot_deluxe.season,
        &state.ot_deluxe.inventory,
        alive,
        &policy.health,
    );
    let affliction_penalty =
        otdeluxe_affliction_health_penalty(&state.ot_deluxe.party, &policy.health);
    let drought_penalty =
        otdeluxe_drought_health_penalty(state.ot_deluxe.weather.rain_accum, &policy.health);
    policy.health.recovery_baseline
        + pace_penalty
        + rations_penalty
        + weather_penalty
        + clothing_penalty
        + affliction_penalty
        + drought_penalty
}

fn otdeluxe_affliction_duration(
    kind: OtDeluxeAfflictionKind,
    policy: &OtDeluxeAfflictionPolicy,
) -> u8 {
    let duration = match kind {
        OtDeluxeAfflictionKind::Illness => policy.illness_duration_days,
        OtDeluxeAfflictionKind::Injury => policy.injury_duration_days,
    };
    duration.max(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtDeluxeHealthLabel {
    Good,
    Fair,
    Poor,
    VeryPoor,
}

#[derive(Debug, Clone, Copy)]
struct OtDeluxeFatalityContext {
    health_general: u16,
    pace: OtDeluxePace,
    rations: OtDeluxeRations,
    weather: Weather,
    occupation: Option<OtDeluxeOccupation>,
}

const fn otdeluxe_health_label(
    health_general: u16,
    policy: &OtDeluxeHealthPolicy,
) -> OtDeluxeHealthLabel {
    if health_general <= policy.label_ranges.good_max {
        OtDeluxeHealthLabel::Good
    } else if health_general <= policy.label_ranges.fair_max {
        OtDeluxeHealthLabel::Fair
    } else if health_general <= policy.label_ranges.poor_max {
        OtDeluxeHealthLabel::Poor
    } else {
        OtDeluxeHealthLabel::VeryPoor
    }
}

const fn sanitize_disease_multiplier(mult: f32) -> f32 {
    if mult.is_finite() { mult.max(0.0) } else { 1.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtDeluxeSparePart {
    Wheel,
    Axle,
    Tongue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtDeluxeNavigationEvent {
    LostTrail,
    WrongTrail,
    Impassable,
    Snowbound,
}

const fn otdeluxe_spare_for_breakdown(part: Part) -> OtDeluxeSparePart {
    match part {
        Part::Battery => OtDeluxeSparePart::Axle,
        Part::Alternator => OtDeluxeSparePart::Tongue,
        Part::Tire | Part::FuelPump => OtDeluxeSparePart::Wheel,
    }
}

const fn otdeluxe_navigation_reason_tag(event: OtDeluxeNavigationEvent) -> &'static str {
    match event {
        OtDeluxeNavigationEvent::LostTrail => "otdeluxe.nav_lost",
        OtDeluxeNavigationEvent::WrongTrail => "otdeluxe.nav_wrong",
        OtDeluxeNavigationEvent::Impassable => "otdeluxe.nav_impassable",
        OtDeluxeNavigationEvent::Snowbound => "otdeluxe.nav_snowbound",
    }
}

const fn otdeluxe_navigation_delay_tag(blocked: bool) -> &'static str {
    if blocked {
        "otdeluxe.nav_blocked"
    } else {
        "otdeluxe.nav_delay"
    }
}

const fn otdeluxe_navigation_event_id(event: OtDeluxeNavigationEvent) -> &'static str {
    match event {
        OtDeluxeNavigationEvent::LostTrail => "lost_trail",
        OtDeluxeNavigationEvent::WrongTrail => "wrong_trail",
        OtDeluxeNavigationEvent::Impassable => "impassable",
        OtDeluxeNavigationEvent::Snowbound => "snowbound",
    }
}

const fn otdeluxe_navigation_is_blocked(event: OtDeluxeNavigationEvent) -> bool {
    matches!(
        event,
        OtDeluxeNavigationEvent::Impassable | OtDeluxeNavigationEvent::Snowbound
    )
}

const fn otdeluxe_navigation_delay_for(
    event: OtDeluxeNavigationEvent,
    policy: &OtDeluxeNavigationPolicy,
) -> OtDeluxeNavigationDelay {
    match event {
        OtDeluxeNavigationEvent::LostTrail => policy.lost_delay,
        OtDeluxeNavigationEvent::WrongTrail => policy.wrong_delay,
        OtDeluxeNavigationEvent::Impassable => policy.impassable_delay,
        OtDeluxeNavigationEvent::Snowbound => policy.snowbound_delay,
    }
}

fn sanitize_event_weight_mult(weight_mult: f32) -> f32 {
    if weight_mult.is_finite() && weight_mult >= 0.0 {
        weight_mult
    } else {
        1.0
    }
}

fn roll_otdeluxe_navigation_delay_days<R: Rng>(delay: OtDeluxeNavigationDelay, rng: &mut R) -> u8 {
    if delay.max_days == 0 {
        return 0;
    }
    let min_days = delay.min_days.min(delay.max_days);
    let max_days = delay.max_days.max(delay.min_days);
    rng.gen_range(min_days..=max_days)
}

fn roll_otdeluxe_navigation_event_with_trace<R: Rng>(
    policy: &OtDeluxeNavigationPolicy,
    snow_depth: f32,
    rng: &mut R,
) -> (Option<OtDeluxeNavigationEvent>, Option<EventDecisionTrace>) {
    let chance = policy.chance_per_day.clamp(0.0, 1.0);
    if chance <= 0.0 {
        return (None, None);
    }
    if rng.r#gen::<f32>() >= chance {
        return (None, None);
    }

    let snow_weight = if snow_depth >= policy.snowbound_min_depth_in {
        policy.snowbound_weight
    } else {
        0
    };
    let lost_weight = u32::from(policy.lost_weight);
    let wrong_weight = u32::from(policy.wrong_weight);
    let impassable_weight = u32::from(policy.impassable_weight);
    let lost = (OtDeluxeNavigationEvent::LostTrail, lost_weight);
    let wrong = (OtDeluxeNavigationEvent::WrongTrail, wrong_weight);
    let impassable = (OtDeluxeNavigationEvent::Impassable, impassable_weight);
    let snowbound = (OtDeluxeNavigationEvent::Snowbound, u32::from(snow_weight));
    let options = [lost, wrong, impassable, snowbound];
    let total_weight: u32 = options.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return (None, None);
    }
    let roll = rng.gen_range(0..total_weight);
    let mut current = 0_u32;
    let mut selected = None;
    for (event, weight) in &options {
        current = current.saturating_add(*weight);
        if selected.is_none() && roll < current {
            selected = Some(*event);
        }
    }
    let selected = selected.or_else(|| options.first().map(|(event, _)| *event));
    let total_weight_f64 = f64::from(total_weight);
    let mut candidates = Vec::new();
    for (event, weight) in &options {
        candidates.push(WeightedCandidate {
            id: otdeluxe_navigation_event_id(*event).to_string(),
            base_weight: f64::from(*weight),
            multipliers: Vec::new(),
            final_weight: f64::from(*weight) / total_weight_f64,
        });
    }
    let trace = selected.map(|event| EventDecisionTrace {
        pool_id: String::from("otdeluxe.navigation"),
        roll: RollValue::U32(roll),
        candidates,
        chosen_id: otdeluxe_navigation_event_id(event).to_string(),
    });
    (selected, trace)
}

fn apply_otdeluxe_disease_effects(
    health_general: &mut u16,
    inventory: &mut OtDeluxeInventory,
    effects: &DiseaseEffects,
) -> f32 {
    if effects.health_general_delta != 0 {
        let current = i32::from(*health_general);
        let next = (current + effects.health_general_delta).max(0);
        *health_general = u16::try_from(next).unwrap_or(u16::MAX);
    }
    if effects.food_lbs_delta != 0 {
        let current = i32::from(inventory.food_lbs);
        let next = (current + effects.food_lbs_delta).max(0);
        inventory.food_lbs = u16::try_from(next).unwrap_or(u16::MAX);
    }
    sanitize_disease_multiplier(effects.travel_speed_mult)
}

fn otdeluxe_fatality_probability(
    model: &FatalityModel,
    context: OtDeluxeFatalityContext,
    policy: &OtDeluxe90sPolicy,
) -> f32 {
    let mut prob = model.base_prob_per_day.max(0.0);
    for modifier in &model.prob_modifiers {
        let mult = match modifier {
            FatalityModifier::Constant { mult } => *mult,
            FatalityModifier::HealthLabel {
                good,
                fair,
                poor,
                very_poor,
            } => match otdeluxe_health_label(context.health_general, &policy.health) {
                OtDeluxeHealthLabel::Good => *good,
                OtDeluxeHealthLabel::Fair => *fair,
                OtDeluxeHealthLabel::Poor => *poor,
                OtDeluxeHealthLabel::VeryPoor => *very_poor,
            },
            FatalityModifier::Pace {
                steady,
                strenuous,
                grueling,
            } => match context.pace {
                OtDeluxePace::Steady => *steady,
                OtDeluxePace::Strenuous => *strenuous,
                OtDeluxePace::Grueling => *grueling,
            },
            FatalityModifier::Rations {
                filling,
                meager,
                bare_bones,
            } => match context.rations {
                OtDeluxeRations::Filling => *filling,
                OtDeluxeRations::Meager => *meager,
                OtDeluxeRations::BareBones => *bare_bones,
            },
            FatalityModifier::Weather { weather: key, mult } => {
                if *key == context.weather {
                    *mult
                } else {
                    1.0
                }
            }
        };
        prob *= sanitize_disease_multiplier(mult);
    }
    if model.apply_doctor_mult && matches!(context.occupation, Some(OtDeluxeOccupation::Doctor)) {
        prob *= sanitize_disease_multiplier(policy.occupation_advantages.doctor_fatality_mult);
    }
    if prob.is_finite() {
        prob.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn otdeluxe_roll_disease_fatality<R>(
    model: &FatalityModel,
    rng: &mut R,
    context: OtDeluxeFatalityContext,
    policy: &OtDeluxe90sPolicy,
) -> bool
where
    R: rand::Rng + ?Sized,
{
    let prob = otdeluxe_fatality_probability(model, context, policy);
    prob > 0.0 && rng.r#gen::<f32>() < prob
}

const fn otdeluxe_mobility_failure_mult(
    occupation: Option<OtDeluxeOccupation>,
    policy: &OtDeluxe90sPolicy,
) -> f32 {
    if matches!(occupation, Some(OtDeluxeOccupation::Farmer)) {
        sanitize_disease_multiplier(policy.occupation_advantages.mobility_failure_mult)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        CLASSIC_BALANCED_TRAVEL_NUDGE, CROSSING_MILESTONES, DEBUG_ENV_VAR,
        DEEP_BALANCED_TRAVEL_NUDGE, EXEC_ORDER_DAILY_CHANCE, LOG_CROSSING_DECISION_PERMIT,
        LOG_CROSSING_DETOUR, LOG_CROSSING_FAILURE, LOG_CROSSING_PASSED, LOG_TRAVEL_BLOCKED,
        LOG_TRAVEL_DELAY_CREDIT, LOG_TRAVEL_PARTIAL, LOG_TRAVELED, PERMIT_REQUIRED_TAGS,
    };
    use crate::crossings::{
        CrossingChoice, CrossingConfig, CrossingContext, CrossingKind, CrossingOutcome,
        CrossingResult,
    };
    use crate::data::{Choice, Effects, Encounter, EncounterData};
    use crate::disease::{
        DiseaseCatalog, DiseaseDef, DiseaseEffects, DiseaseKind, FatalityModel, FatalityModifier,
    };
    use crate::endgame::EndgameTravelCfg;
    use crate::journey::{
        CountingRng, CrossingPolicy, DailyTickKernel, DayOutcome, DetourPolicy, JourneyCfg,
        RngBundle, StrainConfig, StrainLabelBounds, StrainWeights,
    };
    use crate::mechanics::otdeluxe90s::{
        OtDeluxe90sPolicy, OtDeluxeAfflictionCurvePoint, OtDeluxeAfflictionPolicy,
        OtDeluxeAfflictionWeightOverride, OtDeluxeNavigationDelay, OtDeluxeNavigationPolicy,
        OtDeluxeOccupation, OtDeluxePace, OtDeluxePolicyOverride, OtDeluxeRations,
    };
    use crate::otdeluxe_crossings::{OtDeluxeCrossingOutcome, OtDeluxeCrossingResolution};
    use crate::otdeluxe_random_events::OtDeluxeRandomEventSelection;
    use crate::otdeluxe_state::{
        OtDeluxeCrossingMethod, OtDeluxeCrossingState, OtDeluxeDallesChoice, OtDeluxeInventory,
        OtDeluxeOxenState, OtDeluxePartyMember, OtDeluxePartyState, OtDeluxeRiver,
        OtDeluxeRiverBed, OtDeluxeRiverState, OtDeluxeRouteDecision, OtDeluxeRoutePrompt,
        OtDeluxeRouteState, OtDeluxeState, OtDeluxeWagonState,
    };
    use crate::pacing::{PaceCfg, PacingLimits};
    use crate::personas::{Persona, PersonaMods, PersonaStart};
    use crate::store::Grants;
    use crate::weather::Weather;
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rand::rngs::mock::StepRng;
    use std::cell::RefMut;
    use std::collections::{HashMap, VecDeque};
    use std::rc::Rc;
    use std::sync::{Mutex, OnceLock};

    fn bundle_with_roll_below(
        threshold: f32,
        domain: fn(&RngBundle) -> RefMut<'_, CountingRng<SmallRng>>,
    ) -> Rc<RngBundle> {
        for seed in 0..10_000 {
            let probe = RngBundle::from_user_seed(seed);
            {
                let mut rng = domain(&probe);
                if rng.r#gen::<f32>() < threshold {
                    return Rc::new(RngBundle::from_user_seed(seed));
                }
            }
        }
        panic!("unable to find deterministic seed below {threshold}");
    }

    fn encounter_with_choice(effects: Effects) -> Encounter {
        Encounter {
            id: String::from("choice-test"),
            name: String::from("Test Encounter"),
            desc: String::new(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: String::from("Pick"),
                effects,
            }],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }
    }

    fn make_event_selection(event_id: &str, variant: &str) -> OtDeluxeRandomEventSelection {
        OtDeluxeRandomEventSelection {
            event_id: event_id.to_string(),
            variant_id: Some(variant.to_string()),
            chance_roll: 0.2,
            chance_threshold: 0.3,
        }
    }

    fn otdeluxe_state_with_party() -> GameState {
        GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ot_deluxe: OtDeluxeState {
                party: OtDeluxePartyState::from_names(["Leader"]),
                inventory: OtDeluxeInventory {
                    food_lbs: 200,
                    bullets: 60,
                    clothes_sets: 4,
                    spares_wheels: 1,
                    spares_axles: 1,
                    spares_tongues: 1,
                    ..OtDeluxeInventory::default()
                },
                oxen: OtDeluxeOxenState {
                    healthy: 2,
                    sick: 1,
                },
                ..OtDeluxeState::default()
            },
            ..GameState::default()
        }
    }

    #[test]
    fn policy_kind_str_and_parse_cover_variants() {
        assert_eq!(PolicyKind::Balanced.as_str(), "balanced");
        assert_eq!(PolicyKind::Conservative.as_str(), "conservative");
        assert_eq!(PolicyKind::Aggressive.as_str(), "aggressive");
        assert_eq!(PolicyKind::ResourceManager.as_str(), "resource_manager");
        assert_eq!(PolicyKind::Balanced.to_string(), "balanced");
        assert_eq!(
            "conservative".parse::<PolicyKind>().unwrap(),
            PolicyKind::Conservative
        );
        assert!("unknown".parse::<PolicyKind>().is_err());
    }

    #[test]
    fn otdeluxe_starting_cash_tracks_policy_table() {
        let policy = OtDeluxe90sPolicy::default();
        let occupation = OtDeluxeOccupation::Banker;
        let expected = policy
            .occupations
            .iter()
            .find(|spec| spec.occupation == occupation)
            .map_or(0, |spec| spec.starting_cash_dollars);
        assert_eq!(
            otdeluxe_starting_cash_cents(occupation, &policy),
            u32::from(expected).saturating_mul(100)
        );
    }

    #[test]
    fn otdeluxe_affliction_probability_handles_curve_edges() {
        let policy = OtDeluxeAfflictionPolicy {
            probability_max: 0.4,
            curve_pwl: [
                OtDeluxeAfflictionCurvePoint {
                    health: 20,
                    probability: 0.3,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 20,
                    probability: 0.2,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 40,
                    probability: 0.1,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 60,
                    probability: 0.05,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 80,
                    probability: 0.02,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 90,
                    probability: 0.01,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 95,
                    probability: 0.005,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 98,
                    probability: 0.002,
                },
                OtDeluxeAfflictionCurvePoint {
                    health: 100,
                    probability: 0.001,
                },
            ],
            weight_illness: 1,
            weight_injury: 1,
            illness_duration_days: 1,
            injury_duration_days: 1,
        };

        let low = otdeluxe_affliction_probability(10, &policy);
        assert!((low - 0.3).abs() < 1e-6);

        let zero_span = otdeluxe_affliction_probability(20, &policy);
        assert!((zero_span - 0.3).abs() < 1e-6);

        let mid = otdeluxe_affliction_probability(30, &policy);
        assert!((mid - 0.15).abs() < 1e-6);

        let high = otdeluxe_affliction_probability(150, &policy);
        assert!((high - 0.001).abs() < 1e-6);
    }

    #[test]
    fn roll_otdeluxe_affliction_kind_handles_zero_weights() {
        let policy = OtDeluxe90sPolicy::default();
        let overrides = OtDeluxePolicyOverride {
            affliction_weights: OtDeluxeAfflictionWeightOverride {
                illness: Some(0),
                injury: Some(0),
            },
            ..OtDeluxePolicyOverride::default()
        };
        let mut rng = SmallRng::seed_from_u64(3);
        let (kind, trace) = roll_otdeluxe_affliction_kind(&policy.affliction, &overrides, &mut rng);
        assert_eq!(kind, OtDeluxeAfflictionKind::Illness);
        assert!(trace.is_none());
    }

    #[test]
    fn roll_otdeluxe_affliction_kind_prefers_override_weights() {
        let policy = OtDeluxe90sPolicy::default();
        let overrides = OtDeluxePolicyOverride {
            affliction_weights: OtDeluxeAfflictionWeightOverride {
                illness: Some(0),
                injury: Some(5),
            },
            ..OtDeluxePolicyOverride::default()
        };
        let mut rng = SmallRng::seed_from_u64(11);
        let (kind, trace) = roll_otdeluxe_affliction_kind(&policy.affliction, &overrides, &mut rng);
        assert_eq!(kind, OtDeluxeAfflictionKind::Injury);
        assert!(trace.is_some());
    }

    #[test]
    fn roll_otdeluxe_navigation_event_with_trace_emits_candidates() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 1,
            wrong_weight: 1,
            impassable_weight: 1,
            snowbound_weight: 1,
            snowbound_min_depth_in: 0.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = SmallRng::seed_from_u64(12);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 10.0, &mut rng);
        assert!(event.is_some());
        let trace = trace.expect("trace");
        assert_eq!(trace.candidates.len(), 4);
    }

    #[test]
    fn otdeluxe_fatality_probability_returns_zero_for_non_finite() {
        let policy = OtDeluxe90sPolicy::default();
        let model = FatalityModel {
            base_prob_per_day: f32::INFINITY,
            prob_modifiers: Vec::new(),
            apply_doctor_mult: false,
        };
        let context = OtDeluxeFatalityContext {
            health_general: 100,
            pace: OtDeluxePace::Steady,
            rations: OtDeluxeRations::Filling,
            weather: Weather::Clear,
            occupation: None,
        };
        let prob = otdeluxe_fatality_probability(&model, context, &policy);
        assert!(prob.abs() <= f32::EPSILON);
    }

    #[test]
    fn apply_otdeluxe_random_affliction_selects_member() {
        let catalog = DiseaseCatalog {
            diseases: vec![DiseaseDef {
                id: "illness_1".into(),
                kind: DiseaseKind::Illness,
                display_key: "disease.illness_1".into(),
                weight: 1,
                duration_days: Some(2),
                onset_effects: DiseaseEffects::default(),
                daily_tick_effects: DiseaseEffects::default(),
                fatality_model: None,
                tags: Vec::new(),
            }],
        };
        let mut state = GameState::default();
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        let mut rng = SmallRng::seed_from_u64(13);
        let outcome = state.apply_otdeluxe_random_affliction_with_catalog(
            &catalog,
            &mut rng,
            OtDeluxeAfflictionKind::Illness,
        );
        assert!(outcome.is_some());
    }

    #[test]
    fn apply_otdeluxe_random_event_affliction_branches_cover_variants() {
        let mut state = GameState::default();
        state.ot_deluxe.party.members = vec![OtDeluxePartyMember::new("Ada")];
        let mut rng = SmallRng::seed_from_u64(14);
        let shortage =
            state.apply_otdeluxe_random_resource_shortage("bad_water", 0.1, 0.2, &mut rng);
        assert!(shortage.is_some());
        let grass = state.apply_otdeluxe_random_resource_shortage("no_grass", 0.1, 0.2, &mut rng);
        assert!(grass.is_some());
        let incident = state.apply_otdeluxe_random_party_incident("snakebite", 0.1, 0.2, &mut rng);
        assert!(incident.is_some());
        let missing = state.apply_otdeluxe_random_party_incident("unknown", 0.1, 0.2, &mut rng);
        assert!(missing.is_none());
    }

    fn events_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::events)
    }

    fn health_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::health)
    }

    fn breakdown_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::breakdown)
    }

    fn bundle_with_roll_at_or_above(
        threshold: f32,
        domain: fn(&RngBundle) -> RefMut<'_, CountingRng<SmallRng>>,
    ) -> Rc<RngBundle> {
        for seed in 0..10_000 {
            let probe = RngBundle::from_user_seed(seed);
            {
                let mut rng = domain(&probe);
                if rng.r#gen::<f32>() >= threshold {
                    return Rc::new(RngBundle::from_user_seed(seed));
                }
            }
        }
        panic!("unable to find deterministic seed at or above {threshold}");
    }

    fn breakdown_bundle_with_roll_at_or_above(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_at_or_above(threshold, RngBundle::breakdown)
    }

    fn seed_for_roll_below(threshold: f32) -> u64 {
        for seed in 0..10_000 {
            let mut rng = SmallRng::seed_from_u64(seed);
            if rng.r#gen::<f32>() < threshold {
                return seed;
            }
        }
        panic!("unable to find deterministic seed below {threshold}");
    }

    fn seed_for_roll_at_or_above(threshold: f32) -> u64 {
        for seed in 0..10_000 {
            let mut rng = SmallRng::seed_from_u64(seed);
            if rng.r#gen::<f32>() >= threshold {
                return seed;
            }
        }
        panic!("unable to find deterministic seed at or above {threshold}");
    }

    fn with_debug_env<F, T>(f: F) -> T
    where
        F: FnOnce() -> T,
    {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = std::env::var(DEBUG_ENV_VAR).ok();
        unsafe {
            std::env::set_var(DEBUG_ENV_VAR, "1");
        }
        let result = f();
        match previous {
            Some(value) => unsafe {
                std::env::set_var(DEBUG_ENV_VAR, value);
            },
            None => unsafe {
                std::env::remove_var(DEBUG_ENV_VAR);
            },
        }
        result
    }

    fn approx_eq(a: f32, b: f32) {
        let epsilon = 1e-5_f32;
        assert!(
            (a - b).abs() <= epsilon,
            "values differ: {a} vs {b} (ε={epsilon})"
        );
    }

    #[test]
    fn ledger_records_capture_tags_and_counts() {
        let mut state = GameState::default();
        state.features.travel_v2 = true;
        state.start_of_day();
        state.record_travel_day(TravelDayKind::Partial, 4.0, "repair");
        state.end_of_day();

        assert_eq!(state.day_records.len(), 1);
        let record = &state.day_records[0];
        assert_eq!(record.kind, TravelDayKind::Partial);
        assert!(record.tags.iter().any(|tag| tag.0 == "repair"));
        assert_eq!(state.travel_days, 0);
        assert_eq!(state.partial_travel_days, 1);
        assert_eq!(state.non_travel_days, 0);
    }

    #[test]
    fn ledger_serializes_and_roundtrips() {
        let mut state = GameState::default();
        let schedule = [
            (TravelDayKind::Travel, 11.0_f32, "travel"),
            (TravelDayKind::Partial, 5.0_f32, "detour"),
            (TravelDayKind::NonTravel, 0.0_f32, "camp"),
        ];

        for (kind, miles, tag) in schedule {
            state.start_of_day();
            state.record_travel_day(kind, miles, tag);
            state.end_of_day();
        }

        let json = serde_json::to_string(&state).expect("serialize");
        let restored: GameState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.day_records, state.day_records);
        assert_eq!(restored.travel_days, state.travel_days);
        assert_eq!(restored.partial_travel_days, state.partial_travel_days);
        assert_eq!(restored.non_travel_days, state.non_travel_days);
    }

    #[test]
    fn travel_wear_scales_with_pace_weather_and_fatigue() {
        let mut state = GameState::default();
        state.journey_wear.base = 1.0;
        state.journey_wear.fatigue_k = 0.5;
        state.journey_wear.comfort_miles = 0.0;
        state.journey_breakdown.pace_factor =
            HashMap::from([(PaceId::Steady, 1.0), (PaceId::Blitz, 2.0)]);
        state.journey_breakdown.weather_factor =
            HashMap::from([(Weather::Clear, 1.0), (Weather::Storm, 1.5)]);

        state.vehicle.wear = 0.0;
        state.vehicle.health = Vehicle::default().health;
        state.pace = PaceId::Steady;
        state.weather_state.today = Weather::Clear;
        state.miles_traveled_actual = 0.0;
        state.current_day_miles = 10.0;
        state.distance_today = 10.0;
        state.distance_today_raw = 10.0;
        state.apply_travel_wear_for_day(10.0);
        let steady_clear = state.vehicle.wear;

        state.vehicle.wear = 0.0;
        state.vehicle.health = Vehicle::default().health;
        state.pace = PaceId::Blitz;
        state.weather_state.today = Weather::Storm;
        state.miles_traveled_actual = 800.0;
        state.current_day_miles = 10.0;
        state.distance_today = 10.0;
        state.distance_today_raw = 10.0;
        state.apply_travel_wear_for_day(10.0);
        let blitz_storm = state.vehicle.wear;

        assert!(blitz_storm > steady_clear);
    }

    #[test]
    fn balanced_strategy_applies_travel_nudge_by_mode() {
        let mut classic = GameState {
            policy: Some(PolicyKind::Balanced),
            journey_travel: TravelConfig {
                mpd_base: 10.0,
                mpd_min: 1.0,
                mpd_max: 20.0,
                pace_factor: HashMap::from([
                    (PaceId::Steady, 1.0),
                    (PaceId::Heated, 1.0),
                    (PaceId::Blitz, 1.0),
                ]),
                weather_factor: HashMap::from([(Weather::Clear, 1.0), (Weather::Storm, 1.0)]),
            },
            ..GameState::default()
        };

        let pace_cfg = PaceCfg {
            dist_mult: 1.0,
            ..PaceCfg::default()
        };
        let limits = PacingLimits::default();

        let mut control = classic.clone();
        control.policy = Some(PolicyKind::Aggressive);
        let base = control.compute_miles_for_today(&pace_cfg, &limits);
        let nudged = classic.compute_miles_for_today(&pace_cfg, &limits);
        approx_eq(nudged, base * CLASSIC_BALANCED_TRAVEL_NUDGE);

        let mut deep = classic.clone();
        deep.mode = GameMode::Deep;
        deep.policy = Some(PolicyKind::Balanced);
        let mut deep_control = deep.clone();
        deep_control.policy = Some(PolicyKind::ResourceManager);
        let deep_base = deep_control.compute_miles_for_today(&pace_cfg, &limits);
        let deep_nudged = deep.compute_miles_for_today(&pace_cfg, &limits);
        approx_eq(deep_nudged, deep_base * DEEP_BALANCED_TRAVEL_NUDGE);
    }

    #[test]
    fn deep_aggressive_compose_uses_supplies_then_funds() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            stats: Stats {
                supplies: BOSS_COMPOSE_SUPPLY_COST,
                sanity: 0,
                pants: 5,
                ..Stats::default()
            },
            budget_cents: BOSS_COMPOSE_FUNDS_COST * 2,
            ..GameState::default()
        };

        let applied_supplies = state.apply_deep_aggressive_compose();
        assert!(applied_supplies, "expected supply-based compose");
        assert_eq!(state.stats.supplies, 0);
        assert_eq!(state.stats.sanity, 1);
        assert!(state.stats.pants < 5);
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == LOG_BOSS_COMPOSE_SUPPLIES)
        );
        assert!(state.logs.iter().any(|log| log == LOG_BOSS_COMPOSE));

        state.logs.clear();
        state.stats.supplies = 0;
        state.stats.sanity = 0;
        state.stats.pants = 5;
        let baseline_budget = state.budget_cents;
        state.budget = i32::try_from(state.budget_cents / 100).unwrap_or(0);

        let applied_funds = state.apply_deep_aggressive_compose();
        assert!(applied_funds, "expected funds-based compose");
        assert!(state.budget_cents < baseline_budget);
        assert_eq!(state.stats.sanity, 1);
        assert!(state.stats.pants < 5);
        assert!(state.logs.iter().any(|log| log == LOG_BOSS_COMPOSE_FUNDS));
        assert!(state.logs.iter().any(|log| log == LOG_BOSS_COMPOSE));
    }

    #[test]
    fn breakdown_uses_part_weights() {
        let mut state = GameState::default();
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.2));
        state.journey_breakdown.base = 1.0;
        state.journey_breakdown.beta = 0.0;
        state.journey_part_weights = PartWeights {
            tire: 0,
            battery: 100,
            alt: 0,
            pump: 0,
        };
        let triggered = state.vehicle_roll();
        assert!(triggered);
        assert_eq!(state.last_breakdown_part, Some(Part::Battery));
        let trace = state
            .decision_traces_today
            .iter()
            .find(|trace| trace.pool_id == "dystrail.breakdown_part")
            .expect("breakdown trace recorded");
        assert_eq!(trace.chosen_id, Part::Battery.key());
    }

    fn endgame_cfg() -> EndgameTravelCfg {
        EndgameTravelCfg::default()
    }

    fn tick_day(state: &mut GameState, endgame_cfg: &EndgameTravelCfg) -> DayOutcome {
        let cfg = JourneyCfg::default();
        let kernel = DailyTickKernel::new(&cfg, endgame_cfg);
        kernel.tick_day(state)
    }

    fn tick_day_with_hook<F>(
        state: &mut GameState,
        endgame_cfg: &EndgameTravelCfg,
        hook: F,
    ) -> DayOutcome
    where
        F: FnOnce(&mut GameState),
    {
        let cfg = JourneyCfg::default();
        let kernel = DailyTickKernel::new(&cfg, endgame_cfg);
        kernel.tick_day_with_hook(state, hook)
    }

    #[test]
    fn breakdown_consumes_spare_and_clears_block() {
        let mut state = GameState {
            inventory: Inventory {
                spares: Spares {
                    tire: 1,
                    ..Spares::default()
                },
                ..Inventory::default()
            },
            breakdown: Some(Breakdown {
                part: Part::Tire,
                day_started: 1,
            }),
            day_state: DayState {
                travel: TravelDayState {
                    travel_blocked: true,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            data: Some(EncounterData::empty()),
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(1)));

        let cfg = endgame_cfg();
        let _ = tick_day(&mut state, &cfg);

        assert_eq!(state.inventory.spares.tire, 0);
        assert!(!state.day_state.travel.travel_blocked);
        assert!(state.breakdown.is_none());
    }

    #[test]
    fn breakdown_without_spare_resolves_after_stall() {
        let mut state = GameState {
            breakdown: Some(Breakdown {
                part: Part::Battery,
                day_started: 1,
            }),
            day_state: DayState {
                travel: TravelDayState {
                    travel_blocked: true,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            data: Some(EncounterData::empty()),
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(2)));

        let cfg = endgame_cfg();
        let outcome = tick_day(&mut state, &cfg);
        assert_eq!(outcome.log_key, "log.traveled");
        assert!(!state.day_state.travel.travel_blocked);
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
    fn otdeluxe_breakdown_consumes_ot_spare_and_unblocks() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ot_deluxe: OtDeluxeState {
                inventory: OtDeluxeInventory {
                    spares_wheels: 1,
                    ..OtDeluxeInventory::default()
                },
                travel: OtDeluxeTravelState {
                    wagon_state: OtDeluxeWagonState::Blocked,
                    ..OtDeluxeTravelState::default()
                },
                ..OtDeluxeState::default()
            },
            breakdown: Some(Breakdown {
                part: Part::Tire,
                day_started: 1,
            }),
            day_state: DayState {
                travel: TravelDayState {
                    travel_blocked: true,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            ..GameState::default()
        };

        state.resolve_breakdown();

        assert_eq!(state.ot_deluxe.inventory.spares_wheels, 0);
        assert!(!state.day_state.travel.travel_blocked);
        assert!(state.breakdown.is_none());
        assert_eq!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Moving
        );
    }

    #[test]
    fn otdeluxe_breakdown_without_spare_stays_blocked() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ot_deluxe: OtDeluxeState {
                travel: OtDeluxeTravelState {
                    wagon_state: OtDeluxeWagonState::Moving,
                    ..OtDeluxeTravelState::default()
                },
                ..OtDeluxeState::default()
            },
            breakdown: Some(Breakdown {
                part: Part::Battery,
                day_started: 1,
            }),
            day_state: DayState {
                travel: TravelDayState {
                    travel_blocked: false,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            ..GameState::default()
        };

        state.resolve_breakdown();

        assert!(state.day_state.travel.travel_blocked);
        assert!(state.breakdown.is_some());
        assert_eq!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Blocked
        );
    }

    #[test]
    fn exec_order_drain_clamped_to_zero() {
        let mut state = GameState {
            stats: Stats {
                supplies: 0,
                sanity: 0,
                ..Stats::default()
            },
            encounter_chance_today: 0.0,
            data: Some(EncounterData::empty()),
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(3)));

        let cfg = endgame_cfg();
        let _ = tick_day(&mut state, &cfg);

        assert!(state.stats.supplies >= 0, "supplies went negative");
        assert!(state.stats.sanity >= 0, "sanity went negative");
    }

    #[test]
    fn exec_order_expires_and_sets_cooldown() {
        let mut state = GameState {
            current_order: Some(ExecOrder::Shutdown),
            exec_order_days_remaining: 1,
            exec_order_cooldown: 0,
            ..GameState::default()
        };
        state.detach_rng_bundle();
        let supplies_before = state.stats.supplies;
        let morale_before = state.stats.morale;

        let _ = crate::journey::tick_non_travel_day_for_state(
            &mut state,
            TravelDayKind::NonTravel,
            0.0,
            "test",
        );

        assert!(state.current_order.is_none());
        assert_eq!(state.exec_order_cooldown, EXEC_ORDER_MIN_COOLDOWN);
        let end_log = format!("{}{}", LOG_EXEC_END_PREFIX, ExecOrder::Shutdown.key());
        assert!(state.logs.iter().any(|entry| entry == &end_log));
        assert!(state.stats.supplies < supplies_before);
        assert!(state.stats.morale < morale_before);
    }

    #[test]
    fn exec_order_selection_records_decision_trace() {
        let mut state = GameState::default();
        state.attach_rng_bundle(events_bundle_with_roll_below(EXEC_ORDER_DAILY_CHANCE));

        state.tick_exec_order_state();

        assert!(state.current_order.is_some());
        let trace = state
            .decision_traces_today
            .iter()
            .find(|trace| trace.pool_id == "dystrail.exec_order")
            .expect("exec order trace recorded");
        let chosen = state.current_order.expect("expected current exec order");
        assert_eq!(trace.chosen_id, chosen.key());
    }

    #[test]
    fn starvation_stacks_damage() {
        let mut state = GameState {
            stats: Stats {
                supplies: 0,
                ..Stats::default()
            },
            ..GameState::default()
        };

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
        let mut state = GameState {
            vehicle_breakdowns: 10,
            vehicle: Vehicle {
                health: 0.0,
                ..Vehicle::default()
            },
            inventory: Inventory {
                spares: Spares::default(),
                ..Inventory::default()
            },
            budget_cents: 0,
            ..GameState::default()
        };
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
        let mut state = GameState {
            stats: Stats {
                supplies: 0,
                hp: 1,
                ..Stats::default()
            },
            ..GameState::default()
        };
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
        let mut state = GameState {
            stats: Stats {
                supplies: 10,
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::ExposureCold),
            ..GameState::default()
        };
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
        let mut state = GameState {
            pace: PaceId::Steady,
            ..GameState::default()
        };
        state.detach_rng_bundle();
        let cfg = endgame_cfg();
        for _ in 0..30 {
            state.weather_state.today = Weather::Clear;
            state.weather_state.yesterday = Weather::Clear;
            let outcome = tick_day_with_hook(&mut state, &cfg, |state| {
                state.encounter_chance_today = 0.0;
            });
            assert!(!outcome.ended, "run ended prematurely");
        }
        assert!(
            state.travel_days + state.partial_travel_days >= 30,
            "expected at least 30 days with travel credit"
        );
        let moving_days = state.travel_days.saturating_add(state.partial_travel_days);
        let avg_mpd = if moving_days > 0 {
            f64::from(state.miles_traveled_actual) / f64::from(moving_days)
        } else {
            0.0
        };
        assert!(
            avg_mpd >= ASSERT_MIN_AVG_MPD,
            "average miles per day {avg_mpd:.2}"
        );
    }

    #[test]
    fn no_miles_on_camp() {
        let mut state = GameState::default();
        state.detach_rng_bundle();
        for _ in 0..5 {
            state.advance_days(1);
        }
        assert!(state.miles_traveled_actual.abs() <= f32::EPSILON);
        assert_eq!(state.travel_days, 0);
        assert_eq!(state.non_travel_days, 5);
    }

    #[test]
    fn encounter_soft_cap_reduces_chance() {
        let cfg = crate::pacing::PacingConfig::default_config();

        let mut base_state = GameState::default();
        base_state.detach_rng_bundle();
        base_state.start_of_day();
        base_state.apply_pace_and_diet(&cfg);
        let base = base_state.encounter_chance_today;
        assert!((f64::from(base) - f64::from(ENCOUNTER_BASE_DEFAULT)).abs() < FLOAT_EPSILON);

        let mut capped_state = GameState {
            encounter_history: VecDeque::from(vec![2, 1, 1, 1, 0, 0, 0, 0, 0]),
            ..GameState::default()
        };
        capped_state.detach_rng_bundle();
        capped_state.start_of_day();
        capped_state.apply_pace_and_diet(&cfg);
        let capped = capped_state.encounter_chance_today;
        assert!(
            f64::from(base)
                .mul_add(-f64::from(TRAVEL_PARTIAL_RATIO), f64::from(capped))
                .abs()
                < FLOAT_EPSILON,
            "expected soft cap to halve encounter chance (base {base}, capped {capped})"
        );
    }

    #[test]
    fn misc_state_path_exercise() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            features: FeatureFlags {
                travel_v2: true,
                ..FeatureFlags::default()
            },
            stats: Stats {
                supplies: 5,
                pants: 20,
                ..Stats::default()
            },
            distance_today: 5.0,
            distance_today_raw: 5.0,
            partial_distance_today: 2.0,
            current_day_reason_tags: ["camp".into(), "repair".into()].into(),
            recent_travel_days: VecDeque::from(vec![
                TravelDayKind::NonTravel;
                TRAVEL_HISTORY_WINDOW
            ]),
            ..GameState::default()
        };
        state.start_of_day();
        state.enforce_aggressive_delay_cap(0.0);
        state.apply_partial_travel_credit(3.0, LOG_TRAVEL_PARTIAL, "misc");
        state.apply_delay_travel_credit("delay_test");
        state.reset_today_progress();

        state.current_order = Some(ExecOrder::TravelBanLite);
        state.exec_order_days_remaining = 1;
        state.start_of_day();
        assert!(state.exec_order_days_remaining <= EXEC_ORDER_MAX_DURATION);

        state.vehicle.set_breakdown_cooldown(2);
        state.vehicle.tick_breakdown_cooldown();
        assert!(state.vehicle.breakdown_suppressed());
        state.vehicle.tick_breakdown_cooldown();
        assert!(!state.vehicle.breakdown_suppressed());

        state.endgame.active = true;
        state.endgame.failure_guard_miles = 1_900.0;
        state.endgame.health_floor = 30.0;
        state.endgame.wear_reset = 5.0;
        state.endgame.cooldown_days = 2;
        state.miles_traveled_actual = 1_850.0;
        state.vehicle.health = 0.0;
        state.vehicle.wear = 80.0;
        assert!(crate::endgame::enforce_failure_guard(&mut state));
    }

    #[test]
    fn max_two_encounters_per_day() {
        let mut state = GameState {
            encounters_today: MAX_ENCOUNTERS_PER_DAY,
            encounter_cooldown: 0,
            encounter_chance_today: 0.0,
            encounters: EncounterState {
                occurred_today: false,
                ..EncounterState::default()
            },
            current_encounter: None,
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(42)));
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
        let end_cfg = endgame_cfg();
        let outcome = tick_day_with_hook(&mut state, &end_cfg, |state| {
            state.encounter_chance_today = 0.0;
            state.encounters_today = MAX_ENCOUNTERS_PER_DAY;
            if let Some(back) = state.encounter_history.back_mut() {
                *back = state.encounters_today;
            }
        });
        assert!(!outcome.ended);
        assert_eq!(outcome.log_key, LOG_TRAVELED);
        assert!(state.current_encounter.is_none());
    }

    #[test]
    fn allows_two_encounters_before_cooldown() {
        let mut state = GameState::default();
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(99)));
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
        let end_cfg = endgame_cfg();
        let outcome = tick_day_with_hook(&mut state, &end_cfg, |state| {
            state.encounter_chance_today = 1.0;
        });
        assert_eq!(outcome.log_key, "log.encounter");
        assert_eq!(state.encounters_today, 1);
        state.apply_choice(0);
        assert!(!state.encounters.occurred_today);

        let outcome = tick_day_with_hook(&mut state, &end_cfg, |state| {
            state.encounter_chance_today = 1.0;
        });
        assert_eq!(outcome.log_key, "log.encounter");
        assert_eq!(state.encounters_today, 2);
        state.apply_choice(0);
        assert!(state.encounters.occurred_today);

        let outcome = tick_day_with_hook(&mut state, &end_cfg, |state| {
            state.encounter_chance_today = 1.0;
        });
        assert_eq!(outcome.log_key, LOG_TRAVELED);
        assert_eq!(
            state.encounter_history.back(),
            Some(&MAX_ENCOUNTERS_PER_DAY)
        );
    }

    #[test]
    fn stop_cap_conversion_awards_partial_credit() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            features: FeatureFlags {
                travel_v2: true,
                ..FeatureFlags::default()
            },
            recent_travel_days: VecDeque::from(vec![
                TravelDayKind::NonTravel;
                AGGRESSIVE_STOP_WINDOW_DAYS
            ]),
            distance_today: 20.0,
            distance_today_raw: 20.0,
            vehicle: Vehicle {
                wear: 5.0,
                ..Vehicle::default()
            },
            ..GameState::default()
        };

        state.start_of_day();
        state.enforce_aggressive_delay_cap(20.0);

        assert!(
            state.day_state.travel.partial_traveled_today,
            "expected partial credit after stop cap"
        );
        assert_eq!(state.current_day_kind, Some(TravelDayKind::Partial));
        assert!(state.distance_today > 0.0);
        assert_eq!(state.days_with_camp, 0);
        assert!(state.vehicle.wear < 5.0);
        assert!(
            state
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "stop_cap")
        );
    }

    #[test]
    fn sanity_guard_marks_partial_day() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            day: DEEP_AGGRESSIVE_SANITY_DAY,
            miles_traveled_actual: DEEP_AGGRESSIVE_SANITY_MILES,
            stats: Stats {
                sanity: 0,
                ..Stats::default()
            },
            budget_cents: DEEP_AGGRESSIVE_SANITY_COST,
            ..GameState::default()
        };

        state.start_of_day();
        state.apply_deep_aggressive_sanity_guard();

        assert!(state.guards.deep_aggressive_sanity_guard_used);
        assert_eq!(state.stats.sanity, SANITY_POINT_REWARD);
        assert_eq!(state.current_day_kind, Some(TravelDayKind::Partial));
        assert!(
            state
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "da_sanity_guard")
        );
    }

    #[test]
    fn illness_rolls_cover_positive_and_cooldown_paths() {
        let mut state = GameState {
            data: Some(EncounterData::empty()),
            illness_days_remaining: 2,
            stats: Stats {
                hp: 10,
                sanity: 10,
                supplies: 6,
                ..Stats::default()
            },
            disease_cooldown: 0,
            ..GameState::default()
        };
        state.attach_rng_bundle(health_bundle_with_roll_below(0.5));
        state.roll_daily_illness();
        assert_eq!(state.illness_days_remaining, 1);
        assert!(state.day_state.rest.rest_requested);

        // Cooldown prevents new illness.
        state.disease_cooldown = 2;
        state.illness_days_remaining = 0;
        state.roll_daily_illness();
        assert_eq!(state.disease_cooldown, 1);
    }

    #[test]
    fn illness_triggers_when_guard_conditions_met() {
        let mut state = GameState {
            data: Some(EncounterData::empty()),
            disease_cooldown: 0,
            starvation_days: 2,
            stats: Stats {
                hp: 3,
                supplies: 0,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(health_bundle_with_roll_below(0.05));

        state.roll_daily_illness();
        assert!(state.illness_days_remaining > 0);
        assert!(state.logs.iter().any(|log| log == LOG_DISEASE_HIT));
    }

    #[test]
    fn illness_roll_consumes_health_rng_only() {
        let mut state = GameState {
            data: Some(EncounterData::empty()),
            disease_cooldown: 0,
            illness_days_remaining: 0,
            stats: Stats {
                hp: 8,
                sanity: 8,
                supplies: 5,
                ..Stats::default()
            },
            ..GameState::default()
        };
        let bundle = Rc::new(RngBundle::from_user_seed(444));
        state.attach_rng_bundle(bundle.clone());

        state.roll_daily_illness();

        assert!(bundle.health().draws() > 0);
        assert_eq!(bundle.weather().draws(), 0);
        assert_eq!(bundle.events().draws(), 0);
    }

    #[test]
    fn ally_attrition_and_exec_order_paths() {
        let mut state = GameState {
            data: Some(EncounterData::empty()),
            stats: Stats {
                allies: 2,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(events_bundle_with_roll_below(ALLY_ATTRITION_CHANCE * 0.5));
        state.tick_ally_attrition();
        assert!(state.stats.allies <= 1);

        // Exec order branch when current order is active and resolves.
        state.current_order = Some(ExecOrder::Shutdown);
        state.exec_order_days_remaining = 1;
        state.exec_order_cooldown = 0;
        state.attach_rng_bundle(events_bundle_with_roll_below(
            EXEC_ORDER_DAILY_CHANCE + 0.05,
        ));
        state.tick_exec_order_state();
        assert!(state.exec_order_cooldown > 0 || state.current_order.is_none());

        // No current order: force issuing a new one via deterministic RNG.
        state.current_order = None;
        state.exec_order_cooldown = 0;
        state.attach_rng_bundle(events_bundle_with_roll_below(
            EXEC_ORDER_DAILY_CHANCE + 0.05,
        ));
        state.tick_exec_order_state();
        assert!(state.current_order.is_some() || !state.logs.is_empty());
    }

    #[test]
    fn exec_order_effects_cover_all_variants() {
        let mut state = GameState::default();
        for &order in ExecOrder::ALL {
            state.exec_effects.travel_multiplier = 10.0;
            state.exec_effects.breakdown_bonus = 10.0;
            state.inventory.tags.clear();
            state.apply_exec_order_effects(order);
        }
    }

    #[test]
    fn travel_ratio_recent_handles_edge_cases() {
        let mut state = GameState::default();
        assert!((state.travel_ratio_recent(0) - 1.0).abs() < f32::EPSILON);
        state.recent_travel_days.clear();
        assert!((state.travel_ratio_recent(5) - WEATHER_DEFAULT_SPEED).abs() < f32::EPSILON);
        state.recent_travel_days.push_back(TravelDayKind::Travel);
        for _ in 0..6 {
            state.recent_travel_days.push_back(TravelDayKind::NonTravel);
        }
        assert!(state.travel_ratio_recent(5) < 1.0);
    }

    #[test]
    fn partial_travel_credit_resets_and_logs() {
        let mut state = GameState {
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: true,
                    partial_traveled_today: false,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            ..GameState::default()
        };
        state.start_of_day();
        state.apply_partial_travel_credit(5.0, "log.partial", "reason");
        assert!(state.logs.iter().any(|log| log == "log.partial"));
    }

    #[test]
    fn rest_travel_credit_logs_when_enabled() {
        let mut state = GameState {
            features: FeatureFlags {
                travel_v2: true,
                ..FeatureFlags::default()
            },
            ..GameState::default()
        };
        state.start_of_day();
        state.apply_rest_travel_credit();
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_REST_CREDIT));
    }

    #[test]
    fn classic_field_repair_guard_handles_zero_distance() {
        let mut state = GameState {
            features: FeatureFlags {
                travel_v2: false,
                ..FeatureFlags::default()
            },
            distance_today: 0.0,
            partial_distance_today: 0.0,
            ..GameState::default()
        };
        state.start_of_day();
        state.apply_classic_field_repair_guard();
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == LOG_VEHICLE_FIELD_REPAIR_GUARD)
        );
    }

    #[test]
    fn aggressive_emergency_and_field_repair_paths() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            vehicle: Vehicle {
                health: 10.0,
                wear: 40.0,
                ..Vehicle::default()
            },
            miles_traveled_actual: 1_960.0,
            features: FeatureFlags {
                travel_v2: false,
                ..FeatureFlags::default()
            },
            distance_today: 4.0,
            partial_distance_today: 2.0,
            budget_cents: 20_000,
            budget: 200,
            ..GameState::default()
        };
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        state.start_of_day();

        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.miles_traveled_actual = 1_951.0;
        state.distance_today = 5.0;
        let limp_triggered = state.try_emergency_limp_guard();
        assert!(limp_triggered);

        state.miles_traveled_actual = 1_700.0;
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        let deep_repair = state.try_deep_aggressive_field_repair();
        assert!(deep_repair);

        state.prev_miles_traveled = state.miles_traveled_actual - 10.0;
        state.reset_today_progress();
        state.recent_travel_days.clear();
        for _ in 0..6 {
            state.recent_travel_days.push_back(TravelDayKind::NonTravel);
        }
        state.enforce_aggressive_delay_cap(0.0);
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_PARTIAL));

        state.logs.clear();
        state.apply_delay_travel_credit("delay_test");
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_DELAY_CREDIT));
    }

    #[test]
    fn deep_aggressive_safeguards_and_compose() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 1_950.0,
            day: 220,
            stats: Stats {
                sanity: 0,
                pants: 30,
                ..Stats::default()
            },
            budget_cents: 10_000,
            budget: 100,
            current_day_kind: None,
            ..GameState::default()
        };

        state.start_of_day();
        state.apply_deep_aggressive_sanity_guard();
        assert!(state.guards.deep_aggressive_sanity_guard_used);
        assert!(state.logs.iter().any(|log| log == LOG_BOSS_COMPOSE));

        // Compose with supplies available.
        state.stats.supplies = BOSS_COMPOSE_SUPPLY_COST + 1;
        let composed_supplies = state.apply_deep_aggressive_compose();
        assert!(composed_supplies);

        // Compose fallback using funds.
        state.stats.supplies = 0;
        state.budget_cents = BOSS_COMPOSE_FUNDS_COST + 100;
        let composed_funds = state.apply_deep_aggressive_compose();
        assert!(composed_funds);
    }

    #[test]
    fn compute_miles_variations_cover_paths() {
        let mut state = GameState {
            data: Some(EncounterData::empty()),
            mode: GameMode::Classic,
            pace: PaceId::Blitz,
            features: FeatureFlags {
                travel_v2: false,
                ..FeatureFlags::default()
            },
            weather_travel_multiplier: 0.5,
            ..GameState::default()
        };
        let mut limits = crate::pacing::PacingLimits {
            distance_base: 30.0,
            ..crate::pacing::PacingLimits::default()
        };
        let mut pace = crate::pacing::PaceCfg {
            distance: 0.0,
            dist_mult: 0.0,
            ..crate::pacing::PaceCfg::default()
        };
        let classic = state.compute_miles_for_today(&pace, &limits);
        assert!(classic > 0.0);

        // Travel v2 branch with fallback defaults.
        state.features.travel_v2 = true;
        state.mode = GameMode::Deep;
        pace.distance = 0.0;
        pace.dist_mult = 0.0;
        limits.distance_base = 0.0;
        let v2 = state.compute_miles_for_today(&pace, &limits);
        assert!(v2 > 0.0);
        assert!((classic - v2).abs() > f32::EPSILON);
    }

    #[test]
    fn mobility_readiness_tracks_vehicle_health() {
        let mut state = GameState::default();
        state.vehicle.health = VEHICLE_HEALTH_MAX;
        let ready_full = state.mobility_readiness();
        assert!((ready_full - 1.0).abs() <= f32::EPSILON);

        state.vehicle.health = VEHICLE_CRITICAL_THRESHOLD;
        let ready_critical = state.mobility_readiness();
        assert!((ready_critical - VEHICLE_CRITICAL_SPEED_FACTOR).abs() <= f32::EPSILON);
    }

    #[test]
    fn enumeration_roundtrips_cover_branches() {
        use std::str::FromStr;

        assert_eq!(PaceId::Steady.as_str(), "steady");
        assert_eq!(PaceId::from_str("heated").unwrap(), PaceId::Heated);
        assert!(PaceId::from_str("invalid").is_err());
        assert_eq!(String::from(PaceId::Blitz), "blitz");
        assert_eq!(format!("{}", PaceId::Heated), "heated");

        assert_eq!(DietId::Doom.as_str(), "doom");
        assert_eq!(DietId::from_str("mixed").unwrap(), DietId::Mixed);
        assert!(DietId::from_str("bad").is_err());
        assert_eq!(String::from(DietId::Quiet), "quiet");
        assert_eq!(format!("{}", DietId::Mixed), "mixed");

        assert_eq!(PolicyKind::Aggressive.as_str(), "aggressive");
        assert_eq!(
            PolicyKind::from_str("balanced").unwrap(),
            PolicyKind::Balanced
        );
        assert!(PolicyKind::from_str("oops").is_err());
        assert_eq!(
            String::from(PolicyKind::ResourceManager),
            "resource_manager"
        );

        assert!(!GameMode::Classic.is_deep());
        assert!(GameMode::Deep.is_deep());
        assert_eq!(GameMode::Classic.boss_threshold(), 1_000);
        assert_eq!(GameMode::Deep.boss_threshold(), 1_200);

        assert_eq!(Region::Heartland.asset_key(), "Heartland");
        assert_eq!(Region::RustBelt.asset_key(), "RustBelt");
        assert_eq!(Region::Beltway.asset_key(), "Beltway");

        assert_eq!(Season::from_day(1), Season::Spring);
        assert_eq!(Season::from_day(46), Season::Summer);
        assert_eq!(Season::from_day(91), Season::Fall);
        assert_eq!(Season::from_day(150), Season::Winter);

        let causes = [
            CollapseCause::Hunger,
            CollapseCause::Vehicle,
            CollapseCause::Weather,
            CollapseCause::Breakdown,
            CollapseCause::Disease,
            CollapseCause::Crossing,
            CollapseCause::Panic,
        ];
        for cause in causes {
            assert!(!cause.key().is_empty());
        }

        assert_eq!(ExposureKind::Cold.key(), "cold");
        assert_eq!(ExposureKind::Heat.key(), "heat");
    }

    #[test]
    fn end_of_day_variants_cover_remaining_paths() {
        // Early return when already finalized.
        let mut early = GameState {
            encounter_history: VecDeque::from(vec![0]),
            day_state: DayState {
                lifecycle: LifecycleState {
                    did_end_of_day: true,
                    ..LifecycleState::default()
                },
                ..DayState::default()
            },
            ..GameState::default()
        };
        early.end_of_day();
        assert!(early.day_state.lifecycle.did_end_of_day);

        // No travel paths ensure assertion branch executes without panic.
        let mut stagnant = GameState {
            encounter_history: VecDeque::from(vec![0]),
            prev_miles_traveled: 10.0,
            miles_traveled_actual: 10.0,
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: false,
                    partial_traveled_today: false,
                    ..TravelDayState::default()
                },
                lifecycle: LifecycleState {
                    day_initialized: true,
                    ..LifecycleState::default()
                },
                ..DayState::default()
            },
            current_day_kind: Some(TravelDayKind::NonTravel),
            ..GameState::default()
        };
        stagnant.end_of_day();
        assert!(stagnant.day_state.lifecycle.did_end_of_day);
        assert_eq!(stagnant.recent_travel_days.len(), 1);

        // Deep conservative branch applies travel bonus and rotation enforcement.
        let rotation_interval = GameState::default().rotation_force_interval();
        let mut conservative = GameState {
            encounter_history: VecDeque::from(vec![0]),
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Conservative),
            encounters_today: 1,
            prev_miles_traveled: 100.0,
            miles_traveled_actual: 105.0,
            current_day_kind: Some(TravelDayKind::Travel),
            current_day_miles: 3.0,
            distance_today: 2.0,
            distance_today_raw: 2.5,
            partial_distance_today: 1.5,
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: true,
                    ..TravelDayState::default()
                },
                lifecycle: LifecycleState {
                    day_initialized: true,
                    ..LifecycleState::default()
                },
                ..DayState::default()
            },
            distance_cap_today: 6.0,
            current_day_reason_tags: vec!["progress".into()],
            rotation_travel_days: rotation_interval,
            recent_travel_days: VecDeque::from(vec![TravelDayKind::Partial; TRAVEL_HISTORY_WINDOW]),
            ..GameState::default()
        };
        conservative.end_of_day();
        assert!(conservative.encounters.force_rotation_pending);
        assert!(
            conservative
                .day_reason_history
                .last()
                .is_some_and(|entry| entry.contains("progress"))
        );

        // Deep aggressive branch unlocks boss readiness.
        let mut aggressive = GameState {
            encounter_history: VecDeque::from(vec![0]),
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            prev_miles_traveled: DEEP_AGGRESSIVE_BOSS_BIAS_MILES - 10.0,
            miles_traveled_actual: DEEP_AGGRESSIVE_BOSS_BIAS_MILES + 5.0,
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: true,
                    ..TravelDayState::default()
                },
                lifecycle: LifecycleState {
                    day_initialized: true,
                    ..LifecycleState::default()
                },
                ..DayState::default()
            },
            distance_today: 5.0,
            distance_today_raw: 5.0,
            current_day_miles: 5.0,
            current_day_reason_tags: vec!["march".into()],
            ..GameState::default()
        };
        aggressive.end_of_day();
        assert!(aggressive.boss.readiness.ready);
        assert!(aggressive.boss.readiness.reached);
    }

    #[test]
    fn encounter_recording_updates_history() {
        let mut state = GameState {
            encounter_history: VecDeque::from(vec![0]),
            ..GameState::default()
        };
        state.record_encounter("alpha");
        assert_eq!(state.encounters_today, 1);
        assert!(
            state
                .recent_encounters
                .iter()
                .any(|entry| entry.id == "alpha")
        );
    }

    #[test]
    fn travel_and_rotation_helpers_reset_reason_tags() {
        let mut state = GameState {
            current_day_kind: Some(TravelDayKind::Travel),
            current_day_reason_tags: vec!["camp".into(), "repair".into()],
            travel_days: 1,
            partial_travel_days: 1,
            non_travel_days: 1,
            days_with_camp: 1,
            days_with_repair: 1,
            rotation_travel_days: 2,
            ..GameState::default()
        };
        state.revert_current_day_record();
        assert!(state.current_day_reason_tags.is_empty());

        state.start_of_day();
        let _ = state.apply_travel_progress(5.0, TravelProgressKind::Partial);
        assert!(state.day_state.travel.partial_traveled_today);

        assert!(state.rotation_force_interval() >= 3);
        state.recent_travel_days = VecDeque::from(vec![
            TravelDayKind::Travel,
            TravelDayKind::Partial,
            TravelDayKind::NonTravel,
        ]);
        assert!(state.travel_ratio_recent(3) < 1.0);

        state.day_state.travel.traveled_today = true;
        state.day_state.travel.partial_traveled_today = false;
        state.apply_partial_travel_credit(1.0, "log.partial.credit", "delay");
        assert!(state.logs.iter().any(|entry| entry == "log.partial.credit"));
    }

    #[test]
    fn repair_guards_and_limp_paths_execute() {
        let mut state = GameState {
            mode: GameMode::Classic,
            budget_cents: 5_000,
            budget: 50,
            vehicle: Vehicle {
                wear: 40.0,
                ..Vehicle::default()
            },
            breakdown: Some(Breakdown {
                part: Part::Battery,
                day_started: 1,
            }),
            day_state: DayState {
                travel: TravelDayState {
                    travel_blocked: true,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            ..GameState::default()
        };
        state.start_of_day();
        state.apply_classic_field_repair_guard();
        assert!(!state.day_state.travel.travel_blocked);

        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.miles_traveled_actual = 1_920.0;
        state.endgame.last_limp_mile = 0.0;
        state.budget_cents = 8_000;
        state.budget = 80;
        let limp = state.try_emergency_limp_guard();
        assert!(limp);

        state.miles_traveled_actual = 1_700.0;
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        let field = state.try_deep_aggressive_field_repair();
        assert!(field);
    }

    #[test]
    fn encounter_penalties_and_boosts_apply() {
        let mut state = GameState::default();
        state.add_day_reason_tag("camp");
        state.add_day_reason_tag("repair");
        state.add_day_reason_tag("camp");
        state.add_day_reason_tag(" ");
        assert!(state.days_with_camp > 0);
        assert!(state.days_with_repair > 0);

        state.features.encounter_diversity = true;
        state.day = 50;
        state.recent_encounters.push_back(RecentEncounter::new(
            "alpha".into(),
            49,
            Region::Heartland,
        ));
        assert!(state.should_discourage_encounter("alpha"));
        assert!(!state.should_discourage_encounter("beta"));

        state.policy = Some(PolicyKind::Conservative);
        assert!(state.encounter_reroll_penalty() < 1.0);
        state.policy = Some(PolicyKind::Balanced);
        assert!(state.encounter_reroll_penalty() > 0.0);
    }

    #[test]
    fn health_and_sanity_boosts_apply() {
        let mut state = GameState::default();
        assert!((state.vehicle_health() - state.vehicle.health).abs() < f32::EPSILON);

        state.stats.supplies = 10;
        state.starvation_days = 2;
        state.apply_starvation_tick();
        assert_eq!(state.starvation_days, 0);

        state.stats.allies = 2;
        state.logs.clear();
        state.attach_rng_bundle(events_bundle_with_roll_below(ALLY_ATTRITION_CHANCE * 0.5));
        state.tick_ally_attrition();
        assert!(state.logs.iter().any(|entry| entry == LOG_ALLY_LOST));

        state.weather_state.today = Weather::Smoke;
        assert!(state.current_weather_speed_penalty() < WEATHER_DEFAULT_SPEED);

        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Conservative);
        state.day = 150;
        state.miles_traveled_actual = 1_900.0;
        assert!(state.deep_conservative_travel_boost() > 1.0);
        state.policy = Some(PolicyKind::Aggressive);
        assert!(state.deep_aggressive_reach_boost() >= 1.0);

        state.day = DEEP_AGGRESSIVE_SANITY_DAY;
        state.miles_traveled_actual = DEEP_AGGRESSIVE_SANITY_MILES;
        state.stats.sanity = 0;
        state.stats.pants = 30;
        state.budget_cents = DEEP_AGGRESSIVE_SANITY_COST + 1_000;
        state.budget = i32::try_from(state.budget_cents / 100).unwrap_or(0);
        state.guards.deep_aggressive_sanity_guard_used = false;
        state.start_of_day();
        state.apply_deep_aggressive_sanity_guard();
        assert!(state.guards.deep_aggressive_sanity_guard_used);

        state.stats.supplies = BOSS_COMPOSE_SUPPLY_COST + 1;
        assert!(state.apply_deep_aggressive_compose());
        state.stats.supplies = 0;
        state.budget_cents = BOSS_COMPOSE_FUNDS_COST + 500;
        assert!(state.apply_deep_aggressive_compose());
    }

    #[test]
    fn general_strain_combines_components() {
        let mut state = GameState::default();
        state.stats.hp = 8;
        state.stats.sanity = 7;
        state.stats.pants = 12;
        state.malnutrition_level = 2;
        state.vehicle.wear = 5.0;
        state.weather_state.today = Weather::Storm;
        state.current_order = Some(ExecOrder::Shutdown);

        let cfg = StrainConfig {
            weights: StrainWeights {
                hp: 1.0,
                sanity: 1.0,
                pants: 1.0,
                starvation: 1.0,
                vehicle: 1.0,
                weather: 1.0,
                exec: 1.0,
            },
            weather_severity: HashMap::from([(Weather::Storm, 2.0)]),
            exec_order_bonus: HashMap::from([(String::from("shutdown"), 3.0)]),
            vehicle_wear_norm_denom: 10.0,
            strain_norm_denom: 4.0,
            label_bounds: StrainLabelBounds::default(),
        };

        let strain = state.update_general_strain(&cfg);

        let expected = 2.0 + 3.0 + 12.0 + 2.0 + 0.5 + 2.0 + 3.0;
        approx_eq(strain, expected);
        approx_eq(state.general_strain, expected);
    }

    #[test]
    fn general_strain_labels_follow_bounds() {
        let cfg = StrainConfig {
            strain_norm_denom: 4.0,
            label_bounds: StrainLabelBounds {
                good_max: 0.25,
                fair_max: 0.5,
                poor_max: 0.75,
            },
            ..StrainConfig::default()
        };
        let mut state = GameState::default();

        assert_eq!(state.general_strain_label(&cfg), HealthLabel::Good);

        state.general_strain = 1.0;
        assert_eq!(state.general_strain_label(&cfg), HealthLabel::Fair);

        state.general_strain = 1.9;
        assert_eq!(state.general_strain_label(&cfg), HealthLabel::Fair);

        state.general_strain = 2.0;
        assert_eq!(state.general_strain_label(&cfg), HealthLabel::Poor);

        state.general_strain = 3.0;
        assert_eq!(state.general_strain_label(&cfg), HealthLabel::VeryPoor);
    }

    #[test]
    fn otdeluxe_affliction_probability_interpolates() {
        let policy = default_otdeluxe_policy();
        let probability = otdeluxe_affliction_probability(52, &policy.affliction);
        assert!((probability - 0.10).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_affliction_kind_records_decision_trace() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["Ada"]);
        state.ot_deluxe.health_general = 140;
        state.attach_rng_bundle(health_bundle_with_roll_below(0.2));
        state.start_of_day();

        let outcome = state.tick_otdeluxe_afflictions();

        assert!(outcome.is_some());
        assert!(
            state
                .decision_traces_today
                .iter()
                .any(|trace| trace.pool_id == "otdeluxe.affliction_kind")
        );
        assert!(
            state
                .decision_traces_today
                .iter()
                .any(|trace| trace.pool_id.starts_with("otdeluxe.affliction_disease."))
        );
    }

    #[test]
    fn otdeluxe_affliction_override_weights_prefer_injury() {
        let policy = OtDeluxe90sPolicy::default();
        let mut overrides = OtDeluxePolicyOverride::default();
        overrides.affliction_weights.illness = Some(0);
        overrides.affliction_weights.injury = Some(5);

        let mut rng = SmallRng::seed_from_u64(1);
        let (kind, trace) = roll_otdeluxe_affliction_kind(&policy.affliction, &overrides, &mut rng);
        assert!(matches!(kind, OtDeluxeAfflictionKind::Injury));

        let trace = trace.expect("expected decision trace");
        let illness = trace
            .candidates
            .iter()
            .find(|candidate| candidate.id == "illness")
            .expect("illness candidate");
        assert!(illness.final_weight.abs() <= f64::EPSILON);
    }

    #[test]
    fn otdeluxe_travel_scales_with_oxen_and_sick_party() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 2;
        state.ot_deluxe.pace = OtDeluxePace::Steady;
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B"]);
        state.ot_deluxe.party.members[0].sick_days_remaining = 1;

        state.compute_otdeluxe_travel_distance_today();

        assert!((state.distance_today - 9.0).abs() <= 1e-6);
        assert!((state.distance_today_raw - 9.0).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_travel_base_speed_on_plains_is_20() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.pace = OtDeluxePace::Steady;
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B"]);
        state.ot_deluxe.miles_traveled = 0.0;

        state.compute_otdeluxe_travel_distance_today();

        assert!((state.distance_today - 20.0).abs() <= 1e-6);
        assert!((state.distance_today_raw - 20.0).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_travel_mountains_halves_speed() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.pace = OtDeluxePace::Steady;
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B"]);
        state.ot_deluxe.miles_traveled = 932.0;

        state.compute_otdeluxe_travel_distance_today();

        assert!((state.distance_today - 10.0).abs() <= 1e-6);
        assert!((state.distance_today_raw - 10.0).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_travel_sick_member_penalty_reduces_speed() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.pace = OtDeluxePace::Steady;
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B"]);
        state.ot_deluxe.party.members[0].sick_days_remaining = 1;
        state.ot_deluxe.miles_traveled = 0.0;

        state.compute_otdeluxe_travel_distance_today();

        assert!((state.distance_today - 18.0).abs() <= 1e-6);
        assert!((state.distance_today_raw - 18.0).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_travel_sick_ox_scales_effective_oxen() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 3;
        state.ot_deluxe.oxen.sick = 1;
        state.ot_deluxe.pace = OtDeluxePace::Steady;
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B"]);
        state.ot_deluxe.miles_traveled = 0.0;

        state.compute_otdeluxe_travel_distance_today();

        assert!((state.distance_today - 17.5).abs() <= 1e-6);
        assert!((state.distance_today_raw - 17.5).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_travel_applies_snow_multiplier() {
        let mut state = GameState::default();
        let mut policy = OtDeluxe90sPolicy::default();
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.weather.snow_depth = 2.0;
        policy.travel.snow_speed_penalty_per_in = 0.1;

        let miles = state.compute_otdeluxe_miles_for_today(&policy);

        assert!((miles - 16.0).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_terrain_updates_from_miles() {
        let mut state = GameState::default();
        let policy = OtDeluxe90sPolicy::default();
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.miles_traveled = 932.0;

        let _ = state.compute_otdeluxe_miles_for_today(&policy);

        assert!(matches!(
            state.ot_deluxe.terrain,
            OtDeluxeTerrain::Mountains
        ));
    }

    #[test]
    fn otdeluxe_navigation_event_requires_snow_depth() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 0,
            wrong_weight: 0,
            impassable_weight: 0,
            snowbound_weight: 1,
            snowbound_min_depth_in: 5.0,
            ..OtDeluxeNavigationPolicy::default()
        };

        let mut rng = SmallRng::seed_from_u64(7);
        let (none, _) = roll_otdeluxe_navigation_event_with_trace(&policy, 2.0, &mut rng);
        assert!(none.is_none());

        let mut rng = SmallRng::seed_from_u64(7);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 6.0, &mut rng);
        assert!(matches!(event, Some(OtDeluxeNavigationEvent::Snowbound)));
        assert!(trace.is_some());
    }

    #[test]
    fn otdeluxe_navigation_event_uses_events_rng_with_fixed_policy() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.start_of_day();
        state.distance_today = 10.0;
        state.distance_today_raw = 10.0;
        state.partial_distance_today = 2.0;
        state.ot_deluxe.oxen.healthy = 4;
        let bundle = Rc::new(RngBundle::from_user_seed(77));
        state.attach_rng_bundle(bundle.clone());

        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 1,
            wrong_weight: 0,
            impassable_weight: 0,
            snowbound_weight: 0,
            lost_delay: OtDeluxeNavigationDelay {
                min_days: 1,
                max_days: 1,
            },
            wrong_delay: OtDeluxeNavigationDelay {
                min_days: 1,
                max_days: 1,
            },
            impassable_delay: OtDeluxeNavigationDelay {
                min_days: 1,
                max_days: 1,
            },
            snowbound_delay: OtDeluxeNavigationDelay {
                min_days: 1,
                max_days: 1,
            },
            snowbound_min_depth_in: 0.0,
        };

        let draws_before = bundle.events().draws();
        let applied = state.apply_otdeluxe_navigation_event_with_policy(&policy);
        let draws_after = bundle.events().draws();

        assert!(applied);
        assert!(draws_after > draws_before);
        assert!(state.day_state.lifecycle.did_end_of_day);
    }

    #[test]
    fn otdeluxe_navigation_hard_stop_records_nontravel() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            distance_today: 12.0,
            distance_today_raw: 12.0,
            partial_distance_today: 6.0,
            ..GameState::default()
        };

        state.start_of_day();
        state.apply_otdeluxe_navigation_hard_stop(OtDeluxeNavigationEvent::LostTrail, 2);

        assert_eq!(state.ot_deluxe.travel.delay_days_remaining, 1);
        assert_eq!(state.ot_deluxe.travel.blocked_days_remaining, 0);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Delayed
        ));
        approx_eq(state.distance_today, 0.0);
        approx_eq(state.distance_today_raw, 0.0);
        let record = state.day_records.last().expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "otdeluxe.nav_lost"));
    }

    #[test]
    fn otdeluxe_random_event_uses_events_rng() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.food_lbs = 200;
        state.ot_deluxe.oxen.healthy = 4;
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["Ada"]);

        let bundle = events_bundle_with_roll_below(0.2);
        state.attach_rng_bundle(bundle.clone());

        let before_events = bundle.events().draws();
        let before_health = bundle.health().draws();

        let _ = state.apply_otdeluxe_random_event();

        let after_events = bundle.events().draws();
        let after_health = bundle.health().draws();
        assert!(after_events > before_events);
        assert_eq!(after_health, before_health);
    }

    #[test]
    fn otdeluxe_random_event_applies_resource_change() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.food_lbs = 10;
        state.ot_deluxe.inventory.bullets = 0;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("resource_change"),
            variant_id: Some(String::from("wild_fruit")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(7);
        let (log_key, severity, _payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert_eq!(
            log_key,
            "log.otdeluxe.random_event.resource_change.wild_fruit"
        );
        assert!(matches!(severity, EventSeverity::Info));
        assert!(state.ot_deluxe.inventory.food_lbs > 10);
    }

    #[test]
    fn otdeluxe_random_event_applies_weather_catastrophe() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.food_lbs = 100;
        state.ot_deluxe.health_general = 50;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("weather_catastrophe"),
            variant_id: Some(String::from("blizzard")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(11);
        let (log_key, severity, payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert_eq!(
            log_key,
            "log.otdeluxe.random_event.weather_catastrophe.blizzard"
        );
        assert!(matches!(severity, EventSeverity::Warning));
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 90);
        assert_eq!(state.ot_deluxe.health_general, 55);
        assert!(payload.get("deltas").is_some());
    }

    #[test]
    fn otdeluxe_random_event_applies_resource_shortage_bad_water() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["Ada"]);
        state.ot_deluxe.health_general = 10;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("resource_shortage"),
            variant_id: Some(String::from("bad_water")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(23);
        let (_log_key, severity, payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert!(matches!(severity, EventSeverity::Warning));
        assert!(state.ot_deluxe.health_general > 10);
        assert!(payload.get("affliction").is_some());
    }

    #[test]
    fn otdeluxe_random_event_applies_resource_shortage_no_grass() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.healthy = 1;
        state.ot_deluxe.oxen.sick = 0;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("resource_shortage"),
            variant_id: Some(String::from("no_grass")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(31);
        let (_log_key, severity, _payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert!(matches!(severity, EventSeverity::Warning));
        assert_eq!(state.ot_deluxe.oxen.healthy, 0);
        assert_eq!(state.ot_deluxe.oxen.sick, 1);
    }

    #[test]
    fn otdeluxe_random_event_applies_party_incident_lost_member() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["Ada", "Ben"]);

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("party_incident"),
            variant_id: Some(String::from("lost_member")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(41);
        let (_log_key, severity, payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert!(matches!(severity, EventSeverity::Critical));
        assert_eq!(state.ot_deluxe.party.alive_count(), 1);
        assert!(payload.get("lost_members").is_some());
    }

    #[test]
    fn otdeluxe_random_event_applies_oxen_incident() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.oxen.healthy = 1;
        state.ot_deluxe.oxen.sick = 0;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("oxen_incident"),
            variant_id: Some(String::from("ox_sickness")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(43);
        let (_log_key, severity, _payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert!(matches!(severity, EventSeverity::Warning));
        assert_eq!(state.ot_deluxe.oxen.healthy, 0);
        assert_eq!(state.ot_deluxe.oxen.sick, 1);
    }

    #[test]
    fn otdeluxe_random_event_applies_wagon_part_break_without_spares() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.spares_wheels = 0;
        state.ot_deluxe.inventory.spares_axles = 0;
        state.ot_deluxe.inventory.spares_tongues = 0;
        state.ot_deluxe.inventory.food_lbs = 50;
        state.ot_deluxe.inventory.clothes_sets = 2;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("wagon_part_break"),
            variant_id: Some(String::from("unrepairable")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(53);
        let (_log_key, severity, payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert!(matches!(severity, EventSeverity::Critical));
        assert!(state.ot_deluxe.inventory.food_lbs < 50);
        assert!(state.ot_deluxe.inventory.clothes_sets < 2);
        assert!(payload.get("spare_lost").is_some());
    }

    #[test]
    fn otdeluxe_random_event_applies_travel_hazard() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.inventory.food_lbs = 40;
        state.ot_deluxe.health_general = 20;

        let selection = OtDeluxeRandomEventSelection {
            event_id: String::from("travel_hazard"),
            variant_id: Some(String::from("rough_trail")),
            chance_roll: 0.0,
            chance_threshold: 1.0,
        };
        let mut rng = SmallRng::seed_from_u64(61);
        let (_log_key, severity, _payload) = state
            .apply_otdeluxe_random_event_selection(&selection, &mut rng)
            .expect("expected outcome");

        assert!(matches!(severity, EventSeverity::Warning));
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 35);
        assert_eq!(state.ot_deluxe.health_general, 22);
    }

    #[test]
    fn otdeluxe_navigation_delay_consumes_day() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.travel.delay_days_remaining = 2;
        state.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Delayed;

        state.start_of_day();
        assert!(state.consume_otdeluxe_navigation_delay_day());

        assert_eq!(state.ot_deluxe.travel.delay_days_remaining, 1);
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Delayed
        ));
        let record = state.day_records.last().expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "otdeluxe.nav_delay"));
    }

    #[test]
    fn otdeluxe_consumption_scales_with_rations_pace_and_alive_members() {
        let mut state = GameState::default();
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B", "C"]);
        state.ot_deluxe.inventory.food_lbs = 50;
        state.ot_deluxe.rations = OtDeluxeRations::Meager;
        state.ot_deluxe.pace = OtDeluxePace::Grueling;

        let consumed_meager = state.apply_otdeluxe_consumption();

        assert_eq!(consumed_meager, 12);
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 38);

        state.ot_deluxe.inventory.food_lbs = 50;
        state.ot_deluxe.rations = OtDeluxeRations::BareBones;

        let consumed_bare = state.apply_otdeluxe_consumption();

        assert_eq!(consumed_bare, 6);
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 44);
    }

    #[test]
    fn otdeluxe_pace_clears_encounter_chance() {
        let mut state = GameState::default();
        state.weather_state.today = Weather::Clear;
        state.encounter_chance_today = 0.7;

        state.apply_otdeluxe_pace_and_rations();

        assert!(state.encounter_chance_today.abs() < f32::EPSILON);
    }

    #[test]
    fn otdeluxe_health_update_applies_pace_and_rations() {
        let mut state = GameState::default();
        let policy = default_otdeluxe_policy();
        state.ot_deluxe.health_general = 20;
        state.ot_deluxe.pace = OtDeluxePace::Grueling;
        state.ot_deluxe.rations = OtDeluxeRations::BareBones;

        let delta = state.apply_otdeluxe_health_update();

        let expected_delta = otdeluxe_health_delta(&state, policy);
        let expected_health = (20 + expected_delta).max(0);
        assert_eq!(delta, expected_delta);
        assert_eq!(
            state.ot_deluxe.health_general,
            u16::try_from(expected_health).unwrap_or(u16::MAX)
        );
    }

    #[test]
    fn otdeluxe_health_delta_includes_weather_and_clothing_penalties() {
        let mut state = GameState::default();
        state.ot_deluxe.party = OtDeluxePartyState::from_names(["A"]);
        state.ot_deluxe.inventory.clothes_sets = 0;
        state.ot_deluxe.season = Season::Winter;
        state.weather_state.today = Weather::Storm;

        let mut policy = OtDeluxe90sPolicy::default();
        policy.health.weather_penalty.insert(Weather::Storm, 7);
        policy.health.clothing_penalty_winter = 5;
        policy.health.clothing_sets_per_person = 2;

        let delta = otdeluxe_health_delta(&state, &policy);

        let expected = policy.health.recovery_baseline
            + policy.pace_health_penalty.steady
            + policy.rations.health_penalty[0]
            + 7
            + 5;
        assert_eq!(delta, expected);
    }

    #[test]
    fn otdeluxe_death_imminent_counts_down_and_resets() {
        let mut state = GameState::default();
        let policy = OtDeluxe90sPolicy::default();
        state.ot_deluxe.health_general = policy.health.death_threshold;

        state.update_otdeluxe_death_imminent(&policy.health);
        assert_eq!(
            state.ot_deluxe.death_imminent_days_remaining,
            policy.health.death_imminent_grace_days
        );

        state.update_otdeluxe_death_imminent(&policy.health);
        assert_eq!(
            state.ot_deluxe.death_imminent_days_remaining,
            policy.health.death_imminent_grace_days.saturating_sub(1)
        );

        state.ot_deluxe.health_general = policy.health.death_threshold.saturating_sub(1);
        state.update_otdeluxe_death_imminent(&policy.health);
        assert_eq!(state.ot_deluxe.death_imminent_days_remaining, 0);
    }

    #[test]
    fn otdeluxe_doctor_fatality_mult_scales_probability() {
        let policy = OtDeluxe90sPolicy::default();
        let model = FatalityModel {
            base_prob_per_day: 1.0,
            apply_doctor_mult: true,
            prob_modifiers: Vec::new(),
        };
        let context = OtDeluxeFatalityContext {
            health_general: 0,
            pace: OtDeluxePace::Steady,
            rations: OtDeluxeRations::Filling,
            weather: Weather::Clear,
            occupation: Some(OtDeluxeOccupation::Doctor),
        };

        let prob = otdeluxe_fatality_probability(&model, context, &policy);

        assert!((prob - policy.occupation_advantages.doctor_fatality_mult).abs() <= 1e-6);
    }

    #[test]
    fn otdeluxe_route_prompt_clamps_at_south_pass() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.miles_traveled_actual = 900.0;
        state.prev_miles_traveled = 900.0;
        state.ot_deluxe.miles_traveled = 900.0;
        state.sync_otdeluxe_trail_distance();

        let applied = state.apply_travel_progress(50.0, TravelProgressKind::Full);

        assert!((applied - 32.0).abs() <= 1e-3);
        assert!((state.miles_traveled_actual - 932.0).abs() <= 1e-3);
        assert_eq!(
            state.ot_deluxe.route.pending_prompt,
            Some(OtDeluxeRoutePrompt::SubletteCutoff)
        );
    }

    #[test]
    fn otdeluxe_route_prompt_resolves_and_updates_variant() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);

        let resolved = state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::SubletteCutoff);

        assert!(resolved);
        assert_eq!(state.ot_deluxe.route.pending_prompt, None);
        assert_eq!(
            state.ot_deluxe.route.variant,
            OtDeluxeTrailVariant::SubletteCutoff
        );
    }

    #[test]
    fn apply_persona_updates_stats_and_logs() {
        let persona = Persona {
            id: "organizer".to_string(),
            name: "Organizer".to_string(),
            desc: "Test persona".to_string(),
            score_mult: 1.25,
            start: PersonaStart {
                supplies: 5,
                credibility: 7,
                sanity: 3,
                morale: 4,
                allies: 2,
                budget: 15,
            },
            mods: PersonaMods {
                pants_relief: 2,
                pants_relief_threshold: 10,
                ..PersonaMods::default()
            },
        };
        let mut state = GameState::default();
        state.apply_persona(&persona);

        assert_eq!(state.persona_id.as_deref(), Some("organizer"));
        assert!((state.score_mult - 1.25).abs() <= f32::EPSILON);
        assert_eq!(state.stats.supplies, 5);
        assert_eq!(state.stats.credibility, 7);
        assert_eq!(state.stats.sanity, 3);
        assert_eq!(state.stats.morale, 4);
        assert_eq!(state.stats.allies, 2);
        assert_eq!(state.budget, 15);
        assert_eq!(state.budget_cents, 1500);
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == "log.persona.selected.organizer")
        );
    }

    #[test]
    fn set_party_fills_companion_slots() {
        let mut state = GameState::default();
        state.set_party("Leader", vec!["Alice", "Bob"]);
        assert_eq!(state.party.leader, "Leader");
        assert_eq!(state.party.companions.len(), 4);
        assert_eq!(state.party.companions[0], "Alice");
        assert_eq!(state.party.companions[1], "Bob");
        assert!(state.party.companions[2].starts_with("Traveler"));
        assert!(state.party.companions[3].starts_with("Traveler"));
        assert!(state.logs.iter().any(|log| log == "log.party.updated"));
    }

    #[test]
    fn apply_store_purchase_updates_budget_and_grants() {
        let mut state = GameState {
            budget_cents: 1500,
            budget: 15,
            ..GameState::default()
        };
        state.stats = Stats {
            supplies: 0,
            credibility: 0,
            ..Stats::default()
        };
        let grants = Grants {
            supplies: 2,
            credibility: 3,
            spare_tire: 1,
            spare_battery: 2,
            spare_alt: 1,
            spare_pump: 1,
            enabled: true,
        };
        let tags = vec![String::from("safety"), String::from("comfort")];
        state.apply_store_purchase(500, &grants, &tags);

        assert_eq!(state.budget_cents, 1000);
        assert_eq!(state.budget, 10);
        assert_eq!(state.stats.supplies, 2);
        assert_eq!(state.stats.credibility, 3);
        assert_eq!(state.inventory.spares.tire, 1);
        assert_eq!(state.inventory.spares.battery, 2);
        assert_eq!(state.inventory.spares.alt, 1);
        assert_eq!(state.inventory.spares.pump, 1);
        assert!(state.inventory.tags.contains("safety"));
        assert!(state.inventory.tags.contains("comfort"));
    }

    #[test]
    fn otdeluxe_store_pending_flow_updates_state() {
        let mut state = GameState::default();
        let lines = vec![OtDeluxeStoreLineItem {
            item: crate::otdeluxe_store::OtDeluxeStoreItem::FoodLb,
            quantity: 10,
        }];
        assert!(!state.set_otdeluxe_store_purchase(lines.clone()));

        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        assert!(!state.set_otdeluxe_store_purchase(lines.clone()));

        state.ot_deluxe.store.pending_node = Some(0);
        assert!(state.set_otdeluxe_store_purchase(lines));
        assert!(state.ot_deluxe.store.pending_purchase.is_some());

        state.clear_otdeluxe_store_pending();
        assert!(state.ot_deluxe.store.pending_node.is_none());
        assert_eq!(state.ot_deluxe.store.last_node, Some(0));
        assert!(state.ot_deluxe.store.pending_purchase.is_none());

        state.ot_deluxe.store.last_node = None;
        state.ot_deluxe.route.current_node_index = 0;
        state.queue_otdeluxe_store_if_available();
        assert_eq!(state.ot_deluxe.store.pending_node, Some(0));
    }

    #[test]
    fn apply_otdeluxe_store_purchase_updates_inventory() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ot_deluxe: OtDeluxeState {
                inventory: OtDeluxeInventory {
                    cash_cents: 5000,
                    ..OtDeluxeInventory::default()
                },
                ..OtDeluxeState::default()
            },
            ..GameState::default()
        };

        let lines = [OtDeluxeStoreLineItem {
            item: crate::otdeluxe_store::OtDeluxeStoreItem::AmmoBox,
            quantity: 1,
        }];
        let receipt = state
            .apply_otdeluxe_store_purchase(0, &lines)
            .expect("purchase succeeds");
        assert!(receipt.total_cost_cents > 0);
        assert_eq!(state.ot_deluxe.inventory.bullets, 20);
    }

    #[test]
    fn advance_days_with_credit_noops_on_zero() {
        let mut state = GameState::default();
        let day_before = state.day;
        state.advance_days_with_credit(0, TravelDayKind::NonTravel, 0.0, "idle");
        assert_eq!(state.day, day_before);
    }

    #[test]
    fn rehydrate_backfills_records_and_otdeluxe_state() {
        let mut state = GameState {
            state_version: 3,
            day: 7,
            travel_days: 2,
            partial_travel_days: 1,
            miles_traveled_actual: 123.0,
            journey_partial_ratio: 1.5,
            pace: PaceId::Blitz,
            diet: DietId::Doom,
            budget_cents: 9000,
            pending_crossing_choice: Some(CrossingChoice::Detour),
            pending_route_choice: Some(OtDeluxeRouteDecision::StayOnTrail),
            rng_bundle: None,
            ..GameState::default()
        };
        state.party.leader = String::from("Leader");
        state.party.companions = vec![
            String::from("Comp 1"),
            String::from("Comp 2"),
            String::from("Comp 3"),
            String::from("Comp 4"),
            String::from("Comp 5"),
            String::from("Comp 6"),
        ];
        state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Ford);

        let rehydrated = state.rehydrate(EncounterData::empty());

        assert_eq!(rehydrated.state_version, GameState::current_version());
        assert!(rehydrated.rng_bundle.is_some());
        assert!(rehydrated.journey_partial_ratio <= 0.95);
        assert_eq!(rehydrated.day_records.len(), 1);
        let record = &rehydrated.day_records[0];
        assert!(matches!(record.kind, TravelDayKind::Travel));
        assert_eq!(rehydrated.ot_deluxe.party.members.len(), 5);
        assert_eq!(rehydrated.ot_deluxe.party.members[0].name, "Leader");
        assert!(
            rehydrated
                .ot_deluxe
                .party
                .members
                .iter()
                .all(|member| !member.name.trim().is_empty())
        );
        assert_eq!(rehydrated.ot_deluxe.pace, OtDeluxePace::Grueling);
        assert_eq!(rehydrated.ot_deluxe.rations, OtDeluxeRations::BareBones);
        assert_eq!(rehydrated.ot_deluxe.inventory.cash_cents, 9000);
        assert!(rehydrated.pending_crossing_choice.is_none());
        assert!(rehydrated.pending_route_choice.is_none());
        assert!(rehydrated.ot_deluxe.crossing.chosen_method.is_none());
    }

    #[test]
    fn build_otdeluxe_state_fills_missing_names() {
        let mut state = GameState::default();
        state.party.leader = String::new();
        state.party.companions = Vec::new();
        let ot_state = state.build_ot_deluxe_state_from_legacy();
        assert_eq!(ot_state.party.members.len(), 5);
        assert!(ot_state.party.members[0].name.starts_with("Traveler"));
    }

    #[test]
    fn apply_otdeluxe_start_config_sets_occupation_and_cash() {
        let encounters = EncounterData::empty();
        let mut state = GameState::default().with_seed(7, GameMode::Classic, encounters);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;

        state.apply_otdeluxe_start_config(OtDeluxeOccupation::Doctor);

        assert_eq!(
            state.ot_deluxe.mods.occupation,
            Some(OtDeluxeOccupation::Doctor)
        );
        assert_eq!(state.ot_deluxe.inventory.cash_cents, 120_000);
        assert_eq!(state.ot_deluxe.party.members.len(), 5);
    }

    #[test]
    fn sync_otdeluxe_trail_distance_and_prompt_marker() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            miles_traveled_actual: 0.0,
            ..GameState::default()
        };

        state.sync_otdeluxe_trail_distance();
        let policy = default_otdeluxe_policy();
        let expected =
            otdeluxe_trail::total_miles_for_variant(&policy.trail, state.ot_deluxe.route.variant);
        let expected_distance = f32::from(expected).max(1.0);
        assert!((state.trail_distance - expected_distance).abs() <= f32::EPSILON);

        let (prompt, marker) = state.otdeluxe_next_prompt_marker().expect("next prompt");
        assert_eq!(prompt, OtDeluxeRoutePrompt::SubletteCutoff);
        let expected_marker = otdeluxe_trail::mile_marker_for_node(
            &policy.trail,
            state.ot_deluxe.route.variant,
            otdeluxe_trail::SOUTH_PASS_NODE_INDEX,
        )
        .expect("south pass marker");
        assert_eq!(marker, expected_marker);

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesShortcut);
        assert!(state.otdeluxe_next_prompt_marker().is_none());
    }

    #[test]
    fn pre_travel_checks_block_on_pending_prompt() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            distance_today: 12.0,
            distance_today_raw: 9.0,
            ..GameState::default()
        };
        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesShortcut);

        let result = state.pre_travel_checks();

        assert!(matches!(result, Some((false, key, false)) if key == LOG_TRAVEL_BLOCKED));
        assert!(matches!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Stopped
        ));
        assert!(state.distance_today.abs() <= f32::EPSILON);
        assert!(state.distance_today_raw.abs() <= f32::EPSILON);
    }

    #[test]
    fn handle_travel_block_dystrail_applies_delay_credit() {
        let mut state = GameState::default();
        state.day_state.travel.travel_blocked = true;

        state.start_of_day();
        let result = state.handle_travel_block(false);

        assert!(result.is_some());
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_DELAY_CREDIT));
        let record = state.day_records.last().expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::Partial));
        assert!(record.tags.iter().any(|tag| tag.0 == "repair"));
    }

    #[test]
    fn handle_travel_block_otdeluxe_records_repair() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.day_state.travel.travel_blocked = true;
        state.distance_today = 10.0;
        state.distance_today_raw = 10.0;

        state.start_of_day();
        let result = state.handle_travel_block(false);

        assert!(result.is_some());
        let record = state.day_records.last().expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::NonTravel));
        assert!(record.tags.iter().any(|tag| tag.0 == "repair"));
        assert!(state.distance_today.abs() <= f32::EPSILON);
        assert!(state.distance_today_raw.abs() <= f32::EPSILON);
    }

    #[test]
    fn pending_crossing_choice_permit_resolves_pass() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 12.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Permit),
            ..GameState::default()
        };
        state
            .inventory
            .tags
            .insert(PERMIT_REQUIRED_TAGS[0].to_string());
        state.journey_crossing = CrossingPolicy {
            pass: 1.0,
            detour: 0.0,
            terminal: 0.0,
            ..CrossingPolicy::default()
        };
        state.journey_crossing.permit.disable_terminal = true;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(21)));

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_CROSSING_PASSED);
        assert!(outcome.day_consumed);
        assert!(state.crossings_completed >= 1);
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == LOG_CROSSING_DECISION_PERMIT)
        );
    }

    #[test]
    fn pending_crossing_choice_detour_advances_days() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 8.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Detour),
            ..GameState::default()
        };
        state.journey_crossing.detour_days = DetourPolicy { min: 2, max: 2 };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(31)));

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_CROSSING_DETOUR);
        assert!(state.crossing_detours_taken > 0);
        assert!(state.day >= 3);
    }

    #[test]
    fn pending_crossing_choice_bribe_can_fail_terminally() {
        let cfg = JourneyCfg::default();
        let endgame_cfg = EndgameTravelCfg::default_config();
        let kernel = DailyTickKernel::new(&cfg, &endgame_cfg);

        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::BridgeOut,
                computed_miles_today: 6.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Bribe),
            budget_cents: 2000,
            journey_crossing: CrossingPolicy {
                pass: 0.0,
                detour: 0.0,
                terminal: 1.0,
                ..CrossingPolicy::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(17)));

        let outcome = kernel.tick_day(&mut state);

        assert_eq!(outcome.log_key, LOG_CROSSING_FAILURE);
        assert!(state.ending.is_some());
        assert!(state.crossing_bribe_attempts > 0);
    }

    #[test]
    fn pending_crossing_choice_rejects_unavailable_permit() {
        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 5.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Permit),
            ..GameState::default()
        };

        let outcome = state.resolve_pending_crossing_choice(CrossingChoice::Permit);

        assert!(outcome.is_none());
        assert!(state.pending_crossing.is_some());
        assert!(state.pending_crossing_choice.is_none());
    }

    #[test]
    fn handle_crossing_event_sets_pending_choice() {
        let mut state = GameState {
            miles_traveled_actual: CROSSING_MILESTONES[0],
            ..GameState::default()
        };

        let result = state.handle_crossing_event(0.0);

        assert!(matches!(result, Some((false, key)) if key == LOG_TRAVEL_BLOCKED));
        assert!(state.pending_crossing.is_some());
        assert!(state.pending_crossing_choice.is_none());
    }

    #[test]
    fn process_encounter_flow_records_partial_travel() {
        let encounter = Encounter {
            id: String::from("sample"),
            name: String::from("Sample"),
            desc: String::new(),
            weight: 5,
            regions: vec![String::from("heartland")],
            modes: vec![String::from("classic")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        let data = EncounterData::from_encounters(vec![encounter]);
        let bundle = Rc::new(RngBundle::from_user_seed(41));

        let mut state = GameState {
            data: Some(data),
            region: Region::Heartland,
            encounter_chance_today: 1.0,
            features: FeatureFlags {
                travel_v2: true,
                ..FeatureFlags::default()
            },
            ..GameState::default()
        };
        state.attach_rng_bundle(bundle.clone());
        state.start_of_day();
        state.distance_today = 20.0;
        state.distance_today_raw = 20.0;
        state.partial_distance_today = 2.0;

        let outcome = state.process_encounter_flow(Some(&bundle), false);
        state.apply_encounter_partial_travel();

        assert!(outcome.is_some());
        assert!(state.current_encounter.is_some());
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_PARTIAL));
        let record = state
            .current_day_record
            .as_ref()
            .expect("expected day record");
        assert!(matches!(record.kind, TravelDayKind::Partial));
    }

    #[test]
    fn otdeluxe_crossing_costs_losses_and_drownings() {
        let mut state = GameState::default();
        state.ot_deluxe.inventory.cash_cents = 600;
        state.ot_deluxe.inventory.clothes_sets = 5;
        state.ot_deluxe.inventory.food_lbs = 100;
        state.ot_deluxe.inventory.bullets = 50;
        state.ot_deluxe.inventory.spares_wheels = 2;
        state.ot_deluxe.inventory.spares_axles = 2;
        state.ot_deluxe.inventory.spares_tongues = 2;
        let policy = default_otdeluxe_policy();

        state.apply_otdeluxe_crossing_costs(policy, OtDeluxeCrossingMethod::Ferry);
        assert_eq!(state.ot_deluxe.inventory.cash_cents, 100);

        state.apply_otdeluxe_crossing_costs(policy, OtDeluxeCrossingMethod::Guide);
        assert_eq!(state.ot_deluxe.inventory.clothes_sets, 2);

        let losses = state.apply_otdeluxe_crossing_losses(0.5);
        assert!(losses.food_lbs > 0);
        assert!(losses.bullets > 0);

        state.ot_deluxe.party.members = vec![
            OtDeluxePartyMember::new("A"),
            OtDeluxePartyMember::new("B"),
            OtDeluxePartyMember::new("C"),
        ];
        let drowned = state.apply_otdeluxe_drownings(&[1, 2]);
        assert_eq!(drowned, 2);
        assert!(!state.ot_deluxe.party.members[1].alive);
        assert!(!state.ot_deluxe.party.members[2].alive);
    }

    #[test]
    fn select_drowning_indices_handles_empty_and_counts() {
        let mut rng = SmallRng::seed_from_u64(9);
        let empty = GameState::select_drowning_indices(&mut rng, &[], 2);
        assert!(empty.is_empty());

        let alive = vec![0, 1, 2, 3];
        let selected = GameState::select_drowning_indices(&mut rng, &alive, 2);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn region_by_miles_maps_thresholds() {
        assert_eq!(GameState::region_by_miles(0.0), Region::Heartland);
        assert_eq!(GameState::region_by_miles(700.0), Region::RustBelt);
        assert_eq!(GameState::region_by_miles(1500.0), Region::Beltway);
    }

    #[test]
    fn pace_diet_and_policy_parse_roundtrip() {
        assert_eq!(PaceId::from_str("steady"), Ok(PaceId::Steady));
        assert!(PaceId::from_str("unknown").is_err());
        assert_eq!(PaceId::Steady.to_string(), "steady");

        assert_eq!(DietId::from_str("mixed"), Ok(DietId::Mixed));
        assert!(DietId::from_str("unknown").is_err());
        assert_eq!(DietId::Mixed.to_string(), "mixed");

        assert_eq!(PolicyKind::from_str("balanced"), Ok(PolicyKind::Balanced));
        assert!(PolicyKind::from_str("unknown").is_err());
        assert_eq!(PolicyKind::Balanced.to_string(), "balanced");
    }

    #[test]
    fn set_ending_does_not_override_existing() {
        let mut state = GameState::default();
        state.set_ending(Ending::BossVictory);
        state.set_ending(Ending::SanityLoss);
        assert_eq!(state.ending, Some(Ending::BossVictory));
    }

    #[test]
    fn journey_score_sums_expected_components() {
        let state = GameState {
            stats: Stats {
                supplies: 3,
                hp: 4,
                morale: 2,
                credibility: 1,
                allies: 1,
                ..Stats::default()
            },
            day: 5,
            encounters_resolved: 2,
            receipts: vec![String::from("a"), String::from("b")],
            vehicle_breakdowns: 1,
            ..GameState::default()
        };

        let expected = 3 * 10 + 4 * 50 + 2 * 25 + 15 + 5 + 4 * 4 + 2 * 6 + 2 * 8 - 12;
        assert_eq!(state.journey_score(), expected);
    }

    #[test]
    fn encounter_unique_ratio_handles_empty_and_recent() {
        let mut state = GameState::default();
        assert!((state.encounter_unique_ratio(0) - 1.0).abs() <= f32::EPSILON);

        state.recent_encounters.push_back(RecentEncounter::new(
            String::from("alpha"),
            state.day,
            Region::Heartland,
        ));
        state.recent_encounters.push_back(RecentEncounter::new(
            String::from("alpha"),
            state.day,
            Region::Heartland,
        ));
        let ratio = state.encounter_unique_ratio(10);
        assert!(ratio < 1.0);
    }

    #[test]
    fn mark_damage_sets_last_damage() {
        let mut state = GameState::default();
        state.mark_damage(DamageCause::ExposureHeat);
        assert_eq!(state.last_damage, Some(DamageCause::ExposureHeat));
    }

    #[test]
    fn add_day_reason_tag_tracks_camp_and_repair() {
        let mut state = GameState::default();
        state.start_of_day();
        state.add_day_reason_tag("camp");
        state.add_day_reason_tag("repair");
        state.add_day_reason_tag("camp");

        assert_eq!(state.days_with_camp, 1);
        assert_eq!(state.days_with_repair, 1);
        assert!(
            state
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "camp")
        );
        assert!(
            state
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "repair")
        );
    }

    #[test]
    fn apply_travel_progress_sets_boss_ready_on_finish() {
        let mut state = GameState {
            trail_distance: 10.0,
            miles_traveled_actual: 9.0,
            miles_traveled: 9.0,
            ..GameState::default()
        };

        let credited = state.apply_travel_progress(5.0, TravelProgressKind::Full);

        assert!(credited > 0.0);
        assert!(state.boss.readiness.ready);
        assert!(state.boss.readiness.reached);
    }

    #[test]
    fn apply_travel_progress_sets_otdeluxe_ending_on_finish() {
        let policy = default_otdeluxe_policy();
        let total = crate::otdeluxe_trail::total_miles_for_variant(
            &policy.trail,
            OtDeluxeTrailVariant::Main,
        );
        let near_end = f32::from(total.saturating_sub(1));
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            miles_traveled_actual: near_end,
            miles_traveled: near_end,
            ..GameState::default()
        };

        let credited = state.apply_travel_progress(5.0, TravelProgressKind::Full);

        assert!(credited > 0.0);
        assert_eq!(state.ending, Some(Ending::BossVictory));
    }

    #[test]
    fn otdeluxe_route_prompt_handles_decisions() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::SubletteCutoff));
        assert_eq!(
            state.ot_deluxe.route.variant,
            OtDeluxeTrailVariant::SubletteCutoff
        );

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesShortcut);
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::DallesShortcut));
        assert_eq!(
            state.ot_deluxe.route.variant,
            OtDeluxeTrailVariant::SubletteAndDallesShortcut
        );

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesFinal);
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::RaftColumbia));
        assert_eq!(
            state.ot_deluxe.route.dalles_choice,
            Some(OtDeluxeDallesChoice::Raft)
        );
    }

    #[test]
    fn otdeluxe_crossing_context_rejects_unavailable_method() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        state.ot_deluxe.crossing.choice_pending = true;
        state.ot_deluxe.crossing.chosen_method = Some(OtDeluxeCrossingMethod::Guide);
        state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 200.0,
            depth_ft: 3.0,
            swiftness: 1.1,
            bed: OtDeluxeRiverBed::Muddy,
        });
        state.ot_deluxe.inventory.clothes_sets = 0;

        let ctx = state.otdeluxe_crossing_context(OtDeluxeCrossingMethod::Guide);

        assert!(ctx.is_none());
        assert!(state.ot_deluxe.crossing.chosen_method.is_none());
    }

    #[test]
    fn otdeluxe_crossing_state_persists_until_resolved() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        let policy = default_otdeluxe_policy();
        let river = otdeluxe_crossings::river_for_index(0).expect("expected river");
        let node_index = otdeluxe_crossings::node_index_for_river(river);
        let marker = otdeluxe_trail::mile_marker_for_node(
            &policy.trail,
            state.ot_deluxe.route.variant,
            node_index,
        )
        .expect("expected mile marker");
        let marker_miles = f32::from(marker);
        state.miles_traveled_actual = (marker_miles - 1.0).max(0.0);

        let first = state.handle_otdeluxe_crossing_event(2.0);
        assert!(matches!(
            first,
            Some((false, ref log)) if log == LOG_TRAVEL_BLOCKED
        ));
        let river_state = state
            .ot_deluxe
            .crossing
            .river
            .clone()
            .expect("expected river state");

        state.ot_deluxe.weather.rain_accum = 999.0;
        state.ot_deluxe.season = Season::Winter;

        let second = state.handle_otdeluxe_crossing_event(2.0);
        assert!(matches!(
            second,
            Some((false, ref log)) if log == LOG_TRAVEL_BLOCKED
        ));
        assert!(state.ot_deluxe.crossing.choice_pending);
        assert_eq!(state.ot_deluxe.crossing.river_kind, Some(river));
        assert_eq!(state.ot_deluxe.crossing.river.as_ref(), Some(&river_state));
    }

    #[test]
    fn build_otdeluxe_state_truncates_extra_names() {
        let mut state = GameState::default();
        state.party.leader = String::from("Leader");
        state.party.companions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
            String::from("E"),
        ];

        let ot_state = state.build_ot_deluxe_state_from_legacy();

        assert_eq!(ot_state.party.members.len(), 5);
        assert_eq!(ot_state.party.members[0].name, "Leader");
        assert_eq!(ot_state.party.members[4].name, "D");
    }

    #[test]
    fn rng_accessors_return_none_without_bundle() {
        let mut state = GameState::default();
        assert!(state.events_rng().is_none());
        assert!(state.breakdown_rng().is_none());
        assert!(state.crossing_rng().is_none());
        assert!(state.boss_rng().is_none());

        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(5)));
        assert!(state.events_rng().is_some());
        assert!(state.breakdown_rng().is_some());
        assert!(state.crossing_rng().is_some());
        assert!(state.boss_rng().is_some());
    }

    #[test]
    fn journey_factors_apply_defaults() {
        let mut state = GameState::default();
        state.journey_breakdown.pace_factor.clear();
        state.weather_effects.breakdown_mult = -1.0;
        state.journey_wear.fatigue_k = 0.0;

        assert!((state.journey_pace_factor() - 1.0).abs() <= f32::EPSILON);
        assert!((state.journey_weather_factor() - 1.0).abs() <= f32::EPSILON);
        assert!((state.journey_fatigue_multiplier() - 1.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn with_seed_resets_state_fields() {
        let mut state = GameState {
            day: 10,
            ..GameState::default()
        };
        state.intent.pending = DayIntent::Rest;
        state
            .day_records
            .push(DayRecord::new(1, TravelDayKind::Travel, 12.0));
        let data = EncounterData::from_encounters(Vec::new());

        let seeded = state.with_seed(42, GameMode::Classic, data);

        assert_eq!(seeded.day_records.len(), 0);
        assert_eq!(seeded.seed, 42);
        assert!(matches!(seeded.intent.pending, DayIntent::Continue));
        assert!(seeded.logs.iter().any(|log| log == "log.seed-set"));
        assert!(seeded.data.is_some());
    }

    #[test]
    fn apply_choice_applies_receipts_logs_and_travel_bonus_baselines() {
        let effects = Effects {
            hp: -2,
            sanity: 1,
            add_receipt: Some(String::from("receipt.new")),
            use_receipt: true,
            log: Some(String::from("log.choice.test")),
            travel_bonus_ratio: 0.25,
            rest: true,
            ..Effects::default()
        };
        let encounter = encounter_with_choice(effects);

        let mut state = GameState::default();
        state.start_of_day();
        state.receipts.push(String::from("receipt.old"));
        state.distance_today = 4.0;
        state.current_encounter = Some(encounter.clone());
        state.apply_choice(0);
        assert!(state.current_encounter.is_none());
        assert!(state.logs.iter().any(|log| log == "log.choice.test"));
        assert!(state.day_state.rest.rest_requested);
        assert_eq!(state.receipts, vec![String::from("receipt.old")]);

        let mut raw_state = GameState::default();
        raw_state.start_of_day();
        raw_state.distance_today_raw = 3.0;
        raw_state.current_encounter = Some(encounter.clone());
        raw_state.apply_choice(0);

        let mut v2_state = GameState::default();
        v2_state.start_of_day();
        v2_state.features.travel_v2 = true;
        v2_state.current_encounter = Some(encounter.clone());
        v2_state.apply_choice(0);

        let mut classic_state = GameState::default();
        classic_state.start_of_day();
        classic_state.current_encounter = Some(encounter);
        classic_state.apply_choice(0);
    }

    #[test]
    fn party_and_choice_setters_fill_and_store_values() {
        let mut state = GameState::default();
        state.set_party("Leader", vec!["A"]);
        assert_eq!(state.party.leader, "Leader");
        assert_eq!(state.party.companions.len(), 4);
        assert_eq!(state.party.companions[0], "A");
        assert!(state.logs.iter().any(|log| log == "log.party.updated"));

        state.set_crossing_choice(CrossingChoice::Bribe);
        assert_eq!(state.pending_crossing_choice, Some(CrossingChoice::Bribe));
        state.set_otdeluxe_crossing_choice(OtDeluxeCrossingMethod::Ferry);
        assert_eq!(
            state.ot_deluxe.crossing.chosen_method,
            Some(OtDeluxeCrossingMethod::Ferry)
        );
        state.set_route_prompt_choice(OtDeluxeRouteDecision::SubletteCutoff);
        assert_eq!(
            state.pending_route_choice,
            Some(OtDeluxeRouteDecision::SubletteCutoff)
        );
    }

    #[test]
    fn otdeluxe_affliction_duration_and_fatality_modifiers() {
        let mut policy = OtDeluxe90sPolicy::default();
        policy.affliction.illness_duration_days = 0;
        policy.affliction.injury_duration_days = 0;
        assert_eq!(
            otdeluxe_affliction_duration(OtDeluxeAfflictionKind::Illness, &policy.affliction),
            1
        );
        assert_eq!(
            otdeluxe_affliction_duration(OtDeluxeAfflictionKind::Injury, &policy.affliction),
            1
        );

        let model = FatalityModel {
            base_prob_per_day: 0.2,
            prob_modifiers: vec![
                FatalityModifier::Constant { mult: 0.9 },
                FatalityModifier::HealthLabel {
                    good: 0.5,
                    fair: 1.0,
                    poor: 1.5,
                    very_poor: 2.0,
                },
                FatalityModifier::Pace {
                    steady: 0.8,
                    strenuous: 1.1,
                    grueling: 1.3,
                },
                FatalityModifier::Rations {
                    filling: 0.7,
                    meager: 1.0,
                    bare_bones: 1.2,
                },
                FatalityModifier::Weather {
                    weather: Weather::Storm,
                    mult: 0.6,
                },
            ],
            apply_doctor_mult: true,
        };
        let bounds = policy.health.label_ranges;
        let contexts = [
            OtDeluxeFatalityContext {
                health_general: bounds.good_max,
                pace: OtDeluxePace::Steady,
                rations: OtDeluxeRations::Filling,
                weather: Weather::Storm,
                occupation: Some(OtDeluxeOccupation::Doctor),
            },
            OtDeluxeFatalityContext {
                health_general: bounds.fair_max,
                pace: OtDeluxePace::Strenuous,
                rations: OtDeluxeRations::Meager,
                weather: Weather::Clear,
                occupation: None,
            },
            OtDeluxeFatalityContext {
                health_general: bounds.poor_max,
                pace: OtDeluxePace::Grueling,
                rations: OtDeluxeRations::BareBones,
                weather: Weather::Storm,
                occupation: None,
            },
            OtDeluxeFatalityContext {
                health_general: bounds.poor_max.saturating_add(1),
                pace: OtDeluxePace::Steady,
                rations: OtDeluxeRations::Filling,
                weather: Weather::Clear,
                occupation: None,
            },
        ];
        for context in contexts {
            let prob = otdeluxe_fatality_probability(&model, context, &policy);
            assert!((0.0..=1.0).contains(&prob));
        }

        let mut rng = SmallRng::seed_from_u64(11);
        let always = FatalityModel {
            base_prob_per_day: 1.0,
            prob_modifiers: Vec::new(),
            apply_doctor_mult: false,
        };
        assert!(otdeluxe_roll_disease_fatality(
            &always,
            &mut rng,
            contexts[0],
            &policy
        ));
        let mut rng = SmallRng::seed_from_u64(11);
        let never = FatalityModel {
            base_prob_per_day: 0.0,
            prob_modifiers: Vec::new(),
            apply_doctor_mult: false,
        };
        assert!(!otdeluxe_roll_disease_fatality(
            &never,
            &mut rng,
            contexts[0],
            &policy
        ));
    }

    #[test]
    fn apply_u8_delta_handles_zero_and_clamps() {
        let mut value = 10_u8;
        assert_eq!(GameState::apply_u8_delta(&mut value, 0), 0);
        assert_eq!(value, 10);
        assert_eq!(GameState::apply_u8_delta(&mut value, 5), 5);
        assert_eq!(value, 15);
        assert_eq!(GameState::apply_u8_delta(&mut value, -20), -15);
        assert_eq!(value, 0);
        let mut high = u8::MAX;
        assert_eq!(GameState::apply_u8_delta(&mut high, 10), 0);
        assert_eq!(high, u8::MAX);
    }

    #[test]
    fn apply_stop_ratio_floor_converts_non_travel_when_ratio_low() {
        let mut state = GameState::default();
        state.start_of_day();
        state.travel_days = 1;
        state.partial_travel_days = 0;
        state.non_travel_days = 3;
        state.distance_today = 10.0;
        state.distance_today_raw = 10.0;
        let kind = state.apply_stop_ratio_floor(TravelDayKind::NonTravel);
        assert_eq!(kind, TravelDayKind::Partial);
        assert!(state.partial_distance_today > 0.0);
    }

    #[test]
    fn otdeluxe_random_event_selection_covers_event_types() {
        let events = [
            ("weather_catastrophe", "blizzard"),
            ("resource_shortage", "bad_water"),
            ("party_incident", "lost_member"),
            ("oxen_incident", "ox_wandered_off"),
            ("resource_change", "abandoned_wagon_supplies"),
            ("wagon_part_break", "repairable"),
            ("travel_hazard", "rough_trail"),
        ];
        for (event_id, variant) in events {
            let mut state = otdeluxe_state_with_party();
            let mut rng = SmallRng::seed_from_u64(7);
            let selection = make_event_selection(event_id, variant);
            assert!(
                state
                    .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                    .is_some()
            );
        }
    }

    #[test]
    fn otdeluxe_random_event_selection_rejects_unknown_event() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(9);
        let selection = make_event_selection("unknown_event", "unknown_variant");
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn otdeluxe_random_event_variant_branches_cover_losses() {
        let mut rng = SmallRng::seed_from_u64(5);

        let mut weather_state = otdeluxe_state_with_party();
        assert!(
            weather_state
                .apply_otdeluxe_random_weather_catastrophe("hailstorm", 0.1, 0.2)
                .is_some()
        );
        assert!(
            weather_state
                .apply_otdeluxe_random_weather_catastrophe("thunderstorm", 0.1, 0.2)
                .is_some()
        );
        assert!(
            weather_state
                .apply_otdeluxe_random_weather_catastrophe("heavy_fog", 0.1, 0.2)
                .is_some()
        );

        let mut shortage_state = otdeluxe_state_with_party();
        assert!(
            shortage_state
                .apply_otdeluxe_random_resource_shortage("bad_water", 0.1, 0.2, &mut rng)
                .is_some()
        );
        assert!(
            shortage_state
                .apply_otdeluxe_random_resource_shortage("no_water", 0.1, 0.2, &mut rng)
                .is_some()
        );
        shortage_state.ot_deluxe.oxen.healthy = 1;
        shortage_state.ot_deluxe.oxen.sick = 0;
        assert!(
            shortage_state
                .apply_otdeluxe_random_resource_shortage("no_grass", 0.1, 0.2, &mut rng)
                .is_some()
        );

        let mut party_state = otdeluxe_state_with_party();
        assert!(
            party_state
                .apply_otdeluxe_random_party_incident("lost_member", 0.1, 0.2, &mut rng)
                .is_some()
        );
        let mut snake_state = otdeluxe_state_with_party();
        assert!(
            snake_state
                .apply_otdeluxe_random_party_incident("snakebite", 0.1, 0.2, &mut rng)
                .is_some()
        );

        let mut resource_state = otdeluxe_state_with_party();
        assert!(
            resource_state
                .apply_otdeluxe_random_resource_change("fire", 0.1, 0.2)
                .is_some()
        );
        assert!(
            resource_state
                .apply_otdeluxe_random_resource_change("abandoned_wagon_empty", 0.1, 0.2)
                .is_some()
        );
        assert!(
            resource_state
                .apply_otdeluxe_random_resource_change("thief", 0.1, 0.2)
                .is_some()
        );
        assert!(
            resource_state
                .apply_otdeluxe_random_resource_change("mutual_aid_food", 0.1, 0.2)
                .is_some()
        );
        assert!(
            resource_state
                .apply_otdeluxe_random_resource_change("gravesite", 0.1, 0.2)
                .is_some()
        );

        let mut wagon_state = otdeluxe_state_with_party();
        wagon_state.ot_deluxe.inventory.spares_wheels = 0;
        wagon_state.ot_deluxe.inventory.spares_axles = 0;
        wagon_state.ot_deluxe.inventory.spares_tongues = 0;
        assert!(
            wagon_state
                .apply_otdeluxe_random_wagon_part_break("unrepairable", 0.1, 0.2, &mut rng)
                .is_some()
        );
        assert!(
            wagon_state
                .apply_otdeluxe_random_wagon_part_break("replaceable", 0.1, 0.2, &mut rng)
                .is_some()
        );
    }

    #[test]
    fn otdeluxe_daily_afflictions_cover_sick_and_injury() {
        let fatal = FatalityModel {
            base_prob_per_day: 1.0,
            prob_modifiers: Vec::new(),
            apply_doctor_mult: false,
        };
        let diseases = vec![
            DiseaseDef {
                id: String::from("illness"),
                kind: DiseaseKind::Illness,
                display_key: String::from("disease.illness"),
                weight: 1,
                duration_days: Some(2),
                onset_effects: DiseaseEffects::default(),
                daily_tick_effects: DiseaseEffects {
                    health_general_delta: -2,
                    food_lbs_delta: -3,
                    travel_speed_mult: 0.8,
                },
                fatality_model: Some(FatalityModel {
                    base_prob_per_day: 0.0,
                    prob_modifiers: Vec::new(),
                    apply_doctor_mult: false,
                }),
                tags: Vec::new(),
            },
            DiseaseDef {
                id: String::from("injury"),
                kind: DiseaseKind::Injury,
                display_key: String::from("disease.injury"),
                weight: 1,
                duration_days: Some(2),
                onset_effects: DiseaseEffects::default(),
                daily_tick_effects: DiseaseEffects {
                    health_general_delta: -1,
                    food_lbs_delta: -1,
                    travel_speed_mult: 0.9,
                },
                fatality_model: Some(fatal),
                tags: Vec::new(),
            },
        ];
        let catalog = DiseaseCatalog { diseases };
        let policy = OtDeluxe90sPolicy::default();
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.health_general = 50;
        state.ot_deluxe.inventory.food_lbs = 40;
        state.ot_deluxe.party.members = vec![
            {
                let mut member = OtDeluxePartyMember::new("Sick");
                member.sick_days_remaining = 2;
                member.illness_id = Some(String::from("illness"));
                member
            },
            {
                let mut member = OtDeluxePartyMember::new("Injured");
                member.injured_days_remaining = 2;
                member.injury_id = Some(String::from("injury"));
                member
            },
        ];

        let mut rng = SmallRng::seed_from_u64(2);
        state.apply_otdeluxe_daily_afflictions(&catalog, &mut rng, &policy);
        assert!(
            state
                .ot_deluxe
                .party
                .members
                .iter()
                .any(|member| !member.alive)
        );
        assert!(state.ot_deluxe.inventory.food_lbs < 40);
        assert!(state.ot_deluxe.travel.disease_speed_mult <= 1.0);
    }

    #[test]
    fn otdeluxe_death_imminent_progression_and_reset() {
        let mut state = otdeluxe_state_with_party();
        let mut health_policy = OtDeluxe90sPolicy::default().health;
        health_policy.death_imminent_grace_days = 1;
        health_policy.death_imminent_resets_on_recovery_below_threshold = true;
        state.ot_deluxe.health_general = health_policy.death_threshold;
        state.ot_deluxe.death_imminent_days_remaining = 0;
        state.update_otdeluxe_death_imminent(&health_policy);
        assert_eq!(state.ot_deluxe.death_imminent_days_remaining, 1);

        state.update_otdeluxe_death_imminent(&health_policy);
        assert_eq!(state.ot_deluxe.death_imminent_days_remaining, 0);
        assert!(state.ending.is_some());

        state.ot_deluxe.health_general = health_policy.death_threshold.saturating_sub(1);
        state.ot_deluxe.death_imminent_days_remaining = 2;
        state.ending = None;
        state.update_otdeluxe_death_imminent(&health_policy);
        assert_eq!(state.ot_deluxe.death_imminent_days_remaining, 0);
    }

    #[test]
    fn lose_random_spare_covers_all_spare_types() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(1);

        state.ot_deluxe.inventory.spares_wheels = 1;
        state.ot_deluxe.inventory.spares_axles = 0;
        state.ot_deluxe.inventory.spares_tongues = 0;
        assert_eq!(state.lose_random_spare(&mut rng), Some("wheel"));

        state.ot_deluxe.inventory.spares_wheels = 0;
        state.ot_deluxe.inventory.spares_axles = 1;
        state.ot_deluxe.inventory.spares_tongues = 0;
        assert_eq!(state.lose_random_spare(&mut rng), Some("axle"));

        state.ot_deluxe.inventory.spares_wheels = 0;
        state.ot_deluxe.inventory.spares_axles = 0;
        state.ot_deluxe.inventory.spares_tongues = 1;
        assert_eq!(state.lose_random_spare(&mut rng), Some("tongue"));

        state.ot_deluxe.inventory.spares_wheels = 0;
        state.ot_deluxe.inventory.spares_axles = 0;
        state.ot_deluxe.inventory.spares_tongues = 0;
        assert!(state.lose_random_spare(&mut rng).is_none());
    }

    #[test]
    fn otdeluxe_oxen_loss_helpers_cover_all_paths() {
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.oxen.healthy = 1;
        state.ot_deluxe.oxen.sick = 0;
        assert_eq!(state.apply_otdeluxe_no_grass_loss(), (-1, 1));
        state.ot_deluxe.oxen.healthy = 0;
        state.ot_deluxe.oxen.sick = 1;
        assert_eq!(state.apply_otdeluxe_no_grass_loss(), (0, -1));
        state.ot_deluxe.oxen.healthy = 0;
        state.ot_deluxe.oxen.sick = 0;
        assert_eq!(state.apply_otdeluxe_no_grass_loss(), (0, 0));

        state.ot_deluxe.oxen.healthy = 1;
        state.ot_deluxe.oxen.sick = 0;
        assert_eq!(state.apply_otdeluxe_oxen_wander(), (-1, 0));
        state.ot_deluxe.oxen.healthy = 0;
        state.ot_deluxe.oxen.sick = 1;
        assert_eq!(state.apply_otdeluxe_oxen_wander(), (0, -1));
        state.ot_deluxe.oxen.healthy = 0;
        state.ot_deluxe.oxen.sick = 0;
        assert_eq!(state.apply_otdeluxe_oxen_wander(), (0, 0));

        state.ot_deluxe.oxen.healthy = 1;
        state.ot_deluxe.oxen.sick = 0;
        assert_eq!(state.apply_otdeluxe_oxen_sickness(), (-1, 1));
        state.ot_deluxe.oxen.healthy = 0;
        state.ot_deluxe.oxen.sick = 1;
        assert_eq!(state.apply_otdeluxe_oxen_sickness(), (0, -1));
        state.ot_deluxe.oxen.healthy = 0;
        state.ot_deluxe.oxen.sick = 0;
        assert_eq!(state.apply_otdeluxe_oxen_sickness(), (0, 0));
    }

    #[test]
    fn check_vehicle_terminal_state_covers_recovery_and_failures() {
        let mut spare_state = GameState::default();
        spare_state.start_of_day();
        spare_state.vehicle.health = 0.0;
        spare_state.inventory.spares.tire = 1;
        assert!(!spare_state.check_vehicle_terminal_state());
        assert_eq!(spare_state.inventory.spares.tire, 0);

        let mut budget_state = GameState::default();
        budget_state.start_of_day();
        budget_state.vehicle.health = -1.0;
        budget_state.budget_cents = EMERGENCY_REPAIR_COST + 100;
        assert!(!budget_state.check_vehicle_terminal_state());
        assert!(budget_state.budget_cents < EMERGENCY_REPAIR_COST + 100);

        let mut limp_state = GameState::default();
        limp_state.start_of_day();
        limp_state.vehicle.health = -1.0;
        limp_state.vehicle_breakdowns = 0;
        limp_state.budget_cents = 0;
        limp_state.budget = 0;
        assert!(!limp_state.check_vehicle_terminal_state());
        assert!(limp_state.distance_today >= 0.0);

        let mut guard_state = GameState::default();
        guard_state.start_of_day();
        guard_state.vehicle.health = -1.0;
        guard_state.endgame.active = true;
        guard_state.endgame.failure_guard_miles = 2_000.0;
        guard_state.miles_traveled_actual = 100.0;
        assert!(!guard_state.check_vehicle_terminal_state());

        let mut classic_guard = GameState::default();
        classic_guard.start_of_day();
        classic_guard.mode = GameMode::Classic;
        classic_guard.policy = Some(PolicyKind::Balanced);
        classic_guard.vehicle.health = -1.0;
        classic_guard.vehicle_breakdowns = 999;
        classic_guard.miles_traveled_actual = 10.0;
        classic_guard.budget_cents = 0;
        classic_guard.budget = 0;
        assert!(!classic_guard.check_vehicle_terminal_state());

        let mut aggressive_guard = GameState::default();
        aggressive_guard.start_of_day();
        aggressive_guard.mode = GameMode::Deep;
        aggressive_guard.policy = Some(PolicyKind::Aggressive);
        aggressive_guard.vehicle.health = -1.0;
        aggressive_guard.vehicle_breakdowns = 999;
        aggressive_guard.miles_traveled_actual = 1_600.0;
        aggressive_guard.budget_cents = 0;
        aggressive_guard.budget = 0;
        aggressive_guard.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        assert!(!aggressive_guard.check_vehicle_terminal_state());

        let mut limp_guard = GameState::default();
        limp_guard.start_of_day();
        limp_guard.mode = GameMode::Deep;
        limp_guard.policy = Some(PolicyKind::Balanced);
        limp_guard.vehicle.health = -1.0;
        limp_guard.vehicle_breakdowns = 999;
        limp_guard.miles_traveled_actual = 1_900.0;
        limp_guard.budget_cents = 0;
        limp_guard.budget = 0;
        assert!(!limp_guard.check_vehicle_terminal_state());

        let mut deep_balanced = GameState::default();
        deep_balanced.start_of_day();
        deep_balanced.mode = GameMode::Deep;
        deep_balanced.policy = Some(PolicyKind::Balanced);
        deep_balanced.vehicle.health = -1.0;
        deep_balanced.vehicle_breakdowns = 999;
        deep_balanced.miles_traveled_actual = 1_000.0;
        deep_balanced.budget_cents = 0;
        deep_balanced.budget = 0;
        assert!(!deep_balanced.check_vehicle_terminal_state());

        let mut terminal = GameState::default();
        terminal.start_of_day();
        terminal.mode = GameMode::Deep;
        terminal.policy = Some(PolicyKind::Conservative);
        terminal.vehicle.health = -1.0;
        terminal.vehicle_breakdowns = 999;
        terminal.miles_traveled_actual = 1_200.0;
        terminal.budget_cents = 0;
        terminal.budget = 0;
        assert!(terminal.check_vehicle_terminal_state());
        assert!(terminal.ending.is_some());
    }

    #[test]
    fn otdeluxe_helper_branches_cover_edges() {
        let policy = OtDeluxe90sPolicy::default();
        assert_eq!(
            otdeluxe_pace_health_penalty(OtDeluxePace::Strenuous, &policy.pace_health_penalty),
            policy.pace_health_penalty.strenuous
        );
        assert!(
            (otdeluxe_pace_food_multiplier(OtDeluxePace::Strenuous, &policy)
                - policy.pace_mult_strenuous)
                .abs()
                <= f32::EPSILON
        );
        assert_eq!(
            otdeluxe_rations_health_penalty(OtDeluxeRations::Meager, &policy.rations),
            policy.rations.health_penalty[1]
        );
        assert!((otdeluxe_snow_speed_mult(f32::NAN, &policy.travel) - 1.0).abs() <= f32::EPSILON);

        let mut health_policy = policy.health;
        health_policy.clothing_penalty_winter = 0;
        health_policy.clothing_sets_per_person = 2;
        let mut inventory = OtDeluxeInventory {
            clothes_sets: 0,
            ..OtDeluxeInventory::default()
        };
        assert_eq!(
            otdeluxe_clothing_health_penalty(Season::Winter, &inventory, 1, &health_policy),
            0
        );

        health_policy.clothing_penalty_winter = 5;
        health_policy.clothing_sets_per_person = 1;
        inventory.clothes_sets = 2;
        assert_eq!(
            otdeluxe_clothing_health_penalty(Season::Winter, &inventory, 1, &health_policy),
            0
        );
        inventory.clothes_sets = 0;
        assert_eq!(
            otdeluxe_clothing_health_penalty(Season::Winter, &inventory, 1, &health_policy),
            5
        );

        health_policy.drought_threshold = 1.0;
        health_policy.drought_penalty = 3;
        assert_eq!(otdeluxe_drought_health_penalty(0.5, &health_policy), 3);
        assert_eq!(otdeluxe_drought_health_penalty(2.0, &health_policy), 0);
    }

    #[test]
    fn otdeluxe_navigation_helpers_cover_edges() {
        assert_eq!(
            otdeluxe_spare_for_breakdown(Part::Alternator),
            OtDeluxeSparePart::Tongue
        );
        assert_eq!(
            otdeluxe_navigation_reason_tag(OtDeluxeNavigationEvent::WrongTrail),
            "otdeluxe.nav_wrong"
        );
        assert_eq!(
            otdeluxe_navigation_reason_tag(OtDeluxeNavigationEvent::Impassable),
            "otdeluxe.nav_impassable"
        );
        assert_eq!(
            otdeluxe_navigation_reason_tag(OtDeluxeNavigationEvent::Snowbound),
            "otdeluxe.nav_snowbound"
        );

        let delay_policy = OtDeluxeNavigationPolicy::default();
        assert_eq!(
            otdeluxe_navigation_delay_for(OtDeluxeNavigationEvent::WrongTrail, &delay_policy),
            delay_policy.wrong_delay
        );
        assert_eq!(
            otdeluxe_navigation_delay_for(OtDeluxeNavigationEvent::Impassable, &delay_policy),
            delay_policy.impassable_delay
        );
        assert_eq!(
            otdeluxe_navigation_delay_for(OtDeluxeNavigationEvent::Snowbound, &delay_policy),
            delay_policy.snowbound_delay
        );

        let delay_zero = OtDeluxeNavigationDelay {
            min_days: 2,
            max_days: 0,
        };
        let mut rng = SmallRng::seed_from_u64(1);
        assert_eq!(roll_otdeluxe_navigation_delay_days(delay_zero, &mut rng), 0);

        let delay_range = OtDeluxeNavigationDelay {
            min_days: 2,
            max_days: 4,
        };
        let mut rng = SmallRng::seed_from_u64(2);
        let days = roll_otdeluxe_navigation_delay_days(delay_range, &mut rng);
        assert!((2..=4).contains(&days));

        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 0.2,
            lost_weight: 1,
            wrong_weight: 1,
            impassable_weight: 1,
            snowbound_weight: 0,
            snowbound_min_depth_in: 100.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let seed = seed_for_roll_at_or_above(0.2);
        let mut rng = SmallRng::seed_from_u64(seed);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 0.0, &mut rng);
        assert!(event.is_none());
        assert!(trace.is_none());

        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 1,
            wrong_weight: 0,
            impassable_weight: 0,
            snowbound_weight: 0,
            snowbound_min_depth_in: 100.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = SmallRng::seed_from_u64(3);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 0.0, &mut rng);
        assert_eq!(event, Some(OtDeluxeNavigationEvent::LostTrail));
        assert!(trace.is_some());
    }

    #[test]
    fn otdeluxe_policy_overrides_and_legacy_mapping() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::DystrailLegacy,
            ..GameState::default()
        };
        assert_eq!(
            state.otdeluxe_policy_overrides(),
            OtDeluxePolicyOverride::default()
        );

        state.pace = PaceId::Heated;
        state.diet = DietId::Quiet;
        state.party.leader = String::from("Ada");
        let ot_state = state.build_ot_deluxe_state_from_legacy();
        assert_eq!(ot_state.pace, OtDeluxePace::Strenuous);
        assert_eq!(ot_state.rations, OtDeluxeRations::Meager);

        state.mechanical_policy = MechanicalPolicyId::DystrailLegacy;
        assert!(state.otdeluxe_next_prompt_marker().is_none());
    }

    #[test]
    fn otdeluxe_death_imminent_clamps_grace() {
        let mut state = otdeluxe_state_with_party();
        let policy = default_otdeluxe_policy();
        let grace = policy.health.death_imminent_grace_days;
        state.ot_deluxe.health_general = policy.health.death_threshold;
        state.ot_deluxe.death_imminent_days_remaining = grace.saturating_add(2);
        state.update_otdeluxe_death_imminent(&policy.health);
        assert_eq!(
            state.ot_deluxe.death_imminent_days_remaining,
            grace.saturating_sub(1)
        );
    }

    #[test]
    fn general_strain_handles_invalid_inputs() {
        let mut state = GameState {
            vehicle: crate::vehicle::Vehicle {
                wear: 5.0,
                ..crate::vehicle::Vehicle::default()
            },
            ..GameState::default()
        };
        let cfg = StrainConfig {
            vehicle_wear_norm_denom: 0.0,
            ..StrainConfig::default()
        };
        let strain = state.update_general_strain(&cfg);
        assert!(strain >= 0.0);

        let nan_cfg = StrainConfig {
            weights: StrainWeights {
                hp: f32::NAN,
                ..StrainWeights::default()
            },
            ..StrainConfig::default()
        };
        let strain = state.update_general_strain(&nan_cfg);
        assert!((strain - 0.0).abs() <= f32::EPSILON);

        let norm_cfg = StrainConfig {
            strain_norm_denom: 0.0,
            ..StrainConfig::default()
        };
        assert!((state.general_strain_norm(&norm_cfg) - 0.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn tick_exec_order_state_handles_behind_schedule() {
        let mut state = GameState {
            day: 80,
            miles_traveled_actual: 0.0,
            exec_order_cooldown: 0,
            ..GameState::default()
        };
        state.tick_exec_order_state();
        assert!(state.exec_order_cooldown <= crate::constants::EXEC_ORDER_MAX_COOLDOWN);
    }

    #[test]
    fn compute_day_progress_backfills_partial_record() {
        let mut state = GameState {
            day_state: DayState {
                lifecycle: LifecycleState {
                    day_initialized: true,
                    ..LifecycleState::default()
                },
                ..DayState::default()
            },
            current_day_record: Some(DayRecord::new(0, TravelDayKind::NonTravel, 0.0)),
            miles_traveled_actual: 10.0,
            prev_miles_traveled: 0.0,
            ..GameState::default()
        };
        let delta = state.compute_day_progress();
        assert!(delta > 0.0);
        assert!(matches!(
            state.current_day_kind,
            Some(TravelDayKind::Partial)
        ));
    }

    #[test]
    fn record_encounter_prunes_recent_history() {
        let mut state = GameState::default();
        state.start_of_day();
        for idx in 0..ENCOUNTER_RECENT_MEMORY {
            state.recent_encounters.push_back(RecentEncounter::new(
                format!("enc{idx}"),
                state.day,
                state.region,
            ));
        }
        state.record_encounter("new_encounter");
        assert!(state.recent_encounters.len() <= ENCOUNTER_RECENT_MEMORY);
    }

    #[test]
    fn travel_progress_and_wear_cover_edges() {
        let mut state = GameState {
            current_day_miles: 5.0,
            distance_today: 10.0,
            distance_today_raw: 8.0,
            ..GameState::default()
        };
        let before = state.vehicle.wear;
        state.apply_travel_wear_for_day(0.0);
        assert!(state.vehicle.wear >= before);

        let credited = state.apply_travel_progress(0.0, TravelProgressKind::Full);
        assert!((credited - 0.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn emergency_limp_guard_branches_cover_edges() {
        let mut otdeluxe = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        otdeluxe.start_of_day();
        assert!(!otdeluxe.try_emergency_limp_guard());

        let mut classic = GameState {
            mode: GameMode::Classic,
            policy: Some(PolicyKind::Balanced),
            miles_traveled_actual: 2_000.0,
            ..GameState::default()
        };
        classic.start_of_day();
        assert!(!classic.try_emergency_limp_guard());

        let mut low_miles = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 1_000.0,
            ..GameState::default()
        };
        low_miles.start_of_day();
        assert!(!low_miles.try_emergency_limp_guard());

        let mut eligible = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 2_000.0,
            endgame: crate::endgame::EndgameState {
                last_limp_mile: 0.0,
                ..crate::endgame::EndgameState::default()
            },
            ..GameState::default()
        };
        eligible.start_of_day();
        assert!(eligible.try_emergency_limp_guard());
    }

    #[test]
    fn deep_aggressive_field_repair_branches_cover_edges() {
        let mut otdeluxe = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        otdeluxe.start_of_day();
        assert!(!otdeluxe.try_deep_aggressive_field_repair());

        let mut fail_state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 2_000.0,
            ..GameState::default()
        };
        fail_state.start_of_day();
        fail_state.attach_rng_bundle(breakdown_bundle_with_roll_at_or_above(0.65));
        assert!(!fail_state.try_deep_aggressive_field_repair());

        let mut success_state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 2_000.0,
            ..GameState::default()
        };
        success_state.start_of_day();
        let seed = seed_for_roll_below(0.1);
        success_state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));
        assert!(success_state.try_deep_aggressive_field_repair());
    }

    #[test]
    fn encounter_penalty_and_diversity_cover_edges() {
        let state = GameState {
            features: FeatureFlags {
                encounter_diversity: false,
                ..FeatureFlags::default()
            },
            ..GameState::default()
        };
        assert!(!state.should_discourage_encounter("encounter"));

        let state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Conservative),
            ..GameState::default()
        };
        assert!(
            (state.encounter_reroll_penalty() - TRAVEL_RATIO_DEFAULT.min(WEATHER_DEFAULT_SPEED))
                .abs()
                <= f32::EPSILON
        );
    }

    #[test]
    fn roll_daily_illness_behind_schedule_halves_chance() {
        let mut state = GameState {
            day: 100,
            miles_traveled_actual: 0.0,
            ..GameState::default()
        };
        state.roll_daily_illness();
        assert!(state.disease_cooldown <= DISEASE_COOLDOWN_DAYS);
    }

    #[test]
    fn otdeluxe_affliction_roll_respects_probability() {
        let mut state = otdeluxe_state_with_party();
        let policy = OtDeluxe90sPolicy::default();
        state.ot_deluxe.health_general = 100;
        let mut rng = SmallRng::seed_from_u64(seed_for_roll_at_or_above(0.9));
        let result = state.roll_otdeluxe_affliction_with_catalog(
            &DiseaseCatalog::default(),
            &mut rng,
            &policy,
        );
        assert!(result.is_none());
    }

    #[test]
    fn otdeluxe_affliction_roll_injury_sets_ending() {
        let mut state = otdeluxe_state_with_party();
        let mut policy = OtDeluxe90sPolicy::default();
        policy.affliction.weight_illness = 0;
        policy.affliction.weight_injury = 1;
        policy.affliction.probability_max = 1.0;
        for point in &mut policy.affliction.curve_pwl {
            point.probability = 1.0;
        }
        state.ot_deluxe.health_general = 1;
        if let Some(member) = state.ot_deluxe.party.members.first_mut() {
            member.sick_days_remaining = 1;
        }
        let catalog = DiseaseCatalog {
            diseases: Vec::new(),
        };
        let mut rng = SmallRng::seed_from_u64(seed_for_roll_below(0.01));
        let result = state.roll_otdeluxe_affliction_with_catalog(&catalog, &mut rng, &policy);
        assert!(result.is_some());
        assert!(state.ending.is_some());
    }

    #[test]
    fn current_weather_speed_penalty_handles_extremes() {
        let cold_state = GameState {
            weather_state: crate::weather::WeatherState {
                today: Weather::ColdSnap,
                ..crate::weather::WeatherState::default()
            },
            ..GameState::default()
        };
        assert!(
            (cold_state.current_weather_speed_penalty() - WEATHER_COLD_SNAP_SPEED).abs()
                <= f32::EPSILON
        );

        let heat_state = GameState {
            weather_state: crate::weather::WeatherState {
                today: Weather::HeatWave,
                ..crate::weather::WeatherState::default()
            },
            ..GameState::default()
        };
        assert!(
            (heat_state.current_weather_speed_penalty() - WEATHER_HEAT_WAVE_SPEED).abs()
                <= f32::EPSILON
        );
    }

    #[test]
    fn travel_boosts_and_clamps_cover_edges() {
        let pace_cfg = PaceCfg {
            dist_mult: 1.0,
            ..PaceCfg::default()
        };
        let limits = PacingLimits::default();

        let mut behind = GameState {
            day: 80,
            miles_traveled_actual: 0.0,
            ..GameState::default()
        };
        let _ = behind.compute_miles_for_today(&pace_cfg, &limits);

        let (day_threshold, mile_threshold, _) = DEEP_CONSERVATIVE_BOOSTS[0];
        let mut boosted = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Conservative),
            day: day_threshold,
            miles_traveled_actual: mile_threshold - 10.0,
            ..GameState::default()
        };
        let _ = boosted.compute_miles_for_today(&pace_cfg, &limits);

        let mut nan_state = GameState {
            journey_travel: crate::journey::TravelConfig {
                mpd_base: f32::NAN,
                mpd_min: 0.0,
                mpd_max: 0.0,
                ..crate::journey::TravelConfig::default()
            },
            ..GameState::default()
        };
        let miles = nan_state.compute_miles_for_today(&pace_cfg, &limits);
        assert!(miles.is_finite());
    }

    #[test]
    fn otdeluxe_miles_for_today_overrides_cover_edges() {
        let mut state = otdeluxe_state_with_party();
        let policy = OtDeluxe90sPolicy::default();
        state.ot_deluxe.pace = OtDeluxePace::Strenuous;
        let _ = state.compute_otdeluxe_miles_for_today(&policy);

        let mut policy = OtDeluxe90sPolicy::default();
        let override_data = OtDeluxePolicyOverride {
            travel_multiplier: Some(-1.0),
            ..OtDeluxePolicyOverride::default()
        };
        policy
            .per_region_overrides
            .insert(state.region, override_data);
        state.ot_deluxe.pace = OtDeluxePace::Grueling;
        let _ = state.compute_otdeluxe_miles_for_today(&policy);
    }

    #[test]
    fn endgame_bias_and_malnutrition_penalty_cover_edges() {
        let state = GameState {
            endgame: crate::endgame::EndgameState {
                active: true,
                travel_bias: 1.5,
                ..crate::endgame::EndgameState::default()
            },
            ..GameState::default()
        };
        assert!((state.endgame_bias() - 1.5).abs() <= f32::EPSILON);

        let state = GameState {
            malnutrition_level: 2,
            ..GameState::default()
        };
        let penalty = state.malnutrition_penalty();
        assert!(penalty < 1.0);
    }

    #[test]
    fn crossing_helpers_cover_edges() {
        let mut state = GameState {
            day_state: DayState {
                lifecycle: LifecycleState {
                    day_initialized: true,
                    ..LifecycleState::default()
                },
                ..DayState::default()
            },
            current_day_record: Some(DayRecord::new(0, TravelDayKind::NonTravel, 0.0)),
            current_day_miles: 10.0,
            ..GameState::default()
        };
        assert!(matches!(
            state.crossing_kind_for_index(0),
            CrossingKind::Checkpoint
        ));
        assert!(matches!(
            state.crossing_kind_for_index(CROSSING_MILESTONES.len()),
            CrossingKind::BridgeOut
        ));

        state.apply_target_travel(TravelDayKind::Partial, 5.0, "test");
        assert!(state.current_day_miles <= 5.0);

        let mut crossing_state = GameState {
            miles_traveled_actual: CROSSING_MILESTONES[0],
            ..GameState::default()
        };
        let outcome = crossing_state.handle_crossing_event(0.0);
        assert!(outcome.is_some());
        assert!(crossing_state.pending_crossing.is_some());

        let mut pending_state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::Checkpoint,
                computed_miles_today: 10.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Permit),
            ..GameState::default()
        };
        let resolved = pending_state.resolve_pending_crossing_choice(CrossingChoice::Permit);
        assert!(resolved.is_none());

        let mut otdeluxe_state = otdeluxe_state_with_party();
        otdeluxe_state.ot_deluxe.crossing.choice_pending = true;
        otdeluxe_state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        otdeluxe_state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 50.0,
            depth_ft: 1.0,
            swiftness: 0.1,
            bed: OtDeluxeRiverBed::Muddy,
        });
        otdeluxe_state.ot_deluxe.inventory.cash_cents = 0;
        otdeluxe_state.ot_deluxe.inventory.clothes_sets = 0;
        let ctx = otdeluxe_state.otdeluxe_crossing_context(OtDeluxeCrossingMethod::CaulkFloat);
        assert!(ctx.is_none());

        let _ = state.sample_crossing_roll(0);
    }

    #[test]
    fn failure_log_key_covers_branches() {
        let mut vehicle_state = GameState {
            vehicle: crate::vehicle::Vehicle {
                health: 0.0,
                ..crate::vehicle::Vehicle::default()
            },
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Conservative),
            miles_traveled_actual: 1_000.0,
            ..GameState::default()
        };
        vehicle_state.start_of_day();
        assert_eq!(vehicle_state.failure_log_key(), Some(LOG_VEHICLE_FAILURE));

        let mut classic_guard = GameState {
            vehicle: crate::vehicle::Vehicle {
                health: 0.0,
                ..crate::vehicle::Vehicle::default()
            },
            mode: GameMode::Classic,
            policy: Some(PolicyKind::Balanced),
            miles_traveled_actual: 10.0,
            ..GameState::default()
        };
        classic_guard.start_of_day();
        assert!(classic_guard.failure_log_key().is_none());

        let mut pants_state = GameState {
            stats: Stats {
                pants: 100,
                ..Stats::default()
            },
            ..GameState::default()
        };
        assert_eq!(pants_state.failure_log_key(), Some(LOG_PANTS_EMERGENCY));

        let mut exposure_cold = GameState {
            stats: Stats {
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::ExposureCold),
            ..GameState::default()
        };
        assert_eq!(exposure_cold.failure_log_key(), Some(LOG_HEALTH_COLLAPSE));

        let mut exposure_heat = GameState {
            stats: Stats {
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::ExposureHeat),
            ..GameState::default()
        };
        assert_eq!(exposure_heat.failure_log_key(), Some(LOG_HEALTH_COLLAPSE));

        let mut starvation = GameState {
            stats: Stats {
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::Starvation),
            ..GameState::default()
        };
        assert_eq!(starvation.failure_log_key(), Some(LOG_HEALTH_COLLAPSE));

        let mut vehicle = GameState {
            stats: Stats {
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::Vehicle),
            ..GameState::default()
        };
        assert_eq!(vehicle.failure_log_key(), Some(LOG_HEALTH_COLLAPSE));

        let mut disease = GameState {
            stats: Stats {
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::Disease),
            ..GameState::default()
        };
        assert_eq!(disease.failure_log_key(), Some(LOG_HEALTH_COLLAPSE));

        let mut breakdown = GameState {
            stats: Stats {
                hp: 0,
                ..Stats::default()
            },
            last_damage: Some(DamageCause::Breakdown),
            ..GameState::default()
        };
        assert_eq!(breakdown.failure_log_key(), Some(LOG_HEALTH_COLLAPSE));

        let mut sanity = GameState::default();
        sanity.stats.sanity = 0;
        assert_eq!(sanity.failure_log_key(), Some(LOG_SANITY_COLLAPSE));
    }

    #[test]
    fn consume_daily_effects_updates_stats() {
        let mut state = GameState::default();
        state.stats.sanity = 5;
        state.stats.supplies = 5;
        state.consume_daily_effects(-3, -4);
        assert_eq!(state.stats.sanity, 2);
        assert_eq!(state.stats.supplies, 1);
    }

    #[test]
    fn otdeluxe_navigation_event_trace_builds_candidates() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 1,
            wrong_weight: 1,
            impassable_weight: 1,
            snowbound_weight: 0,
            snowbound_min_depth_in: 10.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = SmallRng::seed_from_u64(11);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 0.0, &mut rng);
        assert!(event.is_some());
        assert!(trace.is_some());
    }

    #[test]
    fn otdeluxe_fatality_probability_non_finite_defaults_to_zero() {
        let policy = OtDeluxe90sPolicy::default();
        let model = FatalityModel {
            base_prob_per_day: f32::NAN,
            prob_modifiers: Vec::new(),
            apply_doctor_mult: false,
        };
        let context = OtDeluxeFatalityContext {
            health_general: 12,
            pace: OtDeluxePace::Steady,
            rations: OtDeluxeRations::Filling,
            weather: Weather::Clear,
            occupation: None,
        };
        assert!(
            (otdeluxe_fatality_probability(&model, context, &policy) - 0.0).abs() <= f32::EPSILON
        );
    }

    #[test]
    fn otdeluxe_mobility_multiplier_covers_jobs() {
        let policy = OtDeluxe90sPolicy::default();
        let mobility_mult =
            sanitize_disease_multiplier(policy.occupation_advantages.mobility_failure_mult);
        assert!(
            (otdeluxe_mobility_failure_mult(Some(OtDeluxeOccupation::Farmer), &policy)
                - mobility_mult)
                .abs()
                <= f32::EPSILON
        );
        assert!(
            (otdeluxe_mobility_failure_mult(Some(OtDeluxeOccupation::Doctor), &policy) - 1.0).abs()
                <= f32::EPSILON
        );
    }

    #[test]
    fn emergency_limp_guard_respects_window() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 1_900.0,
            endgame: crate::endgame::EndgameState {
                last_limp_mile: 1_900.0 - (EMERGENCY_LIMP_MILE_WINDOW - 1.0),
                ..crate::endgame::EndgameState::default()
            },
            ..GameState::default()
        };
        assert!(!state.try_emergency_limp_guard());
    }

    #[test]
    fn otdeluxe_afflictions_return_none_for_legacy() {
        let mut state = GameState::default();
        assert!(state.tick_otdeluxe_afflictions().is_none());
        let catalog = DiseaseCatalog::default_catalog();
        assert!(
            state
                .tick_otdeluxe_afflictions_with_catalog(catalog)
                .is_none()
        );
    }

    #[test]
    fn deep_travel_boosts_cover_thresholds() {
        let conservative_state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Conservative),
            day: 100,
            miles_traveled_actual: 2_000.0,
            ..GameState::default()
        };
        assert!((conservative_state.deep_conservative_travel_boost() - 1.0).abs() <= f32::EPSILON);

        let aggressive_state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            day: 90,
            miles_traveled_actual: 0.0,
            ..GameState::default()
        };
        assert!(aggressive_state.deep_aggressive_reach_boost() > 1.0);
    }

    #[test]
    fn deep_aggressive_sanity_guard_adds_tag() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            day: DEEP_AGGRESSIVE_SANITY_DAY,
            miles_traveled_actual: DEEP_AGGRESSIVE_SANITY_MILES,
            stats: Stats {
                sanity: 0,
                ..Stats::default()
            },
            budget_cents: DEEP_AGGRESSIVE_SANITY_COST,
            ..GameState::default()
        };
        state.start_of_day();
        state.record_travel_day(TravelDayKind::Partial, 0.0, "seed");
        state.apply_deep_aggressive_sanity_guard();
        let tags = &state.current_day_record.as_ref().expect("day record").tags;
        assert!(tags.contains(&DayTag::new("da_sanity_guard")));
    }

    #[test]
    fn deep_aggressive_compose_skips_non_aggressive() {
        let mut state = GameState {
            mode: GameMode::Classic,
            policy: Some(PolicyKind::Aggressive),
            ..GameState::default()
        };
        assert!(!state.apply_deep_aggressive_compose());
    }

    #[test]
    fn check_vehicle_terminal_state_applies_tolerance_threshold() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 2_000.0,
            vehicle: crate::vehicle::Vehicle {
                health: 10.0,
                ..crate::vehicle::Vehicle::default()
            },
            ..GameState::default()
        };
        assert!(!state.check_vehicle_terminal_state());
    }

    #[test]
    fn check_vehicle_terminal_state_respects_endgame_guard() {
        let mut state = GameState {
            endgame: crate::endgame::EndgameState {
                active: true,
                failure_guard_miles: 2_000.0,
                ..crate::endgame::EndgameState::default()
            },
            miles_traveled_actual: 1_000.0,
            budget_cents: 0,
            inventory: Inventory {
                spares: Spares::default(),
                ..Inventory::default()
            },
            vehicle_breakdowns: i32::MAX,
            vehicle: crate::vehicle::Vehicle {
                health: 0.0,
                ..crate::vehicle::Vehicle::default()
            },
            ..GameState::default()
        };
        assert!(!state.check_vehicle_terminal_state());
        assert!(state.day_state.rest.rest_requested);
    }

    #[test]
    fn handle_crossing_event_blocks_when_pending() {
        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::BridgeOut,
                computed_miles_today: 12.0,
            }),
            ..GameState::default()
        };
        let outcome = state.handle_crossing_event(0.0);
        assert_eq!(outcome, Some((false, String::from(LOG_TRAVEL_BLOCKED))));
    }

    #[test]
    fn resolve_pending_crossing_choice_rejects_bribe_without_funds() {
        let mut state = GameState {
            pending_crossing: Some(PendingCrossing {
                kind: CrossingKind::BridgeOut,
                computed_miles_today: 12.0,
            }),
            pending_crossing_choice: Some(CrossingChoice::Bribe),
            budget_cents: 0,
            ..GameState::default()
        };
        let outcome = state.resolve_pending_crossing_choice(CrossingChoice::Bribe);
        assert!(outcome.is_none());
        assert!(state.pending_crossing_choice.is_none());
    }

    #[test]
    fn resolve_pending_otdeluxe_crossing_sets_ending_when_party_dead() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ot_deluxe: OtDeluxeState {
                party: OtDeluxePartyState {
                    members: vec![OtDeluxePartyMember {
                        name: String::from("Ada"),
                        alive: false,
                        sick_days_remaining: 0,
                        injured_days_remaining: 0,
                        illness_id: None,
                        injury_id: None,
                    }],
                },
                crossing: OtDeluxeCrossingState {
                    choice_pending: true,
                    river_kind: Some(OtDeluxeRiver::Kansas),
                    river: Some(OtDeluxeRiverState {
                        width_ft: 120.0,
                        depth_ft: 2.0,
                        swiftness: 0.2,
                        bed: OtDeluxeRiverBed::Muddy,
                    }),
                    computed_miles_today: 12.0,
                    ..OtDeluxeCrossingState::default()
                },
                ..OtDeluxeState::default()
            },
            ..GameState::default()
        };
        state.start_of_day();
        let outcome = state.resolve_pending_otdeluxe_crossing_choice(OtDeluxeCrossingMethod::Ford);
        assert!(outcome.is_some());
        assert!(matches!(
            state.ending,
            Some(Ending::Collapse {
                cause: CollapseCause::Crossing
            })
        ));
    }

    #[test]
    fn otdeluxe_crossing_context_returns_none_when_invalid() {
        let mut legacy = GameState::default();
        assert!(
            legacy
                .otdeluxe_crossing_context(OtDeluxeCrossingMethod::Ford)
                .is_none()
        );

        let mut otdeluxe = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        assert!(
            otdeluxe
                .otdeluxe_crossing_context(OtDeluxeCrossingMethod::Ford)
                .is_none()
        );
    }

    #[test]
    fn otdeluxe_crossing_log_and_severity_covers_outcomes() {
        assert_eq!(
            GameState::otdeluxe_crossing_log_and_severity(OtDeluxeCrossingOutcome::SuppliesWet).0,
            LOG_OT_CROSSING_WET
        );
        assert_eq!(
            GameState::otdeluxe_crossing_log_and_severity(OtDeluxeCrossingOutcome::Tipped).0,
            LOG_OT_CROSSING_TIPPED
        );
        assert_eq!(
            GameState::otdeluxe_crossing_log_and_severity(OtDeluxeCrossingOutcome::Sank).0,
            LOG_OT_CROSSING_SANK
        );
        assert_eq!(
            GameState::otdeluxe_crossing_log_and_severity(OtDeluxeCrossingOutcome::Drowned).0,
            LOG_OT_CROSSING_DROWNED
        );
    }

    #[test]
    fn otdeluxe_crossing_delays_advance_days() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        let start_day = state.day;
        let policy = default_otdeluxe_policy();
        let resolution = OtDeluxeCrossingResolution {
            outcome: OtDeluxeCrossingOutcome::SuppliesWet,
            crossing_days: 2,
            wait_days: 1,
            drying_days: 1,
            loss_ratio: 0.0,
            drownings: 0,
        };
        state.apply_otdeluxe_crossing_delays(policy, &resolution);
        assert!(state.day > start_day);
    }

    #[test]
    fn resolve_crossing_outcome_uses_fallback_rng() {
        let mut state = GameState::default();
        state.detach_rng_bundle();
        let policy = CrossingPolicy::default();
        let ctx = CrossingContext {
            policy: &policy,
            kind: CrossingKind::BridgeOut,
            has_permit: false,
            bribe_intent: false,
            prior_bribe_attempts: 0,
        };
        let outcome = state.resolve_crossing_outcome(ctx, 0);
        assert!(matches!(
            outcome.result,
            CrossingResult::Pass | CrossingResult::Detour(_) | CrossingResult::TerminalFail
        ));
    }

    #[test]
    fn apply_crossing_decisions_records_bribe_success() {
        let mut state = GameState {
            budget_cents: 10_000,
            ..GameState::default()
        };
        let cfg = CrossingConfig::default();
        let mut telemetry = CrossingTelemetry::new(
            state.day,
            state.region,
            state.season,
            CrossingKind::BridgeOut,
        );
        let outcome = CrossingOutcome {
            result: CrossingResult::Pass,
            used_permit: false,
            bribe_attempted: true,
            bribe_succeeded: true,
        };
        state.apply_crossing_decisions(outcome, &cfg, CrossingKind::BridgeOut, &mut telemetry);
        assert_eq!(state.crossing_bribe_successes, 1);
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == "crossing.result.bribe.success")
        );
    }

    #[test]
    fn process_crossing_result_detour_records_bribe_attempt() {
        let mut state = GameState::default();
        state.start_of_day();
        let mut telemetry = CrossingTelemetry::new(
            state.day,
            state.region,
            state.season,
            CrossingKind::BridgeOut,
        );
        telemetry.bribe_attempted = true;
        let outcome = CrossingOutcome {
            result: CrossingResult::Detour(2),
            used_permit: false,
            bribe_attempted: true,
            bribe_succeeded: false,
        };
        let _ = state.process_crossing_result(outcome, telemetry, 5.0);
        assert_eq!(
            state
                .crossing_events
                .last()
                .and_then(|event| event.bribe_success),
            Some(false)
        );
    }

    #[test]
    fn rehydrate_backfills_travel_day() {
        let state = GameState {
            state_version: 0,
            day: 2,
            travel_days: 1,
            day_records: Vec::new(),
            ..GameState::default()
        };
        let rehydrated = state.rehydrate(EncounterData::from_encounters(Vec::new()));
        assert_eq!(rehydrated.day_records.len(), 1);
        assert_eq!(rehydrated.day_records[0].kind, TravelDayKind::Travel);
    }

    #[test]
    fn rehydrate_backfills_partial_day() {
        let state = GameState {
            state_version: 0,
            day: 2,
            partial_travel_days: 1,
            day_records: Vec::new(),
            ..GameState::default()
        };
        let rehydrated = state.rehydrate(EncounterData::from_encounters(Vec::new()));
        assert_eq!(rehydrated.day_records.len(), 1);
        assert_eq!(rehydrated.day_records[0].kind, TravelDayKind::Partial);
    }

    #[test]
    fn rehydrate_backfills_non_travel_day() {
        let state = GameState {
            state_version: 0,
            day: 2,
            non_travel_days: 1,
            day_records: Vec::new(),
            ..GameState::default()
        };
        let rehydrated = state.rehydrate(EncounterData::from_encounters(Vec::new()));
        assert_eq!(rehydrated.day_records.len(), 1);
        assert_eq!(rehydrated.day_records[0].kind, TravelDayKind::NonTravel);
    }

    #[test]
    fn consume_otdeluxe_navigation_delay_day_adds_tag() {
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.travel.delay_days_remaining = 1;
        state.start_of_day();
        state.record_travel_day(TravelDayKind::NonTravel, 0.0, "seed");
        assert!(state.consume_otdeluxe_navigation_delay_day());
        let tags = &state.day_records.last().expect("day record").tags;
        assert!(tags.contains(&DayTag::new("otdeluxe.nav_delay")));
    }

    #[test]
    fn apply_otdeluxe_navigation_event_with_policy_respects_preconditions() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut legacy_state = GameState::default();
        assert!(!legacy_state.apply_otdeluxe_navigation_event_with_policy(&policy));

        let mut delay_state = otdeluxe_state_with_party();
        delay_state.ot_deluxe.travel.delay_days_remaining = 1;
        assert!(!delay_state.apply_otdeluxe_navigation_event_with_policy(&policy));

        let mut distance_state = otdeluxe_state_with_party();
        distance_state.distance_today = 0.0;
        distance_state.distance_today_raw = 0.0;
        assert!(!distance_state.apply_otdeluxe_navigation_event_with_policy(&policy));
    }

    #[test]
    fn apply_otdeluxe_navigation_hard_stop_blocks_and_logs() {
        let mut state = otdeluxe_state_with_party();
        state.start_of_day();
        state.record_travel_day(TravelDayKind::NonTravel, 0.0, "seed");
        state.apply_otdeluxe_navigation_hard_stop(OtDeluxeNavigationEvent::Impassable, 2);
        let tags = &state.day_records.last().expect("day record").tags;
        assert!(tags.contains(&DayTag::new("otdeluxe.nav_impassable")));
        assert_eq!(
            state.ot_deluxe.travel.wagon_state,
            OtDeluxeWagonState::Blocked
        );
    }

    #[test]
    fn apply_otdeluxe_random_event_returns_none_for_legacy() {
        let mut state = GameState::default();
        assert!(state.apply_otdeluxe_random_event().is_none());
    }

    #[test]
    fn sanitize_event_weight_mult_handles_invalid() {
        assert!((sanitize_event_weight_mult(f32::NAN) - 1.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn apply_otdeluxe_random_event_selection_rejects_unknown() {
        let mut state = otdeluxe_state_with_party();
        let selection = make_event_selection("unknown", "variant");
        let mut rng = SmallRng::seed_from_u64(42);
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn random_weather_catastrophe_rejects_unknown() {
        let mut state = otdeluxe_state_with_party();
        assert!(
            state
                .apply_otdeluxe_random_weather_catastrophe("unknown", 0.2, 0.3)
                .is_none()
        );
    }

    #[test]
    fn random_resource_shortage_bad_water_payload() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(7);
        let outcome =
            state.apply_otdeluxe_random_resource_shortage("bad_water", 0.2, 0.3, &mut rng);
        assert!(outcome.is_some());
    }

    #[test]
    fn random_party_incident_variants() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(5);
        assert!(
            state
                .apply_otdeluxe_random_party_incident("lost_member", 0.2, 0.3, &mut rng)
                .is_some()
        );
        assert!(
            state
                .apply_otdeluxe_random_party_incident("snakebite", 0.2, 0.3, &mut rng)
                .is_some()
        );
        assert!(
            state
                .apply_otdeluxe_random_party_incident("unknown", 0.2, 0.3, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn random_oxen_incident_rejects_unknown() {
        let mut state = otdeluxe_state_with_party();
        assert!(
            state
                .apply_otdeluxe_random_oxen_incident("unknown", 0.2, 0.3)
                .is_none()
        );
    }

    #[test]
    fn random_resource_change_rejects_unknown() {
        let mut state = otdeluxe_state_with_party();
        assert!(
            state
                .apply_otdeluxe_random_resource_change("unknown", 0.2, 0.3)
                .is_none()
        );
    }

    #[test]
    fn random_wagon_part_break_payload_and_unknown() {
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.inventory.spares_wheels = 0;
        state.ot_deluxe.inventory.spares_axles = 0;
        state.ot_deluxe.inventory.spares_tongues = 0;
        let mut rng = SmallRng::seed_from_u64(3);
        assert!(
            state
                .apply_otdeluxe_random_wagon_part_break("repairable", 0.2, 0.3, &mut rng)
                .is_some()
        );
        assert!(
            state
                .apply_otdeluxe_random_wagon_part_break("unknown", 0.2, 0.3, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn random_travel_hazard_rejects_unknown() {
        let mut state = otdeluxe_state_with_party();
        assert!(
            state
                .apply_otdeluxe_random_travel_hazard("unknown", 0.2, 0.3)
                .is_none()
        );
    }

    #[test]
    fn apply_otdeluxe_random_affliction_handles_empty_catalog() {
        let mut state = otdeluxe_state_with_party();
        let empty_catalog = DiseaseCatalog {
            diseases: Vec::new(),
        };
        let mut rng = SmallRng::seed_from_u64(9);
        let outcome = state.apply_otdeluxe_random_affliction_with_catalog(
            &empty_catalog,
            &mut rng,
            OtDeluxeAfflictionKind::Illness,
        );
        assert!(outcome.is_some());
    }

    #[test]
    fn lose_random_party_members_breaks_when_empty() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(2);
        let lost = state.lose_random_party_members(&mut rng, 2);
        assert_eq!(lost.len(), 1);
    }

    #[test]
    fn handle_travel_block_adds_tag_when_day_started() {
        let mut state = otdeluxe_state_with_party();
        state.day_state.travel.travel_blocked = true;
        state.start_of_day();
        state.record_travel_day(TravelDayKind::NonTravel, 0.0, "seed");
        let outcome = state.handle_travel_block(false);
        assert!(outcome.is_some());
        let tags = &state.day_records.last().expect("day record").tags;
        assert!(tags.contains(&DayTag::new("repair")));
    }

    #[test]
    fn process_encounter_flow_major_repair_records_nontravel() {
        let encounter = Encounter {
            id: String::from("major"),
            name: String::from("Major"),
            desc: String::new(),
            weight: 1,
            regions: vec![String::from("heartland")],
            modes: vec![String::from("classic")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: true,
            chainable: false,
        };
        let mut state = GameState {
            encounter_chance_today: 1.0,
            data: Some(EncounterData::from_encounters(vec![encounter])),
            ..GameState::default()
        };
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(1)));
        state.start_of_day();
        let rng_bundle = state.rng_bundle.clone();
        let outcome = state.process_encounter_flow(rng_bundle.as_ref(), false);
        assert!(outcome.is_some());
        let record = state.current_day_record.as_ref().expect("day record");
        assert_eq!(record.kind, TravelDayKind::NonTravel);
        assert!(record.tags.contains(&DayTag::new("repair")));
    }

    #[test]
    fn apply_encounter_partial_travel_uses_ratio_when_partial_missing() {
        let mut state = GameState::default();
        state.features.travel_v2 = true;
        state.start_of_day();
        state.encounters.occurred_today = true;
        state.current_encounter = Some(Encounter {
            id: String::from("minor"),
            name: String::from("Minor"),
            desc: String::new(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        });
        state.distance_today = 10.0;
        state.partial_distance_today = 0.0;
        state.apply_encounter_partial_travel();
        let record = state.current_day_record.as_ref().expect("day record");
        assert_eq!(record.kind, TravelDayKind::Partial);
    }

    #[test]
    fn maybe_reroll_encounter_resets_rotation_pending() {
        let encounter = Encounter {
            id: String::from("encounter"),
            name: String::from("Encounter"),
            desc: String::new(),
            weight: 1,
            regions: vec![String::from("heartland")],
            modes: vec![String::from("classic")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        let replacement = Encounter {
            id: String::from("replacement"),
            name: String::from("Replacement"),
            desc: String::new(),
            weight: 1,
            regions: vec![String::from("heartland")],
            modes: vec![String::from("classic")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        let mut state = GameState::default();
        state.features.encounter_diversity = true;
        state.day = 10;
        let replacement_clone = replacement.clone();
        state.data = Some(EncounterData::from_encounters(vec![
            encounter.clone(),
            replacement_clone,
        ]));
        state.recent_encounters = VecDeque::from(vec![RecentEncounter {
            id: encounter.id.clone(),
            day: state.day,
            region: None,
        }]);
        state.encounters.force_rotation_pending = true;
        let bundle = bundle_with_roll_below(ENCOUNTER_REROLL_PENALTY, RngBundle::encounter);
        let rotation_backlog = VecDeque::from(vec![replacement.id]);
        let recent_snapshot: Vec<RecentEncounter> =
            state.recent_encounters.iter().cloned().collect();
        let rerolled = state.maybe_reroll_encounter(
            Some(&bundle),
            &recent_snapshot,
            rotation_backlog,
            Some(encounter),
        );
        assert!(rerolled.is_some());
        assert!(!state.encounters.force_rotation_pending);
    }

    #[test]
    fn select_breakdown_part_with_trace_handles_zero_weight() {
        let state = GameState {
            journey_part_weights: PartWeights {
                tire: 0,
                battery: 0,
                alt: 0,
                pump: 0,
            },
            ..GameState::default()
        };
        let mut rng = SmallRng::seed_from_u64(1);
        let (part, trace) = state.select_breakdown_part_with_trace(&mut rng);
        assert_eq!(part, Part::Tire);
        assert!(trace.is_none());
    }

    #[test]
    fn select_breakdown_part_with_trace_picks_weighted_part() {
        let state = GameState {
            journey_part_weights: PartWeights {
                tire: 0,
                battery: 5,
                alt: 0,
                pump: 0,
            },
            ..GameState::default()
        };
        let mut rng = SmallRng::seed_from_u64(2);
        let (part, trace) = state.select_breakdown_part_with_trace(&mut rng);
        assert_eq!(part, Part::Battery);
        assert!(trace.is_some());
    }

    #[test]
    fn sanitize_breakdown_max_chance_defaults_for_invalid() {
        assert!(
            (GameState::sanitize_breakdown_max_chance(f32::NAN) - PROBABILITY_MAX).abs()
                <= f32::EPSILON
        );
    }

    #[test]
    fn apply_choice_uses_classic_base_distance_when_no_distance() {
        let effects = Effects {
            travel_bonus_ratio: 0.5,
            ..Effects::default()
        };
        let encounter = encounter_with_choice(effects);
        let mut state = GameState {
            features: FeatureFlags {
                travel_v2: false,
                ..FeatureFlags::default()
            },
            current_encounter: Some(encounter),
            ..GameState::default()
        };
        state.start_of_day();
        state.apply_choice(0);
        let diff = TRAVEL_CLASSIC_BASE_DISTANCE.mul_add(-0.5, state.distance_today);
        assert!(diff.abs() <= f32::EPSILON);
    }

    #[test]
    fn resolve_breakdown_jury_rigged_after_one_day() {
        let mut state = GameState {
            breakdown: Some(crate::vehicle::Breakdown {
                part: Part::Tire,
                day_started: 0,
            }),
            day: 2,
            budget_cents: 0,
            inventory: Inventory {
                spares: Spares::default(),
                ..Inventory::default()
            },
            ..GameState::default()
        };
        state.resolve_breakdown();
        assert!(state.breakdown.is_none());
        assert!(!state.day_state.travel.travel_blocked);
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == "log.breakdown-jury-rigged")
        );
    }

    #[test]
    fn consume_otdeluxe_spare_for_breakdown_branches() {
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.inventory.spares_axles = 1;
        state.ot_deluxe.inventory.spares_tongues = 1;
        assert!(state.consume_otdeluxe_spare_for_breakdown(Part::Battery));
        assert!(state.consume_otdeluxe_spare_for_breakdown(Part::Alternator));
    }

    #[test]
    fn consume_any_spare_for_emergency_branches() {
        let mut state = GameState {
            inventory: Inventory {
                spares: Spares {
                    tire: 0,
                    battery: 1,
                    alt: 1,
                    pump: 1,
                },
                ..Inventory::default()
            },
            ..GameState::default()
        };
        assert!(state.consume_any_spare_for_emergency());
        assert!(state.consume_any_spare_for_emergency());
        assert!(state.consume_any_spare_for_emergency());
    }

    #[test]
    fn apply_encounter_chance_today_defaults_limits() {
        let mut state = GameState::default();
        let limits = PacingLimits {
            encounter_base: 0.0,
            encounter_ceiling: 0.0,
            ..PacingLimits::default()
        };
        state.apply_encounter_chance_today(0.0, 0.0, 0.0, 1.0, &limits);
        assert!((0.0..=1.0).contains(&state.encounter_chance_today));
    }

    #[test]
    fn apply_pace_and_diet_relief_and_caps() {
        let cfg = crate::pacing::PacingConfig {
            pace: vec![PaceCfg {
                id: String::from("steady"),
                name: String::from("Steady"),
                dist_mult: 1.0,
                distance: 0.0,
                sanity: 0,
                pants: 20,
                encounter_chance_delta: 0.0,
            }],
            diet: vec![crate::pacing::DietCfg {
                id: String::from("mixed"),
                name: String::from("Mixed"),
                sanity: 0,
                pants: 0,
                receipt_find_pct_delta: 0,
            }],
            limits: PacingLimits {
                passive_relief: 5,
                passive_relief_threshold: 10,
                boss_pants_cap: 5,
                boss_passive_relief: 0,
                pants_floor: 0,
                pants_ceiling: 200,
                ..PacingLimits::default()
            },
            enabled: true,
        };
        let mut state = GameState {
            pace: PaceId::Steady,
            diet: DietId::Mixed,
            stats: Stats {
                pants: 10,
                ..Stats::default()
            },
            mods: PersonaMods {
                pants_relief: 3,
                pants_relief_threshold: 10,
                ..PersonaMods::default()
            },
            boss: BossProgress {
                readiness: BossReadiness {
                    ready: true,
                    ..BossReadiness::default()
                },
                ..BossProgress::default()
            },
            ..GameState::default()
        };
        state.apply_pace_and_diet(&cfg);
        assert!(state.stats.pants <= 200);
    }

    #[test]
    fn save_and_load_placeholders() {
        let state = GameState::default();
        state.save();
        assert!(GameState::load().is_none());
    }

    #[test]
    fn tick_camp_cooldowns_and_auto_rest() {
        let mut state = GameState {
            camp: CampState {
                rest_cooldown: 1,
                forage_cooldown: 1,
                repair_cooldown: 1,
            },
            auto_camp_rest: true,
            rest_threshold: 5,
            stats: Stats {
                sanity: 4,
                ..Stats::default()
            },
            ..GameState::default()
        };
        state.tick_camp_cooldowns();
        assert_eq!(state.camp.rest_cooldown, 0);
        assert_eq!(state.camp.forage_cooldown, 0);
        assert_eq!(state.camp.repair_cooldown, 0);
        assert!(state.should_auto_rest());
    }

    #[test]
    fn refresh_exec_order_initializes_day() {
        let mut state = GameState::default();
        assert!(!state.day_state.lifecycle.day_initialized);
        state.refresh_exec_order();
        assert!(state.day_state.lifecycle.day_initialized);
    }

    #[test]
    fn resolve_otdeluxe_route_prompt_branches() {
        let mut legacy = GameState::default();
        assert!(!legacy.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::StayOnTrail));

        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ..GameState::default()
        };
        assert!(!state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::StayOnTrail));

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::StayOnTrail));

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);
        state.ot_deluxe.route.variant = OtDeluxeTrailVariant::DallesShortcut;
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::SubletteCutoff));
        assert_eq!(
            state.ot_deluxe.route.variant,
            OtDeluxeTrailVariant::SubletteAndDallesShortcut
        );

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesShortcut);
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::StayOnTrail));

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesShortcut);
        state.ot_deluxe.route.variant = OtDeluxeTrailVariant::SubletteCutoff;
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::DallesShortcut));
        assert_eq!(
            state.ot_deluxe.route.variant,
            OtDeluxeTrailVariant::SubletteAndDallesShortcut
        );

        state.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::DallesFinal);
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::BarlowRoad));
        assert_eq!(
            state.ot_deluxe.route.dalles_choice,
            Some(OtDeluxeDallesChoice::Barlow)
        );
    }

    #[test]
    fn push_log_adds_event() {
        let mut state = GameState::default();
        state.start_of_day();
        state.push_log("log.test");
        assert_eq!(state.logs.last().map(String::as_str), Some("log.test"));
        let event = state.events_today.last().expect("event");
        assert_eq!(event.kind, EventKind::LegacyLogKey);
        assert!(event.ui_surface_hint.is_some());
    }

    #[test]
    fn roll_otdeluxe_affliction_kind_uses_override_weights() {
        let policy = OtDeluxe90sPolicy::default();
        let overrides = OtDeluxePolicyOverride {
            affliction_weights: OtDeluxeAfflictionWeightOverride {
                illness: Some(2),
                injury: Some(3),
            },
            ..OtDeluxePolicyOverride::default()
        };
        let mut rng = SmallRng::seed_from_u64(12);
        let (_, trace) = roll_otdeluxe_affliction_kind(&policy.affliction, &overrides, &mut rng);
        let trace = trace.expect("trace expected");
        assert_eq!(trace.candidates.len(), 2);
        assert!((trace.candidates[0].base_weight - 2.0).abs() <= f64::EPSILON);
        assert!((trace.candidates[1].base_weight - 3.0).abs() <= f64::EPSILON);
    }

    #[test]
    fn otdeluxe_navigation_event_with_snowbound_weight() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 1,
            wrong_weight: 1,
            impassable_weight: 1,
            snowbound_weight: 2,
            snowbound_min_depth_in: 0.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = SmallRng::seed_from_u64(5);
        let (_, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 12.0, &mut rng);
        let trace = trace.expect("trace expected");
        assert_eq!(trace.candidates.len(), 4);
        assert!(
            trace
                .candidates
                .iter()
                .any(|candidate| candidate.id == "snowbound")
        );
    }

    #[test]
    fn apply_otdeluxe_random_event_selection_branches() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(7);

        let selection = make_event_selection("weather_catastrophe", "blizzard");
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_some()
        );

        let selection = make_event_selection("resource_shortage", "bad_water");
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_some()
        );

        let selection = make_event_selection("party_incident", "snakebite");
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_some()
        );

        let selection = make_event_selection("party_incident", "lost_member");
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_some()
        );

        let selection = make_event_selection("wagon_part_break", "repairable");
        assert!(
            state
                .apply_otdeluxe_random_event_selection(&selection, &mut rng)
                .is_some()
        );
    }

    #[test]
    fn apply_otdeluxe_random_resource_shortage_rejects_unknown() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(3);
        let outcome = state.apply_otdeluxe_random_resource_shortage("unknown", 0.2, 0.4, &mut rng);
        assert!(outcome.is_none());
    }

    #[test]
    fn apply_otdeluxe_random_party_incident_payloads() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(4);
        let lost = state.apply_otdeluxe_random_party_incident("lost_member", 0.1, 0.4, &mut rng);
        assert!(lost.is_some());
        let snake = state.apply_otdeluxe_random_party_incident("snakebite", 0.1, 0.4, &mut rng);
        assert!(snake.is_some());
    }

    #[test]
    fn apply_otdeluxe_random_wagon_part_break_payload() {
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.inventory.spares_wheels = 0;
        state.ot_deluxe.inventory.spares_axles = 0;
        state.ot_deluxe.inventory.spares_tongues = 0;
        let mut rng = SmallRng::seed_from_u64(6);
        let outcome =
            state.apply_otdeluxe_random_wagon_part_break("repairable", 0.2, 0.3, &mut rng);
        assert!(outcome.is_some());
    }

    #[test]
    fn apply_otdeluxe_random_affliction_injury_sets_display_key() {
        let mut state = otdeluxe_state_with_party();
        let catalog = DiseaseCatalog {
            diseases: vec![DiseaseDef {
                id: "injury".into(),
                kind: DiseaseKind::Injury,
                display_key: "disease.injury".into(),
                weight: 1,
                duration_days: Some(3),
                onset_effects: DiseaseEffects::default(),
                daily_tick_effects: DiseaseEffects::default(),
                fatality_model: None,
                tags: Vec::new(),
            }],
        };
        let mut rng = SmallRng::seed_from_u64(2);
        let outcome = state.apply_otdeluxe_random_affliction_with_catalog(
            &catalog,
            &mut rng,
            OtDeluxeAfflictionKind::Injury,
        );
        let outcome = outcome.expect("injury outcome");
        assert_eq!(outcome.display_key.as_deref(), Some("disease.injury"));
    }

    #[test]
    fn lose_random_party_members_handles_exhausted_pool() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(2);
        let lost = state.lose_random_party_members(&mut rng, 2);
        assert_eq!(lost.len(), 1);
    }

    #[test]
    fn select_breakdown_part_with_trace_nonzero_total() {
        let state = GameState {
            journey_part_weights: PartWeights {
                tire: 1,
                battery: 1,
                alt: 0,
                pump: 0,
            },
            ..GameState::default()
        };
        let mut rng = SmallRng::seed_from_u64(9);
        let (_part, trace) = state.select_breakdown_part_with_trace(&mut rng);
        assert!(trace.is_some());
    }

    #[test]
    fn log_travel_debug_emits_when_enabled() {
        let state = GameState::default();
        with_debug_env(|| {
            state.log_travel_debug();
        });
    }

    #[test]
    fn vehicle_roll_logs_when_enabled() {
        let mut state = GameState::default();
        state.journey_breakdown.base = 1.0;
        state.journey_breakdown.beta = 0.0;
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.2));
        let triggered = with_debug_env(|| state.vehicle_roll());
        assert!(triggered);
    }

    #[test]
    fn apply_choice_logs_when_enabled() {
        let mut state = GameState::default();
        let encounter = Encounter {
            id: String::from("log_choice"),
            name: String::from("Log Choice"),
            desc: String::new(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: String::from("Pick"),
                effects: Effects {
                    hp: -1,
                    sanity: 0,
                    ..Effects::default()
                },
            }],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        state.current_encounter = Some(encounter);
        with_debug_env(|| {
            state.apply_choice(0);
        });
    }

    #[test]
    fn consume_daily_effects_logs_when_enabled() {
        let mut state = GameState::default();
        with_debug_env(|| {
            state.consume_daily_effects(1, -1);
        });
    }

    #[test]
    fn apply_store_purchase_logs_when_enabled() {
        let mut state = GameState::default();
        let grants = Grants::default();
        with_debug_env(|| {
            state.apply_store_purchase(200, &grants, &[]);
        });
    }

    #[test]
    fn failure_log_key_uses_deep_aggressive_field_repair() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 1_600.0,
            vehicle: crate::vehicle::Vehicle {
                health: 0.0,
                ..crate::vehicle::Vehicle::default()
            },
            ..GameState::default()
        };
        state.start_of_day();
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        let log_key = state.failure_log_key();
        assert!(log_key.is_none());
    }

    #[test]
    fn failure_log_key_uses_emergency_limp_guard() {
        let mut state = GameState {
            mode: GameMode::Classic,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 1_900.0,
            vehicle: crate::vehicle::Vehicle {
                health: 0.0,
                ..crate::vehicle::Vehicle::default()
            },
            ..GameState::default()
        };
        state.start_of_day();
        let log_key = state.failure_log_key();
        assert!(log_key.is_none());
    }

    #[test]
    fn roll_otdeluxe_affliction_kind_defaults_to_policy_weights() {
        let policy = OtDeluxe90sPolicy::default();
        let overrides = OtDeluxePolicyOverride::default();
        let mut rng = StepRng::new(0, 0);
        let (_, trace) = roll_otdeluxe_affliction_kind(&policy.affliction, &overrides, &mut rng);
        let trace = trace.expect("trace expected");
        assert_eq!(trace.candidates.len(), 2);
        assert!(
            (trace.candidates[0].base_weight - f64::from(policy.affliction.weight_illness)).abs()
                <= f64::EPSILON
        );
        assert!(
            (trace.candidates[1].base_weight - f64::from(policy.affliction.weight_injury)).abs()
                <= f64::EPSILON
        );
    }

    #[test]
    fn otdeluxe_navigation_event_selects_first_weight() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 2,
            wrong_weight: 1,
            impassable_weight: 1,
            snowbound_weight: 0,
            snowbound_min_depth_in: 999.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = StepRng::new(0, 0);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 0.0, &mut rng);
        assert_eq!(event, Some(OtDeluxeNavigationEvent::LostTrail));
        assert!(trace.is_some());
    }

    #[test]
    fn apply_otdeluxe_random_resource_shortage_no_grass_payload() {
        let mut state = otdeluxe_state_with_party();
        let mut rng = SmallRng::seed_from_u64(21);
        let outcome = state.apply_otdeluxe_random_resource_shortage("no_grass", 0.2, 0.4, &mut rng);
        assert!(outcome.is_some());
    }

    #[test]
    fn apply_otdeluxe_random_affliction_uses_default_duration() {
        let mut state = otdeluxe_state_with_party();
        let catalog = DiseaseCatalog {
            diseases: Vec::new(),
        };
        let mut rng = SmallRng::seed_from_u64(22);
        let outcome = state.apply_otdeluxe_random_affliction_with_catalog(
            &catalog,
            &mut rng,
            OtDeluxeAfflictionKind::Injury,
        );
        let policy = OtDeluxe90sPolicy::default();
        let member = &state.ot_deluxe.party.members[0];
        assert!(outcome.is_some());
        assert_eq!(
            member.injured_days_remaining,
            policy.affliction.injury_duration_days
        );
    }

    #[test]
    fn lose_random_party_members_breaks_when_no_alive() {
        let mut state = otdeluxe_state_with_party();
        state.ot_deluxe.party.members[0].alive = false;
        let mut rng = StepRng::new(0, 0);
        let lost = state.lose_random_party_members(&mut rng, 2);
        assert!(lost.is_empty());
    }

    #[test]
    fn select_breakdown_part_with_trace_uses_weights() {
        let state = GameState::default();
        let mut rng = StepRng::new(0, 0);
        let (_, trace) = state.select_breakdown_part_with_trace(&mut rng);
        assert!(trace.is_some());
    }

    #[test]
    fn resolve_otdeluxe_route_prompt_sets_dalles_shortcut_variant() {
        let mut state = GameState {
            mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
            ot_deluxe: OtDeluxeState {
                route: OtDeluxeRouteState {
                    pending_prompt: Some(OtDeluxeRoutePrompt::DallesShortcut),
                    variant: OtDeluxeTrailVariant::Main,
                    ..OtDeluxeRouteState::default()
                },
                ..OtDeluxeState::default()
            },
            ..GameState::default()
        };
        assert!(state.resolve_otdeluxe_route_prompt(OtDeluxeRouteDecision::DallesShortcut));
        assert_eq!(
            state.ot_deluxe.route.variant,
            OtDeluxeTrailVariant::DallesShortcut
        );
    }
}

/// Default diet setting
const fn default_diet() -> DietId {
    DietId::Mixed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Deep,
}

impl GameMode {
    #[must_use]
    pub const fn is_deep(self) -> bool {
        matches!(self, Self::Deep)
    }

    #[must_use]
    pub const fn boss_threshold(self) -> i32 {
        match self {
            Self::Classic => 1_000,
            Self::Deep => 1_200,
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
    pub const fn asset_key(self) -> &'static str {
        match self {
            Self::Heartland => "Heartland",
            Self::RustBelt => "RustBelt",
            Self::Beltway => "Beltway",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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
    pub const fn from_day(day: u32) -> Self {
        let season_len = 45;
        let idx = day.saturating_sub(1) / season_len;
        match idx % 4 {
            0 => Self::Spring,
            1 => Self::Summer,
            2 => Self::Fall,
            _ => Self::Winter,
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
            Self::Hunger => "hunger",
            Self::Vehicle => "vehicle",
            Self::Weather => "weather",
            Self::Breakdown => "breakdown",
            Self::Disease => "disease",
            Self::Crossing => "crossing",
            Self::Panic => "panic",
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
            Self::Cold => "cold",
            Self::Heat => "heat",
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BossReadiness {
    pub ready: bool,
    pub reached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BossResolution {
    pub attempted: bool,
    pub victory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BossProgress {
    #[serde(flatten)]
    pub readiness: BossReadiness,
    #[serde(flatten)]
    pub outcome: BossResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardState {
    pub deep_aggressive_sanity_guard_used: bool,
    pub starvation_backstop_used: bool,
    pub exposure_damage_lockout: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DayIntent {
    #[default]
    Continue,
    Rest,
    Trade,
    Hunt,
    CrossingChoicePending,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PendingCrossing {
    pub kind: CrossingKind,
    pub computed_miles_today: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct OtDeluxeCrossingLosses {
    food_lbs: u16,
    bullets: u16,
    clothes_sets: u16,
    spares_wheels: u8,
    spares_axles: u8,
    spares_tongues: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentState {
    pub pending: DayIntent,
    pub rest_days_remaining: u8,
}

impl Default for IntentState {
    fn default() -> Self {
        Self {
            pending: DayIntent::Continue,
            rest_days_remaining: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WaitState {
    pub ferry_wait_days_remaining: u8,
    pub drying_days_remaining: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RestState {
    pub rest_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TravelDayState {
    pub traveled_today: bool,
    pub partial_traveled_today: bool,
    pub travel_blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleState {
    pub day_initialized: bool,
    pub did_end_of_day: bool,
    pub suppress_stop_ratio: bool,
    #[serde(default)]
    pub log_cursor: u32,
    #[serde(default)]
    pub event_seq: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DayState {
    #[serde(flatten)]
    pub rest: RestState,
    #[serde(flatten)]
    pub travel: TravelDayState,
    #[serde(flatten)]
    pub lifecycle: LifecycleState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterState {
    pub occurred_today: bool,
    pub force_rotation_pending: bool,
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

pub const DEFAULT_STATS: Stats = Stats {
    supplies: 10,
    hp: 10,
    sanity: 10,
    credibility: 5,
    morale: 5,
    allies: 0,
    pants: 0,
};

impl Default for Stats {
    fn default() -> Self {
        DEFAULT_STATS
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthLabel {
    Good,
    Fair,
    Poor,
    VeryPoor,
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
    pub const fn total_spares(&self) -> i32 {
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
    pub const fn new(id: String, day: u32, region: Region) -> Self {
        Self {
            id,
            day,
            region: Some(region),
        }
    }
}

const fn default_rest_threshold() -> i32 {
    4
}

const fn default_trail_distance() -> f32 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub mode: GameMode,
    #[serde(default)]
    pub mechanical_policy: MechanicalPolicyId,
    pub seed: u64,
    #[serde(default = "GameState::current_version")]
    pub state_version: u16,
    pub day: u32,
    pub region: Region,
    #[serde(default)]
    pub season: Season,
    pub stats: Stats,
    #[serde(default)]
    pub general_strain: f32,
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
    #[serde(default = "default_trail_distance")]
    pub trail_distance: f32,
    #[serde(default)]
    pub miles_traveled: f32,
    #[serde(default)]
    pub miles_traveled_actual: f32,
    #[serde(default)]
    pub vehicle_breakdowns: i32,
    #[serde(default)]
    pub crossings_completed: u32,
    #[serde(default)]
    pub crossing_detours_taken: u32,
    #[serde(default)]
    pub crossing_failures: u32,
    #[serde(default)]
    pub crossing_permit_uses: u32,
    #[serde(default)]
    pub crossing_bribe_attempts: u32,
    #[serde(default)]
    pub crossing_bribe_successes: u32,
    #[serde(default)]
    pub crossing_events: Vec<CrossingTelemetry>,
    #[serde(default)]
    pub pending_crossing: Option<PendingCrossing>,
    #[serde(skip)]
    pub pending_crossing_choice: Option<CrossingChoice>,
    #[serde(skip)]
    pub pending_route_choice: Option<OtDeluxeRouteDecision>,
    #[serde(default)]
    pub starvation_days: u32,
    #[serde(default)]
    pub malnutrition_level: u32,
    #[serde(default)]
    pub exposure_streak_heat: u32,
    #[serde(default)]
    pub exposure_streak_cold: u32,
    #[serde(default)]
    pub disease_cooldown: u32,
    #[serde(default)]
    pub guards: GuardState,
    #[serde(default)]
    pub boss: BossProgress,
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
    #[serde(default)]
    pub encounters: EncounterState,
    /// Distance multiplier for today
    #[serde(default)]
    pub distance_today: f32,
    #[serde(default)]
    pub distance_today_raw: f32,
    #[serde(default)]
    pub partial_distance_today: f32,
    #[serde(default)]
    pub distance_cap_today: f32,
    #[serde(default)]
    pub day_records: Vec<DayRecord>,
    #[serde(default = "JourneyCfg::default_partial_ratio")]
    pub journey_partial_ratio: f32,
    #[serde(default)]
    pub journey_travel: TravelConfig,
    #[serde(default)]
    pub journey_wear: WearConfig,
    #[serde(default)]
    pub journey_breakdown: BreakdownConfig,
    #[serde(default)]
    pub journey_part_weights: PartWeights,
    #[serde(default)]
    pub journey_crossing: CrossingPolicy,
    pub logs: Vec<String>,
    pub receipts: Vec<String>,
    #[serde(default)]
    pub encounters_resolved: u32,
    #[serde(default)]
    pub prev_miles_traveled: f32,
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
    pub day_state: DayState,
    #[serde(default)]
    pub intent: IntentState,
    #[serde(default)]
    pub wait: WaitState,
    #[serde(default)]
    pub ot_deluxe: OtDeluxeState,
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
    pub exec_effects: ExecOrderEffects,
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
    /// Weather state and history for streak tracking
    #[serde(default)]
    pub weather_state: WeatherState,
    #[serde(skip)]
    pub weather_effects: WeatherEffects,
    /// Camp state and cooldowns
    #[serde(default)]
    pub camp: CampState,
    #[serde(default)]
    pub endgame: EndgameState,
    #[serde(default)]
    pub rotation_travel_days: u32,
    #[serde(default)]
    pub policy: Option<PolicyKind>,
    #[serde(default)]
    pub recent_travel_days: VecDeque<TravelDayKind>,
    #[serde(default)]
    pub day_reason_history: Vec<String>,
    #[serde(skip)]
    pub rotation_backlog: VecDeque<String>,
    #[serde(skip)]
    pub rng_bundle: Option<Rc<RngBundle>>,
    #[serde(skip)]
    pub data: Option<EncounterData>,
    #[serde(skip)]
    pub last_damage: Option<DamageCause>,
    #[serde(skip)]
    pub decision_traces_today: Vec<EventDecisionTrace>,
    #[serde(skip)]
    pub events_today: Vec<Event>,
    #[serde(skip)]
    pub current_day_record: Option<DayRecord>,
    #[serde(skip)]
    pub current_day_kind: Option<TravelDayKind>,
    #[serde(skip)]
    pub current_day_reason_tags: Vec<String>,
    #[serde(skip)]
    pub current_day_miles: f32,
    #[serde(skip)]
    pub last_breakdown_part: Option<Part>,
    #[serde(skip)]
    pub terminal_log_key: Option<String>,
}

macro_rules! game_state_defaults {
    () => {
        Self {
            mode: GameMode::Classic,
            mechanical_policy: MechanicalPolicyId::default(),
            seed: 0,
            state_version: Self::current_version(),
            day: 1,
            region: Region::Heartland,
            season: Season::default(),
            stats: Stats::default(),
            general_strain: 0.0,
            budget: 100,
            budget_cents: 10_000,
            inventory: Inventory::default(),
            persona_id: None,
            score_mult: 1.0,
            mods: PersonaMods::default(),
            features: FeatureFlags::default(),
            party: Party::default(),
            auto_camp_rest: false,
            rest_threshold: default_rest_threshold(),
            trail_distance: default_trail_distance(),
            miles_traveled: 0.0,
            miles_traveled_actual: 0.0,
            vehicle_breakdowns: 0,
            crossings_completed: 0,
            crossing_detours_taken: 0,
            crossing_failures: 0,
            crossing_permit_uses: 0,
            crossing_bribe_attempts: 0,
            crossing_bribe_successes: 0,
            crossing_events: Vec::new(),
            pending_crossing: None,
            pending_crossing_choice: None,
            pending_route_choice: None,
            starvation_days: 0,
            malnutrition_level: 0,
            exposure_streak_heat: 0,
            exposure_streak_cold: 0,
            disease_cooldown: 0,
            guards: GuardState::default(),
            boss: BossProgress::default(),
            ending: None,
            pace: default_pace(),
            diet: default_diet(),
            receipt_bonus_pct: 0,
            encounter_chance_today: ENCOUNTER_BASE_DEFAULT,
            encounters: EncounterState::default(),
            distance_today: 0.0,
            distance_today_raw: 0.0,
            partial_distance_today: 0.0,
            distance_cap_today: 0.0,
            day_records: Vec::new(),
            journey_partial_ratio: JourneyCfg::default_partial_ratio(),
            journey_travel: TravelConfig::default(),
            journey_wear: WearConfig::default(),
            journey_breakdown: BreakdownConfig::default(),
            journey_part_weights: PartWeights::default(),
            journey_crossing: CrossingPolicy::default(),
            logs: vec![String::from("log.booting")],
            receipts: vec![],
            encounters_resolved: 0,
            prev_miles_traveled: 0.0,
            travel_days: 0,
            partial_travel_days: 0,
            non_travel_days: 0,
            days_with_camp: 0,
            days_with_repair: 0,
            day_state: DayState::default(),
            intent: IntentState::default(),
            wait: WaitState::default(),
            ot_deluxe: OtDeluxeState::default(),
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
            exec_effects: ExecOrderEffects::default(),
            weather_travel_multiplier: 1.0,
            illness_travel_penalty: 1.0,
            illness_days_remaining: 0,
            vehicle: Vehicle::default(),
            breakdown: None,
            weather_state: WeatherState::default(),
            weather_effects: WeatherEffects::default(),
            camp: CampState::default(),
            endgame: EndgameState::default(),
            rotation_travel_days: ROTATION_FORCE_INTERVAL,
            policy: None,
            recent_travel_days: VecDeque::with_capacity(TRAVEL_HISTORY_WINDOW),
            day_reason_history: Vec::new(),
            rotation_backlog: VecDeque::new(),
            rng_bundle: None,
            data: None,
            last_damage: None,
            decision_traces_today: Vec::new(),
            events_today: Vec::new(),
            current_day_record: None,
            current_day_kind: None,
            current_day_reason_tags: Vec::new(),
            current_day_miles: 0.0,
            last_breakdown_part: None,
            terminal_log_key: None,
        }
    };
}

impl Default for GameState {
    fn default() -> Self {
        game_state_defaults!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TravelProgressKind {
    Full,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    const fn new(day: u32, region: Region, season: Season, kind: CrossingKind) -> Self {
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

impl GameState {
    /// Attach a shared RNG bundle for deterministic domain draws.
    pub fn attach_rng_bundle(&mut self, bundle: Rc<RngBundle>) {
        self.rng_bundle = Some(bundle);
    }

    /// Detach any currently attached RNG bundle.
    pub fn detach_rng_bundle(&mut self) {
        self.rng_bundle = None;
    }

    fn health_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.health())
    }

    fn events_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.events())
    }

    fn breakdown_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.breakdown())
    }

    fn crossing_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.crossing())
    }

    fn boss_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.boss())
    }

    fn journey_pace_factor(&self) -> f32 {
        self.journey_breakdown
            .pace_factor
            .get(&self.pace)
            .copied()
            .unwrap_or(1.0)
    }

    fn journey_weather_factor(&self) -> f32 {
        let mult = self.weather_effects.breakdown_mult;
        if mult.is_finite() && mult > 0.0 {
            mult
        } else {
            1.0
        }
    }

    fn journey_fatigue_multiplier(&self) -> f32 {
        if self.journey_wear.fatigue_k <= 0.0 {
            return 1.0;
        }
        let excess = (self.miles_traveled_actual - self.journey_wear.comfort_miles).max(0.0);
        self.journey_wear.fatigue_k.mul_add(excess / 400.0, 1.0)
    }

    pub(crate) fn otdeluxe_policy_overrides(&self) -> OtDeluxePolicyOverride {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return OtDeluxePolicyOverride::default();
        }
        default_otdeluxe_policy().overrides_for(self.region, self.ot_deluxe.season)
    }

    const fn current_version() -> u16 {
        4
    }

    fn build_ot_deluxe_state_from_legacy(&self) -> OtDeluxeState {
        let mut names = Vec::new();
        if !self.party.leader.trim().is_empty() {
            names.push(self.party.leader.clone());
        }
        for companion in &self.party.companions {
            if !companion.trim().is_empty() {
                names.push(companion.clone());
            }
        }
        if names.len() > 5 {
            names.truncate(5);
        }
        while names.len() < 5 {
            let idx = names.len() + 1;
            names.push(format!("Traveler {idx}"));
        }

        let pace = match self.pace {
            PaceId::Steady => OtDeluxePace::Steady,
            PaceId::Heated => OtDeluxePace::Strenuous,
            PaceId::Blitz => OtDeluxePace::Grueling,
        };
        let rations = match self.diet {
            DietId::Mixed => OtDeluxeRations::Filling,
            DietId::Quiet => OtDeluxeRations::Meager,
            DietId::Doom => OtDeluxeRations::BareBones,
        };

        let cash_cents = u32::try_from(self.budget_cents.max(0)).unwrap_or(u32::MAX);

        let calendar = OtDeluxeCalendar::from_day_index(self.day);
        let mut ot_state = OtDeluxeState {
            day: self.day,
            miles_traveled: self.miles_traveled_actual,
            terrain: OtDeluxeTerrain::default(),
            season: calendar.season(),
            calendar,
            party: OtDeluxePartyState::from_names(names),
            inventory: OtDeluxeInventory {
                cash_cents,
                ..OtDeluxeInventory::default()
            },
            pace,
            rations,
            ..OtDeluxeState::default()
        };
        let policy = default_otdeluxe_policy();
        ot_state.route.current_node_index = otdeluxe_trail::node_index_for_miles(
            &policy.trail,
            ot_state.route.variant,
            ot_state.miles_traveled,
        );
        ot_state
    }

    pub fn apply_otdeluxe_start_config(&mut self, occupation: OtDeluxeOccupation) {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if self.ot_deluxe.party.members.is_empty() {
            self.ot_deluxe = self.build_ot_deluxe_state_from_legacy();
        }
        self.ot_deluxe.mods.occupation = Some(occupation);
        if self.day <= 1 && self.day_records.is_empty() {
            let policy = default_otdeluxe_policy();
            self.ot_deluxe.inventory.cash_cents = otdeluxe_starting_cash_cents(occupation, policy);
        }
        self.sync_otdeluxe_trail_distance();
    }

    fn sync_otdeluxe_trail_distance(&mut self) {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        let policy = default_otdeluxe_policy();
        let total =
            otdeluxe_trail::total_miles_for_variant(&policy.trail, self.ot_deluxe.route.variant);
        self.trail_distance = f32::from(total).max(1.0);
    }

    fn otdeluxe_next_prompt_marker(&self) -> Option<(OtDeluxeRoutePrompt, u16)> {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        if self.ot_deluxe.route.pending_prompt.is_some() {
            return None;
        }
        let policy = default_otdeluxe_policy();
        let variant = self.ot_deluxe.route.variant;
        let current_miles = self.miles_traveled_actual;
        let mut candidate: Option<(OtDeluxeRoutePrompt, u16)> = None;

        let mut consider = |prompt: OtDeluxeRoutePrompt, node_index: u8| {
            let Some(marker) =
                otdeluxe_trail::mile_marker_for_node(&policy.trail, variant, node_index)
            else {
                return;
            };
            if f32::from(marker) <= current_miles {
                return;
            }
            match candidate {
                None => candidate = Some((prompt, marker)),
                Some((_, existing)) if marker < existing => candidate = Some((prompt, marker)),
                _ => {}
            }
        };

        if !matches!(
            variant,
            OtDeluxeTrailVariant::SubletteCutoff | OtDeluxeTrailVariant::SubletteAndDallesShortcut
        ) {
            consider(
                OtDeluxeRoutePrompt::SubletteCutoff,
                otdeluxe_trail::SOUTH_PASS_NODE_INDEX,
            );
        }

        if !matches!(
            variant,
            OtDeluxeTrailVariant::DallesShortcut | OtDeluxeTrailVariant::SubletteAndDallesShortcut
        ) {
            consider(
                OtDeluxeRoutePrompt::DallesShortcut,
                otdeluxe_trail::BLUE_MOUNTAINS_NODE_INDEX,
            );
        }

        if self.ot_deluxe.route.dalles_choice.is_none() {
            consider(
                OtDeluxeRoutePrompt::DallesFinal,
                otdeluxe_trail::DALLES_NODE_INDEX,
            );
        }

        candidate
    }

    pub(crate) fn otdeluxe_alive_party_count(&self) -> u16 {
        if !self.ot_deluxe.party.members.is_empty() {
            return self.ot_deluxe.party.alive_count();
        }
        let leader = u16::from(!self.party.leader.trim().is_empty());
        let companions = u16::try_from(self.party.companions.len()).unwrap_or(u16::MAX);
        leader.saturating_add(companions)
    }

    pub(crate) fn apply_otdeluxe_consumption(&mut self) -> u16 {
        let policy = default_otdeluxe_policy();
        let per_person = otdeluxe_rations_food_per_person_scaled(
            self.ot_deluxe.rations,
            self.ot_deluxe.pace,
            policy,
        );
        let alive = self.otdeluxe_alive_party_count();
        let needed = u32::from(per_person).saturating_mul(u32::from(alive));
        let needed_u16 = u16::try_from(needed).unwrap_or(u16::MAX);
        let consumed = needed_u16.min(self.ot_deluxe.inventory.food_lbs);
        self.ot_deluxe.inventory.food_lbs =
            self.ot_deluxe.inventory.food_lbs.saturating_sub(consumed);
        consumed
    }

    pub(crate) fn apply_otdeluxe_health_update(&mut self) -> i32 {
        let policy = default_otdeluxe_policy();
        let total_delta = otdeluxe_health_delta(self, policy);
        let current = i32::from(self.ot_deluxe.health_general);
        let next = (current + total_delta).max(0);
        self.ot_deluxe.health_general = u16::try_from(next).unwrap_or(u16::MAX);
        self.update_otdeluxe_death_imminent(&policy.health);
        total_delta
    }

    fn update_otdeluxe_death_imminent(&mut self, policy: &OtDeluxeHealthPolicy) {
        if self.ot_deluxe.health_general >= policy.death_threshold {
            let grace = policy.death_imminent_grace_days;
            if self.ot_deluxe.death_imminent_days_remaining == 0 {
                self.ot_deluxe.death_imminent_days_remaining = grace;
            } else {
                if self.ot_deluxe.death_imminent_days_remaining > grace {
                    self.ot_deluxe.death_imminent_days_remaining = grace;
                }
                if self.ot_deluxe.death_imminent_days_remaining > 0 {
                    self.ot_deluxe.death_imminent_days_remaining = self
                        .ot_deluxe
                        .death_imminent_days_remaining
                        .saturating_sub(1);
                }
            }
            if grace == 0 || self.ot_deluxe.death_imminent_days_remaining == 0 {
                for member in &mut self.ot_deluxe.party.members {
                    member.alive = false;
                    member.clear_afflictions();
                }
                self.set_ending(Ending::Collapse {
                    cause: CollapseCause::Disease,
                });
            }
        } else if policy.death_imminent_resets_on_recovery_below_threshold {
            self.ot_deluxe.death_imminent_days_remaining = 0;
        }
    }

    pub(crate) fn update_general_strain(&mut self, cfg: &StrainConfig) -> f32 {
        let stats = &self.stats;
        let max_hp = Stats::default().hp;
        let max_sanity = Stats::default().sanity;
        let hp_gap =
            f32::from(u16::try_from((max_hp - stats.hp).clamp(0, max_hp)).unwrap_or(u16::MAX));
        let sanity_gap = f32::from(
            u16::try_from((max_sanity - stats.sanity).clamp(0, max_sanity)).unwrap_or(u16::MAX),
        );
        let pants = f32::from(u16::try_from(stats.pants.clamp(0, 100)).unwrap_or(u16::MAX));
        let malnutrition = f32::from(
            u16::try_from(self.malnutrition_level.min(STARVATION_MAX_STACK)).unwrap_or(u16::MAX),
        );
        let wear_norm = if cfg.vehicle_wear_norm_denom > 0.0 {
            (self.vehicle.wear.max(0.0) / cfg.vehicle_wear_norm_denom).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let weather_severity = cfg
            .weather_severity
            .get(&self.weather_state.today)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        let exec_bonus = self
            .current_order
            .and_then(|order| cfg.exec_order_bonus.get(order.key()).copied())
            .unwrap_or(0.0)
            .max(0.0);
        self.exec_effects.strain_bonus = exec_bonus;

        let weights = &cfg.weights;
        let mut strain = weights.hp.mul_add(hp_gap, weights.sanity * sanity_gap);
        strain = weights.pants.mul_add(pants, strain);
        strain = weights.starvation.mul_add(malnutrition, strain);
        strain = weights.vehicle.mul_add(wear_norm, strain);
        strain = weights.weather.mul_add(weather_severity, strain);
        strain = weights.exec.mul_add(exec_bonus, strain);
        if !strain.is_finite() {
            strain = 0.0;
        }
        self.general_strain = strain.max(0.0);
        self.general_strain
    }

    #[must_use]
    pub fn general_strain_norm(&self, cfg: &StrainConfig) -> f32 {
        if cfg.strain_norm_denom <= 0.0 || !cfg.strain_norm_denom.is_finite() {
            return 0.0;
        }
        (self.general_strain / cfg.strain_norm_denom).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn general_strain_label(&self, cfg: &StrainConfig) -> HealthLabel {
        let norm = self.general_strain_norm(cfg);
        let bounds = &cfg.label_bounds;
        if norm < bounds.good_max {
            HealthLabel::Good
        } else if norm < bounds.fair_max {
            HealthLabel::Fair
        } else if norm < bounds.poor_max {
            HealthLabel::Poor
        } else {
            HealthLabel::VeryPoor
        }
    }

    pub(crate) fn start_of_day(&mut self) {
        if self.day_state.lifecycle.day_initialized {
            return;
        }
        self.day_state.lifecycle.day_initialized = true;
        self.day_state.lifecycle.did_end_of_day = false;
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            self.day_state.lifecycle.suppress_stop_ratio = true;
        }
        self.day_state.travel.traveled_today = false;
        self.day_state.travel.partial_traveled_today = false;
        self.encounters_today = 0;
        self.encounters.occurred_today = false;
        self.prev_miles_traveled = self.miles_traveled_actual;
        self.current_day_kind = None;
        self.current_day_reason_tags.clear();
        self.current_day_miles = 0.0;
        self.decision_traces_today.clear();
        self.events_today.clear();
        self.weather_effects = WeatherEffects::default();
        let day_index = u16::try_from(self.day.saturating_sub(1)).unwrap_or(u16::MAX);
        self.current_day_record = Some(DayRecord::new(day_index, TravelDayKind::NonTravel, 0.0));
        self.terminal_log_key = None;
        self.day_state.lifecycle.log_cursor = u32::try_from(self.logs.len()).unwrap_or(u32::MAX);
        self.day_state.lifecycle.event_seq = 0;
        self.exec_effects = ExecOrderEffects::default();
        self.weather_travel_multiplier = 1.0;
        self.distance_today = 0.0;
        self.distance_today_raw = 0.0;
        self.partial_distance_today = 0.0;
        self.distance_cap_today = 0.0;
        if self.illness_days_remaining == 0 {
            self.illness_travel_penalty = 1.0;
        }
        self.vehicle.tick_breakdown_cooldown();
        self.tick_camp_cooldowns();

        if self.encounter_history.len() >= ENCOUNTER_HISTORY_WINDOW {
            self.encounter_history.pop_front();
        }
        self.encounter_history.push_back(0);

        if self.encounter_cooldown > 0 {
            self.encounter_cooldown -= 1;
        }
    }

    const fn next_event_id(&mut self) -> EventId {
        let seq = self.day_state.lifecycle.event_seq;
        let id = EventId::new(self.day, seq);
        self.day_state.lifecycle.event_seq = seq.saturating_add(1);
        id
    }

    pub(crate) fn push_event(
        &mut self,
        kind: EventKind,
        severity: EventSeverity,
        tags: DayTagSet,
        ui_surface_hint: Option<UiSurfaceHint>,
        ui_key: Option<String>,
        payload: serde_json::Value,
    ) {
        let day = self.day;
        let id = self.next_event_id();
        self.events_today.push(Event {
            id,
            day,
            kind,
            severity,
            tags,
            ui_surface_hint,
            ui_key,
            payload,
        });
    }

    pub(crate) fn push_log(&mut self, log_key: impl Into<String>) {
        let key = log_key.into();
        self.logs.push(key.clone());
        let kind = EventKind::LegacyLogKey;
        let severity = EventSeverity::Info;
        let tags = DayTagSet::new();
        let surface = Some(UiSurfaceHint::Log);
        let payload = serde_json::Value::Null;
        self.push_event(kind, severity, tags, surface, Some(key), payload);
    }

    pub(crate) fn tick_exec_order_state(&mut self) {
        if let Some(order) = self.current_order {
            self.apply_exec_order_effects(order);
            if self.exec_order_days_remaining > 0 {
                self.exec_order_days_remaining -= 1;
            }
            if self.exec_order_days_remaining == 0 {
                self.logs
                    .push(format!("{}{}", LOG_EXEC_END_PREFIX, order.key()));
                self.push_event(
                    EventKind::ExecOrderEnded,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::json!({ "order": order.key() }),
                );
                self.current_order = None;
                let cooldown = self
                    .events_rng()
                    .map_or(EXEC_ORDER_MIN_COOLDOWN, |mut rng| {
                        rng.gen_range(EXEC_ORDER_MIN_COOLDOWN..=EXEC_ORDER_MAX_COOLDOWN)
                    });
                self.exec_order_cooldown = cooldown;
            }
            return;
        }

        if self.exec_order_cooldown > 0 {
            self.exec_order_cooldown -= 1;
            return;
        }

        let behind_active = self.behind_schedule_multiplier() > 1.0;
        let mut exec_chance = EXEC_ORDER_DAILY_CHANCE;
        if behind_active {
            exec_chance *= 0.5;
        }

        let next_order = if let Some(mut rng) = self.events_rng()
            && rng.r#gen::<f32>() < exec_chance
        {
            let idx = rng.gen_range(0..ExecOrder::ALL.len());
            let order = ExecOrder::ALL[idx];
            let duration = rng.gen_range(EXEC_ORDER_MIN_DURATION..=EXEC_ORDER_MAX_DURATION);
            let roll = u32::try_from(idx).unwrap_or(0);
            let candidates = ExecOrder::ALL
                .iter()
                .map(|candidate| WeightedCandidate {
                    id: candidate.key().to_string(),
                    base_weight: 1.0,
                    multipliers: Vec::new(),
                    final_weight: 1.0,
                })
                .collect();
            let trace = EventDecisionTrace {
                pool_id: String::from("dystrail.exec_order"),
                roll: RollValue::U32(roll),
                candidates,
                chosen_id: order.key().to_string(),
            };
            Some((order, duration, trace))
        } else {
            None
        };

        if let Some((order, duration, trace)) = next_order {
            self.current_order = Some(order);
            self.exec_order_days_remaining = duration;
            self.logs
                .push(format!("{}{}", LOG_EXEC_START_PREFIX, order.key()));
            self.push_event(
                EventKind::ExecOrderStarted,
                EventSeverity::Info,
                DayTagSet::new(),
                None,
                None,
                serde_json::json!({ "order": order.key(), "duration_days": duration }),
            );
            self.apply_exec_order_effects(order);
            if self.exec_order_days_remaining > 0 {
                self.exec_order_days_remaining -= 1;
            }
            self.decision_traces_today.push(trace);
        }
    }

    fn apply_exec_order_effects(&mut self, order: ExecOrder) {
        match order {
            ExecOrder::Shutdown => {
                self.exec_effects.morale_delta -= 1;
                self.exec_effects.supplies_delta -= 1;
            }
            ExecOrder::TravelBanLite => {
                self.exec_effects.sanity_delta -= 1;
                self.exec_effects.travel_multiplier *= EXEC_ORDER_SPEED_BONUS;
            }
            ExecOrder::BookPanic => {
                if self.stats.morale < 7 {
                    self.exec_effects.sanity_delta -= 1;
                }
            }
            ExecOrder::TariffTsunami => {
                if !self.inventory.has_tag("legal_fund") {
                    self.exec_effects.supplies_delta -= 1;
                }
            }
            ExecOrder::DoEEliminated => {
                self.exec_effects.morale_delta -= 1;
            }
            ExecOrder::WarDeptReorg => {
                self.exec_effects.breakdown_bonus += EXEC_ORDER_BREAKDOWN_BONUS;
            }
        }
        self.cap_exec_order_effects();
    }

    const fn cap_exec_order_effects(&mut self) {
        self.exec_effects.travel_multiplier = self
            .exec_effects
            .travel_multiplier
            .clamp(EXEC_TRAVEL_MULTIPLIER_CLAMP_MIN, WEATHER_DEFAULT_SPEED);
        self.exec_effects.breakdown_bonus = self
            .exec_effects
            .breakdown_bonus
            .clamp(PROBABILITY_FLOOR, EXEC_BREAKDOWN_BONUS_CLAMP_MAX);
    }

    pub(crate) fn end_of_day(&mut self) {
        if self.day_state.lifecycle.did_end_of_day {
            return;
        }
        if self.mechanical_policy == MechanicalPolicyId::DystrailLegacy {
            self.terminal_log_key = self.failure_log_key().map(str::to_string);
        } else {
            self.terminal_log_key = None;
        }
        self.update_encounter_history();
        let miles_delta = self.compute_day_progress();
        self.assert_travel_consistency(miles_delta);
        self.apply_conservative_travel_bonus();

        let day_kind = self.resolve_day_kind();
        let day_kind = self.apply_stop_ratio_floor(day_kind);
        self.finalize_day(day_kind);
        self.unlock_aggressive_boss_ready();
    }

    fn update_encounter_history(&mut self) {
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
    }

    fn compute_day_progress(&mut self) -> f32 {
        let computed_miles_today = self.distance_today.max(self.distance_today_raw);
        self.enforce_aggressive_delay_cap(computed_miles_today);
        let miles_delta = (self.miles_traveled_actual - self.prev_miles_traveled).max(0.0);
        let needs_backfill = self.current_day_kind.is_none()
            || (matches!(self.current_day_kind, Some(TravelDayKind::NonTravel))
                && miles_delta > 0.0);
        if needs_backfill {
            if miles_delta > 0.0 {
                self.day_state.travel.partial_traveled_today = true;
            }
            let fallback_kind = if self.day_state.travel.traveled_today {
                TravelDayKind::Travel
            } else if self.day_state.travel.partial_traveled_today {
                TravelDayKind::Partial
            } else {
                TravelDayKind::NonTravel
            };
            self.record_travel_day(fallback_kind, 0.0, "");
            if matches!(
                fallback_kind,
                TravelDayKind::Travel | TravelDayKind::Partial
            ) {
                self.current_day_miles = miles_delta;
                self.distance_today = self.distance_today.max(miles_delta);
                self.distance_today_raw = self.distance_today_raw.max(miles_delta);
            }
        }
        miles_delta
    }

    fn assert_travel_consistency(&self, miles_delta: f32) {
        if !self.day_state.travel.traveled_today && !self.day_state.travel.partial_traveled_today {
            assert!(
                miles_delta <= 0.01,
                "distance advanced on non-travel day (delta {miles_delta:.2})"
            );
        }
        if self.day_state.travel.partial_traveled_today {
            let advanced = (self.miles_traveled_actual - self.prev_miles_traveled) > 0.0;
            let at_goal = (self.trail_distance - self.miles_traveled_actual).abs() <= f32::EPSILON;
            debug_assert!(
                advanced || at_goal,
                "partial travel day without distance gain"
            );
        }
    }

    fn apply_conservative_travel_bonus(&mut self) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if !self.mode.is_deep()
            || !matches!(self.policy, Some(PolicyKind::Conservative))
            || self.current_day_miles <= 0.0
        {
            return;
        }

        let had_repair = self
            .current_day_reason_tags
            .iter()
            .any(|tag| tag.contains("repair"));
        let had_crossing = self
            .current_day_reason_tags
            .iter()
            .any(|tag| tag.starts_with("crossing") || tag == "detour");
        if had_repair || had_crossing {
            return;
        }

        let bonus = self.current_day_miles * 0.03;
        if bonus <= 0.0 {
            return;
        }
        let cap = if self.distance_cap_today > 0.0 {
            self.distance_cap_today
        } else {
            self.distance_today.max(self.distance_today_raw)
        };
        let available = if cap > self.current_day_miles {
            cap - self.current_day_miles
        } else {
            0.0
        };
        let applied = bonus.min(available);
        if applied > 0.0 {
            let credited = self.apply_travel_progress(applied, TravelProgressKind::Full);
            if credited > 0.0 {
                self.current_day_miles += credited;
                self.distance_today = self.distance_today.max(self.current_day_miles);
                self.distance_today_raw = self.distance_today_raw.max(self.current_day_miles);
                self.partial_distance_today = self
                    .partial_distance_today
                    .max(credited)
                    .min(self.distance_today);
            }
        }
    }

    fn resolve_day_kind(&self) -> TravelDayKind {
        self.current_day_kind
            .unwrap_or(if self.day_state.travel.traveled_today {
                TravelDayKind::Travel
            } else if self.day_state.travel.partial_traveled_today {
                TravelDayKind::Partial
            } else {
                TravelDayKind::NonTravel
            })
    }

    fn apply_stop_ratio_floor(&mut self, mut day_kind: TravelDayKind) -> TravelDayKind {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return day_kind;
        }
        if matches!(day_kind, TravelDayKind::NonTravel)
            && !self.day_state.lifecycle.suppress_stop_ratio
        {
            let total_days = self.travel_days + self.partial_travel_days + self.non_travel_days;
            if total_days > 0 {
                let travel_days = self.travel_days + self.partial_travel_days;
                let ratio = f64::from(travel_days) / f64::from(total_days);
                if ratio < 0.90_f64 {
                    self.revert_current_day_record();
                    let baseline = self.distance_today.max(self.distance_today_raw);
                    let partial = day_accounting::partial_day_miles(self, baseline);
                    self.record_travel_day(TravelDayKind::Partial, partial, "stop_cap");
                    self.distance_today = self.distance_today.max(partial);
                    self.distance_today_raw = self.distance_today_raw.max(partial);
                    self.partial_distance_today = self.partial_distance_today.max(partial);
                    day_kind = TravelDayKind::Partial;
                }
            }
        }
        day_kind
    }

    fn finalize_day(&mut self, day_kind: TravelDayKind) {
        if self.rotation_travel_days >= self.rotation_force_interval() {
            self.encounters.force_rotation_pending = true;
            self.rotation_travel_days = 0;
        }
        if self.recent_travel_days.len() >= TRAVEL_HISTORY_WINDOW {
            self.recent_travel_days.pop_front();
        }
        self.recent_travel_days.push_back(day_kind);
        if let Some(record) = self.current_day_record.as_mut() {
            record.kind = day_kind;
            record.miles = self.current_day_miles;
        }
        let reason_entry = if self.current_day_reason_tags.is_empty() {
            String::new()
        } else {
            self.current_day_reason_tags.join(";")
        };
        self.day_reason_history.push(reason_entry);
        self.current_day_reason_tags.clear();
        if let Some(record) = self.current_day_record.take() {
            self.day_records.push(record);
        }
        self.recompute_day_counters();
        self.current_day_miles = 0.0;
        self.current_day_kind = None;
        self.day_state.lifecycle.suppress_stop_ratio = false;
        self.day_state.lifecycle.day_initialized = false;
        self.day_state.lifecycle.did_end_of_day = true;
        self.day = self.day.saturating_add(1);
        if self.ot_deluxe.day.saturating_add(1) == self.day {
            self.ot_deluxe.advance_days(1);
        } else {
            self.ot_deluxe.day = self.day;
            self.ot_deluxe.calendar = OtDeluxeCalendar::from_day_index(self.day);
            self.ot_deluxe.season = self.ot_deluxe.calendar.season();
        }
        self.ot_deluxe.miles_traveled = self.miles_traveled_actual;
        let policy = default_otdeluxe_policy();
        self.ot_deluxe.route.current_node_index = otdeluxe_trail::node_index_for_miles(
            &policy.trail,
            self.ot_deluxe.route.variant,
            self.ot_deluxe.miles_traveled,
        );
    }

    fn unlock_aggressive_boss_ready(&mut self) {
        if self.mechanical_policy != MechanicalPolicyId::DystrailLegacy {
            return;
        }
        if self.mode.is_deep()
            && matches!(self.policy, Some(PolicyKind::Aggressive))
            && !self.boss.readiness.ready
            && !self.boss.outcome.attempted
            && self.miles_traveled_actual >= DEEP_AGGRESSIVE_BOSS_BIAS_MILES
        {
            self.boss.readiness.ready = true;
            self.boss.readiness.reached = true;
        }
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
            self.encounters.occurred_today = false;
        }
    }

    fn apply_travel_wear_scaled(&mut self, scale: f32) {
        if scale <= 0.0 {
            return;
        }
        let base = self.journey_wear.base;
        if base <= 0.0 {
            return;
        }
        let wear_delta = base
            * self.journey_pace_factor()
            * self.journey_weather_factor()
            * self.journey_fatigue_multiplier()
            * scale;
        if wear_delta <= 0.0 {
            return;
        }
        self.vehicle.apply_scaled_wear(wear_delta);
    }

    pub(crate) fn apply_travel_wear_for_day(&mut self, baseline_miles: f32) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if self.current_day_miles <= 0.0 {
            return;
        }
        let baseline = if baseline_miles > 0.0 {
            baseline_miles
        } else {
            self.distance_today.max(self.distance_today_raw)
        };
        if baseline <= 0.0 {
            return;
        }
        let scale = (self.current_day_miles / baseline).clamp(0.0, 1.0);
        self.apply_travel_wear_scaled(scale);
    }

    fn revert_current_day_record(&mut self) {
        if matches!(
            self.current_day_kind,
            Some(TravelDayKind::Travel | TravelDayKind::Partial)
        ) {
            self.rotation_travel_days = self.rotation_travel_days.saturating_sub(1);
        }
        self.current_day_kind = None;
        if self.current_day_reason_tags.iter().any(|tag| tag == "camp") {
            self.days_with_camp = self.days_with_camp.saturating_sub(1);
        }
        if self
            .current_day_reason_tags
            .iter()
            .any(|tag| tag == "repair")
        {
            self.days_with_repair = self.days_with_repair.saturating_sub(1);
        }
        if let Some(record) = self.current_day_record.as_mut() {
            record.kind = TravelDayKind::NonTravel;
            record.miles = 0.0;
            record.tags.clear();
        }
        self.current_day_reason_tags.clear();
        self.current_day_miles = 0.0;
    }

    pub(crate) fn apply_travel_progress(&mut self, distance: f32, kind: TravelProgressKind) -> f32 {
        if distance <= 0.0 {
            return 0.0;
        }
        let mut prompt_reached = None;
        let mut distance = distance;
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            self.sync_otdeluxe_trail_distance();
            if let Some((prompt, marker)) = self.otdeluxe_next_prompt_marker() {
                let remaining_to_marker = f32::from(marker) - self.miles_traveled_actual;
                if remaining_to_marker > 0.0 && distance + f32::EPSILON >= remaining_to_marker {
                    distance = remaining_to_marker;
                    prompt_reached = Some(prompt);
                }
            }
        }
        let remaining = (self.trail_distance - self.miles_traveled_actual).max(0.0);
        if remaining <= 0.0 {
            return 0.0;
        }
        let applied = distance.min(remaining);
        let before = self.miles_traveled_actual;
        self.miles_traveled_actual += applied;
        self.miles_traveled = (self.miles_traveled + applied).min(self.trail_distance);
        let advanced = self.miles_traveled_actual > before;
        if advanced {
            match kind {
                TravelProgressKind::Full => self.day_state.travel.traveled_today = true,
                TravelProgressKind::Partial => self.day_state.travel.partial_traveled_today = true,
            }
            if self.ending.is_none() && self.miles_traveled_actual >= self.trail_distance {
                match self.mechanical_policy {
                    MechanicalPolicyId::DystrailLegacy => {
                        self.boss.readiness.ready = true;
                        self.boss.readiness.reached = true;
                    }
                    MechanicalPolicyId::OtDeluxe90s => {
                        self.set_ending(Ending::BossVictory);
                    }
                }
            }
        }
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            self.ot_deluxe.miles_traveled = self.miles_traveled_actual;
            let policy = default_otdeluxe_policy();
            self.ot_deluxe.route.current_node_index = otdeluxe_trail::node_index_for_miles(
                &policy.trail,
                self.ot_deluxe.route.variant,
                self.ot_deluxe.miles_traveled,
            );
            self.ot_deluxe.terrain = if otdeluxe_trail::is_mountain_for_miles(
                &policy.trail,
                self.ot_deluxe.route.variant,
                self.ot_deluxe.miles_traveled,
            ) {
                OtDeluxeTerrain::Mountains
            } else {
                OtDeluxeTerrain::Plains
            };
            if let Some(prompt) = prompt_reached {
                self.ot_deluxe.route.pending_prompt = Some(prompt);
                self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Stopped;
            }
        }
        applied
    }

    fn recompute_day_counters(&mut self) {
        let metrics = day_accounting::compute_day_ledger_metrics(&self.day_records);
        self.travel_days = metrics.travel_days;
        self.partial_travel_days = metrics.partial_days;
        self.non_travel_days = metrics.non_travel_days;
    }

    #[must_use]
    pub fn ledger_metrics(&self) -> DayLedgerMetrics {
        day_accounting::compute_day_ledger_metrics(&self.day_records)
    }

    pub(crate) fn reset_today_progress(&mut self) {
        let day_progress = (self.miles_traveled_actual - self.prev_miles_traveled).max(0.0);
        if day_progress > 0.0 {
            self.miles_traveled_actual -= day_progress;
            self.miles_traveled = self.miles_traveled_actual.min(self.trail_distance);
            if self.miles_traveled_actual < self.trail_distance {
                self.boss.readiness.ready = false;
                self.boss.readiness.reached = false;
            }
        }
        self.revert_current_day_record();
        self.distance_today = 0.0;
        self.distance_today_raw = 0.0;
        self.partial_distance_today = 0.0;
        self.day_state.travel.traveled_today = false;
        self.day_state.travel.partial_traveled_today = false;
    }

    pub(crate) const fn clear_today_travel_distance(&mut self) {
        self.distance_today = 0.0;
        self.distance_today_raw = 0.0;
        self.partial_distance_today = 0.0;
        self.distance_cap_today = 0.0;
    }

    fn rotation_force_interval(&self) -> u32 {
        let mut interval = ROTATION_FORCE_INTERVAL;
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Conservative)) {
            interval = interval.saturating_sub(2).max(1);
        }
        interval
    }

    fn enforce_aggressive_delay_cap(&mut self, computed_miles: f32) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if self.day_state.travel.traveled_today || self.day_state.travel.partial_traveled_today {
            return;
        }
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return;
        }
        let full_stops = self
            .recent_travel_days
            .iter()
            .rev()
            .take(AGGRESSIVE_STOP_WINDOW_DAYS)
            .filter(|kind| matches!(kind, TravelDayKind::NonTravel))
            .count();
        if full_stops < AGGRESSIVE_STOP_CAP {
            return;
        }

        let baseline = if computed_miles > 0.0 {
            computed_miles
        } else if self.features.travel_v2 {
            TRAVEL_V2_BASE_DISTANCE
        } else {
            TRAVEL_CLASSIC_BASE_DISTANCE
        };
        let partial = (baseline * TRAVEL_PARTIAL_RATIO).max(TRAVEL_PARTIAL_MIN_DISTANCE);
        self.reset_today_progress();
        self.record_travel_day(TravelDayKind::Partial, partial, "stop_cap");
        self.distance_today = partial;
        self.distance_today_raw = partial;
        self.partial_distance_today = partial;
        self.current_day_miles = partial;
        self.day_state.travel.partial_traveled_today = true;
        self.day_state.travel.traveled_today = false;
        let new_wear = (self.vehicle.wear - self.journey_wear.base).max(0.0);
        self.vehicle.set_wear(new_wear);
        self.push_log(LOG_TRAVEL_PARTIAL);
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
            if matches!(kind, TravelDayKind::Travel | TravelDayKind::Partial) {
                traveled += 1;
            }
        }
        if total == 0 {
            return WEATHER_DEFAULT_SPEED;
        }
        let traveled_u16 = u16::try_from(traveled).unwrap_or(u16::MAX);
        let total_u16 = u16::try_from(total).unwrap_or(u16::MAX);
        f32::from(traveled_u16) / f32::from(total_u16)
    }

    fn apply_partial_travel_credit(
        &mut self,
        distance: f32,
        log_key: &'static str,
        reason_tag: &str,
    ) {
        if distance <= 0.0 {
            return;
        }
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if self.day_state.travel.traveled_today && !self.day_state.travel.partial_traveled_today {
            self.reset_today_progress();
        }
        self.distance_today += distance;
        self.distance_today_raw += distance;
        self.partial_distance_today = self.partial_distance_today.max(distance);
        self.record_travel_day(TravelDayKind::Partial, distance, reason_tag);
        self.push_log(log_key);
    }

    pub(crate) fn apply_rest_travel_credit(&mut self) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        self.apply_partial_travel_credit(REST_TRAVEL_CREDIT_MILES, LOG_TRAVEL_REST_CREDIT, "camp");
    }

    fn apply_delay_travel_credit(&mut self, reason_tag: &str) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        let miles = day_accounting::partial_day_miles(self, 0.0).max(DELAY_TRAVEL_CREDIT_MILES);
        self.apply_partial_travel_credit(miles, LOG_TRAVEL_DELAY_CREDIT, reason_tag);
    }

    fn apply_classic_field_repair_guard(&mut self) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        let partial = day_accounting::partial_day_miles(self, 0.0);
        self.apply_partial_travel_credit(
            partial,
            LOG_VEHICLE_FIELD_REPAIR_GUARD,
            "field_repair_guard",
        );
        self.vehicle.ensure_health_floor(VEHICLE_EMERGENCY_HEAL);
        self.vehicle.wear = (self.vehicle.wear - CLASSIC_FIELD_REPAIR_WEAR_REDUCTION).max(0.0);
        let field_repair_cost = CLASSIC_FIELD_REPAIR_COST_CENTS;
        let paid = field_repair_cost.min(self.budget_cents.max(0));
        self.budget_cents -= paid;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += paid;
        self.breakdown = None;
        self.day_state.travel.travel_blocked = false;
        self.last_breakdown_part = None;
    }

    fn try_emergency_limp_guard(&mut self) -> bool {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return false;
        }
        if self.mode == GameMode::Classic && matches!(self.policy, Some(PolicyKind::Balanced)) {
            return false;
        }
        if self.miles_traveled_actual < 1_850.0 {
            return false;
        }
        if (self.miles_traveled_actual - self.endgame.last_limp_mile) < EMERGENCY_LIMP_MILE_WINDOW {
            return false;
        }

        let partial = day_accounting::partial_day_miles(self, 0.0);
        self.apply_partial_travel_credit(partial, LOG_VEHICLE_EMERGENCY_LIMP, "emergency_limp");
        self.vehicle.ensure_health_floor(VEHICLE_EMERGENCY_HEAL);
        self.vehicle.wear = (self.vehicle.wear - EMERGENCY_LIMP_WEAR_REDUCTION).max(0.0);
        let limp_cost = EMERGENCY_LIMP_REPAIR_COST_CENTS;
        let paid = limp_cost.min(self.budget_cents.max(0));
        self.budget_cents -= paid;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += paid;
        self.endgame.last_limp_mile = self.miles_traveled_actual;
        self.breakdown = None;
        self.day_state.travel.travel_blocked = false;
        self.last_breakdown_part = None;
        true
    }

    fn try_deep_aggressive_field_repair(&mut self) -> bool {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return false;
        }
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return false;
        }
        if self.miles_traveled_actual < 1_500.0 {
            return false;
        }
        let roll = self
            .breakdown_rng()
            .map_or(1.0, |mut rng| rng.r#gen::<f32>());
        let mut success_chance: f32 = 0.65;
        success_chance = success_chance.clamp(0.0, 1.0);
        if roll >= success_chance {
            return false;
        }

        let partial = day_accounting::partial_day_miles(self, 0.0);
        self.apply_partial_travel_credit(partial, LOG_DEEP_AGGRESSIVE_FIELD_REPAIR, "field_repair");
        self.vehicle.ensure_health_floor(VEHICLE_EMERGENCY_HEAL);
        self.vehicle.wear = (self.vehicle.wear - EMERGENCY_LIMP_WEAR_REDUCTION).max(0.0);
        let limp_cost = EMERGENCY_LIMP_REPAIR_COST_CENTS;
        let paid = limp_cost.min(self.budget_cents.max(0));
        self.budget_cents -= paid;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += paid;
        self.breakdown = None;
        self.day_state.travel.travel_blocked = false;
        self.last_breakdown_part = None;
        true
    }

    pub(crate) fn add_day_reason_tag(&mut self, tag: &str) {
        let trimmed = tag.trim();
        if trimmed.is_empty()
            || self
                .current_day_reason_tags
                .iter()
                .any(|existing| existing == trimmed)
        {
            return;
        }
        if trimmed == "camp" {
            self.days_with_camp = self.days_with_camp.saturating_add(1);
        } else if trimmed == "repair" {
            self.days_with_repair = self.days_with_repair.saturating_add(1);
        }
        self.current_day_reason_tags.push(trimmed.to_string());
        if let Some(record) = self.current_day_record.as_mut() {
            record.push_tag(DayTag::new(trimmed));
        }
    }

    pub(crate) fn record_travel_day(
        &mut self,
        kind: TravelDayKind,
        miles_earned: f32,
        reason_tag: &str,
    ) -> TravelDayKind {
        let (recorded_kind, _) = day_accounting::record_travel_day(self, kind, miles_earned);
        if let Some(record) = self.current_day_record.as_mut() {
            record.kind = recorded_kind;
            record.miles = self.current_day_miles;
        }
        if !reason_tag.is_empty() {
            self.add_day_reason_tag(reason_tag);
        }
        recorded_kind
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
    const fn encounter_reroll_penalty(&self) -> f32 {
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Conservative)) {
            TRAVEL_RATIO_DEFAULT.min(WEATHER_DEFAULT_SPEED)
        } else {
            ENCOUNTER_REROLL_PENALTY
        }
    }

    fn encounter_unique_ratio(&self, window_days: u32) -> f32 {
        if window_days == 0 {
            return 1.0;
        }
        let cutoff = self.day.saturating_sub(window_days);
        let mut unique: HashSet<&str> = HashSet::new();
        let mut total = 0_u32;
        for entry in self.recent_encounters.iter().rev() {
            if entry.day <= cutoff {
                break;
            }
            total = total.saturating_add(1);
            unique.insert(entry.id.as_str());
        }
        if total == 0 {
            1.0
        } else {
            let unique_count = u16::try_from(unique.len()).unwrap_or(u16::MAX);
            let total_days = u16::try_from(total).unwrap_or(u16::MAX);
            f32::from(unique_count) / f32::from(total_days)
        }
    }

    const fn set_ending(&mut self, ending: Ending) {
        if self.ending.is_none() {
            self.ending = Some(ending);
        }
    }

    pub(crate) const fn mark_damage(&mut self, cause: DamageCause) {
        self.last_damage = Some(cause);
    }

    #[must_use]
    pub const fn vehicle_health(&self) -> f32 {
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
    const fn total_spares(&self) -> i32 {
        self.inventory.total_spares()
    }

    pub(crate) fn apply_starvation_tick(&mut self) {
        if self.stats.supplies > 0 {
            if self.starvation_days > 0 {
                self.push_log(LOG_STARVATION_RELIEF);
            }
            self.starvation_days = 0;
            self.malnutrition_level = 0;
            self.guards.starvation_backstop_used = false;
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
        self.push_log(LOG_STARVATION_TICK);
        if self.stats.hp <= 0 {
            if !self.guards.starvation_backstop_used {
                self.guards.starvation_backstop_used = true;
                self.stats.hp = 1;
                self.day_state.rest.rest_requested = true;
                self.push_log(LOG_STARVATION_BACKSTOP);
                return;
            }
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Hunger,
            });
        }
    }

    pub(crate) fn roll_daily_illness(&mut self) {
        if self.disease_cooldown > 0 {
            self.disease_cooldown -= 1;
        }

        if self.illness_days_remaining > 0 {
            self.illness_travel_penalty = ILLNESS_TRAVEL_PENALTY;
            self.stats.hp -= DISEASE_TICK_HP_LOSS;
            self.stats.sanity -= DISEASE_TICK_SANITY_LOSS;
            self.stats.supplies = (self.stats.supplies - DISEASE_SUPPLY_PENALTY).max(0);
            self.day_state.rest.rest_requested = true;
            self.mark_damage(DamageCause::Disease);
            self.push_log(LOG_DISEASE_TICK);
            let recovering = self.illness_days_remaining <= 1;
            self.illness_days_remaining = self.illness_days_remaining.saturating_sub(1);
            if recovering {
                self.clear_illness_penalty();
                self.disease_cooldown = DISEASE_COOLDOWN_DAYS;
            }
            return;
        }

        let behind_active = self.behind_schedule_multiplier() > 1.0;
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
        if behind_active {
            chance *= 0.5;
        }

        let roll = self.health_rng().map_or(1.0, |mut rng| rng.r#gen::<f32>());
        if roll >= chance {
            return;
        }

        let duration = self
            .health_rng()
            .map_or(DISEASE_DURATION_RANGE.0, |mut rng| {
                rng.gen_range(DISEASE_DURATION_RANGE.0..=DISEASE_DURATION_RANGE.1)
            });
        self.illness_days_remaining = duration;
        self.stats.hp -= DISEASE_HP_PENALTY;
        self.stats.sanity -= DISEASE_SANITY_PENALTY;
        self.stats.supplies = (self.stats.supplies - DISEASE_SUPPLY_PENALTY).max(0);
        self.disease_cooldown = DISEASE_COOLDOWN_DAYS;
        self.day_state.rest.rest_requested = true;
        self.illness_travel_penalty = ILLNESS_TRAVEL_PENALTY;
        self.mark_damage(DamageCause::Disease);
        self.push_log(LOG_DISEASE_HIT);
    }

    pub(crate) fn tick_otdeluxe_afflictions(&mut self) -> Option<OtDeluxeAfflictionOutcome> {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        let catalog = DiseaseCatalog::default_catalog();
        self.tick_otdeluxe_afflictions_with_catalog(catalog)
    }

    fn tick_otdeluxe_afflictions_with_catalog(
        &mut self,
        catalog: &DiseaseCatalog,
    ) -> Option<OtDeluxeAfflictionOutcome> {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        let policy = default_otdeluxe_policy();
        let bundle = self.rng_bundle.clone()?;
        let mut rng = bundle.health();
        self.apply_otdeluxe_daily_afflictions(catalog, &mut *rng, policy);
        if self.ot_deluxe.party.alive_count() == 0 {
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Disease,
            });
            return None;
        }
        self.roll_otdeluxe_affliction_with_catalog(catalog, &mut *rng, policy)
    }

    fn apply_otdeluxe_daily_afflictions(
        &mut self,
        catalog: &DiseaseCatalog,
        rng: &mut impl Rng,
        policy: &OtDeluxe90sPolicy,
    ) {
        let weather_today = self.weather_state.today;
        let occupation = self.ot_deluxe.mods.occupation;
        let pace = self.ot_deluxe.pace;
        let rations = self.ot_deluxe.rations;

        let mut travel_mult = 1.0;
        {
            let OtDeluxeState {
                party,
                health_general,
                inventory,
                travel,
                ..
            } = &mut self.ot_deluxe;
            for member in &mut party.members {
                if !member.alive {
                    continue;
                }
                let mut died = false;
                if member.sick_days_remaining > 0
                    && let Some(id) = member.illness_id.as_deref()
                    && let Some(disease) = catalog.find_by_id(id)
                {
                    travel_mult *= apply_otdeluxe_disease_effects(
                        health_general,
                        inventory,
                        &disease.daily_tick_effects,
                    );
                    if let Some(model) = disease.fatality_model.as_ref() {
                        let context = OtDeluxeFatalityContext {
                            health_general: *health_general,
                            pace,
                            rations,
                            weather: weather_today,
                            occupation,
                        };
                        died = otdeluxe_roll_disease_fatality(model, rng, context, policy);
                    }
                }
                if !died
                    && member.injured_days_remaining > 0
                    && let Some(id) = member.injury_id.as_deref()
                    && let Some(disease) = catalog.find_by_id(id)
                {
                    travel_mult *= apply_otdeluxe_disease_effects(
                        health_general,
                        inventory,
                        &disease.daily_tick_effects,
                    );
                    if let Some(model) = disease.fatality_model.as_ref() {
                        let context = OtDeluxeFatalityContext {
                            health_general: *health_general,
                            pace,
                            rations,
                            weather: weather_today,
                            occupation,
                        };
                        died = otdeluxe_roll_disease_fatality(model, rng, context, policy);
                    }
                }
                if died {
                    member.alive = false;
                    member.clear_afflictions();
                }
            }
            party.tick_afflictions();
            travel.disease_speed_mult = sanitize_disease_multiplier(travel_mult);
        }
    }

    fn roll_otdeluxe_affliction_with_catalog(
        &mut self,
        catalog: &DiseaseCatalog,
        rng: &mut impl Rng,
        policy: &OtDeluxe90sPolicy,
    ) -> Option<OtDeluxeAfflictionOutcome> {
        let probability =
            otdeluxe_affliction_probability(self.ot_deluxe.health_general, &policy.affliction);
        if probability <= 0.0 {
            return None;
        }
        let roll: f32 = rng.r#gen();
        if roll >= probability {
            return None;
        }
        let overrides = policy.overrides_for(self.region, self.ot_deluxe.season);
        let (kind, trace) = roll_otdeluxe_affliction_kind(&policy.affliction, &overrides, rng);
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }
        let disease_kind = match kind {
            OtDeluxeAfflictionKind::Illness => DiseaseKind::Illness,
            OtDeluxeAfflictionKind::Injury => DiseaseKind::Injury,
        };
        let (disease, trace) = catalog.pick_by_kind_with_trace(disease_kind, rng);
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }
        let duration = disease.map_or_else(
            || otdeluxe_affliction_duration(kind, &policy.affliction),
            |selected| selected.duration_for(&policy.affliction),
        );
        let disease_id = disease.map(|selected| selected.id.as_str());
        let mut outcome = self
            .ot_deluxe
            .party
            .apply_affliction_random(rng, kind, duration, disease_id);
        if let (Some(selected), Some(ref mut result)) = (disease, outcome.as_mut()) {
            result.display_key = Some(selected.display_key.clone());
            result.disease_id = Some(selected.id.clone());
            if !result.died {
                let combined = self.apply_otdeluxe_disease_onset(selected);
                self.ot_deluxe.travel.disease_speed_mult = sanitize_disease_multiplier(
                    self.ot_deluxe.travel.disease_speed_mult * combined,
                );
            }
        }
        if let Some(ref result) = outcome {
            self.push_event(
                EventKind::AfflictionTriggered,
                EventSeverity::Warning,
                DayTagSet::new(),
                None,
                None,
                serde_json::json!({
                    "member_index": result.member_index,
                    "kind": result.kind,
                    "disease_id": result.disease_id,
                    "display_key": result.display_key,
                    "died": result.died
                }),
            );
        }
        if self.ot_deluxe.party.alive_count() == 0 {
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Disease,
            });
        }
        outcome
    }

    fn apply_otdeluxe_disease_onset(&mut self, disease: &DiseaseDef) -> f32 {
        let onset_mult = apply_otdeluxe_disease_effects(
            &mut self.ot_deluxe.health_general,
            &mut self.ot_deluxe.inventory,
            &disease.onset_effects,
        );
        let daily_mult = apply_otdeluxe_disease_effects(
            &mut self.ot_deluxe.health_general,
            &mut self.ot_deluxe.inventory,
            &disease.daily_tick_effects,
        );
        onset_mult * daily_mult
    }

    pub(crate) fn tick_ally_attrition(&mut self) {
        if self.stats.allies <= 0 {
            return;
        }
        let trigger = self
            .events_rng()
            .is_some_and(|mut rng| rng.r#gen::<f32>() <= ALLY_ATTRITION_CHANCE);
        if trigger {
            self.stats.allies -= 1;
            self.stats.morale -= 1;
            self.push_log(LOG_ALLY_LOST);
            if self.stats.allies == 0 {
                self.stats.sanity -= 2;
                self.push_log(LOG_ALLIES_GONE);
            }
        }
    }

    #[must_use]
    const fn current_weather_speed_penalty(&self) -> f32 {
        match self.weather_state.today {
            Weather::ColdSnap => WEATHER_COLD_SNAP_SPEED,
            Weather::Storm | Weather::Smoke => WEATHER_STORM_SMOKE_SPEED,
            Weather::HeatWave => WEATHER_HEAT_WAVE_SPEED,
            Weather::Clear => WEATHER_DEFAULT_SPEED,
        }
    }

    #[must_use]
    fn behind_schedule_multiplier(&self) -> f32 {
        if self.day >= 70 {
            let target = f64::from(self.day) * f64::from(BEHIND_SCHEDULE_MILES_PER_DAY);
            if f64::from(self.miles_traveled_actual) < target {
                return 1.05;
            }
        }
        1.0
    }

    #[must_use]
    fn deep_conservative_travel_boost(&self) -> f32 {
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Conservative))) {
            return 1.0;
        }
        for &(day_threshold, mile_threshold, boost) in DEEP_CONSERVATIVE_BOOSTS {
            if self.day >= day_threshold && self.miles_traveled_actual < mile_threshold {
                return boost;
            }
        }
        1.0
    }

    #[must_use]
    fn deep_aggressive_reach_boost(&self) -> f32 {
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return 1.0;
        }
        for &(day_threshold, mile_threshold, boost) in DEEP_AGGRESSIVE_BOOSTS {
            if self.day >= day_threshold && self.miles_traveled_actual < mile_threshold {
                return boost;
            }
        }
        1.0
    }

    pub(crate) fn apply_deep_aggressive_sanity_guard(&mut self) {
        if self.guards.deep_aggressive_sanity_guard_used {
            return;
        }
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return;
        }
        if self.day < DEEP_AGGRESSIVE_SANITY_DAY
            || self.miles_traveled_actual < DEEP_AGGRESSIVE_SANITY_MILES
        {
            return;
        }
        if self.stats.sanity > 0 {
            return;
        }
        if self.budget_cents < DEEP_AGGRESSIVE_SANITY_COST {
            return;
        }
        self.budget_cents -= DEEP_AGGRESSIVE_SANITY_COST;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.stats.sanity += SANITY_POINT_REWARD;
        self.stats.pants = (self.stats.pants - DEEP_AGGRESSIVE_SANITY_PANTS_PENALTY).max(0);
        self.stats.clamp();
        if self.current_day_kind.is_none() {
            self.record_travel_day(TravelDayKind::Partial, 0.0, "da_sanity_guard");
        } else {
            self.add_day_reason_tag("da_sanity_guard");
        }
        self.guards.deep_aggressive_sanity_guard_used = true;
        self.push_log(LOG_BOSS_COMPOSE_FUNDS);
        self.push_log(LOG_BOSS_COMPOSE);
    }

    pub(crate) fn apply_deep_aggressive_compose(&mut self) -> bool {
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return false;
        }

        let mut applied = false;
        if self.stats.supplies >= BOSS_COMPOSE_SUPPLY_COST {
            self.stats.supplies -= BOSS_COMPOSE_SUPPLY_COST;
            self.stats.sanity += SANITY_POINT_REWARD;
            self.stats.pants = (self.stats.pants - BOSS_COMPOSE_PANTS_SUPPLY).max(0);
            self.push_log(LOG_BOSS_COMPOSE_SUPPLIES);
            applied = true;
        } else if self.budget_cents >= BOSS_COMPOSE_FUNDS_COST {
            self.budget_cents -= BOSS_COMPOSE_FUNDS_COST;
            self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
            self.stats.sanity += SANITY_POINT_REWARD;
            self.stats.pants = (self.stats.pants - BOSS_COMPOSE_FUNDS_PANTS).max(0);
            self.push_log(LOG_BOSS_COMPOSE_FUNDS);
            applied = true;
        }

        if applied {
            self.stats.clamp();
            self.push_log(LOG_BOSS_COMPOSE);
        }
        applied
    }

    #[must_use]
    fn compute_miles_for_today(
        &mut self,
        pace_cfg: &crate::pacing::PaceCfg,
        limits: &crate::pacing::PacingLimits,
    ) -> f32 {
        let travel_v2 = self.features.travel_v2;
        let travel_cfg = &self.journey_travel;

        let pace_scalar = self.pace_scalar(travel_cfg, pace_cfg);
        let (weather_scalar, penalty_floor) = self.weather_scalar(travel_v2, travel_cfg, limits);

        let mut multiplier = (pace_scalar * weather_scalar).max(penalty_floor);
        if matches!(self.policy, Some(PolicyKind::Balanced)) {
            multiplier *= if self.mode.is_deep() {
                DEEP_BALANCED_TRAVEL_NUDGE
            } else {
                CLASSIC_BALANCED_TRAVEL_NUDGE
            };
        }
        multiplier *= self.endgame_bias();
        let behind_boost = self.behind_schedule_multiplier();
        if behind_boost > 1.0 {
            multiplier *= behind_boost;
        }

        let mut raw_distance = travel_cfg.mpd_base * multiplier;
        let mut distance = raw_distance;
        let ratio = self.journey_partial_ratio.clamp(0.0, 1.0);
        let mut partial_distance = raw_distance * ratio;

        let travel_boost = self.travel_boost_multiplier();
        if travel_boost > 1.0 {
            raw_distance *= travel_boost;
            distance *= travel_boost;
            partial_distance *= travel_boost;
        }

        let mobility_readiness = self.mobility_readiness();
        let stamina_penalty = mobility_readiness * self.malnutrition_penalty();
        distance *= stamina_penalty;
        partial_distance *= stamina_penalty;

        distance *= self.exec_effects.travel_multiplier;
        partial_distance *= self.exec_effects.travel_multiplier;
        let illness_penalty = self.illness_travel_penalty.max(0.0);
        distance *= illness_penalty;
        partial_distance *= illness_penalty;

        self.distance_cap_today = travel_cfg.mpd_max.max(travel_cfg.mpd_base);
        let max_distance = if self.distance_cap_today > 0.0 {
            self.distance_cap_today
        } else {
            travel_cfg.mpd_max
        };

        let mut clamped_distance = distance.clamp(travel_cfg.mpd_min, max_distance);
        if clamped_distance.is_nan() || clamped_distance <= 0.0 {
            clamped_distance = travel_cfg.mpd_min.max(TRAVEL_PARTIAL_MIN_DISTANCE);
        }
        clamped_distance = clamped_distance.max(TRAVEL_PARTIAL_MIN_DISTANCE);

        raw_distance = raw_distance.clamp(0.0, max_distance);

        partial_distance = partial_distance.clamp(0.0, clamped_distance);
        if partial_distance > 0.0 {
            partial_distance =
                partial_distance.max(TRAVEL_PARTIAL_MIN_DISTANCE.min(clamped_distance));
        }

        self.distance_today_raw = raw_distance;
        self.distance_today = clamped_distance;
        self.partial_distance_today = partial_distance;
        self.distance_today
    }

    fn compute_otdeluxe_miles_for_today(&mut self, policy: &OtDeluxe90sPolicy) -> f32 {
        let base = policy.travel.base_mpd_plains_steady_good.max(0.0);
        let pace_mult = match self.ot_deluxe.pace {
            OtDeluxePace::Steady => policy.pace_mult_steady,
            OtDeluxePace::Strenuous => policy.pace_mult_strenuous,
            OtDeluxePace::Grueling => policy.pace_mult_grueling,
        };
        self.ot_deluxe.terrain = if otdeluxe_trail::is_mountain_for_miles(
            &policy.trail,
            self.ot_deluxe.route.variant,
            self.ot_deluxe.miles_traveled,
        ) {
            OtDeluxeTerrain::Mountains
        } else {
            OtDeluxeTerrain::Plains
        };
        let terrain_mult = if matches!(self.ot_deluxe.terrain, OtDeluxeTerrain::Mountains) {
            policy.travel.terrain_mult_mountains
        } else {
            1.0
        };
        let effective_oxen = self.ot_deluxe.effective_oxen(policy);
        let oxen_mult =
            if policy.oxen.min_for_base > 0.0 && effective_oxen < policy.oxen.min_for_base {
                (effective_oxen / policy.oxen.min_for_base).max(0.0)
            } else {
                1.0
            };
        let sick_count = f32::from(self.ot_deluxe.party.sick_count());
        let sick_penalty = policy
            .travel
            .sick_member_speed_penalty
            .mul_add(-sick_count, 1.0)
            .max(0.0);
        let disease_mult = self.ot_deluxe.travel.disease_speed_mult.max(0.0);
        let snow_mult = otdeluxe_snow_speed_mult(self.ot_deluxe.weather.snow_depth, &policy.travel);
        let overrides = policy.overrides_for(self.region, self.ot_deluxe.season);
        let travel_override = overrides.travel_multiplier.unwrap_or(1.0);
        let travel_override = if travel_override.is_finite() && travel_override >= 0.0 {
            travel_override
        } else {
            1.0
        };
        let miles = (base
            * pace_mult
            * terrain_mult
            * oxen_mult
            * sick_penalty
            * disease_mult
            * snow_mult
            * travel_override)
            .max(0.0);
        let ratio = policy.travel.partial_ratio.clamp(0.0, 1.0);
        let partial = (miles * ratio).clamp(0.0, miles);
        self.distance_today_raw = miles;
        self.distance_today = miles;
        self.partial_distance_today = partial;
        self.distance_cap_today = miles;
        miles
    }

    fn pace_scalar(&self, travel_cfg: &TravelConfig, pace_cfg: &crate::pacing::PaceCfg) -> f32 {
        let pace_policy = travel_cfg
            .pace_factor
            .get(&self.pace)
            .copied()
            .unwrap_or(1.0)
            .max(TRAVEL_CONFIG_MIN_MULTIPLIER);
        let pace_cfg_scalar = if pace_cfg.dist_mult > 0.0 {
            pace_cfg.dist_mult
        } else {
            1.0
        };
        (pace_policy * pace_cfg_scalar).max(TRAVEL_CONFIG_MIN_MULTIPLIER)
    }

    fn weather_scalar(
        &self,
        travel_v2: bool,
        travel_cfg: &TravelConfig,
        limits: &crate::pacing::PacingLimits,
    ) -> (f32, f32) {
        let policy_weather = travel_cfg
            .weather_factor
            .get(&self.weather_state.today)
            .copied()
            .unwrap_or(1.0)
            .max(TRAVEL_CONFIG_MIN_MULTIPLIER);
        let runtime_weather = if travel_v2 {
            self.weather_travel_multiplier
                .max(TRAVEL_CONFIG_MIN_MULTIPLIER)
        } else {
            self.current_weather_speed_penalty()
        }
        .max(WEATHER_PACE_MULTIPLIER_FLOOR);
        let weather_scalar = (policy_weather * runtime_weather).max(TRAVEL_CONFIG_MIN_MULTIPLIER);

        let penalty_floor = if travel_v2 {
            if limits.distance_penalty_floor > 0.0 {
                limits.distance_penalty_floor
            } else {
                TRAVEL_V2_PENALTY_FLOOR
            }
        } else {
            TRAVEL_CLASSIC_PENALTY_FLOOR
        };
        (weather_scalar, penalty_floor)
    }

    fn endgame_bias(&self) -> f32 {
        if self.endgame.active && self.endgame.travel_bias > 0.0 {
            self.endgame.travel_bias.max(1.0)
        } else {
            1.0
        }
    }

    fn travel_boost_multiplier(&self) -> f32 {
        self.deep_conservative_travel_boost() * self.deep_aggressive_reach_boost()
    }

    fn mobility_readiness(&self) -> f32 {
        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            VEHICLE_CRITICAL_SPEED_FACTOR
        } else {
            1.0
        }
    }

    fn malnutrition_penalty(&self) -> f32 {
        if self.malnutrition_level > 0 {
            let malnutrition =
                num_traits::cast::<u32, f32>(self.malnutrition_level).unwrap_or_default();
            malnutrition
                .mul_add(-VEHICLE_MALNUTRITION_PENALTY_PER_STACK, 1.0)
                .max(VEHICLE_MALNUTRITION_MIN_FACTOR)
        } else {
            1.0
        }
    }

    fn check_vehicle_terminal_state(&mut self) -> bool {
        let spare_guard = self.total_spares();
        let base_tolerance = if self.mode.is_deep() {
            if matches!(self.policy, Some(PolicyKind::Balanced)) {
                VEHICLE_BASE_TOLERANCE_CLASSIC
            } else {
                VEHICLE_BASE_TOLERANCE_DEEP
            }
        } else {
            VEHICLE_BASE_TOLERANCE_CLASSIC
        };
        let mut tolerance = base_tolerance.max(spare_guard * VEHICLE_SPARE_GUARD_SCALE);
        if self.mode.is_deep() {
            let miles = self.miles_traveled_actual;
            match self.policy {
                Some(PolicyKind::Aggressive | PolicyKind::Conservative) => {
                    for &(threshold, bonus) in DEEP_AGGRESSIVE_TOLERANCE_THRESHOLDS {
                        if miles >= threshold {
                            tolerance = tolerance.saturating_add(bonus);
                            break;
                        }
                    }
                }
                Some(PolicyKind::Balanced) => {
                    for &(threshold, bonus) in DEEP_BALANCED_TOLERANCE_THRESHOLDS {
                        if miles >= threshold {
                            tolerance = tolerance.saturating_add(bonus);
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        if self.vehicle.health <= 0.0 {
            let mut recovered = if spare_guard > 0 {
                self.consume_any_spare_for_emergency()
            } else {
                false
            };
            if !recovered && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.spend_emergency_repair(LOG_EMERGENCY_REPAIR_FORCED);
                recovered = true;
            }
            if !recovered && self.vehicle_breakdowns < tolerance {
                // Limp along by burning time; the vehicle barely holds together.
                self.vehicle.health = self.vehicle.health.max(VEHICLE_JURY_RIG_HEAL);
                self.apply_delay_travel_credit("repair");
                recovered = true;
            }
            if recovered {
                self.mark_damage(DamageCause::Vehicle);
            }
        }

        let health_depleted = self.vehicle.health <= 0.0;
        let out_of_options = spare_guard == 0 && self.budget_cents < EMERGENCY_REPAIR_COST;
        if endgame::enforce_failure_guard(self) {
            return false;
        }
        if health_depleted && self.vehicle_breakdowns >= tolerance && out_of_options {
            if self.mode == GameMode::Classic
                && matches!(self.policy, Some(PolicyKind::Balanced))
                && self.miles_traveled_actual < CLASSIC_BALANCED_FAILURE_GUARD_MILES
            {
                self.apply_classic_field_repair_guard();
                return false;
            }
            if self.try_deep_aggressive_field_repair() {
                return false;
            }
            if self.try_emergency_limp_guard() {
                return false;
            }
            if self.mode.is_deep()
                && matches!(self.policy, Some(PolicyKind::Balanced))
                && self.miles_traveled_actual < DEEP_BALANCED_FAILSAFE_DISTANCE
            {
                self.vehicle.health = self.vehicle.health.max(VEHICLE_JURY_RIG_HEAL);
                self.apply_delay_travel_credit("repair");
                return false;
            }
            self.vehicle.health = 0.0;
            self.mark_damage(DamageCause::Vehicle);
            self.set_ending(Ending::VehicleFailure {
                cause: VehicleFailureCause::Destroyed,
            });
            self.push_log(LOG_VEHICLE_FAILURE);
            return true;
        }
        false
    }

    const fn crossing_kind_for_index(&self, next_idx: usize) -> CrossingKind {
        if next_idx + 1 >= CROSSING_MILESTONES.len() || (self.mode.is_deep() && next_idx % 2 == 1) {
            CrossingKind::BridgeOut
        } else {
            CrossingKind::Checkpoint
        }
    }

    fn apply_target_travel(&mut self, kind: TravelDayKind, target_miles: f32, reason_tag: &str) {
        let tolerance = 0.0001;
        let target = target_miles.max(0.0);
        if target + tolerance < self.current_day_miles {
            self.reset_today_progress();
        }
        let delta = (target - self.current_day_miles).max(0.0);
        self.record_travel_day(kind, delta, reason_tag);
    }

    pub(crate) fn handle_crossing_event(
        &mut self,
        computed_miles_today: f32,
    ) -> Option<(bool, String)> {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            return self.handle_otdeluxe_crossing_event(computed_miles_today);
        }
        if self.pending_crossing.is_some() {
            return Some((false, String::from(LOG_TRAVEL_BLOCKED)));
        }
        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let &milestone = CROSSING_MILESTONES.get(next_idx)?;
        if self.miles_traveled_actual + f32::EPSILON < milestone {
            return None;
        }

        let kind = self.crossing_kind_for_index(next_idx);
        self.pending_crossing = Some(PendingCrossing {
            kind,
            computed_miles_today,
        });
        self.pending_crossing_choice = None;
        self.push_event(
            EventKind::TravelBlocked,
            EventSeverity::Warning,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "reason": "crossing_choice",
                "kind": format!("{kind:?}").to_lowercase(),
            }),
        );
        Some((false, String::from(LOG_TRAVEL_BLOCKED)))
    }

    fn handle_otdeluxe_crossing_event(
        &mut self,
        computed_miles_today: f32,
    ) -> Option<(bool, String)> {
        if self.ot_deluxe.crossing.choice_pending {
            return Some((false, String::from(LOG_TRAVEL_BLOCKED)));
        }
        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let river = otdeluxe_crossings::river_for_index(next_idx)?;
        let node_index = otdeluxe_crossings::node_index_for_river(river);
        let policy = default_otdeluxe_policy();
        let marker = otdeluxe_trail::mile_marker_for_node(
            &policy.trail,
            self.ot_deluxe.route.variant,
            node_index,
        )?;
        let marker_miles = f32::from(marker);
        if self.miles_traveled_actual + computed_miles_today + f32::EPSILON < marker_miles {
            return None;
        }

        let distance_to_marker = (marker_miles - self.miles_traveled_actual).max(0.0);
        let river_state = otdeluxe_crossings::derive_river_state(
            &policy.crossings,
            river,
            self.ot_deluxe.season,
            self.ot_deluxe.weather.rain_accum,
        );
        self.ot_deluxe.crossing.choice_pending = true;
        self.ot_deluxe.crossing.chosen_method = None;
        self.ot_deluxe.crossing.river = Some(river_state);
        self.ot_deluxe.crossing.river_kind = Some(river);
        self.ot_deluxe.crossing.computed_miles_today = distance_to_marker;
        self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Stopped;

        self.push_event(
            EventKind::TravelBlocked,
            EventSeverity::Warning,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "reason": "crossing_choice",
                "river": format!("{river:?}").to_lowercase(),
                "node_index": node_index,
            }),
        );

        Some((false, String::from(LOG_TRAVEL_BLOCKED)))
    }

    pub(crate) fn resolve_pending_crossing_choice(
        &mut self,
        choice: CrossingChoice,
    ) -> Option<(bool, String)> {
        let pending = *self.pending_crossing.as_ref()?;
        let kind = pending.kind;
        let cfg = CrossingConfig::default();
        let (has_permit, can_bribe) = self.crossing_options(&cfg, kind);
        match choice {
            CrossingChoice::Permit if !has_permit => {
                self.pending_crossing_choice = None;
                return None;
            }
            CrossingChoice::Bribe if !can_bribe => {
                self.pending_crossing_choice = None;
                return None;
            }
            _ => {}
        }

        self.pending_crossing_choice = None;
        self.pending_crossing = None;

        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let policy = self.journey_crossing.clone();
        let mut telemetry = CrossingTelemetry::new(self.day, self.region, self.season, kind);
        let resolved = match choice {
            CrossingChoice::Detour => {
                let sample = self.sample_crossing_roll(next_idx);
                let detour_days = Self::detour_days_for_sample(&policy, sample);
                crossings::CrossingOutcome {
                    result: crossings::CrossingResult::Detour(detour_days),
                    used_permit: false,
                    bribe_attempted: false,
                    bribe_succeeded: false,
                }
            }
            CrossingChoice::Bribe => {
                let ctx = CrossingContext {
                    policy: &policy,
                    kind,
                    has_permit: false,
                    bribe_intent: true,
                    prior_bribe_attempts: self.crossing_bribe_attempts,
                };
                self.resolve_crossing_outcome(ctx, next_idx)
            }
            CrossingChoice::Permit => {
                let ctx = CrossingContext {
                    policy: &policy,
                    kind,
                    has_permit: true,
                    bribe_intent: false,
                    prior_bribe_attempts: self.crossing_bribe_attempts,
                };
                self.resolve_crossing_outcome(ctx, next_idx)
            }
        };

        telemetry.permit_used = resolved.used_permit;
        telemetry.bribe_attempted = resolved.bribe_attempted;
        if resolved.bribe_attempted {
            telemetry.bribe_success = Some(resolved.bribe_succeeded);
        }

        self.apply_crossing_decisions(resolved, &cfg, kind, &mut telemetry);
        Some(self.process_crossing_result(resolved, telemetry, pending.computed_miles_today))
    }

    pub(crate) fn resolve_pending_otdeluxe_crossing_choice(
        &mut self,
        method: OtDeluxeCrossingMethod,
    ) -> Option<(bool, String)> {
        let (river_kind, river_state) = self.otdeluxe_crossing_context(method)?;
        let alive_indices = self.otdeluxe_alive_indices();
        let (resolution, trace, drowned_indices) =
            self.roll_otdeluxe_crossing(river_kind, &river_state, method, &alive_indices);
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }

        let policy = default_otdeluxe_policy();
        self.apply_otdeluxe_crossing_costs(policy, method);
        let losses = self.apply_otdeluxe_crossing_losses(resolution.loss_ratio);
        let drown_count = self.apply_otdeluxe_drownings(&drowned_indices);

        let (log_key, severity) = Self::otdeluxe_crossing_log_and_severity(resolution.outcome);
        self.push_log(log_key);
        self.emit_otdeluxe_crossing_event(
            river_kind,
            method,
            &resolution,
            severity,
            losses,
            drown_count,
        );

        let computed_miles_today = self.clear_otdeluxe_crossing_state();
        self.crossings_completed = self.crossings_completed.saturating_add(1);

        let target_miles = computed_miles_today.max(0.0);
        self.apply_target_travel(TravelDayKind::Partial, target_miles, "otdeluxe_crossing");
        self.stats.clamp();

        if self.ot_deluxe.party.alive_count() == 0 {
            self.set_ending(Ending::Collapse {
                cause: CollapseCause::Crossing,
            });
        }

        self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Moving;
        self.end_of_day();

        if self.ending.is_none() {
            self.apply_otdeluxe_crossing_delays(policy, &resolution);
        }

        Some((self.ending.is_some(), String::from(log_key)))
    }

    fn otdeluxe_crossing_context(
        &mut self,
        method: OtDeluxeCrossingMethod,
    ) -> Option<(OtDeluxeRiver, OtDeluxeRiverState)> {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        if !self.ot_deluxe.crossing.choice_pending {
            return None;
        }
        let river_kind = self.ot_deluxe.crossing.river_kind?;
        let river_state = self.ot_deluxe.crossing.river.clone()?;
        let policy = default_otdeluxe_policy();
        let options = otdeluxe_crossings::crossing_options(
            &policy.crossings,
            river_kind,
            &river_state,
            &self.ot_deluxe.inventory,
        );
        if !options.is_allowed(method) {
            self.ot_deluxe.crossing.chosen_method = None;
            return None;
        }
        Some((river_kind, river_state))
    }

    fn otdeluxe_alive_indices(&self) -> Vec<usize> {
        self.ot_deluxe
            .party
            .members
            .iter()
            .enumerate()
            .filter_map(|(idx, member)| member.alive.then_some(idx))
            .collect()
    }

    fn roll_otdeluxe_crossing(
        &self,
        river_kind: OtDeluxeRiver,
        river_state: &OtDeluxeRiverState,
        method: OtDeluxeCrossingMethod,
        alive_indices: &[usize],
    ) -> (
        otdeluxe_crossings::OtDeluxeCrossingResolution,
        Option<EventDecisionTrace>,
        Vec<usize>,
    ) {
        let policy = default_otdeluxe_policy();
        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let seed_mix =
            self.seed ^ (u64::try_from(next_idx).unwrap_or(0) << 32) ^ u64::from(self.day);
        self.crossing_rng().map_or_else(
            || {
                let mut fallback = SmallRng::seed_from_u64(seed_mix);
                let (resolution, trace) = otdeluxe_crossings::resolve_crossing_with_trace(
                    &policy.crossings,
                    river_kind,
                    river_state,
                    method,
                    &mut fallback,
                );
                let drowned_indices = Self::select_drowning_indices(
                    &mut fallback,
                    alive_indices,
                    resolution.drownings,
                );
                (resolution, trace, drowned_indices)
            },
            |mut rng| {
                let (resolution, trace) = otdeluxe_crossings::resolve_crossing_with_trace(
                    &policy.crossings,
                    river_kind,
                    river_state,
                    method,
                    &mut *rng,
                );
                let drowned_indices =
                    Self::select_drowning_indices(&mut *rng, alive_indices, resolution.drownings);
                (resolution, trace, drowned_indices)
            },
        )
    }

    const fn apply_otdeluxe_crossing_costs(
        &mut self,
        policy: &OtDeluxe90sPolicy,
        method: OtDeluxeCrossingMethod,
    ) {
        match method {
            OtDeluxeCrossingMethod::Ferry => {
                let cost = policy.crossings.ferry_cost_cents;
                self.ot_deluxe.inventory.cash_cents =
                    self.ot_deluxe.inventory.cash_cents.saturating_sub(cost);
            }
            OtDeluxeCrossingMethod::Guide => {
                let cost = policy.crossings.guide_cost_clothes_sets;
                self.ot_deluxe.inventory.clothes_sets =
                    self.ot_deluxe.inventory.clothes_sets.saturating_sub(cost);
            }
            OtDeluxeCrossingMethod::Ford | OtDeluxeCrossingMethod::CaulkFloat => {}
        }
    }

    fn apply_otdeluxe_crossing_losses(&mut self, loss_ratio: f32) -> OtDeluxeCrossingLosses {
        if loss_ratio <= 0.0 {
            return OtDeluxeCrossingLosses::default();
        }
        let inventory = &mut self.ot_deluxe.inventory;
        let food_lbs = otdeluxe_crossings::apply_loss_ratio(inventory.food_lbs, loss_ratio);
        let bullets = otdeluxe_crossings::apply_loss_ratio(inventory.bullets, loss_ratio);
        let clothes_sets = otdeluxe_crossings::apply_loss_ratio(inventory.clothes_sets, loss_ratio);
        let wheels_loss_u16 =
            otdeluxe_crossings::apply_loss_ratio(u16::from(inventory.spares_wheels), loss_ratio);
        let axles_loss_u16 =
            otdeluxe_crossings::apply_loss_ratio(u16::from(inventory.spares_axles), loss_ratio);
        let tongues_loss_u16 =
            otdeluxe_crossings::apply_loss_ratio(u16::from(inventory.spares_tongues), loss_ratio);

        let spares_wheels = u8::try_from(wheels_loss_u16).unwrap_or(u8::MAX);
        let spares_axles = u8::try_from(axles_loss_u16).unwrap_or(u8::MAX);
        let spares_tongues = u8::try_from(tongues_loss_u16).unwrap_or(u8::MAX);

        inventory.food_lbs = inventory.food_lbs.saturating_sub(food_lbs);
        inventory.bullets = inventory.bullets.saturating_sub(bullets);
        inventory.clothes_sets = inventory.clothes_sets.saturating_sub(clothes_sets);
        inventory.spares_wheels = inventory.spares_wheels.saturating_sub(spares_wheels);
        inventory.spares_axles = inventory.spares_axles.saturating_sub(spares_axles);
        inventory.spares_tongues = inventory.spares_tongues.saturating_sub(spares_tongues);

        OtDeluxeCrossingLosses {
            food_lbs,
            bullets,
            clothes_sets,
            spares_wheels,
            spares_axles,
            spares_tongues,
        }
    }

    fn apply_otdeluxe_drownings(&mut self, drowned_indices: &[usize]) -> u8 {
        for idx in drowned_indices {
            if let Some(member) = self.ot_deluxe.party.members.get_mut(*idx) {
                member.alive = false;
            }
        }
        u8::try_from(drowned_indices.len()).unwrap_or(u8::MAX)
    }

    const fn otdeluxe_crossing_log_and_severity(
        outcome: otdeluxe_crossings::OtDeluxeCrossingOutcome,
    ) -> (&'static str, EventSeverity) {
        match outcome {
            otdeluxe_crossings::OtDeluxeCrossingOutcome::Safe => {
                (LOG_OT_CROSSING_SAFE, EventSeverity::Info)
            }
            otdeluxe_crossings::OtDeluxeCrossingOutcome::StuckInMud => {
                (LOG_OT_CROSSING_STUCK, EventSeverity::Warning)
            }
            otdeluxe_crossings::OtDeluxeCrossingOutcome::SuppliesWet => {
                (LOG_OT_CROSSING_WET, EventSeverity::Warning)
            }
            otdeluxe_crossings::OtDeluxeCrossingOutcome::Tipped => {
                (LOG_OT_CROSSING_TIPPED, EventSeverity::Critical)
            }
            otdeluxe_crossings::OtDeluxeCrossingOutcome::Sank => {
                (LOG_OT_CROSSING_SANK, EventSeverity::Critical)
            }
            otdeluxe_crossings::OtDeluxeCrossingOutcome::Drowned => {
                (LOG_OT_CROSSING_DROWNED, EventSeverity::Critical)
            }
        }
    }

    fn emit_otdeluxe_crossing_event(
        &mut self,
        river_kind: OtDeluxeRiver,
        method: OtDeluxeCrossingMethod,
        resolution: &otdeluxe_crossings::OtDeluxeCrossingResolution,
        severity: EventSeverity,
        losses: OtDeluxeCrossingLosses,
        drown_count: u8,
    ) {
        self.push_event(
            EventKind::CrossingResolved,
            severity,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "river": format!("{river_kind:?}").to_lowercase(),
                "method": format!("{method:?}").to_lowercase(),
                "outcome": resolution.outcome.id(),
                "wait_days": resolution.wait_days,
                "crossing_days": resolution.crossing_days,
                "drying_days": resolution.drying_days,
                "loss_ratio": resolution.loss_ratio,
                "drownings": drown_count,
                "losses": {
                    "food_lbs": losses.food_lbs,
                    "bullets": losses.bullets,
                    "clothes_sets": losses.clothes_sets,
                    "spares_wheels": losses.spares_wheels,
                    "spares_axles": losses.spares_axles,
                    "spares_tongues": losses.spares_tongues,
                }
            }),
        );
    }

    const fn clear_otdeluxe_crossing_state(&mut self) -> f32 {
        let computed_miles_today = self.ot_deluxe.crossing.computed_miles_today;
        self.ot_deluxe.crossing.choice_pending = false;
        self.ot_deluxe.crossing.chosen_method = None;
        self.ot_deluxe.crossing.river = None;
        self.ot_deluxe.crossing.river_kind = None;
        self.ot_deluxe.crossing.computed_miles_today = 0.0;
        computed_miles_today
    }

    fn apply_otdeluxe_crossing_delays(
        &mut self,
        policy: &OtDeluxe90sPolicy,
        resolution: &otdeluxe_crossings::OtDeluxeCrossingResolution,
    ) {
        let extra_crossing_days = resolution.crossing_days.saturating_sub(1);
        let stuck_days = if matches!(
            resolution.outcome,
            otdeluxe_crossings::OtDeluxeCrossingOutcome::StuckInMud
        ) {
            policy.crossings.stuck_cost_days
        } else {
            0
        };
        if resolution.wait_days == 0
            && extra_crossing_days == 0
            && stuck_days == 0
            && resolution.drying_days == 0
        {
            return;
        }
        self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Delayed;
        if resolution.wait_days > 0 {
            self.advance_days_with_reason(
                u32::from(resolution.wait_days),
                "otdeluxe.crossing.wait",
            );
        }
        if extra_crossing_days > 0 {
            self.advance_days_with_reason(
                u32::from(extra_crossing_days),
                "otdeluxe.crossing.cross",
            );
        }
        if stuck_days > 0 {
            self.advance_days_with_reason(u32::from(stuck_days), "otdeluxe.crossing.stuck");
        }
        if resolution.drying_days > 0 {
            self.advance_days_with_reason(
                u32::from(resolution.drying_days),
                "otdeluxe.crossing.dry",
            );
        }
        self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Moving;
    }

    fn crossing_options(&self, cfg: &CrossingConfig, kind: CrossingKind) -> (bool, bool) {
        let has_permit = crossings::can_use_permit(self, &kind);
        let bribe_offered = !has_permit && crossings::can_afford_bribe(self, cfg, kind);
        (has_permit, bribe_offered)
    }

    fn sample_crossing_roll(&self, next_idx: usize) -> u32 {
        self.crossing_rng().map_or_else(
            || {
                let seed_mix =
                    self.seed ^ (u64::try_from(next_idx).unwrap_or(0) << 32) ^ u64::from(self.day);
                let mut fallback = SmallRng::seed_from_u64(seed_mix);
                fallback.next_u32()
            },
            |mut rng| rng.next_u32(),
        )
    }

    fn detour_days_for_sample(policy: &CrossingPolicy, sample: u32) -> u8 {
        let min = policy.detour_days.min;
        let max = policy.detour_days.max;
        if min >= max {
            return min;
        }
        let span = u32::from(max.saturating_sub(min)) + 1;
        let offset = sample % span;
        let offset_u8 = u8::try_from(offset).unwrap_or(u8::MAX);
        min.saturating_add(offset_u8)
    }

    fn select_drowning_indices<R: RngCore + Rng>(
        rng: &mut R,
        alive_indices: &[usize],
        drownings: u8,
    ) -> Vec<usize> {
        let count = usize::from(drownings);
        if count == 0 || alive_indices.is_empty() {
            return Vec::new();
        }
        let mut indices = alive_indices.to_vec();
        indices.shuffle(rng);
        indices.truncate(count.min(indices.len()));
        indices
    }

    fn resolve_crossing_outcome(
        &mut self,
        ctx: CrossingContext<'_>,
        next_idx: usize,
    ) -> crossings::CrossingOutcome {
        let (outcome, trace) = self.crossing_rng().map_or_else(
            || {
                let seed_mix =
                    self.seed ^ (u64::try_from(next_idx).unwrap_or(0) << 32) ^ u64::from(self.day);
                let mut fallback = SmallRng::seed_from_u64(seed_mix);
                crossings::resolve_crossing_with_trace(ctx, &mut fallback)
            },
            |mut rng| crossings::resolve_crossing_with_trace(ctx, &mut *rng),
        );
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }
        outcome
    }

    fn apply_crossing_decisions(
        &mut self,
        resolved: crossings::CrossingOutcome,
        cfg: &CrossingConfig,
        kind: CrossingKind,
        telemetry: &mut CrossingTelemetry,
    ) {
        if resolved.used_permit {
            self.push_log(LOG_CROSSING_DECISION_PERMIT);
            let permit_log = crossings::apply_permit(self, cfg, kind);
            self.push_log(permit_log);
            self.crossing_permit_uses = self.crossing_permit_uses.saturating_add(1);
        }

        if resolved.bribe_attempted {
            self.push_log(LOG_CROSSING_DECISION_BRIBE);
            let _ = crossings::apply_bribe(self, cfg, kind);
            self.crossing_bribe_attempts = self.crossing_bribe_attempts.saturating_add(1);
            if resolved.bribe_succeeded {
                self.crossing_bribe_successes = self.crossing_bribe_successes.saturating_add(1);
            }
            telemetry.bribe_success = Some(resolved.bribe_succeeded);
            let log_key = if resolved.bribe_succeeded {
                "crossing.result.bribe.success"
            } else {
                "crossing.result.bribe.fail"
            };
            self.push_log(log_key);
        }
    }

    fn process_crossing_result(
        &mut self,
        resolved: crossings::CrossingOutcome,
        mut telemetry: CrossingTelemetry,
        computed_miles_today: f32,
    ) -> (bool, String) {
        match resolved.result {
            crossings::CrossingResult::Pass => {
                telemetry.outcome = CrossingOutcomeTelemetry::Passed;
                self.push_log(LOG_CROSSING_PASSED);
                self.crossings_completed = self.crossings_completed.saturating_add(1);
                let target_miles = day_accounting::partial_day_miles(self, computed_miles_today);
                self.apply_target_travel(TravelDayKind::Partial, target_miles, "crossing_pass");
                self.stats.clamp();
                self.crossing_events.push(telemetry);
                self.end_of_day();
                (false, String::from(LOG_CROSSING_PASSED))
            }
            crossings::CrossingResult::Detour(days) => {
                if telemetry.bribe_attempted {
                    telemetry.bribe_success = telemetry.bribe_success.or(Some(false));
                }
                telemetry.detour_taken = true;
                telemetry.detour_days = Some(u32::from(days));
                telemetry.outcome = CrossingOutcomeTelemetry::Detoured;
                self.crossing_detours_taken = self.crossing_detours_taken.saturating_add(1);
                let per_day_miles = day_accounting::partial_day_miles(self, computed_miles_today);
                self.push_log(LOG_CROSSING_DETOUR);
                self.apply_target_travel(TravelDayKind::Partial, per_day_miles, "detour");
                self.stats.clamp();
                self.crossing_events.push(telemetry);
                self.end_of_day();
                if days > 1 {
                    let extra_days = u32::from(days.saturating_sub(1));
                    self.advance_days_with_credit(
                        extra_days,
                        TravelDayKind::Partial,
                        per_day_miles,
                        "detour",
                    );
                }
                (false, String::from(LOG_CROSSING_DETOUR))
            }
            crossings::CrossingResult::TerminalFail => {
                if telemetry.bribe_attempted {
                    telemetry.bribe_success = telemetry.bribe_success.or(Some(false));
                }
                telemetry.outcome = CrossingOutcomeTelemetry::Failed;
                self.crossing_failures = self.crossing_failures.saturating_add(1);
                self.push_log(LOG_CROSSING_FAILURE);
                self.reset_today_progress();
                self.record_travel_day(TravelDayKind::NonTravel, 0.0, "crossing_fail");
                self.stats.clamp();
                self.set_ending(Ending::Collapse {
                    cause: CollapseCause::Crossing,
                });
                self.crossing_events.push(telemetry);
                self.end_of_day();
                (true, String::from(LOG_CROSSING_FAILURE))
            }
        }
    }

    #[must_use]
    pub fn with_seed(mut self, seed: u64, mode: GameMode, data: EncounterData) -> Self {
        self.mode = mode;
        self.mechanical_policy = MechanicalPolicyId::default();
        self.seed = seed;
        self.state_version = Self::current_version();
        self.day_records.clear();
        self.recompute_day_counters();
        self.current_day_record = None;
        self.general_strain = 0.0;
        self.journey_partial_ratio = JourneyCfg::default_partial_ratio();
        self.journey_travel = TravelConfig::default();
        self.journey_wear = WearConfig::default();
        self.journey_breakdown = BreakdownConfig::default();
        self.journey_part_weights = PartWeights::default();
        self.journey_crossing = CrossingPolicy::default();
        self.intent = IntentState::default();
        self.wait = WaitState::default();
        self.ot_deluxe = OtDeluxeState::default();
        self.exec_effects = ExecOrderEffects::default();
        self.events_today.clear();
        self.decision_traces_today.clear();
        self.weather_effects = WeatherEffects::default();
        self.pending_crossing = None;
        self.pending_crossing_choice = None;
        self.pending_route_choice = None;
        self.push_log("log.seed-set");
        self.data = Some(data);
        self.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));
        self
    }

    #[must_use]
    pub fn rehydrate(mut self, data: EncounterData) -> Self {
        self.data = Some(data);
        if self.state_version < Self::current_version() {
            if self.state_version < 4 {
                self.intent = IntentState::default();
                self.wait = WaitState::default();
                self.ot_deluxe = self.build_ot_deluxe_state_from_legacy();
            }
            self.state_version = Self::current_version();
            if self.day_records.is_empty()
                && (self.travel_days > 0
                    || self.partial_travel_days > 0
                    || self.non_travel_days > 0)
            {
                // Conservatively backfill a single record representing the previous day counts.
                let day_index = u16::try_from(self.day.saturating_sub(1)).unwrap_or(u16::MAX);
                let kind = if self.travel_days > 0 {
                    TravelDayKind::Travel
                } else if self.partial_travel_days > 0 {
                    TravelDayKind::Partial
                } else {
                    TravelDayKind::NonTravel
                };
                let miles = self.miles_traveled_actual;
                self.day_records
                    .push(DayRecord::new(day_index, kind, miles));
            }
        }
        self.journey_partial_ratio = self.journey_partial_ratio.clamp(0.2, 0.95);
        self.journey_travel.sanitize();
        self.journey_crossing.sanitize();
        self.recompute_day_counters();
        self.events_today.clear();
        self.decision_traces_today.clear();
        self.exec_effects = ExecOrderEffects::default();
        self.weather_effects = WeatherEffects::default();
        self.pending_crossing_choice = None;
        self.pending_route_choice = None;
        self.ot_deluxe.crossing.chosen_method = None;
        if self.rng_bundle.is_none() {
            self.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(self.seed)));
        }
        self.day_state.lifecycle.log_cursor = u32::try_from(self.logs.len()).unwrap_or(u32::MAX);
        self.day_state.lifecycle.event_seq = 0;
        self
    }

    #[must_use]
    pub const fn region_by_miles(miles: f32) -> Region {
        if miles < 700.0 {
            Region::Heartland
        } else if miles < 1_400.0 {
            Region::RustBelt
        } else {
            Region::Beltway
        }
    }

    pub(crate) fn guard_boss_gate(&self) -> Option<(bool, String, bool)> {
        if self.mechanical_policy != MechanicalPolicyId::DystrailLegacy {
            return None;
        }
        if self.boss.readiness.ready && !self.boss.outcome.attempted {
            Some((false, String::from(LOG_BOSS_AWAIT), false))
        } else {
            None
        }
    }

    pub(crate) fn pre_travel_checks(&mut self) -> Option<(bool, String, bool)> {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            self.sync_otdeluxe_trail_distance();
            if self.ot_deluxe.route.pending_prompt.is_some() {
                self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Stopped;
                self.clear_today_travel_distance();
                return Some((false, String::from(LOG_TRAVEL_BLOCKED), false));
            }
        }
        self.check_otdeluxe_oxen_gate()
    }

    fn check_otdeluxe_oxen_gate(&mut self) -> Option<(bool, String, bool)> {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        let policy = default_otdeluxe_policy();
        if !self.ot_deluxe.travel_blocked_by_oxen(policy) {
            return None;
        }
        self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Blocked;
        self.clear_today_travel_distance();
        self.record_travel_day(TravelDayKind::NonTravel, 0.0, "otdeluxe.no_oxen");
        self.end_of_day();
        Some((false, String::from(LOG_TRAVEL_BLOCKED), false))
    }

    pub(crate) fn consume_otdeluxe_navigation_delay_day(&mut self) -> bool {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return false;
        }
        let blocked_remaining = self.ot_deluxe.travel.blocked_days_remaining;
        let delay_remaining = self.ot_deluxe.travel.delay_days_remaining;
        if blocked_remaining == 0 && delay_remaining == 0 {
            return false;
        }

        let (blocked, remaining) = if blocked_remaining > 0 {
            let next = blocked_remaining.saturating_sub(1);
            self.ot_deluxe.travel.blocked_days_remaining = next;
            (true, next)
        } else {
            let next = delay_remaining.saturating_sub(1);
            self.ot_deluxe.travel.delay_days_remaining = next;
            (false, next)
        };

        self.day_state.lifecycle.suppress_stop_ratio = true;
        self.clear_today_travel_distance();
        self.day_state.travel.traveled_today = false;
        self.day_state.travel.partial_traveled_today = false;
        let tag = otdeluxe_navigation_delay_tag(blocked);
        if self.current_day_kind.is_none() {
            self.record_travel_day(TravelDayKind::NonTravel, 0.0, tag);
        } else {
            self.add_day_reason_tag(tag);
        }
        self.ot_deluxe.travel.wagon_state = if blocked {
            OtDeluxeWagonState::Blocked
        } else {
            OtDeluxeWagonState::Delayed
        };
        if remaining == 0 {
            self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Moving;
        }
        self.push_event(
            EventKind::TravelBlocked,
            if blocked {
                EventSeverity::Critical
            } else {
                EventSeverity::Warning
            },
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "reason": "navigation_delay",
                "blocked": blocked,
                "remaining_days": remaining
            }),
        );
        self.end_of_day();
        true
    }

    pub(crate) fn apply_otdeluxe_navigation_event(&mut self) -> bool {
        let policy = default_otdeluxe_policy();
        self.apply_otdeluxe_navigation_event_with_policy(&policy.navigation)
    }

    pub(crate) fn apply_otdeluxe_navigation_event_with_policy(
        &mut self,
        policy: &OtDeluxeNavigationPolicy,
    ) -> bool {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return false;
        }
        if self.ot_deluxe.travel.delay_days_remaining > 0
            || self.ot_deluxe.travel.blocked_days_remaining > 0
        {
            return false;
        }
        if self.distance_today <= 0.0 && self.distance_today_raw <= 0.0 {
            return false;
        }

        let Some((event, delay_days, trace)) = (|| {
            let mut rng = self.events_rng()?;
            let (event, trace) = roll_otdeluxe_navigation_event_with_trace(
                policy,
                self.ot_deluxe.weather.snow_depth,
                &mut *rng,
            );
            let event = event?;
            let delay = otdeluxe_navigation_delay_for(event, policy);
            let delay_days = roll_otdeluxe_navigation_delay_days(delay, &mut *rng);
            Some((event, delay_days, trace))
        })() else {
            return false;
        };
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }
        self.apply_otdeluxe_navigation_hard_stop(event, delay_days);
        true
    }

    fn apply_otdeluxe_navigation_hard_stop(
        &mut self,
        event: OtDeluxeNavigationEvent,
        delay_days: u8,
    ) {
        let remaining = delay_days.saturating_sub(1);
        let blocked = otdeluxe_navigation_is_blocked(event);
        self.day_state.lifecycle.suppress_stop_ratio = true;
        self.clear_today_travel_distance();
        self.day_state.travel.traveled_today = false;
        self.day_state.travel.partial_traveled_today = false;
        if blocked {
            self.ot_deluxe.travel.blocked_days_remaining = remaining;
            self.ot_deluxe.travel.delay_days_remaining = 0;
            self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Blocked;
        } else {
            self.ot_deluxe.travel.delay_days_remaining = remaining;
            self.ot_deluxe.travel.blocked_days_remaining = 0;
            self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Delayed;
        }
        let tag = otdeluxe_navigation_reason_tag(event);
        if self.current_day_kind.is_none() {
            self.record_travel_day(TravelDayKind::NonTravel, 0.0, tag);
        } else {
            self.add_day_reason_tag(tag);
        }
        let severity = if blocked {
            EventSeverity::Critical
        } else {
            EventSeverity::Warning
        };
        self.push_event(
            EventKind::NavigationEvent,
            severity,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "event": otdeluxe_navigation_event_id(event),
                "blocked": blocked,
                "delay_days": delay_days,
                "remaining_days": remaining
            }),
        );
        if remaining == 0 {
            self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Moving;
        }
        self.end_of_day();
    }

    pub(crate) fn apply_otdeluxe_random_event(&mut self) -> Option<()> {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        let spares_total = u16::from(self.ot_deluxe.inventory.spares_wheels)
            + u16::from(self.ot_deluxe.inventory.spares_axles)
            + u16::from(self.ot_deluxe.inventory.spares_tongues);
        let policy = default_otdeluxe_policy();
        let overrides = policy.overrides_for(self.region, self.ot_deluxe.season);
        let weight_mult = sanitize_event_weight_mult(overrides.event_weight_mult.unwrap_or(1.0));
        let weight_cap = overrides
            .event_weight_cap
            .and_then(|cap| (cap.is_finite() && cap >= 0.0).then_some(f64::from(cap)));
        let ctx = OtDeluxeRandomEventContext {
            season: self.ot_deluxe.season,
            food_lbs: self.ot_deluxe.inventory.food_lbs,
            oxen_total: self.ot_deluxe.oxen.total(),
            party_alive: self.ot_deluxe.party.alive_count(),
            health_general: self.ot_deluxe.health_general,
            spares_total,
            weight_mult: f64::from(weight_mult),
            weight_cap,
        };
        let bundle = self.rng_bundle.take()?;
        let result = {
            let mut rng = bundle.events();
            let pick = otdeluxe_random_events::pick_random_event_with_trace(
                otdeluxe_random_events::catalog(),
                &ctx,
                &mut *rng,
            )?;
            let (log_key, severity, payload) =
                self.apply_otdeluxe_random_event_selection(&pick.selection, &mut *rng)?;
            Some((pick, log_key, severity, payload))
        };
        self.rng_bundle = Some(bundle);
        let (pick, log_key, severity, payload) = result?;
        self.decision_traces_today.push(pick.decision_trace);
        if let Some(trace) = pick.variant_trace {
            self.decision_traces_today.push(trace);
        }
        self.push_log(log_key);
        self.push_event(
            EventKind::RandomEventResolved,
            severity,
            DayTagSet::new(),
            None,
            None,
            payload,
        );
        Some(())
    }

    fn apply_otdeluxe_random_event_selection<R: Rng + ?Sized>(
        &mut self,
        selection: &OtDeluxeRandomEventSelection,
        rng: &mut R,
    ) -> Option<(String, EventSeverity, serde_json::Value)> {
        let event_id = selection.event_id.as_str();
        let variant = selection.variant_id.as_deref()?;
        let log_key = format!("log.otdeluxe.random_event.{event_id}.{variant}");
        let roll = selection.chance_roll;
        let threshold = selection.chance_threshold;

        let (severity, payload) = match event_id {
            "weather_catastrophe" => {
                self.apply_otdeluxe_random_weather_catastrophe(variant, roll, threshold)?
            }
            "resource_shortage" => {
                self.apply_otdeluxe_random_resource_shortage(variant, roll, threshold, rng)?
            }
            "party_incident" => {
                self.apply_otdeluxe_random_party_incident(variant, roll, threshold, rng)?
            }
            "oxen_incident" => {
                self.apply_otdeluxe_random_oxen_incident(variant, roll, threshold)?
            }
            "resource_change" => {
                self.apply_otdeluxe_random_resource_change(variant, roll, threshold)?
            }
            "wagon_part_break" => {
                self.apply_otdeluxe_random_wagon_part_break(variant, roll, threshold, rng)?
            }
            "travel_hazard" => {
                self.apply_otdeluxe_random_travel_hazard(variant, roll, threshold)?
            }
            _ => return None,
        };

        Some((log_key, severity, payload))
    }

    fn apply_otdeluxe_random_weather_catastrophe(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let (health_delta, food_delta, severity) = match variant {
            "blizzard" => (5, -10, EventSeverity::Warning),
            "hailstorm" => (3, -5, EventSeverity::Warning),
            "thunderstorm" => (2, 0, EventSeverity::Info),
            "heavy_fog" | "strong_winds" => (1, 0, EventSeverity::Info),
            _ => return None,
        };
        let applied_health = self.apply_otdeluxe_health_delta(health_delta);
        let applied_food = self.apply_otdeluxe_food_delta(food_delta);
        let payload = serde_json::json!({
            "event": "weather_catastrophe",
            "variant": variant,
            "chance_roll": chance_roll,
            "chance_threshold": chance_threshold,
            "deltas": {
                "health_general": applied_health,
                "food_lbs": applied_food
            }
        });
        Some((severity, payload))
    }

    fn apply_otdeluxe_random_resource_shortage<R: Rng + ?Sized>(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
        rng: &mut R,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let mut affliction_payload = None;
        let (health_delta, oxen_delta, severity) = match variant {
            "bad_water" => {
                let kind = OtDeluxeAfflictionKind::Illness;
                let outcome = self.apply_otdeluxe_random_affliction(rng, kind);
                affliction_payload = outcome.as_ref().map(Self::otdeluxe_affliction_payload);
                (2, (0_i16, 0_i16), EventSeverity::Warning)
            }
            "no_water" => (4, (0_i16, 0_i16), EventSeverity::Warning),
            "no_grass" => {
                let (healthy_delta, sick_delta) = self.apply_otdeluxe_no_grass_loss();
                (0, (healthy_delta, sick_delta), EventSeverity::Warning)
            }
            _ => return None,
        };
        let applied_health = if health_delta != 0 {
            self.apply_otdeluxe_health_delta(health_delta)
        } else {
            0
        };
        let (oxen_healthy_delta, oxen_sick_delta) = oxen_delta;
        let mut deltas = serde_json::Map::new();
        deltas.insert(
            String::from("health_general"),
            serde_json::json!(applied_health),
        );
        deltas.insert(
            String::from("oxen_healthy"),
            serde_json::json!(oxen_healthy_delta),
        );
        deltas.insert(
            String::from("oxen_sick"),
            serde_json::json!(oxen_sick_delta),
        );
        let mut payload = serde_json::Map::new();
        payload.insert(
            String::from("event"),
            serde_json::json!("resource_shortage"),
        );
        payload.insert(String::from("variant"), serde_json::json!(variant));
        payload.insert(String::from("chance_roll"), serde_json::json!(chance_roll));
        payload.insert(
            String::from("chance_threshold"),
            serde_json::json!(chance_threshold),
        );
        payload.insert(String::from("deltas"), serde_json::Value::Object(deltas));
        payload.insert(
            String::from("affliction"),
            affliction_payload.unwrap_or(serde_json::Value::Null),
        );
        let payload = serde_json::Value::Object(payload);
        Some((severity, payload))
    }

    fn apply_otdeluxe_random_party_incident<R: Rng + ?Sized>(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
        rng: &mut R,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let mut affliction_payload = None;
        let lost_members = if variant == "lost_member" {
            self.lose_random_party_members(rng, 1)
        } else if variant == "snakebite" {
            let kind = OtDeluxeAfflictionKind::Injury;
            let outcome = self.apply_otdeluxe_random_affliction(rng, kind);
            affliction_payload = outcome.as_ref().map(Self::otdeluxe_affliction_payload);
            Vec::new()
        } else {
            return None;
        };
        let severity = if variant == "lost_member" {
            EventSeverity::Critical
        } else {
            EventSeverity::Warning
        };
        let mut payload = serde_json::Map::new();
        payload.insert(String::from("event"), serde_json::json!("party_incident"));
        payload.insert(String::from("variant"), serde_json::json!(variant));
        payload.insert(String::from("chance_roll"), serde_json::json!(chance_roll));
        payload.insert(
            String::from("chance_threshold"),
            serde_json::json!(chance_threshold),
        );
        payload.insert(
            String::from("lost_members"),
            serde_json::json!(lost_members),
        );
        payload.insert(
            String::from("affliction"),
            affliction_payload.unwrap_or(serde_json::Value::Null),
        );
        let payload = serde_json::Value::Object(payload);
        Some((severity, payload))
    }

    fn apply_otdeluxe_random_oxen_incident(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let (healthy_delta, sick_delta) = match variant {
            "ox_wandered_off" => self.apply_otdeluxe_oxen_wander(),
            "ox_sickness" => self.apply_otdeluxe_oxen_sickness(),
            _ => return None,
        };
        let payload = serde_json::json!({
            "event": "oxen_incident",
            "variant": variant,
            "chance_roll": chance_roll,
            "chance_threshold": chance_threshold,
            "deltas": {
                "oxen_healthy": healthy_delta,
                "oxen_sick": sick_delta
            }
        });
        Some((EventSeverity::Warning, payload))
    }

    fn apply_otdeluxe_random_resource_change(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let (food_delta, bullets_delta, clothes_delta, spares_delta, severity) = match variant {
            "abandoned_wagon_empty" => (0, 0, 0, (0_i16, 0_i16, 0_i16), EventSeverity::Info),
            "abandoned_wagon_supplies" => (25, 10, 1, (0_i16, 0_i16, 0_i16), EventSeverity::Info),
            "thief" => (-30, -10, 0, (0_i16, 0_i16, 0_i16), EventSeverity::Warning),
            "wild_fruit" => (15, 0, 0, (0_i16, 0_i16, 0_i16), EventSeverity::Info),
            "mutual_aid_food" => (25, 0, 0, (0_i16, 0_i16, 0_i16), EventSeverity::Info),
            "gravesite" => (0, 5, 1, (0_i16, 0_i16, 0_i16), EventSeverity::Info),
            "fire" => (
                -40,
                -20,
                -1,
                (-1_i16, -1_i16, -1_i16),
                EventSeverity::Critical,
            ),
            _ => return None,
        };
        let applied_food = self.apply_otdeluxe_food_delta(food_delta);
        let applied_bullets = self.apply_otdeluxe_bullets_delta(bullets_delta);
        let applied_clothes = self.apply_otdeluxe_clothes_delta(clothes_delta);
        let (wheels_delta, axles_delta, tongues_delta) =
            self.apply_otdeluxe_spares_delta(spares_delta.0, spares_delta.1, spares_delta.2);
        let payload = serde_json::json!({
            "event": "resource_change",
            "variant": variant,
            "chance_roll": chance_roll,
            "chance_threshold": chance_threshold,
            "deltas": {
                "food_lbs": applied_food,
                "bullets": applied_bullets,
                "clothes_sets": applied_clothes,
                "spares_wheels": wheels_delta,
                "spares_axles": axles_delta,
                "spares_tongues": tongues_delta
            }
        });
        Some((severity, payload))
    }

    fn apply_otdeluxe_random_wagon_part_break<R: Rng + ?Sized>(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
        rng: &mut R,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let (fallback_food, fallback_clothes, severity) = match variant {
            "repairable" => (5, 0, EventSeverity::Warning),
            "replaceable" => (10, 0, EventSeverity::Warning),
            "unrepairable" => (15, 1, EventSeverity::Critical),
            _ => return None,
        };
        let spare_lost = self.lose_random_spare(rng);
        let mut applied_food = 0;
        let mut applied_clothes = 0;
        if spare_lost.is_none() {
            applied_food = self.apply_otdeluxe_food_delta(-fallback_food);
            if fallback_clothes != 0 {
                applied_clothes = self.apply_otdeluxe_clothes_delta(-fallback_clothes);
            }
        }
        let mut deltas = serde_json::Map::new();
        deltas.insert(String::from("food_lbs"), serde_json::json!(applied_food));
        deltas.insert(
            String::from("clothes_sets"),
            serde_json::json!(applied_clothes),
        );
        let mut payload = serde_json::Map::new();
        payload.insert(String::from("event"), serde_json::json!("wagon_part_break"));
        payload.insert(String::from("variant"), serde_json::json!(variant));
        payload.insert(String::from("chance_roll"), serde_json::json!(chance_roll));
        payload.insert(
            String::from("chance_threshold"),
            serde_json::json!(chance_threshold),
        );
        payload.insert(String::from("spare_lost"), serde_json::json!(spare_lost));
        payload.insert(String::from("deltas"), serde_json::Value::Object(deltas));
        let payload = serde_json::Value::Object(payload);
        Some((severity, payload))
    }

    fn apply_otdeluxe_random_travel_hazard(
        &mut self,
        variant: &str,
        chance_roll: f32,
        chance_threshold: f32,
    ) -> Option<(EventSeverity, serde_json::Value)> {
        let (health_delta, food_delta, severity) = match variant {
            "rough_trail" => (2, -5, EventSeverity::Warning),
            _ => return None,
        };
        let applied_health = self.apply_otdeluxe_health_delta(health_delta);
        let applied_food = self.apply_otdeluxe_food_delta(food_delta);
        let payload = serde_json::json!({
            "event": "travel_hazard",
            "variant": variant,
            "chance_roll": chance_roll,
            "chance_threshold": chance_threshold,
            "deltas": {
                "health_general": applied_health,
                "food_lbs": applied_food
            }
        });
        Some((severity, payload))
    }

    fn apply_otdeluxe_random_affliction<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        kind: OtDeluxeAfflictionKind,
    ) -> Option<OtDeluxeAfflictionOutcome> {
        let catalog = DiseaseCatalog::default_catalog();
        self.apply_otdeluxe_random_affliction_with_catalog(catalog, rng, kind)
    }

    fn apply_otdeluxe_random_affliction_with_catalog<R: Rng + ?Sized>(
        &mut self,
        catalog: &DiseaseCatalog,
        rng: &mut R,
        kind: OtDeluxeAfflictionKind,
    ) -> Option<OtDeluxeAfflictionOutcome> {
        let policy = default_otdeluxe_policy();
        let disease_kind = match kind {
            OtDeluxeAfflictionKind::Illness => DiseaseKind::Illness,
            OtDeluxeAfflictionKind::Injury => DiseaseKind::Injury,
        };
        let (disease, trace) = catalog.pick_by_kind_with_trace(disease_kind, rng);
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }
        let duration = disease.map_or_else(
            || match kind {
                OtDeluxeAfflictionKind::Illness => policy.affliction.illness_duration_days,
                OtDeluxeAfflictionKind::Injury => policy.affliction.injury_duration_days,
            },
            |selected| selected.duration_for(&policy.affliction),
        );
        let disease_id = disease.map(|selected| selected.id.as_str());
        let party = &mut self.ot_deluxe.party;
        let mut outcome = party.apply_affliction_random(rng, kind, duration, disease_id);
        if let (Some(selected), Some(ref mut result)) = (disease, outcome.as_mut()) {
            result.disease_id = Some(selected.id.clone());
            result.display_key = Some(selected.display_key.clone());
            let onset_mult = self.apply_otdeluxe_disease_onset(selected);
            self.ot_deluxe.travel.disease_speed_mult =
                sanitize_disease_multiplier(self.ot_deluxe.travel.disease_speed_mult * onset_mult);
        }
        outcome
    }

    fn otdeluxe_affliction_payload(outcome: &OtDeluxeAfflictionOutcome) -> serde_json::Value {
        serde_json::json!({ "member_index": outcome.member_index, "died": outcome.died, "kind": Self::otdeluxe_affliction_kind_key(outcome.kind), "disease_id": outcome.disease_id, "display_key": outcome.display_key })
    }

    const fn otdeluxe_affliction_kind_key(kind: OtDeluxeAfflictionKind) -> &'static str {
        match kind {
            OtDeluxeAfflictionKind::Illness => "illness",
            OtDeluxeAfflictionKind::Injury => "injury",
        }
    }

    fn lose_random_party_members<R: Rng + ?Sized>(&mut self, rng: &mut R, count: u8) -> Vec<usize> {
        let mut alive_indices = Vec::new();
        for (idx, member) in self.ot_deluxe.party.members.iter().enumerate() {
            if member.alive {
                alive_indices.push(idx);
            }
        }
        let mut lost = Vec::new();
        while !alive_indices.is_empty() && lost.len() < usize::from(count) {
            let idx = rng.gen_range(0..alive_indices.len());
            let member_idx = alive_indices.swap_remove(idx);
            if let Some(member) = self.ot_deluxe.party.members.get_mut(member_idx) {
                member.alive = false;
                lost.push(member_idx);
            }
        }
        lost
    }

    fn lose_random_spare<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<&'static str> {
        let wheels = self.ot_deluxe.inventory.spares_wheels;
        let axles = self.ot_deluxe.inventory.spares_axles;
        let tongues = self.ot_deluxe.inventory.spares_tongues;
        let total = u32::from(wheels) + u32::from(axles) + u32::from(tongues);
        if total == 0 {
            return None;
        }
        let roll = rng.gen_range(0..total);
        let mut cursor = u32::from(wheels);
        if roll < cursor {
            self.ot_deluxe.inventory.spares_wheels =
                self.ot_deluxe.inventory.spares_wheels.saturating_sub(1);
            return Some("wheel");
        }
        cursor = cursor.saturating_add(u32::from(axles));
        if roll < cursor {
            self.ot_deluxe.inventory.spares_axles =
                self.ot_deluxe.inventory.spares_axles.saturating_sub(1);
            return Some("axle");
        }
        self.ot_deluxe.inventory.spares_tongues =
            self.ot_deluxe.inventory.spares_tongues.saturating_sub(1);
        Some("tongue")
    }

    fn apply_otdeluxe_health_delta(&mut self, delta: i32) -> i32 {
        Self::apply_u16_delta(&mut self.ot_deluxe.health_general, delta)
    }

    fn apply_otdeluxe_food_delta(&mut self, delta: i32) -> i32 {
        Self::apply_u16_delta(&mut self.ot_deluxe.inventory.food_lbs, delta)
    }

    fn apply_otdeluxe_bullets_delta(&mut self, delta: i32) -> i32 {
        Self::apply_u16_delta(&mut self.ot_deluxe.inventory.bullets, delta)
    }

    fn apply_otdeluxe_clothes_delta(&mut self, delta: i32) -> i32 {
        Self::apply_u16_delta(&mut self.ot_deluxe.inventory.clothes_sets, delta)
    }

    fn apply_otdeluxe_spares_delta(
        &mut self,
        wheels: i16,
        axles: i16,
        tongues: i16,
    ) -> (i16, i16, i16) {
        let wheels_delta =
            Self::apply_u8_delta(&mut self.ot_deluxe.inventory.spares_wheels, wheels);
        let axles_delta = Self::apply_u8_delta(&mut self.ot_deluxe.inventory.spares_axles, axles);
        let tongues_delta =
            Self::apply_u8_delta(&mut self.ot_deluxe.inventory.spares_tongues, tongues);
        (wheels_delta, axles_delta, tongues_delta)
    }

    const fn apply_otdeluxe_no_grass_loss(&mut self) -> (i16, i16) {
        if self.ot_deluxe.oxen.healthy > 0 {
            self.ot_deluxe.oxen.healthy = self.ot_deluxe.oxen.healthy.saturating_sub(1);
            self.ot_deluxe.oxen.sick = self.ot_deluxe.oxen.sick.saturating_add(1);
            (-1, 1)
        } else if self.ot_deluxe.oxen.sick > 0 {
            self.ot_deluxe.oxen.sick = self.ot_deluxe.oxen.sick.saturating_sub(1);
            (0, -1)
        } else {
            (0, 0)
        }
    }

    const fn apply_otdeluxe_oxen_wander(&mut self) -> (i16, i16) {
        if self.ot_deluxe.oxen.healthy > 0 {
            self.ot_deluxe.oxen.healthy = self.ot_deluxe.oxen.healthy.saturating_sub(1);
            (-1, 0)
        } else if self.ot_deluxe.oxen.sick > 0 {
            self.ot_deluxe.oxen.sick = self.ot_deluxe.oxen.sick.saturating_sub(1);
            (0, -1)
        } else {
            (0, 0)
        }
    }

    const fn apply_otdeluxe_oxen_sickness(&mut self) -> (i16, i16) {
        if self.ot_deluxe.oxen.healthy > 0 {
            self.ot_deluxe.oxen.healthy = self.ot_deluxe.oxen.healthy.saturating_sub(1);
            self.ot_deluxe.oxen.sick = self.ot_deluxe.oxen.sick.saturating_add(1);
            (-1, 1)
        } else if self.ot_deluxe.oxen.sick > 0 {
            self.ot_deluxe.oxen.sick = self.ot_deluxe.oxen.sick.saturating_sub(1);
            (0, -1)
        } else {
            (0, 0)
        }
    }

    fn apply_u16_delta(value: &mut u16, delta: i32) -> i32 {
        if delta == 0 {
            return 0;
        }
        let current = i32::from(*value);
        let next = (current + delta).clamp(0, i32::from(u16::MAX));
        *value = u16::try_from(next).unwrap_or(u16::MAX);
        next - current
    }

    fn apply_u8_delta(value: &mut u8, delta: i16) -> i16 {
        if delta == 0 {
            return 0;
        }
        let current = i16::from(*value);
        let next = (current + delta).clamp(0, i16::from(u8::MAX));
        *value = u8::try_from(next).unwrap_or(u8::MAX);
        next - current
    }

    pub(crate) fn handle_vehicle_state(
        &mut self,
        breakdown_started: bool,
    ) -> Option<(bool, String, bool)> {
        if self.check_vehicle_terminal_state() {
            self.end_of_day();
            Some((true, String::from(LOG_VEHICLE_FAILURE), breakdown_started))
        } else {
            None
        }
    }

    pub(crate) fn handle_travel_block(
        &mut self,
        breakdown_started: bool,
    ) -> Option<(bool, String, bool)> {
        if self.day_state.travel.travel_blocked {
            if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
                self.clear_today_travel_distance();
                if self.current_day_kind.is_none() {
                    self.record_travel_day(TravelDayKind::NonTravel, 0.0, "repair");
                } else {
                    self.add_day_reason_tag("repair");
                }
            } else if !self.day_state.travel.partial_traveled_today {
                self.apply_delay_travel_credit("repair");
            }
            self.push_event(
                EventKind::TravelBlocked,
                EventSeverity::Warning,
                DayTagSet::new(),
                None,
                None,
                serde_json::json!({
                    "reason": "vehicle_breakdown",
                    "breakdown_started": breakdown_started
                }),
            );
            self.end_of_day();
            Some((false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started))
        } else {
            None
        }
    }

    pub(crate) fn process_encounter_flow(
        &mut self,
        rng_bundle: Option<&Rc<RngBundle>>,
        breakdown_started: bool,
    ) -> Option<(bool, String, bool)> {
        if self.encounters.occurred_today || self.encounters_today >= MAX_ENCOUNTERS_PER_DAY {
            return None;
        }

        let trigger_encounter = self.should_trigger_encounter(rng_bundle);
        if !trigger_encounter {
            return None;
        }

        let recent_snapshot: Vec<RecentEncounter> =
            self.recent_encounters.iter().cloned().collect();
        let mut rotation_backlog = std::mem::take(&mut self.rotation_backlog);
        let mut encounter = None;
        let mut force_rotation_pending = self.encounters.force_rotation_pending;
        let mut rotation_logged = false;
        if let (Some(bundle), Some(data)) = (rng_bundle, self.data.as_ref()) {
            let forced = force_rotation_pending;
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
            {
                let mut rng = bundle.encounter();
                let pick = pick_encounter(&request, &mut rotation_backlog, &mut *rng);
                if let Some(trace) = pick.decision_trace {
                    self.decision_traces_today.push(trace);
                }
                let satisfied = pick.rotation_satisfied;
                if forced {
                    if satisfied {
                        rotation_logged = true;
                    }
                    force_rotation_pending = !rotation_backlog.is_empty();
                }
                encounter = pick.encounter;
            }
        }
        self.encounters.force_rotation_pending = force_rotation_pending;

        let encounter =
            self.maybe_reroll_encounter(rng_bundle, &recent_snapshot, rotation_backlog, encounter);
        if rotation_logged {
            self.push_log(LOG_ENCOUNTER_ROTATION);
        }
        if let Some(enc) = encounter {
            let is_hard_stop = enc.hard_stop;
            let is_major_repair = enc.major_repair;
            let is_chainable = enc.chainable;
            if is_major_repair {
                self.record_travel_day(TravelDayKind::NonTravel, 0.0, "repair");
            }
            let encounter_id = enc.id.clone();
            self.current_encounter = Some(enc);
            self.encounters.occurred_today = true;
            self.record_encounter(&encounter_id);
            self.push_event(
                EventKind::EncounterTriggered,
                EventSeverity::Info,
                DayTagSet::new(),
                None,
                None,
                serde_json::json!({
                    "id": encounter_id,
                    "hard_stop": is_hard_stop,
                    "major_repair": is_major_repair,
                    "chainable": is_chainable
                }),
            );
            return Some((false, String::from("log.encounter"), breakdown_started));
        }

        None
    }

    pub(crate) fn apply_encounter_partial_travel(&mut self) {
        if !self.features.travel_v2 || !self.encounters.occurred_today {
            return;
        }
        let Some(encounter) = self.current_encounter.as_ref() else {
            return;
        };
        if encounter.hard_stop || encounter.major_repair {
            return;
        }
        if self.distance_today <= 0.0 {
            return;
        }
        let mut partial = if self.partial_distance_today > 0.0 {
            self.partial_distance_today
        } else {
            self.distance_today * TRAVEL_PARTIAL_RECOVERY_RATIO
        };
        partial = partial.min(self.distance_today);
        let wear_scale = (partial / self.distance_today)
            .clamp(TRAVEL_PARTIAL_CLAMP_LOW, TRAVEL_PARTIAL_CLAMP_HIGH);
        self.record_travel_day(TravelDayKind::Partial, partial, "");
        self.apply_travel_wear_scaled(wear_scale);
        self.push_log(LOG_TRAVEL_PARTIAL);
    }

    fn should_trigger_encounter(&self, rng_bundle: Option<&Rc<RngBundle>>) -> bool {
        let Some(bundle) = rng_bundle else {
            return false;
        };
        let roll = {
            let mut rng = bundle.encounter();
            rng.r#gen::<f32>()
        };
        roll < self.encounter_chance_today
    }

    fn maybe_reroll_encounter(
        &mut self,
        rng_bundle: Option<&Rc<RngBundle>>,
        recent_snapshot: &[RecentEncounter],
        mut rotation_backlog: VecDeque<String>,
        encounter: Option<Encounter>,
    ) -> Option<Encounter> {
        let unique_ratio = self.encounter_unique_ratio(ENCOUNTER_UNIQUE_WINDOW);
        let enforce_unique = unique_ratio < ENCOUNTER_UNIQUE_RATIO_FLOOR;
        let should_reroll = encounter.as_ref().is_some_and(|enc| {
            let diversity_reroll =
                self.features.encounter_diversity && self.should_discourage_encounter(&enc.id);
            let recent_repeat = self
                .recent_encounters
                .iter()
                .rev()
                .take(usize::try_from(ENCOUNTER_UNIQUE_WINDOW).unwrap_or(20))
                .any(|entry| entry.id == enc.id);
            diversity_reroll || (enforce_unique && recent_repeat)
        });

        let mut encounter = encounter;
        if should_reroll {
            let reroll_penalty = self.encounter_reroll_penalty();
            let reroll_trigger = rng_bundle.is_some_and(|bundle| {
                let mut rng = bundle.encounter();
                rng.r#gen::<f32>() < reroll_penalty
            });
            if reroll_trigger && let (Some(bundle), Some(data)) = (rng_bundle, self.data.as_ref()) {
                let request = EncounterRequest {
                    region: self.region,
                    is_deep: self.mode.is_deep(),
                    malnutrition_level: self.malnutrition_level,
                    starving: self.stats.supplies <= 0,
                    data,
                    recent: recent_snapshot,
                    current_day: self.day,
                    policy: self.policy,
                    force_rotation: false,
                };
                {
                    let mut rng = bundle.encounter();
                    let replacement = pick_encounter(&request, &mut rotation_backlog, &mut *rng);
                    if let Some(trace) = replacement.decision_trace {
                        self.decision_traces_today.push(trace);
                    }
                    let satisfied = replacement.rotation_satisfied;
                    if satisfied {
                        self.encounters.force_rotation_pending = false;
                    }
                    encounter = replacement.encounter;
                }
            }
        }
        self.rotation_backlog = rotation_backlog;
        encounter
    }

    pub(crate) fn log_travel_debug(&self) {
        if debug_log_enabled() {
            println!(
                "Day {}: distance {:.1}/{:.1} (actual {:.1}), boss.ready {}, HP {}, Sanity {}",
                self.day,
                self.miles_traveled,
                self.trail_distance,
                self.miles_traveled_actual,
                self.boss.readiness.ready,
                self.stats.hp,
                self.stats.sanity
            );
        }
    }

    fn select_breakdown_part_with_trace<R: rand::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> (Part, Option<EventDecisionTrace>) {
        let choices = [
            (Part::Tire, self.journey_part_weights.tire),
            (Part::Battery, self.journey_part_weights.battery),
            (Part::Alternator, self.journey_part_weights.alt),
            (Part::FuelPump, self.journey_part_weights.pump),
        ];
        let mut total = 0_u32;
        for (_, weight) in &choices {
            total = total.saturating_add(*weight);
        }
        if total == 0 {
            return (Part::Tire, None);
        }
        let roll = rng.gen_range(0..total);
        let mut current = 0_u32;
        let mut selected = None;
        for (part, weight) in &choices {
            current = current.saturating_add(*weight);
            if selected.is_none() && roll < current {
                selected = Some(*part);
            }
        }
        let selected = selected.unwrap_or(Part::Tire);
        let candidates = choices
            .iter()
            .map(|(part, weight)| WeightedCandidate {
                id: part.key().to_string(),
                base_weight: f64::from(*weight),
                multipliers: Vec::new(),
                final_weight: f64::from(*weight),
            })
            .collect();
        let trace = EventDecisionTrace {
            pool_id: String::from("dystrail.breakdown_part"),
            roll: RollValue::U32(roll),
            candidates,
            chosen_id: selected.key().to_string(),
        };
        (selected, Some(trace))
    }

    fn sanitize_breakdown_max_chance(max_chance: f32) -> f32 {
        if max_chance.is_finite() && max_chance > 0.0 {
            max_chance
        } else {
            PROBABILITY_MAX
        }
    }

    /// Apply vehicle breakdown logic
    pub(crate) fn vehicle_roll(&mut self) -> bool {
        if self.breakdown.is_some() {
            return false;
        }

        if self.vehicle.breakdown_suppressed() {
            return false;
        }

        let wear_level = self.vehicle.wear.max(0.0);
        let (base, beta, pace_factor, max_chance) =
            if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
                let policy = default_otdeluxe_policy();
                (
                    policy.breakdown.base,
                    policy.breakdown.beta,
                    policy.breakdown.pace_multiplier(self.ot_deluxe.pace),
                    policy.breakdown.max_chance,
                )
            } else {
                (
                    self.journey_breakdown.base,
                    self.journey_breakdown.beta,
                    self.journey_pace_factor(),
                    0.35,
                )
            };
        let mut breakdown_chance =
            base * beta.mul_add(wear_level, 1.0) * pace_factor * self.journey_weather_factor();
        breakdown_chance = (breakdown_chance + self.exec_effects.breakdown_bonus)
            .clamp(PROBABILITY_FLOOR, PROBABILITY_MAX);

        if self.endgame.active && (0.0..1.0).contains(&self.endgame.breakdown_scale) {
            breakdown_chance *= self.endgame.breakdown_scale;
        }
        if self.mechanical_policy == MechanicalPolicyId::DystrailLegacy {
            if matches!(self.policy, Some(PolicyKind::Aggressive)) {
                breakdown_chance = breakdown_chance.max(0.01);
            }
            if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive)) {
                breakdown_chance *= 0.7;
            }
        }
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            let policy = default_otdeluxe_policy();
            breakdown_chance *=
                otdeluxe_mobility_failure_mult(self.ot_deluxe.mods.occupation, policy);
        }
        let max_chance = Self::sanitize_breakdown_max_chance(max_chance);
        breakdown_chance = breakdown_chance
            .clamp(PROBABILITY_FLOOR, PROBABILITY_MAX)
            .min(max_chance);

        let roll = self
            .breakdown_rng()
            .map_or(1.0, |mut rng| rng.r#gen::<f32>());
        if roll >= breakdown_chance {
            return false;
        }

        let (part, trace) = self.breakdown_rng().map_or((Part::Tire, None), |mut rng| {
            self.select_breakdown_part_with_trace(&mut *rng)
        });
        if let Some(trace) = trace {
            self.decision_traces_today.push(trace);
        }
        self.last_breakdown_part = Some(part);
        self.breakdown = Some(crate::vehicle::Breakdown {
            part,
            day_started: i32::try_from(self.day).unwrap_or(0),
        });
        self.day_state.travel.travel_blocked = true;
        self.vehicle_breakdowns += 1;
        self.vehicle.apply_damage(VEHICLE_BREAKDOWN_DAMAGE);
        let breakdown_wear = if self.mode.is_deep() {
            VEHICLE_BREAKDOWN_WEAR
        } else {
            VEHICLE_BREAKDOWN_WEAR_CLASSIC
        };
        self.vehicle.wear = (self.vehicle.wear + breakdown_wear).min(VEHICLE_HEALTH_MAX);
        self.mark_damage(DamageCause::Vehicle);
        self.push_event(
            EventKind::BreakdownStarted,
            EventSeverity::Warning,
            DayTagSet::new(),
            None,
            None,
            serde_json::json!({
                "part": part.key(),
                "roll": roll,
                "chance": breakdown_chance
            }),
        );
        if debug_log_enabled() {
            println!(
                "🚗 Breakdown started: {:?} | health {} | roll {:.3} chance {:.3}",
                part, self.vehicle.health, roll, breakdown_chance
            );
        }
        true
    }

    /// Test helper exposing the breakdown roll with current configuration.
    pub fn vehicle_roll_for_testing(&mut self) -> bool {
        self.vehicle_roll()
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
                self.push_log(log.clone());
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
                    TRAVEL_V2_BASE_DISTANCE
                } else {
                    TRAVEL_CLASSIC_BASE_DISTANCE
                };
                let bonus = (baseline * eff.travel_bonus_ratio).max(0.0);
                if bonus > 0.0 {
                    self.apply_partial_travel_credit(bonus, LOG_TRAVEL_BONUS, "");
                }
            }
            if eff.rest {
                if !self.day_state.rest.rest_requested {
                    self.push_log(LOG_REST_REQUESTED_ENCOUNTER);
                }
                self.request_rest();
            }
        }

        self.finalize_encounter();
    }

    pub const fn set_crossing_choice(&mut self, choice: CrossingChoice) {
        self.pending_crossing_choice = Some(choice);
    }

    pub const fn set_otdeluxe_crossing_choice(&mut self, method: OtDeluxeCrossingMethod) {
        self.ot_deluxe.crossing.chosen_method = Some(method);
    }

    pub const fn set_route_prompt_choice(&mut self, choice: OtDeluxeRouteDecision) {
        self.pending_route_choice = Some(choice);
    }

    pub(crate) fn resolve_breakdown(&mut self) {
        if self.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            if let Some(breakdown) = self.breakdown.clone() {
                if self.consume_otdeluxe_spare_for_breakdown(breakdown.part) {
                    self.vehicle.repair(VEHICLE_JURY_RIG_HEAL);
                    self.breakdown = None;
                    self.day_state.travel.travel_blocked = false;
                    if matches!(
                        self.ot_deluxe.travel.wagon_state,
                        OtDeluxeWagonState::Blocked
                    ) {
                        self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Moving;
                    }
                    self.last_breakdown_part = None;
                    self.push_log("log.breakdown-repaired");
                    self.push_event(
                        EventKind::BreakdownResolved,
                        EventSeverity::Info,
                        DayTagSet::new(),
                        None,
                        None,
                        serde_json::json!({
                            "part": breakdown.part.key(),
                            "resolution": "spare"
                        }),
                    );
                } else {
                    self.day_state.travel.travel_blocked = true;
                    self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Blocked;
                }
            } else {
                self.day_state.travel.travel_blocked = false;
            }
            return;
        }
        if let Some(breakdown) = self.breakdown.clone() {
            if self.consume_spare_for_part(breakdown.part) {
                self.vehicle.repair(VEHICLE_JURY_RIG_HEAL);
                self.breakdown = None;
                self.day_state.travel.travel_blocked = false;
                self.last_breakdown_part = None;
                self.push_log("log.breakdown-repaired");
                self.push_event(
                    EventKind::BreakdownResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::json!({
                        "part": breakdown.part.key(),
                        "resolution": "spare"
                    }),
                );
                return;
            }

            if self.total_spares() == 0 && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.spend_emergency_repair(LOG_VEHICLE_REPAIR_EMERGENCY);
                self.breakdown = None;
                self.day_state.travel.travel_blocked = false;
                self.last_breakdown_part = None;
                self.push_event(
                    EventKind::BreakdownResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::json!({
                        "part": breakdown.part.key(),
                        "resolution": "emergency"
                    }),
                );
                return;
            }

            let day_started = u32::try_from(breakdown.day_started).unwrap_or(0);
            if self.day.saturating_sub(day_started) >= 1 {
                self.vehicle
                    .apply_damage(VEHICLE_BREAKDOWN_DAMAGE * VEHICLE_BREAKDOWN_PARTIAL_FACTOR);
                self.mark_damage(DamageCause::Vehicle);
                self.breakdown = None;
                self.day_state.travel.travel_blocked = false;
                self.last_breakdown_part = None;
                self.push_log("log.breakdown-jury-rigged");
                self.push_event(
                    EventKind::BreakdownResolved,
                    EventSeverity::Info,
                    DayTagSet::new(),
                    None,
                    None,
                    serde_json::json!({
                        "part": breakdown.part.key(),
                        "resolution": "jury_rigged"
                    }),
                );
            } else {
                self.day_state.travel.travel_blocked = true;
            }
        } else {
            self.day_state.travel.travel_blocked = false;
        }
    }

    const fn consume_otdeluxe_spare_for_breakdown(&mut self, part: Part) -> bool {
        let inventory = &mut self.ot_deluxe.inventory;
        match otdeluxe_spare_for_breakdown(part) {
            OtDeluxeSparePart::Wheel if inventory.spares_wheels > 0 => {
                inventory.spares_wheels -= 1;
                true
            }
            OtDeluxeSparePart::Axle if inventory.spares_axles > 0 => {
                inventory.spares_axles -= 1;
                true
            }
            OtDeluxeSparePart::Tongue if inventory.spares_tongues > 0 => {
                inventory.spares_tongues -= 1;
                true
            }
            _ => false,
        }
    }

    pub(crate) const fn consume_spare_for_part(&mut self, part: Part) -> bool {
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

    pub(crate) fn consume_any_spare_for_emergency(&mut self) -> bool {
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
        self.exec_effects.travel_multiplier = (self.exec_effects.travel_multiplier
            * VEHICLE_EXEC_MULTIPLIER_DECAY)
            .max(VEHICLE_EXEC_MULTIPLIER_FLOOR);
        self.push_log(LOG_VEHICLE_REPAIR_SPARE);
        true
    }

    pub(crate) fn spend_emergency_repair(&mut self, log_key: &'static str) {
        self.budget_cents = (self.budget_cents - EMERGENCY_REPAIR_COST).max(0);
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += EMERGENCY_REPAIR_COST;
        let mut repair_amount = VEHICLE_EMERGENCY_HEAL;
        if self.mode.is_deep() && self.miles_traveled_actual >= DEEP_EMERGENCY_REPAIR_THRESHOLD {
            let boost = if matches!(self.policy, Some(PolicyKind::Aggressive)) {
                VEHICLE_DEEP_EMERGENCY_HEAL_AGGRESSIVE
            } else {
                VEHICLE_DEEP_EMERGENCY_HEAL_BALANCED
            };
            repair_amount = repair_amount.max(boost);
        }
        self.vehicle.repair(repair_amount);
        self.exec_effects.travel_multiplier = (self.exec_effects.travel_multiplier
            * VEHICLE_EXEC_MULTIPLIER_DECAY)
            .max(VEHICLE_EXEC_MULTIPLIER_FLOOR);
        self.push_log(log_key);
    }

    pub fn next_u32(&mut self) -> u32 {
        self.boss_rng().map_or(0, |mut rng| rng.next_u32())
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
            self.push_log(LOG_DISEASE_RECOVER);
            self.disease_cooldown = self.disease_cooldown.max(DISEASE_COOLDOWN_DAYS);
        }
    }

    fn apply_encounter_chance_today(
        &mut self,
        pace_delta: f32,
        weather_delta: f32,
        exec_delta: f32,
        weather_cap: f32,
        limits: &PacingLimits,
    ) {
        let encounter_base = if limits.encounter_base == 0.0 {
            ENCOUNTER_BASE_DEFAULT
        } else {
            limits.encounter_base
        };
        let encounter_floor = limits.encounter_floor;
        let base_ceiling = if limits.encounter_ceiling == 0.0 {
            1.0
        } else {
            limits.encounter_ceiling
        };
        let encounter_ceiling = base_ceiling.min(weather_cap);
        let mut encounter = encounter_base + pace_delta + weather_delta + exec_delta;

        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            encounter = (encounter + ENCOUNTER_CRITICAL_VEHICLE_BONUS)
                .clamp(encounter_floor, encounter_ceiling);
        }

        let encounters_last_window: u32 =
            self.encounter_history.iter().copied().map(u32::from).sum();
        if encounters_last_window >= ENCOUNTER_SOFT_CAP_THRESHOLD {
            encounter *= ENCOUNTER_SOFT_CAP_FACTOR;
        }

        if self.encounters_today >= MAX_ENCOUNTERS_PER_DAY
            || (self.encounter_cooldown > 0 && self.encounters_today == 0)
        {
            encounter = PROBABILITY_FLOOR;
        }

        self.encounter_chance_today = encounter
            .clamp(encounter_floor, encounter_ceiling)
            .max(PROBABILITY_FLOOR);
    }

    /// Apply pace and diet configuration (placeholder)
    pub fn apply_pace_and_diet(&mut self, cfg: &crate::pacing::PacingConfig) {
        let pace_cfg = cfg.get_pace_safe(self.pace.as_str());
        let diet_cfg = cfg.get_diet_safe(self.diet.as_str());
        let limits = &cfg.limits;

        let (weather_delta, weather_cap) = self.encounter_weather_adjustment();
        let exec_delta = self.exec_effects.encounter_delta;
        self.apply_encounter_chance_today(
            pace_cfg.encounter_chance_delta,
            weather_delta,
            exec_delta,
            weather_cap,
            limits,
        );

        let pants_floor = limits.pants_floor;
        let pants_ceiling = limits.pants_ceiling;
        let mut pants_value = self.stats.pants;

        if limits.passive_relief != 0 && pants_value >= limits.passive_relief_threshold {
            pants_value = (pants_value + limits.passive_relief).clamp(pants_floor, pants_ceiling);
        }

        if self.mods.pants_relief != 0 && pants_value >= self.mods.pants_relief_threshold {
            pants_value = (pants_value + self.mods.pants_relief).clamp(pants_floor, pants_ceiling);
        }

        let boss_stage = self.boss.readiness.ready || self.miles_traveled >= self.trail_distance;
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

    pub fn compute_travel_distance_today(&mut self, cfg: &crate::pacing::PacingConfig) -> f32 {
        let pace_cfg = cfg.get_pace_safe(self.pace.as_str());
        let limits = &cfg.limits;
        self.compute_miles_for_today(&pace_cfg, limits)
    }

    pub const fn apply_otdeluxe_pace_and_rations(&mut self) {
        self.encounter_chance_today = 0.0;
    }

    pub(crate) fn compute_otdeluxe_travel_distance_today(&mut self) -> f32 {
        let policy = default_otdeluxe_policy();
        self.compute_otdeluxe_miles_for_today(policy)
    }

    const fn encounter_weather_adjustment(&self) -> (f32, f32) {
        (
            self.weather_effects.encounter_delta,
            self.weather_effects.encounter_cap,
        )
    }

    /// Save game state (placeholder - platform specific)
    pub const fn save(&self) {
        // Placeholder - web implementation will handle this
    }

    /// Load game state (placeholder - platform specific)
    #[must_use]
    pub const fn load() -> Option<Self> {
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
        self.push_log("log.party.updated");
    }

    pub const fn request_rest(&mut self) {
        self.day_state.rest.rest_requested = true;
    }

    pub(crate) fn failure_log_key(&mut self) -> Option<&'static str> {
        if self.vehicle.health <= 0.0 {
            if self.mode == GameMode::Classic
                && matches!(self.policy, Some(PolicyKind::Balanced))
                && self.miles_traveled_actual < CLASSIC_BALANCED_FAILURE_GUARD_MILES
            {
                self.apply_classic_field_repair_guard();
                return None;
            }
            if self.try_deep_aggressive_field_repair() {
                return None;
            }
            if self.try_emergency_limp_guard() {
                return None;
            }
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
        if sanity_delta != 0 {
            let max_sanity = Stats::default().sanity;
            self.stats.sanity = (self.stats.sanity + sanity_delta).clamp(0, max_sanity);
        }
        if supplies_delta != 0 {
            self.stats.supplies = (self.stats.supplies + supplies_delta).max(0);
        }
        if debug_log_enabled() && (sanity_delta != 0 || supplies_delta != 0) {
            println!("Daily effects applied | sanity {sanity_delta} | supplies {supplies_delta}");
        }
        self.stats.clamp();
    }

    pub fn advance_days(&mut self, days: u32) {
        self.advance_days_with_reason(days, "");
    }

    pub fn advance_days_with_reason(&mut self, days: u32, reason_tag: &str) {
        self.advance_days_with_credit(days, TravelDayKind::NonTravel, 0.0, reason_tag);
    }

    pub fn advance_days_with_credit(
        &mut self,
        days: u32,
        kind: TravelDayKind,
        miles: f32,
        reason_tag: &str,
    ) {
        if days == 0 {
            return;
        }
        for _ in 0..days {
            if matches!(kind, TravelDayKind::NonTravel) && miles <= 0.0 {
                self.day_state.lifecycle.suppress_stop_ratio = true;
            }
            let _ = crate::journey::tick_non_travel_day_for_state(self, kind, miles, reason_tag);
            self.day_state.lifecycle.suppress_stop_ratio = false;
        }
    }

    pub const fn tick_camp_cooldowns(&mut self) {
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
    pub const fn should_auto_rest(&self) -> bool {
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

    /// Apply an `OTDeluxe` store purchase to the run state.
    ///
    /// # Errors
    ///
    /// Returns an error if the purchase exceeds caps or cash on hand.
    pub fn apply_otdeluxe_store_purchase(
        &mut self,
        node_index: u8,
        lines: &[OtDeluxeStoreLineItem],
    ) -> Result<OtDeluxeStoreReceipt, OtDeluxeStoreError> {
        let policy = default_otdeluxe_policy();
        crate::otdeluxe_store::apply_purchase(
            &policy.store,
            node_index,
            &mut self.ot_deluxe.inventory,
            &mut self.ot_deluxe.oxen,
            lines,
        )
    }

    pub fn set_otdeluxe_store_purchase(&mut self, lines: Vec<OtDeluxeStoreLineItem>) -> bool {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return false;
        }
        if self.ot_deluxe.store.pending_node.is_none() {
            return false;
        }
        self.ot_deluxe.store.pending_purchase = Some(lines);
        true
    }

    pub fn clear_otdeluxe_store_pending(&mut self) {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if let Some(node) = self.ot_deluxe.store.pending_node.take() {
            self.ot_deluxe.store.last_node = Some(node);
        }
        self.ot_deluxe.store.pending_purchase = None;
    }

    pub(crate) fn queue_otdeluxe_store_if_available(&mut self) {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if self.ot_deluxe.store.pending_node.is_some() {
            return;
        }
        let node = self.ot_deluxe.route.current_node_index;
        if self.ot_deluxe.store.last_node == Some(node) {
            return;
        }
        if self.otdeluxe_store_available() {
            self.ot_deluxe.store.pending_node = Some(node);
            self.ot_deluxe.store.pending_purchase = None;
        }
    }

    #[must_use]
    pub fn otdeluxe_store_available(&self) -> bool {
        let policy = default_otdeluxe_policy();
        otdeluxe_trail::store_available_at_node(
            &policy.trail,
            &policy.store,
            self.ot_deluxe.route.variant,
            self.ot_deluxe.route.current_node_index,
        )
    }

    pub fn resolve_otdeluxe_route_prompt(&mut self, decision: OtDeluxeRouteDecision) -> bool {
        if self.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return false;
        }
        let Some(prompt) = self.ot_deluxe.route.pending_prompt else {
            return false;
        };
        let mut handled = false;
        match prompt {
            OtDeluxeRoutePrompt::SubletteCutoff => match decision {
                OtDeluxeRouteDecision::StayOnTrail => handled = true,
                OtDeluxeRouteDecision::SubletteCutoff => {
                    self.ot_deluxe.route.variant = match self.ot_deluxe.route.variant {
                        OtDeluxeTrailVariant::DallesShortcut => {
                            OtDeluxeTrailVariant::SubletteAndDallesShortcut
                        }
                        _ => OtDeluxeTrailVariant::SubletteCutoff,
                    };
                    handled = true;
                }
                _ => {}
            },
            OtDeluxeRoutePrompt::DallesShortcut => match decision {
                OtDeluxeRouteDecision::StayOnTrail => handled = true,
                OtDeluxeRouteDecision::DallesShortcut => {
                    self.ot_deluxe.route.variant = match self.ot_deluxe.route.variant {
                        OtDeluxeTrailVariant::SubletteCutoff => {
                            OtDeluxeTrailVariant::SubletteAndDallesShortcut
                        }
                        _ => OtDeluxeTrailVariant::DallesShortcut,
                    };
                    handled = true;
                }
                _ => {}
            },
            OtDeluxeRoutePrompt::DallesFinal => match decision {
                OtDeluxeRouteDecision::RaftColumbia => {
                    self.ot_deluxe.route.dalles_choice = Some(OtDeluxeDallesChoice::Raft);
                    handled = true;
                }
                OtDeluxeRouteDecision::BarlowRoad => {
                    self.ot_deluxe.route.dalles_choice = Some(OtDeluxeDallesChoice::Barlow);
                    handled = true;
                }
                _ => {}
            },
        }

        if handled {
            self.ot_deluxe.route.pending_prompt = None;
            self.ot_deluxe.travel.wagon_state = OtDeluxeWagonState::Moving;
            self.sync_otdeluxe_trail_distance();
            let policy = default_otdeluxe_policy();
            self.ot_deluxe.route.current_node_index = otdeluxe_trail::node_index_for_miles(
                &policy.trail,
                self.ot_deluxe.route.variant,
                self.ot_deluxe.miles_traveled,
            );
        }
        handled
    }
}
