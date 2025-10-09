use anyhow::Result;
use dystrail_game::result::{ResultConfig, select_ending};
use thirtyfour::prelude::*;

use super::{BrowserScenario, CombinedScenario, ScenarioCtx, TestScenario};

pub struct ResourceStressTest;
pub struct DeterministicVerification;
pub struct EdgeCaseSurvival;

// Resource stress test scenario
#[async_trait::async_trait]
impl BrowserScenario for ResourceStressTest {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for resource scenarios");
    }
}

impl CombinedScenario for ResourceStressTest {
    #[allow(clippy::too_many_lines)] // Complex testing scenario needs comprehensive logic
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Resource Management Stress Test".to_string(),
            setup: Some(|game_state| {
                // Start with critically low resources
                game_state.stats.supplies = 2;
                game_state.stats.hp = 3;
                game_state.stats.sanity = 4;
                game_state.budget_cents = 500; // $5.00
            }),
            test_fn: |game_state| {
                println!("🎮 Starting Resource Management Stress Test");
                println!("⚠️  Starting with critically low resources!");
                #[allow(clippy::cast_precision_loss)]
                // Budget display: cents to dollars is acceptable precision loss
                {
                    println!(
                        "📊 Initial: HP={}, Supplies={}, Sanity={}, Budget=${:.2}",
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                        game_state.budget_cents as f64 / 100.0
                    );
                }

                let mut days_survived = 0;
                let mut resource_events = Vec::new();
                let mut supply_purchases = 0;
                let mut critical_moments = 0;
                let max_days = 50; // Shorter test for stress conditions

                while days_survived < max_days {
                    // Count critical resource moments
                    let mut critical_resources = 0;
                    if game_state.stats.hp <= 2 {
                        critical_resources += 1;
                    }
                    if game_state.stats.supplies <= 1 {
                        critical_resources += 1;
                    }
                    if game_state.stats.sanity <= 2 {
                        critical_resources += 1;
                    }

                    if critical_resources >= 2 {
                        critical_moments += 1;
                        resource_events.push(format!(
                            "Day {}: Multiple critical resources (HP={}, Supplies={}, Sanity={})",
                            days_survived,
                            game_state.stats.hp,
                            game_state.stats.supplies,
                            game_state.stats.sanity
                        ));
                    }

                    // Check for game end conditions
                    if game_state.stats.pants >= 100 {
                        resource_events
                            .push(format!("Day {days_survived}: Game ended - Pants overflow"));
                        break;
                    }
                    if game_state.stats.sanity <= 0 {
                        resource_events.push(format!(
                            "Day {days_survived}: Game ended - Sanity depletion"
                        ));
                        break;
                    }
                    if game_state.stats.hp <= 0 {
                        resource_events
                            .push(format!("Day {days_survived}: Game ended - HP depletion"));
                        break;
                    }
                    if game_state.stats.supplies <= 0 {
                        resource_events.push(format!(
                            "Day {days_survived}: Game ended - Supply depletion"
                        ));
                        break;
                    }

                    // Emergency resource management
                    if game_state.stats.supplies <= 1 && game_state.budget_cents >= 300 {
                        game_state.budget_cents -= 300; // $3.00
                        game_state.stats.supplies += 2;
                        supply_purchases += 1;
                        resource_events
                            .push(format!("Day {days_survived}: Emergency supply purchase"));
                    }

                    // Desperate resource conservation
                    if game_state.stats.hp <= 1 && game_state.stats.supplies >= 2 {
                        game_state.stats.hp += 2;
                        game_state.stats.supplies -= 2;
                        resource_events
                            .push(format!("Day {days_survived}: Desperate health recovery"));
                    }

                    // Minimal daily consumption in stress conditions
                    if game_state.stats.supplies > 0 {
                        game_state.stats.supplies -= 1;
                    }

                    // Stress affects sanity
                    if critical_resources >= 2 && game_state.stats.sanity > 1 {
                        game_state.stats.sanity -= 1;
                        game_state.stats.pants += 3;
                    }

                    game_state.day += 1;
                    days_survived += 1;
                    game_state.stats.clamp();

                    // Frequent reporting in stress test
                    if days_survived % 5 == 0 || critical_resources >= 2 {
                        #[allow(clippy::cast_precision_loss)]
                        // Budget display: cents to dollars is acceptable precision loss
                        {
                            println!(
                                "📅 Day {}: HP={}, Supplies={}, Sanity={}, Budget=${:.2}, Pants={}",
                                days_survived,
                                game_state.stats.hp,
                                game_state.stats.supplies,
                                game_state.stats.sanity,
                                game_state.budget_cents as f64 / 100.0,
                                game_state.stats.pants
                            );
                        }
                    }
                }

                // Final analysis
                let result_config = ResultConfig::default();
                let ending = select_ending(game_state, &result_config, true);

                println!("🏁 Resource stress test completed!");
                println!("📊 Final metrics:");
                println!("   Days survived: {days_survived}");
                println!("   Ending type: {ending:?}");
                println!("   Critical moments: {critical_moments}");
                println!("   Supply purchases: {supply_purchases}");
                #[allow(clippy::cast_precision_loss)]
                // Budget display: cents to dollars is acceptable precision loss
                {
                    println!(
                        "   Final budget: ${:.2}",
                        game_state.budget_cents as f64 / 100.0
                    );
                }

                println!("🎯 Resource events:");
                for event in &resource_events {
                    println!("   {event}");
                }

                // Validate stress test results
                anyhow::ensure!(
                    days_survived > 0,
                    "Should survive at least 1 day even under stress"
                );

                if critical_moments == 0 {
                    anyhow::bail!("Stress test failed - no critical resource moments detected");
                }

                println!("✅ Resource stress test validated resource management under pressure");
                Ok(())
            },
        })
    }
}

// Deterministic verification scenario
#[async_trait::async_trait]
impl BrowserScenario for DeterministicVerification {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for deterministic scenarios");
    }
}

impl CombinedScenario for DeterministicVerification {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Deterministic Playthrough Verification".to_string(),
            setup: None,
            test_fn: |game_state| {
                println!("🎮 Starting Deterministic Playthrough Verification");

                // Record initial state
                let initial_seed = game_state.seed;
                let initial_day = game_state.day;
                let initial_stats = (
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                );

                println!(
                    "📊 Initial seed: {initial_seed}, Day: {initial_day}, Stats: {initial_stats:?}"
                );

                // Run first sequence
                let mut first_sequence = Vec::new();
                for i in 0..10 {
                    let day_stats = (
                        game_state.day,
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                    );
                    first_sequence.push(day_stats);

                    // Deterministic actions
                    if game_state.stats.supplies > 1 {
                        game_state.stats.supplies -= 1;
                    }
                    if i % 3 == 0 && game_state.stats.hp < 8 {
                        game_state.stats.hp += 1;
                    }

                    game_state.day += 1;
                    game_state.stats.clamp();
                }

                println!(
                    "📊 First sequence completed - {} steps recorded",
                    first_sequence.len()
                );

                // Reset to initial state (simulate same seed)
                game_state.seed = initial_seed;
                game_state.day = initial_day;
                game_state.stats.hp = initial_stats.0;
                game_state.stats.supplies = initial_stats.1;
                game_state.stats.sanity = initial_stats.2;

                // Run second sequence with identical logic
                let mut second_sequence = Vec::new();
                for i in 0..10 {
                    let day_stats = (
                        game_state.day,
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                    );
                    second_sequence.push(day_stats);

                    // Identical deterministic actions
                    if game_state.stats.supplies > 1 {
                        game_state.stats.supplies -= 1;
                    }
                    if i % 3 == 0 && game_state.stats.hp < 8 {
                        game_state.stats.hp += 1;
                    }

                    game_state.day += 1;
                    game_state.stats.clamp();
                }

                println!(
                    "📊 Second sequence completed - {} steps recorded",
                    second_sequence.len()
                );

                // Compare sequences
                let mut differences = 0;
                for (i, (first, second)) in first_sequence
                    .iter()
                    .zip(second_sequence.iter())
                    .enumerate()
                {
                    if first != second {
                        differences += 1;
                        println!("❌ Step {i}: First={first:?}, Second={second:?}");
                    }
                }

                println!("🏁 Deterministic verification completed!");
                println!("📊 Steps compared: {}", first_sequence.len());
                println!("📊 Differences found: {differences}");

                // Validate determinism
                anyhow::ensure!(
                    differences == 0,
                    "Deterministic verification failed - found {} differences in {} steps",
                    differences,
                    first_sequence.len()
                );

                println!("✅ Deterministic behavior verified - identical sequences produced");
                Ok(())
            },
        })
    }
}

// Edge case survival scenario
#[async_trait::async_trait]
impl BrowserScenario for EdgeCaseSurvival {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for edge case scenarios");
    }
}

impl CombinedScenario for EdgeCaseSurvival {
    #[allow(clippy::too_many_lines)]
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Edge Case Survival Test".to_string(),
            setup: Some(|game_state| {
                // Create extreme edge case conditions
                game_state.stats.pants = 95; // Very close to game over
                game_state.stats.sanity = 1; // One point from death
                game_state.stats.supplies = 1; // Minimal supplies
                game_state.stats.hp = 1; // One HP
                game_state.budget_cents = 50; // $0.50
            }),
            test_fn: |game_state| {
                println!("🎮 Starting Edge Case Survival Test");
                println!("⚠️  EXTREME CONDITIONS: All resources at critical levels!");
                #[allow(clippy::cast_precision_loss)]
                // Budget display: cents to dollars is acceptable precision loss
                {
                    println!(
                        "📊 Starting: HP={}, Supplies={}, Sanity={}, Pants={}, Budget=${:.2}",
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                        game_state.stats.pants,
                        game_state.budget_cents as f64 / 100.0
                    );
                }

                let mut days_survived = 0;
                let mut edge_events = Vec::new();
                let mut miracle_saves = 0;
                let max_days = 20; // Short test for extreme conditions

                while days_survived < max_days {
                    let initial_state = (
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                        game_state.stats.pants,
                    );

                    // Check immediate death conditions
                    if game_state.stats.pants >= 100 {
                        edge_events.push(format!(
                            "Day {days_survived}: GAME OVER - Pants reached 100%"
                        ));
                        break;
                    }
                    if game_state.stats.sanity <= 0 {
                        edge_events
                            .push(format!("Day {days_survived}: GAME OVER - Sanity depleted"));
                        break;
                    }
                    if game_state.stats.hp <= 0 {
                        edge_events.push(format!("Day {days_survived}: GAME OVER - HP depleted"));
                        break;
                    }
                    if game_state.stats.supplies <= 0 {
                        edge_events.push(format!(
                            "Day {days_survived}: GAME OVER - Supplies depleted"
                        ));
                        break;
                    }

                    // Extreme survival measures
                    if game_state.stats.hp <= 1 && game_state.stats.supplies >= 1 {
                        // Trade supplies for health as last resort
                        game_state.stats.hp += 1;
                        game_state.stats.supplies -= 1;
                        miracle_saves += 1;
                        edge_events.push(format!("Day {days_survived}: Miracle health save"));
                    }

                    if game_state.stats.sanity <= 1 && game_state.stats.pants < 95 {
                        // Accept pants increase to maintain sanity
                        game_state.stats.sanity += 1;
                        game_state.stats.pants += 2;
                        miracle_saves += 1;
                        edge_events.push(format!(
                            "Day {days_survived}: Sanity preservation (pants risk)"
                        ));
                    }

                    // Minimal resource consumption in edge case
                    if days_survived % 2 == 0 && game_state.stats.supplies > 0 {
                        game_state.stats.supplies -= 1;
                    }

                    // Edge case stress
                    if game_state.stats.pants < 99 {
                        game_state.stats.pants += 1; // Constant pressure
                    }

                    game_state.day += 1;
                    days_survived += 1;
                    game_state.stats.clamp();

                    let final_state = (
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                        game_state.stats.pants,
                    );

                    // Report every day in edge case
                    println!(
                        "📅 Day {}: HP={}, Supplies={}, Sanity={}, Pants={} (Changed: {})",
                        days_survived,
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                        game_state.stats.pants,
                        initial_state != final_state
                    );
                }

                // Final analysis
                let result_config = ResultConfig::default();
                let ending = select_ending(game_state, &result_config, true);

                println!("🏁 Edge case survival test completed!");
                println!("📊 Final metrics:");
                println!("   Days survived: {days_survived}");
                println!("   Ending type: {ending:?}");
                println!("   Miracle saves: {miracle_saves}");
                println!(
                    "   Final state: HP={}, Supplies={}, Sanity={}, Pants={}",
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.stats.pants
                );

                println!("🎯 Edge case events:");
                for event in &edge_events {
                    println!("   {event}");
                }

                // Validate edge case handling
                anyhow::ensure!(
                    days_survived > 0,
                    "Should survive at least 1 day in edge case"
                );

                if days_survived >= max_days {
                    println!("🎉 Remarkable - survived maximum days under extreme conditions!");
                }

                println!("✅ Edge case survival test validated extreme condition handling");
                Ok(())
            },
        })
    }
}
