use dystrail_game::activities::Activity;
use dystrail_game::boss::{self, BossConfig, BossOutcome};
use dystrail_game::camp::{self, CampConfig};
use dystrail_game::data::EncounterData;
use dystrail_game::endgame::EndgameTravelCfg;
use dystrail_game::{
    GameMode, GameState, JourneyController, MechanicalPolicyId, PaceId, PolicyId, StrategyId,
};

use crate::logic::policy::{GameplayStrategy, PlayerPolicy, PolicyDecision};

use dystrail_game::pacing::PacingConfig;
use dystrail_game::repairs::RepairChoice;

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
    state: GameState,
    pacing_config: PacingConfig,
    camp_config: CampConfig,
    boss_config: BossConfig,
    max_days: u32,
    strategy: GameplayStrategy,
    conservative_heat_days: u32,
    aggressive_heat_days: u32,
    controller: JourneyController,
}

impl SimulationSession {
    pub fn new(
        config: SimulationConfig,
        encounters: EncounterData,
        pacing_config: PacingConfig,
        camp_config: CampConfig,
        endgame_config: EndgameTravelCfg,
        boss_config: BossConfig,
    ) -> Self {
        let mut state = GameState::default().with_seed(config.seed, config.mode, encounters);
        let strategy_id = strategy_id_for(config.strategy);
        let mut controller = JourneyController::new(
            MechanicalPolicyId::DystrailLegacy,
            PolicyId::from(config.mode),
            strategy_id,
            config.seed,
        );
        controller.set_endgame_config(endgame_config);
        controller.configure_state(&mut state);
        state.sync_route_location();
        state.policy = Some(strategy_id.into());
        state.continuity.interactive_repairs = true;
        state.attach_rng_bundle(controller.rng_bundle());
        Self {
            state,
            pacing_config,
            camp_config,
            boss_config,
            max_days: config.max_days,
            strategy: config.strategy,
            conservative_heat_days: 0,
            aggressive_heat_days: 0,
            controller,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &GameState {
        &self.state
    }

    #[must_use]
    pub const fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    #[must_use]
    pub fn into_state(self) -> GameState {
        self.state
    }

    pub fn advance(
        &mut self,
        policy: &mut dyn PlayerPolicy,
        visit_stop: impl FnOnce(&mut GameState),
    ) -> TurnOutcome {
        // Follow the browser's decision priority before spending time on the road.
        if self.has_ended() {
            return self.finished_turn(None);
        }
        self.resolve_crew_care();
        if self.has_ended() {
            return self.finished_turn(None);
        }
        let decision = self.resolve_encounter(policy);
        if self.has_ended() {
            return self.finished_turn(decision);
        }
        self.resolve_repair();
        visit_stop(&mut self.state);
        self.gather_before_departure();

        if self.state.boss.readiness.ready
            && !self.state.boss.outcome.attempted
            && self.state.camp.rest_cooldown == 0
            && !self.state.day_state.rest.rest_requested
        {
            self.state.day_state.rest.rest_requested = true;
        }

        if let Some(message) = self.camp_before_travel() {
            return TurnOutcome {
                day: self.state.day,
                travel_message: message,
                breakdown_started: false,
                game_ended: self.has_ended(),
                decision,
                miles_traveled_actual: self.state.miles_traveled_actual,
            };
        }

        self.adjust_daily_pace();
        if !self.state.day_state.lifecycle.day_initialized {
            self.state.apply_pace_and_diet(&self.pacing_config);
        }

        let before = self.state.clone();
        let outcome = self.controller.tick_day(&mut self.state);
        if self.state.day > before.day
            || self.state.miles_traveled_actual > before.miles_traveled_actual
        {
            self.state
                .update_route_services(before.miles_traveled_actual);
        }
        self.state.check_crew(before.day);
        let mut game_ended = outcome.ended;
        let mut travel_message = outcome.log_key.clone();
        let breakdown_started = outcome.breakdown_started;
        if !game_ended && self.state.day >= self.max_days {
            game_ended = true;
            travel_message = String::from("Max days reached");
        }

        if self.state.boss.readiness.ready && !self.state.boss.outcome.attempted {
            let boss_cfg = self.boss_config.clone();
            let before = self.state.clone();
            let outcome = boss::run_boss_minigame(self.state_mut(), &boss_cfg);
            self.state.advance_clock(&before, 120);
            game_ended = true;
            travel_message = match outcome {
                BossOutcome::PassedCloture => String::from("log.boss.victory"),
                BossOutcome::SurvivedFlood => String::from("log.boss.failure"),
                BossOutcome::Exhausted => String::from("log.sanity-collapse"),
            };
            self.state.boss.readiness.ready = false;
        }

        TurnOutcome {
            day: self.state.day,
            travel_message,
            breakdown_started,
            game_ended,
            decision,
            miles_traveled_actual: self.state.miles_traveled_actual,
        }
    }

    const fn has_ended(&self) -> bool {
        self.state.ending.is_some()
            || self.state.continuity.abandoned
            || self.state.boss.outcome.attempted
            || self.state.day >= self.max_days
    }

    fn finished_turn(&self, decision: Option<DecisionRecord>) -> TurnOutcome {
        TurnOutcome {
            day: self.state.day,
            travel_message: String::from("Journey ended"),
            breakdown_started: false,
            game_ended: true,
            decision,
            miles_traveled_actual: self.state.miles_traveled_actual,
        }
    }

    fn camp_before_travel(&mut self) -> Option<String> {
        let camp_cfg = self.camp_config.clone();
        let before = self.state.clone();
        if self.state.stats.supplies <= 2 && self.state.can_activity(Activity::Forage) {
            let outcome = camp::camp_forage(self.state_mut(), &camp_cfg);
            return Some(outcome.message);
        }
        if self.state.day_state.rest.rest_requested || self.state.should_auto_rest() {
            self.state.day_state.rest.rest_requested = false;
            let outcome = camp::camp_rest(self.state_mut(), &camp_cfg);
            if outcome.rested {
                self.state.advance_clock(&before, 0);
                return Some(outcome.message);
            }
        }
        None
    }

    fn resolve_encounter(&mut self, policy: &mut dyn PlayerPolicy) -> Option<DecisionRecord> {
        let encounter = self.state.current_encounter.clone()?;
        let PolicyDecision {
            choice_index,
            rationale,
        } = policy.pick_choice(&self.state, &encounter);
        let safe_index = clamp_choice_index(choice_index, &encounter);
        let choice_label = encounter.choices.get(safe_index).map_or_else(
            || "No available choice".to_string(),
            |choice| choice.label.clone(),
        );
        let decision = DecisionRecord {
            day: self.state.day,
            encounter_id: encounter.id,
            encounter_name: encounter.name,
            choice_index: safe_index,
            choice_label,
            policy_name: policy.name().to_string(),
            rationale,
        };
        self.state.resolve_encounter_choice(safe_index);
        Some(decision)
    }

    fn resolve_crew_care(&mut self) {
        if self.state.continuity.crew_care.pending.is_none() {
            return;
        }
        let before = self.state.clone();
        let choice = if self.state.stats.supplies >= 2 { 0 } else { 2 };
        if self.state.resolve_crew_care(choice).is_some() {
            self.state.advance_clock(&before, 60);
        }
    }

    fn resolve_repair(&mut self) {
        let choice = RepairChoice::ALL
            .into_iter()
            .find(|c| self.state.can_repair(*c));
        if let Some(choice) = choice {
            let before = self.state.clone();
            if self.state.choose_repair(choice) {
                self.state.advance_clock(&before, choice.minutes());
            }
        }
    }

    fn gather_before_departure(&mut self) {
        if self.state.stats.supplies > 10 {
            return;
        }
        let action = if self.state.stats.supplies <= 4
            && self.state.stats.hp >= 8
            && self.state.stats.sanity >= 7
        {
            Activity::Glean
        } else {
            Activity::Forage
        };
        let _ = self.state.perform_activity(action);
    }

    fn adjust_daily_pace(&mut self) {
        let state = &mut self.state;
        match self.strategy {
            GameplayStrategy::Balanced | GameplayStrategy::ResourceManager => {
                let healthy = state.stats.hp >= 8 && state.stats.sanity >= 7;
                let supplies_ok = state.stats.supplies >= 6;
                let illness_active = state.illness_travel_penalty < 0.99;
                if healthy && supplies_ok && !illness_active {
                    if matches!(state.pace, PaceId::Steady) {
                        state.pace = PaceId::Heated;
                    }
                } else if state.stats.hp <= 5 || state.stats.sanity <= 5 || illness_active {
                    state.pace = PaceId::Steady;
                }
            }
            GameplayStrategy::Aggressive => {
                if state.stats.hp <= 4 || state.stats.sanity <= 4 {
                    state.pace = PaceId::Steady;
                    self.aggressive_heat_days = 0;
                } else {
                    if state.mode.is_deep() && self.aggressive_heat_days == 0 {
                        let ratio_10 = state.travel_ratio_recent(10);
                        if ratio_10 < 0.85 {
                            self.aggressive_heat_days = 3;
                        }
                    }
                    state.pace = PaceId::Heated;
                    if self.aggressive_heat_days > 0 {
                        self.aggressive_heat_days = self.aggressive_heat_days.saturating_sub(1);
                    }
                }
            }
            GameplayStrategy::Conservative => {
                if self.conservative_heat_days > 0 {
                    if state.stats.hp <= 4 || state.stats.sanity <= 4 {
                        self.conservative_heat_days = 0;
                        state.pace = PaceId::Steady;
                    } else {
                        state.pace = PaceId::Heated;
                        self.conservative_heat_days = self.conservative_heat_days.saturating_sub(1);
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
                            self.conservative_heat_days = if severe { 5 } else { 3 };
                            state.pace = PaceId::Heated;
                        }
                    }
                }
            }
        }
        if matches!(self.strategy, GameplayStrategy::ResourceManager)
            && state.stats.sanity <= 3
            && state.camp.rest_cooldown == 0
        {
            state.day_state.rest.rest_requested = true;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::game_tester::PlayabilityMetrics;
    use dystrail_game::travel_time::{TRAVEL_DAY_END, TRAVEL_DAY_MINUTES, TRAVEL_DAY_START};

    fn session() -> SimulationSession {
        SimulationSession::new(
            SimulationConfig::new(GameMode::Classic, GameplayStrategy::Balanced, 42),
            EncounterData::empty(),
            PacingConfig::default(),
            CampConfig::default_config(),
            EndgameTravelCfg::default_config(),
            BossConfig::load_from_static(),
        )
    }

    fn hearing_session(clock: u16) -> SimulationSession {
        let mut session = SimulationSession::new(
            SimulationConfig::new(GameMode::Deep, GameplayStrategy::ResourceManager, 1570),
            EncounterData::empty(),
            PacingConfig::default(),
            CampConfig::default_config(),
            EndgameTravelCfg::default_config(),
            BossConfig::load_from_static(),
        );
        session.state.diet = dystrail_game::DietId::Quiet;
        session.state.stats.allies = 0;
        session.state.disease_cooldown = 100;
        session.state.exec_order_cooldown = 100;
        session.state.weather_state.today = dystrail_game::Weather::Clear;
        session.state.weather_state.neutral_buffer = 100;
        session.state.continuity.crew_care.last_check_day = u32::MAX;
        session.state.apply_pace_and_diet(&session.pacing_config);
        session.state.continuity.clock_minutes = clock;
        session.state.stats.hp = 10;
        session.state.stats.sanity = 10;
        session.state.stats.supplies = 20;
        session.state.camp.rest_cooldown = 2;
        session.state.day_state.rest.rest_requested = false;
        session.state.boss.readiness.ready = true;
        session.state.boss.readiness.reached = true;
        session
    }

    #[test]
    fn final_hearing_spends_two_hours_once_and_records_every_stationary_day() {
        for (clock, elapsed_days, final_clock, stationary_days) in [
            (TRAVEL_DAY_START, 0, TRAVEL_DAY_START + 120, 1),
            (TRAVEL_DAY_END - 120, 0, TRAVEL_DAY_END, 1),
            (TRAVEL_DAY_END - 60, 1, TRAVEL_DAY_START + 60, 2),
            (TRAVEL_DAY_END, 1, TRAVEL_DAY_START + 120, 2),
        ] {
            let mut session = hearing_session(clock);
            let before = session.state.clone();
            let mut policy = GameplayStrategy::ResourceManager.create_policy(1570);
            let outcome = session.advance(policy.as_mut(), |_| {});
            assert!(outcome.game_ended);
            assert!(session.state.boss.outcome.attempted);
            assert_eq!(session.state.day, before.day + elapsed_days);
            assert_eq!(session.state.continuity.clock_minutes, final_clock);
            assert_eq!(
                elapsed_days * u32::from(TRAVEL_DAY_MINUTES) + u32::from(final_clock)
                    - u32::from(clock),
                120
            );
            assert_eq!(
                session.state.miles_traveled_actual.to_bits(),
                before.miles_traveled_actual.to_bits()
            );
            assert_eq!(session.state.continuity.driving_minutes_total, 0);
            let mut metrics = PlayabilityMetrics::default();
            metrics.finalize(&session.state, &outcome);
            assert_eq!(metrics.non_travel_days, stationary_days);
            assert_eq!(metrics.travel_days, 0);
            assert_eq!(metrics.partial_travel_days, 0);
            assert_eq!(metrics.miles_traveled.to_bits(), 0.0_f32.to_bits());
            assert_eq!(metrics.days_with_camp, 0);
            assert_eq!(metrics.days_with_repair, 0);

            let finished = serde_json::to_value(&session.state).unwrap();
            for reload in [false, true] {
                if reload {
                    session.state = serde_json::from_value(finished.clone()).unwrap();
                }
                let repeated = session.advance(policy.as_mut(), |_| {
                    panic!("a completed hearing cannot perform another action");
                });
                assert!(repeated.game_ended);
                assert_eq!(serde_json::to_value(&session.state).unwrap(), finished);
            }

            // Deferred daily costs cannot decide whether the elapsed day exists.
            session.state.apply_pace_and_diet(&session.pacing_config);
            let mut initialized_metrics = PlayabilityMetrics::default();
            initialized_metrics.finalize(&session.state, &outcome);
            assert_eq!(initialized_metrics.non_travel_days, metrics.non_travel_days);
            assert_eq!(initialized_metrics.travel_days, metrics.travel_days);
            assert_eq!(
                initialized_metrics.partial_travel_days,
                metrics.partial_travel_days
            );
            assert_eq!(
                initialized_metrics.miles_traveled.to_bits(),
                metrics.miles_traveled.to_bits()
            );
        }
    }

    #[test]
    fn gathering_uses_the_players_cost_and_persisted_cooldown() {
        let mut session = session();
        session.state.stats.supplies = 5;
        session.state.stats.sanity = 6;
        let before = session.state.clone();
        session.gather_before_departure();
        assert_eq!(session.state.stats.supplies, 7);
        assert_eq!(session.state.stats.sanity, 7);
        assert_eq!(session.state.day, before.day);
        assert_eq!(
            session.state.continuity.clock_minutes,
            before.continuity.clock_minutes + Activity::Forage.minutes()
        );
        session.state =
            serde_json::from_str(&serde_json::to_string(&session.state).unwrap()).unwrap();
        session.gather_before_departure();
        assert_eq!(session.state.stats.supplies, 7);
        assert_eq!(
            session.state.continuity.clock_minutes,
            before.continuity.clock_minutes + Activity::Forage.minutes()
        );
    }

    #[test]
    fn town_and_route_preparation_match_the_configured_journey() {
        let mut session = session();
        assert!(
            (session.state.trail_distance - session.controller.config().victory_miles).abs()
                < f32::EPSILON
        );
        session.state.continuity.route_services.stop = Some(100);
        session.state.stats.supplies = 4;
        session.gather_before_departure();
        assert_eq!(
            session.state.stats.supplies, 4,
            "roadside gathering is unavailable in town"
        );
    }

    #[test]
    fn roadside_repairs_use_owned_parts_then_affordable_player_options() {
        use dystrail_game::vehicle::{Breakdown, Part};
        let mut session = session();
        session.state.inventory.spares.battery = 1;
        session.state.breakdown = Some(Breakdown {
            part: Part::Battery,
            day_started: 1,
        });
        session.state.vehicle.health = 80.0;
        let before = session.state.clone();
        session.resolve_repair();
        assert!(session.state.breakdown.is_none());
        assert_eq!(session.state.inventory.spares.battery, 0);
        assert_eq!(session.state.budget_cents, before.budget_cents);
        assert!((session.state.vehicle.health - 88.0).abs() < f32::EPSILON);
        assert_eq!(
            session.state.continuity.clock_minutes,
            before.continuity.clock_minutes + 60
        );

        session.state.breakdown = Some(Breakdown {
            part: Part::FuelPump,
            day_started: 1,
        });
        session.state.budget_cents = 0;
        session.state.stats.supplies = 0;
        let before = session.state.clone();
        session.resolve_repair();
        assert!(session.state.breakdown.is_none());
        assert_eq!(session.state.stats.sanity, before.stats.sanity - 2);
        assert_eq!(session.state.stats.morale, before.stats.morale - 1);
        assert_eq!(
            session.state.continuity.clock_minutes,
            before.continuity.clock_minutes + 240
        );
    }

    #[test]
    fn care_spends_supplies_and_a_fatal_deferral_stops_all_later_actions() {
        let mut session = session();
        session.state.persona_id = Some("journalist".into());
        session.state.party.initialize("journalist", 42);
        session.state.continuity.crew_care.pending = Some("journalist".into());
        let before = session.state.clone();
        session.resolve_crew_care();
        assert_eq!(session.state.stats.supplies, before.stats.supplies - 2);
        assert!(session.state.continuity.crew_care.pending.is_none());
        assert_eq!(
            session.state.continuity.clock_minutes,
            before.continuity.clock_minutes + 60
        );

        session.state.stats.supplies = 0;
        session.state.continuity.crew_care.pending = Some("journalist".into());
        session
            .state
            .continuity
            .crew_care
            .strain
            .insert("journalist".into(), 3);
        let before = session.state.clone();
        let outcome = session.advance(
            GameplayStrategy::Balanced.create_policy(42).as_mut(),
            |_| {
                panic!("a terminal decision cannot visit a shop or gather supplies");
            },
        );
        assert!(outcome.game_ended);
        assert!(session.state.ending.is_some());
        assert_eq!(session.state.day, before.day);
        assert!(
            (session.state.miles_traveled_actual - before.miles_traveled_actual).abs()
                < f32::EPSILON
        );
        assert_eq!(session.state.stats.supplies, 0);
        assert_eq!(
            session.state.continuity.clock_minutes,
            before.continuity.clock_minutes + 60
        );
    }

    #[test]
    fn a_completed_camp_day_resets_the_clock_without_driving() {
        let mut session = session();
        session.state.continuity.clock_minutes = 19 * 60;
        session.state.stats.sanity = 3;
        session.state.day_state.rest.rest_requested = true;
        let before = session.state.clone();
        assert!(session.camp_before_travel().is_some());
        assert_eq!(session.state.day, before.day + 1);
        assert_eq!(session.state.continuity.clock_minutes, 8 * 60);
        assert!(
            (session.state.miles_traveled_actual - before.miles_traveled_actual).abs()
                < f32::EPSILON
        );
    }
}
