use anyhow::Result;
use thirtyfour::prelude::*;

use crate::browser::TestBridge;
use crate::logic::game_tester::{GameTester, GameplayStrategy};
use dystrail_web::game::{GameMode, GameState};

pub mod full_game;
pub mod playability;
pub mod smoke;

#[derive(Debug, Clone)]
pub struct ScenarioCtx<'a> {
    pub base_url: String,
    pub seed: i64,
    pub bridge: TestBridge<'a>,
    pub verbose: bool,
}

// Logic test scenario
pub struct TestScenario {
    pub name: String,
    pub setup: Option<fn(&mut GameState)>,
    pub test_fn: fn(&mut GameState) -> Result<()>,
}

// Browser test scenario
#[async_trait::async_trait]
pub trait BrowserScenario {
    async fn run_browser(&self, driver: &WebDriver, ctx: &ScenarioCtx<'_>) -> Result<()>;
}

// Combined scenario that can run both logic and browser tests
pub trait CombinedScenario: BrowserScenario {
    fn as_logic_scenario(&self) -> Option<TestScenario>;
}

// Simple test scenario to verify real game testing works
pub struct RealGameTest;
pub struct ConservativeStrategyTest;
pub struct AggressiveStrategyTest;
pub struct ResourceManagerTest;

// Modernized comprehensive test scenarios
pub struct BasicGameCreationTest;
pub struct ShareCodeConsistencyTest;
pub struct DeterministicGameplayTest;
pub struct EncounterChoicesTest;
pub struct VehicleSystemTest;
pub struct WeatherEffectsTest;
pub struct ResourceManagementTest;
pub struct StatsBoundariesTest;
pub struct InventoryOperationsTest;
pub struct GameModeVariationsTest;

#[async_trait::async_trait]
impl BrowserScenario for RealGameTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for real game test");
    }
}

impl CombinedScenario for RealGameTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Real Game Test".to_string(),
            setup: None,
            test_fn: |game_state| {
                let tester = GameTester::new(false);
                let seed = game_state.seed;

                // Play one simple game
                let metrics = tester.play_game(GameMode::Classic, GameplayStrategy::Balanced, seed);

                // Basic validation - game should run and produce reasonable results
                anyhow::ensure!(
                    metrics.days_survived > 0,
                    "Game should survive at least 1 day"
                );
                anyhow::ensure!(
                    !metrics.ending_type.starts_with("Error"),
                    "Game should not end with error"
                );

                Ok(())
            },
        })
    }
}

#[async_trait::async_trait]
impl BrowserScenario for ConservativeStrategyTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for conservative strategy test");
    }
}

impl CombinedScenario for ConservativeStrategyTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Conservative Strategy Test".to_string(),
            setup: None,
            test_fn: |game_state| {
                let tester = GameTester::new(false);
                let seed = game_state.seed;
                let metrics =
                    tester.play_game(GameMode::Classic, GameplayStrategy::Conservative, seed);
                anyhow::ensure!(
                    metrics.days_survived > 0,
                    "Game should survive at least 1 day"
                );
                Ok(())
            },
        })
    }
}

#[async_trait::async_trait]
impl BrowserScenario for AggressiveStrategyTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for aggressive strategy test");
    }
}

impl CombinedScenario for AggressiveStrategyTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Aggressive Strategy Test".to_string(),
            setup: None,
            test_fn: |game_state| {
                let tester = GameTester::new(false);
                let seed = game_state.seed;
                let metrics =
                    tester.play_game(GameMode::Classic, GameplayStrategy::Aggressive, seed);
                anyhow::ensure!(
                    metrics.days_survived > 0,
                    "Game should survive at least 1 day"
                );
                Ok(())
            },
        })
    }
}

#[async_trait::async_trait]
impl BrowserScenario for ResourceManagerTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for resource manager test");
    }
}

impl CombinedScenario for ResourceManagerTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Resource Manager Test".to_string(),
            setup: None,
            test_fn: |game_state| {
                let tester = GameTester::new(false);
                let seed = game_state.seed;
                let metrics =
                    tester.play_game(GameMode::Classic, GameplayStrategy::ResourceManager, seed);
                anyhow::ensure!(
                    metrics.days_survived > 0,
                    "Game should survive at least 1 day"
                );
                Ok(())
            },
        })
    }
}

// Implementations for modernized comprehensive test scenarios
#[async_trait::async_trait]
impl BrowserScenario for BasicGameCreationTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for basic game creation test");
    }
}

impl CombinedScenario for BasicGameCreationTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Basic Game State Creation")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for ShareCodeConsistencyTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for share code consistency test");
    }
}

impl CombinedScenario for ShareCodeConsistencyTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Share Code Generation and Parsing")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for DeterministicGameplayTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for deterministic gameplay test");
    }
}

impl CombinedScenario for DeterministicGameplayTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Deterministic Game Behavior")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for EncounterChoicesTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for encounter choices test");
    }
}

impl CombinedScenario for EncounterChoicesTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Encounter Choice Processing")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for VehicleSystemTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for vehicle system test");
    }
}

impl CombinedScenario for VehicleSystemTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Vehicle System Integration")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for WeatherEffectsTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for weather effects test");
    }
}

impl CombinedScenario for WeatherEffectsTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Weather System Effects")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for ResourceManagementTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for resource management test");
    }
}

impl CombinedScenario for ResourceManagementTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Resource Management")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for StatsBoundariesTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for stats boundaries test");
    }
}

impl CombinedScenario for StatsBoundariesTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Stats Boundary Conditions")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for InventoryOperationsTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for inventory operations test");
    }
}

impl CombinedScenario for InventoryOperationsTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Inventory Operations")
            .unwrap())
    }
}

#[async_trait::async_trait]
impl BrowserScenario for GameModeVariationsTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for game mode variations test");
    }
}

impl CombinedScenario for GameModeVariationsTest {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(crate::common::scenarios::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Game Mode Variations")
            .unwrap())
    }
}

pub fn get_scenario(name: &str) -> Option<Box<dyn CombinedScenario + Send + Sync>> {
    match name.to_lowercase().as_str() {
        "smoke" => Some(Box::new(smoke::Smoke)),
        "real-game" | "real" => Some(Box::new(RealGameTest)),
        "conservative-strategy" => Some(Box::new(ConservativeStrategyTest)),
        "aggressive-strategy" => Some(Box::new(AggressiveStrategyTest)),
        "resource-manager" => Some(Box::new(ResourceManagerTest)),
        "full-game-conservative" | "conservative" => {
            Some(Box::new(full_game::FullGameConservative))
        }
        "full-game-aggressive" | "aggressive" => Some(Box::new(full_game::FullGameAggressive)),
        "full-game-balanced" | "balanced" => Some(Box::new(full_game::FullGameBalanced)),
        "resource-stress" | "stress" => Some(Box::new(playability::ResourceStressTest)),
        "deterministic" | "deterministic-verification" => {
            Some(Box::new(playability::DeterministicVerification))
        }
        "edge-case" | "edge" => Some(Box::new(playability::EdgeCaseSurvival)),

        // Comprehensive test scenarios
        "basic-game-creation" | "basic" => Some(Box::new(BasicGameCreationTest)),
        "share-code-consistency" | "share-code" => Some(Box::new(ShareCodeConsistencyTest)),
        "deterministic-gameplay" | "deterministic-game" => Some(Box::new(DeterministicGameplayTest)),
        "encounter-choices" | "encounters" => Some(Box::new(EncounterChoicesTest)),
        "vehicle-system" | "vehicle" => Some(Box::new(VehicleSystemTest)),
        "weather-effects" | "weather" => Some(Box::new(WeatherEffectsTest)),
        "resource-management" | "resources" => Some(Box::new(ResourceManagementTest)),
        "stats-boundaries" | "stats" => Some(Box::new(StatsBoundariesTest)),
        "inventory-operations" | "inventory" => Some(Box::new(InventoryOperationsTest)),
        "game-mode-variations" | "game-modes" => Some(Box::new(GameModeVariationsTest)),

        "all" => {
            // For "all", we return the first scenario but the caller should handle this differently
            // This is a limitation of the current architecture
            Some(Box::new(smoke::Smoke))
        }
        _ => None,
    }
}
