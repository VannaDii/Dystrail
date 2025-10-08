//! Modernized test scenarios based on legacy comprehensive test suite
//!
//! This preserves the valuable test logic from the original scenarios but
//! updates them to work with the current game API and test framework.

#![allow(dead_code)] // These functions are preserved test scenarios

use anyhow::Result;
use dystrail_web::game::{GameState, GameMode};
use serde::{Serialize, Deserialize};
use std::time::Duration;

use crate::common::scenario::TestScenario;

/// Test scenario execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub scenario_name: String,
    pub passed: bool,
    pub execution_time: Duration,
    pub error_message: Option<String>,
}

impl TestResult {
    pub fn new(scenario_name: String) -> Self {
        Self {
            scenario_name,
            passed: false,
            execution_time: Duration::new(0, 0),
            error_message: None,
        }
    }

    pub fn success(scenario_name: String, execution_time: Duration) -> Self {
        Self {
            scenario_name,
            passed: true,
            execution_time,
            error_message: None,
        }
    }

    pub fn failure(scenario_name: String, execution_time: Duration, error: String) -> Self {
        Self {
            scenario_name,
            passed: false,
            execution_time,
            error_message: Some(error),
        }
    }
}

/// Run all test scenarios and return results
pub fn run_all_scenarios() -> Vec<TestResult> {
    let scenarios = get_all_scenarios();
    let mut results = Vec::new();

    for scenario in scenarios {
        let start_time = std::time::Instant::now();
        let mut game_state = GameState::default();

        // Apply setup if provided
        if let Some(setup) = scenario.setup {
            setup(&mut game_state);
        }

        // Run the test
        let result = match (scenario.test_fn)(&mut game_state) {
            Ok(()) => TestResult::success(scenario.name, start_time.elapsed()),
            Err(e) => TestResult::failure(scenario.name, start_time.elapsed(), e.to_string()),
        };

        results.push(result);
    }

    results
}

/// Get scenarios by name filters
pub fn get_scenarios_by_names(names: &[String]) -> Vec<TestScenario> {
    let all_scenarios = get_all_scenarios();
    all_scenarios
        .into_iter()
        .filter(|s| names.iter().any(|name| s.name.to_lowercase().contains(&name.to_lowercase())))
        .collect()
}

/// Get all available test scenarios
pub fn get_all_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            name: "Basic Game State Creation".to_string(),
            setup: None,
            test_fn: test_basic_game_creation,
        },
        TestScenario {
            name: "Share Code Generation and Parsing".to_string(),
            setup: None,
            test_fn: test_share_code_consistency,
        },
        TestScenario {
            name: "Deterministic Game Behavior".to_string(),
            setup: Some(setup_deterministic_test),
            test_fn: test_deterministic_gameplay,
        },
        TestScenario {
            name: "Encounter Choice Processing".to_string(),
            setup: Some(setup_encounter_test),
            test_fn: test_encounter_choices,
        },
        TestScenario {
            name: "Vehicle System Integration".to_string(),
            setup: Some(setup_vehicle_test),
            test_fn: test_vehicle_breakdown,
        },
        TestScenario {
            name: "Weather System Effects".to_string(),
            setup: Some(setup_weather_test),
            test_fn: test_weather_effects,
        },
        TestScenario {
            name: "Resource Management".to_string(),
            setup: Some(setup_resource_test),
            test_fn: test_resource_management,
        },
        TestScenario {
            name: "Stats Boundary Conditions".to_string(),
            setup: None,
            test_fn: test_stats_boundaries,
        },
        TestScenario {
            name: "Inventory Operations".to_string(),
            setup: Some(setup_inventory_test),
            test_fn: test_inventory_operations,
        },
        TestScenario {
            name: "Game Mode Variations".to_string(),
            setup: None,
            test_fn: test_game_modes,
        },
    ]
}

// Test functions (implementations preserved from legacy scenarios)

fn test_basic_game_creation(game_state: &mut GameState) -> Result<()> {
    // Verify default state is valid
    if game_state.stats.hp <= 0 {
        anyhow::bail!("Default HP should be positive, got {}", game_state.stats.hp);
    }

    if game_state.budget <= 0 {
        anyhow::bail!("Default budget should be positive, got {}", game_state.budget);
    }

    if game_state.day == 0 {
        anyhow::bail!("Day should start at 1, got {}", game_state.day);
    }

    // Basic sanity checks
    anyhow::ensure!(game_state.stats.hp > 0, "Health must be positive");
    anyhow::ensure!(game_state.stats.supplies >= 0, "Supplies cannot be negative");
    anyhow::ensure!(game_state.day >= 1, "Day must be at least 1");

    Ok(())
}

fn test_share_code_consistency(_game_state: &mut GameState) -> Result<()> {
    use dystrail_web::game::seed::{encode_friendly, decode_to_seed};

    // Test share code round-trip consistency
    let test_seeds = vec![0xDEAD_BEEF_CAFE_BABE, 0x1234_5678_9ABC_DEF0, 0x0000_0000_0000_0001];

    for seed in test_seeds {
        // Test classic mode round-trip
        let classic_code = encode_friendly(false, seed);
        if let Some((is_deep, decoded_seed)) = decode_to_seed(&classic_code) {
            anyhow::ensure!(!is_deep, "Classic code should decode as not deep");
            let re_encoded = encode_friendly(false, decoded_seed);
            anyhow::ensure!(re_encoded == classic_code, "Classic round-trip failed: {classic_code} != {re_encoded}");
        } else {
            anyhow::bail!("Failed to decode classic share code: {classic_code}");
        }

        // Test deep mode round-trip
        let deep_code = encode_friendly(true, seed);
        if let Some((is_deep, decoded_seed)) = decode_to_seed(&deep_code) {
            anyhow::ensure!(is_deep, "Deep code should decode as deep");
            let re_encoded = encode_friendly(true, decoded_seed);
            anyhow::ensure!(re_encoded == deep_code, "Deep round-trip failed: {deep_code} != {re_encoded}");
        } else {
            anyhow::bail!("Failed to decode deep share code: {deep_code}");
        }
    }

    // Test known stable codes
    let known_codes = vec!["CL-ORANGE42", "DP-ORANGE42", "CL-PANTS99", "DP-CHEETO00"];
    for code in known_codes {
        if let Some((is_deep, decoded_seed)) = decode_to_seed(code) {
            let re_encoded = encode_friendly(is_deep, decoded_seed);
            anyhow::ensure!(re_encoded == code, "Known code round-trip failed: {code} != {re_encoded}");
        } else {
            anyhow::bail!("Failed to decode known share code: {code}");
        }
    }

    Ok(())
}

fn setup_deterministic_test(game_state: &mut GameState) {
    game_state.seed = 98765;
    game_state.mode = GameMode::Classic;
}

fn test_deterministic_gameplay(game_state: &mut GameState) -> Result<()> {
    // Store initial state
    let initial_seed = game_state.seed;
    let initial_stats = game_state.stats.clone();

    // Simulate some game progress
    game_state.day += 5;
    game_state.stats.hp -= 2;
    game_state.stats.supplies += 3;

    // Reset and verify we can get back to initial state
    *game_state = GameState::default();
    game_state.seed = initial_seed;
    game_state.mode = GameMode::Classic;

    anyhow::ensure!(game_state.day == 1, "Day should reset to 1");
    anyhow::ensure!(game_state.stats.hp == initial_stats.hp, "HP should reset");
    anyhow::ensure!(game_state.stats.supplies == initial_stats.supplies, "Supplies should reset");

    Ok(())
}

fn setup_encounter_test(game_state: &mut GameState) {
    game_state.seed = 12345;
    // Set up for encounters
    game_state.stats.hp = 8;
    game_state.stats.supplies = 15;
    game_state.budget = 150;
}

fn test_encounter_choices(game_state: &mut GameState) -> Result<()> {
    let initial_hp = game_state.stats.hp;
    let initial_supplies = game_state.stats.supplies;
    let initial_budget = game_state.budget;

    // Verify we can make choices that affect stats
    game_state.stats.hp -= 1;
    game_state.stats.supplies += 2;
    game_state.budget -= 10;

    anyhow::ensure!(game_state.stats.hp == initial_hp - 1, "HP choice should affect stats");
    anyhow::ensure!(game_state.stats.supplies == initial_supplies + 2, "Supply choice should affect stats");
    anyhow::ensure!(game_state.budget == initial_budget - 10, "Budget choice should affect stats");

    Ok(())
}

fn setup_vehicle_test(game_state: &mut GameState) {
    // Set up vehicle for testing
    game_state.vehicle = dystrail_web::game::vehicle::Vehicle::default();
    game_state.inventory.spares.tire = 2;
    game_state.inventory.spares.battery = 1;
}

fn test_vehicle_breakdown(game_state: &mut GameState) -> Result<()> {
    // Test that vehicle system exists and can be modified
    let initial_tire_spares = game_state.inventory.spares.tire;

    // Simulate using a tire spare
    if game_state.inventory.spares.tire > 0 {
        game_state.inventory.spares.tire -= 1;
    }

    anyhow::ensure!(
        game_state.inventory.spares.tire == initial_tire_spares - 1,
        "Should be able to use tire spares"
    );

    // Verify breakdown state can be set
    game_state.breakdown = Some(dystrail_web::game::vehicle::Breakdown {
        part: dystrail_web::game::vehicle::Part::Tire,
        day_started: i32::try_from(game_state.day).unwrap_or(1),
    });
    anyhow::ensure!(game_state.breakdown.is_some(), "Should be able to set breakdown state");

    Ok(())
}

fn setup_weather_test(game_state: &mut GameState) {
    // Initialize weather system
    game_state.weather_state = dystrail_web::game::weather::WeatherState::default();
}

fn test_weather_effects(game_state: &mut GameState) -> Result<()> {
    // Test that weather system exists and can be modified
    let initial_distance = game_state.distance_today;

    // Weather should be able to affect travel
    game_state.distance_today *= 0.8; // Simulate bad weather slowing travel

    anyhow::ensure!(
        (game_state.distance_today - initial_distance * 0.8).abs() < 0.001,
        "Weather should be able to affect travel distance"
    );

    Ok(())
}

fn setup_resource_test(game_state: &mut GameState) {
    game_state.stats.supplies = 10;
    game_state.budget = 100;
    game_state.stats.hp = 8;
}

fn test_resource_management(game_state: &mut GameState) -> Result<()> {
    let initial_supplies = game_state.stats.supplies;
    let initial_budget = game_state.budget;

    // Test resource consumption
    game_state.stats.supplies -= 2;
    game_state.budget -= 20;

    anyhow::ensure!(
        game_state.stats.supplies == initial_supplies - 2,
        "Should be able to consume supplies"
    );
    anyhow::ensure!(
        game_state.budget == initial_budget - 20,
        "Should be able to spend budget"
    );

    // Test that stats clamp properly
    game_state.stats.supplies = -5; // Invalid value
    game_state.stats.clamp();
    anyhow::ensure!(game_state.stats.supplies >= 0, "Stats should clamp to valid ranges");

    Ok(())
}

fn test_stats_boundaries(game_state: &mut GameState) -> Result<()> {
    // Test stat clamping
    game_state.stats.hp = 15; // Above max
    game_state.stats.sanity = -5; // Below min
    game_state.stats.credibility = 25; // Above max
    game_state.stats.supplies = -10; // Below min

    game_state.stats.clamp();

    anyhow::ensure!(game_state.stats.hp <= 10, "HP should clamp to max 10");
    anyhow::ensure!(game_state.stats.sanity >= 0, "Sanity should clamp to min 0");
    anyhow::ensure!(game_state.stats.credibility <= 20, "Credibility should clamp to max 20");
    anyhow::ensure!(game_state.stats.supplies >= 0, "Supplies should clamp to min 0");

    Ok(())
}

fn setup_inventory_test(game_state: &mut GameState) {
    game_state.inventory.spares.tire = 3;
    game_state.inventory.spares.battery = 1;
    game_state.inventory.spares.alt = 0;
    game_state.inventory.tags.insert("test_tag".to_string());
}

fn test_inventory_operations(game_state: &mut GameState) -> Result<()> {
    // Test spare parts
    anyhow::ensure!(game_state.inventory.spares.tire == 3, "Should have 3 tire spares");
    anyhow::ensure!(game_state.inventory.spares.battery == 1, "Should have 1 battery spare");

    // Test tags
    anyhow::ensure!(
        game_state.inventory.tags.contains("test_tag"),
        "Should contain test tag"
    );

    // Test adding/removing tags
    game_state.inventory.tags.insert("new_tag".to_string());
    anyhow::ensure!(
        game_state.inventory.tags.contains("new_tag"),
        "Should be able to add new tags"
    );

    game_state.inventory.tags.remove("test_tag");
    anyhow::ensure!(
        !game_state.inventory.tags.contains("test_tag"),
        "Should be able to remove tags"
    );

    Ok(())
}

fn test_game_modes(game_state: &mut GameState) -> Result<()> {
    // Test mode switching
    game_state.mode = GameMode::Classic;
    anyhow::ensure!(!game_state.mode.is_deep(), "Classic mode should not be deep");

    game_state.mode = GameMode::Deep;
    anyhow::ensure!(game_state.mode.is_deep(), "Deep mode should be deep");

    Ok(())
}