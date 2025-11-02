//! Web-specific game engine implementation
//!
//! This module provides web-specific implementations of the dystrail-game traits
//! and re-exports the core game logic types.

use crate::dom;
use serde::de::DeserializeOwned;

// Re-export all types from dystrail-game
pub use dystrail_game::*;

/// Web-specific data loader that fetches data from static assets
pub struct WebDataLoader;

#[derive(Debug, thiserror::Error)]
pub enum WebDataError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

impl DataLoader for WebDataLoader {
    type Error = WebDataError;

    fn load_encounter_data(&self) -> Result<dystrail_game::EncounterData, Self::Error> {
        // Return default data for now - in real implementation this would be async
        // For now, we'll use embedded fallback data
        let json = include_str!("../static/assets/data/game.json");
        dystrail_game::EncounterData::from_json(json).map_err(WebDataError::Json)
    }

    fn load_config<T>(&self, config_name: &str) -> Result<T, Self::Error>
    where
        T: DeserializeOwned,
    {
        // Load config from static assets - placeholder implementation
        let json = match config_name {
            "personas" => include_str!("../static/assets/data/personas.json"),
            "store" => include_str!("../static/assets/data/store.json"),
            "vehicle" => include_str!("../static/assets/data/vehicle.json"),
            "weather" => include_str!("../static/assets/data/weather.json"),
            "pacing" => include_str!("../static/assets/data/pacing.json"),
            "camp" => include_str!("../static/assets/data/camp.json"),
            "crossings" => include_str!("../static/assets/data/crossings.json"),
            "result" => include_str!("../static/assets/data/result.json"),
            _ => {
                return Err(WebDataError::Network(format!(
                    "Unknown config: {config_name}"
                )));
            }
        };
        serde_json::from_str(json).map_err(WebDataError::Json)
    }
}

/// Web-specific game storage using localStorage
pub struct WebGameStorage;

#[derive(Debug, thiserror::Error)]
pub enum WebStorageError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl GameStorage for WebGameStorage {
    type Error = WebStorageError;

    fn save_game(
        &self,
        save_name: &str,
        game_state: &dystrail_game::GameState,
    ) -> Result<(), Self::Error> {
        let key = format!("dystrail.save.{save_name}");
        let storage = dom::local_storage()
            .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
        let serialized = serde_json::to_string(game_state)?;
        storage
            .set_item(&key, &serialized)
            .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
        Ok(())
    }

    fn load_game(&self, save_name: &str) -> Result<Option<dystrail_game::GameState>, Self::Error> {
        let key = format!("dystrail.save.{save_name}");
        let storage = dom::local_storage()
            .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
        let value = storage
            .get_item(&key)
            .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
        match value {
            Some(json) => {
                let state = serde_json::from_str(&json)?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    fn delete_save(&self, save_name: &str) -> Result<(), Self::Error> {
        let key = format!("dystrail.save.{save_name}");
        let storage = dom::local_storage()
            .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
        storage
            .remove_item(&key)
            .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
        Ok(())
    }
}

/// Create a web-compatible game engine with `WebDataLoader` and `WebGameStorage`
#[must_use]
pub const fn create_web_game_engine() -> dystrail_game::GameEngine<WebDataLoader, WebGameStorage> {
    dystrail_game::GameEngine::new(WebDataLoader, WebGameStorage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dystrail_game::pacing::PacingConfig;
    use dystrail_game::weather::WeatherConfig;

    #[test]
    fn web_data_loader_parses_static_assets() {
        let loader = WebDataLoader;
        let encounters = loader
            .load_encounter_data()
            .expect("static encounter data should parse");
        assert!(
            !encounters.encounters.is_empty(),
            "encounter dataset should not be empty"
        );

        // Check that config parsing hits multiple JSON structures.
        let pacing: PacingConfig = loader
            .load_config("pacing")
            .expect("pacing config should deserialize");
        assert!(
            !pacing.pace.is_empty(),
            "pacing config should include pace definitions"
        );

        let weather: WeatherConfig = loader
            .load_config("weather")
            .expect("weather config should deserialize");
        assert!(
            !weather.effects.is_empty(),
            "weather config should describe effects"
        );
    }

    #[test]
    fn web_data_loader_reports_unknown_config() {
        let loader = WebDataLoader;
        let err = loader.load_config::<serde_json::Value>("unknown-key");
        match err {
            Err(WebDataError::Network(msg)) => {
                assert!(msg.contains("Unknown config"));
            }
            other => panic!("Expected network error, got {other:?}"),
        }
    }

    #[test]
    fn engine_factory_wires_default_components() {
        let engine = create_web_game_engine();
        let game = engine
            .create_game(42, dystrail_game::GameMode::Classic)
            .expect("engine should build a default game");
        assert_eq!(game.seed, 42);
        assert_eq!(game.mode, dystrail_game::GameMode::Classic);
    }
}
