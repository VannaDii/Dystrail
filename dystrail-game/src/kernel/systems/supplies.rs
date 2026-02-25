use crate::mechanics::otdeluxe90s::{
    OtDeluxe90sPolicy, OtDeluxePace, OtDeluxePaceHealthPolicy, OtDeluxeRations,
    OtDeluxeRationsPolicy,
};
use crate::numbers::round_f32_to_i32;

#[must_use]
pub const fn otdeluxe_pace_health_penalty(
    pace: OtDeluxePace,
    policy: &OtDeluxePaceHealthPolicy,
) -> i32 {
    match pace {
        OtDeluxePace::Steady => policy.steady,
        OtDeluxePace::Strenuous => policy.strenuous,
        OtDeluxePace::Grueling => policy.grueling,
    }
}

#[must_use]
pub const fn otdeluxe_pace_food_multiplier(pace: OtDeluxePace, policy: &OtDeluxe90sPolicy) -> f32 {
    let mult = match pace {
        OtDeluxePace::Steady => policy.pace_mult_steady,
        OtDeluxePace::Strenuous => policy.pace_mult_strenuous,
        OtDeluxePace::Grueling => policy.pace_mult_grueling,
    };
    if mult < 0.0 { 0.0 } else { mult }
}

#[must_use]
pub const fn otdeluxe_rations_food_per_person(
    rations: OtDeluxeRations,
    policy: &OtDeluxeRationsPolicy,
) -> u16 {
    match rations {
        OtDeluxeRations::Filling => policy.food_lbs_per_person[0],
        OtDeluxeRations::Meager => policy.food_lbs_per_person[1],
        OtDeluxeRations::BareBones => policy.food_lbs_per_person[2],
    }
}

#[must_use]
pub const fn otdeluxe_rations_health_penalty(
    rations: OtDeluxeRations,
    policy: &OtDeluxeRationsPolicy,
) -> i32 {
    match rations {
        OtDeluxeRations::Filling => policy.health_penalty[0],
        OtDeluxeRations::Meager => policy.health_penalty[1],
        OtDeluxeRations::BareBones => policy.health_penalty[2],
    }
}

#[must_use]
pub fn otdeluxe_rations_food_per_person_scaled(
    rations: OtDeluxeRations,
    pace: OtDeluxePace,
    policy: &OtDeluxe90sPolicy,
) -> u16 {
    let per_person = otdeluxe_rations_food_per_person(rations, &policy.rations);
    let scaled = f32::from(per_person) * otdeluxe_pace_food_multiplier(pace, policy);
    let rounded = round_f32_to_i32(scaled).max(0);
    u16::try_from(rounded).unwrap_or(u16::MAX)
}
