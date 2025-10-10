use anyhow::Result;
use thirtyfour::prelude::*;

use crate::browser::TestBridge;
pub mod catalog;

use crate::logic::{
    DEFAULT_POLICY_SIM_DAYS, GameplayStrategy, SimulationPlan, default_policy_setup,
};
use catalog::find_catalog_scenario;
use dystrail_game::GameMode;

pub mod full_game;
pub mod playability;
pub mod smoke;

#[derive(Debug, Clone)]
pub struct ScenarioCtx<'a> {
    pub base_url: String,
    pub seed: u64,
    pub bridge: TestBridge<'a>,
    pub verbose: bool,
}

// Logic test scenario
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub plan: SimulationPlan,
}

impl TestScenario {
    #[must_use]
    pub fn simulation(name: impl Into<String>, plan: SimulationPlan) -> Self {
        Self {
            name: name.into(),
            plan,
        }
    }
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

#[derive(Clone)]
pub struct SimulationScenario {
    name: &'static str,
    plan: SimulationPlan,
    browser_message: &'static str,
}

impl SimulationScenario {
    pub fn new(name: &'static str, plan: SimulationPlan) -> Self {
        Self {
            name,
            plan,
            browser_message: "Browser testing not implemented for this simulation scenario",
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[async_trait::async_trait]
impl BrowserScenario for SimulationScenario {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!(self.browser_message)
    }
}

impl CombinedScenario for SimulationScenario {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario::simulation(self.name, self.plan.clone()))
    }
}

fn survival_expectation(summary: &crate::logic::game_tester::SimulationSummary) -> Result<()> {
    anyhow::ensure!(
        summary.metrics.days_survived > 0,
        "Game should survive at least 1 day"
    );
    Ok(())
}

fn real_game_expectation(summary: &crate::logic::game_tester::SimulationSummary) -> Result<()> {
    survival_expectation(summary)?;
    anyhow::ensure!(
        !summary.metrics.ending_type.contains("Error"),
        "Game should not end with error"
    );
    Ok(())
}

fn real_game_scenario() -> SimulationScenario {
    SimulationScenario::new(
        "Real Game Test",
        SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_expectation(real_game_expectation),
    )
}

fn strategy_scenario(name: &'static str, strategy: GameplayStrategy) -> SimulationScenario {
    SimulationScenario::new(
        name,
        SimulationPlan::new(GameMode::Classic, strategy)
            .with_max_days(DEFAULT_POLICY_SIM_DAYS)
            .with_setup(default_policy_setup(strategy))
            .with_expectation(survival_expectation),
    )
}

pub fn get_scenario(name: &str) -> Option<Box<dyn CombinedScenario + Send + Sync>> {
    match name.to_lowercase().as_str() {
        "smoke" | "all" => Some(Box::new(smoke::SmokeScenario)),
        "real-game" | "real" => Some(Box::new(real_game_scenario())),
        "conservative-strategy" => Some(Box::new(strategy_scenario(
            "Conservative Strategy Test",
            GameplayStrategy::Conservative,
        ))),
        "aggressive-strategy" => Some(Box::new(strategy_scenario(
            "Aggressive Strategy Test",
            GameplayStrategy::Aggressive,
        ))),
        "resource-manager" => Some(Box::new(strategy_scenario(
            "Resource Manager Test",
            GameplayStrategy::ResourceManager,
        ))),
        "full-game-conservative" | "conservative" => {
            Some(Box::new(full_game::full_game_conservative_scenario()))
        }
        "full-game-aggressive" | "aggressive" => {
            Some(Box::new(full_game::full_game_aggressive_scenario()))
        }
        "full-game-balanced" | "balanced" => {
            Some(Box::new(full_game::full_game_balanced_scenario()))
        }
        "resource-stress" | "stress" => Some(Box::new(playability::resource_stress_scenario())),
        "deterministic" | "deterministic-verification" => {
            Some(Box::new(playability::deterministic_verification_scenario()))
        }
        "edge-case" | "edge" => Some(Box::new(playability::edge_case_survival_scenario())),

        // Comprehensive test scenarios
        "basic-game-creation" | "basic" => find_catalog_scenario("Basic Game State Creation")
            .map(|scenario| Box::new(scenario) as _),
        "share-code-consistency" | "share-code" => {
            find_catalog_scenario("Share Code Generation and Parsing")
                .map(|scenario| Box::new(scenario) as _)
        }
        "deterministic-gameplay" | "deterministic-game" => {
            find_catalog_scenario("Deterministic Game Behavior")
                .map(|scenario| Box::new(scenario) as _)
        }
        "encounter-choices" | "encounters" => find_catalog_scenario("Encounter Choice Processing")
            .map(|scenario| Box::new(scenario) as _),
        "vehicle-system" | "vehicle" => find_catalog_scenario("Vehicle System Integration")
            .map(|scenario| Box::new(scenario) as _),
        "weather-effects" | "weather" => {
            find_catalog_scenario("Weather System Effects").map(|scenario| Box::new(scenario) as _)
        }
        "resource-management" | "resources" => {
            find_catalog_scenario("Resource Management").map(|scenario| Box::new(scenario) as _)
        }
        "stats-boundaries" | "stats" => find_catalog_scenario("Stats Boundary Conditions")
            .map(|scenario| Box::new(scenario) as _),
        "inventory-operations" | "inventory" => {
            find_catalog_scenario("Inventory Operations").map(|scenario| Box::new(scenario) as _)
        }
        "game-mode-variations" | "game-modes" => {
            find_catalog_scenario("Game Mode Variations").map(|scenario| Box::new(scenario) as _)
        }
        _ => None,
    }
}

pub fn list_scenarios() -> Vec<(&'static str, &'static str)> {
    vec![
        ("smoke", "Smoke Test"),
        ("real-game", "Real Game Test"),
        ("conservative-strategy", "Conservative Strategy Test"),
        ("aggressive-strategy", "Aggressive Strategy Test"),
        ("resource-manager", "Resource Manager Test"),
        (
            "full-game-conservative",
            "Full Game - Conservative Strategy",
        ),
        ("full-game-aggressive", "Full Game - Aggressive Strategy"),
        ("full-game-balanced", "Full Game - Balanced Strategy"),
        ("resource-stress", "Resource Management Stress Test"),
        ("deterministic", "Deterministic Playthrough Verification"),
        ("edge-case", "Edge Case Survival Test"),
        ("basic-game-creation", "Basic Game State Creation"),
        (
            "share-code-consistency",
            "Share Code Generation and Parsing",
        ),
        ("deterministic-gameplay", "Deterministic Game Behavior"),
        ("encounter-choices", "Encounter Choice Processing"),
        ("vehicle-system", "Vehicle System Integration"),
        ("weather-effects", "Weather System Effects"),
        ("resource-management", "Resource Management"),
        ("stats-boundaries", "Stats Boundary Conditions"),
        ("inventory-operations", "Inventory Operations"),
        ("game-mode-variations", "Game Mode Variations"),
    ]
}
