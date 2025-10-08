use anyhow::Result;
use std::time::Duration;
use thirtyfour::prelude::*;

use super::{BrowserScenario, CombinedScenario, ScenarioCtx, TestScenario};

pub struct Smoke;

#[async_trait::async_trait]
impl BrowserScenario for Smoke {
    async fn run_browser(&self, driver: &WebDriver, ctx: &ScenarioCtx<'_>) -> Result<()> {
        // Navigate to the game
        driver.goto(&ctx.base_url).await?;

        // Wait for app root (DOM or canvas container)
        let _app_element = driver.find(By::Css("#app, canvas")).await?;

        // Ensure test bridge is available
        ctx.bridge.ensure_available().await?;

        // Set deterministic seed and fast sim
        ctx.bridge.seed(ctx.seed).await?;
        ctx.bridge.speed(4.0).await?;

        if ctx.verbose {
            println!("  🌐 Browser loaded, bridge connected, seed: {}", ctx.seed);
        }

        // Try DOM button; fallback to canvas bridge click
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

        // Drive some keyboard input via bridge
        ctx.bridge.key("wwaassdd").await?;
        if ctx.verbose {
            println!("  ⌨️  Sent keyboard input: wwaassdd");
        }

        // Wait for game state transition
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check that we're in some kind of gameplay state
        let state = ctx.bridge.state().await?;
        if ctx.verbose {
            println!("  📊 Final state: {state:?}");
        }

        // Basic assertions - game should be running
        if let Some(hp) = state.hp {
            anyhow::ensure!(hp > 0, "Player HP should be > 0, got {hp}");
        }

        if let Some(day) = state.day {
            anyhow::ensure!(day >= 1, "Game day should be >= 1, got {day}");
        }

        Ok(())
    }
}

impl CombinedScenario for Smoke {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Smoke Test".to_string(),
            setup: None,
            test_fn: |game_state| {
                // Verify initial state is reasonable
                anyhow::ensure!(
                    game_state.stats.hp > 0,
                    "Initial HP should be > 0, got {}",
                    game_state.stats.hp
                );
                anyhow::ensure!(
                    game_state.stats.supplies >= 0,
                    "Initial supplies should be >= 0, got {}",
                    game_state.stats.supplies
                );
                anyhow::ensure!(
                    game_state.stats.sanity >= 0,
                    "Initial sanity should be >= 0, got {}",
                    game_state.stats.sanity
                );
                anyhow::ensure!(
                    game_state.day >= 1,
                    "Initial day should be >= 1, got {}",
                    game_state.day
                );

                // Verify stats are within reasonable bounds
                anyhow::ensure!(
                    game_state.stats.hp <= 10,
                    "HP should be <= 10, got {}",
                    game_state.stats.hp
                );
                anyhow::ensure!(
                    game_state.stats.supplies <= 20,
                    "Supplies should be <= 20, got {}",
                    game_state.stats.supplies
                );
                anyhow::ensure!(
                    game_state.stats.sanity <= 10,
                    "Sanity should be <= 10, got {}",
                    game_state.stats.sanity
                );

                // Test that we can advance one day without crashing
                // Note: This is a minimal test since we don't have advance_day method
                // We just verify the state is consistent

                Ok(())
            },
        })
    }
}
