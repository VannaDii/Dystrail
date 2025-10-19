//! Centralized balance and tuning constants for Dystrail game logic.
//!
//! These values define the deterministic math for the core simulation.
//! Keeping them together ensures that gameplay can only be adjusted via
//! code changes reviewed in version control, rather than through external
//! JSON assets.

// Logging keys -------------------------------------------------------------
pub(crate) const DEBUG_ENV_VAR: &str = "DYSTRAIL_DEBUG_LOGS";
pub(crate) const LOG_PANTS_EMERGENCY: &str = "log.pants-emergency";
pub(crate) const LOG_HEALTH_COLLAPSE: &str = "log.health-collapse";
pub(crate) const LOG_SANITY_COLLAPSE: &str = "log.sanity-collapse";
pub(crate) const LOG_TRAVEL_BLOCKED: &str = "log.travel-blocked";
pub(crate) const LOG_TRAVELED: &str = "log.traveled";
pub(crate) const LOG_EXEC_START_PREFIX: &str = "exec.start.";
pub(crate) const LOG_EXEC_END_PREFIX: &str = "exec.end.";
pub(crate) const LOG_STARVATION_TICK: &str = "log.starvation.tick";
pub(crate) const LOG_STARVATION_RELIEF: &str = "log.starvation.relief";
pub(crate) const LOG_ALLY_LOST: &str = "log.ally.lost";
pub(crate) const LOG_ALLIES_GONE: &str = "log.allies.gone";
pub(crate) const LOG_VEHICLE_FAILURE: &str = "log.vehicle.failure";
pub(crate) const LOG_VEHICLE_FIELD_REPAIR_GUARD: &str = "log.vehicle.field-repair-guard";
pub(crate) const LOG_VEHICLE_EMERGENCY_LIMP: &str = "log.vehicle.emergency-limp";
pub(crate) const LOG_DEEP_AGGRESSIVE_FIELD_REPAIR: &str = "log.vehicle.da-field-repair";
pub(crate) const LOG_VEHICLE_REPAIR_EMERGENCY: &str = "log.vehicle.repair.emergency";
pub(crate) const LOG_EMERGENCY_REPAIR_FORCED: &str = "log.vehicle.repair.forced";
pub(crate) const LOG_VEHICLE_REPAIR_SPARE: &str = "log.vehicle.repair.spare";
pub(crate) const LOG_BOSS_COMPOSE: &str = "log.boss.compose";
pub(crate) const LOG_BOSS_COMPOSE_SUPPLIES: &str = "log.boss.compose.supplies";
pub(crate) const LOG_BOSS_COMPOSE_FUNDS: &str = "log.boss.compose.funds";
pub(crate) const LOG_CROSSING_DETOUR: &str = "log.crossing.detour";
pub(crate) const LOG_CROSSING_PASSED: &str = "log.crossing.passed";
pub(crate) const LOG_CROSSING_FAILURE: &str = "log.crossing.failure";
pub(crate) const LOG_CROSSING_DECISION_BRIBE: &str = "log.crossing.decision.bribe";
pub(crate) const LOG_CROSSING_DECISION_PERMIT: &str = "log.crossing.decision.permit";
pub(crate) const LOG_TRAVEL_PARTIAL: &str = "log.travel.partial";
pub(crate) const LOG_TRAVEL_REST_CREDIT: &str = "log.travel.rest-credit";
pub(crate) const LOG_TRAVEL_DELAY_CREDIT: &str = "log.travel.delay-credit";
pub(crate) const LOG_ENCOUNTER_ROTATION: &str = "log.encounter.rotation";
pub(crate) const LOG_TRAVEL_BONUS: &str = "log.travel.bonus";
pub(crate) const LOG_ENDGAME_ACTIVATE: &str = "log.endgame.activate";
pub(crate) const LOG_ENDGAME_FIELD_REPAIR: &str = "log.endgame.field-repair";
pub(crate) const LOG_ENDGAME_FAILURE_GUARD: &str = "log.endgame.guard";
pub(crate) const LOG_DISEASE_HIT: &str = "log.disease.hit";
pub(crate) const LOG_DISEASE_TICK: &str = "log.disease.tick";
pub(crate) const LOG_DISEASE_RECOVER: &str = "log.disease.recover";
pub(crate) const LOG_STARVATION_BACKSTOP: &str = "log.starvation.backstop";
pub(crate) const LOG_REST_REQUESTED_ENCOUNTER: &str = "log.encounter.rest-requested";

// Vehicle tuning -----------------------------------------------------------
pub(crate) const VEHICLE_BREAKDOWN_DAMAGE: f32 = 6.0;
pub(crate) const VEHICLE_DAILY_WEAR: f32 = 0.2;
pub(crate) const VEHICLE_CRITICAL_THRESHOLD: f32 = 20.0;
pub(crate) const VEHICLE_HEALTH_MAX: f32 = 100.0;
pub(crate) const VEHICLE_BREAKDOWN_WEAR: f32 = 6.0;
pub(crate) const VEHICLE_BREAKDOWN_WEAR_CLASSIC: f32 = 5.0;
pub(crate) const VEHICLE_EMERGENCY_HEAL: f32 = 10.0;
pub(crate) const VEHICLE_JURY_RIG_HEAL: f32 = 4.0;
pub(crate) const VEHICLE_CRITICAL_SPEED_FACTOR: f32 = 0.5;
pub(crate) const VEHICLE_MALNUTRITION_PENALTY_PER_STACK: f32 = 0.05;
pub(crate) const VEHICLE_MALNUTRITION_MIN_FACTOR: f32 = 0.3;
pub(crate) const VEHICLE_BREAKDOWN_BASE_CHANCE: f32 = 0.04;
pub(crate) const VEHICLE_BREAKDOWN_WEAR_COEFFICIENT: f32 = 0.2;
pub(crate) const VEHICLE_BREAKDOWN_EXTREME_WEATHER_BONUS: f32 = 0.04;
pub(crate) const VEHICLE_BREAKDOWN_CRITICAL_BONUS: f32 = 0.05;
pub(crate) const VEHICLE_EXEC_MULTIPLIER_DECAY: f32 = 0.85;
pub(crate) const VEHICLE_EXEC_MULTIPLIER_FLOOR: f32 = 0.7;
pub(crate) const VEHICLE_BREAKDOWN_PARTIAL_FACTOR: f32 = 0.5;
pub(crate) const VEHICLE_BASE_TOLERANCE_DEEP: i32 = 4;
pub(crate) const VEHICLE_BASE_TOLERANCE_CLASSIC: i32 = 5;
pub(crate) const VEHICLE_SPARE_GUARD_SCALE: i32 = 3;
pub(crate) const VEHICLE_DEEP_EMERGENCY_HEAL_BALANCED: f32 = VEHICLE_HEALTH_MAX * 0.12;
pub(crate) const VEHICLE_DEEP_EMERGENCY_HEAL_AGGRESSIVE: f32 = VEHICLE_HEALTH_MAX * 0.15;
pub(crate) const DEEP_EMERGENCY_REPAIR_THRESHOLD: f32 = 1_900.0;
pub(crate) const CLASSIC_BALANCED_FAILURE_GUARD_MILES: f32 = 1_950.0;
pub(crate) const CLASSIC_FIELD_REPAIR_COST_CENTS: i64 = 2_500;
pub(crate) const EMERGENCY_LIMP_REPAIR_COST_CENTS: i64 = 1_500;
pub(crate) const CLASSIC_FIELD_REPAIR_WEAR_REDUCTION: f32 = 0.35;
pub(crate) const EMERGENCY_LIMP_WEAR_REDUCTION: f32 = 0.20;
pub(crate) const EMERGENCY_LIMP_MILE_WINDOW: f32 = 200.0;

// Encounter tuning ---------------------------------------------------------
pub(crate) const DEFAULT_SUPPLY_COST: i32 = 1;
pub(crate) const BLITZ_SUPPLY_COST: i32 = 2;
pub(crate) const ENCOUNTER_BASE_DEFAULT: f32 = 0.27;
pub(crate) const ENCOUNTER_COOLDOWN_DAYS: u8 = 1;
pub(crate) const ENCOUNTER_SOFT_CAP_THRESHOLD: u32 = 5;
pub(crate) const ENCOUNTER_HISTORY_WINDOW: usize = 10;
pub(crate) const MAX_ENCOUNTERS_PER_DAY: u8 = 2;
pub(crate) const ENCOUNTER_RECENT_MEMORY: usize = 8;
pub(crate) const ENCOUNTER_REPEAT_WINDOW_DAYS: u32 = 6;
pub(crate) const ENCOUNTER_EXTENDED_MEMORY_DAYS: u32 = ENCOUNTER_REPEAT_WINDOW_DAYS * 2;
pub(crate) const ENCOUNTER_REROLL_PENALTY: f32 = 0.8;
pub(crate) const ENCOUNTER_CRITICAL_VEHICLE_BONUS: f32 = 0.12;
pub(crate) const ENCOUNTER_SOFT_CAP_FACTOR: f32 = TRAVEL_PARTIAL_RATIO;

// Executive order tuning ---------------------------------------------------
pub(crate) const EXEC_ORDER_DAILY_CHANCE: f32 = 0.06;
pub(crate) const EXEC_ORDER_MIN_DURATION: u8 = 2;
pub(crate) const EXEC_ORDER_MAX_DURATION: u8 = 4;
pub(crate) const EXEC_ORDER_MIN_COOLDOWN: u8 = 6;
pub(crate) const EXEC_ORDER_MAX_COOLDOWN: u8 = 9;
pub(crate) const EXEC_ORDER_SPEED_BONUS: f32 = 0.88;
pub(crate) const EXEC_ORDER_BREAKDOWN_BONUS: f32 = 0.10;
pub(crate) const EXEC_TRAVEL_MULTIPLIER_CLAMP_MIN: f32 = 0.72;
pub(crate) const EXEC_BREAKDOWN_BONUS_CLAMP_MAX: f32 = 0.2;

// Travel parameters --------------------------------------------------------
pub(crate) const CROSSING_MILESTONES: [f32; 3] = [650.0, 1_250.0, 1_900.0];
pub(crate) const REST_TRAVEL_CREDIT_MILES: f32 = 12.0;
pub(crate) const DELAY_TRAVEL_CREDIT_MILES: f32 = 9.0;
pub(crate) const TRAVEL_HISTORY_WINDOW: usize = 10;
pub(crate) const TRAVEL_PARTIAL_MIN_DISTANCE: f32 = 1.0;
pub(crate) const TRAVEL_V2_BASE_DISTANCE: f32 = 13.5;
pub(crate) const TRAVEL_CLASSIC_BASE_DISTANCE: f32 = 12.0;
pub(crate) const TRAVEL_CONFIG_MIN_MULTIPLIER: f32 = 0.1;
pub(crate) const TRAVEL_V2_PENALTY_FLOOR: f32 = 0.7;
pub(crate) const TRAVEL_CLASSIC_PENALTY_FLOOR: f32 = 0.6;
pub(crate) const TRAVEL_PARTIAL_RATIO: f32 = 0.45;
pub(crate) const TRAVEL_PARTIAL_CLAMP_LOW: f32 = 0.55;
pub(crate) const TRAVEL_PARTIAL_CLAMP_HIGH: f32 = 0.99;
pub(crate) const TRAVEL_PARTIAL_RECOVERY_RATIO: f32 = 0.92;
pub(crate) const TRAVEL_PARTIAL_DEFAULT_WEAR: f32 = 0.85;
pub(crate) const TRAVEL_RATIO_DEFAULT: f32 = 0.9;
pub(crate) const WEATHER_PACE_MULTIPLIER_FLOOR: f32 = 0.90;
pub(crate) const BEHIND_SCHEDULE_MILES_PER_DAY: f32 = 26.5;

pub(crate) const ROTATION_FORCE_INTERVAL: u32 = 5;
pub(crate) const ROTATION_LOOKBACK_DAYS: u32 = 5;

pub(crate) const DEEP_CONSERVATIVE_BOOSTS: &[(u32, f32, f32)] =
    &[(145, 1_950.0, 1.05), (130, 1_750.0, 1.04)];

pub(crate) const DEEP_AGGRESSIVE_BOOSTS: &[(u32, f32, f32)] = &[
    (140, 1_900.0, 1.15),
    (120, 1_650.0, 1.10),
    (100, 1_400.0, 1.06),
];
pub(crate) const DEEP_AGGRESSIVE_BOSS_BIAS_MILES: f32 = 2_050.0;

pub(crate) const DEEP_BALANCED_TOLERANCE_THRESHOLDS: &[(f32, i32)] = &[(1_950.0, 2), (1_900.0, 1)];
pub(crate) const DEEP_BALANCED_FAILSAFE_DISTANCE: f32 = 1_950.0;
pub(crate) const DEEP_BALANCED_TRAVEL_NUDGE: f32 = 1.003;
pub(crate) const DEEP_AGGRESSIVE_TOLERANCE_THRESHOLDS: &[(f32, i32)] =
    &[(1_950.0, 3), (1_850.0, 2)];

// Weather tuning -----------------------------------------------------------
pub(crate) const WEATHER_COLD_SNAP_SPEED: f32 = 0.98;
pub(crate) const WEATHER_STORM_SMOKE_SPEED: f32 = 0.99;
pub(crate) const WEATHER_HEAT_WAVE_SPEED: f32 = 0.97;
pub(crate) const WEATHER_DEFAULT_SPEED: f32 = 1.0;
pub(crate) const PROBABILITY_FLOOR: f32 = 0.0;
pub(crate) const PROBABILITY_MAX: f32 = 1.0;

// Disease tuning -----------------------------------------------------------
pub(crate) const DISEASE_DAILY_CHANCE: f32 = 0.012;
pub(crate) const DISEASE_COOLDOWN_DAYS: u32 = 5;
pub(crate) const DISEASE_SANITY_PENALTY: i32 = 1;
pub(crate) const DISEASE_HP_PENALTY: i32 = 1;
pub(crate) const DISEASE_SUPPLY_PENALTY: i32 = 1;
pub(crate) const ILLNESS_TRAVEL_PENALTY: f32 = 0.85;
pub(crate) const DISEASE_DURATION_RANGE: (u32, u32) = (2, 4);
pub(crate) const DISEASE_SUPPLIES_BONUS: f32 = 0.02;
pub(crate) const DISEASE_STARVATION_BONUS: f32 = 0.015;
pub(crate) const DISEASE_LOW_HP_BONUS: f32 = 0.01;
pub(crate) const DISEASE_MAX_DAILY_CHANCE: f32 = 0.18;
pub(crate) const DISEASE_TICK_HP_LOSS: i32 = 1;
pub(crate) const DISEASE_TICK_SANITY_LOSS: i32 = 1;

// Starvation tuning --------------------------------------------------------
pub(crate) const STARVATION_BASE_HP_LOSS: i32 = 1;
pub(crate) const STARVATION_SANITY_LOSS: i32 = 1;
pub(crate) const STARVATION_PANTS_GAIN: i32 = 1;
pub(crate) const STARVATION_MAX_STACK: u32 = 5;
pub(crate) const STARVATION_GRACE_DAYS: u32 = 1;
// Miscellaneous thresholds -------------------------------------------------
pub(crate) const ALLY_ATTRITION_CHANCE: f32 = 0.02;
pub(crate) const EMERGENCY_REPAIR_COST: i64 = 1_000;
#[cfg(test)]
pub(crate) const ASSERT_MIN_AVG_MPD: f64 = 12.0;
#[cfg(test)]
pub(crate) const FLOAT_EPSILON: f64 = 1e-6;

pub(crate) const AGGRESSIVE_STOP_WINDOW_DAYS: usize = 10;
pub(crate) const AGGRESSIVE_STOP_CAP: usize = 2;

pub(crate) const DEEP_AGGRESSIVE_SANITY_DAY: u32 = 130;
pub(crate) const DEEP_AGGRESSIVE_SANITY_MILES: f32 = 1_800.0;
pub(crate) const DEEP_AGGRESSIVE_SANITY_COST: i64 = 2_000;
pub(crate) const DEEP_AGGRESSIVE_SANITY_PANTS_PENALTY: i32 = 3;

pub(crate) const BOSS_COMPOSE_SUPPLY_COST: i32 = 4;
pub(crate) const BOSS_COMPOSE_PANTS_SUPPLY: i32 = 5;
pub(crate) const BOSS_COMPOSE_FUNDS_PANTS: i32 = 3;
pub(crate) const BOSS_COMPOSE_FUNDS_COST: i64 = 2_000;
pub(crate) const SANITY_POINT_REWARD: i32 = 1;

pub(crate) const CONSERVATIVE_BREAKDOWN_FACTOR: f32 = 0.90;
pub(crate) const CONSERVATIVE_DEEP_MULTIPLIER: f32 = 0.95;

pub(crate) const PACE_STEADY_BASE: f32 = 1.0;
pub(crate) const PACE_HEATED_BASE: f32 = 1.15;
pub(crate) const PACE_BLITZ_BASE: f32 = 1.30;
pub(crate) const PACE_BREAKDOWN_STEADY: f32 = 0.95;
pub(crate) const PACE_BREAKDOWN_HEATED: f32 = 1.0;
pub(crate) const PACE_BREAKDOWN_BLITZ: f32 = 1.10;

pub(crate) const PERMIT_REQUIRED_TAGS: &[&str] = &["permit", "press_pass"];
