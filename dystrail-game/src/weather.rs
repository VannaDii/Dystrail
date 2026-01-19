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
    #[serde(default)]
    pub rain_delta: f32,
    #[serde(default)]
    pub snow_delta: f32,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherState {
    pub today: Weather,
    pub yesterday: Weather,
    pub extreme_streak: i32,
    pub heatwave_streak: i32,
    pub coldsnap_streak: i32,
    pub neutral_buffer: u8,
    #[serde(default)]
    pub rain_accum: f32,
    #[serde(default)]
    pub snow_depth: f32,
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
            rain_accum: 0.0,
            snow_depth: 0.0,
        }
    }
}

/// Weather fan-out values used by downstream phases.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeatherEffects {
    pub travel_mult: f32,
    pub supplies_delta: i32,
    pub sanity_delta: i32,
    pub pants_delta: i32,
    pub encounter_delta: f32,
    pub encounter_cap: f32,
    pub breakdown_mult: f32,
    pub rain_accum_delta: f32,
    pub snow_depth_delta: f32,
}

impl Default for WeatherEffects {
    fn default() -> Self {
        Self {
            travel_mult: 1.0,
            supplies_delta: 0,
            sanity_delta: 0,
            pants_delta: 0,
            encounter_delta: 0.0,
            encounter_cap: 1.0,
            breakdown_mult: 1.0,
            rain_accum_delta: 0.0,
            snow_depth_delta: 0.0,
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
        .unwrap_or_else(|_| Self::fallback_config())
    }

    fn fallback_config() -> Self {
        Self {
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
        }
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
    let mut rng = rngs.weather();

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
pub fn apply_weather_effects(gs: &mut GameState, cfg: &WeatherConfig) -> WeatherEffects {
    let today = gs.weather_state.today;
    update_weather_streaks(&mut gs.weather_state, today);

    let Some(effect) = cfg.effects.get(&today) else {
        gs.weather_effects = WeatherEffects::default();
        gs.weather_travel_multiplier = gs.weather_effects.travel_mult;
        return gs.weather_effects;
    };

    let (delta_sup, delta_san, delta_pants) = apply_mitigation(effect, cfg, gs);
    let encounter_cap = if cfg.limits.encounter_cap > 0.0 {
        cfg.limits.encounter_cap
    } else {
        1.0
    };
    let breakdown_mult = gs
        .journey_breakdown
        .weather_factor
        .get(&today)
        .copied()
        .unwrap_or(1.0);
    let effects = WeatherEffects {
        travel_mult: effect.travel_mult.max(0.1),
        supplies_delta: delta_sup,
        sanity_delta: delta_san,
        pants_delta: delta_pants,
        encounter_delta: effect.enc_delta,
        encounter_cap,
        breakdown_mult,
        rain_accum_delta: effect.rain_delta,
        snow_depth_delta: effect.snow_delta,
    };

    gs.weather_travel_multiplier = effects.travel_mult;
    apply_stat_changes(
        gs,
        cfg,
        effects.supplies_delta,
        effects.sanity_delta,
        effects.pants_delta,
    );
    update_precip_accumulators(gs, effects);
    apply_exposure(gs, today);
    gs.weather_effects = effects;
    effects
}

fn apply_stat_changes(
    gs: &mut GameState,
    cfg: &WeatherConfig,
    delta_sup: i32,
    delta_san: i32,
    delta_pants: i32,
) {
    gs.stats.supplies += delta_sup;
    gs.stats.sanity += delta_san;
    gs.stats.pants =
        (gs.stats.pants + delta_pants).clamp(cfg.limits.pants_floor, cfg.limits.pants_ceiling);
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

fn update_precip_accumulators(gs: &mut GameState, effects: WeatherEffects) {
    let rain = gs.weather_state.rain_accum + effects.rain_accum_delta;
    let snow = gs.weather_state.snow_depth + effects.snow_depth_delta;
    gs.weather_state.rain_accum = rain.max(0.0);
    gs.weather_state.snow_depth = snow.max(0.0);
    gs.ot_deluxe.weather.rain_accum = gs.weather_state.rain_accum;
    gs.ot_deluxe.weather.snow_depth = gs.weather_state.snow_depth;
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
    let _ = apply_weather_effects(gs, cfg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journey::RngBundle;
    use crate::state::{GameMode, Season, Stats};
    use rand::RngCore;
    use std::collections::HashSet;

    struct FixedRng {
        value: u32,
    }

    impl FixedRng {
        const fn new(value: u32) -> Self {
            Self { value }
        }
    }

    impl RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 {
            self.value
        }

        fn next_u64(&mut self) -> u64 {
            u64::from(self.next_u32())
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let value = self.next_u32().to_le_bytes();
            for (idx, byte) in dest.iter_mut().enumerate() {
                *byte = value[idx % value.len()];
            }
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    fn base_config(weights: HashMap<Region, HashMap<Weather, u32>>) -> WeatherConfig {
        WeatherConfig {
            limits: WeatherLimits {
                max_extreme_streak: 1,
                encounter_cap: 0.4,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects: HashMap::new(),
            mitigation: HashMap::new(),
            weights,
            exec_mods: HashMap::new(),
        }
    }

    #[test]
    fn weather_keys_and_extremes_are_stable() {
        assert!(Weather::Storm.is_extreme());
        assert!(!Weather::ColdSnap.is_extreme());
        assert_eq!(Weather::HeatWave.i18n_key(), "weather.states.HeatWave");
    }

    #[test]
    fn weather_i18n_keys_cover_all_variants() {
        assert_eq!(Weather::Clear.i18n_key(), "weather.states.Clear");
        assert_eq!(Weather::Storm.i18n_key(), "weather.states.Storm");
        assert_eq!(Weather::ColdSnap.i18n_key(), "weather.states.ColdSnap");
        assert_eq!(Weather::Smoke.i18n_key(), "weather.states.Smoke");
    }

    #[test]
    fn seasonal_override_applies_when_rng_is_low() {
        let mut rng = FixedRng::new(0);
        assert_eq!(
            seasonal_override(Season::Winter, Weather::Clear, &mut rng),
            Weather::ColdSnap
        );
        assert_eq!(
            seasonal_override(Season::Summer, Weather::Clear, &mut rng),
            Weather::HeatWave
        );
        assert_eq!(
            seasonal_override(Season::Fall, Weather::Clear, &mut rng),
            Weather::Storm
        );
        assert_eq!(
            seasonal_override(Season::Spring, Weather::Clear, &mut rng),
            Weather::Smoke
        );
    }

    #[test]
    fn seasonal_override_keeps_current_when_rng_is_high() {
        let mut rng = FixedRng::new(u32::MAX);
        assert_eq!(
            seasonal_override(Season::Winter, Weather::Storm, &mut rng),
            Weather::Storm
        );
    }

    #[test]
    fn pick_neutral_weather_prefers_weighted_choice() {
        let weights = HashMap::from([(Weather::Clear, 0_u32), (Weather::Smoke, 5_u32)]);
        let mut rng = FixedRng::new(0);
        assert_eq!(pick_neutral_weather(&weights, &mut rng), Weather::Smoke);
    }

    #[test]
    fn pick_neutral_weather_returns_clear_when_smoke_weight_zero() {
        let weights = HashMap::from([(Weather::Clear, 4_u32), (Weather::Smoke, 0_u32)]);
        let mut rng = FixedRng::new(0);
        let weather = pick_neutral_weather(&weights, &mut rng);
        assert_eq!(weather, Weather::Clear);
    }

    #[test]
    fn apply_neutral_buffer_sets_length() {
        let weights = HashMap::from([(Weather::Clear, 5_u32), (Weather::Smoke, 0_u32)]);
        let mut rng = FixedRng::new(0);
        let mut buffer = 0_u8;
        let weather = apply_neutral_buffer(&mut buffer, &weights, &mut rng);
        assert!(matches!(weather, Weather::Clear | Weather::Smoke));
        assert!(buffer <= NEUTRAL_BUFFER_MAX.saturating_sub(1));
    }

    #[test]
    fn apply_weather_effects_returns_default_when_missing_effect() {
        let cfg = WeatherConfig {
            limits: WeatherLimits {
                max_extreme_streak: 1,
                encounter_cap: 0.3,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects: HashMap::new(),
            mitigation: HashMap::new(),
            weights: HashMap::new(),
            exec_mods: HashMap::new(),
        };
        let mut state = GameState::default();
        state.weather_state.today = Weather::Storm;

        let effects = apply_weather_effects(&mut state, &cfg);

        assert!((effects.travel_mult - 1.0).abs() <= f32::EPSILON);
        assert!((state.weather_travel_multiplier - effects.travel_mult).abs() <= f32::EPSILON);
    }

    #[test]
    fn apply_weather_effects_caps_encounter_when_limit_zero() {
        let mut effects_map = HashMap::new();
        effects_map.insert(
            Weather::Clear,
            WeatherEffect {
                supplies: 0,
                sanity: 0,
                pants: 0,
                enc_delta: 0.0,
                travel_mult: 1.0,
                rain_delta: 0.0,
                snow_delta: 0.0,
            },
        );
        let cfg = WeatherConfig {
            limits: WeatherLimits {
                max_extreme_streak: 1,
                encounter_cap: 0.0,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects: effects_map,
            mitigation: HashMap::new(),
            weights: HashMap::new(),
            exec_mods: HashMap::new(),
        };
        let mut state = GameState::default();
        state.weather_state.today = Weather::Clear;

        let effects = apply_weather_effects(&mut state, &cfg);
        assert!((effects.encounter_cap - 1.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn selection_avoids_extreme_when_streak_limit_reached() {
        let mut region_weights = HashMap::new();
        region_weights.insert(Weather::Storm, 10);
        region_weights.insert(Weather::Clear, 5);
        region_weights.insert(Weather::ColdSnap, 5);
        let mut weights = HashMap::new();
        weights.insert(Region::Heartland, region_weights);
        let cfg = WeatherConfig {
            limits: WeatherLimits {
                max_extreme_streak: 0,
                encounter_cap: 0.3,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects: HashMap::new(),
            mitigation: HashMap::new(),
            weights,
            exec_mods: HashMap::new(),
        };

        let rngs = RngBundle::from_user_seed(42);
        let mut state = GameState {
            region: Region::Heartland,
            weather_state: WeatherState {
                extreme_streak: 2,
                ..WeatherState::default()
            },
            ..GameState::default()
        };

        let selected = select_weather_for_today(&mut state, &cfg, &rngs).expect("select weather");
        assert!(!selected.is_extreme(), "should avoid extremes when capped");
    }

    #[test]
    fn selection_applies_neutral_buffer_when_heatwave_streak_hits_limit() {
        let mut region_weights = HashMap::new();
        region_weights.insert(Weather::Clear, 0);
        region_weights.insert(Weather::Storm, 0);
        region_weights.insert(Weather::HeatWave, 10);
        region_weights.insert(Weather::ColdSnap, 0);
        region_weights.insert(Weather::Smoke, 0);
        let weights = HashMap::from([(Region::Heartland, region_weights)]);
        let mut cfg = base_config(weights);
        cfg.limits.max_extreme_streak = 10;

        let rngs = RngBundle::from_user_seed(12);
        let mut state = GameState {
            region: Region::Heartland,
            season: Season::Summer,
            weather_state: WeatherState {
                heatwave_streak: HEATWAVE_MAX_STREAK,
                ..WeatherState::default()
            },
            ..GameState::default()
        };

        let weather = select_weather_for_today(&mut state, &cfg, &rngs).unwrap();
        assert_eq!(weather, Weather::Clear);
        assert!(state.weather_state.neutral_buffer > 0);
    }

    #[test]
    fn apply_weather_effects_uses_mitigation_and_updates_accumulators() {
        let mut state = GameState::default();
        state.weather_state.today = Weather::Storm;
        state.inventory.tags = HashSet::from([String::from("rain_gear")]);
        state.stats = Stats {
            supplies: 10,
            sanity: 10,
            pants: 10,
            ..Stats::default()
        };

        let mut effects = HashMap::new();
        effects.insert(
            Weather::Storm,
            WeatherEffect {
                supplies: -2,
                sanity: -3,
                pants: -4,
                enc_delta: 0.2,
                travel_mult: 0.8,
                rain_delta: 1.5,
                snow_delta: 0.0,
            },
        );
        let mut mitigation = HashMap::new();
        mitigation.insert(
            Weather::Storm,
            WeatherMitigation {
                tag: "rain_gear".into(),
                sanity: Some(-1),
                pants: Some(-2),
            },
        );

        let mut weights = HashMap::new();
        weights.insert(Region::Heartland, HashMap::new());
        let mut cfg = base_config(weights);
        cfg.effects = effects;
        cfg.mitigation = mitigation;

        let output = apply_weather_effects(&mut state, &cfg);
        assert_eq!(output.supplies_delta, -2);
        assert_eq!(output.sanity_delta, -1);
        assert_eq!(output.pants_delta, -2);
        assert!(state.weather_state.rain_accum > 0.0);
    }

    #[test]
    fn exposure_basic_and_streak_lockout_apply_damage() {
        let mut base_weights = HashMap::new();
        base_weights.insert(Region::Heartland, HashMap::new());
        let mut cfg = base_config(base_weights);
        cfg.effects.insert(
            Weather::HeatWave,
            WeatherEffect {
                supplies: 0,
                sanity: 0,
                pants: 0,
                enc_delta: 0.0,
                travel_mult: 1.0,
                rain_delta: 0.0,
                snow_delta: 0.0,
            },
        );
        cfg.effects.insert(
            Weather::ColdSnap,
            WeatherEffect {
                supplies: 0,
                sanity: 0,
                pants: 0,
                enc_delta: 0.0,
                travel_mult: 1.0,
                rain_delta: 0.0,
                snow_delta: 0.0,
            },
        );

        let mut state = GameState::default();
        state.weather_state.today = Weather::HeatWave;
        state.features.exposure_streaks = false;
        state.exposure_streak_heat = 2;
        state.stats.hp = 3;
        state.stats.sanity = 5;
        apply_weather_effects(&mut state, &cfg);
        assert!(state.stats.hp < 3);
        assert!(state.logs.contains(&String::from(LOG_WEATHER_HEATSTROKE)));

        state.weather_state.today = Weather::ColdSnap;
        state.features.exposure_streaks = true;
        state.exposure_streak_cold = 2;
        state.stats.hp = 3;
        state.logs.clear();
        apply_weather_effects(&mut state, &cfg);
        assert!(state.stats.hp < 3);
        assert!(state.logs.contains(&String::from(LOG_WEATHER_EXPOSURE)));
    }

    #[test]
    fn exposure_lockout_resets_when_no_damage() {
        let mut state = GameState::default();
        state.features.exposure_streaks = true;
        state.guards.exposure_damage_lockout = true;
        let (damage, _) = apply_exposure_with_streak_lockout(&mut state, false, false);
        assert_eq!(damage, 0);
        assert!(!state.guards.exposure_damage_lockout);
    }

    #[test]
    fn process_daily_weather_updates_yesterday_and_today() {
        let mut state = GameState {
            mode: GameMode::Classic,
            season: Season::Spring,
            weather_state: WeatherState {
                today: Weather::Clear,
                ..WeatherState::default()
            },
            ..GameState::default()
        };

        let cfg = WeatherConfig::default_config();
        let rngs = RngBundle::from_user_seed(7);
        process_daily_weather(&mut state, &cfg, Some(&rngs));

        assert_eq!(state.weather_state.yesterday, Weather::Clear);
    }

    #[test]
    fn weather_weight_returns_zero_for_missing_key() {
        let weights = HashMap::from([(Weather::Clear, 3_u32)]);
        assert_eq!(weather_weight(&weights, Weather::Storm), 0);
    }

    #[test]
    fn select_weather_errors_when_region_missing() {
        let mut cfg = WeatherConfig::default_config();
        cfg.weights.clear();
        let rngs = RngBundle::from_user_seed(12);
        let mut state = GameState {
            region: Region::Beltway,
            ..GameState::default()
        };
        let err = select_weather_for_today(&mut state, &cfg, &rngs).unwrap_err();
        assert!(err.contains("Weather weights must exist"));
    }

    #[test]
    fn weather_config_from_json_detects_missing_effects() {
        let json = serde_json::json!({
            "limits": {
                "max_extreme_streak": 1,
                "encounter_cap": 0.3,
                "pants_floor": 0,
                "pants_ceiling": 100
            },
            "effects": {
                "Clear": {
                    "supplies": 0,
                    "sanity": 0,
                    "pants": 0,
                    "enc_delta": 0.0
                }
            },
            "mitigation": {},
            "weights": {
                "Heartland": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1},
                "RustBelt": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1},
                "Beltway": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1}
            },
            "exec_mods": {}
        })
        .to_string();

        let err = WeatherConfig::from_json(&json).unwrap_err();
        assert!(err.contains("Missing effect for weather"));
    }

    #[test]
    fn weather_config_from_json_parses_defaults() {
        let json = include_str!("../../dystrail-web/static/assets/data/weather.json");
        let cfg = WeatherConfig::from_json(json).expect("expected valid weather config");
        assert!(cfg.effects.contains_key(&Weather::Clear));
        assert!(cfg.weights.contains_key(&Region::Heartland));
    }

    #[test]
    fn weather_config_from_json_detects_missing_weight() {
        let json = serde_json::json!({
            "limits": {
                "max_extreme_streak": 1,
                "encounter_cap": 0.3,
                "pants_floor": 0,
                "pants_ceiling": 100
            },
            "effects": {
                "Clear": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "Storm": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "HeatWave": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "ColdSnap": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "Smoke": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0}
            },
            "mitigation": {},
            "weights": {
                "Heartland": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1},
                "RustBelt": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1},
                "Beltway": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1}
            },
            "exec_mods": {}
        })
        .to_string();

        let err = WeatherConfig::from_json(&json).unwrap_err();
        assert!(err.contains("Missing weight for"));
    }

    #[test]
    fn weather_config_from_json_detects_missing_region_weights() {
        let json = serde_json::json!({
            "limits": {
                "max_extreme_streak": 1,
                "encounter_cap": 0.3,
                "pants_floor": 0,
                "pants_ceiling": 100
            },
            "effects": {
                "Clear": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "Storm": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "HeatWave": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "ColdSnap": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "Smoke": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0}
            },
            "mitigation": {},
            "weights": {
                "Heartland": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1},
                "RustBelt": {"Clear": 1, "Storm": 1, "HeatWave": 1, "ColdSnap": 1, "Smoke": 1}
            },
            "exec_mods": {}
        })
        .to_string();

        let err = WeatherConfig::from_json(&json).unwrap_err();
        assert!(err.contains("Missing weights for region"));
    }

    #[test]
    fn fallback_weather_config_is_empty() {
        let cfg = WeatherConfig::fallback_config();
        assert!(cfg.effects.is_empty());
        assert!(cfg.weights.is_empty());
        assert!((cfg.limits.encounter_cap - 0.35).abs() <= f32::EPSILON);
    }
}
