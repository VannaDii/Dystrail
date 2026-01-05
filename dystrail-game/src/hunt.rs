//! Hunting resolution for `OTDeluxe` parity scaffolding.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::state::{GameState, Region};
use crate::weather::Weather;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HuntBlockReason {
    NoBullets,
    SevereWeather,
    CrowdedLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HuntOutcome {
    Success {
        bullets_spent: u16,
        food_gained_lbs: u16,
    },
    Blocked(HuntBlockReason),
}

#[must_use]
pub fn resolve_hunt_with_rng(state: &mut GameState, rng: &mut impl Rng) -> HuntOutcome {
    if let Some(blocked) = hunt_block_reason(state) {
        return HuntOutcome::Blocked(blocked);
    }

    let bullets_available = state.ot_deluxe.inventory.bullets;
    let max_spend = bullets_available.clamp(1, 40);
    let bullets_spent = rng.gen_range(1..=max_spend);
    let food_per_bullet: u16 = rng.gen_range(2..=6);
    let food_shot = u32::from(bullets_spent) * u32::from(food_per_bullet);
    let carry_cap = carry_cap_lbs(state);
    let food_gained = clamp_u16(food_shot.min(u32::from(carry_cap)));

    state.ot_deluxe.inventory.bullets = state
        .ot_deluxe
        .inventory
        .bullets
        .saturating_sub(bullets_spent);
    state.ot_deluxe.inventory.food_lbs = state
        .ot_deluxe
        .inventory
        .food_lbs
        .saturating_add(food_gained);

    HuntOutcome::Success {
        bullets_spent,
        food_gained_lbs: food_gained,
    }
}

#[must_use]
pub fn resolve_hunt(state: &mut GameState) -> HuntOutcome {
    if let Some(blocked) = hunt_block_reason(state) {
        return HuntOutcome::Blocked(blocked);
    }

    let bullets_spent = 1;
    let food_shot = u32::from(bullets_spent) * 2;
    let carry_cap = carry_cap_lbs(state);
    let food_gained = clamp_u16(food_shot.min(u32::from(carry_cap)));

    state.ot_deluxe.inventory.bullets = state
        .ot_deluxe
        .inventory
        .bullets
        .saturating_sub(bullets_spent);
    state.ot_deluxe.inventory.food_lbs = state
        .ot_deluxe
        .inventory
        .food_lbs
        .saturating_add(food_gained);

    HuntOutcome::Success {
        bullets_spent,
        food_gained_lbs: food_gained,
    }
}

const fn hunt_block_reason(state: &GameState) -> Option<HuntBlockReason> {
    if state.ot_deluxe.inventory.bullets == 0 {
        return Some(HuntBlockReason::NoBullets);
    }
    if is_severe_weather(state.weather_state.today) {
        return Some(HuntBlockReason::SevereWeather);
    }
    if is_crowded_location(state.region) {
        return Some(HuntBlockReason::CrowdedLocation);
    }
    None
}

const fn is_severe_weather(weather: Weather) -> bool {
    matches!(weather, Weather::Storm | Weather::Smoke)
}

const fn is_crowded_location(region: Region) -> bool {
    matches!(region, Region::Beltway)
}

fn carry_cap_lbs(state: &GameState) -> u16 {
    let alive = state.otdeluxe_alive_party_count();
    alive.saturating_mul(100)
}

fn clamp_u16(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
