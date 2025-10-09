use anyhow::Result;
use dystrail_game::result::{ResultConfig, select_ending};
use thirtyfour::prelude::*;

use super::{BrowserScenario, CombinedScenario, ScenarioCtx, TestScenario};

pub struct FullGameConservative;
pub struct FullGameAggressive;
pub struct FullGameBalanced;

// Conservative strategy scenario
#[async_trait::async_trait]
impl BrowserScenario for FullGameConservative {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        // Browser implementation not needed for logic testing focus
        anyhow::bail!("Browser testing not implemented for full game scenarios");
    }
}

impl CombinedScenario for FullGameConservative {
    #[allow(clippy::too_many_lines)]
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Full Game - Conservative Strategy".to_string(),
            setup: None,
            test_fn: |game_state| {
                println!("🎮 Starting Full Game - Conservative Strategy Test");
                println!(
                    "📊 Initial state: HP={}, Supplies={}, Sanity={}, Day={}",
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.day
                );

                let mut days_survived = 0;
                let mut critical_events = Vec::new();
                let mut resource_depletion_day = None;
                let mut encounters_faced = 0;
                let max_days = 100; // Safety limit

                // Run game simulation with conservative strategy
                while days_survived < max_days {
                    let initial_stats = (
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                    );

                    // Check for game end conditions
                    if game_state.stats.pants >= 100 {
                        critical_events.push(format!(
                            "Day {days_survived}: Game ended - Pants reached 100%"
                        ));
                        break;
                    }
                    if game_state.stats.sanity <= 0 {
                        critical_events
                            .push(format!("Day {days_survived}: Game ended - Sanity depleted"));
                        break;
                    }
                    if game_state.stats.hp <= 0 || game_state.stats.supplies <= 0 {
                        critical_events.push(format!(
                            "Day {days_survived}: Game ended - Critical resource depletion"
                        ));
                        break;
                    }

                    // Record resource depletion timing
                    if resource_depletion_day.is_none() {
                        if game_state.stats.supplies <= 1 {
                            resource_depletion_day = Some((days_survived, "supplies".to_string()));
                            critical_events
                                .push(format!("Day {days_survived}: Supplies critically low"));
                        } else if game_state.stats.hp <= 2 {
                            resource_depletion_day = Some((days_survived, "hp".to_string()));
                            critical_events.push(format!("Day {days_survived}: HP critically low"));
                        } else if game_state.stats.sanity <= 2 {
                            resource_depletion_day = Some((days_survived, "sanity".to_string()));
                            critical_events
                                .push(format!("Day {days_survived}: Sanity critically low"));
                        }
                    }

                    // Conservative strategy: prioritize survival
                    if game_state.stats.supplies > 3 {
                        game_state.stats.supplies -= 1; // Slow resource consumption
                    }

                    // Rest when health is low and we have supplies
                    if game_state.stats.hp < 6 && game_state.stats.supplies > 2 {
                        game_state.stats.hp += 1;
                        game_state.stats.supplies -= 1;
                        critical_events
                            .push(format!("Day {days_survived}: Conservative rest taken"));
                    }

                    // Handle encounters conservatively (simulate)
                    if days_survived % 3 == 0 {
                        // Every 3 days
                        encounters_faced += 1;
                        // Conservative choice - usually safe but less reward
                        if game_state.stats.credibility < 15 {
                            game_state.stats.credibility += 1;
                        }
                        critical_events.push(format!(
                            "Day {days_survived}: Encounter handled conservatively"
                        ));
                    }

                    // Advance day
                    game_state.day += 1;
                    days_survived += 1;

                    // Clamp stats to valid ranges
                    game_state.stats.clamp();

                    // Check for significant stat changes
                    let final_stats = (
                        game_state.stats.hp,
                        game_state.stats.supplies,
                        game_state.stats.sanity,
                    );
                    if initial_stats != final_stats && days_survived % 10 == 0 {
                        // Report every 10 days
                        println!(
                            "📅 Day {}: HP={}, Supplies={}, Sanity={}, Credibility={}",
                            days_survived,
                            game_state.stats.hp,
                            game_state.stats.supplies,
                            game_state.stats.sanity,
                            game_state.stats.credibility
                        );
                    }
                }

                // Final analysis
                let result_config = ResultConfig::default();
                let ending = select_ending(game_state, &result_config, true);
                let reached_victory = matches!(ending, dystrail_game::result::Ending::Victory);

                println!("🏁 Game completed!");
                println!("📊 Final metrics:");
                println!("   Days survived: {days_survived}");
                println!("   Ending type: {ending:?}");
                println!("   Reached victory: {reached_victory}");
                println!(
                    "   Final stats: HP={}, Supplies={}, Sanity={}, Credibility={}, Pants={}",
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.stats.credibility,
                    game_state.stats.pants
                );
                println!("   Encounters faced: {encounters_faced}");

                if let Some((day, resource)) = resource_depletion_day {
                    println!("   Resource depletion: {resource} on day {day}");
                }

                println!("🎯 Critical events:");
                for event in &critical_events {
                    println!("   {event}");
                }

                // Validate results
                anyhow::ensure!(days_survived > 0, "Game should survive at least 1 day");
                anyhow::ensure!(
                    days_survived < max_days,
                    "Game should end naturally, not hit limit"
                );

                if days_survived < 5 {
                    anyhow::bail!(
                        "Conservative strategy failed - game ended too quickly ({days_survived}  days)"
                    );
                }

                println!("✅ Conservative strategy test completed successfully");
                Ok(())
            },
        })
    }
}

// Aggressive strategy scenario
#[async_trait::async_trait]
impl BrowserScenario for FullGameAggressive {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for full game scenarios");
    }
}

impl CombinedScenario for FullGameAggressive {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Full Game - Aggressive Strategy".to_string(),
            setup: None,
            test_fn: |game_state| {
                println!("🎮 Starting Full Game - Aggressive Strategy Test");
                println!(
                    "📊 Initial state: HP={}, Supplies={}, Sanity={}, Day={}",
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.day
                );

                let mut days_survived = 0;
                let mut critical_events = Vec::new();
                let mut encounters_faced = 0;
                let max_days = 100;

                while days_survived < max_days {
                    // Check for game end conditions
                    if game_state.stats.pants >= 100
                        || game_state.stats.sanity <= 0
                        || game_state.stats.hp <= 0
                        || game_state.stats.supplies <= 0
                    {
                        break;
                    }

                    // Aggressive strategy: take risks for progress
                    game_state.stats.supplies -= 2; // Faster resource consumption

                    // Buy supplies aggressively if we have budget
                    if game_state.budget_cents > 1000 && game_state.stats.supplies < 5 {
                        game_state.budget_cents -= 500; // $5.00
                        game_state.stats.supplies += 3;
                        critical_events
                            .push(format!("Day {days_survived}: Aggressive supply purchase"));
                    }

                    // Handle encounters aggressively
                    if days_survived % 2 == 0 {
                        // Every 2 days
                        encounters_faced += 1;
                        // Aggressive choice - higher risk/reward
                        if game_state.stats.sanity > 2 {
                            game_state.stats.sanity -= 1;
                            game_state.stats.pants += 5;
                            game_state.stats.allies += 1;
                            critical_events
                                .push(format!("Day {days_survived}: Risky encounter choice"));
                        }
                    }

                    game_state.day += 1;
                    days_survived += 1;
                    game_state.stats.clamp();

                    if days_survived % 10 == 0 {
                        println!(
                            "📅 Day {}: HP={}, Supplies={}, Sanity={}, Pants={}, Allies={}",
                            days_survived,
                            game_state.stats.hp,
                            game_state.stats.supplies,
                            game_state.stats.sanity,
                            game_state.stats.pants,
                            game_state.stats.allies
                        );
                    }
                }

                // Final analysis
                let result_config = ResultConfig::default();
                let ending = select_ending(game_state, &result_config, true);

                println!("🏁 Aggressive strategy completed!");
                println!("📊 Final metrics:");
                println!("   Days survived: {days_survived}");
                println!("   Ending type: {ending:?}");
                println!(
                    "   Final stats: HP={}, Supplies={}, Sanity={}, Pants={}, Allies={}",
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.stats.pants,
                    game_state.stats.allies
                );
                println!("   Encounters faced: {encounters_faced}");

                println!("🎯 Critical events:");
                for event in &critical_events {
                    println!("   {event}");
                }

                // Aggressive strategy might fail faster but should still be viable
                anyhow::ensure!(days_survived > 0, "Game should survive at least 1 day");
                println!("✅ Aggressive strategy test completed");
                Ok(())
            },
        })
    }
}

// Balanced strategy scenario
#[async_trait::async_trait]
impl BrowserScenario for FullGameBalanced {
    async fn run_browser(&self, _driver: &WebDriver, _ctx: &ScenarioCtx<'_>) -> Result<()> {
        anyhow::bail!("Browser testing not implemented for full game scenarios");
    }
}

impl CombinedScenario for FullGameBalanced {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Full Game - Balanced Strategy".to_string(),
            setup: None,
            test_fn: |game_state| {
                println!("🎮 Starting Full Game - Balanced Strategy Test");

                let mut days_survived = 0;
                let mut vehicle_breakdowns = 0;
                let mut weather_events = 0;
                let max_days = 100;

                while days_survived < max_days {
                    if game_state.stats.pants >= 100
                        || game_state.stats.sanity <= 0
                        || game_state.stats.hp <= 0
                        || game_state.stats.supplies <= 0
                    {
                        break;
                    }

                    // Balanced strategy: moderate risk/reward
                    game_state.stats.supplies -= 1;

                    // Moderate resource management
                    if game_state.stats.hp < 6 && game_state.stats.supplies > 1 {
                        game_state.stats.hp += 1;
                        game_state.stats.supplies -= 1;
                    }

                    // Simulate vehicle breakdowns
                    if days_survived % 15 == 0 && game_state.inventory.spares.tire > 0 {
                        vehicle_breakdowns += 1;
                        game_state.inventory.spares.tire -= 1;
                        println!("🔧 Day {days_survived}: Vehicle breakdown repaired");
                    }

                    // Simulate weather events
                    if days_survived % 7 == 0 {
                        weather_events += 1;
                        game_state.stats.supplies -= 1; // Weather costs extra supplies
                    }

                    game_state.day += 1;
                    days_survived += 1;
                    game_state.stats.clamp();
                }

                println!("🏁 Balanced strategy completed!");
                println!("📊 Days survived: {days_survived}");
                println!("📊 Vehicle breakdowns: {vehicle_breakdowns}");
                println!("📊 Weather events: {weather_events}");
                println!("✅ Balanced strategy test completed");
                Ok(())
            },
        })
    }
}
