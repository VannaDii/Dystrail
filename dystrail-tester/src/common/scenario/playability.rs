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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::scenario::CombinedScenario;
    use crate::logic::TesterAssets;
    use crate::logic::game_tester::PlayabilityMetrics;
    use crate::logic::simulation::TurnOutcome;
    use std::sync::Arc;

    fn summary_with(metrics: PlayabilityMetrics) -> SimulationSummary {
        SimulationSummary {
            seed: 7,
            mode: GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            turns: vec![TurnOutcome {
                day: 1,
                travel_message: String::from("ok"),
                breakdown_started: false,
                game_ended: false,
                decision: None,
                miles_traveled_actual: 0.0,
            }],
            metrics,
            final_state: dystrail_game::GameState::default(),
            ending_message: String::from("ok"),
            game_ended: false,
        }
    }

    #[test]
    fn resource_stress_setup_sets_low_resources() {
        let mut state = dystrail_game::GameState::default();
        resource_stress_setup(&mut state);
        assert_eq!(state.stats.supplies, 2);
        assert_eq!(state.stats.hp, 3);
        assert_eq!(state.stats.sanity, 4);
        assert_eq!(state.budget_cents, 500);
    }

    #[test]
    fn edge_case_survival_setup_sets_fragile_stats() {
        let mut state = dystrail_game::GameState::default();
        edge_case_survival_setup(&mut state);
        assert_eq!(state.stats.pants, 95);
        assert_eq!(state.stats.sanity, 1);
        assert_eq!(state.stats.supplies, 1);
        assert_eq!(state.stats.hp, 1);
        assert_eq!(state.budget_cents, 50);
    }

    #[test]
    fn resource_stress_expectation_accepts_depleted_metrics() {
        let mut metrics = PlayabilityMetrics::default();
        metrics.days_survived = 10;
        metrics.final_supplies = 0;
        let summary = summary_with(metrics);
        resource_stress_expectation(&summary).expect("stress expectation ok");
    }

    #[test]
    fn resource_stress_expectation_rejects_empty_turns() {
        let metrics = PlayabilityMetrics::default();
        let mut summary = summary_with(metrics);
        summary.turns.clear();
        let err = resource_stress_expectation(&summary).expect_err("turns should fail");
        assert!(err.to_string().contains("no turns"));
    }

    #[test]
    fn edge_case_survival_expectation_accepts_failure_metrics() {
        let mut metrics = PlayabilityMetrics::default();
        metrics.days_survived = 5;
        metrics.final_hp = 0;
        let summary = summary_with(metrics);
        edge_case_survival_expectation(&summary).expect("edge expectation ok");
    }

    #[test]
    fn edge_case_survival_expectation_rejects_non_failure() {
        let mut metrics = PlayabilityMetrics::default();
        metrics.days_survived = 5;
        metrics.final_hp = 5;
        metrics.final_supplies = 5;
        metrics.final_sanity = 5;
        metrics.final_pants = 10;
        let summary = summary_with(metrics);
        let err = edge_case_survival_expectation(&summary).expect_err("should fail");
        assert!(err.to_string().contains("failure"));
    }

    #[test]
    fn deterministic_verification_expectation_compares_runs() {
        let assets = Arc::new(TesterAssets::load_default());
        let scenario = deterministic_verification_scenario(
            crate::logic::game_tester::GameTester::new(assets.clone(), false),
        );
        let runner = crate::logic::game_tester::GameTester::new(assets, false);
        let summary = runner.run_plan(&scenario.plan, 123);
        for expectation in scenario.plan.expectations {
            expectation.evaluate(&summary).expect("determinism ok");
        }
    }

    #[test]
    fn resource_stress_scenario_builds_logic_plan() {
        let scenario = resource_stress_scenario();
        let logic = scenario.as_logic_scenario().expect("logic scenario");
        assert!(logic.name.contains("Resource Management Stress"));
        assert_eq!(logic.plan.max_days, Some(40));
        assert!(logic.plan.setup.is_some());
    }

    #[test]
    fn edge_case_survival_scenario_builds_logic_plan() {
        let scenario = edge_case_survival_scenario();
        let logic = scenario.as_logic_scenario().expect("logic scenario");
        assert!(logic.name.contains("Edge Case Survival"));
        assert_eq!(logic.plan.max_days, Some(40));
        assert!(logic.plan.setup.is_some());
    }

    #[test]
    fn deterministic_verification_scenario_builds_logic_plan() {
        let assets = Arc::new(TesterAssets::load_default());
        let scenario = deterministic_verification_scenario(
            crate::logic::game_tester::GameTester::new(assets, false),
        );
        let logic = scenario.as_logic_scenario().expect("logic scenario");
        assert_eq!(logic.plan.max_days, Some(20));
        assert_eq!(logic.plan.expectations.len(), 1);
    }
}
