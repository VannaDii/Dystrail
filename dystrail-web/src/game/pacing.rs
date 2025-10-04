use serde::{Deserialize, Serialize};

pub type PaceId = &'static str;
pub type DietId = &'static str;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaceCfg {
    pub id: String,
    pub name: String,
    pub dist_mult: f32,
    pub sanity: i32,
    pub pants: i32,
    pub encounter_chance_delta: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DietCfg {
    pub id: String,
    pub name: String,
    pub receipt_find_pct_delta: i32,
    pub sanity: i32,
    pub pants: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PacingLimits {
    pub pants_floor: i32,
    pub pants_ceiling: i32,
    pub encounter_base: f32,
    pub distance_base: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PacingConfig {
    pub pace: Vec<PaceCfg>,
    pub diet: Vec<DietCfg>,
    pub limits: PacingLimits,
}

impl PacingConfig {
    /// Load pacing configuration from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string cannot be parsed or if validation fails.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: PacingConfig =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;
        config.validate()?;
        Ok(config)
    }

    /// Load pacing configuration from static assets
    pub async fn load_from_static() -> Self {
        let url = "/static/assets/data/pacing.json";
        if let Ok(resp) = gloo_net::http::Request::get(url).send().await
            && resp.status() == 200
            && let Ok(json_str) = resp.text().await
            && let Ok(config) = Self::from_json(&json_str)
        {
            return config;
        }
        // If loading fails, use embedded defaults
        Self::default_config()
    }

    /// Get embedded default configuration if loading fails
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            pace: vec![
                PaceCfg {
                    id: "steady".to_string(),
                    name: "Steady".to_string(),
                    dist_mult: 1.0,
                    sanity: 0,
                    pants: 0,
                    encounter_chance_delta: 0.00,
                },
                PaceCfg {
                    id: "heated".to_string(),
                    name: "Heated".to_string(),
                    dist_mult: 1.2,
                    sanity: -1,
                    pants: 3,
                    encounter_chance_delta: 0.05,
                },
                PaceCfg {
                    id: "blitz".to_string(),
                    name: "Blitz".to_string(),
                    dist_mult: 1.4,
                    sanity: -2,
                    pants: 6,
                    encounter_chance_delta: 0.10,
                },
            ],
            diet: vec![
                DietCfg {
                    id: "quiet".to_string(),
                    name: "Quiet".to_string(),
                    receipt_find_pct_delta: -5,
                    sanity: 1,
                    pants: -2,
                },
                DietCfg {
                    id: "mixed".to_string(),
                    name: "Mixed".to_string(),
                    receipt_find_pct_delta: 0,
                    sanity: 0,
                    pants: 0,
                },
                DietCfg {
                    id: "doom".to_string(),
                    name: "Doomscroll".to_string(),
                    receipt_find_pct_delta: 8,
                    sanity: -2,
                    pants: 4,
                },
            ],
            limits: PacingLimits {
                pants_floor: 0,
                pants_ceiling: 100,
                encounter_base: 0.35,
                distance_base: 1.0,
            },
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), String> {
        // Validate pace configurations
        for pace in &self.pace {
            if pace.dist_mult <= 0.0 {
                return Err(format!(
                    "Invalid dist_mult for pace '{}': must be > 0",
                    pace.id
                ));
            }
        }

        // Validate limits
        if self.limits.pants_floor > self.limits.pants_ceiling {
            return Err("pants_floor cannot be greater than pants_ceiling".to_string());
        }

        if self.limits.encounter_base < 0.0 || self.limits.encounter_base > 1.0 {
            return Err("encounter_base must be between 0.0 and 1.0".to_string());
        }

        // Check for duplicate IDs
        let pace_ids: Vec<&str> = self.pace.iter().map(|p| p.id.as_str()).collect();
        let unique_pace_ids: std::collections::HashSet<&str> = pace_ids.iter().copied().collect();
        if pace_ids.len() != unique_pace_ids.len() {
            return Err("Duplicate pace IDs found".to_string());
        }

        let diet_ids: Vec<&str> = self.diet.iter().map(|d| d.id.as_str()).collect();
        let unique_diet_ids: std::collections::HashSet<&str> = diet_ids.iter().copied().collect();
        if diet_ids.len() != unique_diet_ids.len() {
            return Err("Duplicate diet IDs found".to_string());
        }

        Ok(())
    }

    /// Find pace configuration by ID
    #[must_use]
    pub fn find_pace(&self, id: &str) -> Option<&PaceCfg> {
        self.pace.iter().find(|p| p.id == id)
    }

    /// Find diet configuration by ID
    #[must_use]
    pub fn find_diet(&self, id: &str) -> Option<&DietCfg> {
        self.diet.iter().find(|d| d.id == id)
    }

    /// Get pace configuration by ID, falling back to "steady" if not found
    #[must_use]
    pub fn get_pace_safe(&self, id: &str) -> &PaceCfg {
        self.find_pace(id)
            .or_else(|| self.find_pace("steady"))
            .unwrap_or(&self.pace[0]) // Should always have at least one pace
    }

    /// Get diet configuration by ID, falling back to "mixed" if not found
    #[must_use]
    pub fn get_diet_safe(&self, id: &str) -> &DietCfg {
        self.find_diet(id)
            .or_else(|| self.find_diet("mixed"))
            .unwrap_or(&self.diet[0]) // Should always have at least one diet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = PacingConfig::default_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_from_json_valid() {
        let json = r#"{
            "pace": [
                {
                    "id": "steady",
                    "name": "Steady",
                    "dist_mult": 1.0,
                    "sanity": 0,
                    "pants": 0,
                    "encounter_chance_delta": 0.0
                }
            ],
            "diet": [
                {
                    "id": "mixed",
                    "name": "Mixed",
                    "receipt_find_pct_delta": 0,
                    "sanity": 0,
                    "pants": 0
                }
            ],
            "limits": {
                "pants_floor": 0,
                "pants_ceiling": 100,
                "encounter_base": 0.35,
                "distance_base": 1.0
            }
        }"#;

        let config = PacingConfig::from_json(json);
        assert!(config.is_ok());
    }

    #[test]
    fn test_find_pace_and_diet() {
        let config = PacingConfig::default_config();

        assert!(config.find_pace("steady").is_some());
        assert!(config.find_pace("nonexistent").is_none());

        assert!(config.find_diet("mixed").is_some());
        assert!(config.find_diet("nonexistent").is_none());
    }

    #[test]
    fn test_safe_getters() {
        let config = PacingConfig::default_config();

        // Should find existing
        assert_eq!(config.get_pace_safe("steady").id, "steady");

        // Should fallback to steady for unknown
        assert_eq!(config.get_pace_safe("unknown").id, "steady");

        // Should find existing
        assert_eq!(config.get_diet_safe("mixed").id, "mixed");

        // Should fallback to mixed for unknown
        assert_eq!(config.get_diet_safe("unknown").id, "mixed");
    }

    #[test]
    fn test_validation_errors() {
        let mut config = PacingConfig::default_config();

        // Test invalid dist_mult
        config.pace[0].dist_mult = -1.0;
        assert!(config.validate().is_err());

        config.pace[0].dist_mult = 1.0; // Fix it

        // Test invalid pants range
        config.limits.pants_floor = 50;
        config.limits.pants_ceiling = 25;
        assert!(config.validate().is_err());

        config.limits.pants_floor = 0; // Fix it
        config.limits.pants_ceiling = 100;

        // Test invalid encounter_base
        config.limits.encounter_base = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_duplicate_ids() {
        let mut config = PacingConfig::default_config();

        // Add duplicate pace ID
        let duplicate_pace = config.pace[0].clone();
        config.pace.push(duplicate_pace);
        assert!(config.validate().is_err());
    }
}
