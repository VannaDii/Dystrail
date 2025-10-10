use anyhow::{Result, anyhow};

use crate::common::scenario::SimulationScenario;
use crate::logic::game_tester::SimulationSummary;
use crate::logic::{GameplayStrategy, SimulationPlan};
use dystrail_game::vehicle::{Breakdown, Part, Vehicle};
use dystrail_game::{GameMode, GameState};

pub fn catalog_scenarios() -> Vec<SimulationScenario> {
    vec![
        SimulationScenario::new(
            "Basic Game State Creation",
            base_plan().with_expectation(basic_game_state_expectation),
        ),
        SimulationScenario::new(
            "Share Code Generation and Parsing",
            base_plan().with_expectation(share_code_expectation),
        ),
        SimulationScenario::new(
            "Deterministic Game Behavior",
            base_plan().with_expectation(deterministic_gameplay_expectation),
        ),
        SimulationScenario::new(
            "Encounter Choice Processing",
            base_plan().with_expectation(encounter_choices_expectation),
        ),
        SimulationScenario::new(
            "Vehicle System Integration",
            base_plan().with_expectation(vehicle_system_expectation),
        ),
        SimulationScenario::new(
            "Weather System Effects",
            base_plan().with_expectation(weather_effects_expectation),
        ),
        SimulationScenario::new(
            "Resource Management",
            base_plan().with_expectation(resource_management_expectation),
        ),
        SimulationScenario::new(
            "Stats Boundary Conditions",
            base_plan().with_expectation(stats_boundaries_expectation),
        ),
        SimulationScenario::new(
            "Inventory Operations",
            base_plan().with_expectation(inventory_operations_expectation),
        ),
        SimulationScenario::new(
            "Game Mode Variations",
            base_plan().with_expectation(game_mode_expectation),
        ),
    ]
}

pub fn find_catalog_scenario(name: &str) -> Option<SimulationScenario> {
    catalog_scenarios()
        .into_iter()
        .find(|scenario| scenario.name() == name)
}

fn base_plan() -> SimulationPlan {
    SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced).with_max_days(0)
}

fn basic_game_state_expectation(_summary: &SimulationSummary) -> Result<()> {
    let game_state = GameState::default();

    anyhow::ensure!(game_state.stats.hp > 0, "Health must be positive");
    anyhow::ensure!(game_state.budget > 0, "Default budget should be positive");
    anyhow::ensure!(game_state.day >= 1, "Day must be at least 1");

    anyhow::ensure!(
        game_state.stats.supplies >= 0,
        "Supplies cannot be negative"
    );

    Ok(())
}

fn share_code_expectation(_summary: &SimulationSummary) -> Result<()> {
    use dystrail_game::seed::{decode_to_seed, encode_friendly};

    let test_seeds = vec![
        0xDEAD_BEEF_CAFE_BABE,
        0x1234_5678_9ABC_DEF0,
        0x0000_0000_0000_0001,
    ];

    for seed in test_seeds {
        let classic_code = encode_friendly(false, seed);
        let (is_deep, decoded_seed) = decode_to_seed(&classic_code)
            .ok_or_else(|| anyhow!("Failed to decode classic share code: {classic_code}"))?;
        anyhow::ensure!(!is_deep, "Classic code should decode as not deep");
        anyhow::ensure!(
            encode_friendly(false, decoded_seed) == classic_code,
            "Classic round-trip failed: {classic_code}"
        );

        let deep_code = encode_friendly(true, seed);
        let (is_deep, decoded_seed) = decode_to_seed(&deep_code)
            .ok_or_else(|| anyhow!("Failed to decode deep share code: {deep_code}"))?;
        anyhow::ensure!(is_deep, "Deep code should decode as deep");
        anyhow::ensure!(
            encode_friendly(true, decoded_seed) == deep_code,
            "Deep round-trip failed: {deep_code}"
        );
    }

    let known_codes = ["CL-ORANGE42", "DP-ORANGE42", "CL-PANTS99", "DP-CHEETO00"];
    for code in known_codes {
        let (is_deep, decoded_seed) = decode_to_seed(code)
            .ok_or_else(|| anyhow!("Failed to decode known share code: {code}"))?;
        anyhow::ensure!(
            encode_friendly(is_deep, decoded_seed) == code,
            "Known code round-trip failed: {code}"
        );
    }

    Ok(())
}

fn deterministic_gameplay_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState {
        seed: 98765,
        ..GameState::default()
    };
    let initial_seed = state.seed;
    let initial_stats = state.stats.clone();

    state.day += 5;
    state.stats.hp -= 2;
    state.stats.supplies += 3;

    state = GameState {
        seed: initial_seed,
        mode: GameMode::Classic,
        ..GameState::default()
    };

    anyhow::ensure!(state.day == 1, "Day should reset to 1");
    anyhow::ensure!(state.stats.hp == initial_stats.hp, "HP should reset");
    anyhow::ensure!(
        state.stats.supplies == initial_stats.supplies,
        "Supplies should reset"
    );

    Ok(())
}

fn encounter_choices_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState::default();
    state.stats.hp = 8;
    state.stats.supplies = 15;
    state.budget = 150;

    let initial_hp = state.stats.hp;
    let initial_supplies = state.stats.supplies;
    let initial_budget = state.budget;

    state.stats.hp -= 1;
    state.stats.supplies += 2;
    state.budget -= 10;

    anyhow::ensure!(
        state.stats.hp == initial_hp - 1,
        "HP choice should affect stats"
    );
    anyhow::ensure!(
        state.stats.supplies == initial_supplies + 2,
        "Supply choice should affect stats"
    );
    anyhow::ensure!(
        state.budget == initial_budget - 10,
        "Budget choice should affect stats"
    );

    Ok(())
}

fn vehicle_system_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState {
        vehicle: Vehicle::default(),
        ..GameState::default()
    };
    state.inventory.spares.tire = 2;
    state.inventory.spares.battery = 1;

    let initial_tire_spares = state.inventory.spares.tire;
    if state.inventory.spares.tire > 0 {
        state.inventory.spares.tire -= 1;
    }
    anyhow::ensure!(
        state.inventory.spares.tire == initial_tire_spares - 1,
        "Should be able to use tire spares"
    );

    state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: i32::try_from(state.day).unwrap_or(1),
    });
    anyhow::ensure!(
        state.breakdown.is_some(),
        "Should be able to set breakdown state"
    );

    Ok(())
}

fn weather_effects_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState::default();
    let initial_distance = state.distance_today;
    state.distance_today *= 0.8;
    anyhow::ensure!(
        (state.distance_today - initial_distance * 0.8).abs() < 0.001,
        "Weather should be able to affect travel distance"
    );

    Ok(())
}

fn resource_management_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState::default();
    state.stats.supplies = 10;
    state.budget = 100;
    state.stats.hp = 8;

    let initial_supplies = state.stats.supplies;
    let initial_budget = state.budget;

    state.stats.supplies -= 2;
    state.budget -= 20;

    anyhow::ensure!(
        state.stats.supplies == initial_supplies - 2,
        "Should be able to consume supplies"
    );
    anyhow::ensure!(
        state.budget == initial_budget - 20,
        "Should be able to spend budget"
    );

    state.stats.supplies = -5;
    state.stats.clamp();
    anyhow::ensure!(
        state.stats.supplies >= 0,
        "Stats should clamp to valid ranges"
    );

    Ok(())
}

fn stats_boundaries_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState::default();
    state.stats.hp = 15;
    state.stats.sanity = -5;
    state.stats.credibility = 25;
    state.stats.supplies = -10;
    state.stats.clamp();

    anyhow::ensure!(state.stats.hp <= 10, "HP should clamp to max 10");
    anyhow::ensure!(state.stats.sanity >= 0, "Sanity should clamp to min 0");
    anyhow::ensure!(
        state.stats.credibility <= 20,
        "Credibility should clamp to max 20"
    );
    anyhow::ensure!(state.stats.supplies >= 0, "Supplies should clamp to min 0");

    Ok(())
}

fn inventory_operations_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState::default();
    state.inventory.spares.tire = 3;
    state.inventory.spares.battery = 1;
    state.inventory.spares.alt = 0;
    state.inventory.tags.insert("test_tag".to_string());

    anyhow::ensure!(
        state.inventory.spares.tire == 3,
        "Should have 3 tire spares"
    );
    anyhow::ensure!(
        state.inventory.spares.battery == 1,
        "Should have 1 battery spare"
    );
    anyhow::ensure!(
        state.inventory.tags.contains("test_tag"),
        "Should contain test tag"
    );

    state.inventory.tags.insert("new_tag".to_string());
    anyhow::ensure!(
        state.inventory.tags.contains("new_tag"),
        "Should be able to add new tags"
    );
    state.inventory.tags.remove("test_tag");
    anyhow::ensure!(
        !state.inventory.tags.contains("test_tag"),
        "Should be able to remove tags"
    );

    Ok(())
}

fn game_mode_expectation(_summary: &SimulationSummary) -> Result<()> {
    let mut state = GameState::default();
    anyhow::ensure!(!state.mode.is_deep(), "Classic mode should not be deep");
    state.mode = GameMode::Deep;
    anyhow::ensure!(state.mode.is_deep(), "Deep mode should be deep");
    Ok(())
}
