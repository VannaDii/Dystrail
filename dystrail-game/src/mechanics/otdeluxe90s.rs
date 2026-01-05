//! Oregon Trail Deluxe (DOS v3.0) mechanical policy defaults.
//!
//! This module is intentionally data-only: it provides extracted constants and
//! policy defaults needed for parity, but it does not implement simulation
//! behavior by itself.

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeHealthPolicy {
    pub recovery_baseline: i32,
    pub death_threshold: u16,
    pub label_ranges: OtDeluxeHealthLabelRanges,
    pub death_imminent_grace_days: u8,
    pub death_imminent_resets_on_recovery_below_threshold: bool,
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
}

impl Default for OtDeluxeTravelPolicy {
    fn default() -> Self {
        Self {
            base_mpd_plains_steady_good: 20.0,
            terrain_mult_mountains: 0.5,
            sick_member_speed_penalty: 0.10,
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
    pub health: OtDeluxeHealthPolicy,
    pub affliction: OtDeluxeAfflictionPolicy,
    pub crossings: OtDeluxeCrossingPolicy,
    pub actions: OtDeluxeActionTimeCosts,
    pub hunt: OtDeluxeHuntPolicy,
    pub trail: OtDeluxeTrailPolicy,
    pub score: OtDeluxeScorePolicy,
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
            health: OtDeluxeHealthPolicy::default(),
            affliction: OtDeluxeAfflictionPolicy::default(),
            crossings: OtDeluxeCrossingPolicy::default(),
            actions: OtDeluxeActionTimeCosts::default(),
            hunt: OtDeluxeHuntPolicy::default(),
            trail: OtDeluxeTrailPolicy::default(),
            score: OtDeluxeScorePolicy::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(policy.score.points_per_person_by_health.good, 500);
        assert_eq!(policy.score.divisor_cash_cents, 500);
    }
}
