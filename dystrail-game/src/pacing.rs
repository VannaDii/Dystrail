//! Pace and diet system
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PacingConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PaceCfg {
    pub name: String,
    pub encounter_mult: f32,
    pub distance_mult: f32,
    pub sanity: i32,
    pub pants: i32,
    pub encounter_chance_delta: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DietCfg {
    pub name: String,
    pub receipt_mult: f32,
    pub sanity: i32,
    pub pants: i32,
    pub receipt_find_pct_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PacingLimits {
    pub min_encounter: f32,
    pub max_encounter: f32,
}

impl PacingConfig {
    /// Load pacing configuration from static assets (function for web compatibility)
    /// This is a placeholder that returns default data - web implementation should override this
    #[must_use]
    pub fn load_from_static() -> Self {
        Self::default()
    }

    /// Get default configuration
    #[must_use]
    pub fn default_config() -> Self {
        Self::default()
    }

    /// Get pace configuration by ID - placeholder implementation
    #[must_use]
    pub fn get_pace_safe(&self, _pace_id: &str) -> PaceCfg {
        PaceCfg::default()
    }

    /// Get diet configuration by ID - placeholder implementation
    #[must_use]
    pub fn get_diet_safe(&self, _diet_id: &str) -> DietCfg {
        DietCfg::default()
    }
}
