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
