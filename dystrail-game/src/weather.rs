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
const fn default_very_hot_min_f() -> i16 {
    91
}
const fn default_hot_min_f() -> i16 {
    70
}
const fn default_warm_min_f() -> i16 {
    50
}
const fn default_cool_min_f() -> i16 {
    30
}
const fn default_cold_min_f() -> i16 {
    10
}
const fn default_heavy_precip_in() -> f32 {
    0.5
}
const fn default_snow_temp_f() -> i16 {
    32
}
const fn default_rain_evap_rate() -> f32 {
    0.15
}
const fn default_snow_evap_rate() -> f32 {
    0.05
}
const fn default_snow_melt_rate() -> f32 {
    0.25
}
const fn default_melt_temp_f() -> i16 {
    40
}

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

const fn base_temperature_f(season: Season) -> i16 {
    match season {
        Season::Winter => 25,
        Season::Spring => 55,
        Season::Summer => 80,
        Season::Fall => 50,
    }
}

const fn weather_temperature_delta_f(weather: Weather) -> i16 {
    match weather {
        Weather::Clear => 0,
        Weather::Storm => -8,
        Weather::HeatWave => 15,
        Weather::ColdSnap => -25,
        Weather::Smoke => 5,
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
    #[serde(default)]
    pub report: WeatherReportConfig,
    #[serde(default)]
    pub accumulation: WeatherAccumulationConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherReportLabel {
    VeryHot,
    Hot,
    Warm,
    Cool,
    Cold,
    VeryCold,
    Rainy,
    VeryRainy,
    Snowy,
    VerySnowy,
}

impl WeatherReportLabel {
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::VeryHot => "weather.report.very_hot",
            Self::Hot => "weather.report.hot",
            Self::Warm => "weather.report.warm",
            Self::Cool => "weather.report.cool",
            Self::Cold => "weather.report.cold",
            Self::VeryCold => "weather.report.very_cold",
            Self::Rainy => "weather.report.rainy",
            Self::VeryRainy => "weather.report.very_rainy",
            Self::Snowy => "weather.report.snowy",
            Self::VerySnowy => "weather.report.very_snowy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherTempBands {
    #[serde(default = "default_very_hot_min_f")]
    pub very_hot_min_f: i16,
    #[serde(default = "default_hot_min_f")]
    pub hot_min_f: i16,
    #[serde(default = "default_warm_min_f")]
    pub warm_min_f: i16,
    #[serde(default = "default_cool_min_f")]
    pub cool_min_f: i16,
    #[serde(default = "default_cold_min_f")]
    pub cold_min_f: i16,
}

impl Default for WeatherTempBands {
    fn default() -> Self {
        Self {
            very_hot_min_f: default_very_hot_min_f(),
            hot_min_f: default_hot_min_f(),
            warm_min_f: default_warm_min_f(),
            cool_min_f: default_cool_min_f(),
            cold_min_f: default_cold_min_f(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherReportConfig {
    #[serde(default = "default_heavy_precip_in")]
    pub heavy_precip_in: f32,
    #[serde(default = "default_snow_temp_f")]
    pub snow_temp_f: i16,
    #[serde(default)]
    pub temp_bands: WeatherTempBands,
}

impl Default for WeatherReportConfig {
    fn default() -> Self {
        Self {
            heavy_precip_in: default_heavy_precip_in(),
            snow_temp_f: default_snow_temp_f(),
            temp_bands: WeatherTempBands::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherAccumulationConfig {
    #[serde(default = "default_rain_evap_rate")]
    pub rain_evap_rate: f32,
    #[serde(default = "default_snow_evap_rate")]
    pub snow_evap_rate: f32,
    #[serde(default = "default_snow_melt_rate")]
    pub snow_melt_rate: f32,
    #[serde(default = "default_melt_temp_f")]
    pub melt_temp_f: i16,
}

impl Default for WeatherAccumulationConfig {
    fn default() -> Self {
        Self {
            rain_evap_rate: default_rain_evap_rate(),
            snow_evap_rate: default_snow_evap_rate(),
            snow_melt_rate: default_snow_melt_rate(),
            melt_temp_f: default_melt_temp_f(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeatherSample {
    pub weather: Weather,
    pub temperature_f: i16,
    pub precip_in: f32,
}

pub trait WeatherModel {
    /// Generate weather for the current day.
    ///
    /// # Errors
    ///
    /// Returns an error if required configuration for the current context is missing.
    fn generate_weather_today(
        &self,
        gs: &mut GameState,
        rngs: &RngBundle,
    ) -> Result<WeatherSample, String>;
    fn apply_weather_effects(&self, gs: &mut GameState, sample: WeatherSample) -> WeatherEffects;
    fn sample_from_weather(&self, gs: &GameState, weather: Weather) -> WeatherSample;
}

#[derive(Debug, Clone)]
pub struct DystrailRegionalWeather {
    config: WeatherConfig,
}

impl DystrailRegionalWeather {
    #[must_use]
    pub const fn new(config: WeatherConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub const fn config(&self) -> &WeatherConfig {
        &self.config
    }
}

impl Default for DystrailRegionalWeather {
    fn default() -> Self {
        Self::new(WeatherConfig::default_config())
    }
}

impl WeatherModel for DystrailRegionalWeather {
    fn generate_weather_today(
        &self,
        gs: &mut GameState,
        rngs: &RngBundle,
    ) -> Result<WeatherSample, String> {
        let weather = select_weather_for_today(gs, &self.config, rngs)?;
        Ok(self.sample_from_weather(gs, weather))
    }

    fn apply_weather_effects(&self, gs: &mut GameState, sample: WeatherSample) -> WeatherEffects {
        apply_weather_effects(gs, &self.config, sample)
    }

    fn sample_from_weather(&self, gs: &GameState, weather: Weather) -> WeatherSample {
        let base_temp = base_temperature_f(gs.season);
        let delta = weather_temperature_delta_f(weather);
        let temp_f = base_temp.saturating_add(delta);
        let (rain, snow) = self
            .config
            .effects
            .get(&weather)
            .map_or((0.0, 0.0), |effect| (effect.rain_delta, effect.snow_delta));
        let mut precip = rain + snow;
        if !precip.is_finite() {
            precip = 0.0;
        }
        let precip = precip.max(0.0);
        WeatherSample {
            weather,
            temperature_f: temp_f,
            precip_in: precip,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OtDeluxeStationsWeather {
    report: WeatherReportConfig,
    accumulation: WeatherAccumulationConfig,
}

impl WeatherModel for OtDeluxeStationsWeather {
    fn generate_weather_today(
        &self,
        gs: &mut GameState,
        _rngs: &RngBundle,
    ) -> Result<WeatherSample, String> {
        Ok(WeatherSample {
            weather: Weather::Clear,
            temperature_f: base_temperature_f(gs.season),
            precip_in: 0.0,
        })
    }

    fn apply_weather_effects(&self, gs: &mut GameState, sample: WeatherSample) -> WeatherEffects {
        apply_weather_report(gs, sample, &self.report);
        let mut effects = WeatherEffects::default();
        let (rain_delta, snow_delta) =
            update_precip_accumulators(gs, sample, &self.report, &self.accumulation);
        effects.rain_accum_delta = rain_delta;
        effects.snow_depth_delta = snow_delta;
        gs.weather_effects = effects;
        effects
    }

    fn sample_from_weather(&self, gs: &GameState, _weather: Weather) -> WeatherSample {
        WeatherSample {
            weather: Weather::Clear,
            temperature_f: base_temperature_f(gs.season),
            precip_in: 0.0,
        }
    }
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

        if !self.report.heavy_precip_in.is_finite() || self.report.heavy_precip_in < 0.0 {
            return Err(String::from("Report heavy_precip_in must be non-negative"));
        }
        if !self.accumulation.rain_evap_rate.is_finite() || self.accumulation.rain_evap_rate < 0.0 {
            return Err(String::from(
                "Accumulation rain_evap_rate must be non-negative",
            ));
        }
        if !self.accumulation.snow_evap_rate.is_finite() || self.accumulation.snow_evap_rate < 0.0 {
            return Err(String::from(
                "Accumulation snow_evap_rate must be non-negative",
            ));
        }
        if !self.accumulation.snow_melt_rate.is_finite() || self.accumulation.snow_melt_rate < 0.0 {
            return Err(String::from(
                "Accumulation snow_melt_rate must be non-negative",
            ));
        }
        let bands = &self.report.temp_bands;
        if bands.cold_min_f > bands.cool_min_f
            || bands.cool_min_f > bands.warm_min_f
            || bands.warm_min_f > bands.hot_min_f
            || bands.hot_min_f > bands.very_hot_min_f
        {
            return Err(String::from("Report temperature bands must be ascending"));
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
            report: WeatherReportConfig::default(),
            accumulation: WeatherAccumulationConfig::default(),
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
pub fn apply_weather_effects(
    gs: &mut GameState,
    cfg: &WeatherConfig,
    sample: WeatherSample,
) -> WeatherEffects {
    let today = gs.weather_state.today;
    update_weather_streaks(&mut gs.weather_state, today);

    apply_weather_report(gs, sample, &cfg.report);

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
    let (rain_accum_delta, snow_depth_delta) =
        update_precip_accumulators(gs, sample, &cfg.report, &cfg.accumulation);
    let effects = WeatherEffects {
        travel_mult: effect.travel_mult.max(0.1),
        supplies_delta: delta_sup,
        sanity_delta: delta_san,
        pants_delta: delta_pants,
        encounter_delta: effect.enc_delta,
        encounter_cap,
        breakdown_mult,
        rain_accum_delta,
        snow_depth_delta,
    };

    gs.weather_travel_multiplier = effects.travel_mult;
    apply_stat_changes(
        gs,
        cfg,
        effects.supplies_delta,
        effects.sanity_delta,
        effects.pants_delta,
    );
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

fn apply_weather_report(gs: &mut GameState, sample: WeatherSample, report: &WeatherReportConfig) {
    let label = derive_weather_report_label(sample, report);
    gs.ot_deluxe.weather.today.temperature_f = sample.temperature_f;
    gs.ot_deluxe.weather.today.precip_in = sample.precip_in;
    gs.ot_deluxe.weather.today.label = label.as_key().to_string();
}

fn derive_weather_report_label(
    sample: WeatherSample,
    report: &WeatherReportConfig,
) -> WeatherReportLabel {
    let precip = if sample.precip_in.is_finite() {
        sample.precip_in.max(0.0)
    } else {
        0.0
    };
    if precip > 0.0 {
        let heavy = precip >= report.heavy_precip_in;
        let snow = sample.temperature_f <= report.snow_temp_f;
        return match (snow, heavy) {
            (true, true) => WeatherReportLabel::VerySnowy,
            (true, false) => WeatherReportLabel::Snowy,
            (false, true) => WeatherReportLabel::VeryRainy,
            (false, false) => WeatherReportLabel::Rainy,
        };
    }

    let bands = &report.temp_bands;
    let temp_f = sample.temperature_f;
    if temp_f > bands.very_hot_min_f {
        WeatherReportLabel::VeryHot
    } else if temp_f >= bands.hot_min_f {
        WeatherReportLabel::Hot
    } else if temp_f >= bands.warm_min_f {
        WeatherReportLabel::Warm
    } else if temp_f >= bands.cool_min_f {
        WeatherReportLabel::Cool
    } else if temp_f >= bands.cold_min_f {
        WeatherReportLabel::Cold
    } else {
        WeatherReportLabel::VeryCold
    }
}

fn update_precip_accumulators(
    gs: &mut GameState,
    sample: WeatherSample,
    report: &WeatherReportConfig,
    accumulation: &WeatherAccumulationConfig,
) -> (f32, f32) {
    let rain_before = gs.weather_state.rain_accum;
    let snow_before = gs.weather_state.snow_depth;
    let mut rain_accum = rain_before;
    let mut snow_depth = snow_before;
    let precip = sample.precip_in.max(0.0);

    if precip > 0.0 {
        if sample.temperature_f <= report.snow_temp_f {
            snow_depth += precip;
        } else {
            rain_accum += precip;
        }
    }

    if sample.temperature_f >= accumulation.melt_temp_f && snow_depth > 0.0 {
        let melt_amount = accumulation.snow_melt_rate.min(snow_depth);
        snow_depth -= melt_amount;
        rain_accum += melt_amount;
    }

    rain_accum = (rain_accum - accumulation.rain_evap_rate).max(0.0);
    snow_depth = (snow_depth - accumulation.snow_evap_rate).max(0.0);

    gs.weather_state.rain_accum = rain_accum;
    gs.weather_state.snow_depth = snow_depth;
    gs.ot_deluxe.weather.rain_accum = rain_accum;
    gs.ot_deluxe.weather.snow_depth = snow_depth;

    (rain_accum - rain_before, snow_depth - snow_before)
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
        gs.push_log(LOG_WEATHER_EXPOSURE);
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
        gs.push_log(LOG_WEATHER_HEATSTROKE);
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
            gs.push_log(LOG_WEATHER_EXPOSURE);
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
            gs.push_log(LOG_WEATHER_HEATSTROKE);
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
pub fn process_daily_weather(
    gs: &mut GameState,
    model: &impl WeatherModel,
    rngs: Option<&RngBundle>,
) {
    // Move today to yesterday
    gs.weather_state.yesterday = gs.weather_state.today;

    let sample = if let Some(rngs) = rngs
        && let Ok(new_sample) = model.generate_weather_today(gs, rngs)
    {
        gs.weather_state.today = new_sample.weather;
        new_sample
    } else {
        model.sample_from_weather(gs, gs.weather_state.today)
    };

    let _ = model.apply_weather_effects(gs, sample);
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
            report: WeatherReportConfig::default(),
            accumulation: WeatherAccumulationConfig::default(),
        }
    }

    fn sample_with_temp(weather: Weather, temp_f: i16, precip_in: f32) -> WeatherSample {
        WeatherSample {
            weather,
            temperature_f: temp_f,
            precip_in,
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
            report: WeatherReportConfig::default(),
            accumulation: WeatherAccumulationConfig::default(),
        };
        let mut state = GameState::default();
        state.weather_state.today = Weather::Storm;

        let sample = sample_with_temp(Weather::Storm, base_temperature_f(state.season), 0.0);
        let effects = apply_weather_effects(&mut state, &cfg, sample);

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
            report: WeatherReportConfig::default(),
            accumulation: WeatherAccumulationConfig::default(),
        };
        let mut state = GameState::default();
        state.weather_state.today = Weather::Clear;

        let sample = sample_with_temp(Weather::Clear, base_temperature_f(state.season), 0.0);
        let effects = apply_weather_effects(&mut state, &cfg, sample);
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
            report: WeatherReportConfig::default(),
            accumulation: WeatherAccumulationConfig::default(),
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

        let sample = sample_with_temp(Weather::Storm, 60, 1.5);
        let output = apply_weather_effects(&mut state, &cfg, sample);
        assert_eq!(output.supplies_delta, -2);
        assert_eq!(output.sanity_delta, -1);
        assert_eq!(output.pants_delta, -2);
        assert!(state.weather_state.rain_accum > 0.0);
        assert_eq!(
            state.ot_deluxe.weather.today.label,
            WeatherReportLabel::VeryRainy.as_key()
        );
    }

    #[test]
    fn weather_report_label_prefers_precip_over_temp() {
        let report = WeatherReportConfig {
            heavy_precip_in: 0.5,
            snow_temp_f: 32,
            temp_bands: WeatherTempBands::default(),
        };

        let rainy = derive_weather_report_label(sample_with_temp(Weather::Storm, 60, 0.4), &report);
        assert_eq!(rainy, WeatherReportLabel::Rainy);

        let very_rainy =
            derive_weather_report_label(sample_with_temp(Weather::Storm, 60, 0.6), &report);
        assert_eq!(very_rainy, WeatherReportLabel::VeryRainy);

        let snowy = derive_weather_report_label(sample_with_temp(Weather::Storm, 30, 0.4), &report);
        assert_eq!(snowy, WeatherReportLabel::Snowy);

        let very_snowy =
            derive_weather_report_label(sample_with_temp(Weather::Storm, 30, 0.6), &report);
        assert_eq!(very_snowy, WeatherReportLabel::VerySnowy);
    }

    #[test]
    fn weather_report_label_uses_temp_bands() {
        let report = WeatherReportConfig::default();

        assert_eq!(
            derive_weather_report_label(sample_with_temp(Weather::Clear, 95, 0.0), &report),
            WeatherReportLabel::VeryHot
        );
        assert_eq!(
            derive_weather_report_label(sample_with_temp(Weather::Clear, 80, 0.0), &report),
            WeatherReportLabel::Hot
        );
        assert_eq!(
            derive_weather_report_label(sample_with_temp(Weather::Clear, 60, 0.0), &report),
            WeatherReportLabel::Warm
        );
        assert_eq!(
            derive_weather_report_label(sample_with_temp(Weather::Clear, 45, 0.0), &report),
            WeatherReportLabel::Cool
        );
        assert_eq!(
            derive_weather_report_label(sample_with_temp(Weather::Clear, 20, 0.0), &report),
            WeatherReportLabel::Cold
        );
        assert_eq!(
            derive_weather_report_label(sample_with_temp(Weather::Clear, 5, 0.0), &report),
            WeatherReportLabel::VeryCold
        );
    }

    #[test]
    fn precip_accumulators_apply_melt_and_evap() {
        let report = WeatherReportConfig {
            heavy_precip_in: 0.5,
            snow_temp_f: 32,
            temp_bands: WeatherTempBands::default(),
        };
        let accumulation = WeatherAccumulationConfig {
            rain_evap_rate: 0.2,
            snow_evap_rate: 0.1,
            snow_melt_rate: 0.5,
            melt_temp_f: 40,
        };
        let mut state = GameState::default();
        state.weather_state.rain_accum = 1.0;
        state.weather_state.snow_depth = 1.0;

        let sample = sample_with_temp(Weather::Storm, 50, 0.4);
        let (rain_delta, snow_delta) =
            update_precip_accumulators(&mut state, sample, &report, &accumulation);

        assert!((state.weather_state.rain_accum - 1.7).abs() < 0.01);
        assert!((state.weather_state.snow_depth - 0.4).abs() < 0.01);
        assert!((rain_delta - 0.7).abs() < 0.01);
        assert!((snow_delta + 0.6).abs() < 0.01);
        assert!((state.ot_deluxe.weather.rain_accum - state.weather_state.rain_accum).abs() < 0.01);
        assert!((state.ot_deluxe.weather.snow_depth - state.weather_state.snow_depth).abs() < 0.01);
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
        let sample = sample_with_temp(Weather::HeatWave, 96, 0.0);
        apply_weather_effects(&mut state, &cfg, sample);
        assert!(state.stats.hp < 3);
        assert!(state.logs.contains(&String::from(LOG_WEATHER_HEATSTROKE)));

        state.weather_state.today = Weather::ColdSnap;
        state.features.exposure_streaks = true;
        state.exposure_streak_cold = 2;
        state.stats.hp = 3;
        state.logs.clear();
        let sample = sample_with_temp(Weather::ColdSnap, 5, 0.0);
        apply_weather_effects(&mut state, &cfg, sample);
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

        let model = DystrailRegionalWeather::default();
        let rngs = RngBundle::from_user_seed(7);
        process_daily_weather(&mut state, &model, Some(&rngs));

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
    fn weather_config_validation_rejects_bad_temp_bands() {
        let mut cfg = WeatherConfig::default_config();
        cfg.report.temp_bands.cold_min_f = 40;
        cfg.report.temp_bands.cool_min_f = 30;
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("temperature bands"));
    }

    #[test]
    fn weather_config_from_json_parses_defaults() {
        let json = include_str!("../../dystrail-web/static/assets/data/weather.json");
        let cfg = WeatherConfig::from_json(json).expect("expected valid weather config");
        assert!(cfg.effects.contains_key(&Weather::Clear));
        assert!(cfg.weights.contains_key(&Region::Heartland));
        let storm = cfg.effects.get(&Weather::Storm).expect("storm effect");
        assert!((storm.rain_delta - 0.4).abs() < 0.01);
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
