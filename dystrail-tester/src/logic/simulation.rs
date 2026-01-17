use dystrail_game::boss::{self, BossConfig, BossOutcome};
use dystrail_game::camp::{self, CampConfig};
use dystrail_game::data::EncounterData;
use dystrail_game::endgame::EndgameTravelCfg;
use dystrail_game::{
    CrossingChoice, CrossingConfig, DayOutcome, GameMode, GameState, JourneySession,
    MechanicalPolicyId, OtDeluxe90sPolicy, OtDeluxeCrossingMethod, OtDeluxeRouteDecision,
    OtDeluxeRoutePrompt, PaceId, StrategyId, can_afford_bribe, can_use_permit,
    otdeluxe_crossing_options,
};

use crate::logic::policy::{GameplayStrategy, PlayerPolicy, PolicyDecision};

/// Configuration for a simulation session.
#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub seed: u64,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub max_days: u32,
}

impl SimulationConfig {
    #[must_use]
    pub const fn new(mode: GameMode, strategy: GameplayStrategy, seed: u64) -> Self {
        Self {
            seed,
            mode,
            strategy,
            max_days: 200,
        }
    }

    #[must_use]
    pub const fn with_max_days(mut self, max_days: u32) -> Self {
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
    pub miles_traveled_actual: f32,
}

const fn strategy_id_for(strategy: GameplayStrategy) -> StrategyId {
    match strategy {
        GameplayStrategy::Balanced => StrategyId::Balanced,
        GameplayStrategy::Aggressive => StrategyId::Aggressive,
        GameplayStrategy::Conservative => StrategyId::Conservative,
        GameplayStrategy::ResourceManager => StrategyId::ResourceManager,
    }
}

/// Core deterministic simulation harness used by the tester.
pub struct SimulationSession {
    session: JourneySession,
    camp_config: CampConfig,
    boss_config: BossConfig,
    max_days: u32,
    strategy: GameplayStrategy,
    conservative_heat_days: u32,
    aggressive_heat_days: u32,
}

impl SimulationSession {
    pub fn new(
        config: SimulationConfig,
        encounters: EncounterData,
        camp_config: CampConfig,
        endgame_config: &EndgameTravelCfg,
        boss_config: BossConfig,
    ) -> Self {
        let strategy_id = strategy_id_for(config.strategy);
        let mut session = JourneySession::new(
            config.mode,
            strategy_id,
            config.seed,
            encounters,
            endgame_config,
        );
        session.state_mut().trail_distance = boss_config.distance_required;
        Self {
            session,
            camp_config,
            boss_config,
            max_days: config.max_days,
            strategy: config.strategy,
            conservative_heat_days: 0,
            aggressive_heat_days: 0,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &GameState {
        self.session.state()
    }

    #[must_use]
    pub const fn state_mut(&mut self) -> &mut GameState {
        self.session.state_mut()
    }

    #[must_use]
    pub fn into_state(self) -> GameState {
        self.session.into_state()
    }

    pub fn advance(&mut self, policy: &mut dyn PlayerPolicy) -> TurnOutcome {
        if let Some(outcome) = self.try_resolve_route_prompt() {
            return outcome;
        }
        if let Some(outcome) = self.try_resolve_crossing() {
            return outcome;
        }
        self.session.state_mut().tick_camp_cooldowns();
        self.queue_boss_rest();

        if let Some(outcome) = self.try_forage_day() {
            return outcome;
        }
        if let Some(outcome) = self.try_rest_day() {
            return outcome;
        }

        self.adjust_daily_pace();
        let decision = self.resolve_encounter_choice(policy);

        let outcome = self.session.tick_day();
        self.finalize_outcome(outcome, decision)
    }

    const fn queue_boss_rest(&mut self) {
        let boss_ready = {
            let state = self.session.state();
            state.boss.readiness.ready
                && !state.boss.outcome.attempted
                && state.camp.rest_cooldown == 0
                && !state.day_state.rest.rest_requested
        };
        if boss_ready {
            self.session.state_mut().day_state.rest.rest_requested = true;
        }
    }

    fn try_forage_day(&mut self) -> Option<TurnOutcome> {
        let forage_cfg = self.camp_config.forage.clone();
        let should_forage = {
            let state = self.session.state();
            forage_cfg.supplies > 0
                && state.camp.forage_cooldown == 0
                && state.stats.supplies <= forage_cfg.supplies.max(2)
        };
        if !should_forage {
            return None;
        }
        let camp_cfg = self.camp_config.clone();
        let outcome = camp::camp_forage(self.session.state_mut(), &camp_cfg);
        Some(self.build_nontravel_outcome(outcome.message))
    }

    fn try_rest_day(&mut self) -> Option<TurnOutcome> {
        let wants_rest = {
            let state = self.session.state();
            state.day_state.rest.rest_requested || state.should_auto_rest()
        };
        if !wants_rest {
            return None;
        }
        self.session.state_mut().day_state.rest.rest_requested = false;
        let camp_cfg = self.camp_config.clone();
        let outcome = camp::camp_rest(self.session.state_mut(), &camp_cfg);
        if outcome.rested {
            Some(self.build_nontravel_outcome(outcome.message))
        } else {
            None
        }
    }

    fn resolve_encounter_choice(
        &mut self,
        policy: &mut dyn PlayerPolicy,
    ) -> Option<DecisionRecord> {
        let encounter = self.session.state().current_encounter.clone()?;
        let PolicyDecision {
            choice_index,
            rationale,
        } = policy.pick_choice(self.session.state(), &encounter);
        let safe_index = clamp_choice_index(choice_index, &encounter);
        let choice_label = encounter.choices.get(safe_index).map_or_else(
            || "No available choice".to_string(),
            |choice| choice.label.clone(),
        );
        let decision = DecisionRecord {
            day: self.session.state().day,
            encounter_id: encounter.id.clone(),
            encounter_name: encounter.name.clone(),
            choice_index: safe_index,
            choice_label,
            policy_name: policy.name().to_string(),
            rationale,
        };
        self.session.state_mut().apply_choice(safe_index);
        Some(decision)
    }

    fn try_resolve_route_prompt(&mut self) -> Option<TurnOutcome> {
        let state = self.session.state();
        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return None;
        }
        let prompt = state.ot_deluxe.route.pending_prompt?;
        let decision = match (prompt, self.strategy) {
            (OtDeluxeRoutePrompt::SubletteCutoff, GameplayStrategy::Aggressive) => {
                OtDeluxeRouteDecision::SubletteCutoff
            }
            (OtDeluxeRoutePrompt::DallesShortcut, GameplayStrategy::Aggressive) => {
                OtDeluxeRouteDecision::DallesShortcut
            }
            (OtDeluxeRoutePrompt::DallesFinal, GameplayStrategy::Aggressive) => {
                OtDeluxeRouteDecision::RaftColumbia
            }
            (OtDeluxeRoutePrompt::DallesFinal, _) => OtDeluxeRouteDecision::BarlowRoad,
            _ => OtDeluxeRouteDecision::StayOnTrail,
        };
        self.session.state_mut().set_route_prompt_choice(decision);
        let outcome = self.session.tick_day();
        Some(self.finalize_outcome(outcome, None))
    }

    fn try_resolve_crossing(&mut self) -> Option<TurnOutcome> {
        if self.session.state().mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            if !self.session.state().ot_deluxe.crossing.choice_pending {
                return None;
            }
            let river_kind = self.session.state().ot_deluxe.crossing.river_kind?;
            let river_state = self.session.state().ot_deluxe.crossing.river.as_ref()?;
            let policy = OtDeluxe90sPolicy::default();
            let options = otdeluxe_crossing_options(
                &policy.crossings,
                river_kind,
                river_state,
                &self.session.state().ot_deluxe.inventory,
            );
            let method = if options.ferry() {
                OtDeluxeCrossingMethod::Ferry
            } else if options.guide() {
                OtDeluxeCrossingMethod::Guide
            } else if options.caulk_float() {
                OtDeluxeCrossingMethod::CaulkFloat
            } else {
                OtDeluxeCrossingMethod::Ford
            };
            self.session
                .state_mut()
                .set_otdeluxe_crossing_choice(method);
            let outcome = self.session.tick_day();
            return Some(self.finalize_outcome(outcome, None));
        }

        let pending = self.session.state().pending_crossing?;
        let kind = pending.kind;
        let cfg = CrossingConfig::default();
        let choice = if can_use_permit(self.session.state(), &kind) {
            CrossingChoice::Permit
        } else if can_afford_bribe(self.session.state(), &cfg, kind) {
            CrossingChoice::Bribe
        } else {
            CrossingChoice::Detour
        };
        self.session.state_mut().set_crossing_choice(choice);
        let outcome = self.session.tick_day();
        Some(self.finalize_outcome(outcome, None))
    }

    fn try_boss_minigame(&mut self) -> Option<String> {
        let boss_ready = {
            let state = self.session.state();
            state.boss.readiness.ready && !state.boss.outcome.attempted
        };
        if !boss_ready {
            return None;
        }
        let boss_cfg = self.boss_config.clone();
        let outcome = boss::run_boss_minigame(self.session.state_mut(), &boss_cfg);
        self.session.state_mut().boss.readiness.ready = false;
        Some(match outcome {
            BossOutcome::PassedCloture => String::from("log.boss.victory"),
            BossOutcome::SurvivedFlood => String::from("log.boss.failure"),
            BossOutcome::PantsEmergency => String::from("log.pants-emergency"),
            BossOutcome::Exhausted => String::from("log.sanity-collapse"),
        })
    }

    const fn build_nontravel_outcome(&self, travel_message: String) -> TurnOutcome {
        let game_ended = self.session.state().day >= self.max_days;
        self.build_turn_outcome(travel_message, false, game_ended, None)
    }

    fn finalize_outcome(
        &mut self,
        outcome: DayOutcome,
        decision: Option<DecisionRecord>,
    ) -> TurnOutcome {
        let breakdown_started = outcome.breakdown_started;
        let mut game_ended = outcome.ended;
        let day_limit_reached = !game_ended && self.session.state().day >= self.max_days;
        if day_limit_reached {
            game_ended = true;
        }
        let boss_message = self.try_boss_minigame();
        if boss_message.is_some() {
            game_ended = true;
        }
        let travel_message = if let Some(message) = boss_message {
            message
        } else if day_limit_reached {
            String::from("Max days reached")
        } else {
            outcome.log_key
        };

        self.build_turn_outcome(travel_message, breakdown_started, game_ended, decision)
    }

    const fn build_turn_outcome(
        &self,
        travel_message: String,
        breakdown_started: bool,
        game_ended: bool,
        decision: Option<DecisionRecord>,
    ) -> TurnOutcome {
        let state = self.session.state();
        TurnOutcome {
            day: state.day,
            travel_message,
            breakdown_started,
            game_ended,
            decision,
            miles_traveled_actual: state.miles_traveled_actual,
        }
    }

    fn adjust_daily_pace(&mut self) {
        let strategy = self.strategy;
        match strategy {
            GameplayStrategy::Balanced | GameplayStrategy::ResourceManager => {
                let (healthy, supplies_ok, illness_active) = {
                    let state = self.session.state();
                    (
                        state.stats.hp >= 8 && state.stats.sanity >= 7,
                        state.stats.supplies >= 6,
                        state.illness_travel_penalty < 0.99,
                    )
                };
                let state = self.session.state_mut();
                if healthy && supplies_ok && !illness_active {
                    if matches!(state.pace, PaceId::Steady) {
                        state.pace = PaceId::Heated;
                    }
                } else if state.stats.hp <= 5 || state.stats.sanity <= 5 || illness_active {
                    state.pace = PaceId::Steady;
                }
            }
            GameplayStrategy::Aggressive => {
                let mut aggressive_heat_days = self.aggressive_heat_days;
                {
                    let state = self.session.state_mut();
                    if state.stats.hp <= 4 || state.stats.sanity <= 4 {
                        state.pace = PaceId::Steady;
                        aggressive_heat_days = 0;
                    } else {
                        if state.mode.is_deep() && aggressive_heat_days == 0 {
                            let ratio_10 = state.travel_ratio_recent(10);
                            if ratio_10 < 0.85 {
                                aggressive_heat_days = 3;
                            }
                        }
                        state.pace = PaceId::Heated;
                        if aggressive_heat_days > 0 {
                            aggressive_heat_days = aggressive_heat_days.saturating_sub(1);
                        }
                    }
                }
                self.aggressive_heat_days = aggressive_heat_days;
            }
            GameplayStrategy::Conservative => {
                let mut conservative_heat_days = self.conservative_heat_days;
                {
                    let state = self.session.state_mut();
                    if conservative_heat_days > 0 {
                        if state.stats.hp <= 4 || state.stats.sanity <= 4 {
                            conservative_heat_days = 0;
                            state.pace = PaceId::Steady;
                        } else {
                            state.pace = PaceId::Heated;
                            conservative_heat_days = conservative_heat_days.saturating_sub(1);
                        }
                    } else {
                        state.pace = PaceId::Steady;
                        if state.day > 60 && state.stats.hp > 4 && state.stats.sanity > 4 {
                            let travel_ratio = f64::from(state.travel_ratio_recent(10));
                            let days_survived = state.day.saturating_sub(1).max(1);
                            let avg_mpd =
                                f64::from(state.miles_traveled_actual) / f64::from(days_survived);
                            if travel_ratio < 0.90_f64 || avg_mpd < 11.5_f64 {
                                let severe = travel_ratio < 0.85_f64 || avg_mpd < 10.8_f64;
                                conservative_heat_days = if severe { 5 } else { 3 };
                                state.pace = PaceId::Heated;
                            }
                        }
                    }
                }
                self.conservative_heat_days = conservative_heat_days;
            }
        }
        if matches!(strategy, GameplayStrategy::ResourceManager) {
            let rest_ready = {
                let state = self.session.state();
                state.stats.pants >= 65 && state.camp.rest_cooldown == 0
            };
            if rest_ready {
                self.session.state_mut().day_state.rest.rest_requested = true;
            }
        }
    }
}

const fn clamp_choice_index(index: usize, encounter: &dystrail_game::data::Encounter) -> usize {
    if encounter.choices.is_empty() {
        0
    } else if index >= encounter.choices.len() {
        encounter.choices.len() - 1
    } else {
        index
    }
}

impl SimulationSession {}
