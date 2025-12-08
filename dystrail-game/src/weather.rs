//! Weather system and effects
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::journey::RngBundle;
use crate::state::{DamageCause, Ending, ExposureKind, GameState, Region, Season};

const LOG_WEATHER_EXPOSURE: &str = "log.weather.exposure";
const LOG_WEATHER_HEATSTROKE: &str = "log.weather.heatstroke";
const HEATWAVE_MAX_STREAK: i32 = 4;
const COLDSNAP_MAX_STREAK: i32 = 4;
const NEUTRAL_BUFFER_MIN: u8 = 2;
const NEUTRAL_BUFFER_MAX: u8 = 3;

/// Weather conditions that affect daily gameplay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Weather {
    #[default]
    Clear,
    Storm,
    HeatWave,
    ColdSnap,
    Smoke,
}

impl Weather {
    /// Check if weather is considered extreme (streak-limited)
    #[must_use]
    pub const fn is_extreme(self) -> bool {
        matches!(self, Self::Storm | Self::HeatWave | Self::Smoke)
    }

    /// Get i18n key for weather state name
    #[must_use]
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Clear => "weather.states.Clear",
            Self::Storm => "weather.states.Storm",
            Self::HeatWave => "weather.states.HeatWave",
            Self::ColdSnap => "weather.states.ColdSnap",
            Self::Smoke => "weather.states.Smoke",
        }
    }
}

const WEATHER_ORDER: [Weather; 5] = [
    Weather::Clear,
    Weather::Storm,
    Weather::HeatWave,
    Weather::ColdSnap,
    Weather::Smoke,
];

fn weather_weight(weights: &HashMap<Weather, u32>, weather: Weather) -> u32 {
    *weights.get(&weather).unwrap_or(&0)
}

const fn default_travel_mult() -> f32 {
    1.0
}

/// Daily effects from weather conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherEffect {
    pub supplies: i32,
    pub sanity: i32,
    pub pants: i32,
    pub enc_delta: f32,
    #[serde(default = "default_travel_mult")]
    pub travel_mult: f32,
}

/// Gear-based mitigation for weather effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherMitigation {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pants: Option<i32>,
}

/// Executive order modifiers for weather encounters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecWeatherMod {
    pub enc_delta: f32,
}

/// Configuration limits for weather system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherLimits {
    pub max_extreme_streak: i32,
    pub encounter_cap: f32,
    pub pants_floor: i32,
    pub pants_ceiling: i32,
}

/// Complete weather system configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub limits: WeatherLimits,
    pub effects: HashMap<Weather, WeatherEffect>,
    pub mitigation: HashMap<Weather, WeatherMitigation>,
    pub weights: HashMap<Region, HashMap<Weather, u32>>,
    pub exec_mods: HashMap<String, ExecWeatherMod>,
}

/// Weather state tracking for streaks and history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherState {
    pub today: Weather,
    pub yesterday: Weather,
    pub extreme_streak: i32,
    pub heatwave_streak: i32,
    pub coldsnap_streak: i32,
    pub neutral_buffer: u8,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            today: Weather::Clear,
            yesterday: Weather::Clear,
            extreme_streak: 0,
            heatwave_streak: 0,
            coldsnap_streak: 0,
            neutral_buffer: 0,
        }
    }
}

impl Eq for WeatherEffect {}
impl Eq for WeatherMitigation {}
impl Eq for ExecWeatherMod {}
impl Eq for WeatherLimits {}
impl Eq for WeatherConfig {}

impl WeatherConfig {
    /// Load weather configuration from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string cannot be parsed or if validation fails.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: Self =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration completeness
    fn validate(&self) -> Result<(), String> {
        // Check that all weather types have effects
        for weather in [
            Weather::Clear,
            Weather::Storm,
            Weather::HeatWave,
            Weather::ColdSnap,
            Weather::Smoke,
        ] {
            if !self.effects.contains_key(&weather) {
                return Err(format!("Missing effect for weather: {weather:?}"));
            }
        }

        // Check that all regions have weights
        for region in [Region::Heartland, Region::RustBelt, Region::Beltway] {
            let Some(region_weights) = self.weights.get(&region) else {
                return Err(format!("Missing weights for region: {region:?}"));
            };
            for weather in [
                Weather::Clear,
                Weather::Storm,
                Weather::HeatWave,
                Weather::ColdSnap,
                Weather::Smoke,
            ] {
                if !region_weights.contains_key(&weather) {
                    return Err(format!("Missing weight for {weather:?} in {region:?}"));
                }
            }
        }

        Ok(())
    }

    /// Load weather configuration from static assets (function for web compatibility)
    /// This is a placeholder that returns default data - web implementation should override this
    #[must_use]
    pub fn load_from_static() -> Self {
        Self::default_config()
    }

    /// Get embedded default configuration if loading fails
    #[must_use]
    pub fn default_config() -> Self {
        serde_json::from_str(include_str!(
            "../../dystrail-web/static/assets/data/weather.json"
        ))
        .unwrap_or_else(|_| Self {
            limits: WeatherLimits {
                max_extreme_streak: 2,
                encounter_cap: 0.35,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects: HashMap::new(),
            mitigation: HashMap::new(),
            weights: HashMap::new(),
            exec_mods: HashMap::new(),
        })
    }
}

/// Select today's weather based on region weights and streak limits
///
/// # Errors
///
/// Returns an error if RNG is not initialized or if weather weights don't exist for the current region.
pub fn select_weather_for_today(
    gs: &mut GameState,
    cfg: &WeatherConfig,
    rngs: &RngBundle,
) -> Result<Weather, String> {
    let mut rng = rngs.travel();

    let Some(region_weights) = cfg.weights.get(&gs.region) else {
        return Err(format!(
            "Weather weights must exist for region {:?}",
            gs.region
        ));
    };

    // Calculate total weight and make initial selection
    let total: u32 = WEATHER_ORDER
        .iter()
        .map(|weather| weather_weight(region_weights, *weather))
        .sum();
    let mut roll = rng.gen_range(0..total);
    let mut candidate = Weather::Clear;

    for weather in WEATHER_ORDER {
        let weight = weather_weight(region_weights, weather);
        if weight == 0 {
            continue;
        }
        if roll < weight {
            candidate = weather;
            break;
        }
        roll -= weight;
    }

    // Enforce extreme streak limit
    if candidate.is_extreme() && gs.weather_state.extreme_streak >= cfg.limits.max_extreme_streak {
        // Reselect from non-extremes deterministically
        let non_extreme_total: u32 = WEATHER_ORDER
            .iter()
            .filter(|weather| !weather.is_extreme())
            .map(|weather| weather_weight(region_weights, *weather))
            .sum();

        if non_extreme_total > 0 {
            let mut r2 = rng.gen_range(0..non_extreme_total);
            for weather in WEATHER_ORDER {
                if weather.is_extreme() {
                    continue;
                }
                let weight = weather_weight(region_weights, weather);
                if weight == 0 {
                    continue;
                }
                if r2 < weight {
                    candidate = weather;
                    break;
                }
                r2 -= weight;
            }
        }
    }

    let mut final_weather = seasonal_override(gs.season, candidate, &mut *rng);

    if gs.weather_state.neutral_buffer > 0 {
        final_weather = pick_neutral_weather(region_weights, &mut *rng);
        gs.weather_state.neutral_buffer = gs.weather_state.neutral_buffer.saturating_sub(1);
    } else {
        let needs_buffer = match final_weather {
            Weather::HeatWave => gs.weather_state.heatwave_streak >= HEATWAVE_MAX_STREAK,
            Weather::ColdSnap => gs.weather_state.coldsnap_streak >= COLDSNAP_MAX_STREAK,
            _ => false,
        };
        if needs_buffer {
            final_weather = apply_neutral_buffer(
                &mut gs.weather_state.neutral_buffer,
                region_weights,
                &mut *rng,
            );
        }
    }

    if final_weather.is_extreme()
        && gs.weather_state.extreme_streak >= cfg.limits.max_extreme_streak
    {
        final_weather = Weather::Clear;
    }

    Ok(final_weather)
}

fn seasonal_override<R: Rng>(season: Season, current: Weather, rng: &mut R) -> Weather {
    match season {
        Season::Winter => {
            if rng.r#gen::<f32>() < 0.20 {
                Weather::ColdSnap
            } else {
                current
            }
        }
        Season::Summer => {
            if rng.r#gen::<f32>() < 0.20 {
                Weather::HeatWave
            } else {
                current
            }
        }
        Season::Fall => {
            if rng.r#gen::<f32>() < 0.15 {
                Weather::Storm
            } else {
                current
            }
        }
        Season::Spring => {
            if rng.r#gen::<f32>() < 0.12 {
                Weather::Smoke
            } else {
                current
            }
        }
    }
}

fn pick_neutral_weather<R: Rng>(
    weights: &std::collections::HashMap<Weather, u32>,
    rng: &mut R,
) -> Weather {
    let neutral_order = [Weather::Clear, Weather::Smoke];
    let total: u32 = neutral_order
        .iter()
        .map(|weather| weather_weight(weights, *weather))
        .sum();

    if total == 0 {
        return Weather::Clear;
    }

    let mut roll = rng.gen_range(0..total);
    for weather in neutral_order {
        let weight = weather_weight(weights, weather);
        if weight == 0 {
            continue;
        }
        if roll < weight {
            return weather;
        }
        roll -= weight;
    }
    Weather::Clear
}

fn apply_neutral_buffer<R: Rng>(
    neutral_buffer: &mut u8,
    region_weights: &std::collections::HashMap<Weather, u32>,
    rng: &mut R,
) -> Weather {
    let new_weather = pick_neutral_weather(region_weights, rng);
    let buffer_len: u8 = rng.gen_range(NEUTRAL_BUFFER_MIN..=NEUTRAL_BUFFER_MAX);
    *neutral_buffer = buffer_len.saturating_sub(1);
    new_weather
}

/// Apply weather effects to game state
pub fn apply_weather_effects(gs: &mut GameState, cfg: &WeatherConfig) {
    let today = gs.weather_state.today;
    update_weather_streaks(&mut gs.weather_state, today);

    let Some(effect) = cfg.effects.get(&today) else {
        return;
    };

    gs.weather_travel_multiplier = effect.travel_mult.max(0.1);
    let (delta_sup, delta_san, delta_pants) = apply_mitigation(effect, cfg, gs);
    apply_stat_changes(gs, cfg, delta_sup, delta_san, delta_pants, effect.enc_delta);
    apply_exposure(gs, today);
}

fn apply_stat_changes(
    gs: &mut GameState,
    cfg: &WeatherConfig,
    delta_sup: i32,
    delta_san: i32,
    delta_pants: i32,
    delta_enc: f32,
) {
    gs.stats.supplies += delta_sup;
    gs.stats.sanity += delta_san;
    gs.stats.pants =
        (gs.stats.pants + delta_pants).clamp(cfg.limits.pants_floor, cfg.limits.pants_ceiling);
    gs.encounter_chance_today =
        (gs.encounter_chance_today + delta_enc).clamp(0.0, cfg.limits.encounter_cap);
}

fn apply_mitigation(
    effect: &WeatherEffect,
    cfg: &WeatherConfig,
    gs: &GameState,
) -> (i32, i32, i32) {
    let mut delta_san = effect.sanity;
    let mut delta_pants = effect.pants;
    if let Some(mitigation) = cfg
        .mitigation
        .get(&gs.weather_state.today)
        .filter(|m| gs.inventory.tags.contains(&m.tag))
    {
        if let Some(san) = mitigation.sanity {
            delta_san = san;
        }
        if let Some(pants) = mitigation.pants {
            delta_pants = pants;
        }
    }
    (effect.supplies, delta_san, delta_pants)
}

fn apply_exposure(gs: &mut GameState, today: Weather) {
    let has_heat_gear = gs.inventory.has_tag("water_jugs") || gs.inventory.has_tag("water");
    let has_cold_gear = gs.inventory.has_tag("warm_coat") || gs.inventory.has_tag("cold_resist");
    let heat_conditions = today == Weather::HeatWave && !has_heat_gear;
    let cold_conditions = today == Weather::ColdSnap && !has_cold_gear;

    let (hp_damage, exposure_kind) = if gs.features.exposure_streaks {
        apply_exposure_with_streak_lockout(gs, heat_conditions, cold_conditions)
    } else {
        apply_exposure_basic(gs, heat_conditions, cold_conditions)
    };

    if hp_damage > 0 {
        gs.stats.hp -= hp_damage;
        if let Some(kind) = exposure_kind.filter(|_| gs.stats.hp <= 0 && gs.ending.is_none()) {
            gs.ending = Some(Ending::Exposure { kind });
        }
    }
}

fn apply_exposure_with_streak_lockout(
    gs: &mut GameState,
    heat_conditions: bool,
    cold_conditions: bool,
) -> (i32, Option<ExposureKind>) {
    let mut hp_damage = 0;
    let mut exposure_kind: Option<ExposureKind> = None;

    if heat_conditions {
        gs.exposure_streak_heat = gs.exposure_streak_heat.saturating_add(1);
    } else {
        gs.exposure_streak_heat = 0;
    }
    if cold_conditions {
        gs.exposure_streak_cold = gs.exposure_streak_cold.saturating_add(1);
    } else {
        gs.exposure_streak_cold = 0;
    }

    let cold_trigger =
        cold_conditions && gs.exposure_streak_cold >= 3 && !gs.guards.exposure_damage_lockout;
    if cold_trigger {
        hp_damage = 1;
        gs.mark_damage(DamageCause::ExposureCold);
        gs.logs.push(String::from(LOG_WEATHER_EXPOSURE));
        exposure_kind = Some(ExposureKind::Cold);
    }

    let heat_trigger = if heat_conditions {
        gs.stats.sanity -= 1;
        gs.exposure_streak_heat >= 3 && !gs.guards.exposure_damage_lockout
    } else {
        false
    };
    if heat_trigger {
        hp_damage = 1;
        gs.mark_damage(DamageCause::ExposureHeat);
        gs.logs.push(String::from(LOG_WEATHER_HEATSTROKE));
        exposure_kind = Some(ExposureKind::Heat);
    }

    if gs.guards.exposure_damage_lockout && hp_damage == 0 {
        gs.guards.exposure_damage_lockout = false;
    } else {
        gs.guards.exposure_damage_lockout = cold_trigger || heat_trigger;
    }

    (hp_damage, exposure_kind)
}

fn apply_exposure_basic(
    gs: &mut GameState,
    heat_conditions: bool,
    cold_conditions: bool,
) -> (i32, Option<ExposureKind>) {
    let mut hp_damage = 0;
    let mut exposure_kind: Option<ExposureKind> = None;

    if cold_conditions {
        gs.exposure_streak_cold = gs.exposure_streak_cold.saturating_add(1);
        if gs.exposure_streak_cold >= 3 {
            hp_damage += 1;
            gs.mark_damage(DamageCause::ExposureCold);
            gs.logs.push(String::from(LOG_WEATHER_EXPOSURE));
            exposure_kind = Some(ExposureKind::Cold);
        }
    } else {
        gs.exposure_streak_cold = 0;
    }

    if heat_conditions {
        gs.exposure_streak_heat = gs.exposure_streak_heat.saturating_add(1);
        gs.stats.sanity -= 1;
        if gs.exposure_streak_heat >= 3 {
            hp_damage += 1;
            gs.mark_damage(DamageCause::ExposureHeat);
            gs.logs.push(String::from(LOG_WEATHER_HEATSTROKE));
            exposure_kind = Some(ExposureKind::Heat);
        }
    } else {
        gs.exposure_streak_heat = 0;
    }
    gs.guards.exposure_damage_lockout = false;

    (hp_damage, exposure_kind)
}

fn update_weather_streaks(state: &mut WeatherState, today: Weather) {
    let was_extreme = state.yesterday.is_extreme();
    let is_extreme = today.is_extreme();
    state.extreme_streak = if is_extreme {
        if was_extreme {
            state.extreme_streak + 1
        } else {
            1
        }
    } else {
        0
    };

    if today == Weather::HeatWave {
        state.heatwave_streak = state.heatwave_streak.saturating_add(1);
    } else {
        state.heatwave_streak = 0;
    }

    if today == Weather::ColdSnap {
        state.coldsnap_streak = state.coldsnap_streak.saturating_add(1);
    } else {
        state.coldsnap_streak = 0;
    }
}

/// Process daily weather step in game tick
pub fn process_daily_weather(gs: &mut GameState, cfg: &WeatherConfig, rngs: Option<&RngBundle>) {
    // Move today to yesterday
    gs.weather_state.yesterday = gs.weather_state.today;

    // Select new weather for today
    if let Some(rngs) = rngs
        && let Ok(weather) = select_weather_for_today(gs, cfg, rngs)
    {
        gs.weather_state.today = weather;
    }
    // If weather selection fails, keep previous weather

    // Apply effects
    apply_weather_effects(gs, cfg);
}
