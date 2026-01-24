//! Oregon Trail Deluxe (DOS v3.0) mechanical policy defaults.
//!
//! This module is intentionally data-only: it provides extracted constants and
//! policy defaults needed for parity, but it does not implement simulation
//! behavior by itself.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::{
    PACE_BREAKDOWN_BLITZ, PACE_BREAKDOWN_HEATED, PACE_BREAKDOWN_STEADY,
    VEHICLE_BREAKDOWN_BASE_CHANCE, VEHICLE_BREAKDOWN_WEAR_COEFFICIENT,
};
use crate::otdeluxe_state::{OtDeluxeRiver, OtDeluxeRiverBed};
use crate::state::{Region, Season};
use crate::weather::{Weather, WeatherEffects};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxePace {
    Steady,
    Strenuous,
    Grueling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxePaceHealthPolicy {
    pub steady: i32,
    pub strenuous: i32,
    pub grueling: i32,
}

impl Default for OtDeluxePaceHealthPolicy {
    fn default() -> Self {
        Self {
            steady: 0,
            strenuous: 5,
            grueling: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeRations {
    Filling,
    Meager,
    BareBones,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeRationsPolicy {
    pub food_lbs_per_person: [u16; 3],
    pub health_penalty: [i32; 3],
}

impl Default for OtDeluxeRationsPolicy {
    fn default() -> Self {
        Self {
            food_lbs_per_person: [3, 2, 1],
            health_penalty: [0, 5, 10],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeOccupation {
    Banker,
    Doctor,
    Merchant,
    Blacksmith,
    Carpenter,
    Saddlemaker,
    Farmer,
    Teacher,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeTrailVariant {
    Main,
    SubletteCutoff,
    DallesShortcut,
    SubletteAndDallesShortcut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeStoreBasePricesCents {
    pub ox: u32,
    pub clothes_set: u32,
    pub bullet: u32,
    pub ammo_box: u32,
    pub food_lb: u32,
    pub wheel: u32,
    pub axle: u32,
    pub tongue: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeStoreMaxBuy {
    pub oxen: u16,
    pub ammo_boxes: u16,
    pub clothes_sets: u16,
    pub wheels: u16,
    pub axles: u16,
    pub tongues: u16,
    pub food_lbs: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeOccupationSpec {
    pub occupation: OtDeluxeOccupation,
    pub starting_cash_dollars: u16,
    pub final_bonus_mult: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeScorePolicy {
    pub points_per_person_by_health: OtDeluxeScorePointsPerPersonByHealth,
    pub points_wagon: u32,
    pub points_ox: u32,
    pub points_spare_part: u32,
    pub points_clothes: u32,
    pub divisor_bullets: u32,
    pub divisor_food_lbs: u32,
    pub divisor_cash_cents: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeScorePointsPerPersonByHealth {
    pub good: u32,
    pub fair: u32,
    pub poor: u32,
    pub very_poor: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeAfflictionCurvePoint {
    pub health: u16,
    pub probability: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeAfflictionPolicy {
    pub probability_max: f32,
    pub curve_pwl: [OtDeluxeAfflictionCurvePoint; 9],
    #[serde(default)]
    pub weight_illness: u16,
    #[serde(default)]
    pub weight_injury: u16,
    #[serde(default)]
    pub illness_duration_days: u8,
    #[serde(default)]
    pub injury_duration_days: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeHealthPolicy {
    pub recovery_baseline: i32,
    pub death_threshold: u16,
    pub label_ranges: OtDeluxeHealthLabelRanges,
    pub death_imminent_grace_days: u8,
    pub death_imminent_resets_on_recovery_below_threshold: bool,
    #[serde(default)]
    pub weather_penalty: HashMap<Weather, i32>,
    #[serde(default)]
    pub clothing_sets_per_person: u16,
    #[serde(default)]
    pub clothing_penalty_winter: i32,
    #[serde(default)]
    pub affliction_illness_penalty: i32,
    #[serde(default)]
    pub affliction_injury_penalty: i32,
    #[serde(default)]
    pub drought_threshold: f32,
    #[serde(default)]
    pub drought_penalty: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeHealthLabelRanges {
    pub good_max: u16,
    pub fair_max: u16,
    pub poor_max: u16,
    pub very_poor_max: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeCrossingPolicy {
    pub ferry_cost_cents: u32,
    pub guide_cost_clothes_sets: u16,
    pub ferry_min_depth_ft: f32,
    pub float_min_depth_ft: f32,
    pub wet_goods_min_depth_ft: f32,
    pub swamped_min_depth_ft: f32,
    pub drying_cost_days: u8,
    pub crossing_cost_days: u8,
    pub guide_risk_mult: f32,
    pub guide_loss_mult: f32,
    pub ferry_wait_days_min: u8,
    pub ferry_wait_days_max: u8,
    #[serde(default)]
    pub ferry_accident_risk_max: f32,
    #[serde(default)]
    pub outcome_weights: OtDeluxeCrossingOutcomeWeightsByMethod,
    #[serde(default)]
    pub river_profiles: OtDeluxeRiverProfiles,
    #[serde(default)]
    pub seasonal_depth_mult: OtDeluxeSeasonalFactors,
    #[serde(default)]
    pub seasonal_swiftness_mult: OtDeluxeSeasonalFactors,
    #[serde(default)]
    pub rain_accum_max: f32,
    #[serde(default)]
    pub rain_depth_mult: f32,
    #[serde(default)]
    pub rain_width_mult: f32,
    #[serde(default)]
    pub rain_swiftness_mult: f32,
    #[serde(default)]
    pub swiftness_risk_mult: f32,
    #[serde(default)]
    pub shallow_safe_bonus: f32,
    #[serde(default)]
    pub wet_goods_bonus: f32,
    #[serde(default)]
    pub swamped_sank_bonus: f32,
    #[serde(default)]
    pub stuck_muddy_mult: f32,
    #[serde(default)]
    pub tipped_rocky_mult: f32,
    #[serde(default)]
    pub stuck_cost_days: u8,
    #[serde(default)]
    pub tipped_loss_ratio: f32,
    #[serde(default)]
    pub sank_loss_ratio: f32,
    #[serde(default)]
    pub drownings_min: u8,
    #[serde(default)]
    pub drownings_max: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeCrossingOutcomeWeights {
    pub safe: f32,
    pub stuck: f32,
    pub wet: f32,
    pub tipped: f32,
    pub sank: f32,
    pub drowned: f32,
}

impl Default for OtDeluxeCrossingOutcomeWeights {
    fn default() -> Self {
        Self {
            safe: 0.6,
            stuck: 0.1,
            wet: 0.15,
            tipped: 0.1,
            sank: 0.04,
            drowned: 0.01,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeCrossingOutcomeWeightsByMethod {
    pub ford: OtDeluxeCrossingOutcomeWeights,
    pub caulk_float: OtDeluxeCrossingOutcomeWeights,
    pub ferry: OtDeluxeCrossingOutcomeWeights,
}

impl Default for OtDeluxeCrossingOutcomeWeightsByMethod {
    fn default() -> Self {
        Self {
            ford: OtDeluxeCrossingOutcomeWeights::default(),
            caulk_float: OtDeluxeCrossingOutcomeWeights {
                safe: 0.45,
                stuck: 0.1,
                wet: 0.15,
                tipped: 0.15,
                sank: 0.1,
                drowned: 0.05,
            },
            ferry: OtDeluxeCrossingOutcomeWeights {
                safe: 0.85,
                stuck: 0.0,
                wet: 0.1,
                tipped: 0.03,
                sank: 0.02,
                drowned: 0.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeSeasonalFactors {
    pub spring: f32,
    pub summer: f32,
    pub fall: f32,
    pub winter: f32,
}

impl OtDeluxeSeasonalFactors {
    #[must_use]
    pub const fn for_season(self, season: Season) -> f32 {
        match season {
            Season::Spring => self.spring,
            Season::Summer => self.summer,
            Season::Fall => self.fall,
            Season::Winter => self.winter,
        }
    }
}

impl Default for OtDeluxeSeasonalFactors {
    fn default() -> Self {
        Self {
            spring: 1.1,
            summer: 0.85,
            fall: 0.95,
            winter: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeRiverProfile {
    pub min_depth_ft: f32,
    pub max_depth_ft: f32,
    pub min_width_ft: f32,
    pub max_width_ft: f32,
    pub min_swiftness: f32,
    pub max_swiftness: f32,
    pub bed: OtDeluxeRiverBed,
    pub ferry_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeRiverProfiles {
    pub kansas: OtDeluxeRiverProfile,
    pub big_blue: OtDeluxeRiverProfile,
    pub green: OtDeluxeRiverProfile,
    pub snake: OtDeluxeRiverProfile,
}

impl Default for OtDeluxeRiverProfiles {
    fn default() -> Self {
        Self {
            kansas: OtDeluxeRiverProfile {
                min_depth_ft: 1.8,
                max_depth_ft: 4.5,
                min_width_ft: 180.0,
                max_width_ft: 520.0,
                min_swiftness: 0.25,
                max_swiftness: 0.85,
                bed: OtDeluxeRiverBed::Muddy,
                ferry_available: true,
            },
            big_blue: OtDeluxeRiverProfile {
                min_depth_ft: 2.0,
                max_depth_ft: 5.0,
                min_width_ft: 160.0,
                max_width_ft: 480.0,
                min_swiftness: 0.3,
                max_swiftness: 0.9,
                bed: OtDeluxeRiverBed::Muddy,
                ferry_available: false,
            },
            green: OtDeluxeRiverProfile {
                min_depth_ft: 2.4,
                max_depth_ft: 6.0,
                min_width_ft: 200.0,
                max_width_ft: 600.0,
                min_swiftness: 0.35,
                max_swiftness: 1.0,
                bed: OtDeluxeRiverBed::Rocky,
                ferry_available: true,
            },
            snake: OtDeluxeRiverProfile {
                min_depth_ft: 2.8,
                max_depth_ft: 7.0,
                min_width_ft: 260.0,
                max_width_ft: 720.0,
                min_swiftness: 0.45,
                max_swiftness: 1.15,
                bed: OtDeluxeRiverBed::Rocky,
                ferry_available: true,
            },
        }
    }
}

impl OtDeluxeRiverProfiles {
    #[must_use]
    pub const fn profile_for(&self, river: OtDeluxeRiver) -> &OtDeluxeRiverProfile {
        match river {
            OtDeluxeRiver::Kansas => &self.kansas,
            OtDeluxeRiver::BigBlue => &self.big_blue,
            OtDeluxeRiver::Green => &self.green,
            OtDeluxeRiver::Snake => &self.snake,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeActionTimeCosts {
    pub rest_days_min: u8,
    pub rest_days_max: u8,
    pub trade_cost_days: u8,
    pub hunt_cost_days: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeHuntPolicy {
    pub carry_cap_lbs_per_alive_member: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeStorePolicy {
    pub buy_only_at_forts: bool,
    pub bullets_per_box: u16,
    pub store_node_indices: [u8; 7],
    pub base_prices_cents: OtDeluxeStoreBasePricesCents,
    pub price_mult_pct_by_node: [u16; 19],
    pub max_buy: OtDeluxeStoreMaxBuy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeTrailPolicy {
    pub total_miles_main: u16,
    pub mile_markers_main: [u16; 17],
    pub mile_markers_sublette: [u16; 17],
    pub mile_markers_dalles_shortcut: [u16; 17],
    pub mile_markers_sublette_and_dalles_shortcut: [u16; 17],
    #[serde(default)]
    pub mountain_nodes: [bool; 18],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeNavigationDelay {
    pub min_days: u8,
    pub max_days: u8,
}

impl Default for OtDeluxeNavigationDelay {
    fn default() -> Self {
        Self {
            min_days: 1,
            max_days: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeNavigationPolicy {
    pub chance_per_day: f32,
    pub lost_weight: u16,
    pub wrong_weight: u16,
    pub impassable_weight: u16,
    pub snowbound_weight: u16,
    #[serde(default)]
    pub lost_delay: OtDeluxeNavigationDelay,
    #[serde(default)]
    pub wrong_delay: OtDeluxeNavigationDelay,
    #[serde(default)]
    pub impassable_delay: OtDeluxeNavigationDelay,
    #[serde(default)]
    pub snowbound_delay: OtDeluxeNavigationDelay,
    #[serde(default)]
    pub snowbound_min_depth_in: f32,
}

impl Default for OtDeluxeNavigationPolicy {
    fn default() -> Self {
        Self {
            chance_per_day: 0.0,
            lost_weight: 1,
            wrong_weight: 1,
            impassable_weight: 1,
            snowbound_weight: 1,
            lost_delay: OtDeluxeNavigationDelay::default(),
            wrong_delay: OtDeluxeNavigationDelay::default(),
            impassable_delay: OtDeluxeNavigationDelay::default(),
            snowbound_delay: OtDeluxeNavigationDelay::default(),
            snowbound_min_depth_in: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeOccupationAdvantages {
    pub doctor_fatality_mult: f32,
    pub repair_success_mult: f32,
    pub mobility_failure_mult: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeTravelPolicy {
    pub base_mpd_plains_steady_good: f32,
    pub terrain_mult_mountains: f32,
    pub sick_member_speed_penalty: f32,
    #[serde(default)]
    pub snow_speed_penalty_per_in: f32,
    #[serde(default)]
    pub snow_speed_floor: f32,
    #[serde(default = "OtDeluxeTravelPolicy::default_partial_ratio")]
    pub partial_ratio: f32,
}

impl Default for OtDeluxeTravelPolicy {
    fn default() -> Self {
        Self {
            base_mpd_plains_steady_good: 20.0,
            terrain_mult_mountains: 0.5,
            sick_member_speed_penalty: 0.10,
            snow_speed_penalty_per_in: 0.0,
            snow_speed_floor: 0.0,
            partial_ratio: Self::default_partial_ratio(),
        }
    }
}

impl OtDeluxeTravelPolicy {
    const fn default_partial_ratio() -> f32 {
        0.5
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeBreakdownPolicy {
    pub base: f32,
    pub beta: f32,
    pub pace_mult_steady: f32,
    pub pace_mult_strenuous: f32,
    pub pace_mult_grueling: f32,
    pub max_chance: f32,
}

impl OtDeluxeBreakdownPolicy {
    #[must_use]
    pub const fn pace_multiplier(&self, pace: OtDeluxePace) -> f32 {
        match pace {
            OtDeluxePace::Steady => self.pace_mult_steady,
            OtDeluxePace::Strenuous => self.pace_mult_strenuous,
            OtDeluxePace::Grueling => self.pace_mult_grueling,
        }
    }
}

impl Default for OtDeluxeBreakdownPolicy {
    fn default() -> Self {
        Self {
            base: VEHICLE_BREAKDOWN_BASE_CHANCE,
            beta: VEHICLE_BREAKDOWN_WEAR_COEFFICIENT,
            pace_mult_steady: PACE_BREAKDOWN_STEADY,
            pace_mult_strenuous: PACE_BREAKDOWN_HEATED,
            pace_mult_grueling: PACE_BREAKDOWN_BLITZ,
            max_chance: 0.35,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeOxenPolicy {
    pub sick_ox_weight: f32,
    pub min_to_move: f32,
    pub min_for_base: f32,
}

impl Default for OtDeluxeOxenPolicy {
    fn default() -> Self {
        Self {
            sick_ox_weight: 0.5,
            min_to_move: 1.0,
            min_for_base: 4.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct OtDeluxeWeatherEffectsOverride {
    pub travel_mult: Option<f32>,
    pub supplies_delta: Option<i32>,
    pub sanity_delta: Option<i32>,
    pub pants_delta: Option<i32>,
    pub encounter_delta: Option<f32>,
    pub encounter_cap: Option<f32>,
    pub breakdown_mult: Option<f32>,
    pub rain_accum_delta: Option<f32>,
    pub snow_depth_delta: Option<f32>,
}

impl OtDeluxeWeatherEffectsOverride {
    pub const fn merge_from(&mut self, other: &Self) {
        if other.travel_mult.is_some() {
            self.travel_mult = other.travel_mult;
        }
        if other.supplies_delta.is_some() {
            self.supplies_delta = other.supplies_delta;
        }
        if other.sanity_delta.is_some() {
            self.sanity_delta = other.sanity_delta;
        }
        if other.pants_delta.is_some() {
            self.pants_delta = other.pants_delta;
        }
        if other.encounter_delta.is_some() {
            self.encounter_delta = other.encounter_delta;
        }
        if other.encounter_cap.is_some() {
            self.encounter_cap = other.encounter_cap;
        }
        if other.breakdown_mult.is_some() {
            self.breakdown_mult = other.breakdown_mult;
        }
        if other.rain_accum_delta.is_some() {
            self.rain_accum_delta = other.rain_accum_delta;
        }
        if other.snow_depth_delta.is_some() {
            self.snow_depth_delta = other.snow_depth_delta;
        }
    }

    pub const fn apply(&self, effects: &mut WeatherEffects) {
        if let Some(value) = self.travel_mult {
            effects.travel_mult = value;
        }
        if let Some(value) = self.supplies_delta {
            effects.supplies_delta = value;
        }
        if let Some(value) = self.sanity_delta {
            effects.sanity_delta = value;
        }
        if let Some(value) = self.pants_delta {
            effects.pants_delta = value;
        }
        if let Some(value) = self.encounter_delta {
            effects.encounter_delta = value;
        }
        if let Some(value) = self.encounter_cap {
            effects.encounter_cap = value;
        }
        if let Some(value) = self.breakdown_mult {
            effects.breakdown_mult = value;
        }
        if let Some(value) = self.rain_accum_delta {
            effects.rain_accum_delta = value;
        }
        if let Some(value) = self.snow_depth_delta {
            effects.snow_depth_delta = value;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeAfflictionWeightOverride {
    pub illness: Option<u16>,
    pub injury: Option<u16>,
}

impl OtDeluxeAfflictionWeightOverride {
    pub const fn merge_from(&mut self, other: &Self) {
        if other.illness.is_some() {
            self.illness = other.illness;
        }
        if other.injury.is_some() {
            self.injury = other.injury;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OtDeluxePolicyOverride {
    pub travel_multiplier: Option<f32>,
    pub event_weight_mult: Option<f32>,
    pub event_weight_cap: Option<f32>,
    #[serde(default)]
    pub weather_effects: OtDeluxeWeatherEffectsOverride,
    #[serde(default)]
    pub affliction_weights: OtDeluxeAfflictionWeightOverride,
}

impl OtDeluxePolicyOverride {
    pub const fn merge_from(&mut self, other: &Self) {
        if other.travel_multiplier.is_some() {
            self.travel_multiplier = other.travel_multiplier;
        }
        if other.event_weight_mult.is_some() {
            self.event_weight_mult = other.event_weight_mult;
        }
        if other.event_weight_cap.is_some() {
            self.event_weight_cap = other.event_weight_cap;
        }
        self.weather_effects.merge_from(&other.weather_effects);
        self.affliction_weights
            .merge_from(&other.affliction_weights);
    }
}

/// Parity-oriented policy for Oregon Trail Deluxe (DOS v3.0 lineage).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxe90sPolicy {
    pub pace_mult_steady: f32,
    pub pace_mult_strenuous: f32,
    pub pace_mult_grueling: f32,
    #[serde(default)]
    pub pace_health_penalty: OtDeluxePaceHealthPolicy,
    pub rations_enum: [OtDeluxeRations; 3],
    #[serde(default)]
    pub rations: OtDeluxeRationsPolicy,
    pub store: OtDeluxeStorePolicy,
    pub occupations: [OtDeluxeOccupationSpec; 8],
    pub occupation_advantages: OtDeluxeOccupationAdvantages,
    pub oxen: OtDeluxeOxenPolicy,
    pub travel: OtDeluxeTravelPolicy,
    #[serde(default)]
    pub breakdown: OtDeluxeBreakdownPolicy,
    pub navigation: OtDeluxeNavigationPolicy,
    pub health: OtDeluxeHealthPolicy,
    pub affliction: OtDeluxeAfflictionPolicy,
    pub crossings: OtDeluxeCrossingPolicy,
    pub actions: OtDeluxeActionTimeCosts,
    pub hunt: OtDeluxeHuntPolicy,
    pub trail: OtDeluxeTrailPolicy,
    pub score: OtDeluxeScorePolicy,
    #[serde(default)]
    pub per_region_overrides: HashMap<Region, OtDeluxePolicyOverride>,
    #[serde(default)]
    pub per_season_overrides: HashMap<Season, OtDeluxePolicyOverride>,
}

const DEFAULT_RATIONS_ENUM: [OtDeluxeRations; 3] = [
    OtDeluxeRations::Filling,
    OtDeluxeRations::Meager,
    OtDeluxeRations::BareBones,
];

const DEFAULT_OCCUPATIONS: [OtDeluxeOccupationSpec; 8] = [
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Banker,
        starting_cash_dollars: 1600,
        final_bonus_mult: 1.0,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Doctor,
        starting_cash_dollars: 1200,
        final_bonus_mult: 1.0,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Merchant,
        starting_cash_dollars: 1200,
        final_bonus_mult: 1.5,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Blacksmith,
        starting_cash_dollars: 800,
        final_bonus_mult: 2.0,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Carpenter,
        starting_cash_dollars: 800,
        final_bonus_mult: 2.0,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Saddlemaker,
        starting_cash_dollars: 800,
        final_bonus_mult: 2.5,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Farmer,
        starting_cash_dollars: 400,
        final_bonus_mult: 3.0,
    },
    OtDeluxeOccupationSpec {
        occupation: OtDeluxeOccupation::Teacher,
        starting_cash_dollars: 400,
        final_bonus_mult: 3.5,
    },
];

const DEFAULT_AFFLICTION_CURVE_PWL: [OtDeluxeAfflictionCurvePoint; 9] = [
    OtDeluxeAfflictionCurvePoint {
        health: 0,
        probability: 0.00,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 34,
        probability: 0.05,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 35,
        probability: 0.05,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 69,
        probability: 0.15,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 70,
        probability: 0.15,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 104,
        probability: 0.25,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 105,
        probability: 0.25,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 139,
        probability: 0.40,
    },
    OtDeluxeAfflictionCurvePoint {
        health: 140,
        probability: 0.40,
    },
];

impl Default for OtDeluxeStorePolicy {
    fn default() -> Self {
        Self {
            buy_only_at_forts: true,
            bullets_per_box: 20,
            store_node_indices: [0, 3, 5, 8, 11, 13, 15],
            base_prices_cents: OtDeluxeStoreBasePricesCents {
                ox: 2000,
                clothes_set: 1000,
                bullet: 10,
                ammo_box: 200,
                food_lb: 20,
                wheel: 1000,
                axle: 1000,
                tongue: 1000,
            },
            price_mult_pct_by_node: [
                100, 100, 100, 100, 125, 125, 150, 150, 150, 175, 175, 175, 200, 200, 225, 250,
                250, 250, 250,
            ],
            max_buy: OtDeluxeStoreMaxBuy {
                oxen: 20,
                ammo_boxes: 50,
                clothes_sets: 99,
                wheels: 3,
                axles: 3,
                tongues: 3,
                food_lbs: 2000,
            },
        }
    }
}

impl Default for OtDeluxeTrailPolicy {
    fn default() -> Self {
        const DEFAULT_MOUNTAIN_NODES: [bool; 18] = [
            false, false, false, false, false, false, false, true, false, false, false, false,
            false, false, true, false, false, false,
        ];
        Self {
            total_miles_main: 2083,
            mile_markers_main: [
                102, 185, 304, 554, 640, 830, 932, 989, 1151, 1295, 1352, 1534, 1648, 1808, 1863,
                1983, 2083,
            ],
            mile_markers_sublette: [
                102, 185, 304, 554, 640, 830, 932, 0, 1057, 1201, 1258, 1440, 1554, 1714, 1769,
                1889, 1989,
            ],
            mile_markers_dalles_shortcut: [
                102, 185, 304, 554, 640, 830, 932, 989, 1151, 1295, 1352, 1534, 1648, 1808, 0,
                1933, 2033,
            ],
            mile_markers_sublette_and_dalles_shortcut: [
                102, 185, 304, 554, 640, 830, 932, 0, 1057, 1201, 1258, 1440, 1554, 1714, 0, 1839,
                1939,
            ],
            mountain_nodes: DEFAULT_MOUNTAIN_NODES,
        }
    }
}

impl Default for OtDeluxeScorePolicy {
    fn default() -> Self {
        Self {
            points_per_person_by_health: OtDeluxeScorePointsPerPersonByHealth {
                good: 500,
                fair: 0,
                poor: 0,
                very_poor: 0,
            },
            points_wagon: 50,
            points_ox: 4,
            points_spare_part: 2,
            points_clothes: 2,
            divisor_bullets: 50,
            divisor_food_lbs: 25,
            divisor_cash_cents: 500,
        }
    }
}

impl Default for OtDeluxeHealthPolicy {
    fn default() -> Self {
        Self {
            recovery_baseline: -10,
            death_threshold: 140,
            label_ranges: OtDeluxeHealthLabelRanges {
                good_max: 34,
                fair_max: 69,
                poor_max: 104,
                very_poor_max: 139,
            },
            death_imminent_grace_days: 3,
            death_imminent_resets_on_recovery_below_threshold: true,
            weather_penalty: HashMap::new(),
            clothing_sets_per_person: 2,
            clothing_penalty_winter: 0,
            affliction_illness_penalty: 0,
            affliction_injury_penalty: 0,
            drought_threshold: 0.0,
            drought_penalty: 0,
        }
    }
}

impl Default for OtDeluxeActionTimeCosts {
    fn default() -> Self {
        Self {
            rest_days_min: 1,
            rest_days_max: 9,
            trade_cost_days: 1,
            hunt_cost_days: 1,
        }
    }
}

impl Default for OtDeluxeHuntPolicy {
    fn default() -> Self {
        Self {
            carry_cap_lbs_per_alive_member: 100,
        }
    }
}

impl Default for OtDeluxeOccupationAdvantages {
    fn default() -> Self {
        Self {
            doctor_fatality_mult: 0.50,
            repair_success_mult: 1.25,
            mobility_failure_mult: 0.75,
        }
    }
}

impl Default for OtDeluxeAfflictionPolicy {
    fn default() -> Self {
        Self {
            probability_max: 0.40,
            curve_pwl: DEFAULT_AFFLICTION_CURVE_PWL,
            weight_illness: 1,
            weight_injury: 1,
            illness_duration_days: 10,
            injury_duration_days: 30,
        }
    }
}

impl Default for OtDeluxeCrossingPolicy {
    fn default() -> Self {
        Self {
            ferry_cost_cents: 500,
            guide_cost_clothes_sets: 3,
            ferry_min_depth_ft: 2.5,
            float_min_depth_ft: 1.5,
            wet_goods_min_depth_ft: 2.5,
            swamped_min_depth_ft: 3.0,
            drying_cost_days: 1,
            crossing_cost_days: 1,
            guide_risk_mult: 0.20,
            guide_loss_mult: 0.50,
            ferry_wait_days_min: 0,
            ferry_wait_days_max: 6,
            ferry_accident_risk_max: 0.10,
            outcome_weights: OtDeluxeCrossingOutcomeWeightsByMethod::default(),
            river_profiles: OtDeluxeRiverProfiles::default(),
            seasonal_depth_mult: OtDeluxeSeasonalFactors::default(),
            seasonal_swiftness_mult: OtDeluxeSeasonalFactors::default(),
            rain_accum_max: 4.0,
            rain_depth_mult: 1.0,
            rain_width_mult: 1.0,
            rain_swiftness_mult: 1.0,
            swiftness_risk_mult: 0.8,
            shallow_safe_bonus: 1.2,
            wet_goods_bonus: 1.5,
            swamped_sank_bonus: 2.0,
            stuck_muddy_mult: 1.25,
            tipped_rocky_mult: 1.25,
            stuck_cost_days: 1,
            tipped_loss_ratio: 0.10,
            sank_loss_ratio: 0.30,
            drownings_min: 1,
            drownings_max: 1,
        }
    }
}

impl Default for OtDeluxe90sPolicy {
    fn default() -> Self {
        Self {
            pace_mult_steady: 1.0,
            pace_mult_strenuous: 1.5,
            pace_mult_grueling: 2.0,
            pace_health_penalty: OtDeluxePaceHealthPolicy::default(),
            rations_enum: DEFAULT_RATIONS_ENUM,
            rations: OtDeluxeRationsPolicy::default(),
            store: OtDeluxeStorePolicy::default(),
            occupations: DEFAULT_OCCUPATIONS,
            occupation_advantages: OtDeluxeOccupationAdvantages::default(),
            oxen: OtDeluxeOxenPolicy::default(),
            travel: OtDeluxeTravelPolicy::default(),
            breakdown: OtDeluxeBreakdownPolicy::default(),
            navigation: OtDeluxeNavigationPolicy::default(),
            health: OtDeluxeHealthPolicy::default(),
            affliction: OtDeluxeAfflictionPolicy::default(),
            crossings: OtDeluxeCrossingPolicy::default(),
            actions: OtDeluxeActionTimeCosts::default(),
            hunt: OtDeluxeHuntPolicy::default(),
            trail: OtDeluxeTrailPolicy::default(),
            score: OtDeluxeScorePolicy::default(),
            per_region_overrides: HashMap::new(),
            per_season_overrides: HashMap::new(),
        }
    }
}

impl OtDeluxe90sPolicy {
    #[must_use]
    pub fn overrides_for(&self, region: Region, season: Season) -> OtDeluxePolicyOverride {
        let mut combined = OtDeluxePolicyOverride::default();
        if let Some(region_override) = self.per_region_overrides.get(&region) {
            combined.merge_from(region_override);
        }
        if let Some(season_override) = self.per_season_overrides.get(&season) {
            combined.merge_from(season_override);
        }
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Region, Season};
    use crate::weather::WeatherEffects;

    fn assert_f32_eq(a: f32, b: f32) {
        let epsilon = 1e-6_f32;
        assert!(
            (a - b).abs() <= epsilon,
            "floats differ: {a} vs {b} (ε={epsilon})"
        );
    }

    #[test]
    fn default_policy_matches_extracted_trail_and_store_constants() {
        let policy = OtDeluxe90sPolicy::default();

        assert_eq!(policy.trail.total_miles_main, 2083);
        assert_eq!(policy.trail.mile_markers_main[16], 2083);
        assert_eq!(policy.trail.mile_markers_sublette[7], 0);
        assert_eq!(policy.trail.mile_markers_dalles_shortcut[14], 0);
        assert_eq!(policy.trail.mile_markers_sublette_and_dalles_shortcut[7], 0);
        assert_eq!(
            policy.trail.mile_markers_sublette_and_dalles_shortcut[14],
            0
        );

        assert_eq!(policy.store.base_prices_cents.ox, 2000);
        assert_eq!(policy.store.base_prices_cents.food_lb, 20);
        assert_eq!(policy.store.max_buy.food_lbs, 2000);
        assert_eq!(policy.store.price_mult_pct_by_node[0], 100);
        assert_eq!(policy.store.price_mult_pct_by_node[15], 250);
        assert_eq!(policy.store.price_mult_pct_by_node[18], 250);
    }

    #[test]
    fn default_policy_includes_required_parity_defaults() {
        let policy = OtDeluxe90sPolicy::default();

        assert_eq!(policy.rations_enum, DEFAULT_RATIONS_ENUM);
        assert_f32_eq(policy.affliction.probability_max, 0.40);
        for (observed, expected) in policy
            .affliction
            .curve_pwl
            .iter()
            .zip(DEFAULT_AFFLICTION_CURVE_PWL.iter())
        {
            assert_eq!(observed.health, expected.health);
            assert_f32_eq(observed.probability, expected.probability);
        }

        assert_eq!(policy.crossings.ferry_cost_cents, 500);
        assert_eq!(policy.crossings.guide_cost_clothes_sets, 3);
        assert_eq!(policy.crossings.ferry_wait_days_min, 0);
        assert_eq!(policy.crossings.ferry_wait_days_max, 6);
        assert_f32_eq(policy.oxen.sick_ox_weight, 0.5);
        assert_f32_eq(policy.oxen.min_to_move, 1.0);
        assert_f32_eq(policy.oxen.min_for_base, 4.0);
        assert_f32_eq(policy.travel.base_mpd_plains_steady_good, 20.0);
        assert_f32_eq(policy.travel.terrain_mult_mountains, 0.5);
        assert_f32_eq(policy.travel.sick_member_speed_penalty, 0.10);
        assert_eq!(policy.pace_health_penalty.steady, 0);
        assert_eq!(policy.pace_health_penalty.strenuous, 5);
        assert_eq!(policy.pace_health_penalty.grueling, 10);
        assert_eq!(policy.rations.food_lbs_per_person, [3, 2, 1]);
        assert_eq!(policy.rations.health_penalty, [0, 5, 10]);
        assert_eq!(policy.affliction.weight_illness, 1);
        assert_eq!(policy.affliction.weight_injury, 1);
        assert_eq!(policy.affliction.illness_duration_days, 10);
        assert_eq!(policy.affliction.injury_duration_days, 30);
        assert!(policy.health.weather_penalty.is_empty());
        assert_eq!(policy.health.clothing_sets_per_person, 2);
        assert_eq!(policy.health.clothing_penalty_winter, 0);
        assert_eq!(policy.health.affliction_illness_penalty, 0);
        assert_eq!(policy.health.affliction_injury_penalty, 0);
        assert_f32_eq(policy.health.drought_threshold, 0.0);
        assert_eq!(policy.health.drought_penalty, 0);
        assert_f32_eq(policy.navigation.chance_per_day, 0.0);
        assert_eq!(policy.navigation.lost_weight, 1);
        assert_eq!(policy.navigation.wrong_weight, 1);
        assert_eq!(policy.navigation.impassable_weight, 1);
        assert_eq!(policy.navigation.snowbound_weight, 1);
        assert_eq!(policy.navigation.lost_delay.min_days, 1);
        assert_eq!(policy.navigation.lost_delay.max_days, 3);
        assert_eq!(policy.navigation.wrong_delay.min_days, 1);
        assert_eq!(policy.navigation.wrong_delay.max_days, 3);
        assert_eq!(policy.navigation.impassable_delay.min_days, 1);
        assert_eq!(policy.navigation.impassable_delay.max_days, 3);
        assert_eq!(policy.navigation.snowbound_delay.min_days, 1);
        assert_eq!(policy.navigation.snowbound_delay.max_days, 3);
        assert_f32_eq(policy.navigation.snowbound_min_depth_in, 0.0);

        assert_eq!(policy.score.points_per_person_by_health.good, 500);
        assert_eq!(policy.score.divisor_cash_cents, 500);
    }

    #[test]
    fn seasonal_factors_return_per_season_values() {
        let factors = OtDeluxeSeasonalFactors {
            spring: 1.0,
            summer: 2.0,
            fall: 3.0,
            winter: 4.0,
        };
        assert_f32_eq(factors.for_season(Season::Spring), 1.0);
        assert_f32_eq(factors.for_season(Season::Summer), 2.0);
        assert_f32_eq(factors.for_season(Season::Fall), 3.0);
        assert_f32_eq(factors.for_season(Season::Winter), 4.0);
    }

    #[test]
    fn policy_overrides_merge_region_then_season() {
        let mut policy = OtDeluxe90sPolicy::default();
        let region_override = OtDeluxePolicyOverride {
            travel_multiplier: Some(0.8),
            weather_effects: OtDeluxeWeatherEffectsOverride {
                travel_mult: Some(0.9),
                ..OtDeluxeWeatherEffectsOverride::default()
            },
            ..OtDeluxePolicyOverride::default()
        };
        policy
            .per_region_overrides
            .insert(Region::RustBelt, region_override);

        let season_override = OtDeluxePolicyOverride {
            travel_multiplier: Some(1.2),
            affliction_weights: OtDeluxeAfflictionWeightOverride {
                illness: Some(3),
                ..OtDeluxeAfflictionWeightOverride::default()
            },
            ..OtDeluxePolicyOverride::default()
        };
        policy
            .per_season_overrides
            .insert(Season::Winter, season_override);

        let combined = policy.overrides_for(Region::RustBelt, Season::Winter);
        assert_eq!(combined.travel_multiplier, Some(1.2));
        assert_eq!(combined.weather_effects.travel_mult, Some(0.9));
        assert_eq!(combined.affliction_weights.illness, Some(3));
    }

    #[test]
    fn weather_effect_overrides_apply_to_effects() {
        let overrides = OtDeluxeWeatherEffectsOverride {
            travel_mult: Some(0.7),
            supplies_delta: Some(-2),
            sanity_delta: None,
            pants_delta: Some(1),
            encounter_delta: None,
            encounter_cap: Some(0.6),
            breakdown_mult: Some(1.3),
            rain_accum_delta: Some(0.5),
            snow_depth_delta: None,
        };
        let mut effects = WeatherEffects::default();
        overrides.apply(&mut effects);

        assert_f32_eq(effects.travel_mult, 0.7);
        assert_eq!(effects.supplies_delta, -2);
        assert_eq!(effects.pants_delta, 1);
        assert_f32_eq(effects.encounter_cap, 0.6);
        assert_f32_eq(effects.breakdown_mult, 1.3);
        assert_f32_eq(effects.rain_accum_delta, 0.5);
    }

    #[test]
    fn breakdown_policy_pace_multiplier_covers_variants() {
        let policy = OtDeluxeBreakdownPolicy::default();
        assert_f32_eq(
            policy.pace_multiplier(OtDeluxePace::Strenuous),
            PACE_BREAKDOWN_HEATED,
        );
        assert_f32_eq(
            policy.pace_multiplier(OtDeluxePace::Grueling),
            PACE_BREAKDOWN_BLITZ,
        );
    }

    #[test]
    fn weather_effect_overrides_merge_and_apply() {
        let mut base = OtDeluxeWeatherEffectsOverride::default();
        let overlay = OtDeluxeWeatherEffectsOverride {
            travel_mult: Some(0.8),
            supplies_delta: Some(-2),
            sanity_delta: Some(-1),
            pants_delta: Some(-3),
            encounter_delta: Some(0.2),
            encounter_cap: Some(0.5),
            breakdown_mult: Some(1.5),
            rain_accum_delta: Some(0.3),
            snow_depth_delta: Some(0.4),
        };
        base.merge_from(&overlay);
        assert_eq!(base.travel_mult, Some(0.8));
        assert_eq!(base.supplies_delta, Some(-2));
        assert_eq!(base.sanity_delta, Some(-1));
        assert_eq!(base.pants_delta, Some(-3));
        assert_eq!(base.encounter_delta, Some(0.2));
        assert_eq!(base.encounter_cap, Some(0.5));
        assert_eq!(base.breakdown_mult, Some(1.5));
        assert_eq!(base.rain_accum_delta, Some(0.3));
        assert_eq!(base.snow_depth_delta, Some(0.4));

        let mut effects = WeatherEffects::default();
        base.apply(&mut effects);
        assert_f32_eq(effects.travel_mult, 0.8);
        assert_eq!(effects.supplies_delta, -2);
        assert_eq!(effects.sanity_delta, -1);
        assert_eq!(effects.pants_delta, -3);
        assert_f32_eq(effects.encounter_delta, 0.2);
        assert_f32_eq(effects.encounter_cap, 0.5);
        assert_f32_eq(effects.breakdown_mult, 1.5);
        assert_f32_eq(effects.rain_accum_delta, 0.3);
        assert_f32_eq(effects.snow_depth_delta, 0.4);
    }

    #[test]
    fn policy_override_merge_updates_fields() {
        let mut base = OtDeluxePolicyOverride::default();
        let overlay = OtDeluxePolicyOverride {
            travel_multiplier: Some(0.9),
            event_weight_mult: Some(1.1),
            event_weight_cap: Some(2.0),
            weather_effects: OtDeluxeWeatherEffectsOverride {
                travel_mult: Some(0.7),
                ..OtDeluxeWeatherEffectsOverride::default()
            },
            affliction_weights: OtDeluxeAfflictionWeightOverride {
                illness: Some(2),
                injury: Some(3),
            },
        };
        base.merge_from(&overlay);
        assert_eq!(base.travel_multiplier, Some(0.9));
        assert_eq!(base.event_weight_mult, Some(1.1));
        assert_eq!(base.event_weight_cap, Some(2.0));
        assert_eq!(base.weather_effects.travel_mult, Some(0.7));
        assert_eq!(base.affliction_weights.illness, Some(2));
        assert_eq!(base.affliction_weights.injury, Some(3));
    }
}
