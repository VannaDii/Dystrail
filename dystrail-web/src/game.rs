//! Web-specific game engine implementation
//!
//! This module provides web-specific implementations of the dystrail-game traits
//! and re-exports the core game logic types.

#[cfg(target_arch = "wasm32")]
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
        let json = config_json(config_name)?;
        serde_json::from_str(json).map_err(WebDataError::Json)
    }
}

fn config_json(config_name: &str) -> Result<&'static str, WebDataError> {
    match config_name {
        "personas" => Ok(include_str!("../static/assets/data/personas.json")),
        "store" => Ok(include_str!("../static/assets/data/store.json")),
        "vehicle" => Ok(include_str!("../static/assets/data/vehicle.json")),
        "weather" => Ok(include_str!("../static/assets/data/weather.json")),
        "pacing" => Ok(include_str!("../static/assets/data/pacing.json")),
        "camp" => Ok(include_str!("../static/assets/data/camp.json")),
        "crossings" => Ok(include_str!("../static/assets/data/crossings.json")),
        "result" => Ok(include_str!("../static/assets/data/result.json")),
        _ => Err(WebDataError::Network(format!(
            "Unknown config: {config_name}"
        ))),
    }
}

/// Load encounter data from embedded static assets.
///
/// # Errors
///
/// Returns an error if the bundled JSON cannot be parsed.
pub fn load_encounter_data() -> Result<dystrail_game::EncounterData, WebDataError> {
    let json = include_str!("../static/assets/data/game.json");
    dystrail_game::EncounterData::from_json(json).map_err(WebDataError::Json)
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
        #[cfg(target_arch = "wasm32")]
        {
            let key = format!("dystrail.save.{save_name}");
            let storage = dom::local_storage()
                .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
            let serialized = serde_json::to_string(game_state)?;
            storage
                .set_item(&key, &serialized)
                .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (save_name, game_state);
            Err(WebStorageError::Storage(String::from(
                "Storage unavailable",
            )))
        }
    }

    fn load_game(&self, save_name: &str) -> Result<Option<dystrail_game::GameState>, Self::Error> {
        #[cfg(target_arch = "wasm32")]
        {
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = save_name;
            Err(WebStorageError::Storage(String::from(
                "Storage unavailable",
            )))
        }
    }

    fn delete_save(&self, save_name: &str) -> Result<(), Self::Error> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = format!("dystrail.save.{save_name}");
            let storage = dom::local_storage()
                .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
            storage
                .remove_item(&key)
                .map_err(|err| WebStorageError::Storage(dom::js_error_message(&err)))?;
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = save_name;
            Err(WebStorageError::Storage(String::from(
                "Storage unavailable",
            )))
        }
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

        let personas: serde_json::Value = loader
            .load_config("personas")
            .expect("personas config should deserialize");
        assert!(personas.is_object());

        let store: serde_json::Value = loader
            .load_config("store")
            .expect("store config should deserialize");
        assert!(store.is_object());

        let vehicle: serde_json::Value = loader
            .load_config("vehicle")
            .expect("vehicle config should deserialize");
        assert!(vehicle.is_object());

        let camp: serde_json::Value = loader
            .load_config("camp")
            .expect("camp config should deserialize");
        assert!(camp.is_object());

        let crossings: serde_json::Value = loader
            .load_config("crossings")
            .expect("crossings config should deserialize");
        assert!(crossings.is_object());

        let result: serde_json::Value = loader
            .load_config("result")
            .expect("result config should deserialize");
        assert!(result.is_object());
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
    fn config_json_maps_known_keys() {
        let json = config_json("personas").expect("personas config should exist");
        assert!(json.contains("\"journalist\""));
        let json = config_json("store").expect("store config should exist");
        assert!(json.contains("\"categories\""));
    }

    #[test]
    fn config_json_reports_unknown_key() {
        let err = config_json("unknown-key");
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

    #[test]
    fn web_storage_errors_without_browser_storage() {
        let storage = WebGameStorage;
        let state = dystrail_game::GameState::default();

        let err = storage
            .save_game("test", &state)
            .expect_err("save should fail without storage");
        assert!(matches!(err, WebStorageError::Storage(_)));

        let err = storage
            .load_game("test")
            .expect_err("load should fail without storage");
        assert!(matches!(err, WebStorageError::Storage(_)));

        let err = storage
            .delete_save("test")
            .expect_err("delete should fail without storage");
        assert!(matches!(err, WebStorageError::Storage(_)));
    }
}
