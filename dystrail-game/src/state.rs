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
#[allow(clippy::wildcard_imports)]
use crate::constants::*;
use crate::crossings::{self, CrossingConfig, CrossingContext, CrossingKind};
use crate::data::{Encounter, EncounterData};
use crate::day_accounting;
use crate::encounters::{EncounterRequest, pick_encounter};
use crate::endgame::{self, EndgameState, EndgameTravelCfg};
use crate::exec_orders::ExecOrder;
use crate::journey::{
    BreakdownConfig, CountingRng, CrossingPolicy, DayRecord, DayTag, JourneyCfg, RngBundle,
    TravelConfig, TravelDayKind, WearConfig,
};
use crate::personas::{Persona, PersonaMods};
use crate::vehicle::{Breakdown, Part, PartWeights, Vehicle, weighted_pick};
use crate::weather::{Weather, WeatherConfig, WeatherState};

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
const fn default_pace() -> PaceId {
    PaceId::Steady
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::field_reassign_with_default,
        clippy::float_cmp,
        clippy::cognitive_complexity,
        clippy::too_many_lines
    )]
    use super::*;
    use crate::data::{Choice, Effects, Encounter};
    use crate::journey::{CountingRng, RngBundle};
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
                if rng.random::<f32>() < threshold {
                    return Rc::new(RngBundle::from_user_seed(seed));
                }
            }
        }
        panic!("unable to find deterministic seed below {threshold}");
    }

    fn encounter_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::encounter)
    }

    fn travel_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::travel)
    }

    fn breakdown_bundle_with_roll_below(threshold: f32) -> Rc<RngBundle> {
        bundle_with_roll_below(threshold, RngBundle::breakdown)
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
    fn breakdown_uses_part_weights() {
        let mut state = GameState::default();
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(7)));
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
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.inventory.spares.tire = 1;
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.travel_blocked = true;
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(1)));
        state.data = Some(EncounterData::empty());

        let cfg = endgame_cfg();
        let (_ended, _msg, _started) = state.travel_next_leg(&cfg);

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
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(2)));
        state.data = Some(EncounterData::empty());

        let cfg = endgame_cfg();
        let (_ended_first, msg_first, _started_first) = state.travel_next_leg(&cfg);
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
        state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(3)));
        state.encounter_chance_today = 0.0;
        state.data = Some(EncounterData::empty());

        let cfg = endgame_cfg();
        let (_ended, _msg, _started) = state.travel_next_leg(&cfg);

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
        state.detach_rng_bundle();
        state.pace = PaceId::Steady;
        let pacing = crate::pacing::PacingConfig::default_config();
        let cfg = endgame_cfg();
        for _ in 0..30 {
            state.start_of_day();
            state.weather_state.today = Weather::Clear;
            state.weather_state.yesterday = Weather::Clear;
            state.apply_pace_and_diet(&pacing);
            state.encounter_chance_today = 0.0;
            let (ended, _, _) = state.travel_next_leg(&cfg);
            assert!(!ended, "run ended prematurely");
        }
        assert!(
            state.travel_days + state.partial_travel_days >= 30,
            "expected at least 30 days with travel credit"
        );
        let travel_days = state.travel_days.max(1);
        let avg_mpd = if state.travel_days > 0 {
            f64::from(state.miles_traveled_actual) / f64::from(travel_days)
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
        #![allow(clippy::field_reassign_with_default)]
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
        #![allow(clippy::field_reassign_with_default)]
        let cfg = crate::pacing::PacingConfig::default_config();

        let mut base_state = GameState::default();
        base_state.detach_rng_bundle();
        base_state.apply_pace_and_diet(&cfg);
        let base = base_state.encounter_chance_today;
        assert!((f64::from(base) - f64::from(ENCOUNTER_BASE_DEFAULT)).abs() < FLOAT_EPSILON);

        let mut capped_state = GameState::default();
        capped_state.detach_rng_bundle();
        capped_state.encounter_history = VecDeque::from(vec![2, 1, 1, 1, 0, 0, 0, 0, 0]);
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
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.features.travel_v2 = true;
        state.stats.supplies = 5;
        state.stats.pants = 20;
        state.distance_today = 5.0;
        state.distance_today_raw = 5.0;
        state.partial_distance_today = 2.0;
        state
            .current_day_reason_tags
            .extend(["camp".into(), "repair".into()]);
        state.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::NonTravel; TRAVEL_HISTORY_WINDOW]);
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
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
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

        let end_cfg = endgame_cfg();
        let (ended, message, _) = state.travel_next_leg(&end_cfg);
        assert!(!ended);
        assert_eq!(message, LOG_TRAVELED);
        assert!(state.current_encounter.is_none());
    }

    #[test]
    fn allows_two_encounters_before_cooldown() {
        #![allow(clippy::field_reassign_with_default)]
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
        let cfg = crate::pacing::PacingConfig::default_config();

        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        let end_cfg = endgame_cfg();
        let (_ended_first, msg_first, _) = state.travel_next_leg(&end_cfg);
        assert_eq!(msg_first, "log.encounter");
        assert_eq!(state.encounters_today, 1);
        state.apply_choice(0);
        assert!(!state.encounter_occurred_today);

        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        let (_ended_second, msg_second, _) = state.travel_next_leg(&end_cfg);
        assert_eq!(msg_second, "log.encounter");
        assert_eq!(state.encounters_today, 2);
        state.apply_choice(0);
        assert!(state.encounter_occurred_today);

        state.apply_pace_and_diet(&cfg);
        state.encounter_chance_today = 1.0;
        let (_ended_third, msg_third, _) = state.travel_next_leg(&end_cfg);
        assert_eq!(msg_third, LOG_TRAVELED);
        assert_eq!(
            state.encounter_history.back(),
            Some(&MAX_ENCOUNTERS_PER_DAY)
        );
    }

    #[test]
    fn stop_cap_conversion_awards_partial_credit() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.features.travel_v2 = true;
        state.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::NonTravel; AGGRESSIVE_STOP_WINDOW_DAYS]);
        state.distance_today = 20.0;
        state.distance_today_raw = 20.0;
        state.vehicle.wear = 5.0;

        state.enforce_aggressive_delay_cap(20.0);

        assert!(
            state.partial_traveled_today,
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
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.day = DEEP_AGGRESSIVE_SANITY_DAY;
        state.miles_traveled_actual = DEEP_AGGRESSIVE_SANITY_MILES;
        state.stats.sanity = 0;
        state.budget_cents = DEEP_AGGRESSIVE_SANITY_COST;

        state.apply_deep_aggressive_sanity_guard();

        assert!(state.deep_aggressive_sanity_guard_used);
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
    #[allow(clippy::float_cmp, clippy::too_many_lines)]
    fn roll_and_exec_paths_cover_branches() {
        let mut state = GameState::default();
        state.data = Some(EncounterData::empty());

        // Existing illness countdown branch.
        state.illness_days_remaining = 2;
        state.stats.hp = 10;
        state.stats.sanity = 10;
        state.stats.supplies = 6;
        state.disease_cooldown = 0;
        state.attach_rng_bundle(travel_bundle_with_roll_below(0.5));
        state.roll_daily_illness();
        assert_eq!(state.illness_days_remaining, 1);
        assert!(state.rest_requested);

        // Cooldown prevents new illness.
        state.disease_cooldown = 2;
        state.illness_days_remaining = 0;
        state.roll_daily_illness();
        assert_eq!(state.disease_cooldown, 1);

        // Fresh illness triggered by RNG when cooldown expired.
        state.disease_cooldown = 0;
        state.starvation_days = 2;
        state.stats.hp = 3;
        state.stats.supplies = 0;
        state.attach_rng_bundle(travel_bundle_with_roll_below(0.05));
        state.roll_daily_illness();
        assert!(state.illness_days_remaining > 0);
        assert!(state.logs.iter().any(|log| log == LOG_DISEASE_HIT));

        // Ally attrition path exercises positive case.
        state.stats.allies = 2;
        state.attach_rng_bundle(encounter_bundle_with_roll_below(
            ALLY_ATTRITION_CHANCE + 0.05,
        ));
        state.tick_ally_attrition();
        assert!(state.stats.allies <= 1);

        // Exec order branch when current order is active and resolves.
        state.current_order = Some(ExecOrder::Shutdown);
        state.exec_order_days_remaining = 1;
        state.exec_order_cooldown = 0;
        state.attach_rng_bundle(encounter_bundle_with_roll_below(
            EXEC_ORDER_DAILY_CHANCE + 0.05,
        ));
        state.tick_exec_order_state();
        assert!(state.exec_order_cooldown > 0 || state.current_order.is_none());

        // No current order: force issuing a new one via deterministic RNG.
        state.current_order = None;
        state.exec_order_cooldown = 0;
        state.attach_rng_bundle(encounter_bundle_with_roll_below(
            EXEC_ORDER_DAILY_CHANCE + 0.05,
        ));
        state.tick_exec_order_state();
        assert!(state.current_order.is_some() || !state.logs.is_empty());

        // Exercise every exec order effect explicitly to ensure match branches run.
        for &order in ExecOrder::ALL {
            state.exec_travel_multiplier = 10.0;
            state.exec_breakdown_bonus = 10.0;
            state.inventory.tags.clear();
            state.apply_exec_order_effects(order);
        }

        // travel_ratio_recent edge cases
        assert_eq!(state.travel_ratio_recent(0), 1.0);
        state.recent_travel_days.clear();
        assert_eq!(state.travel_ratio_recent(5), WEATHER_DEFAULT_SPEED);
        state.recent_travel_days.push_back(TravelDayKind::Travel);
        for _ in 0..6 {
            state.recent_travel_days.push_back(TravelDayKind::NonTravel);
        }
        assert!(state.travel_ratio_recent(5) < 1.0);
    }

    #[test]
    #[allow(clippy::float_cmp, clippy::too_many_lines)]
    fn field_repair_and_travel_credit_paths() {
        let mut state = GameState::default();
        state.data = Some(EncounterData::empty());
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.vehicle.health = 10.0;
        state.vehicle.wear = 40.0;
        state.miles_traveled_actual = 1_960.0;
        state.features.travel_v2 = false;
        state.distance_today = 4.0;
        state.partial_distance_today = 2.0;
        state.budget_cents = 20_000;
        state.budget = 200;
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));

        // Partial credit resets when already traveled.
        state.traveled_today = true;
        state.partial_traveled_today = false;
        state.apply_partial_travel_credit(5.0, "log.partial", "reason");
        assert!(state.logs.iter().any(|log| log == "log.partial"));

        // Rest travel credit path helper.
        state.logs.clear();
        state.features.travel_v2 = true;
        state.apply_rest_travel_credit();
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_REST_CREDIT));

        // Classic field repair guard exercises both credit and zero-distance branch.
        state.features.travel_v2 = false;
        state.distance_today = 0.0;
        state.partial_distance_today = 0.0;
        state.apply_classic_field_repair_guard();
        assert!(
            state
                .logs
                .iter()
                .any(|log| log == LOG_VEHICLE_FIELD_REPAIR_GUARD)
        );

        // Emergency limp guard triggers once conditions are satisfied.
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.miles_traveled_actual = 1_951.0;
        state.distance_today = 5.0;
        let limp_triggered = state.try_emergency_limp_guard();
        assert!(limp_triggered);

        // Deep aggressive field repair path (multiple calls ensure RNG and cooldown).
        state.miles_traveled_actual = 1_700.0;
        state.attach_rng_bundle(breakdown_bundle_with_roll_below(0.1));
        let deep_repair = state.try_deep_aggressive_field_repair();
        assert!(deep_repair);

        // Reset progress and ensure revert functions execute.
        state.prev_miles_traveled = state.miles_traveled_actual - 10.0;
        state.reset_today_progress();
        state.recent_travel_days.clear();
        for _ in 0..6 {
            state.recent_travel_days.push_back(TravelDayKind::NonTravel);
        }
        state.enforce_aggressive_delay_cap(0.0);
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_PARTIAL));

        // Delay travel credit branch.
        state.logs.clear();
        state.apply_delay_travel_credit("delay_test");
        assert!(state.logs.iter().any(|log| log == LOG_TRAVEL_DELAY_CREDIT));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn deep_aggressive_safeguards_and_compose() {
        let mut state = GameState::default();
        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Aggressive);
        state.miles_traveled_actual = 1_950.0;
        state.day = 220;
        state.stats.sanity = 0;
        state.stats.pants = 30;
        state.budget_cents = 10_000;
        state.budget = 100;
        state.current_day_kind = None;

        state.apply_deep_aggressive_sanity_guard();
        assert!(state.deep_aggressive_sanity_guard_used);
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
        let mut state = GameState::default();
        state.data = Some(EncounterData::empty());
        state.mode = GameMode::Classic;
        state.pace = PaceId::Blitz;
        state.features.travel_v2 = false;
        state.weather_travel_multiplier = 0.5;
        let mut limits = crate::pacing::PacingLimits::default();
        limits.distance_base = 30.0;
        let mut pace = crate::pacing::PaceCfg::default();
        pace.distance = 0.0;
        pace.dist_mult = 0.0;
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
        assert_ne!(classic, v2);
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
        assert_eq!(format!("{}", PolicyKind::MonteCarlo), "monte_carlo");

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
        #![allow(clippy::field_reassign_with_default)]
        // Early return when already finalized.
        let mut early = GameState::default();
        early.encounter_history = VecDeque::from(vec![0]);
        early.did_end_of_day = true;
        early.end_of_day();
        assert!(early.did_end_of_day);

        // No travel paths ensure assertion branch executes without panic.
        let mut stagnant = GameState::default();
        stagnant.encounter_history = VecDeque::from(vec![0]);
        stagnant.prev_miles_traveled = 10.0;
        stagnant.miles_traveled_actual = 10.0;
        stagnant.traveled_today = false;
        stagnant.partial_traveled_today = false;
        stagnant.current_day_kind = Some(TravelDayKind::NonTravel);
        stagnant.end_of_day();
        assert!(stagnant.did_end_of_day);
        assert_eq!(stagnant.recent_travel_days.len(), 1);

        // Deep conservative branch applies travel bonus and rotation enforcement.
        let mut conservative = GameState::default();
        conservative.encounter_history = VecDeque::from(vec![0]);
        conservative.mode = GameMode::Deep;
        conservative.policy = Some(PolicyKind::Conservative);
        conservative.start_of_day();
        conservative.encounters_today = 1;
        conservative.prev_miles_traveled = 100.0;
        conservative.miles_traveled_actual = 105.0;
        conservative.current_day_kind = Some(TravelDayKind::Travel);
        conservative.current_day_miles = 3.0;
        conservative.distance_today = 2.0;
        conservative.distance_today_raw = 2.5;
        conservative.partial_distance_today = 1.5;
        conservative.traveled_today = true;
        conservative.distance_cap_today = 6.0;
        conservative.current_day_reason_tags = vec!["progress".into()];
        conservative.rotation_travel_days = conservative.rotation_force_interval();
        conservative.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::Partial; TRAVEL_HISTORY_WINDOW]);
        conservative.end_of_day();
        assert!(conservative.force_rotation_pending);
        assert!(
            conservative
                .day_reason_history
                .last()
                .is_some_and(|entry| entry.contains("progress"))
        );

        // Deep aggressive branch unlocks boss readiness.
        let mut aggressive = GameState::default();
        aggressive.encounter_history = VecDeque::from(vec![0]);
        aggressive.mode = GameMode::Deep;
        aggressive.policy = Some(PolicyKind::Aggressive);
        aggressive.prev_miles_traveled = DEEP_AGGRESSIVE_BOSS_BIAS_MILES - 10.0;
        aggressive.miles_traveled_actual = DEEP_AGGRESSIVE_BOSS_BIAS_MILES + 5.0;
        aggressive.traveled_today = true;
        aggressive.distance_today = 5.0;
        aggressive.distance_today_raw = 5.0;
        aggressive.current_day_miles = 5.0;
        aggressive.current_day_reason_tags = vec!["march".into()];
        aggressive.end_of_day();
        assert!(aggressive.boss_ready);
        assert!(aggressive.boss_reached);
    }

    #[test]
    fn state_helper_methods_cover_remaining_paths() {
        #![allow(clippy::field_reassign_with_default)]
        let mut state = GameState::default();
        state.encounter_history = VecDeque::from(vec![0]);
        state.record_encounter("alpha");
        assert_eq!(state.encounters_today, 1);
        assert!(
            state
                .recent_encounters
                .iter()
                .any(|entry| entry.id == "alpha")
        );

        state.current_day_kind = Some(TravelDayKind::Travel);
        state.current_day_reason_tags = vec!["camp".into(), "repair".into()];
        state.travel_days = 1;
        state.partial_travel_days = 1;
        state.non_travel_days = 1;
        state.days_with_camp = 1;
        state.days_with_repair = 1;
        state.rotation_travel_days = 2;
        state.revert_current_day_record();
        assert!(state.current_day_reason_tags.is_empty());

        state.apply_travel_progress(5.0, TravelProgressKind::Partial);
        assert!(state.partial_traveled_today);

        assert!(state.rotation_force_interval() >= 3);
        state.recent_travel_days = VecDeque::from(vec![
            TravelDayKind::Travel,
            TravelDayKind::Partial,
            TravelDayKind::NonTravel,
        ]);
        assert!(state.travel_ratio_recent(3) < 1.0);

        state.traveled_today = true;
        state.partial_traveled_today = false;
        state.apply_partial_travel_credit(1.0, "log.partial.credit", "delay");
        assert!(state.logs.iter().any(|entry| entry == "log.partial.credit"));

        state.mode = GameMode::Classic;
        state.budget_cents = 5_000;
        state.budget = 50;
        state.vehicle.wear = 40.0;
        state.breakdown = Some(Breakdown {
            part: Part::Battery,
            day_started: 1,
        });
        state.travel_blocked = true;
        state.apply_classic_field_repair_guard();
        assert!(!state.travel_blocked);

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

        assert_eq!(state.vehicle_health(), state.vehicle.health);

        state.stats.supplies = 10;
        state.starvation_days = 2;
        state.apply_starvation_tick();
        assert_eq!(state.starvation_days, 0);

        state.stats.allies = 2;
        state.logs.clear();
        state.attach_rng_bundle(encounter_bundle_with_roll_below(
            ALLY_ATTRITION_CHANCE * 2.0,
        ));
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
        state.deep_aggressive_sanity_guard_used = false;
        state.apply_deep_aggressive_sanity_guard();
        assert!(state.deep_aggressive_sanity_guard_used);

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
    #[allow(clippy::too_many_lines)]
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

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub mode: GameMode,
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
    #[serde(default)]
    pub rest_requested: bool,
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
    pub deep_aggressive_sanity_guard_used: bool,
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
    pub endgame: EndgameState,
    #[serde(default)]
    pub rotation_travel_days: u32,
    #[serde(default)]
    pub force_rotation_pending: bool,
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
    pub current_day_record: Option<DayRecord>,
    #[serde(skip)]
    pub current_day_kind: Option<TravelDayKind>,
    #[serde(skip)]
    pub current_day_reason_tags: Vec<String>,
    #[serde(skip)]
    pub current_day_miles: f32,
    #[serde(skip)]
    pub suppress_stop_ratio: bool,
    #[serde(skip)]
    pub last_breakdown_part: Option<Part>,
}

impl Default for GameState {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        Self {
            mode: GameMode::Classic,
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
            rest_requested: false,
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
            deep_aggressive_sanity_guard_used: false,
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
            endgame: EndgameState::default(),
            rotation_travel_days: 0,
            force_rotation_pending: false,
            policy: None,
            recent_travel_days: VecDeque::with_capacity(TRAVEL_HISTORY_WINDOW),
            day_reason_history: Vec::new(),
            rotation_backlog: VecDeque::new(),
            rng_bundle: None,
            data: None,
            last_damage: None,
            current_day_record: None,
            current_day_kind: None,
            current_day_reason_tags: Vec::new(),
            current_day_miles: 0.0,
            suppress_stop_ratio: false,
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

    fn travel_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.travel())
    }

    fn breakdown_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.breakdown())
    }

    fn encounter_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.encounter())
    }

    fn crossing_rng(&self) -> Option<RefMut<'_, CountingRng<SmallRng>>> {
        self.rng_bundle.as_ref().map(|bundle| bundle.crossing())
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
        if self.day_initialized {
            return;
        }
        self.day_initialized = true;
        self.did_end_of_day = false;
        self.traveled_today = false;
        self.partial_traveled_today = false;
        self.encounters_today = 0;
        self.encounter_occurred_today = false;
        self.prev_miles_traveled = self.miles_traveled_actual;
        self.current_day_kind = None;
        self.current_day_reason_tags.clear();
        self.current_day_miles = 0.0;
        let day_index = u16::try_from(self.day.saturating_sub(1)).unwrap_or(u16::MAX);
        self.current_day_record = Some(DayRecord::new(day_index, TravelDayKind::NonTravel, 0.0));
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

        if self.encounter_cooldown > 0 {
            self.encounter_cooldown -= 1;
        }

        self.tick_exec_order_state();

        self.apply_starvation_tick();
        self.roll_daily_illness();
        self.apply_deep_aggressive_sanity_guard();
        let weather_cfg = WeatherConfig::default_config();
        let weather_rng = self.rng_bundle.as_ref().map(Rc::clone);
        crate::weather::process_daily_weather(self, &weather_cfg, weather_rng.as_deref());
        self.stats.clamp();

        if !self.features.travel_v2 {
            self.apply_travel_wear_scaled(1.0);
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
                let cooldown = self
                    .encounter_rng()
                    .map_or(EXEC_ORDER_MIN_COOLDOWN, |mut rng| {
                        rng.random_range(EXEC_ORDER_MIN_COOLDOWN..=EXEC_ORDER_MAX_COOLDOWN)
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

        let next_order = if let Some(mut rng) = self.encounter_rng()
            && rng.random::<f32>() < exec_chance
        {
            let idx = rng.random_range(0..ExecOrder::ALL.len());
            let order = ExecOrder::ALL[idx];
            let duration = rng.random_range(EXEC_ORDER_MIN_DURATION..=EXEC_ORDER_MAX_DURATION);
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
        match order {
            ExecOrder::Shutdown => {
                self.stats.morale -= 1;
                self.stats.supplies = (self.stats.supplies - 1).max(0);
            }
            ExecOrder::TravelBanLite => {
                self.stats.sanity -= 1;
                self.exec_travel_multiplier *= EXEC_ORDER_SPEED_BONUS;
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
                self.exec_breakdown_bonus += EXEC_ORDER_BREAKDOWN_BONUS;
            }
        }
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

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub(crate) fn end_of_day(&mut self) {
        if self.did_end_of_day {
            return;
        }
        if let Some(back) = self.encounter_history.back_mut() {
            *back = self.encounters_today;
        }
        let computed_miles_today = self.distance_today.max(self.distance_today_raw);
        self.enforce_aggressive_delay_cap(computed_miles_today);
        let miles_delta = (self.miles_traveled_actual - self.prev_miles_traveled).max(0.0);
        if matches!(self.current_day_kind, None | Some(TravelDayKind::NonTravel)) {
            if miles_delta > 0.0 {
                self.partial_traveled_today = true;
            }
            let fallback_kind = if self.traveled_today {
                TravelDayKind::Travel
            } else if self.partial_traveled_today {
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
        if !self.traveled_today && !self.partial_traveled_today {
            assert!(
                miles_delta <= 0.01,
                "distance advanced on non-travel day (delta {miles_delta:.2})"
            );
        }
        if self.partial_traveled_today {
            debug_assert!(
                (self.miles_traveled_actual - self.prev_miles_traveled) > 0.0,
                "partial travel day without distance gain"
            );
        }

        if self.mode.is_deep()
            && matches!(self.policy, Some(PolicyKind::Conservative))
            && self.current_day_miles > 0.0
        {
            let had_repair = self
                .current_day_reason_tags
                .iter()
                .any(|tag| tag.contains("repair"));
            let had_crossing = self
                .current_day_reason_tags
                .iter()
                .any(|tag| tag.starts_with("crossing") || tag == "detour");
            if !had_repair && !had_crossing {
                let bonus = self.current_day_miles * 0.03;
                if bonus > 0.0 {
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
                        self.apply_travel_progress(applied, TravelProgressKind::Full);
                        self.current_day_miles += applied;
                        self.distance_today = self.distance_today.max(self.current_day_miles);
                        self.distance_today_raw =
                            self.distance_today_raw.max(self.current_day_miles);
                        self.partial_distance_today = self
                            .partial_distance_today
                            .max(applied)
                            .min(self.distance_today);
                    }
                }
            }
        }

        let mut day_kind = self.current_day_kind.unwrap_or(if self.traveled_today {
            TravelDayKind::Travel
        } else if self.partial_traveled_today {
            TravelDayKind::Partial
        } else {
            TravelDayKind::NonTravel
        });
        if matches!(day_kind, TravelDayKind::NonTravel) && !self.suppress_stop_ratio {
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
        if self.rotation_travel_days >= self.rotation_force_interval() {
            self.force_rotation_pending = true;
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
        self.suppress_stop_ratio = false;
        self.did_end_of_day = true;
        if self.mode.is_deep()
            && matches!(self.policy, Some(PolicyKind::Aggressive))
            && !self.boss_ready
            && !self.boss_attempted
            && self.miles_traveled_actual >= DEEP_AGGRESSIVE_BOSS_BIAS_MILES
        {
            self.boss_ready = true;
            self.boss_reached = true;
        }
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

    fn apply_travel_wear(&mut self) {
        self.apply_travel_wear_scaled(1.0);
    }

    fn revert_current_day_record(&mut self) {
        if let Some(kind) = self.current_day_kind.take() {
            match kind {
                TravelDayKind::Travel => {
                    self.travel_days = self.travel_days.saturating_sub(1);
                    self.rotation_travel_days = self.rotation_travel_days.saturating_sub(1);
                }
                TravelDayKind::Partial => {
                    self.partial_travel_days = self.partial_travel_days.saturating_sub(1);
                    self.rotation_travel_days = self.rotation_travel_days.saturating_sub(1);
                }
                TravelDayKind::NonTravel => {
                    self.non_travel_days = self.non_travel_days.saturating_sub(1);
                }
            }
        }
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

    pub(crate) fn apply_travel_progress(&mut self, distance: f32, kind: TravelProgressKind) {
        if distance <= 0.0 {
            return;
        }
        let before = self.miles_traveled_actual;
        self.miles_traveled_actual += distance;
        self.miles_traveled = (self.miles_traveled + distance).min(self.trail_distance);
        let advanced = self.miles_traveled_actual > before;
        if advanced {
            match kind {
                TravelProgressKind::Full => self.traveled_today = true,
                TravelProgressKind::Partial => self.partial_traveled_today = true,
            }
            if self.ending.is_none() && self.miles_traveled_actual >= self.trail_distance {
                self.boss_ready = true;
                self.boss_reached = true;
            }
        }
    }

    fn recompute_day_counters(&mut self) {
        let mut travel = 0_u32;
        let mut partial = 0_u32;
        let mut non = 0_u32;
        for record in &self.day_records {
            match record.kind {
                TravelDayKind::Travel => travel = travel.saturating_add(1),
                TravelDayKind::Partial => partial = partial.saturating_add(1),
                TravelDayKind::NonTravel => non = non.saturating_add(1),
            }
        }
        self.travel_days = travel;
        self.partial_travel_days = partial;
        self.non_travel_days = non;
    }

    pub(crate) fn reset_today_progress(&mut self) {
        let day_progress = (self.miles_traveled_actual - self.prev_miles_traveled).max(0.0);
        if day_progress > 0.0 {
            self.miles_traveled_actual -= day_progress;
            self.miles_traveled = self.miles_traveled_actual.min(self.trail_distance);
            if self.miles_traveled_actual < self.trail_distance {
                self.boss_ready = false;
                self.boss_reached = false;
            }
        }
        self.revert_current_day_record();
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

    fn enforce_aggressive_delay_cap(&mut self, computed_miles: f32) {
        if self.traveled_today || self.partial_traveled_today {
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
        self.partial_traveled_today = true;
        self.traveled_today = false;
        let new_wear = (self.vehicle.wear - self.journey_wear.base).max(0.0);
        self.vehicle.set_wear(new_wear);
        self.logs.push(String::from(LOG_TRAVEL_PARTIAL));
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

    fn apply_partial_travel_credit(
        &mut self,
        distance: f32,
        log_key: &'static str,
        reason_tag: &str,
    ) {
        if distance <= 0.0 {
            return;
        }
        if self.traveled_today && !self.partial_traveled_today {
            self.reset_today_progress();
        }
        self.distance_today += distance;
        self.distance_today_raw += distance;
        self.partial_distance_today = self.partial_distance_today.max(distance);
        self.record_travel_day(TravelDayKind::Partial, distance, reason_tag);
        self.logs.push(String::from(log_key));
    }

    pub(crate) fn apply_rest_travel_credit(&mut self) {
        self.apply_partial_travel_credit(REST_TRAVEL_CREDIT_MILES, LOG_TRAVEL_REST_CREDIT, "camp");
    }

    fn apply_delay_travel_credit(&mut self, reason_tag: &str) {
        self.apply_partial_travel_credit(
            DELAY_TRAVEL_CREDIT_MILES,
            LOG_TRAVEL_DELAY_CREDIT,
            reason_tag,
        );
    }

    fn apply_classic_field_repair_guard(&mut self) {
        let partial = day_accounting::partial_day_miles(self, 0.0);
        if partial > 0.0 {
            self.apply_partial_travel_credit(
                partial,
                LOG_VEHICLE_FIELD_REPAIR_GUARD,
                "field_repair_guard",
            );
        } else {
            self.record_travel_day(TravelDayKind::Partial, 0.0, "field_repair_guard");
            self.logs.push(String::from(LOG_VEHICLE_FIELD_REPAIR_GUARD));
        }
        self.vehicle.ensure_health_floor(VEHICLE_EMERGENCY_HEAL);
        self.vehicle.wear = (self.vehicle.wear - CLASSIC_FIELD_REPAIR_WEAR_REDUCTION).max(0.0);
        let field_repair_cost = CLASSIC_FIELD_REPAIR_COST_CENTS;
        let paid = field_repair_cost.min(self.budget_cents.max(0));
        self.budget_cents -= paid;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += paid;
        self.breakdown = None;
        self.travel_blocked = false;
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

        let partial = day_accounting::partial_day_miles(self, 0.0);
        if partial > 0.0 {
            self.apply_partial_travel_credit(partial, LOG_VEHICLE_EMERGENCY_LIMP, "emergency_limp");
        } else {
            self.record_travel_day(TravelDayKind::Partial, 0.0, "emergency_limp");
            self.logs.push(String::from(LOG_VEHICLE_EMERGENCY_LIMP));
        }
        self.vehicle.ensure_health_floor(VEHICLE_EMERGENCY_HEAL);
        self.vehicle.wear = (self.vehicle.wear - EMERGENCY_LIMP_WEAR_REDUCTION).max(0.0);
        let limp_cost = EMERGENCY_LIMP_REPAIR_COST_CENTS;
        let paid = limp_cost.min(self.budget_cents.max(0));
        self.budget_cents -= paid;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += paid;
        self.endgame.last_limp_mile = self.miles_traveled_actual;
        self.breakdown = None;
        self.travel_blocked = false;
        self.last_breakdown_part = None;
        true
    }

    fn try_deep_aggressive_field_repair(&mut self) -> bool {
        if !(self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Aggressive))) {
            return false;
        }
        if self.miles_traveled_actual < 1_600.0 {
            return false;
        }
        let roll = self
            .breakdown_rng()
            .map_or(1.0, |mut rng| rng.random::<f32>());
        if roll >= 0.15 {
            return false;
        }

        let partial = day_accounting::partial_day_miles(self, 0.0);
        if partial > 0.0 {
            self.apply_partial_travel_credit(
                partial,
                LOG_DEEP_AGGRESSIVE_FIELD_REPAIR,
                "field_repair",
            );
        } else {
            self.record_travel_day(TravelDayKind::Partial, 0.0, "field_repair");
            self.logs
                .push(String::from(LOG_DEEP_AGGRESSIVE_FIELD_REPAIR));
        }
        self.vehicle.ensure_health_floor(VEHICLE_EMERGENCY_HEAL);
        self.vehicle.wear = (self.vehicle.wear - EMERGENCY_LIMP_WEAR_REDUCTION).max(0.0);
        let limp_cost = EMERGENCY_LIMP_REPAIR_COST_CENTS;
        let paid = limp_cost.min(self.budget_cents.max(0));
        self.budget_cents -= paid;
        self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
        self.repairs_spent_cents += paid;
        self.breakdown = None;
        self.travel_blocked = false;
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

        let roll = self.travel_rng().map_or(1.0, |mut rng| rng.random::<f32>());
        if roll >= chance {
            return;
        }

        let duration = self
            .travel_rng()
            .map_or(DISEASE_DURATION_RANGE.0, |mut rng| {
                rng.random_range(DISEASE_DURATION_RANGE.0..=DISEASE_DURATION_RANGE.1)
            });
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
        let trigger = self
            .encounter_rng()
            .is_some_and(|mut rng| rng.random::<f32>() <= ALLY_ATTRITION_CHANCE);
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
        if self.deep_aggressive_sanity_guard_used {
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
        self.deep_aggressive_sanity_guard_used = true;
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
            self.stats.pants = (self.stats.pants - BOSS_COMPOSE_PANTS_SUPPLY).max(0);
            self.logs.push(String::from(LOG_BOSS_COMPOSE_SUPPLIES));
            applied = true;
        } else if self.budget_cents >= BOSS_COMPOSE_FUNDS_COST {
            self.budget_cents -= BOSS_COMPOSE_FUNDS_COST;
            self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
            self.stats.sanity += SANITY_POINT_REWARD;
            self.stats.pants = (self.stats.pants - BOSS_COMPOSE_FUNDS_PANTS).max(0);
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
    #[allow(clippy::too_many_lines)]
    fn compute_miles_for_today(
        &mut self,
        pace_cfg: &crate::pacing::PaceCfg,
        limits: &crate::pacing::PacingLimits,
    ) -> f32 {
        let travel_v2 = self.features.travel_v2;
        let travel_cfg = &self.journey_travel;

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
        let pace_scalar = (pace_policy * pace_cfg_scalar).max(TRAVEL_CONFIG_MIN_MULTIPLIER);

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
        let mut weather_scalar = policy_weather * runtime_weather;

        let penalty_floor = if travel_v2 {
            if limits.distance_penalty_floor > 0.0 {
                limits.distance_penalty_floor
            } else {
                TRAVEL_V2_PENALTY_FLOOR
            }
        } else {
            TRAVEL_CLASSIC_PENALTY_FLOOR
        };

        weather_scalar = weather_scalar.max(TRAVEL_CONFIG_MIN_MULTIPLIER);

        let mut multiplier = (pace_scalar * weather_scalar).max(penalty_floor);
        if self.mode.is_deep() && matches!(self.policy, Some(PolicyKind::Balanced)) {
            multiplier *= DEEP_BALANCED_TRAVEL_NUDGE;
        }
        let behind_boost = self.behind_schedule_multiplier();
        if behind_boost > 1.0 {
            multiplier *= behind_boost;
        }

        let mut raw_distance = travel_cfg.mpd_base * multiplier;
        let mut distance = raw_distance;
        let ratio = self.journey_partial_ratio.clamp(0.0, 1.0);
        let mut partial_distance = raw_distance * ratio;

        let travel_boost =
            self.deep_conservative_travel_boost() * self.deep_aggressive_reach_boost();
        if travel_boost > 1.0 {
            raw_distance *= travel_boost;
            distance *= travel_boost;
            partial_distance *= travel_boost;
        }

        if self.vehicle.health <= VEHICLE_CRITICAL_THRESHOLD {
            distance *= VEHICLE_CRITICAL_SPEED_FACTOR;
            partial_distance *= VEHICLE_CRITICAL_SPEED_FACTOR;
        }

        if self.malnutrition_level > 0 {
            #[allow(clippy::cast_precision_loss)]
            let malnutrition = self.malnutrition_level as f32;
            let starvation_penalty = malnutrition
                .mul_add(-VEHICLE_MALNUTRITION_PENALTY_PER_STACK, 1.0)
                .max(VEHICLE_MALNUTRITION_MIN_FACTOR);
            distance *= starvation_penalty;
            partial_distance *= starvation_penalty;
        }

        distance *= self.exec_travel_multiplier;
        partial_distance *= self.exec_travel_multiplier;
        distance *= self.illness_travel_penalty.max(0.0);
        partial_distance *= self.illness_travel_penalty.max(0.0);

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

    fn apply_target_travel(&mut self, kind: TravelDayKind, target_miles: f32, reason_tag: &str) {
        let tolerance = 0.0001;
        let target = target_miles.max(0.0);
        if target + tolerance < self.current_day_miles {
            self.reset_today_progress();
        }
        let delta = (target - self.current_day_miles).max(0.0);
        self.record_travel_day(kind, delta, reason_tag);
    }

    #[allow(clippy::too_many_lines)]
    fn handle_crossing_event(&mut self, computed_miles_today: f32) -> Option<(bool, String)> {
        let next_idx = usize::try_from(self.crossings_completed).unwrap_or(usize::MAX);
        let &milestone = CROSSING_MILESTONES.get(next_idx)?;
        if self.miles_traveled_actual + f32::EPSILON < milestone {
            return None;
        }

        let kind = self.crossing_kind_for_index(next_idx);
        let cfg = CrossingConfig::default();
        let has_permit = crossings::can_use_permit(self, &kind);
        let bribe_offered = !has_permit && crossings::can_afford_bribe(self, &cfg, kind);

        let ctx = CrossingContext {
            policy: &self.journey_crossing,
            kind,
            has_permit,
            bribe_intent: bribe_offered,
            prior_bribe_attempts: self.crossing_bribe_attempts,
        };
        let resolved = if let Some(mut rng) = self.crossing_rng() {
            crossings::resolve_crossing(ctx, &mut *rng)
        } else {
            let seed_mix =
                self.seed ^ (u64::try_from(next_idx).unwrap_or(0) << 32) ^ u64::from(self.day);
            let mut fallback = SmallRng::seed_from_u64(seed_mix);
            crossings::resolve_crossing(ctx, &mut fallback)
        };

        let mut telemetry = CrossingTelemetry::new(self.day, self.region, self.season, kind);
        telemetry.permit_used = resolved.used_permit;
        telemetry.bribe_attempted = resolved.bribe_attempted;
        if resolved.bribe_attempted {
            telemetry.bribe_success = Some(resolved.bribe_succeeded);
        }

        if resolved.used_permit {
            self.logs.push(String::from(LOG_CROSSING_DECISION_PERMIT));
            let permit_log = crossings::apply_permit(self, &cfg, kind);
            self.logs.push(permit_log);
            self.crossing_permit_uses = self.crossing_permit_uses.saturating_add(1);
        }

        if resolved.bribe_attempted {
            self.logs.push(String::from(LOG_CROSSING_DECISION_BRIBE));
            let _ = crossings::apply_bribe(self, &cfg, kind);
            self.crossing_bribe_attempts = self.crossing_bribe_attempts.saturating_add(1);
            if resolved.bribe_succeeded {
                self.crossing_bribe_successes = self.crossing_bribe_successes.saturating_add(1);
            }
            let log_key = if resolved.bribe_succeeded {
                "crossing.result.bribe.success"
            } else {
                "crossing.result.bribe.fail"
            };
            self.logs.push(log_key.to_string());
        }

        match resolved.result {
            crossings::CrossingResult::Pass => {
                if resolved.bribe_attempted {
                    telemetry.bribe_success = Some(resolved.bribe_succeeded);
                }
                telemetry.outcome = CrossingOutcomeTelemetry::Passed;
                self.logs.push(String::from(LOG_CROSSING_PASSED));
                self.crossings_completed = self.crossings_completed.saturating_add(1);
                let target_miles = day_accounting::partial_day_miles(self, computed_miles_today);
                self.apply_target_travel(TravelDayKind::Partial, target_miles, "crossing_pass");
                self.stats.clamp();
                self.crossing_events.push(telemetry);
                self.end_of_day();
                Some((false, String::from(LOG_CROSSING_PASSED)))
            }
            crossings::CrossingResult::Detour(days) => {
                if resolved.bribe_attempted {
                    telemetry.bribe_success = Some(false);
                }
                telemetry.detour_taken = true;
                telemetry.detour_days = Some(u32::from(days));
                telemetry.outcome = CrossingOutcomeTelemetry::Detoured;
                self.crossing_detours_taken = self.crossing_detours_taken.saturating_add(1);
                let per_day_miles = day_accounting::partial_day_miles(self, computed_miles_today);
                self.logs.push(String::from(LOG_CROSSING_DETOUR));
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
                Some((false, String::from(LOG_CROSSING_DETOUR)))
            }
            crossings::CrossingResult::TerminalFail => {
                if resolved.bribe_attempted {
                    telemetry.bribe_success = Some(false);
                }
                telemetry.outcome = CrossingOutcomeTelemetry::Failed;
                self.crossing_failures = self.crossing_failures.saturating_add(1);
                self.logs.push(String::from(LOG_CROSSING_FAILURE));
                self.reset_today_progress();
                self.record_travel_day(TravelDayKind::NonTravel, 0.0, "crossing_fail");
                self.stats.clamp();
                self.set_ending(Ending::Collapse {
                    cause: CollapseCause::Crossing,
                });
                self.crossing_events.push(telemetry);
                self.end_of_day();
                Some((true, String::from(LOG_CROSSING_FAILURE)))
            }
        }
    }

    #[must_use]
    pub fn with_seed(mut self, seed: u64, mode: GameMode, data: EncounterData) -> Self {
        self.mode = mode;
        self.seed = seed;
        self.state_version = Self::current_version();
        self.day_records.clear();
        self.recompute_day_counters();
        self.current_day_record = None;
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
        self.data = Some(data);
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
    pub const fn region_by_day(day: u32) -> Region {
        match day {
            0..=4 => Region::Heartland,
            5..=9 => Region::RustBelt,
            _ => Region::Beltway,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn travel_next_leg(&mut self, endgame_cfg: &EndgameTravelCfg) -> (bool, String, bool) {
        self.start_of_day();

        let rng_bundle = self.rng_bundle.as_ref().map(Rc::clone);

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
            if !self.partial_traveled_today {
                self.apply_delay_travel_credit("repair");
            }
            self.end_of_day();
            return (false, String::from(LOG_TRAVEL_BLOCKED), breakdown_started);
        }

        let mut trigger_encounter = false;
        if self.encounter_occurred_today {
            // Already had an encounter; keep trigger false.
        } else if let Some(bundle) = rng_bundle.as_ref() {
            let roll = {
                let mut rng = bundle.encounter();
                rng.random::<f32>()
            };
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
            let mut rotation_backlog = std::mem::take(&mut self.rotation_backlog);
            let mut encounter = None;
            let mut force_rotation_pending = self.force_rotation_pending;
            let mut rotation_logged = false;
            if let (Some(bundle), Some(data)) = (rng_bundle.as_ref(), self.data.as_ref()) {
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
                    let (pick, satisfied) =
                        pick_encounter(&request, &mut rotation_backlog, &mut *rng);
                    if forced {
                        if satisfied {
                            rotation_logged = true;
                        }
                        force_rotation_pending = !rotation_backlog.is_empty();
                    }
                    encounter = pick;
                }
            }

            let should_reroll = encounter.as_ref().is_some_and(|enc| {
                self.features.encounter_diversity && self.should_discourage_encounter(&enc.id)
            });

            if should_reroll {
                let reroll_penalty = self.encounter_reroll_penalty();
                let reroll_trigger = rng_bundle.as_ref().is_some_and(|bundle| {
                    let mut rng = bundle.encounter();
                    rng.random::<f32>() < reroll_penalty
                });
                if reroll_trigger
                    && let (Some(bundle), Some(data)) = (rng_bundle.as_ref(), self.data.as_ref())
                {
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
                    {
                        let mut rng = bundle.encounter();
                        let (replacement, satisfied) =
                            pick_encounter(&request, &mut rotation_backlog, &mut *rng);
                        if satisfied {
                            force_rotation_pending = false;
                        }
                        encounter = replacement;
                    }
                }
            }

            if rotation_logged {
                self.logs.push(String::from(LOG_ENCOUNTER_ROTATION));
            }
            self.force_rotation_pending = force_rotation_pending;
            self.rotation_backlog = rotation_backlog;

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
                    self.apply_travel_wear_scaled(wear_scale);
                    self.logs.push(String::from(LOG_TRAVEL_PARTIAL));
                }
                if is_major_repair {
                    self.record_travel_day(TravelDayKind::NonTravel, 0.0, "repair");
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

        let computed_miles_today = self.distance_today.max(self.distance_today_raw);
        if let Some((ended, log)) = self.handle_crossing_event(computed_miles_today) {
            return (ended, log, breakdown_started);
        }

        let additional_miles = (self.distance_today - self.current_day_miles).max(0.0);
        self.record_travel_day(TravelDayKind::Travel, additional_miles, "");
        endgame::run_endgame_controller(self, computed_miles_today, breakdown_started, endgame_cfg);

        if debug_log_enabled() {
            println!(
                "Day {}: distance {:.1}/{:.1} (actual {:.1}), boss_ready {}, HP {}, Sanity {}",
                self.day,
                self.miles_traveled,
                self.trail_distance,
                self.miles_traveled_actual,
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

        let roll = self
            .breakdown_rng()
            .map_or(1.0, |mut rng| rng.random::<f32>());
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
                part, self.vehicle.health, roll, breakdown_chance
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
                self.last_breakdown_part = None;
                self.logs.push(String::from("log.breakdown-repaired"));
                return;
            }

            if self.total_spares() == 0 && self.budget_cents >= EMERGENCY_REPAIR_COST {
                self.spend_emergency_repair(LOG_VEHICLE_REPAIR_EMERGENCY);
                self.breakdown = None;
                self.travel_blocked = false;
                self.last_breakdown_part = None;
                return;
            }

            let day_started = u32::try_from(breakdown.day_started).unwrap_or(0);
            if self.day.saturating_sub(day_started) >= 1 {
                self.vehicle
                    .apply_damage(VEHICLE_BREAKDOWN_DAMAGE * VEHICLE_BREAKDOWN_PARTIAL_FACTOR);
                self.mark_damage(DamageCause::Vehicle);
                self.breakdown = None;
                self.travel_blocked = false;
                self.last_breakdown_part = None;
                self.logs.push(String::from("log.breakdown-jury-rigged"));
            } else {
                self.travel_blocked = true;
            }
        } else {
            self.travel_blocked = false;
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
        self.encounter_rng().map_or(0, |mut rng| rng.next_u32())
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

        let pants_floor = limits.pants_floor;
        let pants_ceiling = limits.pants_ceiling;
        let mut pants_value = self.stats.pants;

        if limits.passive_relief != 0 && pants_value >= limits.passive_relief_threshold {
            pants_value = (pants_value + limits.passive_relief).clamp(pants_floor, pants_ceiling);
        }

        if self.mods.pants_relief != 0 && pants_value >= self.mods.pants_relief_threshold {
            pants_value = (pants_value + self.mods.pants_relief).clamp(pants_floor, pants_ceiling);
        }

        let boss_stage = self.boss_ready || self.miles_traveled >= self.trail_distance;
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
        self.logs.push(String::from("log.party.updated"));
    }

    pub const fn request_rest(&mut self) {
        self.rest_requested = true;
    }

    fn failure_log_key(&mut self) -> Option<&'static str> {
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
                self.suppress_stop_ratio = true;
            }
            self.start_of_day();
            self.record_travel_day(kind, miles, reason_tag);
            self.end_of_day();
            self.suppress_stop_ratio = false;
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
}
