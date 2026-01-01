//! Dystrail Game Engine
//!
//! Platform-agnostic core game logic for the Dystrail satirical survival game.
//! This crate provides all game mechanics without UI or platform-specific dependencies.

pub mod boss;
pub mod camp;
pub mod constants;
pub mod coverage_anchor;
pub mod crossings;
pub mod data;
pub mod day_accounting;
pub mod encounters;
pub mod endgame;
pub mod exec_orders;
pub mod journey;
pub mod numbers;
pub mod pacing;
pub mod personas;
pub mod result;
pub mod seed;
pub mod state;
pub mod store;
pub mod vehicle;
pub mod weather;

// Re-export commonly used types
pub use boss::{BossConfig, BossOutcome, run_boss_minigame};
pub use camp::{
    CampConfig, CampOutcome, CampState, camp_forage, camp_repair_hack, camp_repair_spare,
    camp_rest, camp_therapy, can_repair, can_therapy,
};
pub use crossings::{
    CrossingConfig, CrossingKind, ThresholdEntry, ThresholdTable, apply_bribe, apply_detour,
    apply_permit, calculate_bribe_cost, can_afford_bribe, can_use_permit,
};
pub use data::{Choice, Effects, Encounter, EncounterData};
pub use day_accounting::{DayLedgerMetrics, compute_day_ledger_metrics, record_travel_day};
pub use endgame::{EndgameState, EndgameTravelCfg};
pub use journey::{
    AcceptanceGuards, BribePolicy, CrossingPolicy, CrossingPolicyOverlay, DailyTickOutcome,
    DayOutcome, DayRecord, DayTag, DayTagSet, DetourPolicy, JourneyCfg, JourneyController,
    JourneySession, MechanicalPolicyId, PermitPolicy, PolicyId, RngBundle, StrategyId,
    TravelDayKind, apply_daily_effect,
};
pub use pacing::{DietCfg, PaceCfg, PacingConfig, PacingLimits};
pub use personas::{Persona, PersonaMods, PersonaStart, PersonasList};
pub use result::{ResultConfig, ResultSummary, load_result_config, result_summary};
pub use seed::{decode_to_seed, encode_friendly, generate_code_from_entropy, parse_share_code};
pub use state::{
    BossProgress, BossReadiness, BossResolution, CollapseCause, CrossingOutcomeTelemetry,
    CrossingTelemetry, DayState, DietId, EncounterState, Ending, FeatureFlags, GameMode, GamePhase,
    GameState, GuardState, Inventory, PaceId, PolicyKind, Region, Spares, Stats,
};
pub use store::{
    Cart, CartLine, Grants, Store, StoreItem, calculate_cart_total, calculate_effective_price,
};
pub use vehicle::{Breakdown, Part, Vehicle, VehicleConfig};
pub use weather::{Weather, WeatherConfig, WeatherEffect, WeatherMitigation, WeatherState};

/// Trait for abstracting data loading operations
/// Platform-specific implementations should provide this
pub trait DataLoader {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Load encounter data from the platform-specific source
    ///
    /// # Errors
    ///
    /// Returns an error if the encounter data cannot be loaded.
    fn load_encounter_data(&self) -> Result<EncounterData, Self::Error>;

    /// Load configuration data for a specific system
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or parsed.
    fn load_config<T>(&self, config_name: &str) -> Result<T, Self::Error>
    where
        T: serde::de::DeserializeOwned;
}

/// Trait for abstracting save/load operations\
/// Platform-specific implementations should provide this
pub trait GameStorage {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Save game state
    ///
    /// # Errors
    ///
    /// Returns an error if the game state cannot be saved.
    fn save_game(&self, save_name: &str, game_state: &GameState) -> Result<(), Self::Error>;

    /// Load game state
    ///
    /// # Errors
    ///
    /// Returns an error if the game state cannot be loaded.
    fn load_game(&self, save_name: &str) -> Result<Option<GameState>, Self::Error>;

    /// Delete saved game
    ///
    /// # Errors
    ///
    /// Returns an error if the save cannot be deleted.
    fn delete_save(&self, save_name: &str) -> Result<(), Self::Error>;
}

/// Main game engine for managing game instances
pub struct GameEngine<L, S>
where
    L: DataLoader,
    S: GameStorage,
{
    data_loader: L,
    storage: S,
}

impl<L, S> GameEngine<L, S>
where
    L: DataLoader,
    S: GameStorage,
{
    /// Create a new game engine with the provided data loader and storage
    pub const fn new(data_loader: L, storage: S) -> Self {
        Self {
            data_loader,
            storage,
        }
    }

    /// Create a new game with the specified seed and mode
    ///
    /// # Errors
    ///
    /// Returns an error if the encounter data cannot be loaded.
    pub fn create_game(&self, seed: u64, mode: GameMode) -> Result<GameState, L::Error> {
        self.create_session(seed, mode, StrategyId::Balanced)
            .map(JourneySession::into_state)
    }

    /// Construct a new journey session encompassing controller and state.
    ///
    /// # Errors
    ///
    /// Returns an error if encounter data cannot be loaded.
    pub fn create_session(
        &self,
        seed: u64,
        mode: GameMode,
        strategy: StrategyId,
    ) -> Result<JourneySession, L::Error> {
        let data = self.data_loader.load_encounter_data()?;
        let endgame_cfg = EndgameTravelCfg::default_config();
        Ok(JourneySession::new(
            mode,
            strategy,
            seed,
            data,
            &endgame_cfg,
        ))
    }

    /// Save a game state
    ///
    /// # Errors
    ///
    /// Returns an error if the game state cannot be saved.
    pub fn save_game(&self, save_name: &str, game_state: &GameState) -> Result<(), S::Error> {
        self.storage.save_game(save_name, game_state)
    }

    /// Load a game state
    ///
    /// # Errors
    ///
    /// Returns an error if the game state cannot be loaded or rehydrated.
    pub fn load_game(&self, save_name: &str) -> Result<Option<GameState>, anyhow::Error>
    where
        L::Error: Into<anyhow::Error>,
        S::Error: Into<anyhow::Error>,
    {
        if let Some(mut game_state) = self.storage.load_game(save_name).map_err(Into::into)? {
            // Rehydrate with fresh data
            let data = self.data_loader.load_encounter_data().map_err(Into::into)?;
            game_state = game_state.rehydrate(data);
            Ok(Some(game_state))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::rc::Rc;

    #[derive(Clone, Copy, Default)]
    struct FixtureLoader;

    impl DataLoader for FixtureLoader {
        type Error = Infallible;

        fn load_encounter_data(&self) -> Result<EncounterData, Self::Error> {
            Ok(EncounterData::empty())
        }

        fn load_config<T>(&self, _config_name: &str) -> Result<T, Self::Error>
        where
            T: DeserializeOwned,
        {
            let parsed = serde_json::from_str("{}")
                .or_else(|_| serde_json::from_str("null"))
                .unwrap();
            Ok(parsed)
        }
    }

    #[derive(Clone, Default)]
    struct MemoryStorage {
        saves: Rc<RefCell<HashMap<String, GameState>>>,
    }

    impl GameStorage for MemoryStorage {
        type Error = Infallible;

        fn save_game(&self, save_name: &str, game_state: &GameState) -> Result<(), Self::Error> {
            self.saves
                .borrow_mut()
                .insert(save_name.to_string(), game_state.clone());
            Ok(())
        }

        fn load_game(&self, save_name: &str) -> Result<Option<GameState>, Self::Error> {
            Ok(self.saves.borrow().get(save_name).cloned())
        }

        fn delete_save(&self, save_name: &str) -> Result<(), Self::Error> {
            self.saves.borrow_mut().remove(save_name);
            Ok(())
        }
    }

    #[test]
    fn engine_creates_and_roundtrips_state() {
        let engine = GameEngine::new(FixtureLoader, MemoryStorage::default());
        let mut session = engine
            .create_session(0xABCD, GameMode::Deep, StrategyId::Aggressive)
            .unwrap();
        session.with_state_mut(|state| {
            state.budget = 250;
            state.day = 3;
        });
        let snapshot = session.into_state();
        engine.save_game("slot-one", &snapshot).unwrap();

        let loaded = engine.load_game("slot-one").unwrap().expect("save exists");
        assert_eq!(loaded.budget, 250);
        assert_eq!(loaded.mode, GameMode::Deep);
        assert!(engine.load_game("missing-slot").unwrap().is_none());
    }

    #[test]
    fn create_game_defaults_to_balanced_strategy() {
        let engine = GameEngine::new(FixtureLoader, MemoryStorage::default());
        let state = engine.create_game(7, GameMode::Classic).unwrap();
        assert_eq!(state.policy, Some(PolicyKind::Balanced));
    }
}
