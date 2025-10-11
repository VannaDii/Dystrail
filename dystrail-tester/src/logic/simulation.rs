use dystrail_game::boss::{self, BossConfig, BossOutcome};
use dystrail_game::camp::{self, CampConfig};
use dystrail_game::data::EncounterData;
use dystrail_game::exec_orders::{DailyEffect, ExecOrdersConfig};
use dystrail_game::{GameMode, GameState};

use crate::logic::policy::{PlayerPolicy, PolicyDecision};

use dystrail_game::pacing::PacingConfig;

/// Configuration for a simulation session.
#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub seed: u64,
    pub mode: GameMode,
    pub max_days: u32,
}

impl SimulationConfig {
    #[must_use]
    pub fn new(mode: GameMode, seed: u64) -> Self {
        Self {
            seed,
            mode,
            max_days: 200,
        }
    }

    #[must_use]
    pub fn with_max_days(mut self, max_days: u32) -> Self {
        self.max_days = max_days;
        self
    }
}

/// Snapshot of a resolved encounter.
#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub day: u32,
    pub encounter_id: String,
    pub encounter_name: String,
    pub choice_index: usize,
    pub choice_label: String,
    pub policy_name: String,
    pub rationale: Option<String>,
}

/// Result of advancing the simulation by one turn/day.
#[derive(Debug, Clone)]
pub struct TurnOutcome {
    pub day: u32,
    pub travel_message: String,
    pub breakdown_started: bool,
    pub game_ended: bool,
    pub decision: Option<DecisionRecord>,
}

/// Core deterministic simulation harness used by the tester.
pub struct SimulationSession {
    state: GameState,
    pacing_config: PacingConfig,
    camp_config: CampConfig,
    exec_config: ExecOrdersConfig,
    boss_config: BossConfig,
    max_days: u32,
}

impl SimulationSession {
    pub fn new(
        config: SimulationConfig,
        encounters: EncounterData,
        pacing_config: PacingConfig,
        camp_config: CampConfig,
        exec_config: ExecOrdersConfig,
        boss_config: BossConfig,
    ) -> Self {
        let mut state = GameState::default().with_seed(config.seed, config.mode, encounters);
        state.trail_distance = boss_config.distance_required;
        Self {
            state,
            pacing_config,
            camp_config,
            exec_config,
            boss_config,
            max_days: config.max_days,
        }
    }

    #[must_use]
    pub fn state(&self) -> &GameState {
        &self.state
    }

    #[must_use]
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    #[must_use]
    pub fn into_state(self) -> GameState {
        self.state
    }

    pub fn advance(&mut self, policy: &mut dyn PlayerPolicy) -> TurnOutcome {
        self.state.tick_camp_cooldowns();
        self.state.refresh_exec_order();

        if self.state.boss_ready
            && !self.state.boss_attempted
            && self.state.camp.rest_cooldown == 0
            && !self.state.rest_requested
        {
            self.state.rest_requested = true;
        }

        let forage_cfg = self.camp_config.forage.clone();
        if forage_cfg.supplies > 0
            && self.state.camp.forage_cooldown == 0
            && self.state.stats.supplies <= forage_cfg.supplies.max(2)
        {
            let camp_cfg = self.camp_config.clone();
            let outcome = camp::camp_forage(self.state_mut(), &camp_cfg);
            let day = self.state.day;
            let game_ended = day >= self.max_days;
            return TurnOutcome {
                day,
                travel_message: outcome.message,
                breakdown_started: false,
                game_ended,
                decision: None,
            };
        }

        let wants_rest = self.state.rest_requested || self.state.should_auto_rest();

        if wants_rest {
            self.state.rest_requested = false;
            let camp_cfg = self.camp_config.clone();
            let outcome = camp::camp_rest(self.state_mut(), &camp_cfg);
            if outcome.rested {
                let day = self.state.day;
                let game_ended = day >= self.max_days;
                return TurnOutcome {
                    day,
                    travel_message: outcome.message,
                    breakdown_started: false,
                    game_ended,
                    decision: None,
                };
            }
        }

        self.state.apply_pace_and_diet(&self.pacing_config);
        let daily_effect = self.daily_effect();
        self.state
            .consume_daily_effects(daily_effect.sanity, daily_effect.supplies);

        let mut decision: Option<DecisionRecord> = None;

        if let Some(encounter) = self.state.current_encounter.clone() {
            let PolicyDecision {
                choice_index,
                rationale,
            } = policy.pick_choice(&self.state, &encounter);

            let safe_index = clamp_choice_index(choice_index, &encounter);
            let choice_label = encounter.choices.get(safe_index).map_or_else(
                || "No available choice".to_string(),
                |choice| choice.label.clone(),
            );

            decision = Some(DecisionRecord {
                day: self.state.day,
                encounter_id: encounter.id.clone(),
                encounter_name: encounter.name.clone(),
                choice_index: safe_index,
                choice_label,
                policy_name: policy.name().to_string(),
                rationale,
            });

            self.state.apply_choice(safe_index);
        }

        let (mut game_ended, mut travel_message, breakdown_started) = self.state.travel_next_leg();
        if !game_ended && self.state.day >= self.max_days {
            game_ended = true;
            travel_message = String::from("Max days reached");
        }

        if self.state.boss_ready && !self.state.boss_attempted {
            let boss_cfg = self.boss_config.clone();
            let outcome = boss::run_boss_minigame(self.state_mut(), &boss_cfg);
            game_ended = true;
            travel_message = match outcome {
                BossOutcome::PassedCloture => String::from("log.boss.victory"),
                BossOutcome::SurvivedFlood => String::from("log.boss.failure"),
                BossOutcome::PantsEmergency => String::from("log.pants-emergency"),
                BossOutcome::Exhausted => String::from("log.sanity-collapse"),
            };
            self.state.boss_ready = false;
        }

        TurnOutcome {
            day: self.state.day,
            travel_message,
            breakdown_started,
            game_ended,
            decision,
        }
    }
}

fn clamp_choice_index(index: usize, encounter: &dystrail_game::data::Encounter) -> usize {
    if encounter.choices.is_empty() {
        0
    } else if index >= encounter.choices.len() {
        encounter.choices.len() - 1
    } else {
        index
    }
}

impl SimulationSession {
    fn daily_effect(&mut self) -> DailyEffect {
        let pace = self
            .pacing_config
            .get_pace_safe(self.state.pace.as_str());
        let diet = self
            .pacing_config
            .get_diet_safe(self.state.diet.as_str());
        let exec = self
            .state
            .current_order
            .effect(&self.exec_config, self.state.day);

        DailyEffect {
            sanity: pace.sanity + diet.sanity + exec.sanity,
            supplies: exec.supplies,
        }
    }
}
