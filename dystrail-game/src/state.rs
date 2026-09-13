use rand::rngs::SmallRng;
use rand::{Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::cell::RefMut;
use std::collections::{HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

use crate::camp::CampState;
#[cfg(debug_assertions)]
use crate::constants::DEBUG_ENV_VAR;
#[cfg(test)]
use crate::constants::{AGGRESSIVE_STOP_WINDOW_DAYS, FLOAT_EPSILON, TRAVEL_PARTIAL_RATIO};
use crate::constants::{
    ALLY_ATTRITION_CHANCE, BEHIND_SCHEDULE_MILES_PER_DAY, BOSS_COMPOSE_FUNDS_COST,
    BOSS_COMPOSE_SUPPLY_COST, CLASSIC_BALANCED_FAILURE_GUARD_MILES,
    CLASSIC_FIELD_REPAIR_COST_CENTS, CLASSIC_FIELD_REPAIR_WEAR_REDUCTION, CROSSING_MILESTONES,
    DEEP_AGGRESSIVE_BOOSTS, DEEP_AGGRESSIVE_BOSS_BIAS_MILES, DEEP_AGGRESSIVE_SANITY_COST,
    DEEP_AGGRESSIVE_SANITY_DAY, DEEP_AGGRESSIVE_SANITY_MILES, DEEP_AGGRESSIVE_TOLERANCE_THRESHOLDS,
    DEEP_BALANCED_FAILSAFE_DISTANCE, DEEP_BALANCED_TOLERANCE_THRESHOLDS, DEEP_CONSERVATIVE_BOOSTS,
    DEEP_EMERGENCY_REPAIR_THRESHOLD, DISEASE_COOLDOWN_DAYS, DISEASE_DAILY_CHANCE,
    DISEASE_DURATION_RANGE, DISEASE_HP_PENALTY, DISEASE_LOW_HP_BONUS, DISEASE_MAX_DAILY_CHANCE,
    DISEASE_SANITY_PENALTY, DISEASE_STARVATION_BONUS, DISEASE_SUPPLIES_BONUS,
    DISEASE_SUPPLY_PENALTY, DISEASE_TICK_HP_LOSS, DISEASE_TICK_SANITY_LOSS,
    EMERGENCY_LIMP_MILE_WINDOW, EMERGENCY_LIMP_REPAIR_COST_CENTS, EMERGENCY_LIMP_WEAR_REDUCTION,
    EMERGENCY_REPAIR_COST, ENCOUNTER_BASE_DEFAULT, ENCOUNTER_CRITICAL_VEHICLE_BONUS,
    ENCOUNTER_EXTENDED_MEMORY_DAYS, ENCOUNTER_HISTORY_WINDOW, ENCOUNTER_RECENT_MEMORY,
    ENCOUNTER_REPEAT_WINDOW_DAYS, ENCOUNTER_REROLL_PENALTY, ENCOUNTER_SOFT_CAP_FACTOR,
    ENCOUNTER_SOFT_CAP_THRESHOLD, EXEC_BREAKDOWN_BONUS_CLAMP_MAX, EXEC_ORDER_DAILY_CHANCE,
    EXEC_ORDER_MAX_COOLDOWN, EXEC_ORDER_MAX_DURATION, EXEC_ORDER_MIN_COOLDOWN,
    EXEC_ORDER_MIN_DURATION, EXEC_TRAVEL_MULTIPLIER_CLAMP_MIN, ILLNESS_TRAVEL_PENALTY,
    LOG_ALLIES_GONE, LOG_ALLY_LOST, LOG_BOSS_COMPOSE, LOG_BOSS_COMPOSE_FUNDS,
    LOG_BOSS_COMPOSE_SUPPLIES, LOG_CROSSING_DECISION_BRIBE, LOG_CROSSING_DECISION_PERMIT,
    LOG_CROSSING_DETOUR, LOG_CROSSING_FAILURE, LOG_CROSSING_PASSED,
    LOG_DEEP_AGGRESSIVE_FIELD_REPAIR, LOG_DISEASE_HIT, LOG_DISEASE_RECOVER, LOG_DISEASE_TICK,
    LOG_EMERGENCY_REPAIR_FORCED, LOG_ENCOUNTER_ROTATION, LOG_EXEC_END_PREFIX,
    LOG_EXEC_START_PREFIX, LOG_HEALTH_COLLAPSE, LOG_REST_REQUESTED_ENCOUNTER, LOG_SANITY_COLLAPSE,
    LOG_STARVATION_BACKSTOP, LOG_STARVATION_RELIEF, LOG_STARVATION_TICK, LOG_TRAVEL_BLOCKED,
    LOG_TRAVEL_BONUS, LOG_TRAVEL_DELAY_CREDIT, LOG_TRAVEL_PARTIAL, LOG_TRAVEL_REST_CREDIT,
    LOG_TRAVELED, LOG_VEHICLE_EMERGENCY_LIMP, LOG_VEHICLE_FAILURE, LOG_VEHICLE_FIELD_REPAIR_GUARD,
    LOG_VEHICLE_REPAIR_EMERGENCY, LOG_VEHICLE_REPAIR_SPARE, MAX_ENCOUNTERS_PER_DAY,
    PROBABILITY_FLOOR, PROBABILITY_MAX, ROTATION_FORCE_INTERVAL, SANITY_POINT_REWARD,
    STARVATION_BASE_HP_LOSS, STARVATION_GRACE_DAYS, STARVATION_MAX_STACK, STARVATION_SANITY_LOSS,
    TRAVEL_CLASSIC_PENALTY_FLOOR, TRAVEL_CONFIG_MIN_MULTIPLIER, TRAVEL_HISTORY_WINDOW,
    TRAVEL_PARTIAL_CLAMP_HIGH, TRAVEL_PARTIAL_CLAMP_LOW, TRAVEL_PARTIAL_DEFAULT_WEAR,
    TRAVEL_PARTIAL_RECOVERY_RATIO, TRAVEL_RATIO_DEFAULT, TRAVEL_V2_PENALTY_FLOOR,
    VEHICLE_BASE_TOLERANCE_CLASSIC, VEHICLE_BASE_TOLERANCE_DEEP, VEHICLE_BREAKDOWN_DAMAGE,
    VEHICLE_BREAKDOWN_PARTIAL_FACTOR, VEHICLE_BREAKDOWN_WEAR, VEHICLE_BREAKDOWN_WEAR_CLASSIC,
    VEHICLE_CRITICAL_SPEED_FACTOR, VEHICLE_CRITICAL_THRESHOLD,
    VEHICLE_DEEP_EMERGENCY_HEAL_AGGRESSIVE, VEHICLE_DEEP_EMERGENCY_HEAL_BALANCED,
    VEHICLE_EMERGENCY_HEAL, VEHICLE_EXEC_MULTIPLIER_DECAY, VEHICLE_EXEC_MULTIPLIER_FLOOR,
    VEHICLE_HEALTH_MAX, VEHICLE_JURY_RIG_HEAL, VEHICLE_MALNUTRITION_MIN_FACTOR,
    VEHICLE_MALNUTRITION_PENALTY_PER_STACK, VEHICLE_SPARE_GUARD_SCALE, WEATHER_COLD_SNAP_SPEED,
    WEATHER_DEFAULT_SPEED, WEATHER_HEAT_WAVE_SPEED, WEATHER_PACE_MULTIPLIER_FLOOR,
    WEATHER_STORM_SMOKE_SPEED,
};
use crate::crossings::{self, CrossingConfig, CrossingContext, CrossingKind};
use crate::data::{Encounter, EncounterData};
use crate::day_accounting::{self, DayLedgerMetrics};
use crate::encounters::{EncounterRequest, encounter_matches_context, pick_encounter};
use crate::endgame::{self, EndgameState, EndgameTravelCfg};
use crate::exec_orders::ExecOrder;
use crate::journey::{
    BreakdownConfig, CountingRng, CrossingPolicy, DayRecord, DayTag, EventDecisionTrace,
    JourneyCfg, MechanicalPolicyId, RngBundle, TravelConfig, TravelDayKind, WearConfig,
};
use crate::numbers::clamp_f64_to_f32;
use crate::personas::{Persona, PersonaMods};
use crate::vehicle::{Breakdown, Part, PartWeights, Vehicle, weighted_pick};
use crate::weather::{Weather, WeatherConfig, WeatherState};

#[cfg(test)]
mod illness_protection_tests;

#[cfg(test)]
mod crossing_arrival_tests;

const ENCOUNTER_UNIQUE_WINDOW: u32 = 20;
const ENCOUNTER_UNIQUE_RATIO_FLOOR: f32 = 0.075;
const ENCOUNTER_EARLY_MIN_DRIVING_MINUTES: u32 = 90;
const ENCOUNTER_FIRST_MAX_DRIVING_MINUTES: u32 = 120;
const ENCOUNTER_SECOND_MAX_DRIVING_MINUTES: u32 = 180;
const ENCOUNTER_MIN_DRIVING_MINUTES: u32 = 120;
const ENCOUNTER_MAX_DRIVING_MINUTES: u32 = 8 * 60;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Choice, Effects, Encounter};
    use crate::journey::{CountingRng, RngBundle};
    use crate::pacing::{PaceCfg, PacingLimits};
    use crate::weather::Weather;
    use rand::Rng;
    use std::cell::RefMut;
    use std::collections::{HashMap, VecDeque};
    use std::rc::Rc;

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

    fn events_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::events)
    }

    fn health_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::health)
    }

    fn breakdown_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::breakdown)
    }

    fn approx_eq(a: f32, b: f32) {
        let epsilon = 1e-5_f32;
        assert!(
            (a - b).abs() <= epsilon,
            "values differ: {a} vs {b} (ε={epsilon})"
        );
    }

    fn clear_road_state(pace: PaceId) -> GameState {
        let mut state = GameState::default();
        state.continuity.route_services.route_id = Some("uninterrupted-test-road".into());
        state.start_of_day();
        state.pace = pace;
        state.weather_state.today = Weather::Clear;
        state.weather_travel_multiplier = 1.0;
        state.exec_travel_multiplier = 1.0;
        state.exec_breakdown_bonus = 0.0;
        state.illness_travel_penalty = 1.0;
        state.stats.hp = 10;
        state.stats.sanity = 10;
        state.stats.supplies = 20;
        state.journey_breakdown.base = 0.0;
        state
    }

    fn final_approach_state(pace: PaceId, route_id: &str, road_miles_left: f32) -> GameState {
        let mut state = clear_road_state(pace);
        state.continuity.route_services.route_id = Some(route_id.into());
        state.trail_distance = 2400.0;
        state.miles_traveled_actual =
            state.trail_distance - crate::route::simulation_distance(&state, road_miles_left);
        state.miles_traveled = state.miles_traveled_actual;
        state.prev_miles_traveled = state.miles_traveled_actual;
        state.crossings_completed = u32::try_from(CROSSING_MILESTONES.len()).unwrap();
        state.sync_route_location();
        state
    }

    #[test]
    fn fractional_route_remainder_uses_only_the_final_driving_minute() {
        let pacing = crate::pacing::PacingConfig::default_config();
        for route in crate::route::routes() {
            let marker_gap = route.total_miles - f32::from(route.stops.last().unwrap().mile);
            let remaining = if marker_gap > 0.0 {
                marker_gap / 2.0
            } else {
                0.125
            };
            for pace in [PaceId::Steady, PaceId::Heated, PaceId::Blitz] {
                let mut state = final_approach_state(pace, &route.id, remaining);
                let before = state.clone();
                state.apply_pace_and_diet(&pacing);
                assert_eq!(state.leg_minutes, 1, "{} {pace:?}", route.id);
                assert!(
                    (state.distance_today - (state.trail_distance - before.miles_traveled_actual))
                        .abs()
                        < 0.001
                );
                let (ended, message, breakdown) = state.travel_next_leg(&endgame_cfg());
                assert!(!ended);
                assert!(!breakdown);
                assert_eq!(message, LOG_TRAVELED);
                assert_eq!(state.day, before.day);
                assert_eq!(
                    state.continuity.clock_minutes,
                    before.continuity.clock_minutes + 1
                );
                assert_eq!(state.continuity.driving_minutes_total, 1);
                assert_eq!(
                    state.miles_traveled_actual.to_bits(),
                    state.trail_distance.to_bits()
                );
                assert!(state.boss.readiness.ready);
                assert!(state.boss.readiness.reached);
                assert!(
                    (crate::route::physical_miles(&state)
                        - crate::route::physical_miles(&before)
                        - remaining)
                        .abs()
                        < 0.001
                );
                assert!(
                    (state.ledger.current_day_miles
                        - (state.miles_traveled_actual - before.miles_traveled_actual))
                        .abs()
                        < 0.001
                );
            }
        }
    }

    #[test]
    fn exact_endpoint_arrival_charges_only_its_driving_time_and_cannot_repeat() {
        let pacing = crate::pacing::PacingConfig::default_config();
        for (pace, mph) in [
            (PaceId::Steady, 60.0),
            (PaceId::Heated, 70.0),
            (PaceId::Blitz, 80.0),
        ] {
            for minutes in [30_u16, 60] {
                let remaining = mph * f32::from(minutes) / 60.0;
                let mut state = final_approach_state(pace, "uninterrupted-test-road", remaining);
                let before = state.clone();
                state.apply_pace_and_diet(&pacing);
                assert_eq!(state.leg_minutes, minutes);
                approx_eq(state.distance_today, remaining);
                let (ended, message, breakdown) = state.travel_next_leg(&endgame_cfg());
                assert!(!ended);
                assert!(!breakdown);
                assert_eq!(message, LOG_TRAVELED);
                assert_eq!(state.day, before.day);
                assert_eq!(
                    state.continuity.clock_minutes,
                    before.continuity.clock_minutes + minutes
                );
                assert_eq!(state.continuity.driving_minutes_total, u32::from(minutes));
                assert_eq!(
                    state.miles_traveled_actual.to_bits(),
                    state.trail_distance.to_bits()
                );
                approx_eq(state.ledger.current_day_miles, remaining);
                assert!(state.boss.readiness.ready);
                assert!(!state.boss.outcome.attempted);

                let arrived = serde_json::to_value(&state).unwrap();
                let repeated = state.travel_next_leg(&endgame_cfg());
                assert_eq!(repeated, (false, "log.boss.await".into(), false));
                assert_eq!(serde_json::to_value(&state).unwrap(), arrived);
                state.apply_pace_and_diet(&pacing);
                assert_eq!(state.leg_minutes, 0);
                approx_eq(state.distance_today, 0.0);
            }
        }
    }

    #[test]
    fn pace_fatigue_charges_the_final_minute_once_and_no_more_after_arrival() {
        let pacing = crate::pacing::PacingConfig::default_config();
        for (pace, remainder, expected_sanity, expected_remainder) in [
            (PaceId::Steady, 299, 8, 299),
            (PaceId::Heated, 299, 7, 0),
            (PaceId::Blitz, 298, 7, 0),
        ] {
            let mut state = final_approach_state(pace, "uninterrupted-test-road", 0.125);
            state.stats.sanity = 8;
            state.continuity.pace_fatigue_remainder = remainder;
            let before = state.clone();
            state.apply_pace_and_diet(&pacing);
            assert_eq!(state.leg_minutes, 1);
            let (ended, _, breakdown) = state.travel_next_leg(&endgame_cfg());
            assert!(!ended && !breakdown);
            assert_eq!(state.stats.sanity, expected_sanity);
            assert_eq!(state.continuity.pace_fatigue_remainder, expected_remainder);
            assert_eq!(state.continuity.driving_minutes_total, 1);
            assert_eq!(
                state.continuity.clock_minutes,
                before.continuity.clock_minutes + 1
            );
            assert_eq!(
                state.continuity.weather_impact,
                before.continuity.weather_impact
            );
            assert!(state.boss.readiness.ready);
            approx_eq(
                state.miles_traveled_actual - before.miles_traveled_actual,
                0.125,
            );
            let arrived = serde_json::to_value(&state).unwrap();
            assert_eq!(
                state.travel_next_leg(&endgame_cfg()),
                (false, "log.boss.await".into(), false)
            );
            assert_eq!(serde_json::to_value(&state).unwrap(), arrived);
        }
    }

    #[test]
    fn final_approach_still_respects_remaining_daylight() {
        let pacing = crate::pacing::PacingConfig::default_config();
        for (pace, mph) in [
            (PaceId::Steady, 60.0),
            (PaceId::Heated, 70.0),
            (PaceId::Blitz, 80.0),
        ] {
            let mut state = final_approach_state(pace, "uninterrupted-test-road", mph / 2.0);
            state.continuity.clock_minutes = crate::travel_time::TRAVEL_DAY_END - 10;
            let before = state.clone();
            state.apply_pace_and_diet(&pacing);
            assert_eq!(state.leg_minutes, 10);
            approx_eq(state.distance_today, mph / 6.0);
            let (ended, message, breakdown) = state.travel_next_leg(&endgame_cfg());
            assert!(!ended);
            assert!(!breakdown);
            assert_eq!(message, LOG_TRAVELED);
            assert_eq!(state.day, before.day + 1);
            assert_eq!(
                state.continuity.clock_minutes,
                crate::travel_time::TRAVEL_DAY_START
            );
            assert_eq!(state.continuity.driving_minutes_total, 10);
            assert!(
                (state.miles_traveled_actual - before.miles_traveled_actual - mph / 6.0).abs()
                    < 0.001
            );
            assert_eq!(state.day_records.len(), 1);
            approx_eq(state.day_records[0].miles, mph / 6.0);
            assert!(!state.boss.readiness.ready);
            assert!(!state.boss.readiness.reached);
        }
    }

    fn ride_encounter(ratio: f32) -> Encounter {
        Encounter {
            id: "test_ride".into(),
            name: "A ride with the convoy".into(),
            desc: "Join the convoy for part of the next half hour.".into(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: "Ride along".into(),
                effects: Effects {
                    travel_bonus_ratio: ratio,
                    ..Effects::default()
                },
            }],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }
    }

    #[test]
    fn encounter_rides_preserve_earlier_driving_within_the_choice_time() {
        for (pace, mph) in [
            (PaceId::Steady, 60.0),
            (PaceId::Heated, 70.0),
            (PaceId::Blitz, 80.0),
        ] {
            let mut state = clear_road_state(pace);
            state.record_travel_day(TravelDayKind::Travel, mph, "travel");
            state.spend_driving_time(60);
            state.current_encounter = Some(ride_encounter(0.5));
            let before = state.clone();
            state.apply_choice(0);
            approx_eq(state.miles_traveled_actual, mph * 1.5);
            approx_eq(state.ledger.current_day_miles, mph * 1.5);
            assert_eq!(state.continuity.driving_minutes_total, 90);
            assert_eq!(
                state.continuity.clock_minutes,
                before.continuity.clock_minutes + 30
            );
            state.advance_clock(&before, 30);
            assert_eq!(
                state.continuity.clock_minutes,
                before.continuity.clock_minutes + 30
            );
            assert_eq!(state.continuity.driving_minutes_total, 90);
            state.apply_choice(0);
            approx_eq(state.miles_traveled_actual, mph * 1.5);
        }
    }

    #[test]
    fn encounter_rides_respect_current_speed_and_remaining_daylight() {
        use crate::travel_time::{TRAVEL_DAY_END, TRAVEL_DAY_START};

        for (clock, multiplier, ratio, minutes, miles, final_clock) in [
            (TRAVEL_DAY_START, 0.5, 0.5, 30, 15.0, TRAVEL_DAY_START + 30),
            (TRAVEL_DAY_START, 3.0, 0.5, 30, 30.0, TRAVEL_DAY_START + 30),
            (TRAVEL_DAY_START, 1.0, 0.25, 15, 15.0, TRAVEL_DAY_START + 30),
            (
                TRAVEL_DAY_END - 10,
                1.0,
                2.0,
                10,
                10.0,
                TRAVEL_DAY_START + 20,
            ),
            (TRAVEL_DAY_END, 1.0, 0.5, 0, 0.0, TRAVEL_DAY_START + 30),
            (3 * 60, 1.0, 0.5, 0, 0.0, TRAVEL_DAY_START + 30),
            (
                TRAVEL_DAY_START,
                1.0,
                f32::NAN,
                0,
                0.0,
                TRAVEL_DAY_START + 30,
            ),
        ] {
            let mut state = clear_road_state(PaceId::Steady);
            state.continuity.clock_minutes = clock;
            state.exec_travel_multiplier = multiplier;
            state.current_encounter = Some(ride_encounter(ratio));
            let before = state.clone();
            state.apply_choice(0);
            approx_eq(state.miles_traveled_actual, miles);
            assert_eq!(state.continuity.driving_minutes_total, minutes);
            state.advance_clock(&before, 30);
            assert_eq!(state.continuity.clock_minutes, final_clock);
        }
    }

    #[test]
    fn encounter_rides_cannot_bypass_a_breakdown() {
        let mut state = clear_road_state(PaceId::Steady);
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.current_encounter = Some(ride_encounter(0.5));
        state.apply_choice(0);
        approx_eq(state.miles_traveled_actual, 0.0);
        assert_eq!(state.continuity.driving_minutes_total, 0);
        assert!(state.breakdown.is_some());
    }

    #[test]
    fn deep_aggressive_readiness_unlocks_on_progress_without_time_or_rng() {
        for kind in [TravelProgressKind::Full, TravelProgressKind::Partial] {
            let mut state = clear_road_state(PaceId::Heated);
            state.mode = GameMode::Deep;
            state.policy = Some(PolicyKind::Aggressive);
            state.miles_traveled_actual = DEEP_AGGRESSIVE_BOSS_BIAS_MILES - 1.0;
            state.miles_traveled = state.miles_traveled_actual;
            let bundle = Rc::new(RngBundle::from_user_seed(42));
            state.attach_rng_bundle(Rc::clone(&bundle));
            let rng_before = serde_json::to_value(bundle.as_ref()).unwrap();
            let day = state.day;
            let minute = state.continuity.clock_minutes;

            approx_eq(state.apply_travel_progress(0.5, kind), 0.5);
            assert!(!state.boss.readiness.ready);
            assert!(!state.boss.readiness.reached);
            let mut state: GameState =
                serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            approx_eq(state.apply_travel_progress(0.5, kind), 0.5);

            assert!(state.boss.readiness.ready);
            assert!(state.boss.readiness.reached);
            assert!(!state.boss.outcome.attempted);
            assert!(!state.day_state.rest.rest_requested);
            assert_eq!(state.day, day);
            assert_eq!(state.continuity.clock_minutes, minute);
            assert_eq!(state.policy, Some(PolicyKind::Aggressive));
            approx_eq(state.miles_traveled_actual, DEEP_AGGRESSIVE_BOSS_BIAS_MILES);
            assert_eq!(
                serde_json::to_value(state.rng_bundle.as_deref().unwrap()).unwrap(),
                rng_before
            );
        }
    }

    #[test]
    fn early_readiness_preserves_mode_policy_and_terminal_guards() {
        for (mode, policy, ended, attempted) in [
            (GameMode::Classic, PolicyKind::Aggressive, false, false),
            (GameMode::Deep, PolicyKind::Balanced, false, false),
            (GameMode::Deep, PolicyKind::Conservative, false, false),
            (GameMode::Deep, PolicyKind::ResourceManager, false, false),
            (GameMode::Deep, PolicyKind::Aggressive, true, false),
            (GameMode::Deep, PolicyKind::Aggressive, false, true),
        ] {
            let mut state = clear_road_state(PaceId::Heated);
            state.mode = mode;
            state.policy = Some(policy);
            state.ending = ended.then_some(Ending::SanityLoss);
            state.boss.outcome.attempted = attempted;
            state.miles_traveled_actual = DEEP_AGGRESSIVE_BOSS_BIAS_MILES - 1.0;
            approx_eq(
                state.apply_travel_progress(1.0, TravelProgressKind::Full),
                1.0,
            );
            assert!(!state.boss.readiness.ready);
            assert!(!state.boss.readiness.reached);
            assert_eq!(state.policy, Some(policy));
        }
    }

    #[test]
    fn stationary_work_and_camp_cannot_unlock_early_readiness() {
        let mut state = clear_road_state(PaceId::Heated);
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.miles_traveled_actual = DEEP_AGGRESSIVE_BOSS_BIAS_MILES;
        state.miles_traveled = state.miles_traveled_actual;
        state.prev_miles_traveled = state.miles_traveled_actual;
        state.continuity.route_services.stop = Some(42);
        state.continuity.clock_minutes = crate::travel_time::TRAVEL_DAY_END - 60;
        let day = state.day;

        approx_eq(
            state.apply_travel_progress(0.0, TravelProgressKind::Full),
            0.0,
        );
        assert!(state.perform_activity(crate::activities::Activity::WorkCash));
        assert_eq!(state.day, day + 1);
        assert!(!state.boss.readiness.ready);
        assert!(!state.boss.readiness.reached);
        assert!(
            crate::camp::camp_rest(&mut state, &crate::camp::CampConfig::default_config()).rested
        );
        assert_eq!(state.day, day + 2);
        assert!(!state.boss.readiness.ready);
        assert!(!state.boss.readiness.reached);
        approx_eq(state.miles_traveled_actual, DEEP_AGGRESSIVE_BOSS_BIAS_MILES);
        assert!(state.day_records.iter().all(|record| record.miles == 0.0));
    }

    fn shipped_encounter(id: &str) -> Encounter {
        EncounterData::from_json(include_str!(
            "../../dystrail-web/static/assets/data/game.json"
        ))
        .unwrap()
        .encounters
        .into_iter()
        .find(|encounter| encounter.id == id)
        .unwrap()
    }

    #[test]
    fn clinic_choices_spend_only_the_short_visit_and_preserve_earlier_driving() {
        for (choice, supplies_cost, hp_gain, credibility_gain) in [(1, 1, 1, 0), (2, 2, 0, 2)] {
            let mut state = clear_road_state(PaceId::Steady);
            state.stats.hp = 8;
            state.stats.credibility = 0;
            state.record_travel_day(TravelDayKind::Travel, 60.0, "");
            state.spend_driving_time(60);
            let bundle = Rc::new(RngBundle::from_user_seed(42));
            state.attach_rng_bundle(Rc::clone(&bundle));
            state.current_encounter = Some(shipped_encounter("clinic_triage"));
            let before = state.clone();
            let rng_before = serde_json::to_value(bundle.as_ref()).unwrap();

            state.apply_choice(choice);
            state.advance_clock(&before, 30);

            assert_eq!(state.day, before.day);
            assert_eq!(
                state.continuity.clock_minutes,
                before.continuity.clock_minutes + 30
            );
            assert!(!state.day_state.rest.rest_requested);
            assert!(state.current_encounter.is_none());
            assert_eq!(state.encounters_resolved, before.encounters_resolved + 1);
            assert_eq!(state.stats.supplies, before.stats.supplies - supplies_cost);
            assert_eq!(state.stats.hp, before.stats.hp + hp_gain);
            assert_eq!(state.stats.credibility, credibility_gain);
            assert_eq!(state.days_with_camp, 0);
            assert_eq!(state.continuity.driving_minutes_total, 60);
            approx_eq(state.miles_traveled_actual, 60.0);
            approx_eq(state.ledger.current_day_miles, 60.0);
            assert_eq!(serde_json::to_value(bundle.as_ref()).unwrap(), rng_before);
        }
    }

    #[test]
    fn overnight_briefing_choices_still_request_a_full_rest() {
        for choice in [1, 2] {
            let mut state = clear_road_state(PaceId::Steady);
            state.current_encounter = Some(shipped_encounter("overnight_briefing"));
            let before = state.clone();
            state.apply_choice(choice);
            state.advance_clock(&before, 30);
            assert!(state.day_state.rest.rest_requested);
            assert!(
                crate::camp::camp_rest(&mut state, &crate::camp::CampConfig::default_config())
                    .rested
            );
            assert_eq!(state.day, before.day + 1);
            assert_eq!(state.days_with_camp, 1);
            assert!(!state.day_state.rest.rest_requested);
            approx_eq(state.miles_traveled_actual, 0.0);
        }
    }

    #[test]
    fn beginning_a_day_while_parked_does_not_wear_the_vehicle() {
        for travel_v2 in [false, true] {
            let mut state = GameState::default();
            state.features.travel_v2 = travel_v2;
            state.leg_minutes = 60;
            state.vehicle.wear = 8.0;
            state.start_of_day();
            state.start_of_day();
            approx_eq(state.vehicle.wear, 8.0);
            approx_eq(state.miles_traveled_actual, 0.0);
            assert_eq!(state.continuity.driving_minutes_total, 0);
        }
    }

    #[test]
    fn crossing_pass_and_detour_spend_time_without_driving_wear() {
        for detour in [false, true] {
            let mut state = clear_road_state(PaceId::Steady);
            state.miles_traveled_actual = CROSSING_MILESTONES[0];
            state.prev_miles_traveled = state.miles_traveled_actual;
            state.miles_traveled = state.miles_traveled_actual;
            state.distance_today = 60.0;
            state.distance_today_raw = 60.0;
            state.leg_minutes = 60;
            state.vehicle.wear = 8.0;
            state.budget_cents = 0;
            state.encounter_chance_today = 0.0;
            state.journey_crossing.pass = if detour { 0.0 } else { 1.0 };
            state.journey_crossing.detour = if detour { 1.0 } else { 0.0 };
            state.journey_crossing.terminal = 0.0;
            state.journey_crossing.detour_hours.min = 1;
            state.journey_crossing.detour_hours.max = 1;
            let (ended, message, breakdown) = state.travel_next_leg(&endgame_cfg());
            assert!(!ended);
            assert!(!breakdown);
            assert_eq!(
                message,
                if detour {
                    LOG_CROSSING_DETOUR
                } else {
                    LOG_CROSSING_PASSED
                }
            );
            approx_eq(state.vehicle.wear, 8.0);
            approx_eq(state.miles_traveled_actual, CROSSING_MILESTONES[0]);
            assert_eq!(state.continuity.driving_minutes_total, 0);
            assert_eq!(
                state.crossing_events.last().unwrap().detour_reason,
                detour.then_some(CrossingDetourReason::RouteDiversion)
            );
            assert_eq!(
                state.continuity.clock_minutes,
                crate::journal::morning() + if detour { 60 } else { 30 }
            );
        }
    }

    fn crossing_state(mode: GameMode, seed: u64, completed: usize) -> (GameState, Rc<RngBundle>) {
        let mut state = clear_road_state(PaceId::Steady);
        state.mode = mode;
        state.seed = seed;
        let bundle = Rc::new(RngBundle::from_user_seed(seed));
        state.attach_rng_bundle(Rc::clone(&bundle));
        state.crossings_completed = u32::try_from(completed).unwrap();
        state.miles_traveled_actual = CROSSING_MILESTONES[completed] - 60.0;
        state.miles_traveled = state.miles_traveled_actual;
        state.prev_miles_traveled = state.miles_traveled_actual;
        state.record_travel_day(TravelDayKind::Travel, 60.0, "travel");
        state.spend_driving_time(60);
        state.vehicle.wear = 8.0;
        state.budget_cents = 2_000;
        state.budget = 20;
        state.inventory.tags.clear();
        state.receipts.clear();
        state.stats.credibility = 6;
        state.journey_crossing.pass = 0.0;
        state.journey_crossing.detour = 0.0;
        state.journey_crossing.terminal = 1.0;
        state.journey_crossing.bribe = crate::journey::BribePolicy::default();
        state.journey_crossing.detour_hours.min = 1;
        state.journey_crossing.detour_hours.max = 2;
        (state, bundle)
    }

    #[test]
    fn first_checkpoint_denials_are_clocked_detours_with_honest_bribe_records() {
        for mode in [GameMode::Classic, GameMode::Deep] {
            for seed in [0, 1, 4242, u64::MAX] {
                for attempted in [false, true] {
                    let (mut state, bundle) = crossing_state(mode, seed, 0);
                    if !attempted {
                        state.budget_cents = 0;
                        state.budget = 0;
                    }
                    let before = state.clone();
                    let (ended, message) = state.handle_crossing_event(60.0).unwrap();
                    assert!(!ended);
                    assert!(state.ending.is_none());
                    assert_eq!(message, "log.crossing.denied");
                    assert_eq!(state.crossings_completed, 1);
                    assert_eq!(state.crossing_detours_taken, 1);
                    assert_eq!(state.crossing_failures, 0);
                    assert_eq!(state.crossing_bribe_attempts, u32::from(attempted));
                    assert_eq!(state.crossing_bribe_successes, 0);
                    let cost = if attempted { 1_000 } else { 0 };
                    assert_eq!(state.budget_cents, before.budget_cents - cost);
                    assert_eq!(state.bribes_spent_cents, cost);
                    assert_eq!(
                        state.continuity.clock_minutes,
                        before.continuity.clock_minutes + 120
                    );
                    assert_eq!(state.continuity.driving_minutes_total, 60);
                    approx_eq(state.miles_traveled_actual, before.miles_traveled_actual);
                    approx_eq(state.ledger.current_day_miles, 60.0);
                    approx_eq(state.vehicle.wear, before.vehicle.wear);
                    assert_eq!(bundle.crossing().draws(), 1);
                    let event = state.crossing_events.last().unwrap();
                    assert_eq!(
                        event.detour_reason,
                        Some(CrossingDetourReason::CheckpointDenied)
                    );
                    assert_eq!(event.outcome, CrossingOutcomeTelemetry::Detoured);
                    assert!(event.detour_taken);
                    assert_eq!(event.detour_hours, Some(2));
                    assert_eq!(event.bribe_attempted, attempted);
                    assert_eq!(event.bribe_success, attempted.then_some(false));
                    assert_eq!(event.bribe_cost_cents, cost);
                    assert!(!event.permit_used);
                    assert!(
                        state
                            .ledger
                            .current_day_reason_tags
                            .iter()
                            .any(|tag| tag == "crossing_denied")
                    );
                    assert!(
                        state
                            .ledger
                            .current_day_reason_tags
                            .iter()
                            .any(|tag| tag == "detour")
                    );
                    assert_eq!(
                        state
                            .logs
                            .iter()
                            .any(|log| log == "crossing.result.bribe.fail"),
                        attempted
                    );
                    assert!(!state.logs.iter().any(|log| log == LOG_CROSSING_FAILURE));
                    let restored: GameState =
                        serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
                    assert_eq!(
                        restored.crossing_events.last().unwrap().detour_reason,
                        Some(CrossingDetourReason::CheckpointDenied)
                    );
                    assert!(state.handle_crossing_event(60.0).is_none());
                    assert_eq!(bundle.crossing().draws(), 1);
                }
            }
        }
    }

    #[test]
    fn first_checkpoint_permits_and_successful_bribes_remain_passes() {
        for mode in [GameMode::Classic, GameMode::Deep] {
            for permit in [false, true] {
                let (mut state, bundle) = crossing_state(mode, 4242, 0);
                if permit {
                    state.inventory.tags.insert("press_pass".into());
                    state.journey_crossing.permit.disable_terminal = true;
                    state.journey_crossing.permit.eligible = vec!["checkpoint".into()];
                } else {
                    state.journey_crossing.pass = 1.0;
                    state.journey_crossing.terminal = 0.0;
                }
                let before = state.clone();
                let (ended, message) = state.handle_crossing_event(60.0).unwrap();
                assert!(!ended);
                assert_eq!(message, LOG_CROSSING_PASSED);
                let event = state.crossing_events.last().unwrap();
                assert_eq!(event.outcome, CrossingOutcomeTelemetry::Passed);
                assert_eq!(event.detour_reason, None);
                assert!(!event.detour_taken);
                assert_eq!(event.permit_used, permit);
                assert_eq!(event.bribe_attempted, !permit);
                assert_eq!(event.bribe_success, (!permit).then_some(true));
                let cost = if permit { 0 } else { 1_000 };
                assert_eq!(event.bribe_cost_cents, cost);
                assert_eq!(state.budget_cents, before.budget_cents - cost);
                assert_eq!(state.crossing_permit_uses, u32::from(permit));
                assert_eq!(state.crossing_bribe_successes, u32::from(!permit));
                assert_eq!(
                    state.stats.credibility,
                    before.stats.credibility + i32::from(permit)
                );
                assert_eq!(
                    state.continuity.clock_minutes,
                    before.continuity.clock_minutes + 30
                );
                assert_eq!(state.continuity.driving_minutes_total, 60);
                assert_eq!(bundle.crossing().draws(), 1);
            }
        }
    }

    #[test]
    fn later_checkpoint_and_bridge_failures_still_end_the_journey() {
        for mode in [GameMode::Classic, GameMode::Deep] {
            for completed in [1, 2] {
                let (mut state, bundle) = crossing_state(mode, 4242, completed);
                let before = state.clone();
                let (ended, message) = state.handle_crossing_event(60.0).unwrap();
                assert!(ended);
                assert_eq!(message, LOG_CROSSING_FAILURE);
                assert!(matches!(
                    state.ending,
                    Some(Ending::Collapse {
                        cause: CollapseCause::Crossing
                    })
                ));
                assert_eq!(state.crossing_failures, 1);
                assert_eq!(state.crossings_completed, before.crossings_completed);
                assert_eq!(state.crossing_detours_taken, 0);
                assert_eq!(
                    state.continuity.clock_minutes,
                    before.continuity.clock_minutes
                );
                assert_eq!(state.continuity.driving_minutes_total, 60);
                approx_eq(state.miles_traveled_actual, before.miles_traveled_actual);
                approx_eq(state.vehicle.wear, before.vehicle.wear);
                let event = state.crossing_events.last().unwrap();
                assert_eq!(event.detour_reason, None);
                assert_eq!(event.outcome, CrossingOutcomeTelemetry::Failed);
                assert!(!event.detour_taken);
                assert_eq!(event.bribe_success, Some(false));
                assert_eq!(
                    event.bribe_cost_cents,
                    before.budget_cents - state.budget_cents
                );
                assert_eq!(bundle.crossing().draws(), 1);
            }
        }
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
    fn crossing_records_never_invent_a_bribe_attempt() {
        for result in [
            crossings::CrossingResult::Pass,
            crossings::CrossingResult::Detour(1),
            crossings::CrossingResult::TerminalFail,
        ] {
            for attempted in [false, true] {
                let mut state = GameState::default();
                state.start_of_day();
                let resolved = crossings::CrossingOutcome {
                    result,
                    used_permit: false,
                    bribe_attempted: attempted,
                    bribe_succeeded: false,
                };
                let mut telemetry = CrossingTelemetry::new(
                    state.day,
                    state.region,
                    state.season,
                    CrossingKind::Checkpoint,
                );
                telemetry.bribe_attempted = attempted;
                telemetry.bribe_success = attempted.then_some(false);
                let _ = state.process_crossing_result(resolved, telemetry, 10.0);
                let event = state.crossing_events.last().unwrap();
                assert_eq!(event.bribe_attempted, attempted);
                assert_eq!(event.bribe_success, attempted.then_some(false));
            }
        }
    }

    #[test]
    fn travel_wear_scales_with_pace_weather_and_fatigue() {
        let mut state = GameState {
            leg_minutes: 60,
            ..GameState::default()
        };
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
        state.apply_travel_wear();
        let steady_clear = state.vehicle.wear;

        state.vehicle.wear = 0.0;
        state.vehicle.health = Vehicle::default().health;
        state.pace = PaceId::Blitz;
        state.weather_state.today = Weather::Storm;
        state.miles_traveled_actual = 800.0;
        state.apply_travel_wear();
        let blitz_storm = state.vehicle.wear;

        assert!(blitz_storm > steady_clear);
    }

    #[test]
    fn strategy_does_not_secretly_change_the_selected_travel_pace() {
        let mut classic = GameState {
            policy: Some(PolicyKind::Balanced),
            journey_travel: TravelConfig {
                weather_factor: HashMap::from([(Weather::Clear, 1.0), (Weather::Storm, 1.0)]),
            },
            ..GameState::default()
        };

        let pace_cfg = PaceCfg::default();
        let limits = PacingLimits::default();

        let mut control = classic.clone();
        control.policy = Some(PolicyKind::Aggressive);
        let base = control.compute_miles_for_today(&pace_cfg, &limits);
        let nudged = classic.compute_miles_for_today(&pace_cfg, &limits);
        approx_eq(nudged, base);

        let mut deep = classic.clone();
        deep.mode = GameMode::Deep;
        deep.policy = Some(PolicyKind::Balanced);
        let mut deep_control = deep.clone();
        deep_control.policy = Some(PolicyKind::ResourceManager);
        let deep_base = deep_control.compute_miles_for_today(&pace_cfg, &limits);
        let deep_nudged = deep.compute_miles_for_today(&pace_cfg, &limits);
        approx_eq(deep_nudged, deep_base);
    }

    #[test]
    fn deep_aggressive_compose_uses_supplies_then_funds() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            stats: Stats {
                supplies: BOSS_COMPOSE_SUPPLY_COST,
                sanity: 0,
                ..Stats::default()
            },
            budget_cents: BOSS_COMPOSE_FUNDS_COST * 2,
            ..GameState::default()
        };

        let applied_supplies = state.apply_deep_aggressive_compose();
        assert!(applied_supplies, "expected supply-based compose");
        assert_eq!(state.stats.supplies, 0);
        assert_eq!(state.stats.sanity, 1);
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

        let baseline_budget = state.budget_cents;
        state.budget = i32::try_from(state.budget_cents / 100).unwrap_or(0);

        let applied_funds = state.apply_deep_aggressive_compose();
        assert!(applied_funds, "expected funds-based compose");
        assert!(state.budget_cents < baseline_budget);
        assert_eq!(state.stats.sanity, 1);
        assert!(state.logs.iter().any(|log| log == LOG_BOSS_COMPOSE_FUNDS));
        assert!(state.logs.iter().any(|log| log == LOG_BOSS_COMPOSE));
    }

    #[test]
    fn breakdown_uses_part_weights() {
        let mut state = GameState::default();
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.2));
        state.leg_minutes = crate::travel_time::TRAVEL_DAY_MINUTES;
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
    }

    fn endgame_cfg() -> EndgameTravelCfg {
        EndgameTravelCfg::default()
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
        let (_ended, _msg, _started) = state.travel_next_leg(&cfg);

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
        let (_ended_first, msg_first, _started_first) = state.travel_next_leg(&cfg);
        assert_eq!(msg_first, "log.traveled");
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
        let (_ended, _msg, _started) = state.travel_next_leg(&cfg);

        assert!(state.stats.supplies >= 0, "supplies went negative");
        assert!(state.stats.sanity >= 0, "sanity went negative");
    }

    #[test]
    fn camp_cooldowns_tick_once_per_completed_day() {
        let mut state = GameState::default();
        state.camp.rest_cooldown = 3;
        state.start_of_day();
        state.end_of_day();
        assert_eq!(state.camp.rest_cooldown, 2);
        state.end_of_day();
        assert_eq!(state.camp.rest_cooldown, 2);
        state.start_of_day();
        state.end_of_day();
        assert_eq!(state.camp.rest_cooldown, 1);
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
    fn steady_clear_progress_uses_hours_and_the_actual_road_distance() {
        let mut state = GameState::default();
        state.continuity.route_services.route_id = Some(String::from("uninterrupted-road"));
        state.trail_distance = 20_000.0;
        state.crossings_completed = u32::try_from(CROSSING_MILESTONES.len()).unwrap();
        state.detach_rng_bundle();
        let pacing = crate::pacing::PacingConfig::default_config();
        let cfg = endgame_cfg();
        for _ in 0..30 {
            state.start_of_day();
            state.weather_state.today = Weather::Clear;
            state.weather_travel_multiplier = 1.0;
            state.exec_travel_multiplier = 1.0;
            state.illness_travel_penalty = 1.0;
            state.apply_pace_and_diet(&pacing);
            state.encounter_chance_today = 0.0;
            let (ended, _, _) = state.travel_next_leg(&cfg);
            assert!(!ended, "run ended prematurely");
        }
        assert_eq!(state.day, 7);
        assert_eq!(
            state.continuity.clock_minutes,
            crate::travel_time::TRAVEL_DAY_START
        );
        assert!((state.miles_traveled_actual - 1_800.0).abs() < 0.01);
        assert_eq!(state.day_records.len(), 6);
        assert!(
            state
                .day_records
                .iter()
                .all(|record| (record.miles - 300.0).abs() < 0.01)
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
        base_state.apply_pace_and_diet(&cfg);
        let base = base_state.encounter_chance_today;
        assert!((f64::from(base) - f64::from(cfg.limits.encounter_base)).abs() < FLOAT_EPSILON);

        let mut capped_state = GameState {
            encounter_history: VecDeque::from(vec![2, 1, 1, 1, 0, 0, 0, 0, 0]),
            ..GameState::default()
        };
        capped_state.detach_rng_bundle();
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
            ledger: crate::day_accounting::OpenDayLedger {
                current_day_reason_tags: ["camp".into(), "repair".into()].into(),
                ..crate::day_accounting::OpenDayLedger::default()
            },
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            features: FeatureFlags {
                travel_v2: true,
                ..FeatureFlags::default()
            },
            stats: Stats {
                supplies: 5,
                ..Stats::default()
            },
            distance_today: 5.0,
            distance_today_raw: 5.0,
            partial_distance_today: 2.0,
            recent_travel_days: VecDeque::from(vec![
                TravelDayKind::NonTravel;
                TRAVEL_HISTORY_WINDOW
            ]),
            ..GameState::default()
        };
        let parked_miles = state.miles_traveled_actual;
        state.advance_days(1);
        assert_eq!(
            (state.miles_traveled_actual).to_bits(),
            (parked_miles).to_bits()
        );
        state.record_travel_day(TravelDayKind::Partial, 3.0, "misc");
        state.apply_delay_travel_credit("delay_test");

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
        state.day_state.lifecycle.day_initialized = true;
        state.continuity.driving_minutes_total = ENCOUNTER_MAX_DRIVING_MINUTES;
        state.encounter_history.push_back(MAX_ENCOUNTERS_PER_DAY);
        let cfg = crate::pacing::PacingConfig::default_config();
        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        assert!(state.encounter_drought_due());

        let end_cfg = endgame_cfg();
        let (ended, message, _) = state.travel_next_leg(&end_cfg);
        assert!(!ended);
        assert_eq!(message, LOG_TRAVELED);
        assert!(state.current_encounter.is_none());
    }

    #[test]
    fn allows_two_spaced_encounters_before_daily_cap() {
        let mut state = encounter_schedule_state(GameMode::Classic, 99);
        state.encounter_chance_today = 1.0;
        drive_encounter_minutes(&mut state, 90);
        assert!(draw_scheduled_encounter(&mut state).is_some());
        assert_eq!(state.encounters_today, 1);
        state.apply_choice(0);
        assert!(!state.encounters.occurred_today);

        drive_encounter_minutes(&mut state, 90);
        assert!(draw_scheduled_encounter(&mut state).is_some());
        assert_eq!(state.encounters_today, 2);
        state.apply_choice(0);
        assert!(state.encounters.occurred_today);

        assert!(draw_scheduled_encounter(&mut state).is_none());
        assert_eq!(
            state.encounter_history.back(),
            Some(&MAX_ENCOUNTERS_PER_DAY)
        );
    }

    fn encounter_schedule_state(mode: GameMode, seed: u64) -> GameState {
        let data = EncounterData::from_encounters(
            ["alpha", "beta", "gamma", "delta"]
                .into_iter()
                .map(|id| Encounter {
                    id: id.to_owned(),
                    name: id.to_owned(),
                    ..ride_encounter(0.0)
                })
                .collect(),
        );
        let mut state = clear_road_state(PaceId::Steady).with_seed(seed, mode, data);
        state.encounter_chance_today = 0.0;
        state.leg_minutes = 60;
        state
    }

    fn drive_encounter_minutes(state: &mut GameState, minutes: u16) {
        let mut remaining = minutes;
        while remaining > 0 {
            state.prepare_travel_clock();
            state.start_of_day();
            let elapsed = remaining.min(state.travel_minutes_available());
            let miles = crate::route::simulation_distance(state, f32::from(elapsed));
            state.record_travel_day(TravelDayKind::Travel, miles, "travel");
            state.spend_driving_time(elapsed);
            remaining -= elapsed;
        }
    }

    fn draw_scheduled_encounter(state: &mut GameState) -> Option<String> {
        let bundle = state.rng_bundle.as_ref().map(Rc::clone);
        state.process_encounter_flow(bundle.as_ref(), false)?;
        state.current_encounter.as_ref().map(|enc| enc.id.clone())
    }

    #[test]
    fn driving_drought_introduces_two_distinct_early_encounters_in_both_modes() {
        for mode in [GameMode::Classic, GameMode::Deep] {
            for seed in [0, 1, 42, 4242, u64::MAX] {
                let mut state = encounter_schedule_state(mode, seed);
                drive_encounter_minutes(&mut state, 119);
                assert!(draw_scheduled_encounter(&mut state).is_none());
                drive_encounter_minutes(&mut state, 1);
                let first_miles = state.miles_traveled_actual;
                let first = draw_scheduled_encounter(&mut state).expect("first scene by two hours");
                assert_eq!(state.continuity.driving_minutes_total, 120);
                approx_eq(state.miles_traveled_actual, first_miles);
                state.apply_choice(0);

                drive_encounter_minutes(&mut state, 179);
                assert!(draw_scheduled_encounter(&mut state).is_none());
                drive_encounter_minutes(&mut state, 1);
                let second_miles = state.miles_traveled_actual;
                let second = draw_scheduled_encounter(&mut state)
                    .expect("second scene within three more hours");
                assert_ne!(first, second, "mode {mode:?}, seed {seed}");
                assert_eq!(state.continuity.driving_minutes_total, 300);
                approx_eq(state.miles_traveled_actual, second_miles);
                assert_eq!(state.encounters_today, 2);
            }
        }
    }

    #[test]
    fn encounter_minimum_gap_uses_ninety_then_one_hundred_twenty_driving_minutes() {
        let mut state = encounter_schedule_state(GameMode::Classic, 22);
        state.encounter_chance_today = 1.0;
        drive_encounter_minutes(&mut state, 89);
        assert!(draw_scheduled_encounter(&mut state).is_none());
        drive_encounter_minutes(&mut state, 1);
        assert!(draw_scheduled_encounter(&mut state).is_some());
        state.apply_choice(0);

        drive_encounter_minutes(&mut state, 89);
        assert!(draw_scheduled_encounter(&mut state).is_none());
        drive_encounter_minutes(&mut state, 1);
        assert!(draw_scheduled_encounter(&mut state).is_some());
        state.apply_choice(0);

        state.advance_days_with_reason(1, "camp");
        drive_encounter_minutes(&mut state, 119);
        assert!(draw_scheduled_encounter(&mut state).is_none());
        drive_encounter_minutes(&mut state, 1);
        // The new day's empty count cannot shorten the road-time gap.
        assert!(draw_scheduled_encounter(&mut state).is_some());
    }

    #[test]
    fn eight_hour_drought_bypasses_soft_cap_and_defers_when_no_unseen_scene_is_eligible() {
        let mut state = encounter_schedule_state(GameMode::Deep, 23);
        state.record_encounter("alpha");
        state.record_encounter("beta");
        state.encounter_history = VecDeque::from(vec![2, 2, 2]);
        drive_encounter_minutes(&mut state, 479);
        assert!(draw_scheduled_encounter(&mut state).is_none());
        drive_encounter_minutes(&mut state, 1);
        let encounter =
            draw_scheduled_encounter(&mut state).expect("drought ends after eight driving hours");
        assert!(!["alpha", "beta"].contains(&encounter.as_str()));
        assert_eq!(state.continuity.driving_minutes_total, 480);
        state.apply_choice(0);

        let other = if encounter == "gamma" {
            "delta"
        } else {
            "gamma"
        };
        state.record_encounter(other);
        drive_encounter_minutes(&mut state, 480);
        assert!(!state.encounter_drought_due());
        assert!(draw_scheduled_encounter(&mut state).is_none());
    }

    #[test]
    fn work_and_camp_do_not_advance_encounter_exposure() {
        let mut state = encounter_schedule_state(GameMode::Classic, 24);
        drive_encounter_minutes(&mut state, 89);
        let exposure = state.continuity.driving_minutes_total;
        let before_work = state.clone();
        state.advance_clock(&before_work, 180);
        assert_eq!(state.continuity.driving_minutes_total, exposure);
        let before_camp = state.clone();
        state.advance_days_with_reason(1, "camp");
        assert!(state.day > before_camp.day);
        assert_eq!(state.continuity.driving_minutes_total, exposure);
        assert!(!state.encounter_drought_due());
        state.encounter_chance_today = 1.0;
        assert!(draw_scheduled_encounter(&mut state).is_none());
    }

    #[test]
    fn encounter_schedule_and_weighted_choice_survive_save_reload() {
        let mut state = encounter_schedule_state(GameMode::Deep, 25);
        drive_encounter_minutes(&mut state, 120);
        assert!(draw_scheduled_encounter(&mut state).is_some());
        state.apply_choice(0);
        drive_encounter_minutes(&mut state, 179);
        let data = state.data.clone().unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let mut restored: GameState = serde_json::from_str(&json).unwrap();
        restored = restored.rehydrate(data);
        assert_eq!(
            restored.continuity.last_encounter_driving_minutes,
            Some(120)
        );
        assert!(!restored.encounter_drought_due());
        drive_encounter_minutes(&mut state, 1);
        drive_encounter_minutes(&mut restored, 1);
        assert_eq!(
            draw_scheduled_encounter(&mut state),
            draw_scheduled_encounter(&mut restored)
        );
        assert_eq!(
            restored.continuity.last_encounter_driving_minutes,
            Some(300)
        );
        assert_eq!(restored.recent_encounters, state.recent_encounters);
    }

    #[test]
    fn repeated_stops_do_not_award_unearned_miles() {
        let mut state = GameState {
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            recent_travel_days: VecDeque::from(vec![
                TravelDayKind::NonTravel;
                AGGRESSIVE_STOP_WINDOW_DAYS
            ]),
            ..GameState::default()
        };
        state.advance_days(4);
        assert_eq!((state.miles_traveled_actual).to_bits(), (0.0f32).to_bits());
        assert!(
            state
                .day_records
                .iter()
                .all(|r| r.kind == TravelDayKind::NonTravel && r.miles == 0.0)
        );
        assert!(
            !state
                .day_reason_history
                .iter()
                .any(|reason| reason.contains("stop_cap"))
        );
    }

    #[test]
    fn sanity_guard_recovers_without_inventing_distance() {
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

        state.day_state.lifecycle.day_initialized = true;
        state.apply_deep_aggressive_sanity_guard();

        assert!(state.guards.deep_aggressive_sanity_guard_used);
        assert_eq!(state.stats.sanity, SANITY_POINT_REWARD);
        assert_eq!(
            state.ledger.current_day_kind,
            Some(TravelDayKind::NonTravel)
        );
        assert_eq!(
            (state.miles_traveled_actual).to_bits(),
            (DEEP_AGGRESSIVE_SANITY_MILES).to_bits()
        );
        assert!(
            state
                .ledger
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
            state.exec_travel_multiplier = 10.0;
            state.exec_breakdown_bonus = 10.0;
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
    fn rest_and_delay_helpers_cannot_create_or_erase_movement() {
        for earlier_miles in [0.0, 60.0] {
            let mut state = clear_road_state(PaceId::Steady);
            if earlier_miles > 0.0 {
                state.record_travel_day(TravelDayKind::Partial, earlier_miles, "travel");
                state.spend_driving_time(60);
            }
            state.endgame.active = true;
            state.endgame.wear_shave_ratio = 0.5;
            state.vehicle.wear = 8.0;
            let before = state.clone();
            state.apply_rest_travel_credit();
            state.apply_delay_travel_credit("repair");
            approx_eq(state.miles_traveled_actual, earlier_miles);
            approx_eq(state.ledger.current_day_miles, earlier_miles);
            approx_eq(state.vehicle.wear, 8.0);
            assert_eq!(
                state.continuity.clock_minutes,
                before.continuity.clock_minutes
            );
            assert_eq!(
                state.continuity.driving_minutes_total,
                before.continuity.driving_minutes_total
            );
            assert_eq!(
                state.ledger.current_day_kind,
                before
                    .ledger
                    .current_day_kind
                    .or(Some(TravelDayKind::NonTravel))
            );
            assert!(
                state
                    .ledger
                    .current_day_reason_tags
                    .iter()
                    .any(|tag| tag == "camp")
            );
            assert!(
                state
                    .ledger
                    .current_day_reason_tags
                    .iter()
                    .any(|tag| tag == "repair")
            );
        }
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
        state.apply_classic_field_repair_guard();
        approx_eq(state.miles_traveled_actual, 0.0);
        approx_eq(state.ledger.current_day_miles, 0.0);
        assert_eq!(state.continuity.driving_minutes_total, 0);
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

        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.miles_traveled_actual = 1_951.0;
        state.distance_today = 5.0;
        let before_limp = state.miles_traveled_actual;
        let limp_triggered = state.try_emergency_limp_guard();
        assert!(limp_triggered);
        approx_eq(state.miles_traveled_actual, before_limp);

        state.miles_traveled_actual = 1_700.0;
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        let deep_repair = state.try_deep_aggressive_field_repair();
        assert!(deep_repair);
        approx_eq(state.miles_traveled_actual, 1_700.0);

        state.recent_travel_days.clear();
        for _ in 0..6 {
            state.recent_travel_days.push_back(TravelDayKind::NonTravel);
        }
        let parked_miles = state.miles_traveled_actual;
        state.advance_days(1);
        assert_eq!(
            (state.miles_traveled_actual).to_bits(),
            (parked_miles).to_bits()
        );
        assert!(
            !state
                .ledger
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "stop_cap")
        );

        state.logs.clear();
        state.apply_delay_travel_credit("delay_test");
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_DELAY_CREDIT));
    }

    #[test]
    fn deep_aggressive_safeguards_and_compose() {
        let mut state = GameState {
            ledger: crate::day_accounting::OpenDayLedger {
                current_day_kind: None,
                ..crate::day_accounting::OpenDayLedger::default()
            },
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            miles_traveled_actual: 1_950.0,
            day: 220,
            stats: Stats {
                sanity: 0,
                ..Stats::default()
            },
            budget_cents: 10_000,
            budget: 100,
            ..GameState::default()
        };

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
        let limits = crate::pacing::PacingLimits::default();
        let pace = crate::pacing::PaceCfg::default();
        let classic = state.compute_miles_for_today(&pace, &limits);
        assert!(classic > 0.0);

        // Travel v2 branch with fallback defaults.
        state.features.travel_v2 = true;
        state.mode = GameMode::Deep;
        let v2 = state.compute_miles_for_today(&pace, &limits);
        assert!(v2 > 0.0);
        assert!((classic - v2).abs() > f32::EPSILON);
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
            ledger: crate::day_accounting::OpenDayLedger {
                current_day_kind: Some(TravelDayKind::NonTravel),
                ..crate::day_accounting::OpenDayLedger::default()
            },
            encounter_history: VecDeque::from(vec![0]),
            prev_miles_traveled: 10.0,
            miles_traveled_actual: 10.0,
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: false,
                    partial_traveled_today: false,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            ..GameState::default()
        };
        stagnant.end_of_day();
        assert!(stagnant.day_state.lifecycle.did_end_of_day);
        assert_eq!(stagnant.recent_travel_days.len(), 1);

        // Deep conservative branch applies travel bonus and rotation enforcement.
        let rotation_interval = GameState::default().rotation_force_interval();
        let mut conservative = GameState {
            ledger: crate::day_accounting::OpenDayLedger {
                current_day_kind: Some(TravelDayKind::Travel),
                current_day_miles: 3.0,
                current_day_reason_tags: vec!["progress".into()],
                ..crate::day_accounting::OpenDayLedger::default()
            },
            encounter_history: VecDeque::from(vec![0]),
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Conservative),
            encounters_today: 1,
            prev_miles_traveled: 100.0,
            miles_traveled_actual: 105.0,
            distance_today: 2.0,
            distance_today_raw: 2.5,
            partial_distance_today: 1.5,
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: true,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            distance_cap_today: 6.0,
            rotation_travel_days: rotation_interval,
            recent_travel_days: VecDeque::from(vec![TravelDayKind::Partial; TRAVEL_HISTORY_WINDOW]),
            ..GameState::default()
        };
        conservative.day_state.lifecycle.day_initialized = true;
        conservative.end_of_day();
        assert!(conservative.encounters.force_rotation_pending);
        assert!(
            conservative
                .day_reason_history
                .last()
                .is_some_and(|entry| entry.contains("progress"))
        );

        // Readiness earned while driving survives day finalization.
        let mut aggressive = GameState {
            ledger: crate::day_accounting::OpenDayLedger {
                current_day_reason_tags: vec!["march".into()],
                ..crate::day_accounting::OpenDayLedger::default()
            },
            encounter_history: VecDeque::from(vec![0]),
            mode: GameMode::Deep,
            policy: Some(PolicyKind::Aggressive),
            prev_miles_traveled: DEEP_AGGRESSIVE_BOSS_BIAS_MILES - 10.0,
            miles_traveled_actual: DEEP_AGGRESSIVE_BOSS_BIAS_MILES - 10.0,
            day_state: DayState {
                travel: TravelDayState {
                    traveled_today: true,
                    ..TravelDayState::default()
                },
                ..DayState::default()
            },
            distance_today: 5.0,
            distance_today_raw: 5.0,
            ..GameState::default()
        };
        aggressive.day_state.lifecycle.day_initialized = true;
        aggressive.record_travel_day(TravelDayKind::Travel, 15.0, "march");
        assert!(aggressive.boss.readiness.ready);
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
    fn stationary_stops_preserve_travel_and_rotation_history() {
        let mut state = clear_road_state(PaceId::Steady);
        state.record_travel_day(TravelDayKind::Partial, 5.0, "camp");
        state.add_day_reason_tag("repair");
        assert!(state.day_state.travel.partial_traveled_today);

        assert!(state.rotation_force_interval() >= 3);
        state.recent_travel_days = VecDeque::from(vec![
            TravelDayKind::Travel,
            TravelDayKind::Partial,
            TravelDayKind::NonTravel,
        ]);
        assert!(state.travel_ratio_recent(3) < 1.0);

        state.record_stationary_stop("log.stopped", "delay");
        approx_eq(state.miles_traveled_actual, 5.0);
        assert!(state.logs.iter().any(|entry| entry == "log.stopped"));
        assert!(
            state
                .ledger
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "camp")
        );
        assert!(
            state
                .ledger
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "repair")
        );
        assert!(
            state
                .ledger
                .current_day_reason_tags
                .iter()
                .any(|tag| tag == "delay")
        );
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

        state.budget_cents = DEEP_AGGRESSIVE_SANITY_COST + 1_000;
        state.budget = i32::try_from(state.budget_cents / 100).unwrap_or(0);
        state.guards.deep_aggressive_sanity_guard_used = false;
        state.apply_deep_aggressive_sanity_guard();
        assert!(state.guards.deep_aggressive_sanity_guard_used);

        state.stats.supplies = BOSS_COMPOSE_SUPPLY_COST + 1;
        assert!(state.apply_deep_aggressive_compose());
        state.stats.supplies = 0;
        state.budget_cents = BOSS_COMPOSE_FUNDS_COST + 500;
        assert!(state.apply_deep_aggressive_compose());
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
    PacificCoast,
    MountainWest,
    Southwest,
    Heartland,
    RustBelt,
    Beltway,
}

impl Region {
    pub const ALL: [Self; 6] = [
        Self::PacificCoast,
        Self::MountainWest,
        Self::Southwest,
        Self::Heartland,
        Self::RustBelt,
        Self::Beltway,
    ];
    #[must_use]
    pub const fn asset_key(self) -> &'static str {
        match self {
            Self::PacificCoast => "PacificCoast",
            Self::MountainWest => "MountainWest",
            Self::Southwest => "Southwest",
            Self::Heartland => "Heartland",
            Self::RustBelt => "RustBelt",
            Self::Beltway => "Beltway",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

pub const DEFAULT_STATS: Stats = Stats {
    supplies: 10,
    hp: 10,
    sanity: 10,
    credibility: 5,
    morale: 5,
    allies: 0,
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

pub use crate::party::Party;

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
    #[serde(flatten)]
    pub continuity: crate::journal::Continuity,
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
    pub leg_minutes: u16,
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
    #[serde(default)]
    pub journey_daily: crate::journey::DailyTickConfig,
    #[serde(default)]
    pub daily_remainders: crate::journey::daily::DailyRemainders,
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
    pub encounters_today: u8,
    #[serde(default)]
    pub encounter_history: VecDeque<u8>,
    #[serde(default)]
    pub recent_encounters: VecDeque<RecentEncounter>,
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
    /// Weather state and history for streak tracking
    #[serde(default)]
    pub weather_state: WeatherState,
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
    #[serde(default)]
    pub rotation_backlog: VecDeque<String>,
    #[serde(default, with = "crate::journey::rng_save")]
    pub rng_bundle: Option<Rc<RngBundle>>,
    #[serde(skip)]
    pub data: Option<EncounterData>,
    #[serde(default)]
    pub last_damage: Option<DamageCause>,
    #[serde(default)]
    pub decision_traces_today: Vec<EventDecisionTrace>,
    #[serde(flatten)]
    pub ledger: crate::day_accounting::OpenDayLedger,
    #[serde(default)]
    pub last_breakdown_part: Option<Part>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            continuity: crate::journal::Continuity::default(),
            mode: GameMode::Classic,
            mechanical_policy: MechanicalPolicyId::default(),
            seed: 0,
            state_version: Self::current_version(),
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
            leg_minutes: 0,
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
            journey_daily: crate::journey::DailyTickConfig::default(),
            daily_remainders: crate::journey::daily::DailyRemainders::default(),
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
            encounters_today: 0,
            encounter_history: VecDeque::with_capacity(ENCOUNTER_HISTORY_WINDOW + 2),
            recent_encounters: VecDeque::with_capacity(ENCOUNTER_RECENT_MEMORY),
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
            weather_state: WeatherState::default(),
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
            ledger: crate::day_accounting::OpenDayLedger::default(),
            last_breakdown_part: None,
        }
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

/// Why the crew took an alternate route after a crossing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossingDetourReason {
    RouteDiversion,
    CheckpointDenied,
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
    /// Distinguishes an ordinary diversion from an administrative refusal of passage.
    #[serde(default)]
    pub detour_reason: Option<CrossingDetourReason>,
    pub detour_taken: bool,
    pub detour_hours: Option<u32>,
    pub detour_base_supplies_delta: Option<i32>,
    pub detour_extra_supplies_loss: Option<i32>,
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
            detour_reason: None,
            detour_taken: false,
            detour_hours: None,
            detour_base_supplies_delta: None,
            detour_extra_supplies_loss: None,
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
        self.journey_breakdown
            .weather_factor
            .get(&self.weather_state.today)
            .copied()
            .unwrap_or(1.0)
    }

    fn journey_fatigue_multiplier(&self) -> f32 {
        if self.journey_wear.fatigue_k <= 0.0 {
            return 1.0;
        }
        let excess = (self.miles_traveled_actual - self.journey_wear.comfort_miles).max(0.0);
        self.journey_wear.fatigue_k.mul_add(excess / 400.0, 1.0)
    }

    const fn current_version() -> u16 {
        3
    }

    pub(crate) fn start_of_day(&mut self) {
        if self.day_state.lifecycle.day_initialized {
            return;
        }
        self.day_state.lifecycle.day_initialized = true;
        self.day_state.lifecycle.did_end_of_day = false;
        self.day_state.travel.traveled_today = false;
        self.day_state.travel.partial_traveled_today = false;
        self.encounters_today = 0;
        self.encounters.occurred_today = false;
        self.prev_miles_traveled = self.miles_traveled_actual;
        self.ledger.day_start_remainder = self.ledger.distance_remainder;
        self.ledger.current_day_kind = None;
        self.ledger.current_day_reason_tags.clear();
        self.ledger.current_day_miles = 0.0;
        self.decision_traces_today.clear();
        let day_index = u16::try_from(self.day.saturating_sub(1)).unwrap_or(u16::MAX);
        self.ledger.current_day_record =
            Some(DayRecord::new(day_index, TravelDayKind::NonTravel, 0.0));
        self.exec_travel_multiplier = 1.0;
        self.exec_breakdown_bonus = 0.0;
        self.weather_travel_multiplier = 1.0;
        self.distance_today = 0.0;
        self.distance_today_raw = 0.0;
        self.partial_distance_today = 0.0;
        self.distance_cap_today = 0.0;
        if self.illness_days_remaining == 0 {
            self.illness_travel_penalty = 1.0;
        }
        self.vehicle.tick_breakdown_cooldown();

        if self.encounter_history.len() >= ENCOUNTER_HISTORY_WINDOW {
            self.encounter_history.pop_front();
        }
        self.encounter_history.push_back(0);

        self.tick_exec_order_state();
        self.tick_ally_attrition();

        self.apply_starvation_tick();
        self.roll_daily_illness();
        self.apply_deep_aggressive_sanity_guard();
        let weather_cfg = WeatherConfig::default_config();
        let weather_rng = self.rng_bundle.as_ref().map(Rc::clone);
        crate::weather::process_daily_weather(self, &weather_cfg, weather_rng.as_deref());
        let daily = self.journey_daily.clone();
        let _ = crate::journey::apply_daily_effect(&daily, self);
        let pacing = crate::pacing::PacingConfig::default_config();
        let diet = pacing.get_diet_safe(self.diet.as_str());
        self.consume_daily_effects(diet.sanity, 0);
        self.stats.clamp();
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
            Some((order, duration))
        } else {
            None
        };

        if let Some((order, duration)) = next_order {
            self.current_order = Some(order);
            self.exec_order_days_remaining = duration;
            self.logs
                .push(format!("{}{}", LOG_EXEC_START_PREFIX, order.key()));
            self.apply_exec_order_effects(order);
            if self.exec_order_days_remaining > 0 {
                self.exec_order_days_remaining -= 1;
            }
        }
    }

    fn apply_exec_order_effects(&mut self, order: ExecOrder) {
        let effect = order.daily_effect(self.stats.morale, self.inventory.has_tag("legal_fund"));
        self.stats.supplies = (self.stats.supplies + effect.supplies).max(0);
        self.stats.sanity += effect.sanity;
        self.stats.morale += effect.morale;
        self.exec_travel_multiplier *= effect.travel_multiplier;
        self.exec_breakdown_bonus += effect.breakdown_bonus;
        self.cap_exec_order_effects();
        self.stats.clamp();
    }

    const fn cap_exec_order_effects(&mut self) {
        self.exec_travel_multiplier = self
            .exec_travel_multiplier
            .clamp(EXEC_TRAVEL_MULTIPLIER_CLAMP_MIN, WEATHER_DEFAULT_SPEED);
        self.exec_breakdown_bonus = self
            .exec_breakdown_bonus
            .clamp(PROBABILITY_FLOOR, EXEC_BREAKDOWN_BONUS_CLAMP_MAX);
    }

    pub(crate) fn end_of_day(&mut self) {
        if self.day_state.lifecycle.did_end_of_day {
            return;
        }
        self.tick_camp_cooldowns();
        self.update_encounter_history();
        let miles_delta = self.compute_day_progress();
        self.assert_travel_consistency(miles_delta);

        let day_kind = self.resolve_day_kind();
        self.finalize_day(day_kind);
    }

    fn update_encounter_history(&mut self) {
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
    }

    fn compute_day_progress(&mut self) -> f32 {
        let miles_delta = day_accounting::current_day_distance(self);
        let needs_backfill = self.ledger.current_day_kind.is_none()
            || (matches!(self.ledger.current_day_kind, Some(TravelDayKind::NonTravel))
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
                self.distance_today = self.distance_today.max(miles_delta);
                self.distance_today_raw = self.distance_today_raw.max(miles_delta);
            }
        }
        self.ledger.current_day_miles = miles_delta;
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

    fn resolve_day_kind(&self) -> TravelDayKind {
        if self.ledger.current_day_miles <= 0.0 {
            TravelDayKind::NonTravel
        } else {
            self.ledger
                .current_day_kind
                .unwrap_or(TravelDayKind::Partial)
        }
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
        if let Some(record) = self.ledger.current_day_record.as_mut() {
            record.kind = day_kind;
            record.miles = self.ledger.current_day_miles;
        }
        let reason_entry = if self.ledger.current_day_reason_tags.is_empty() {
            String::new()
        } else {
            self.ledger.current_day_reason_tags.join(";")
        };
        self.day_reason_history.push(reason_entry);
        self.ledger.current_day_reason_tags.clear();
        if let Some(record) = self.ledger.current_day_record.take() {
            self.day_records.push(record);
        }
        self.recompute_day_counters();
        self.ledger.current_day_miles = 0.0;
        self.ledger.current_day_kind = None;
        self.day_state.lifecycle.suppress_stop_ratio = false;
        self.day_state.lifecycle.day_initialized = false;
        self.day_state.lifecycle.did_end_of_day = true;
        self.day = self.day.saturating_add(1);
        self.continuity.clock_minutes = crate::journal::morning();
    }

    fn unlock_aggressive_boss_ready(&mut self) {
        if self.mode.is_deep()
            && matches!(self.policy, Some(PolicyKind::Aggressive))
            && self.ending.is_none()
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
        self.continuity.last_encounter_driving_minutes =
            Some(self.continuity.driving_minutes_total);
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
        self.apply_travel_wear_for_minutes(self.leg_minutes, scale);
    }

    fn apply_travel_wear_for_minutes(&mut self, minutes: u16, scale: f32) {
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
            * scale
            * f32::from(minutes)
            / f32::from(crate::travel_time::TRAVEL_DAY_MINUTES);
        if wear_delta <= 0.0 {
            return;
        }
        self.vehicle.apply_scaled_wear(wear_delta);
    }

    fn apply_travel_wear(&mut self) {
        self.apply_travel_wear_scaled(1.0);
    }

    pub(crate) fn apply_travel_progress(&mut self, distance: f32, kind: TravelProgressKind) -> f32 {
        if distance <= 0.0 {
            return 0.0;
        }
        let before =
            f64::from(self.miles_traveled_actual) + f64::from(self.ledger.distance_remainder);
        let goal = f64::from(self.trail_distance);
        let remaining = (goal - before).max(0.0);
        if remaining <= 0.0 {
            return 0.0;
        }
        let next = (before + f64::from(distance).min(remaining)).min(goal);
        self.miles_traveled_actual = clamp_f64_to_f32(next);
        let next = if self.miles_traveled_actual >= self.trail_distance {
            goal
        } else {
            next
        };
        self.ledger.distance_remainder =
            clamp_f64_to_f32(next - f64::from(self.miles_traveled_actual));
        let applied = clamp_f64_to_f32(next - before);
        self.sync_route_location();
        self.miles_traveled = self.miles_traveled_actual;
        let advanced = applied > 0.0;
        if advanced {
            match kind {
                TravelProgressKind::Full => self.day_state.travel.traveled_today = true,
                TravelProgressKind::Partial => self.day_state.travel.partial_traveled_today = true,
            }
            // Readiness is earned by movement, never by finishing a parked activity.
            self.unlock_aggressive_boss_ready();
            if self.ending.is_none() && self.miles_traveled_actual >= self.trail_distance {
                self.boss.readiness.ready = true;
                self.boss.readiness.reached = true;
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

    fn rotation_force_interval(&self) -> u32 {
        let mut interval = ROTATION_FORCE_INTERVAL;
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Conservative)) {
            interval = interval.saturating_sub(2).max(1);
        }
        interval
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
        if total_u16 == 0 {
            WEATHER_DEFAULT_SPEED
        } else {
            f32::from(traveled_u16) / f32::from(total_u16)
        }
    }

    fn record_stationary_stop(&mut self, log_key: &'static str, reason_tag: &str) {
        self.record_travel_day(TravelDayKind::NonTravel, 0.0, reason_tag);
        self.logs.push(String::from(log_key));
    }

    pub(crate) fn apply_rest_travel_credit(&mut self) {
        self.record_stationary_stop(LOG_TRAVEL_REST_CREDIT, "camp");
    }

    fn apply_delay_travel_credit(&mut self, reason_tag: &str) {
        self.record_stationary_stop(LOG_TRAVEL_DELAY_CREDIT, reason_tag);
    }

    fn apply_classic_field_repair_guard(&mut self) {
        self.record_stationary_stop(LOG_VEHICLE_FIELD_REPAIR_GUARD, "field_repair_guard");
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
        if self.mode == GameMode::Classic && matches!(self.policy, Some(PolicyKind::Balanced)) {
            return false;
        }
        if self.miles_traveled_actual < 1_850.0 {
            return false;
        }
        if (self.miles_traveled_actual - self.endgame.last_limp_mile) < EMERGENCY_LIMP_MILE_WINDOW {
            return false;
        }

        self.record_stationary_stop(LOG_VEHICLE_EMERGENCY_LIMP, "emergency_limp");
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
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return false;
        }
        if self.miles_traveled_actual < 1_500.0 {
            return false;
        }
        let roll = self
            .breakdown_rng()
            .map_or(1.0, |mut rng| rng.r#gen::<f32>());
        if roll >= 0.65 {
            return false;
        }

        self.record_stationary_stop(LOG_DEEP_AGGRESSIVE_FIELD_REPAIR, "field_repair");
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
                .ledger
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
        self.ledger
            .current_day_reason_tags
            .push(trimmed.to_string());
        if let Some(record) = self.ledger.current_day_record.as_mut() {
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
        if let Some(record) = self.ledger.current_day_record.as_mut() {
            record.kind = recorded_kind;
            record.miles = self.ledger.current_day_miles;
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

    fn apply_starvation_tick(&mut self) {
        if self.stats.supplies > 0 {
            if self.starvation_days > 0 {
                self.logs.push(String::from(LOG_STARVATION_RELIEF));
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

        self.mark_damage(DamageCause::Starvation);
        self.logs.push(String::from(LOG_STARVATION_TICK));
        if self.stats.hp <= 0 {
            if !self.guards.starvation_backstop_used {
                self.guards.starvation_backstop_used = true;
                self.stats.hp = 1;
                self.day_state.rest.rest_requested = true;
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
            self.day_state.rest.rest_requested = true;
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

        // Masks reduce new illness risk; existing illness still needs recovery.
        if self.inventory.has_tag("plague_resist") {
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
        self.logs.push(String::from(LOG_DISEASE_HIT));
    }

    fn tick_ally_attrition(&mut self) {
        if self.stats.allies <= 0 {
            return;
        }
        let trigger = self
            .events_rng()
            .is_some_and(|mut rng| rng.r#gen::<f32>() <= ALLY_ATTRITION_CHANCE);
        if trigger {
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

    fn apply_deep_aggressive_sanity_guard(&mut self) {
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

        self.stats.clamp();
        if self.ledger.current_day_kind.is_none() {
            self.record_travel_day(TravelDayKind::Partial, 0.0, "da_sanity_guard");
        } else {
            self.add_day_reason_tag("da_sanity_guard");
        }
        self.guards.deep_aggressive_sanity_guard_used = true;
        self.logs.push(String::from(LOG_BOSS_COMPOSE_FUNDS));
        self.logs.push(String::from(LOG_BOSS_COMPOSE));
    }

    pub(crate) fn apply_deep_aggressive_compose(&mut self) -> bool {
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return false;
        }

        let mut applied = false;
        if self.stats.supplies >= BOSS_COMPOSE_SUPPLY_COST {
            self.stats.supplies -= BOSS_COMPOSE_SUPPLY_COST;
            self.stats.sanity += SANITY_POINT_REWARD;

            self.logs.push(String::from(LOG_BOSS_COMPOSE_SUPPLIES));
            applied = true;
        } else if self.budget_cents >= BOSS_COMPOSE_FUNDS_COST {
            self.budget_cents -= BOSS_COMPOSE_FUNDS_COST;
            self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
            self.stats.sanity += SANITY_POINT_REWARD;

            self.logs.push(String::from(LOG_BOSS_COMPOSE_FUNDS));
            applied = true;
        }

        if applied {
            self.stats.clamp();
            self.logs.push(String::from(LOG_BOSS_COMPOSE));
        }
        applied
    }

    #[must_use]
    fn compute_miles_for_today(
        &mut self,
        pace_cfg: &crate::pacing::PaceCfg,
        limits: &crate::pacing::PacingLimits,
    ) -> f32 {
        let speed = self.current_road_speed(pace_cfg, limits);
        let minutes = self.travel_leg_minutes();
        let (road_miles, distance) =
            self.distance_before_next_stop(speed * f32::from(minutes) / 60.0);
        self.leg_minutes = if speed > 0.0 {
            u16::try_from(crate::numbers::round_f32_to_i32(
                (road_miles * 60.0 / speed).ceil(),
            ))
            .unwrap_or(minutes)
            .min(minutes)
        } else {
            minutes
        };
        self.distance_today_raw = distance;
        self.distance_today = distance;
        self.partial_distance_today = distance * self.journey_partial_ratio.clamp(0.0, 1.0);
        self.distance_cap_today = self.ledger.current_day_miles + distance;
        distance
    }

    fn current_road_speed(
        &self,
        pace_cfg: &crate::pacing::PaceCfg,
        limits: &crate::pacing::PacingLimits,
    ) -> f32 {
        let (weather_scalar, penalty_floor) =
            self.weather_scalar(self.features.travel_v2, &self.journey_travel, limits);
        let cap = pace_cfg.speed_mph.max(1.0);
        (cap * weather_scalar.max(penalty_floor)
            * self.vehicle_penalty()
            * self.malnutrition_penalty()
            * self.exec_travel_multiplier
            * self.illness_travel_penalty.max(0.0)
            * self.endgame_bias()
            * self.travel_boost_multiplier())
        .clamp(0.0, cap)
    }

    fn distance_before_next_stop(&self, road_miles: f32) -> (f32, f32) {
        // The final town marker can precede the route's exact fractional endpoint.
        let remaining = (self.trail_distance - self.miles_traveled_actual).max(0.0);
        let crossing_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let remaining = CROSSING_MILESTONES
            .get(crossing_idx)
            .map_or(remaining, |milestone| {
                remaining.min((milestone - self.miles_traveled_actual).max(0.0))
            });
        let road_to_limit = crate::route::physical_at(self, remaining);
        let road_to_stop = crate::route::upcoming(self)
            .next()
            .map_or(road_miles, |next| {
                let to_town = (f32::from(next.mile) - crate::route::physical_miles(self)).max(0.0);
                road_miles.min(to_town)
            });
        if road_to_limit <= road_to_stop {
            // Carry the exact internal boundary through the physical-mile conversion.
            // Rounding it back from road miles can otherwise require a second tiny leg.
            (road_to_limit, remaining)
        } else {
            (
                road_to_stop,
                crate::route::simulation_distance(self, road_to_stop),
            )
        }
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

    fn vehicle_penalty(&self) -> f32 {
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
        if self.continuity.interactive_repairs {
            if self.vehicle.health <= 0.0 {
                self.set_ending(Ending::VehicleFailure {
                    cause: VehicleFailureCause::Destroyed,
                });
                return true;
            }
            return false;
        }
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
            self.logs.push(String::from(LOG_VEHICLE_FAILURE));
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

    fn handle_crossing_event(&mut self, computed_miles_today: f32) -> Option<(bool, String)> {
        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let &milestone = CROSSING_MILESTONES.get(next_idx)?;
        if self.miles_traveled_actual + f32::EPSILON < milestone {
            return None;
        }

        let kind = self.crossing_kind_for_index(next_idx);
        let cfg = CrossingConfig::default();
        let policy = self.journey_crossing.clone();
        let (has_permit, bribe_offered) = self.crossing_options(&cfg, kind);
        let ctx = CrossingContext {
            policy: &policy,
            kind,
            has_permit,
            bribe_intent: bribe_offered,
            prior_bribe_attempts: self.crossing_bribe_attempts,
        };
        let resolved = self.resolve_crossing_outcome(ctx, next_idx);
        let mut telemetry = CrossingTelemetry::new(self.day, self.region, self.season, kind);
        telemetry.permit_used = resolved.used_permit;
        telemetry.bribe_attempted = resolved.bribe_attempted;
        if resolved.bribe_attempted {
            telemetry.bribe_success = Some(resolved.bribe_succeeded);
        }

        self.apply_crossing_decisions(resolved, &cfg, kind, &mut telemetry);
        let result = self.process_crossing_result(resolved, telemetry, computed_miles_today);
        if !result.0 && self.travel_minutes_available() == 0 {
            self.end_of_day();
        }
        Some(result)
    }

    fn crossing_options(&self, cfg: &CrossingConfig, kind: CrossingKind) -> (bool, bool) {
        let has_permit = crossings::can_use_permit(self, &kind);
        let bribe_offered = !has_permit && crossings::can_afford_bribe(self, cfg, kind);
        (has_permit, bribe_offered)
    }

    fn resolve_crossing_outcome(
        &self,
        ctx: CrossingContext<'_>,
        next_idx: usize,
    ) -> crossings::CrossingOutcome {
        self.crossing_rng().map_or_else(
            || {
                let seed_mix =
                    self.seed ^ (u64::try_from(next_idx).unwrap_or(0) << 32) ^ u64::from(self.day);
                let mut fallback = SmallRng::seed_from_u64(seed_mix);
                crossings::resolve_crossing(ctx, &mut fallback)
            },
            |mut rng| crossings::resolve_crossing(ctx, &mut *rng),
        )
    }

    fn apply_crossing_decisions(
        &mut self,
        resolved: crossings::CrossingOutcome,
        cfg: &CrossingConfig,
        kind: CrossingKind,
        telemetry: &mut CrossingTelemetry,
    ) {
        if resolved.used_permit {
            self.logs.push(String::from(LOG_CROSSING_DECISION_PERMIT));
            let permit_log = crossings::apply_permit(self, cfg, kind);
            self.logs.push(permit_log);
            self.crossing_permit_uses = self.crossing_permit_uses.saturating_add(1);
        }

        if resolved.bribe_attempted {
            self.logs.push(String::from(LOG_CROSSING_DECISION_BRIBE));
            let budget_before = self.budget_cents;
            let _ = crossings::apply_bribe(self, cfg, kind);
            telemetry.bribe_cost_cents = (budget_before - self.budget_cents).max(0);
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
            self.logs.push(log_key.to_string());
        }
    }

    fn process_crossing_result(
        &mut self,
        resolved: crossings::CrossingOutcome,
        mut telemetry: CrossingTelemetry,
        _computed_miles_today: f32,
    ) -> (bool, String) {
        let result = match resolved.result {
            crossings::CrossingResult::TerminalFail
                if self.crossings_completed == 0 && telemetry.kind == CrossingKind::Checkpoint =>
            {
                // The first checkpoint can refuse passage, but an alternate route remains.
                // Keep the original roll and any failed bribe; only its consequence changes.
                telemetry.detour_reason = Some(CrossingDetourReason::CheckpointDenied);
                self.add_day_reason_tag("crossing_denied");
                crossings::CrossingResult::Detour(self.journey_crossing.detour_hours.max.max(1))
            }
            result => result,
        };
        match result {
            crossings::CrossingResult::Pass => {
                telemetry.outcome = CrossingOutcomeTelemetry::Passed;
                self.logs.push(String::from(LOG_CROSSING_PASSED));
                self.crossings_completed = self.crossings_completed.saturating_add(1);
                self.add_day_reason_tag("crossing_pass");
                let before = self.clone();
                self.advance_clock(&before, 30);
                self.stats.clamp();
                self.crossing_events.push(telemetry);
                (false, String::from(LOG_CROSSING_PASSED))
            }
            crossings::CrossingResult::Detour(hours) => {
                let reason = telemetry
                    .detour_reason
                    .unwrap_or(CrossingDetourReason::RouteDiversion);
                telemetry.detour_reason = Some(reason);
                telemetry.detour_taken = true;
                telemetry.detour_hours = Some(u32::from(hours));
                telemetry.outcome = CrossingOutcomeTelemetry::Detoured;
                self.crossing_detours_taken = self.crossing_detours_taken.saturating_add(1);
                self.crossings_completed = self.crossings_completed.saturating_add(1);
                let log_key = match reason {
                    CrossingDetourReason::CheckpointDenied => "log.crossing.denied",
                    CrossingDetourReason::RouteDiversion => LOG_CROSSING_DETOUR,
                };
                self.logs.push(String::from(log_key));
                self.add_day_reason_tag("detour");
                let before = self.clone();
                self.advance_clock(&before, u16::from(hours) * 60);
                self.stats.clamp();
                self.crossing_events.push(telemetry);
                (false, String::from(log_key))
            }
            crossings::CrossingResult::TerminalFail => {
                telemetry.outcome = CrossingOutcomeTelemetry::Failed;
                self.crossing_failures = self.crossing_failures.saturating_add(1);
                self.logs.push(String::from(LOG_CROSSING_FAILURE));
                self.add_day_reason_tag("crossing_fail");
                self.day_state.lifecycle.suppress_stop_ratio = true;
                self.stats.clamp();
                self.set_ending(Ending::Collapse {
                    cause: CollapseCause::Crossing,
                });
                self.crossing_events.push(telemetry);
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
        self.ledger.current_day_record = None;
        self.journey_partial_ratio = JourneyCfg::default_partial_ratio();
        self.journey_travel = TravelConfig::default();
        self.journey_wear = WearConfig::default();
        self.journey_breakdown = BreakdownConfig::default();
        self.journey_part_weights = PartWeights::default();
        self.journey_crossing = CrossingPolicy::default();
        self.logs.push(String::from("log.seed-set"));
        self.data = Some(data);
        self.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));
        self
    }

    #[must_use]
    pub fn rehydrate(mut self, data: EncounterData) -> Self {
        // Older discounted purchases could leave cents. Preserve their purchasing power.
        if self.budget_cents % 100 != 0 {
            self.budget_cents = crate::numbers::whole_dollar_cents(self.budget_cents.max(0));
            self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(i32::MAX);
        }
        self.sync_route_location();
        self.data = Some(data);
        if let Some(player) = self.persona_id.as_deref() {
            self.party.initialize(player, self.seed);
        }
        if self.state_version < Self::current_version() {
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
        if self.rng_bundle.is_none() {
            self.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(self.seed)));
        }
        self
    }

    #[must_use]
    pub const fn region_by_miles(miles: f32) -> Region {
        crate::route::region(miles)
    }

    pub fn travel_next_leg(&mut self, endgame_cfg: &EndgameTravelCfg) -> (bool, String, bool) {
        if self.continuity.interactive_repairs && self.breakdown.is_some() {
            self.day_state.travel.travel_blocked = true;
            return (false, String::from(LOG_TRAVEL_BLOCKED), false);
        }
        self.start_of_day();

        let rng_bundle = self.rng_bundle.as_ref().map(Rc::clone);

        if let Some(result) = self.guard_boss_gate() {
            return result;
        }
        if let Some(result) = self.pre_travel_checks() {
            return result;
        }

        let breakdown_started = self.vehicle_roll();
        if self.continuity.interactive_repairs && self.breakdown.is_some() {
            self.day_state.travel.travel_blocked = true;
            return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
        }
        self.resolve_breakdown();
        if let Some(result) = self.handle_vehicle_state(breakdown_started) {
            return result;
        }
        if let Some(result) = self.handle_travel_block(breakdown_started) {
            return result;
        }

        if let Some(result) = self.process_encounter_flow(rng_bundle.as_ref(), breakdown_started) {
            return result;
        }

        let computed_miles_today = self.distance_today.max(self.distance_today_raw);
        endgame::run_endgame_controller(self, computed_miles_today, breakdown_started, endgame_cfg);
        if let Some((ended, log)) = self.handle_crossing_event(computed_miles_today) {
            return (ended, log, breakdown_started);
        }

        self.apply_travel_wear();
        self.record_travel_day(TravelDayKind::Travel, self.distance_today, "");
        self.spend_driving_time(self.leg_minutes);
        self.log_travel_debug();

        if let Some(log_key) = self.failure_log_key() {
            return (true, String::from(log_key), breakdown_started);
        }

        if let Some((ended, log)) = self.handle_crossing_event(computed_miles_today) {
            return (ended, log, breakdown_started);
        }

        if self.travel_minutes_available() == 0 {
            self.end_of_day();
        }
        (false, String::from(LOG_TRAVELED), breakdown_started)
    }

    fn guard_boss_gate(&self) -> Option<(bool, String, bool)> {
        if self.boss.readiness.ready && !self.boss.outcome.attempted {
            Some((false, String::from("log.boss.await"), false))
        } else {
            None
        }
    }

    fn pre_travel_checks(&mut self) -> Option<(bool, String, bool)> {
        self.stats.clamp();
        self.failure_log_key()
            .map(|log_key| (true, String::from(log_key), false))
    }

    fn handle_vehicle_state(&mut self, breakdown_started: bool) -> Option<(bool, String, bool)> {
        if self.check_vehicle_terminal_state() {
            Some((true, String::from(LOG_VEHICLE_FAILURE), breakdown_started))
        } else {
            None
        }
    }

    fn handle_travel_block(&mut self, breakdown_started: bool) -> Option<(bool, String, bool)> {
        if self.day_state.travel.travel_blocked {
            if !self.day_state.travel.partial_traveled_today {
                self.apply_delay_travel_credit("repair");
            }
            self.end_of_day();
            Some((false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started))
        } else {
            None
        }
    }

    fn process_encounter_flow(
        &mut self,
        rng_bundle: Option<&Rc<RngBundle>>,
        breakdown_started: bool,
    ) -> Option<(bool, String, bool)> {
        if self.encounters.occurred_today || self.encounters_today >= MAX_ENCOUNTERS_PER_DAY {
            return None;
        }

        let drought_due = self.encounter_drought_due();
        let trigger_encounter = self.should_trigger_encounter(rng_bundle, drought_due);
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
            let forced = force_rotation_pending || drought_due;
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
            self.logs.push(String::from(LOG_ENCOUNTER_ROTATION));
        }
        if let Some(enc) = encounter {
            let is_hard_stop = enc.hard_stop;
            let is_major_repair = enc.major_repair;
            if self.features.travel_v2
                && self.distance_today > 0.0
                && !(is_hard_stop || is_major_repair)
                && !drought_due
            {
                let mut partial = if self.partial_distance_today > 0.0 {
                    self.partial_distance_today
                } else {
                    self.distance_today * TRAVEL_PARTIAL_RECOVERY_RATIO
                };
                partial = partial.min(self.distance_today);
                let wear_scale = if self.distance_today > 0.0 {
                    (partial / self.distance_today)
                        .clamp(TRAVEL_PARTIAL_CLAMP_LOW, TRAVEL_PARTIAL_CLAMP_HIGH)
                } else {
                    TRAVEL_PARTIAL_DEFAULT_WEAR
                };
                self.record_travel_day(TravelDayKind::Partial, partial, "");
                let minutes =
                    crate::numbers::round_f32_to_i32(f32::from(self.leg_minutes) * wear_scale);
                self.spend_driving_time(u16::try_from(minutes).unwrap_or(0));
                self.apply_travel_wear_scaled(wear_scale);
                self.logs.push(String::from(LOG_TRAVEL_PARTIAL));
            }
            if is_major_repair {
                self.add_day_reason_tag("repair");
            }
            let encounter_id = enc.id.clone();
            self.current_encounter = Some(enc);
            self.encounters.occurred_today = true;
            self.record_encounter(&encounter_id);
            return Some((false, String::from("log.encounter"), breakdown_started));
        }

        None
    }

    fn encounter_spacing_minutes(&self) -> (u32, u32) {
        let mut distinct = self.recent_encounters.iter().map(|entry| entry.id.as_str());
        let Some(first) = distinct.next() else {
            return (
                ENCOUNTER_EARLY_MIN_DRIVING_MINUTES,
                ENCOUNTER_FIRST_MAX_DRIVING_MINUTES,
            );
        };
        if distinct.any(|id| id != first) {
            (ENCOUNTER_MIN_DRIVING_MINUTES, ENCOUNTER_MAX_DRIVING_MINUTES)
        } else {
            (
                ENCOUNTER_EARLY_MIN_DRIVING_MINUTES,
                ENCOUNTER_SECOND_MAX_DRIVING_MINUTES,
            )
        }
    }

    fn driving_minutes_since_encounter(&self) -> u32 {
        self.continuity
            .driving_minutes_total
            .saturating_sub(self.continuity.last_encounter_driving_minutes.unwrap_or(0))
    }

    /// A drought can bring forward unseen content, never force a repeated scene.
    fn encounter_drought_due(&self) -> bool {
        let (_, maximum) = self.encounter_spacing_minutes();
        self.driving_minutes_since_encounter() >= maximum
            && self.data.as_ref().is_some_and(|data| {
                data.encounters.iter().any(|encounter| {
                    encounter_matches_context(encounter, self.region, self.mode.is_deep())
                        && !self
                            .recent_encounters
                            .iter()
                            .any(|entry| entry.id == encounter.id)
                })
            })
    }

    fn should_trigger_encounter(
        &self,
        rng_bundle: Option<&Rc<RngBundle>>,
        drought_due: bool,
    ) -> bool {
        let Some(bundle) = rng_bundle else {
            return false;
        };
        let (minimum, _) = self.encounter_spacing_minutes();
        if self.driving_minutes_since_encounter() < minimum {
            return false;
        }
        if drought_due {
            return true;
        }
        let roll = {
            let mut rng = bundle.encounter();
            rng.r#gen::<f32>()
        };
        roll < crate::travel_time::probability_for_minutes(
            self.encounter_chance_today,
            self.leg_minutes,
        )
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

    fn log_travel_debug(&self) {
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

    /// Apply vehicle breakdown logic
    fn vehicle_roll(&mut self) -> bool {
        if self.breakdown.is_some() {
            return false;
        }

        if self.vehicle.breakdown_suppressed() {
            return false;
        }

        let wear_level = self.vehicle.wear.max(0.0);
        let mut breakdown_chance = self.journey_breakdown.base
            * self.journey_breakdown.beta.mul_add(wear_level, 1.0)
            * self.journey_pace_factor()
            * self.journey_weather_factor();
        breakdown_chance = (breakdown_chance + self.exec_breakdown_bonus)
            .clamp(PROBABILITY_FLOOR, PROBABILITY_MAX);

        if self.endgame.active && (0.0..1.0).contains(&self.endgame.breakdown_scale) {
            breakdown_chance *= self.endgame.breakdown_scale;
        }
        if matches!(self.policy, Some(PolicyKind::Aggressive)) {
            breakdown_chance = breakdown_chance.max(0.01);
        }
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive)) {
            breakdown_chance *= 0.7;
        }
        breakdown_chance = crate::travel_time::probability_for_minutes(
            breakdown_chance.min(0.35),
            self.leg_minutes,
        );

        let roll = self
            .breakdown_rng()
            .map_or(1.0, |mut rng| rng.r#gen::<f32>());
        if roll >= breakdown_chance {
            return false;
        }

        let choices = [
            (Part::Tire, self.journey_part_weights.tire),
            (Part::Battery, self.journey_part_weights.battery),
            (Part::Alternator, self.journey_part_weights.alt),
            (Part::FuelPump, self.journey_part_weights.pump),
        ];
        let part = if let Some(mut rng) = self.breakdown_rng()
            && let Some(selected) = weighted_pick(&choices, &mut *rng)
        {
            selected
        } else {
            Part::Tire
        };
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

    /// Resolve an encounter, its half-hour cost, and any arrival as one action.
    /// Ride time belongs to the choice; crossing delays are additional elapsed time.
    pub fn resolve_encounter_choice(&mut self, idx: usize) -> bool {
        let affordable = self
            .current_encounter
            .as_ref()
            .and_then(|encounter| encounter.choices.get(idx))
            .is_some_and(|choice| {
                choice
                    .effects
                    .affordable(&self.stats, self.budget_cents, self.receipts.len())
            });
        if !affordable {
            return false;
        }
        let before = self.clone();
        self.apply_choice(idx);
        self.advance_clock(&before, 30);
        if self.day > before.day
            && self.continuity.clock_minutes > crate::travel_time::TRAVEL_DAY_START
        {
            self.start_of_day();
        }
        if self.miles_traveled_actual > before.miles_traveled_actual {
            if self.failure_log_key().is_none() {
                let _ = self.handle_crossing_event(self.distance_today);
            }
            self.update_route_services(before.miles_traveled_actual);
        }
        if self.day > before.day
            && self.continuity.clock_minutes > crate::travel_time::TRAVEL_DAY_START
            && self.ending.is_none()
        {
            self.start_of_day();
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
            if !eff.affordable(&self.stats, self.budget_cents, self.receipts.len()) {
                return;
            }
            self.budget_cents = self.budget_cents.saturating_add(eff.cash_cents);
            self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
            self.stats.hp += eff.hp;
            self.stats.sanity += eff.sanity;
            self.stats.credibility += eff.credibility;
            self.stats.supplies += eff.supplies;
            self.stats.morale += eff.morale;
            self.stats.allies += eff.allies;

            if eff.hp < 0 {
                self.mark_damage(DamageCause::Breakdown);
            }
            if let Some(r) = &eff.add_receipt {
                self.collect_receipt(r);
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

            self.apply_encounter_ride(eff.travel_bonus_ratio);
            if eff.rest {
                if !self.day_state.rest.rest_requested {
                    self.logs.push(String::from(LOG_REST_REQUESTED_ENCOUNTER));
                }
                self.request_rest();
            }
        }

        self.finalize_encounter();
    }

    fn apply_encounter_ride(&mut self, ratio: f32) {
        const CHOICE_MINUTES: u16 = 30;
        if !ratio.is_finite()
            || ratio <= 0.0
            || self.breakdown.is_some()
            || self.day_state.travel.travel_blocked
            || self.ending.is_some()
            || self.stats.hp <= 0
            || self.stats.sanity <= 0
            || self.continuity.clock_minutes < crate::travel_time::TRAVEL_DAY_START
        {
            return;
        }
        let requested = crate::numbers::round_f32_to_i32(ratio.clamp(0.0, 0.5) * 60.0);
        let minutes = u16::try_from(requested)
            .unwrap_or(0)
            .min(CHOICE_MINUTES)
            .min(self.travel_minutes_available());
        if minutes == 0 {
            return;
        }
        self.start_of_day();
        if self.stats.hp <= 0 || self.stats.sanity <= 0 {
            return;
        }
        let pacing = crate::pacing::PacingConfig::default_config();
        let pace = pacing.get_pace_safe(self.pace.as_str());
        let speed = self.current_road_speed(&pace, &pacing.limits);
        if speed <= 0.0 {
            return;
        }
        let (_, distance) = self.distance_before_next_stop(speed * f32::from(minutes) / 60.0);
        let before = crate::route::physical_miles(self);
        self.record_travel_day(TravelDayKind::Partial, distance, "");
        let traveled = crate::route::physical_miles(self) - before;
        if traveled <= 0.0 {
            return;
        }
        let elapsed = crate::numbers::round_f32_to_i32((traveled * 60.0 / speed).ceil());
        let driven_minutes = u16::try_from(elapsed).unwrap_or(minutes).min(minutes);
        self.spend_driving_time(driven_minutes);
        self.apply_travel_wear_for_minutes(driven_minutes, 1.0);
        self.logs.push(String::from(LOG_TRAVEL_BONUS));
    }

    fn resolve_breakdown(&mut self) {
        if let Some(breakdown) = self.breakdown.clone() {
            if self.consume_spare_for_part(breakdown.part) {
                self.vehicle.repair(VEHICLE_JURY_RIG_HEAL);
                self.breakdown = None;
                self.day_state.travel.travel_blocked = false;
                self.last_breakdown_part = None;
                self.logs.push(String::from("log.breakdown-repaired"));
                return;
            }

            if self.total_spares() == 0 && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.spend_emergency_repair(LOG_VEHICLE_REPAIR_EMERGENCY);
                self.breakdown = None;
                self.day_state.travel.travel_blocked = false;
                self.last_breakdown_part = None;
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
                self.logs.push(String::from("log.breakdown-jury-rigged"));
            } else {
                self.day_state.travel.travel_blocked = true;
            }
        } else {
            self.day_state.travel.travel_blocked = false;
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
        self.exec_travel_multiplier = (self.exec_travel_multiplier * VEHICLE_EXEC_MULTIPLIER_DECAY)
            .max(VEHICLE_EXEC_MULTIPLIER_FLOOR);
        self.logs.push(String::from(LOG_VEHICLE_REPAIR_SPARE));
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
        self.exec_travel_multiplier = (self.exec_travel_multiplier * VEHICLE_EXEC_MULTIPLIER_DECAY)
            .max(VEHICLE_EXEC_MULTIPLIER_FLOOR);
        self.logs.push(String::from(log_key));
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
            self.logs.push(String::from(LOG_DISEASE_RECOVER));
            self.disease_cooldown = self.disease_cooldown.max(DISEASE_COOLDOWN_DAYS);
        }
    }

    /// Recompute the next leg from remaining time without repeating daily costs.
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
        let weather = crate::weather::WeatherConfig::default_config();
        let weather_encounter = weather
            .effects
            .get(&self.weather_state.today)
            .map_or(0.0, |effect| effect.enc_delta);
        let mut encounter = encounter_base + pace_cfg.encounter_chance_delta + weather_encounter;

        let _ = self.compute_miles_for_today(&pace_cfg, limits);

        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            encounter = (encounter + ENCOUNTER_CRITICAL_VEHICLE_BONUS)
                .clamp(encounter_floor, encounter_ceiling);
        }

        let encounters_last_window: u32 =
            self.encounter_history.iter().copied().map(u32::from).sum();
        if encounters_last_window >= ENCOUNTER_SOFT_CAP_THRESHOLD {
            encounter *= ENCOUNTER_SOFT_CAP_FACTOR;
        }

        if self.encounters_today >= MAX_ENCOUNTERS_PER_DAY {
            encounter = PROBABILITY_FLOOR;
        }

        self.encounter_chance_today = encounter
            .clamp(encounter_floor, encounter_ceiling)
            .max(PROBABILITY_FLOOR);

        self.receipt_bonus_pct = diet_cfg.receipt_find_pct_delta.clamp(-100, 100);
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
        self.continuity.route_services.route_id = Some(persona.id.clone());
        self.sync_route_location();
        self.logs
            .push(format!("log.persona.selected.{}", persona.id));
    }

    pub fn set_party<I, S>(&mut self, leader: S, companions: I)
    where
        I: IntoIterator,
        I::Item: Into<String>,
        S: Into<String>,
    {
        let player = self.persona_id.as_deref().unwrap_or("journalist");
        self.party.initialize(player, self.seed);
        let mut names = companions.into_iter().map(Into::into);
        let leader = leader.into();
        for member in &mut self.party.members {
            if member.persona == player {
                member.name.clone_from(&leader);
            } else if let Some(name) = names.next() {
                member.name = name;
            }
        }
        self.party.sync_names(player);
        self.logs.push(String::from("log.party.updated"));
    }

    pub const fn request_rest(&mut self) {
        self.day_state.rest.rest_requested = true;
    }

    fn failure_log_key(&mut self) -> Option<&'static str> {
        if self.vehicle.health <= 0.0 {
            if self.continuity.interactive_repairs {
                self.set_ending(Ending::VehicleFailure {
                    cause: VehicleFailureCause::Destroyed,
                });
                return Some(LOG_VEHICLE_FAILURE);
            }
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
            self.start_of_day();
            self.record_travel_day(kind, miles, reason_tag);
            self.end_of_day();
            self.day_state.lifecycle.suppress_stop_ratio = false;
        }
    }

    #[cfg(test)]
    pub fn vehicle_roll_test(&mut self) -> bool {
        self.vehicle_roll()
    }

    pub const fn tick_camp_cooldowns(&mut self) {
        if self.camp.rest_cooldown > 0 {
            self.camp.rest_cooldown -= 1;
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
}
