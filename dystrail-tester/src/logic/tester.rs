use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::common::scenario::TestScenario;
use crate::logic::game_tester::{GameTester, SimulationPlan, SimulationSummary};

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
    pub const fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn run_scenario(
        &self,
        scenario: &TestScenario,
        seeds: &[u64],
        iterations: usize,
    ) -> Vec<ScenarioResult> {
        let mut results = Vec::new();

        for &seed in seeds {
            if self.verbose {
                let mode_label = format!("{:?}", scenario.plan.mode);
                println!(
                    "🧪 Testing scenario: {} (mode: {} seed: {})",
                    scenario.name.bright_white(),
                    mode_label,
                    seed
                );
            }

            let result = self.run_single_scenario(scenario, seed, iterations);
            results.push(result);
        }

        results
    }

    fn run_single_scenario(
        &self,
        scenario: &TestScenario,
        seed: u64,
        iterations: usize,
    ) -> ScenarioResult {
        let (successes, failures, performance_data) =
            self.run_simulation_iterations(&scenario.plan, seed, iterations);

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

    fn run_simulation_iterations(
        &self,
        plan: &SimulationPlan,
        seed: u64,
        iterations: usize,
    ) -> (usize, Vec<String>, Vec<Duration>) {
        let tester = GameTester::try_new(self.verbose);

        let mut successes = 0;
        let mut failures = Vec::new();
        let mut performance_data = Vec::new();

        for i in 0..iterations {
            let start_time = Instant::now();
            let iteration_seed = seed.wrapping_add(u64::try_from(i).unwrap_or(u64::MAX));

            let summary = tester.run_plan(plan, iteration_seed);

            if let Some(err) = evaluate_expectations(plan, &summary) {
                let context = summarize_decision_path(&summary);
                let turns = summary.turns.len();
                let final_stats = &summary.final_state.stats;
                let status = if summary.game_ended {
                    "ended"
                } else {
                    "halted"
                };
                failures.push(format!(
                    "Iteration {} (mode {:?}, strategy {}, seed {}, turns {}, status {}, ending '{}'): {} | {} | final HP {} Supplies {} Sanity {} Pants {}",
                    i + 1,
                    summary.mode,
                    summary.strategy.label(),
                    summary.seed,
                    turns,
                    status,
                    summary.ending_message,
                    err,
                    context,
                    final_stats.hp,
                    final_stats.supplies,
                    final_stats.sanity,
                    final_stats.pants
                ));

                if self.verbose {
                    println!(
                        "  ❌ Iteration {}/{} failed: {}",
                        i + 1,
                        iterations,
                        err.clone().red()
                    );
                    println!(
                        "     ↳ Seed {} | Turns {} | Final HP {} Supplies {} Sanity {} Pants {} | Decisions: {}",
                        summary.seed,
                        turns,
                        final_stats.hp,
                        final_stats.supplies,
                        final_stats.sanity,
                        final_stats.pants,
                        context
                    );
                }
            } else {
                successes += 1;
                let duration = start_time.elapsed();
                performance_data.push(duration);

                if self.verbose {
                    println!(
                        "  ✅ Iteration {}/{} passed ({duration:?}) days:{} ending:{} strategy:{}",
                        i + 1,
                        iterations,
                        summary.metrics.days_survived,
                        summary.ending_message,
                        summary.strategy.label()
                    );
                }
            }
        }

        (successes, failures, performance_data)
    }
}

fn evaluate_expectations(plan: &SimulationPlan, summary: &SimulationSummary) -> Option<String> {
    for expectation in &plan.expectations {
        if let Err(err) = expectation(summary) {
            return Some(err.to_string());
        }
    }
    None
}

fn summarize_decision_path(summary: &SimulationSummary) -> String {
    if summary.metrics.decision_log.is_empty() {
        return "no decisions recorded".to_string();
    }

    summary
        .metrics
        .decision_log
        .iter()
        .rev()
        .take(3)
        .map(|entry| {
            let rationale = entry
                .rationale
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("-");
            format!(
                "day {} ({}): {} -> {} [{}] idx {} reason {}",
                entry.day,
                entry.encounter_id,
                entry.encounter_name,
                entry.choice_label,
                entry.policy_name,
                entry.choice_index,
                rationale
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
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
