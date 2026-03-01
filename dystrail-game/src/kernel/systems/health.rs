use crate::kernel::systems::supplies::{
    otdeluxe_pace_health_penalty, otdeluxe_rations_health_penalty,
};
use crate::mechanics::otdeluxe90s::{
    OtDeluxe90sPolicy, OtDeluxeHealthPolicy, OtDeluxePace, OtDeluxeRations,
};
use crate::otdeluxe_state::{OtDeluxeInventory, OtDeluxePartyState};
use crate::state::Season;
use crate::weather::Weather;

#[derive(Debug, Clone, Copy)]
pub struct OtDeluxeHealthDeltaContext<'a> {
    pub pace: OtDeluxePace,
    pub rations: OtDeluxeRations,
    pub weather: Weather,
    pub season: Season,
    pub inventory: &'a OtDeluxeInventory,
    pub alive: u16,
    pub party: &'a OtDeluxePartyState,
    pub rain_accum: f32,
}

#[must_use]
pub fn otdeluxe_health_delta(
    context: OtDeluxeHealthDeltaContext<'_>,
    policy: &OtDeluxe90sPolicy,
) -> i32 {
    let pace_penalty = otdeluxe_pace_health_penalty(context.pace, &policy.pace_health_penalty);
    let rations_penalty = otdeluxe_rations_health_penalty(context.rations, &policy.rations);
    let weather_penalty = otdeluxe_weather_health_penalty(context.weather, &policy.health);
    let clothing_penalty = otdeluxe_clothing_health_penalty(
        context.season,
        context.inventory,
        context.alive,
        &policy.health,
    );
    let affliction_penalty = otdeluxe_affliction_health_penalty(context.party, &policy.health);
    let drought_penalty = otdeluxe_drought_health_penalty(context.rain_accum, &policy.health);
    policy.health.recovery_baseline
        + pace_penalty
        + rations_penalty
        + weather_penalty
        + clothing_penalty
        + affliction_penalty
        + drought_penalty
}

#[must_use]
pub fn otdeluxe_weather_health_penalty(weather: Weather, policy: &OtDeluxeHealthPolicy) -> i32 {
    *policy.weather_penalty.get(&weather).unwrap_or(&0)
}

#[must_use]
pub fn otdeluxe_clothing_health_penalty(
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

#[must_use]
pub fn otdeluxe_affliction_health_penalty(
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

#[must_use]
pub fn otdeluxe_drought_health_penalty(rain_accum: f32, policy: &OtDeluxeHealthPolicy) -> i32 {
    if !rain_accum.is_finite() || policy.drought_threshold <= 0.0 {
        return 0;
    }
    if rain_accum <= policy.drought_threshold {
        policy.drought_penalty
    } else {
        0
    }
}
