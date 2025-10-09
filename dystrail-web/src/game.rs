//! Web-specific game engine implementation
//!
//! This module provides web-specific implementations of the dystrail-game traits
//! and re-exports the core game logic types.

use gloo::storage::{LocalStorage, Storage};
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
        LocalStorage::set(&key, game_state).map_err(|e| WebStorageError::Storage(format!("{e:?}")))
    }

    fn load_game(&self, save_name: &str) -> Result<Option<dystrail_game::GameState>, Self::Error> {
        let key = format!("dystrail.save.{save_name}");
        match LocalStorage::get(&key) {
            Ok(game_state) => Ok(Some(game_state)),
            Err(_) => Ok(None), // No save found
        }
    }

    fn delete_save(&self, save_name: &str) -> Result<(), Self::Error> {
        let key = format!("dystrail.save.{save_name}");
        LocalStorage::delete(&key);
        Ok(())
    }
}

/// Create a web-compatible game engine with `WebDataLoader` and `WebGameStorage`
#[must_use]
pub fn create_web_game_engine() -> dystrail_game::GameEngine<WebDataLoader, WebGameStorage> {
    dystrail_game::GameEngine::new(WebDataLoader, WebGameStorage)
}
