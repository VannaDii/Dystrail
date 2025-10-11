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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PaceCfg {
    pub id: String,
    pub name: String,
    #[serde(default = "default_one_f32")]
    pub dist_mult: f32,
    #[serde(default)]
    pub distance: f32,
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub pants: i32,
    #[serde(default)]
    pub encounter_chance_delta: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DietCfg {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub pants: i32,
    #[serde(default)]
    pub receipt_find_pct_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PacingLimits {
    #[serde(default = "default_zero_f32")]
    pub encounter_base: f32,
    #[serde(default = "default_zero_f32")]
    pub distance_base: f32,
    #[serde(default = "default_zero_f32")]
    pub encounter_floor: f32,
    #[serde(default = "default_one_f32")]
    pub encounter_ceiling: f32,
    #[serde(default = "default_zero_i32")]
    pub pants_floor: i32,
    #[serde(default = "default_pants_ceiling")]
    pub pants_ceiling: i32,
    #[serde(default)]
    pub passive_relief: i32,
    #[serde(default = "default_passive_threshold")]
    pub passive_relief_threshold: i32,
    #[serde(default)]
    pub boss_pants_cap: i32,
    #[serde(default)]
    pub boss_passive_relief: i32,
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

fn default_one_f32() -> f32 {
    1.0
}

fn default_zero_f32() -> f32 {
    0.0
}

fn default_zero_i32() -> i32 {
    0
}

fn default_pants_ceiling() -> i32 {
    100
}

fn default_passive_threshold() -> i32 {
    0
}
