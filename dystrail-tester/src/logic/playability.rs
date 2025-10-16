use anyhow::{Context, Result, ensure};
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
    pub mean_travel_ratio: f64,
    pub mean_unique_per_20: f64,
    pub mean_rotation_events: f64,
    pub pct_reached_2k_by_150: f64,
    pub min_unique_per_20: f64,
    pub min_travel_ratio: f64,
    pub mean_crossing_events: f64,
    pub crossing_permit_rate: f64,
    pub mean_crossing_bribes: f64,
    pub crossing_bribe_success_rate: f64,
    pub mean_crossing_detours: f64,
    pub crossing_failure_rate: f64,
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
    let mut warn_counts: BTreeMap<String, usize> = BTreeMap::new();

    for record in records {
        let entry = aggregates
            .entry(record.scenario_name.clone())
            .or_insert_with(|| AggregateBuilder::new(record));
        entry.ingest(&record.metrics);

        if record.scenario_name == "Classic - Balanced" {
            if record.metrics.travel_ratio < 0.85 {
                let counter = warn_counts
                    .entry(format!("{}::travel_ratio", record.scenario_name))
                    .or_insert(0);
                if *counter < 3 {
                    println!(
                        "WARN: Classic Balanced seed {} travel ratio {:.1}%",
                        record.seed_code,
                        record.metrics.travel_ratio * 100.0
                    );
                }
                *counter += 1;
            }
            if record.metrics.unique_per_20_days < 1.45 {
                let counter = warn_counts
                    .entry(format!("{}::unique_per_20", record.scenario_name))
                    .or_insert(0);
                if *counter < 3 {
                    println!(
                        "WARN: Classic Balanced seed {} unique encounters per 20d {:.2} < 1.5",
                        record.seed_code, record.metrics.unique_per_20_days
                    );
                }
                *counter += 1;
            }
        }
        if record.scenario_name == "Deep - Conservative" {
            if !record.metrics.reached_2000_by_day150 {
                let counter = warn_counts
                    .entry(format!("{}::deep_conservative_2k", record.scenario_name))
                    .or_insert(0);
                if *counter < 5 {
                    println!(
                        "WARN: Deep Conservative seed {} failed ≥2k@150",
                        record.seed_code
                    );
                }
                *counter += 1;
            }
            if record.metrics.final_pants >= 100 {
                let counter = warn_counts
                    .entry(format!("{}::deep_conservative_pants", record.scenario_name))
                    .or_insert(0);
                if *counter < 3 {
                    println!(
                        "WARN: Deep Conservative seed {} ended via pants emergency",
                        record.seed_code
                    );
                }
                *counter += 1;
            }
        }
        if record.scenario_name == "Deep - Aggressive" {
            if !record.metrics.boss_reached {
                let counter = warn_counts
                    .entry(format!("{}::deep_aggressive_reach", record.scenario_name))
                    .or_insert(0);
                if *counter < 5 {
                    println!(
                        "WARN: Deep Aggressive seed {} failed to reach the boss",
                        record.seed_code
                    );
                }
                *counter += 1;
            } else if !record.metrics.boss_won {
                let counter = warn_counts
                    .entry(format!("{}::deep_aggressive_win", record.scenario_name))
                    .or_insert(0);
                if *counter < 5 {
                    println!(
                        "WARN: Deep Aggressive seed {} reached boss but did not win",
                        record.seed_code
                    );
                }
                *counter += 1;
            }
        }
    }

    aggregates
        .into_values()
        .map(AggregateBuilder::finish)
        .collect()
}

pub fn validate_playability_targets(aggregates: &[PlayabilityAggregate]) -> Result<()> {
    let find = |name: &str| -> Result<&PlayabilityAggregate> {
        aggregates
            .iter()
            .find(|agg| agg.scenario_name == name)
            .with_context(|| format!("missing playability summary for {name}"))
    };

    let classic_balanced = find("Classic - Balanced")?;
    ensure!(
        classic_balanced.min_unique_per_20 >= 1.5,
        "Classic Balanced min unique encounters per 20 days {:.2} below 1.5 requirement",
        classic_balanced.min_unique_per_20
    );
    ensure!(
        classic_balanced.mean_unique_per_20 >= 1.5,
        "Classic Balanced mean unique encounters per 20 days {:.2} below 1.5 target",
        classic_balanced.mean_unique_per_20
    );
    ensure!(
        classic_balanced.pct_reached_2k_by_150 >= 0.25,
        "Classic Balanced reached 2,000 miles by day 150 {:.1}% < 25% threshold",
        classic_balanced.pct_reached_2k_by_150 * 100.0
    );

    let classic_resource = find("Classic - Resource Manager")?;
    ensure!(
        classic_resource.pct_reached_2k_by_150 >= 0.70,
        "Classic Resource Manager reached 2,000 miles by day 150 {:.1}% < 70% threshold",
        classic_resource.pct_reached_2k_by_150 * 100.0
    );
    ensure!(
        classic_resource.pants_failure_pct <= 0.35,
        "Classic Resource Manager pants failure rate {:.1}% exceeds 35% cap",
        classic_resource.pants_failure_pct * 100.0
    );

    let deep_balanced = find("Deep - Balanced")?;
    ensure!(
        deep_balanced.mean_unique_per_20 >= 1.5,
        "Deep Balanced mean unique encounters per 20 days {:.2} below 1.5 target",
        deep_balanced.mean_unique_per_20
    );
    ensure!(
        deep_balanced.min_unique_per_20 >= 1.5,
        "Deep Balanced min unique encounters per 20 days {:.2} below 1.5 requirement",
        deep_balanced.min_unique_per_20
    );
    ensure!(
        deep_balanced.mean_travel_ratio >= 0.92,
        "Deep Balanced travel ratio {:.1}% below 92% target",
        deep_balanced.mean_travel_ratio * 100.0
    );
    ensure!(
        deep_balanced.mean_miles >= 2000.0,
        "Deep Balanced average mileage {:.0} below 2000 mi goal",
        deep_balanced.mean_miles
    );
    ensure!(
        deep_balanced.pct_reached_2k_by_150 >= 0.25,
        "Deep Balanced reached 2,000 miles by day 150 {:.1}% < 25% threshold",
        deep_balanced.pct_reached_2k_by_150 * 100.0
    );

    let deep_conservative = find("Deep - Conservative")?;
    ensure!(
        deep_conservative.pct_reached_2k_by_150 >= 0.35,
        "Deep Conservative ≥2k@150 {:.1}% < 35% threshold",
        deep_conservative.pct_reached_2k_by_150 * 100.0
    );
    ensure!(
        deep_conservative.pants_failure_pct <= 0.30,
        "Deep Conservative pants failure rate {:.1}% exceeds 30% cap",
        deep_conservative.pants_failure_pct * 100.0
    );
    ensure!(
        deep_conservative.mean_travel_ratio >= 0.90,
        "Deep Conservative travel ratio {:.1}% below 90% target",
        deep_conservative.mean_travel_ratio * 100.0
    );

    let deep_aggressive = find("Deep - Aggressive")?;
    ensure!(
        deep_aggressive.boss_reach_pct >= 0.65,
        "Deep Aggressive boss reach {:.1}% below 65% target",
        deep_aggressive.boss_reach_pct * 100.0
    );
    ensure!(
        deep_aggressive.boss_win_pct >= 0.02,
        "Deep Aggressive boss win {:.1}% below 2% target",
        deep_aggressive.boss_win_pct * 100.0
    );
    ensure!(
        deep_aggressive.mean_miles >= 1980.0,
        "Deep Aggressive average mileage {:.0} below 1980 mi goal",
        deep_aggressive.mean_miles
    );
    ensure!(
        deep_aggressive.pct_reached_2k_by_150 >= 0.70,
        "Deep Aggressive ≥2k@150 {:.1}% < 70% threshold",
        deep_aggressive.pct_reached_2k_by_150 * 100.0
    );

    Ok(())
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
    use dystrail_game::data::EncounterData;
    use std::collections::HashSet;
    use std::path::PathBuf;

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
            balanced_summary.boss_win_pct >= 0.0 && balanced_summary.boss_win_pct <= 0.25,
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
            ratio <= 0.45,
            "expected ≤45% of balanced runs to reach 120 days under new fail states, observed {:.1}%",
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

    #[test]
    fn encounter_catalog_contains_rotation_additions() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("dystrail-web")
            .join("static")
            .join("assets")
            .join("data")
            .join("game.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let data = EncounterData::from_json(&json).expect("parse game.json");
        let ids: HashSet<_> = data.encounters.iter().map(|enc| enc.id.as_str()).collect();
        for expected in ["classic_civic_potluck", "deep_watchdog_sync"] {
            assert!(
                ids.contains(expected),
                "expected encounter id {expected} to be present in game.json"
            );
        }
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
    travel_ratio_sum: f64,
    unique_per_20_sum: f64,
    rotation_event_sum: u32,
    milestone_hits: u32,
    min_unique_per_20: f64,
    min_travel_ratio: f64,
    crossing_events_sum: u32,
    crossing_permit_sum: u32,
    crossing_bribe_attempt_sum: u32,
    crossing_bribe_success_sum: u32,
    crossing_detour_sum: u32,
    crossing_failure_sum: u32,
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
            travel_ratio_sum: 0.0,
            unique_per_20_sum: 0.0,
            rotation_event_sum: 0,
            milestone_hits: 0,
            min_unique_per_20: f64::INFINITY,
            min_travel_ratio: f64::INFINITY,
            crossing_events_sum: 0,
            crossing_permit_sum: 0,
            crossing_bribe_attempt_sum: 0,
            crossing_bribe_success_sum: 0,
            crossing_detour_sum: 0,
            crossing_failure_sum: 0,
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
        if metrics.reached_2000_by_day150 {
            self.milestone_hits += 1;
        }
        self.travel_ratio_sum += metrics.travel_ratio;
        self.unique_per_20_sum += metrics.unique_per_20_days;
        self.rotation_event_sum = self
            .rotation_event_sum
            .saturating_add(metrics.rotation_events);
        self.min_unique_per_20 = self.min_unique_per_20.min(metrics.unique_per_20_days);
        self.min_travel_ratio = self.min_travel_ratio.min(metrics.travel_ratio);
        self.crossing_events_sum = self
            .crossing_events_sum
            .saturating_add(u32::try_from(metrics.crossing_events.len()).unwrap_or(0));
        self.crossing_permit_sum = self
            .crossing_permit_sum
            .saturating_add(metrics.crossing_permit_uses);
        self.crossing_bribe_attempt_sum = self
            .crossing_bribe_attempt_sum
            .saturating_add(metrics.crossing_bribe_attempts);
        self.crossing_bribe_success_sum = self
            .crossing_bribe_success_sum
            .saturating_add(metrics.crossing_bribe_successes);
        self.crossing_detour_sum = self
            .crossing_detour_sum
            .saturating_add(metrics.crossing_detours_taken);
        self.crossing_failure_sum = self
            .crossing_failure_sum
            .saturating_add(metrics.crossing_failures);
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
            mean_travel_ratio: self.travel_ratio_sum / denom,
            mean_unique_per_20: self.unique_per_20_sum / denom,
            mean_rotation_events: f64::from(self.rotation_event_sum) / denom,
            pct_reached_2k_by_150: f64::from(self.milestone_hits) / denom,
            min_unique_per_20: if self.min_unique_per_20.is_finite() {
                self.min_unique_per_20
            } else {
                0.0
            },
            min_travel_ratio: if self.min_travel_ratio.is_finite() {
                self.min_travel_ratio
            } else {
                0.0
            },
            mean_crossing_events: f64::from(self.crossing_events_sum) / denom,
            crossing_permit_rate: if self.crossing_events_sum == 0 {
                0.0
            } else {
                f64::from(self.crossing_permit_sum) / f64::from(self.crossing_events_sum)
            },
            mean_crossing_bribes: f64::from(self.crossing_bribe_attempt_sum) / denom,
            crossing_bribe_success_rate: if self.crossing_bribe_attempt_sum == 0 {
                0.0
            } else {
                f64::from(self.crossing_bribe_success_sum)
                    / f64::from(self.crossing_bribe_attempt_sum)
            },
            mean_crossing_detours: f64::from(self.crossing_detour_sum) / denom,
            crossing_failure_rate: if self.crossing_events_sum == 0 {
                0.0
            } else {
                f64::from(self.crossing_failure_sum) / f64::from(self.crossing_events_sum)
            },
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
