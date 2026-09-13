//! Partial daily-cost overrides preserve all unspecified base-policy values.
use super::{DailyChannelConfig, DailyTickConfig, HealthTickConfig};
use crate::{DietId, PaceId, Weather};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DailyTickOverlay {
    pub supplies: Option<DailyChannelOverlay>,
    pub sanity: Option<DailyChannelOverlay>,
    pub health: Option<HealthTickOverlay>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DailyChannelOverlay {
    pub base: Option<f32>,
    pub pace: Option<HashMap<PaceId, f32>>,
    pub diet: Option<HashMap<DietId, f32>>,
    pub weather: Option<HashMap<Weather, f32>>,
    pub exec: Option<HashMap<String, f32>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HealthTickOverlay {
    pub decay: Option<f32>,
    pub rest_heal: Option<f32>,
    pub weather: Option<HashMap<Weather, f32>>,
    pub exec: Option<HashMap<String, f32>>,
}

impl DailyTickConfig {
    pub(super) fn apply_overlay(&mut self, overlay: &DailyTickOverlay) {
        if let Some(supplies) = &overlay.supplies {
            self.supplies.apply_overlay(supplies);
        }
        if let Some(sanity) = &overlay.sanity {
            self.sanity.apply_overlay(sanity);
        }
        if let Some(health) = &overlay.health {
            self.health.apply_overlay(health);
        }
    }
}

impl DailyChannelConfig {
    fn apply_overlay(&mut self, overlay: &DailyChannelOverlay) {
        if let Some(base) = overlay.base {
            self.base = base;
        }
        if let Some(pace) = &overlay.pace {
            self.pace.extend(pace.clone());
        }
        if let Some(diet) = &overlay.diet {
            self.diet.extend(diet.clone());
        }
        if let Some(weather) = &overlay.weather {
            self.weather.extend(weather.clone());
        }
        if let Some(exec) = &overlay.exec {
            self.exec.extend(exec.clone());
        }
    }
}

impl HealthTickConfig {
    fn apply_overlay(&mut self, overlay: &HealthTickOverlay) {
        if let Some(decay) = overlay.decay {
            self.decay = decay;
        }
        if let Some(heal) = overlay.rest_heal {
            self.rest_heal = heal;
        }
        if let Some(weather) = &overlay.weather {
            self.weather.extend(weather.clone());
        }
        if let Some(exec) = &overlay.exec {
            self.exec.extend(exec.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{JourneyCfg, JourneyOverlay};
    use crate::Weather;

    #[test]
    fn a_partial_daily_override_retains_food_healing_and_other_weather_values() {
        let mut base = JourneyCfg::default();
        base.daily.supplies.base = 1.0;
        base.daily.health.rest_heal = 0.5;
        base.daily.health.weather.insert(Weather::Storm, 1.25);
        let overlay: JourneyOverlay = serde_json::from_str(
            r#"{
            "daily": {"health": {"decay": 0.12, "weather": {"HeatWave": 1.5}}}
        }"#,
        )
        .unwrap();
        let merged = base.merge_overlay(&overlay);
        assert_eq!((merged.daily.supplies.base).to_bits(), (1.0f32).to_bits());
        assert_eq!((merged.daily.health.decay).to_bits(), (0.12f32).to_bits());
        assert_eq!(
            (merged.daily.health.rest_heal).to_bits(),
            (0.5f32).to_bits()
        );
        assert_eq!(
            (merged.daily.health.weather[&Weather::Storm]).to_bits(),
            (1.25f32).to_bits()
        );
        assert_eq!(
            (merged.daily.health.weather[&Weather::HeatWave]).to_bits(),
            (1.5f32).to_bits()
        );
        assert_eq!((base.daily.health.decay).to_bits(), (0.0f32).to_bits());
    }
}
