use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use dystrail_game::GameState;

// LogicTestResult removed as it was unused

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

pub struct LogicTester {
    verbose: bool,
}

impl LogicTester {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn run_scenario(
        &mut self,
        scenario: &crate::common::scenario::TestScenario,
        seeds: &[u64],
        iterations: usize,
    ) -> Vec<ScenarioResult> {
        let mut results = Vec::new();

        for &seed in seeds {
            if self.verbose {
                println!(
                    "🧪 Testing scenario: {} (seed: {})",
                    scenario.name.bright_white(),
                    seed
                );
            }

            let result = self.run_single_scenario(scenario, seed, iterations);
            results.push(result);
        }

        results
    }

    fn run_single_scenario(
        &mut self,
        scenario: &crate::common::scenario::TestScenario,
        seed: u64,
        iterations: usize,
    ) -> ScenarioResult {
        let mut successes = 0;
        let mut failures = Vec::new();
        let mut performance_data = Vec::new();

        for i in 0..iterations {
            let start_time = Instant::now();

            // Create a fresh game state for each iteration
            let mut game_state = GameState {
                seed: seed.wrapping_add(u64::try_from(i).unwrap_or(u64::MAX)),
                ..Default::default()
            };

            // Apply scenario setup
            if let Some(setup) = &scenario.setup {
                setup(&mut game_state);
            }

            // Run the test
            match (scenario.test_fn)(&mut game_state) {
                Ok(()) => {
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

        let avg_duration = if performance_data.is_empty() {
            Duration::ZERO
        } else {
            performance_data.iter().sum::<Duration>()
                / u32::try_from(performance_data.len()).unwrap_or(1)
        };

        ScenarioResult {
            scenario_name: scenario.name.clone(),
            passed: failures.is_empty(),
            iterations_run: iterations,
            successful_iterations: successes,
            failures,
            average_duration: avg_duration,
            performance_data,
        }
    }
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
        Ok(Duration::from_millis(u64::try_from(millis).unwrap_or(0)))
    }
}

mod duration_vec_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(durations: &[Duration], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis: Vec<u128> = durations
            .iter()
            .map(std::time::Duration::as_millis)
            .collect();
        millis.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis_vec = Vec::<u128>::deserialize(deserializer)?;
        Ok(millis_vec
            .into_iter()
            .map(|m| Duration::from_millis(u64::try_from(m).unwrap_or(0)))
            .collect())
    }
}
