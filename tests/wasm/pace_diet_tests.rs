//! WASM tests for pace and diet functionality

use dystrail::game::pacing::{DietCfg, PaceCfg, PacingConfig};
use dystrail::game::personas::Persona;
use dystrail::game::state::{GameMode, GameState, Region};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_pacing_config_validation() {
    // Test valid pace options
    let config = PacingConfig::from_json(
        r#"{
        "pace": {
            "steady": {"dist_mult": 1.0, "sanity_delta": 0, "pants_delta": 0, "encounter_chance_delta": 0.0},
            "heated": {"dist_mult": 1.5, "sanity_delta": 2, "pants_delta": 1, "encounter_chance_delta": 0.1},
            "blitz": {"dist_mult": 2.0, "sanity_delta": 5, "pants_delta": 3, "encounter_chance_delta": 0.3}
        },
        "diet": {
            "quiet": {"receipt_find_pct_delta": -0.2, "sanity_delta": -1, "pants_delta": 0},
            "mixed": {"receipt_find_pct_delta": 0.0, "sanity_delta": 0, "pants_delta": 0},
            "doom": {"receipt_find_pct_delta": 0.3, "sanity_delta": 3, "pants_delta": 1}
        }
    }"#,
    );

    assert!(config.is_ok(), "Valid pacing config should parse correctly");
    let config = config.unwrap();

    // Test safe getters
    assert!(config.get_pace_safe("steady").is_some());
    assert!(config.get_pace_safe("invalid").is_none());
    assert!(config.get_diet_safe("quiet").is_some());
    assert!(config.get_diet_safe("invalid").is_none());
}

#[wasm_bindgen_test]
fn test_pace_diet_effects() {
    let config = PacingConfig::from_json(r#"{
        "pace": {
            "steady": {"dist_mult": 1.0, "sanity_delta": 0, "pants_delta": 0, "encounter_chance_delta": 0.0},
            "heated": {"dist_mult": 1.5, "sanity_delta": 2, "pants_delta": 1, "encounter_chance_delta": 0.1}
        },
        "diet": {
            "quiet": {"receipt_find_pct_delta": -0.2, "sanity_delta": -1, "pants_delta": 0},
            "doom": {"receipt_find_pct_delta": 0.3, "sanity_delta": 3, "pants_delta": 1}
        }
    }"#).unwrap();

    let mut game_state = GameState::new(
        Persona::get_test_persona(),
        Region::Beltway,
        GameMode::Classic,
        42,
    );

    // Set pace and diet
    game_state.pace = "heated".to_string();
    game_state.diet = "doom".to_string();

    let initial_sanity = game_state.stats.sanity;
    let initial_pants = game_state.stats.pants;

    // Apply pace and diet effects
    game_state.apply_pace_and_diet(&config);

    // Check that effects were applied
    assert_eq!(game_state.stats.sanity, initial_sanity + 2 + 3); // heated + doom
    assert_eq!(game_state.stats.pants, initial_pants + 1 + 1); // heated + doom
    assert!((game_state.encounter_chance_today - 0.1).abs() < 0.01); // heated encounter chance
    assert!((game_state.receipt_bonus_pct - 0.3).abs() < 0.01); // doom receipt bonus
}

#[wasm_bindgen_test]
fn test_default_pace_diet() {
    let config = PacingConfig::default();

    let mut game_state = GameState::new(
        Persona::get_test_persona(),
        Region::Beltway,
        GameMode::Classic,
        42,
    );

    // Test default values work
    game_state.apply_pace_and_diet(&config);

    // Should not crash and should have reasonable defaults
    assert!(game_state.distance_today >= 0);
    assert!(game_state.encounter_chance_today >= 0.0);
    assert!(game_state.receipt_bonus_pct >= 0.0);
}
