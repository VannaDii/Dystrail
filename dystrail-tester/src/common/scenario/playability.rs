use anyhow::Result;

use super::SimulationScenario;
use crate::logic::game_tester::{GameTester, SimulationSummary};
use crate::logic::{GameplayStrategy, SimulationPlan};
use dystrail_game::GameMode;

pub fn resource_stress_scenario() -> SimulationScenario {
    SimulationScenario::new(
        "Resource Management Stress Test",
        stress_plan()
            .with_setup(resource_stress_setup)
            .with_expectation(resource_stress_expectation),
    )
}

pub fn deterministic_verification_scenario(tester: GameTester) -> SimulationScenario {
    SimulationScenario::new(
        "Deterministic Playthrough Verification",
        deterministic_plan(deterministic_verification_expectation(tester)),
    )
}

pub fn edge_case_survival_scenario() -> SimulationScenario {
    SimulationScenario::new(
        "Edge Case Survival Test",
        stress_plan()
            .with_setup(edge_case_survival_setup)
            .with_expectation(edge_case_survival_expectation),
    )
}

fn stress_plan() -> SimulationPlan {
    SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced).with_max_days(40)
}

fn deterministic_plan(
    expectation: impl Into<crate::logic::SimulationExpectation>,
) -> SimulationPlan {
    SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
        .with_max_days(20)
        .with_expectation(expectation)
}

const fn resource_stress_setup(game_state: &mut dystrail_game::GameState) {
    game_state.stats.supplies = 2;
    game_state.stats.hp = 3;
    game_state.stats.sanity = 4;
    game_state.budget_cents = 500;
}

fn resource_stress_expectation(summary: &SimulationSummary) -> Result<()> {
    anyhow::ensure!(
        !summary.turns.is_empty(),
        "Resource stress simulation produced no turns"
    );

    let metrics = &summary.metrics;
    anyhow::ensure!(
        metrics.days_survived < 40,
        "Stress scenario should end before the max day limit"
    );
    anyhow::ensure!(
        metrics.final_supplies <= 0 || metrics.final_sanity <= 0 || metrics.final_pants >= 100,
        "Stress scenario should fail via resource depletion; observed stats supplies={supplies}, sanity={sanity}, pants={pants}",
        supplies = metrics.final_supplies,
        sanity = metrics.final_sanity,
        pants = metrics.final_pants
    );
    Ok(())
}

fn deterministic_verification_expectation(
    tester: GameTester,
) -> crate::logic::SimulationExpectation {
    crate::logic::SimulationExpectation::new(move |summary: &SimulationSummary| {
        let comparison_plan =
            deterministic_plan(crate::logic::SimulationExpectation::new(|_| Ok(())));
        let comparison = tester.run_plan(&comparison_plan, summary.seed);

        anyhow::ensure!(
            summary.turns.len() == comparison.turns.len(),
            "Deterministic runs should have identical turn counts"
        );
        anyhow::ensure!(
            summary.metrics.final_hp == comparison.metrics.final_hp
                && summary.metrics.final_supplies == comparison.metrics.final_supplies
                && summary.metrics.final_sanity == comparison.metrics.final_sanity
                && summary.metrics.final_pants == comparison.metrics.final_pants,
            "Deterministic runs diverged: original {metrics:?}, comparison {comparison:?}",
            metrics = summary.metrics,
            comparison = comparison.metrics
        );
        Ok(())
    })
}

const fn edge_case_survival_setup(game_state: &mut dystrail_game::GameState) {
    game_state.stats.pants = 95;
    game_state.stats.sanity = 1;
    game_state.stats.supplies = 1;
    game_state.stats.hp = 1;
    game_state.budget_cents = 50;
}

fn edge_case_survival_expectation(summary: &SimulationSummary) -> Result<()> {
    anyhow::ensure!(
        !summary.turns.is_empty(),
        "Edge case scenario produced no turns"
    );

    let metrics = &summary.metrics;
    anyhow::ensure!(
        metrics.days_survived < 40,
        "Edge-case scenario should fail quickly"
    );
    anyhow::ensure!(
        metrics.final_pants >= 100
            || metrics.final_hp <= 0
            || metrics.final_sanity <= 0
            || metrics.final_supplies <= 0,
        "Edge-case run should trigger a failure condition; observed metrics {metrics:?}"
    );
    Ok(())
}
