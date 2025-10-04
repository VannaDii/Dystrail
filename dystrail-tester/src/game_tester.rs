use anyhow::Result;
use colored::*;
use dystrail_web::game::{GameMode, GameState};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::scenarios::{ScenarioResult, TestScenario};

pub struct GameTester {
    rng: ChaCha8Rng,
    verbose: bool,
}

impl GameTester {
    pub fn new(verbose: bool) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(42); // Default seed
        Self { rng, verbose }
    }

    pub fn set_seed(&mut self, seed_str: &str) {
        // Try to parse as share code first, then as raw seed
        if let Ok(seed) = dystrail_web::game::seed::decode_to_seed(seed_str) {
            self.rng = ChaCha8Rng::seed_from_u64(seed);
        } else if let Ok(seed) = seed_str.parse::<u64>() {
            self.rng = ChaCha8Rng::seed_from_u64(seed);
        } else {
            // Use string hash as seed
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            seed_str.hash(&mut hasher);
            self.rng = ChaCha8Rng::seed_from_u64(hasher.finish());
        }
    }

    pub async fn run_scenarios(
        &mut self,
        scenario_names: &[String],
        iterations: usize,
    ) -> Result<Vec<ScenarioResult>> {
        let mut results = Vec::new();

        let scenarios = if scenario_names.contains(&"all".to_string()) {
            crate::scenarios::get_all_scenarios()
        } else {
            crate::scenarios::get_scenarios_by_names(scenario_names)
        };

        for scenario in scenarios {
            if self.verbose {
                println!("🧪 Testing scenario: {}", scenario.name.bright_white());
            }

            let result = self.run_scenario(&scenario, iterations).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn run_scenario(
        &mut self,
        scenario: &TestScenario,
        iterations: usize,
    ) -> Result<ScenarioResult> {
        let mut successes = 0;
        let mut failures = Vec::new();
        let mut performance_data = Vec::new();

        for i in 0..iterations {
            let start_time = std::time::Instant::now();

            // Create a fresh game state for each iteration
            let mut game_state =
                GameState::new(scenario.mode.clone(), scenario.seed_base + i as u64);

            // Apply scenario setup
            if let Some(setup) = &scenario.setup {
                setup(&mut game_state);
            }

            // Run the test
            match self.run_single_test(&mut game_state, scenario).await {
                Ok(_) => {
                    successes += 1;
                    let duration = start_time.elapsed();
                    performance_data.push(duration);

                    if self.verbose {
                        println!(
                            "  ✅ Iteration {}/{} passed ({:?})",
                            i + 1,
                            iterations,
                            duration
                        );
                    }
                }
                Err(e) => {
                    failures.push(format!("Iteration {}: {}", i + 1, e));

                    if self.verbose {
                        println!(
                            "  ❌ Iteration {}/{} failed: {}",
                            i + 1,
                            iterations,
                            e.to_string().red()
                        );
                    }
                }
            }
        }

        let avg_duration = if !performance_data.is_empty() {
            performance_data.iter().sum::<std::time::Duration>() / performance_data.len() as u32
        } else {
            std::time::Duration::ZERO
        };

        Ok(ScenarioResult {
            scenario_name: scenario.name.clone(),
            passed: failures.is_empty(),
            iterations_run: iterations,
            successful_iterations: successes,
            failures,
            average_duration: avg_duration,
            performance_data,
        })
    }

    async fn run_single_test(
        &mut self,
        game_state: &mut GameState,
        scenario: &TestScenario,
    ) -> Result<()> {
        // Run the test function
        (scenario.test_fn)(game_state, &mut self.rng)
    }
}
