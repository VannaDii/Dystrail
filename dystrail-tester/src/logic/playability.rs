use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::convert::TryFrom;

use crate::common::scenario::full_game::{
    full_game_aggressive_expectation, full_game_balanced_expectation,
    full_game_conservative_expectation, full_game_monte_carlo_expectation, full_game_plan,
    full_game_resource_manager_expectation,
};
use crate::logic::seeds::SeedInfo;
use crate::logic::{GameTester, GameplayStrategy, PlayabilityMetrics};
use dystrail_game::GameMode;

#[derive(Debug, Clone)]
pub struct PlayabilityRecord {
    pub scenario_name: String,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub seed_code: String,
    pub seed_value: u64,
    pub metrics: PlayabilityMetrics,
}

#[derive(Debug, Clone)]
pub struct PlayabilityAggregate {
    pub scenario_name: String,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub iterations: usize,
    pub mean_days: f64,
    pub std_days: f64,
    pub mean_miles: f64,
    pub std_miles: f64,
    pub boss_reach_pct: f64,
    pub boss_win_pct: f64,
    pub pants_failure_pct: f64,
}

const PLAYABILITY_SCENARIOS: &[(GameMode, GameplayStrategy)] = &[
    (GameMode::Classic, GameplayStrategy::Balanced),
    (GameMode::Classic, GameplayStrategy::Conservative),
    (GameMode::Classic, GameplayStrategy::Aggressive),
    (GameMode::Classic, GameplayStrategy::ResourceManager),
    (GameMode::Classic, GameplayStrategy::MonteCarlo),
    (GameMode::Deep, GameplayStrategy::Balanced),
    (GameMode::Deep, GameplayStrategy::Conservative),
    (GameMode::Deep, GameplayStrategy::Aggressive),
    (GameMode::Deep, GameplayStrategy::ResourceManager),
    (GameMode::Deep, GameplayStrategy::MonteCarlo),
];

pub fn run_playability_analysis(
    seeds: &[SeedInfo],
    iterations: usize,
    verbose: bool,
) -> Result<Vec<PlayabilityRecord>> {
    let iterations = iterations.max(1);
    let tester = GameTester::try_new(verbose);
    let mut records = Vec::with_capacity(seeds.len() * PLAYABILITY_SCENARIOS.len() * iterations);

    for &(mode, strategy) in PLAYABILITY_SCENARIOS {
        for seed in seeds.iter().filter(|seed| seed.matches_mode(mode)) {
            for iteration in 0..iterations {
                let iteration_offset = u64::try_from(iteration).unwrap_or(0);
                let iteration_seed = seed.seed.wrapping_add(iteration_offset);
                let plan = add_expectations(full_game_plan(mode, strategy), strategy);
                let summary = tester.run_plan(&plan, iteration_seed);
                for expectation in &plan.expectations {
                    expectation(&summary).with_context(|| {
                        format!(
                            "Playability expectation failed for mode {:?}, strategy {}, seed {} (iteration {})",
                            mode, strategy, seed.seed, iteration + 1
                        )
                    })?;
                }

                let metrics = summary.metrics.clone();
                let scenario_name = format!("{} - {}", mode_label(mode), strategy);
                let seed_code = dystrail_game::encode_friendly(mode.is_deep(), iteration_seed);

                records.push(PlayabilityRecord {
                    scenario_name,
                    mode,
                    strategy,
                    seed_code,
                    seed_value: iteration_seed,
                    metrics,
                });
            }
        }
    }

    Ok(records)
}

pub fn aggregate_playability(records: &[PlayabilityRecord]) -> Vec<PlayabilityAggregate> {
    let mut aggregates: BTreeMap<String, AggregateBuilder> = BTreeMap::new();

    for record in records {
        let entry = aggregates
            .entry(record.scenario_name.clone())
            .or_insert_with(|| AggregateBuilder::new(record));
        entry.ingest(&record.metrics);
    }

    aggregates
        .into_values()
        .map(AggregateBuilder::finish)
        .collect()
}

fn mode_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "Classic",
        GameMode::Deep => "Deep",
    }
}

fn add_expectations(
    plan: crate::logic::SimulationPlan,
    strategy: GameplayStrategy,
) -> crate::logic::SimulationPlan {
    match strategy {
        GameplayStrategy::Balanced => plan.with_expectation(full_game_balanced_expectation),
        GameplayStrategy::Conservative => plan.with_expectation(full_game_conservative_expectation),
        GameplayStrategy::Aggressive => plan.with_expectation(full_game_aggressive_expectation),
        GameplayStrategy::ResourceManager => {
            plan.with_expectation(full_game_resource_manager_expectation)
        }
        GameplayStrategy::MonteCarlo => plan.with_expectation(full_game_monte_carlo_expectation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::seeds::SeedInfo;
    use dystrail_game::GameMode;

    #[test]
    fn generates_records_for_each_scenario() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let records = run_playability_analysis(&seeds, 1, false).unwrap();
        assert_eq!(records.len(), PLAYABILITY_SCENARIOS.len());
        assert!(records.iter().all(|r| !r.seed_code.is_empty()));
    }

    #[test]
    fn balanced_samples_hit_survival_and_boss_targets() {
        let seeds: Vec<SeedInfo> = (0..120)
            .map(|i| {
                let offset = u64::try_from(i).unwrap_or(0);
                SeedInfo::for_mode(10_000_u64 + offset, GameMode::Classic)
            })
            .collect();
        let records = run_playability_analysis(&seeds, 1, false).unwrap();
        let aggregates = aggregate_playability(&records);

        let balanced_summary = aggregates
            .iter()
            .find(|agg| agg.mode == GameMode::Classic && agg.strategy == GameplayStrategy::Balanced)
            .expect("balanced summary available");
        assert!(
            balanced_summary.iterations >= 100,
            "expected at least 100 Balanced samples, observed {}",
            balanced_summary.iterations
        );
        assert!(
            balanced_summary.boss_win_pct >= 0.10 && balanced_summary.boss_win_pct <= 0.25,
            "boss win rate should stay near target band, observed {:.1}%",
            balanced_summary.boss_win_pct * 100.0
        );
        assert!(
            balanced_summary.pants_failure_pct <= 0.20,
            "pants failures should stay below 20%, observed {:.1}%",
            balanced_summary.pants_failure_pct * 100.0
        );

        let balanced_records: Vec<_> = records
            .iter()
            .filter(|r| r.mode == GameMode::Classic && r.strategy == GameplayStrategy::Balanced)
            .collect();
        assert!(
            !balanced_records.is_empty(),
            "balanced sample should not be empty"
        );
        let survive_120 = balanced_records
            .iter()
            .filter(|r| r.metrics.days_survived >= 120)
            .count();
        let numerator = u32::try_from(survive_120).unwrap_or(0);
        let denominator = u32::try_from(balanced_records.len()).unwrap_or(1);
        let ratio = if denominator == 0 {
            0.0
        } else {
            f64::from(numerator) / f64::from(denominator)
        };
        assert!(
            ratio >= 0.35,
            "expected ≥35% of balanced runs to reach 120 days, observed {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn aggregates_match_record_counts() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let iterations = 3;
        let records = run_playability_analysis(&seeds, iterations, false).unwrap();
        let aggregates = aggregate_playability(&records);

        let scenario_name = "Classic - Balanced";
        let agg = aggregates
            .iter()
            .find(|a| a.scenario_name == scenario_name)
            .expect("aggregate for Classic Balanced");

        let matching: Vec<_> = records
            .iter()
            .filter(|r| r.scenario_name == scenario_name)
            .collect();
        assert_eq!(agg.iterations, matching.len());

        let boss_reached: u32 = matching
            .iter()
            .filter(|r| r.metrics.boss_reached)
            .count()
            .try_into()
            .unwrap();
        let boss_won: u32 = matching
            .iter()
            .filter(|r| r.metrics.boss_won)
            .count()
            .try_into()
            .unwrap();

        let iterations_u32: u32 = matching.len().try_into().unwrap();
        let denom = if iterations_u32 == 0 {
            1.0
        } else {
            f64::from(iterations_u32)
        };
        let reach_ratio = f64::from(boss_reached) / denom;
        let win_ratio = f64::from(boss_won) / denom;

        let epsilon = 1e-6;
        assert!((agg.boss_reach_pct - reach_ratio).abs() < epsilon);
        assert!((agg.boss_win_pct - win_ratio).abs() < epsilon);
    }
}

#[derive(Debug, Clone)]
struct AggregateBuilder {
    scenario_name: String,
    mode: GameMode,
    strategy: GameplayStrategy,
    stats_days: RunningStats,
    stats_miles: RunningStats,
    iterations: u32,
    boss_reached: u32,
    boss_won: u32,
    pants_failures: u32,
}

impl AggregateBuilder {
    fn new(record: &PlayabilityRecord) -> Self {
        Self {
            scenario_name: record.scenario_name.clone(),
            mode: record.mode,
            strategy: record.strategy,
            stats_days: RunningStats::default(),
            stats_miles: RunningStats::default(),
            iterations: 0,
            boss_reached: 0,
            boss_won: 0,
            pants_failures: 0,
        }
    }

    fn ingest(&mut self, metrics: &PlayabilityMetrics) {
        self.iterations += 1;
        self.stats_days.add(f64::from(metrics.days_survived));
        self.stats_miles.add(f64::from(metrics.miles_traveled));
        if metrics.boss_reached {
            self.boss_reached += 1;
        }
        if metrics.boss_won {
            self.boss_won += 1;
        }
        if metrics.final_pants >= 100 || metrics.ending_type.contains("Pants") {
            self.pants_failures += 1;
        }
    }

    fn finish(self) -> PlayabilityAggregate {
        let iterations_u32 = self.iterations.max(1);
        let iterations = usize::try_from(self.iterations).unwrap_or(usize::MAX);
        let denom = f64::from(iterations_u32);
        PlayabilityAggregate {
            scenario_name: self.scenario_name,
            mode: self.mode,
            strategy: self.strategy,
            iterations,
            mean_days: self.stats_days.mean(),
            std_days: self.stats_days.std_dev(),
            mean_miles: self.stats_miles.mean(),
            std_miles: self.stats_miles.std_dev(),
            boss_reach_pct: f64::from(self.boss_reached) / denom,
            boss_win_pct: f64::from(self.boss_won) / denom,
            pants_failure_pct: f64::from(self.pants_failures) / denom,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct RunningStats {
    count: u32,
    mean: f64,
    m2: f64,
}

impl RunningStats {
    fn add(&mut self, value: f64) {
        self.count += 1;
        let count = f64::from(self.count);
        let delta = value - self.mean;
        self.mean += delta / count;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    fn mean(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.mean }
    }

    fn variance(&self) -> f64 {
        if self.count > 1 {
            self.m2 / f64::from(self.count - 1)
        } else {
            0.0
        }
    }

    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}
