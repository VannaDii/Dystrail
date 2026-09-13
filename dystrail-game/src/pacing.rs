//! Pace and diet system
use serde::{Deserialize, Serialize};

const DEFAULT_PACING_DATA: &str = include_str!("../../dystrail-web/static/assets/data/pacing.json");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PacingConfig {
    #[serde(default)]
    pub pace: Vec<PaceCfg>,
    #[serde(default)]
    pub diet: Vec<DietCfg>,
    #[serde(default)]
    pub limits: PacingLimits,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaceCfg {
    pub id: String,
    pub name: String,
    #[serde(default = "default_speed_mph")]
    pub speed_mph: f32,
    #[serde(default)]
    /// Direct fatigue over 300 driving minutes; parked time does not incur this penalty.
    pub sanity: i32,
    #[serde(default)]
    pub encounter_chance_delta: f32,
}

impl Default for PaceCfg {
    fn default() -> Self {
        Self {
            id: String::from("steady"),
            name: String::from("Steady"),
            speed_mph: default_speed_mph(),
            sanity: 0,
            encounter_chance_delta: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DietCfg {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub receipt_find_pct_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PacingLimits {
    #[serde(default = "default_zero_f32")]
    pub encounter_base: f32,
    #[serde(default = "default_distance_penalty_floor")]
    pub distance_penalty_floor: f32,
    #[serde(default = "default_zero_f32")]
    pub encounter_floor: f32,
    #[serde(default = "default_one_f32")]
    pub encounter_ceiling: f32,
}

impl PacingConfig {
    #[must_use]
    pub fn load_from_static() -> Self {
        serde_json::from_str(DEFAULT_PACING_DATA).unwrap_or_default()
    }

    #[must_use]
    pub fn default_config() -> Self {
        Self::load_from_static()
    }

    #[must_use]
    pub fn get_pace_safe(&self, pace_id: &str) -> PaceCfg {
        self.pace
            .iter()
            .find(|p| p.id == pace_id)
            .cloned()
            .or_else(|| self.pace.first().cloned())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn get_diet_safe(&self, diet_id: &str) -> DietCfg {
        self.diet
            .iter()
            .find(|d| d.id == diet_id)
            .cloned()
            .or_else(|| self.diet.first().cloned())
            .unwrap_or_default()
    }
}

const fn default_one_f32() -> f32 {
    1.0
}

const fn default_speed_mph() -> f32 {
    60.0
}

const fn default_zero_f32() -> f32 {
    0.0
}

const fn default_distance_penalty_floor() -> f32 {
    0.6
}
