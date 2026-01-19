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
    tester: GameTester,
}

impl LogicTester {
    pub const fn new(tester: GameTester) -> Self {
        Self { tester }
    }

    const fn is_verbose(&self) -> bool {
        self.tester.verbose()
    }

    pub fn run_scenario(
        &self,
        scenario: &TestScenario,
        seeds: &[u64],
        iterations: usize,
    ) -> Vec<ScenarioResult> {
        let mut results = Vec::new();
        let verbose = self.is_verbose();

        for &seed in seeds {
            if verbose {
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
        let tester = self.tester.clone();
        let verbose = self.is_verbose();

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

                if verbose {
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

                if verbose {
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
        if let Err(err) = expectation.evaluate(summary) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::GameplayStrategy;
    use crate::logic::game_tester::{
        GameTester, PlayabilityMetrics, SimulationSummary, TesterAssets,
    };
    use crate::logic::simulation::DecisionRecord;
    use dystrail_game::{GameMode, GameState};
    use std::sync::Arc;

    fn base_summary() -> SimulationSummary {
        SimulationSummary {
            seed: 1,
            mode: GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            turns: Vec::new(),
            metrics: PlayabilityMetrics::default(),
            final_state: GameState::default(),
            ending_message: "ok".to_string(),
            game_ended: false,
        }
    }

    #[test]
    fn summarize_decision_path_handles_empty() {
        let summary = base_summary();
        assert_eq!(summarize_decision_path(&summary), "no decisions recorded");
    }

    #[test]
    fn summarize_decision_path_includes_recent_entries() {
        let mut summary = base_summary();
        summary.metrics.decision_log = vec![
            DecisionRecord {
                day: 1,
                encounter_id: "enc1".into(),
                encounter_name: "Encounter".into(),
                choice_index: 0,
                choice_label: "Choice".into(),
                policy_name: "Policy".into(),
                rationale: Some("Because".into()),
            },
            DecisionRecord {
                day: 2,
                encounter_id: "enc2".into(),
                encounter_name: "Encounter 2".into(),
                choice_index: 1,
                choice_label: "Choice 2".into(),
                policy_name: "Policy".into(),
                rationale: None,
            },
        ];
        let summary_text = summarize_decision_path(&summary);
        assert!(summary_text.contains("enc2"));
    }

    #[test]
    fn evaluate_expectations_reports_errors() {
        fn always_fail(_: &SimulationSummary) -> anyhow::Result<()> {
            anyhow::bail!("expected failure");
        }

        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_expectation(always_fail);
        let summary = base_summary();
        let err = evaluate_expectations(&plan, &summary);
        assert!(err.is_some());
        assert!(err.unwrap().contains("expected failure"));
    }

    #[test]
    fn evaluate_expectations_returns_none_on_success() {
        fn ok_on_seed(summary: &SimulationSummary) -> anyhow::Result<()> {
            anyhow::ensure!(summary.seed == 1, "unexpected seed");
            Ok(())
        }

        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_expectation(ok_on_seed);
        let summary = base_summary();
        assert!(evaluate_expectations(&plan, &summary).is_none());
    }

    #[test]
    fn run_scenario_records_results() {
        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, false);
        let runner = LogicTester::new(tester);
        let plan =
            SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced).with_max_days(0);
        let scenario = TestScenario::simulation("smoke", plan);
        let results = runner.run_scenario(&scenario, &[1_u64], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].iterations_run, 1);
    }

    #[test]
    fn run_scenario_records_failures() {
        fn always_fail(_: &SimulationSummary) -> anyhow::Result<()> {
            anyhow::bail!("boom");
        }

        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, true);
        let runner = LogicTester::new(tester);
        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_expectation(always_fail);
        let scenario = TestScenario::simulation("fail", plan);
        let results = runner.run_scenario(&scenario, &[1_u64], 1);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert_eq!(results[0].successful_iterations, 0);
        assert!(!results[0].failures.is_empty());
    }

    #[test]
    fn run_scenario_verbose_failure_marks_halted_status() {
        fn always_fail(_: &SimulationSummary) -> anyhow::Result<()> {
            anyhow::bail!("boom");
        }

        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, true);
        let runner = LogicTester::new(tester);
        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_max_days(0)
            .with_expectation(always_fail);
        let scenario = TestScenario::simulation("fail-verbose", plan);

        let results = runner.run_scenario(&scenario, &[42_u64], 1);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(results[0].failures[0].contains("halted"));
    }

    #[test]
    fn run_scenario_verbose_success_records_duration() {
        fn ok_on_seed(summary: &SimulationSummary) -> anyhow::Result<()> {
            anyhow::ensure!(summary.seed == 1, "unexpected seed");
            Ok(())
        }

        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, true);
        let runner = LogicTester::new(tester);
        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_max_days(0)
            .with_expectation(ok_on_seed);
        let scenario = TestScenario::simulation("pass-verbose", plan);

        let results = runner.run_scenario(&scenario, &[1_u64], 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].iterations_run, 1);
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

#[cfg(test)]
mod duration_tests {
    use super::{duration_serde, duration_vec_serde};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct DurationWrapper {
        #[serde(with = "duration_serde")]
        value: Duration,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct DurationVecWrapper {
        #[serde(with = "duration_vec_serde")]
        values: Vec<Duration>,
    }

    #[test]
    fn duration_serde_roundtrip() {
        let wrapper = DurationWrapper {
            value: Duration::from_millis(1500),
        };
        let json = serde_json::to_string(&wrapper).expect("serialize duration");
        let decoded: DurationWrapper = serde_json::from_str(&json).expect("deserialize duration");
        assert_eq!(decoded, wrapper);
    }

    #[test]
    fn duration_vec_serde_roundtrip() {
        let wrapper = DurationVecWrapper {
            values: vec![Duration::from_millis(50), Duration::from_millis(1200)],
        };
        let json = serde_json::to_string(&wrapper).expect("serialize durations");
        let decoded: DurationVecWrapper =
            serde_json::from_str(&json).expect("deserialize durations");
        assert_eq!(decoded, wrapper);
    }
}
