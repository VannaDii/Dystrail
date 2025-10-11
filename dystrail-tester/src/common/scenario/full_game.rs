use anyhow::{Context, Result};

use super::SimulationScenario;
use crate::logic::game_tester::SimulationSummary;
use crate::logic::{GameplayStrategy, SimulationPlan, default_policy_setup};
use dystrail_game::GameMode;

const FULL_GAME_MAX_DAYS: u32 = 200;

pub fn full_game_conservative_scenario() -> SimulationScenario {
    SimulationScenario::new(
        "Full Game - Conservative Strategy",
        full_game_plan(GameMode::Classic, GameplayStrategy::Conservative)
            .with_expectation(full_game_conservative_expectation),
    )
}

pub fn full_game_aggressive_scenario() -> SimulationScenario {
    SimulationScenario::new(
        "Full Game - Aggressive Strategy",
        full_game_plan(GameMode::Classic, GameplayStrategy::Aggressive)
            .with_expectation(full_game_aggressive_expectation),
    )
}

pub fn full_game_balanced_scenario() -> SimulationScenario {
    SimulationScenario::new(
        "Full Game - Balanced Strategy",
        full_game_plan(GameMode::Classic, GameplayStrategy::Balanced)
            .with_expectation(full_game_balanced_expectation),
    )
}

pub fn full_game_plan(mode: GameMode, strategy: GameplayStrategy) -> SimulationPlan {
    SimulationPlan::new(mode, strategy)
        .with_max_days(FULL_GAME_MAX_DAYS)
        .with_setup(default_policy_setup(strategy))
}

fn ensure_basic_progress(summary: &SimulationSummary, min_days: u32) -> Result<()> {
    let observed_turns = summary.turns.len();
    anyhow::ensure!(observed_turns > 0, "Simulation produced no turns");

    let observed_days: u32 = summary
        .metrics
        .days_survived
        .try_into()
        .context("days_survived overflowed u32")?;
    anyhow::ensure!(
        observed_days >= min_days,
        "Expected at least {min_days} days of survival, observed {observed_days}"
    );

    anyhow::ensure!(
        !summary.metrics.ending_type.is_empty(),
        "Ending type should not be empty"
    );
    anyhow::ensure!(
        !summary.metrics.ending_type.contains("Unknown"),
        "Unexpected ending type: {}",
        summary.metrics.ending_type
    );
    anyhow::ensure!(
        summary.metrics.final_hp >= 0 && summary.metrics.final_sanity >= 0,
        "Final stats should remain non-negative: HP={}, Sanity={}",
        summary.metrics.final_hp,
        summary.metrics.final_sanity
    );

    Ok(())
}

pub fn full_game_conservative_expectation(summary: &SimulationSummary) -> Result<()> {
    ensure_basic_progress(summary, 2)?;

    anyhow::ensure!(
        summary.metrics.final_pants <= 110,
        "Conservative run should keep pants under control, observed {}",
        summary.metrics.final_pants
    );
    anyhow::ensure!(
        summary.metrics.vehicle_breakdowns >= 0,
        "Vehicle breakdown count should be non-negative, observed {}",
        summary.metrics.vehicle_breakdowns
    );
    Ok(())
}

pub fn full_game_aggressive_expectation(summary: &SimulationSummary) -> Result<()> {
    ensure_basic_progress(summary, 2)?;

    anyhow::ensure!(
        summary.metrics.final_pants >= 5,
        "Aggressive runs should accumulate risk, observed pants {}",
        summary.metrics.final_pants
    );
    let encounters = usize::try_from(summary.metrics.encounters_faced)
        .context("encounters_faced should be non-negative")?;
    anyhow::ensure!(
        encounters <= summary.turns.len(),
        "Encounter count exceeds turn count"
    );
    Ok(())
}

pub fn full_game_balanced_expectation(summary: &SimulationSummary) -> Result<()> {
    ensure_basic_progress(summary, 1)?;

    anyhow::ensure!(
        summary.metrics.final_supplies >= -5,
        "Balanced run should avoid catastrophic supply loss, observed {}",
        summary.metrics.final_supplies
    );
    anyhow::ensure!(
        summary.metrics.final_sanity >= -2,
        "Balanced run should preserve sanity, observed {}",
        summary.metrics.final_sanity
    );
    Ok(())
}

pub fn full_game_resource_manager_expectation(summary: &SimulationSummary) -> Result<()> {
    ensure_basic_progress(summary, 1)?;

    anyhow::ensure!(
        summary.metrics.final_supplies >= 0,
        "Resource Manager should not end in supply debt, observed {}",
        summary.metrics.final_supplies
    );
    anyhow::ensure!(
        summary.metrics.final_budget_cents >= 12_000,
        "Resource Manager should preserve budget, observed {}",
        summary.metrics.final_budget_cents
    );
    Ok(())
}

pub fn full_game_monte_carlo_expectation(summary: &SimulationSummary) -> Result<()> {
    ensure_basic_progress(summary, 1)?;

    anyhow::ensure!(
        summary.metrics.encounters_faced >= 1,
        "Monte Carlo runs should engage with encounters, observed {}",
        summary.metrics.encounters_faced
    );
    anyhow::ensure!(
        summary.metrics.final_pants >= 0,
        "Monte Carlo pants should be non-negative, observed {}",
        summary.metrics.final_pants
    );
    Ok(())
}
