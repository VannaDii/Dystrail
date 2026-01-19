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

    if summary.metrics.days_with_camp == 0 {
        anyhow::ensure!(
            summary.metrics.final_pants >= 3,
            "Aggressive runs should accumulate risk, observed pants {}",
            summary.metrics.final_pants
        );
    }
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
        summary.metrics.final_budget_cents >= 0,
        "Resource Manager should not go into debt, observed {}",
        summary.metrics.final_budget_cents
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::scenario::CombinedScenario;
    use crate::logic::game_tester::{PlayabilityMetrics, SimulationSummary};
    use crate::logic::simulation::TurnOutcome;

    fn base_summary() -> SimulationSummary {
        let mut metrics = PlayabilityMetrics::default();
        metrics.days_survived = 3;
        metrics.ending_type = "Victory".to_string();
        metrics.ending_cause = "None".to_string();
        metrics.final_hp = 5;
        metrics.final_sanity = 5;
        metrics.final_supplies = 5;
        metrics.final_pants = 3;
        metrics.vehicle_breakdowns = 0;
        metrics.encounters_faced = 1;
        metrics.final_budget_cents = 0;

        SimulationSummary {
            seed: 1,
            mode: GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            turns: vec![TurnOutcome {
                day: 1,
                travel_message: "ok".to_string(),
                breakdown_started: false,
                game_ended: false,
                decision: None,
                miles_traveled_actual: 0.0,
            }],
            metrics,
            final_state: dystrail_game::GameState::default(),
            ending_message: "ok".to_string(),
            game_ended: false,
        }
    }

    #[test]
    fn full_game_plan_sets_max_days_and_setup() {
        let plan = full_game_plan(GameMode::Classic, GameplayStrategy::Balanced);
        assert_eq!(plan.max_days, Some(FULL_GAME_MAX_DAYS));
        assert!(plan.setup.is_some());
    }

    #[test]
    fn full_game_expectations_accept_valid_summary() {
        let summary = base_summary();
        full_game_balanced_expectation(&summary).expect("balanced ok");
        full_game_conservative_expectation(&summary).expect("conservative ok");
        full_game_aggressive_expectation(&summary).expect("aggressive ok");
        full_game_resource_manager_expectation(&summary).expect("resource manager ok");
    }

    #[test]
    fn full_game_expectations_reject_empty_turns() {
        let mut summary = base_summary();
        summary.turns.clear();
        let err = full_game_balanced_expectation(&summary).expect_err("should fail");
        assert!(err.to_string().contains("Simulation produced no turns"));
    }

    #[test]
    fn full_game_aggressive_rejects_excess_encounters() {
        let mut summary = base_summary();
        summary.metrics.encounters_faced = 5;
        let err = full_game_aggressive_expectation(&summary).expect_err("should fail");
        assert!(
            err.to_string()
                .contains("Encounter count exceeds turn count")
        );
    }

    #[test]
    fn full_game_scenarios_build_logic_plans() {
        let conservative = full_game_conservative_scenario();
        let conservative_logic = conservative.as_logic_scenario().expect("logic scenario");
        assert!(conservative_logic.name.contains("Conservative"));

        let aggressive = full_game_aggressive_scenario();
        let aggressive_logic = aggressive.as_logic_scenario().expect("logic scenario");
        assert!(aggressive_logic.name.contains("Aggressive"));

        let balanced = full_game_balanced_scenario();
        let balanced_logic = balanced.as_logic_scenario().expect("logic scenario");
        assert!(balanced_logic.name.contains("Balanced"));
    }
}
