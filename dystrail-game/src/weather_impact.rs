//! The most recent weather-only cost, recorded after protection and exposure.
use crate::{GameState, Stats, Weather};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherImpact {
    pub day: u32,
    pub weather: Weather,
    pub supplies: i32,
    pub hp: i32,
    pub sanity: i32,
}

impl WeatherImpact {
    #[must_use]
    pub fn between(before: &Stats, after: &GameState) -> Self {
        let mut before = before.clone();
        let mut stats = after.stats.clone();
        before.clamp();
        stats.clamp();
        Self {
            day: after.day,
            weather: after.weather_state.today,
            supplies: stats.supplies - before.supplies,
            hp: stats.hp - before.hp,
            sanity: stats.sanity - before.sanity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::{WeatherConfig, apply_weather_effects};

    #[test]
    fn weather_receipt_includes_exposure_and_gear_then_clears() {
        let cfg = WeatherConfig::default_config();
        let mut gs = GameState::default();
        gs.weather_state.today = Weather::HeatWave;
        gs.exposure_streak_heat = 2;
        apply_weather_effects(&mut gs, &cfg);
        let impact = gs.continuity.weather_impact.as_ref().unwrap();
        assert_eq!((impact.supplies, impact.sanity, impact.hp), (-1, -2, -1));
        let encoded = serde_json::to_string(&gs).unwrap();
        let restored: GameState = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            restored.continuity.weather_impact,
            gs.continuity.weather_impact
        );
        assert_eq!(restored.stats, gs.stats);

        gs.inventory.tags.insert("water_jugs".into());
        apply_weather_effects(&mut gs, &cfg);
        let impact = gs.continuity.weather_impact.as_ref().unwrap();
        assert_eq!((impact.sanity, impact.hp), (-1, 0));
        gs.weather_state.today = Weather::Clear;
        apply_weather_effects(&mut gs, &cfg);
        let impact = gs.continuity.weather_impact.as_ref().unwrap();
        assert_eq!((impact.supplies, impact.sanity, impact.hp), (0, 0, 0));
    }

    #[test]
    fn receipt_observes_cold_protection_and_stat_floors() {
        let cfg = WeatherConfig::default_config();
        let mut gs = GameState::default();
        gs.weather_state.today = Weather::ColdSnap;
        gs.inventory.tags.insert("cold_resist".into());
        apply_weather_effects(&mut gs, &cfg);
        assert_eq!(gs.continuity.weather_impact.as_ref().unwrap().sanity, 0);
        gs.inventory.tags.clear();
        gs.stats.sanity = 0;
        apply_weather_effects(&mut gs, &cfg);
        assert_eq!(gs.continuity.weather_impact.as_ref().unwrap().sanity, 0);
        let mut old = serde_json::to_value(&gs).unwrap();
        old.as_object_mut().unwrap().remove("weather_impact");
        let restored: GameState = serde_json::from_value(old).unwrap();
        assert!(restored.continuity.weather_impact.is_none());
    }
}
