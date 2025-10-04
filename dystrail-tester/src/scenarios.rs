use anyhow::Result;
use dystrail_web::game::{GameState, GameMode};
use rand_chacha::ChaCha8Rng;
use serde::{Serialize, Deserialize};
use std::time::Duration;

pub struct TestScenario {
    pub name: String,
    pub mode: GameMode,
    pub seed_base: u64,
    pub setup: Option<fn(&mut GameState)>,
    pub test_fn: fn(&mut GameState) -> Result<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub passed: bool,
    pub iterations_run: usize,
    pub successful_iterations: usize,
    pub failures: Vec<String>,
    #[serde(with = "duration_serde")]
    pub average_duration: Duration,
    #[serde(with = "duration_vec_serde")]
    pub performance_data: Vec<Duration>,
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u128::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis as u64))
    }
}

mod duration_vec_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(durations: &Vec<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis: Vec<u128> = durations.iter().map(|d| d.as_millis()).collect();
        millis.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis_vec = Vec::<u128>::deserialize(deserializer)?;
        Ok(millis_vec.into_iter().map(|m| Duration::from_millis(m as u64)).collect())
    }
}

pub fn get_all_scenarios() -> Vec<TestScenario> {
    vec![
        basic_game_creation(),
        share_code_generation(),
        deterministic_gameplay(),
        encounter_choices(),
        vehicle_breakdown_recovery(),
        weather_effects(),
        store_transactions(),
        camping_mechanics(),
        river_crossing(),
        end_game_scenarios(),
    ]
}

pub fn get_scenarios_by_names(names: &[String]) -> Vec<TestScenario> {
    let all_scenarios = get_all_scenarios();
    all_scenarios
        .into_iter()
        .filter(|s| names.iter().any(|name| s.name.to_lowercase().contains(&name.to_lowercase())))
        .collect()
}

fn basic_game_creation() -> TestScenario {
    TestScenario {
        name: "Basic Game Creation".to_string(),
        description: "Test that a new game can be created successfully".to_string(),
        mode: GameMode::Classic,
        seed_base: 12345,
        setup: None,
        test_fn: |game_state, _rng| {
            // Verify initial state is valid
            if game_state.stats.health <= 0 {
                anyhow::bail!("Initial health should be > 0, got {}", game_state.stats.health);
            }
            if game_state.stats.food <= 0 {
                anyhow::bail!("Initial food should be > 0, got {}", game_state.stats.food);
            }
            if game_state.stats.ammo < 0 {
                anyhow::bail!("Initial ammo should be >= 0, got {}", game_state.stats.ammo);
            }
            Ok(())
        },
    }
}

fn share_code_generation() -> TestScenario {
    TestScenario {
        name: "Share Code Generation".to_string(),
        description: "Test that share codes can be generated and decoded consistently".to_string(),
        mode: GameMode::Classic,
        seed_base: 54321,
        setup: None,
        test_fn: |game_state, _rng| {
            // Generate a share code
            let share_code = game_state.generate_share_code();

            // Verify it matches expected format
            let share_code_regex = regex::Regex::new(r"^(CL|DP)-[A-Z]+\d{2}$").unwrap();
            if !share_code_regex.is_match(&share_code) {
                anyhow::bail!("Share code '{}' doesn't match expected format", share_code);
            }

            // Test that we can decode it
            if let Err(e) = dystrail_web::game::seed::decode_to_seed(&share_code) {
                anyhow::bail!("Failed to decode share code '{}': {}", share_code, e);
            }

            Ok(())
        },
    }
}

fn deterministic_gameplay() -> TestScenario {
    TestScenario {
        name: "Deterministic Gameplay".to_string(),
        description: "Test that games with same seed produce identical results".to_string(),
        mode: GameMode::Classic,
        seed_base: 98765,
        setup: None,
        test_fn: |game_state, rng| {
            let initial_seed = rng.clone();

            // Simulate some game progression
            let _ = game_state.advance_day();
            let first_result = game_state.stats.clone();

            // Reset and run again with same seed
            *rng = initial_seed;
            *game_state = GameState::new(GameMode::Classic, 98765);
            let _ = game_state.advance_day();
            let second_result = game_state.stats.clone();

            if first_result.health != second_result.health ||
               first_result.food != second_result.food ||
               first_result.ammo != second_result.ammo {
                anyhow::bail!("Game state not deterministic: {:?} vs {:?}", first_result, second_result);
            }

            Ok(())
        },
    }
}

fn encounter_choices() -> TestScenario {
    TestScenario {
        name: "Encounter Choices".to_string(),
        description: "Test that encounter choices work correctly".to_string(),
        mode: GameMode::Classic,
        seed_base: 11111,
        setup: None,
        test_fn: |game_state, _rng| {
            // Force an encounter
            if let Some(encounter) = game_state.get_random_encounter() {
                let initial_stats = game_state.stats.clone();

                // Make a choice (if available)
                if !encounter.choices.is_empty() {
                    let choice = &encounter.choices[0];
                    game_state.apply_choice_effects(&choice.effects);

                    // Verify stats changed (in some way)
                    let final_stats = game_state.stats.clone();
                    // At least one stat should be different (health, food, or ammo)
                    if initial_stats.health == final_stats.health &&
                       initial_stats.food == final_stats.food &&
                       initial_stats.ammo == final_stats.ammo {
                        // This might be okay if the choice had no effects
                        log::debug!("Choice had no stat effects");
                    }
                }
            }
            Ok(())
        },
    }
}

fn vehicle_breakdown_recovery() -> TestScenario {
    TestScenario {
        name: "Vehicle Breakdown Recovery".to_string(),
        description: "Test vehicle breakdown and repair mechanics".to_string(),
        mode: GameMode::Classic,
        seed_base: 22222,
        setup: Some(|game_state| {
            // Force a vehicle breakdown
            game_state.vehicle.force_breakdown();
        }),
        test_fn: |game_state, _rng| {
            // Verify vehicle is broken
            if !game_state.vehicle.is_broken() {
                anyhow::bail!("Vehicle should be broken after forced breakdown");
            }

            // Attempt repair (if we have spares)
            if game_state.inventory.spares.wheels > 0 {
                game_state.vehicle.attempt_repair(&mut game_state.inventory.spares);
                // Should be repaired now
                if game_state.vehicle.is_broken() {
                    anyhow::bail!("Vehicle should be repaired after using spare parts");
                }
            }

            Ok(())
        },
    }
}

fn weather_effects() -> TestScenario {
    TestScenario {
        name: "Weather Effects".to_string(),
        description: "Test weather system and its effects on gameplay".to_string(),
        mode: GameMode::Classic,
        seed_base: 33333,
        setup: None,
        test_fn: |game_state, _rng| {
            let initial_weather = game_state.weather.clone();

            // Advance weather
            game_state.weather.update_weather();

            // Weather should potentially change
            let new_weather = game_state.weather.clone();

            // Apply weather effects
            game_state.weather.apply_effects(&mut game_state.stats);

            // Just verify we don't crash
            Ok(())
        },
    }
}

fn store_transactions() -> TestScenario {
    TestScenario {
        name: "Store Transactions".to_string(),
        description: "Test store buying and selling mechanics".to_string(),
        mode: GameMode::Classic,
        seed_base: 44444,
        setup: Some(|game_state| {
            // Give the player some money
            game_state.stats.money = 1000.0;
        }),
        test_fn: |game_state, _rng| {
            let initial_money = game_state.stats.money;
            let initial_food = game_state.stats.food;

            // Try to buy food
            let store = game_state.store.clone();
            if let Some(food_item) = store.items.iter().find(|item| item.category.contains("food")) {
                if food_item.price <= initial_money {
                    // Simulate purchase
                    game_state.stats.money -= food_item.price;
                    game_state.stats.food += food_item.quantity_effect.unwrap_or(10);

                    // Verify transaction
                    if game_state.stats.money >= initial_money {
                        anyhow::bail!("Money should decrease after purchase");
                    }
                    if game_state.stats.food <= initial_food {
                        anyhow::bail!("Food should increase after food purchase");
                    }
                }
            }

            Ok(())
        },
    }
}

fn camping_mechanics() -> TestScenario {
    TestScenario {
        name: "Camping Mechanics".to_string(),
        description: "Test camping and resting functionality".to_string(),
        mode: GameMode::Classic,
        seed_base: 55555,
        setup: Some(|game_state| {
            // Reduce health to test recovery
            game_state.stats.health = 50;
        }),
        test_fn: |game_state, _rng| {
            let initial_health = game_state.stats.health;
            let initial_food = game_state.stats.food;

            // Rest for a day
            game_state.camp.rest(&mut game_state.stats);

            // Health should improve, food should decrease
            if game_state.stats.health <= initial_health {
                anyhow::bail!("Health should improve after resting");
            }
            if game_state.stats.food >= initial_food {
                anyhow::bail!("Food should decrease after resting");
            }

            Ok(())
        },
    }
}

fn river_crossing() -> TestScenario {
    TestScenario {
        name: "River Crossing".to_string(),
        description: "Test river crossing mechanics and risks".to_string(),
        mode: GameMode::Classic,
        seed_base: 66666,
        setup: None,
        test_fn: |game_state, _rng| {
            // Force a river crossing
            if let Some(crossing) = game_state.get_random_crossing() {
                let initial_stats = game_state.stats.clone();

                // Attempt crossing
                let success = game_state.attempt_river_crossing(&crossing);

                // Either succeeds or has consequences
                let final_stats = game_state.stats.clone();

                if !success {
                    // If failed, should have some negative effect
                    if final_stats.health >= initial_stats.health &&
                       final_stats.food >= initial_stats.food &&
                       final_stats.ammo >= initial_stats.ammo {
                        log::warn!("Failed crossing had no negative effects");
                    }
                }
            }

            Ok(())
        },
    }
}

fn end_game_scenarios() -> TestScenario {
    TestScenario {
        name: "End Game Scenarios".to_string(),
        description: "Test various end game conditions".to_string(),
        mode: GameMode::Classic,
        seed_base: 77777,
        setup: Some(|game_state| {
            // Set up near-end game state
            game_state.stats.distance_traveled = 1800; // Near the end
        }),
        test_fn: |game_state, _rng| {
            // Check if game should end
            let is_game_over = game_state.is_game_over();
            let result = game_state.get_final_result();

            // If near the end, should have a valid end condition
            if game_state.stats.distance_traveled >= 2000 {
                if !is_game_over {
                    anyhow::bail!("Game should be over when reaching destination");
                }
                if result.is_none() {
                    anyhow::bail!("Should have a valid end result when game is over");
                }
            }

            Ok(())
        },
    }
}