use anyhow::{Context, Result};
use std::time::Duration;
use thirtyfour::prelude::*;

use super::{BrowserScenario, CombinedScenario, ScenarioCtx, TestScenario};
use crate::logic::game_tester::SimulationSummary;
use crate::logic::{GameplayStrategy, SimulationPlan};
use dystrail_game::GameMode;

pub struct SmokeScenario;

impl SmokeScenario {
    fn plan() -> SimulationPlan {
        SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_max_days(0)
            .with_expectation(smoke_expectation)
    }
}

#[async_trait::async_trait]
impl BrowserScenario for SmokeScenario {
    async fn run_browser(&self, driver: &WebDriver, ctx: &ScenarioCtx<'_>) -> Result<()> {
        driver.goto(&ctx.base_url).await?;

        let _app_element = driver.find(By::Css("#app, canvas")).await?;

        ctx.bridge.ensure_available().await?;

        let bridge_seed = i64::try_from(ctx.seed).context("seed exceeds browser bridge range")?;
        ctx.bridge.seed(bridge_seed).await?;
        ctx.bridge.speed(4.0).await?;

        if ctx.verbose {
            println!("  🌐 Browser loaded, bridge connected, seed: {}", ctx.seed);
        }

        if let Ok(btn) = driver
            .find(By::Css("button.start, button[data-action='start']"))
            .await
        {
            btn.click().await?;
            if ctx.verbose {
                println!("  🖱️  Clicked start button via DOM");
            }
        } else {
            ctx.bridge.click(120, 180).await?;
            if ctx.verbose {
                println!("  🖱️  Clicked start via bridge coordinates");
            }
        }

        ctx.bridge.screen("travel").await?;
        if ctx.verbose {
            println!("  🧭 Jumped to travel screen via test bridge");
        }

        ctx.bridge.key("wwaassdd").await?;
        if ctx.verbose {
            println!("  ⌨️  Sent keyboard input: wwaassdd");
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        let state = ctx.bridge.state().await?;
        if ctx.verbose {
            println!("  📊 Final state: {state:?}");
        }

        if let Some(hp) = state.hp {
            anyhow::ensure!(hp > 0, "Player HP should be > 0, got {hp}");
        }

        if let Some(day) = state.day {
            anyhow::ensure!(day >= 1, "Game day should be >= 1, got {day}");
        }

        Ok(())
    }
}

impl CombinedScenario for SmokeScenario {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario::simulation("Smoke Test", Self::plan()))
    }
}

fn smoke_expectation(summary: &SimulationSummary) -> Result<()> {
    let state = &summary.final_state;

    anyhow::ensure!(
        state.stats.hp > 0,
        "Initial HP should be > 0, got {}",
        state.stats.hp
    );
    anyhow::ensure!(
        state.stats.supplies >= 0,
        "Initial supplies should be >= 0, got {}",
        state.stats.supplies
    );
    anyhow::ensure!(
        state.stats.sanity >= 0,
        "Initial sanity should be >= 0, got {}",
        state.stats.sanity
    );
    anyhow::ensure!(
        state.day >= 1,
        "Initial day should be >= 1, got {}",
        state.day
    );

    anyhow::ensure!(
        state.stats.hp <= 10,
        "HP should be <= 10, got {}",
        state.stats.hp
    );
    anyhow::ensure!(
        state.stats.supplies <= 20,
        "Supplies should be <= 20, got {}",
        state.stats.supplies
    );
    anyhow::ensure!(
        state.stats.sanity <= 10,
        "Sanity should be <= 10, got {}",
        state.stats.sanity
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::game_tester::{PlayabilityMetrics, SimulationSummary};

    fn base_summary() -> SimulationSummary {
        SimulationSummary {
            seed: 1,
            mode: GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            turns: Vec::new(),
            metrics: PlayabilityMetrics::default(),
            final_state: dystrail_game::GameState::default(),
            ending_message: String::from("ok"),
            game_ended: false,
        }
    }

    #[test]
    fn smoke_scenario_exposes_logic_plan() {
        let scenario = SmokeScenario;
        let logic = scenario.as_logic_scenario().expect("logic scenario");
        assert_eq!(logic.name, "Smoke Test");
    }

    #[test]
    fn smoke_expectation_accepts_default_state() {
        let summary = base_summary();
        smoke_expectation(&summary).expect("smoke ok");
    }

    #[test]
    fn smoke_plan_sets_expectations() {
        let plan = SmokeScenario::plan();
        assert_eq!(plan.mode, GameMode::Classic);
        assert_eq!(plan.strategy, GameplayStrategy::Balanced);
        assert_eq!(plan.max_days, Some(0));
        assert_eq!(plan.expectations.len(), 1);
    }

    #[test]
    fn smoke_expectation_rejects_invalid_stats() {
        let mut summary = base_summary();
        summary.final_state.stats.hp = 0;
        let err = smoke_expectation(&summary).expect_err("hp should fail");
        assert!(err.to_string().contains("HP"));

        let mut summary = base_summary();
        summary.final_state.stats.supplies = -1;
        let err = smoke_expectation(&summary).expect_err("supplies should fail");
        assert!(err.to_string().to_lowercase().contains("supplies"));
    }

    #[test]
    fn smoke_expectation_rejects_out_of_range_stats() {
        let mut summary = base_summary();
        summary.final_state.stats.hp = 11;
        let err = smoke_expectation(&summary).expect_err("hp upper bound should fail");
        assert!(err.to_string().contains("HP should be <= 10"));

        let mut summary = base_summary();
        summary.final_state.stats.sanity = 11;
        let err = smoke_expectation(&summary).expect_err("sanity upper bound should fail");
        assert!(err.to_string().contains("Sanity should be <= 10"));
    }

    #[test]
    fn smoke_expectation_rejects_zero_day() {
        let mut summary = base_summary();
        summary.final_state.day = 0;
        let err = smoke_expectation(&summary).expect_err("day lower bound should fail");
        assert!(err.to_string().contains("day"));
    }
}
