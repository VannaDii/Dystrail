use std::collections::HashMap;

use anyhow::{Context, Result};

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
    verbose: bool,
) -> Result<Vec<PlayabilityRecord>> {
    let tester = GameTester::try_new(verbose);
    let mut records = Vec::with_capacity(seeds.len() * PLAYABILITY_SCENARIOS.len());
    let mut sanity_check: HashMap<(GameMode, u64), HashMap<GameplayStrategy, PlayabilityMetrics>> =
        HashMap::new();

    for &(mode, strategy) in PLAYABILITY_SCENARIOS {
        for seed in seeds.iter().filter(|seed| seed.matches_mode(mode)) {
            let plan = add_expectations(full_game_plan(mode, strategy), strategy);
            // Ensure expectations run by evaluating them after simulation
            let summary = tester.run_plan(&plan, seed.seed);
            for expectation in &plan.expectations {
                expectation(&summary).with_context(|| {
                    format!(
                        "Playability expectation failed for mode {:?}, strategy {}, seed {}",
                        mode, strategy, seed.seed
                    )
                })?;
            }

            let metrics = summary.metrics.clone();
            let scenario_name = format!("{} - {}", mode_label(mode), strategy);
            let seed_code = seed.share_code_for_mode(mode);

            records.push(PlayabilityRecord {
                scenario_name,
                mode,
                strategy,
                seed_code,
                seed_value: seed.seed,
                metrics,
            });

            sanity_check
                .entry((mode, seed.seed))
                .or_default()
                .insert(strategy, summary.metrics);
        }
    }

    enforce_cross_strategy_expectations(&sanity_check)?;

    Ok(records)
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

fn enforce_cross_strategy_expectations(
    sanity_check: &HashMap<(GameMode, u64), HashMap<GameplayStrategy, PlayabilityMetrics>>,
) -> Result<()> {
    for ((mode, seed), metrics_by_strategy) in sanity_check {
        if let (Some(balanced), Some(aggressive)) = (
            metrics_by_strategy.get(&GameplayStrategy::Balanced),
            metrics_by_strategy.get(&GameplayStrategy::Aggressive),
        ) {
            anyhow::ensure!(
                balanced.final_sanity >= aggressive.final_sanity,
                "Balanced strategy should retain sanity better than Aggressive for mode {:?} seed {} (balanced: {}, aggressive: {})",
                mode,
                seed,
                balanced.final_sanity,
                aggressive.final_sanity
            );
        }

        if let (Some(resource), Some(balanced)) = (
            metrics_by_strategy.get(&GameplayStrategy::ResourceManager),
            metrics_by_strategy.get(&GameplayStrategy::Balanced),
        ) {
            anyhow::ensure!(
                resource.final_supplies >= balanced.final_supplies,
                "Resource Manager should not end with fewer supplies than Balanced for mode {:?} seed {} (resource: {}, balanced: {})",
                mode,
                seed,
                resource.final_supplies,
                balanced.final_supplies
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::seeds::SeedInfo;

    #[test]
    fn generates_records_for_each_scenario() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let records = run_playability_analysis(&seeds, false).unwrap();
        assert_eq!(records.len(), PLAYABILITY_SCENARIOS.len());
        assert!(records.iter().all(|r| !r.seed_code.is_empty()));
    }
}
