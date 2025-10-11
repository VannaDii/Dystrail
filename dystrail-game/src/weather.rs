//! Weather system and effects
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::state::{DamageCause, GameState, Region, Season};

const LOG_WEATHER_EXPOSURE: &str = "log.weather.exposure";
const LOG_WEATHER_HEATSTROKE: &str = "log.weather.heatstroke";

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
    pub fn is_extreme(self) -> bool {
        matches!(self, Weather::Storm | Weather::HeatWave | Weather::Smoke)
    }

    /// Get i18n key for weather state name
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            Weather::Clear => "weather.states.Clear",
            Weather::Storm => "weather.states.Storm",
            Weather::HeatWave => "weather.states.HeatWave",
            Weather::ColdSnap => "weather.states.ColdSnap",
            Weather::Smoke => "weather.states.Smoke",
        }
    }
}

/// Daily effects from weather conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherEffect {
    pub supplies: i32,
    pub sanity: i32,
    pub pants: i32,
    pub enc_delta: f32,
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
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            today: Weather::Clear,
            yesterday: Weather::Clear,
            extreme_streak: 0,
        }
    }
}

impl WeatherConfig {
    /// Load weather configuration from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string cannot be parsed or if validation fails.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: WeatherConfig =
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
    #[allow(clippy::too_many_lines)]
    pub fn default_config() -> Self {
        use std::collections::HashMap;

        let mut effects = HashMap::new();
        effects.insert(
            Weather::Clear,
            WeatherEffect {
                supplies: 0,
                sanity: 0,
                pants: 0,
                enc_delta: 0.00,
            },
        );
        effects.insert(
            Weather::Storm,
            WeatherEffect {
                supplies: 1,
                sanity: -1,
                pants: 2,
                enc_delta: 0.05,
            },
        );
        effects.insert(
            Weather::HeatWave,
            WeatherEffect {
                supplies: 1,
                sanity: -1,
                pants: 1,
                enc_delta: 0.03,
            },
        );
        effects.insert(
            Weather::ColdSnap,
            WeatherEffect {
                supplies: 0,
                sanity: -1,
                pants: 0,
                enc_delta: 0.00,
            },
        );
        effects.insert(
            Weather::Smoke,
            WeatherEffect {
                supplies: 1,
                sanity: -1,
                pants: 1,
                enc_delta: 0.03,
            },
        );

        let mut mitigation = HashMap::new();
        mitigation.insert(
            Weather::ColdSnap,
            WeatherMitigation {
                tag: "cold_resist".to_string(),
                sanity: Some(0),
                pants: None,
            },
        );
        mitigation.insert(
            Weather::Smoke,
            WeatherMitigation {
                tag: "plague_resist".to_string(),
                sanity: Some(0),
                pants: None,
            },
        );
        mitigation.insert(
            Weather::Storm,
            WeatherMitigation {
                tag: "rain_resist".to_string(),
                sanity: None,
                pants: Some(1),
            },
        );

        let mut weights = HashMap::new();

        let mut heartland = HashMap::new();
        heartland.insert(Weather::Clear, 60);
        heartland.insert(Weather::Storm, 15);
        heartland.insert(Weather::HeatWave, 10);
        heartland.insert(Weather::ColdSnap, 10);
        heartland.insert(Weather::Smoke, 5);
        weights.insert(Region::Heartland, heartland);

        let mut rust_belt = HashMap::new();
        rust_belt.insert(Weather::Clear, 55);
        rust_belt.insert(Weather::Storm, 20);
        rust_belt.insert(Weather::HeatWave, 8);
        rust_belt.insert(Weather::ColdSnap, 12);
        rust_belt.insert(Weather::Smoke, 5);
        weights.insert(Region::RustBelt, rust_belt);

        let mut beltway = HashMap::new();
        beltway.insert(Weather::Clear, 50);
        beltway.insert(Weather::Storm, 25);
        beltway.insert(Weather::HeatWave, 10);
        beltway.insert(Weather::ColdSnap, 10);
        beltway.insert(Weather::Smoke, 5);
        weights.insert(Region::Beltway, beltway);

        let mut exec_mods = HashMap::new();
        exec_mods.insert(
            "TariffTsunami".to_string(),
            ExecWeatherMod { enc_delta: 0.00 },
        );
        exec_mods.insert(
            "NatGuardDeployment".to_string(),
            ExecWeatherMod { enc_delta: 0.02 },
        );
        exec_mods.insert(
            "DoEEliminated".to_string(),
            ExecWeatherMod { enc_delta: 0.00 },
        );
        exec_mods.insert("WarDept".to_string(), ExecWeatherMod { enc_delta: 0.00 });

        Self {
            limits: WeatherLimits {
                max_extreme_streak: 3,
                encounter_cap: 1.0,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects,
            mitigation,
            weights,
            exec_mods,
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
) -> Result<Weather, String> {
    let Some(rng) = gs.rng.as_mut() else {
        return Err("RNG must be initialized".to_string());
    };

    let Some(region_weights) = cfg.weights.get(&gs.region) else {
        return Err(format!(
            "Weather weights must exist for region {:?}",
            gs.region
        ));
    };

    // Calculate total weight and make initial selection
    let total: u32 = region_weights.values().sum();
    let mut roll = rng.random_range(0..total);
    let mut candidate = Weather::Clear;

    for (weather, weight) in region_weights {
        if roll < *weight {
            candidate = *weather;
            break;
        }
        roll -= *weight;
    }

    // Enforce extreme streak limit
    if candidate.is_extreme() && gs.weather_state.extreme_streak >= cfg.limits.max_extreme_streak {
        // Reselect from non-extremes deterministically
        let non_extreme_total: u32 = region_weights
            .iter()
            .filter(|(w, _)| !w.is_extreme())
            .map(|(_, wt)| *wt)
            .sum();

        if non_extreme_total > 0 {
            let mut r2 = rng.random_range(0..non_extreme_total);
            for (weather, weight) in region_weights {
                if weather.is_extreme() {
                    continue;
                }
                if r2 < *weight {
                    candidate = *weather;
                    break;
                }
                r2 -= *weight;
            }
        }
    }

    let seasonal_candidate = seasonal_override(gs.season, candidate, rng);
    let mut final_weather = seasonal_candidate;
    if seasonal_candidate.is_extreme()
        && gs.weather_state.extreme_streak >= cfg.limits.max_extreme_streak
    {
        final_weather = Weather::Clear;
    }

    Ok(final_weather)
}

fn seasonal_override<R: Rng>(season: Season, current: Weather, rng: &mut R) -> Weather {
    match season {
        Season::Winter => {
            if rng.random::<f32>() < 0.20 {
                Weather::ColdSnap
            } else {
                current
            }
        }
        Season::Summer => {
            if rng.random::<f32>() < 0.20 {
                Weather::HeatWave
            } else {
                current
            }
        }
        Season::Fall => {
            if rng.random::<f32>() < 0.15 {
                Weather::Storm
            } else {
                current
            }
        }
        Season::Spring => {
            if rng.random::<f32>() < 0.12 {
                Weather::Smoke
            } else {
                current
            }
        }
    }
}

/// Apply weather effects to game state
pub fn apply_weather_effects(gs: &mut GameState, cfg: &WeatherConfig) {
    // Update streak counters
    let today = gs.weather_state.today;
    let was_extreme = gs.weather_state.yesterday.is_extreme();
    let is_extreme = today.is_extreme();

    gs.weather_state.extreme_streak = if is_extreme {
        if was_extreme {
            gs.weather_state.extreme_streak + 1
        } else {
            1
        }
    } else {
        0
    };

    // Get base effect
    let Some(effect) = cfg.effects.get(&today) else {
        // Return early with no effects if configuration is missing
        return;
    };

    let delta_sup = effect.supplies;
    let mut delta_san = effect.sanity;
    let mut delta_pants = effect.pants;
    let delta_enc = effect.enc_delta;

    // Apply gear mitigation
    if let Some(mitigation) = cfg.mitigation.get(&today)
        && gs.inventory.tags.contains(&mitigation.tag)
    {
        if let Some(san) = mitigation.sanity {
            delta_san = san;
        }
        if let Some(pants) = mitigation.pants {
            delta_pants = pants;
        }
    }

    // Apply stat changes
    gs.stats.supplies += delta_sup;
    gs.stats.sanity += delta_san;
    gs.stats.pants =
        (gs.stats.pants + delta_pants).clamp(cfg.limits.pants_floor, cfg.limits.pants_ceiling);

    let mut hp_damage = 0;
    match today {
        Weather::ColdSnap => {
            if !gs.inventory.has_tag("warm_coat") {
                hp_damage += 1;
                gs.mark_damage(DamageCause::ExposureCold);
                gs.logs.push(String::from(LOG_WEATHER_EXPOSURE));
            }
        }
        Weather::HeatWave => {
            if !gs.inventory.has_tag("water_jugs") {
                hp_damage += 1;
                gs.stats.sanity -= 1;
                gs.mark_damage(DamageCause::ExposureHeat);
                gs.logs.push(String::from(LOG_WEATHER_HEATSTROKE));
            }
        }
        _ => {}
    }
    if hp_damage > 0 {
        gs.stats.hp -= hp_damage;
    }

    // Add weather encounter chance delta
    gs.encounter_chance_today =
        (gs.encounter_chance_today + delta_enc).clamp(0.0, cfg.limits.encounter_cap);

    // Apply executive order modifiers if applicable
    let exec_order_key = format!("{:?}", gs.current_order);
    if let Some(exec_mod) = cfg.exec_mods.get(&exec_order_key) {
        gs.encounter_chance_today =
            (gs.encounter_chance_today + exec_mod.enc_delta).clamp(0.0, cfg.limits.encounter_cap);
    }
}

/// Process daily weather step in game tick
pub fn process_daily_weather(gs: &mut GameState, cfg: &WeatherConfig) {
    // Move today to yesterday
    gs.weather_state.yesterday = gs.weather_state.today;

    // Select new weather for today
    if let Ok(weather) = select_weather_for_today(gs, cfg) {
        gs.weather_state.today = weather;
    }
    // If weather selection fails, keep previous weather

    // Apply effects
    apply_weather_effects(gs, cfg);
}
