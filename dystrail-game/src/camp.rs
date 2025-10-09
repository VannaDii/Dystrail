//! Camping and rest system
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CampState {
    pub rest_cooldown: u32,
    pub repair_cooldown: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CampConfig {
    pub enabled: bool,
    pub rest_bonus: i32,
}

impl CampConfig {
    /// Load camp configuration from static assets (function for web compatibility)
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
}

/// Camp action functions - placeholders for now
pub fn camp_rest(_gs: &mut crate::GameState, _cfg: &CampConfig) -> String {
    // Placeholder implementation
    "You rest and recover some health.".to_string()
}

pub fn camp_forage(_gs: &mut crate::GameState, _cfg: &CampConfig) -> String {
    // Placeholder implementation
    "You forage for supplies.".to_string()
}

pub fn camp_therapy(_gs: &mut crate::GameState, _cfg: &CampConfig) -> String {
    // Placeholder implementation
    "You engage in therapy activities.".to_string()
}

pub fn camp_repair_spare(
    _gs: &mut crate::GameState,
    _cfg: &CampConfig,
    _part: crate::vehicle::Part,
) -> String {
    // Placeholder implementation
    "You repair the vehicle using spare parts.".to_string()
}

pub fn camp_repair_hack(_gs: &mut crate::GameState, _cfg: &CampConfig) -> String {
    // Placeholder implementation
    "You perform an improvised repair.".to_string()
}

#[must_use]
pub fn can_repair(gs: &crate::GameState, _cfg: &CampConfig) -> bool {
    // Check if there's a breakdown to repair
    gs.breakdown.is_some()
}

#[must_use]
pub fn can_therapy(_gs: &crate::GameState, _cfg: &CampConfig) -> bool {
    // Placeholder - could check sanity levels, etc.
    true
}
