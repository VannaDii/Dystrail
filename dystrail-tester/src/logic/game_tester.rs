use dystrail_game::data::EncounterData;
use dystrail_game::pacing::PacingConfig;
use dystrail_game::{GameMode, GameState};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
pub struct PlayabilityMetrics {
    pub days_survived: i32,
    pub ending_type: String,
    pub encounters_faced: i32,
    pub vehicle_breakdowns: i32,
    pub final_hp: i32,
    pub final_supplies: i32,
    pub final_sanity: i32,
    pub final_pants: i32,
    pub final_budget_cents: i64,
}

impl PlayabilityMetrics {
    pub fn new() -> Self {
        Self {
            days_survived: 0,
            ending_type: "In Progress".to_string(),
            encounters_faced: 0,
            vehicle_breakdowns: 0,
            final_hp: 10,
            final_supplies: 10,
            final_sanity: 10,
            final_pants: 0,
            final_budget_cents: 10000,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GameplayStrategy {
    Conservative,    // Prioritize survival over speed
    Aggressive,      // Take risks for faster progress
    Balanced,        // Moderate approach
    ResourceManager, // Focus on resource efficiency
}

pub struct GameTester {
    verbose: bool,
}

impl GameTester {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Play a complete game using the given strategy and seed
    pub fn play_game(
        &self,
        mode: GameMode,
        strategy: GameplayStrategy,
        seed: u64,
    ) -> PlayabilityMetrics {
        // Create game state with provided seed
        let mut game_state = GameState {
            mode,
            seed,
            ..GameState::default()
        };

        // Load encounter data (use empty for testing - encounters will be minimal)
        let encounter_data = EncounterData::empty();
        game_state.data = Some(encounter_data);

        // Initialize RNG with seed
        let rng = ChaCha20Rng::seed_from_u64(seed);
        game_state.rng = Some(rng);

        let mut metrics = PlayabilityMetrics::new();
        let max_days = 200; // Safety limit to prevent infinite loops

        if self.verbose {
            println!("🎮 Starting game with seed: {seed}, mode: {mode:?}");
            #[allow(clippy::cast_precision_loss)] // Budget display: cents to dollars is acceptable
            {
                println!(
                    "📊 Initial state: HP={}, Supplies={}, Sanity={}, Pants={}, Budget=${:.2}",
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.stats.pants,
                    game_state.budget_cents as f64 / 100.0
                );
            }
        }

        // Main gameplay loop - simulate actual turns
        loop {
            // Record pre-turn state for metrics

            // Apply pace and diet effects (this is normally done in UI before travel_next_leg)
            let pacing_config = PacingConfig::default_config();
            game_state.apply_pace_and_diet(&pacing_config);

            // Handle any active encounter first
            if let Some(encounter) = game_state.current_encounter.clone() {
                let choice_idx = Self::choose_encounter_action(&encounter, &game_state, strategy);
                game_state.apply_choice(choice_idx);
                metrics.encounters_faced += 1;

                if self.verbose {
                    println!(
                        "🎯 Day {}: Handled encounter '{}' with choice {}",
                        game_state.day, encounter.name, choice_idx
                    );
                }
            }

            // Travel to next leg (this advances the day and handles game logic)
            let (game_ended, travel_message) = game_state.travel_next_leg();

            // Record turn metrics
            // Track vehicle breakdowns
            if game_state.travel_blocked {
                metrics.vehicle_breakdowns += 1;
            }

            if self.verbose && game_state.day.is_multiple_of(10) {
                println!(
                    "📅 Day {}: HP={}, Supplies={}, Sanity={}, Pants={}",
                    game_state.day,
                    game_state.stats.hp,
                    game_state.stats.supplies,
                    game_state.stats.sanity,
                    game_state.stats.pants
                );
            }

            // Check for game end conditions
            if game_ended || game_state.day >= max_days {
                metrics.ending_type =
                    Self::determine_ending_type(&game_state, game_ended, &travel_message);
                break;
            }
        }

        // Record final metrics
        metrics.days_survived = i32::try_from(game_state.day).unwrap_or(i32::MAX);
        metrics.final_hp = game_state.stats.hp;
        metrics.final_supplies = game_state.stats.supplies;
        metrics.final_sanity = game_state.stats.sanity;
        metrics.final_pants = game_state.stats.pants;
        metrics.final_budget_cents = game_state.budget_cents;
        // TODO: Calculate final score when scoring system is available

        if self.verbose {
            println!(
                "🏁 Game ended after {} days: {}",
                metrics.days_survived, metrics.ending_type
            );
        }

        metrics
    }

    fn choose_encounter_action(
        encounter: &dystrail_game::data::Encounter,
        _game_state: &GameState,
        strategy: GameplayStrategy,
    ) -> usize {
        if encounter.choices.is_empty() {
            return 0;
        }

        match strategy {
            GameplayStrategy::Conservative => {
                // Choose option with least negative impact on critical resources
                encounter
                    .choices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, choice)| {
                        let mut risk = 0;
                        if choice.effects.hp < 0 {
                            risk += (-choice.effects.hp) * 4;
                        }
                        if choice.effects.supplies < 0 {
                            risk += (-choice.effects.supplies) * 3;
                        }
                        if choice.effects.sanity < 0 {
                            risk += (-choice.effects.sanity) * 2;
                        }
                        if choice.effects.pants > 0 {
                            risk += choice.effects.pants * 2;
                        }
                        risk
                    })
                    .map_or(0, |(i, _)| i)
            }
            GameplayStrategy::Aggressive => {
                // Choose option with highest potential reward, accepting risk
                encounter
                    .choices
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, choice)| {
                        let mut reward = 0;
                        reward += choice.effects.hp.max(0) * 2;
                        reward += choice.effects.supplies.max(0) * 2;
                        reward += choice.effects.credibility.max(0) * 3;
                        reward += choice.effects.allies.max(0);
                        // Slightly penalize pants increase but don't avoid it entirely
                        reward -= choice.effects.pants.max(0);
                        reward
                    })
                    .map_or(0, |(i, _)| i)
            }
            GameplayStrategy::Balanced => {
                // Balance risk vs reward
                encounter
                    .choices
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, choice)| {
                        let reward = choice.effects.hp.max(0)
                            + choice.effects.supplies.max(0)
                            + choice.effects.credibility.max(0)
                            + choice.effects.allies.max(0);
                        let risk = (-choice.effects.hp).max(0)
                            + (-choice.effects.supplies).max(0)
                            + (-choice.effects.sanity).max(0)
                            + choice.effects.pants.max(0);
                        reward - risk
                    })
                    .map_or(0, |(i, _)| i)
            }
            GameplayStrategy::ResourceManager => {
                // Prioritize resource preservation above all else
                encounter
                    .choices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, choice)| {
                        (-choice.effects.hp).max(0) * 5
                            + (-choice.effects.supplies).max(0) * 4
                            + (-choice.effects.sanity).max(0) * 3
                            + choice.effects.pants.max(0) * 2
                    })
                    .map_or(0, |(i, _)| i)
            }
        }
    }

    fn determine_ending_type(
        game_state: &GameState,
        game_ended: bool,
        travel_message: &str,
    ) -> String {
        if game_state.stats.pants >= 100 {
            "Pants Emergency - Game Over".to_string()
        } else if game_state.stats.hp <= 0 {
            "Health Depleted - Game Over".to_string()
        } else if game_state.stats.sanity <= 0 {
            "Sanity Depleted - Game Over".to_string()
        } else if game_state.stats.supplies <= 0 {
            "Supplies Depleted - Game Over".to_string()
        } else if game_state.day >= 200 {
            "Max Days Reached".to_string()
        } else if travel_message.contains("victory") || travel_message.contains("boss") {
            "Victory - Boss Defeated".to_string()
        } else if game_ended {
            format!("Game Ended: {travel_message}")
        } else {
            "Unknown Ending".to_string()
        }
    }
}
