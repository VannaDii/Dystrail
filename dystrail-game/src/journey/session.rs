use crate::endgame::EndgameTravelCfg;
use crate::journey::{JourneyController, MechanicalPolicyId, PolicyId, StrategyId};
use crate::state::GameState;
use crate::{DayOutcome, EncounterData, GameMode};

/// High-level session wrapper binding a journey controller to a mutable game state.
#[derive(Debug, Clone)]
pub struct JourneySession {
    controller: JourneyController,
    state: GameState,
}

impl JourneySession {
    /// Construct a fresh session from seed, mode, strategy, and encounter data.
    #[must_use]
    pub fn new(
        mode: GameMode,
        strategy: StrategyId,
        seed: u64,
        data: EncounterData,
        endgame_cfg: &EndgameTravelCfg,
    ) -> Self {
        let state = GameState::default().with_seed(seed, mode, data);
        let controller = Self::build_controller(
            MechanicalPolicyId::DystrailLegacy,
            mode,
            strategy,
            seed,
            endgame_cfg,
        );
        let mut session = Self { controller, state };
        session.reset_state_policy(strategy);
        session
    }

    /// Build a session from an existing game state.
    #[must_use]
    pub fn from_state(
        state: GameState,
        strategy: StrategyId,
        endgame_cfg: &EndgameTravelCfg,
    ) -> Self {
        let mode = state.mode;
        let seed = state.seed;
        let mut controller =
            Self::build_controller(state.mechanical_policy, mode, strategy, seed, endgame_cfg);
        if let Some(rng) = state.rng_bundle.as_ref() {
            controller.rng = rng.clone();
        }
        let mut session = Self { controller, state };
        session.reset_state_policy(strategy);
        session
    }

    fn build_controller(
        mechanics: MechanicalPolicyId,
        mode: GameMode,
        strategy: StrategyId,
        seed: u64,
        endgame_cfg: &EndgameTravelCfg,
    ) -> JourneyController {
        let mut controller =
            JourneyController::new(mechanics, PolicyId::from(mode), strategy, seed);
        controller.set_endgame_config(endgame_cfg.clone());
        controller
    }

    fn reset_state_policy(&mut self, strategy: StrategyId) {
        self.controller.configure_state(&mut self.state);
        self.state.policy = Some(strategy.into());
        // Route position and the first travel tick must use the same distance scale.
        self.state.trail_distance = self.controller.config().victory_miles;
        self.state.sync_route_location();
        self.state.attach_rng_bundle(self.controller.rng_bundle());
    }

    /// Advance the simulation by one day, returning the resulting outcome.
    pub fn tick_day(&mut self) -> DayOutcome {
        self.controller.tick_day(&mut self.state)
    }

    /// Current strategy assigned to the session.
    #[must_use]
    pub const fn strategy(&self) -> StrategyId {
        self.controller.strategy()
    }

    /// Current policy family.
    #[must_use]
    pub const fn policy(&self) -> PolicyId {
        self.controller.policy()
    }

    /// Borrow the underlying immutable game state.
    #[must_use]
    pub const fn state(&self) -> &GameState {
        &self.state
    }

    /// Apply a closure to the mutable game state.
    pub fn with_state_mut<R>(&mut self, f: impl FnOnce(&mut GameState) -> R) -> R {
        f(&mut self.state)
    }

    /// Borrow the controller.
    #[must_use]
    pub const fn controller(&self) -> &JourneyController {
        &self.controller
    }

    /// Deterministically reseed the session.
    pub fn reseed(&mut self, seed: u64) {
        self.controller.reseed(seed);
        self.state.seed = seed;
        self.state.attach_rng_bundle(self.controller.rng_bundle());
    }

    /// Consume the session, returning the underlying game state.
    #[must_use]
    pub fn into_state(self) -> GameState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PolicyKind;

    #[test]
    fn hourly_pace_caps_are_consistent_for_every_route_and_strategy() {
        let pacing = crate::PacingConfig::default_config();
        for mode in [GameMode::Classic, GameMode::Deep] {
            for strategy in [
                StrategyId::Balanced,
                StrategyId::Aggressive,
                StrategyId::Conservative,
                StrategyId::ResourceManager,
            ] {
                for route in crate::route::routes() {
                    for (pace, expected) in [
                        (crate::PaceId::Steady, 60.0),
                        (crate::PaceId::Heated, 70.0),
                        (crate::PaceId::Blitz, 80.0),
                    ] {
                        let mut session = JourneySession::new(
                            mode,
                            strategy,
                            42,
                            EncounterData::empty(),
                            &EndgameTravelCfg::default_config(),
                        );
                        session.state.persona_id = Some(route.id.clone());
                        session.state.continuity.route_services.route_id = Some(route.id.clone());
                        session.state.pace = pace;
                        session.state.day_state.lifecycle.day_initialized = true;
                        session.state.apply_pace_and_diet(&pacing);
                        let miles =
                            crate::route::physical_at(&session.state, session.state.distance_today);
                        assert!(
                            (miles - expected).abs() < 0.05,
                            "{mode:?} {strategy:?} {} {pace:?}: {miles}",
                            route.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn daily_costs_apply_once_when_pace_initializes_the_day_and_after_reload() {
        let mut session = JourneySession::new(
            GameMode::Classic,
            StrategyId::Balanced,
            42,
            EncounterData::empty(),
            &EndgameTravelCfg::default_config(),
        );
        session.controller.cfg.daily.health.decay = 1.0;
        session.controller.cfg.breakdown.base = 0.0;
        session.controller.configure_state(&mut session.state);
        session.state.weather_state.neutral_buffer = 100;
        session.state.disease_cooldown = 100;
        let pacing = crate::pacing::PacingConfig::default_config();
        session.state.apply_pace_and_diet(&pacing);
        assert_eq!(session.state.stats.hp, 9);
        session.state.apply_pace_and_diet(&pacing);
        assert_eq!(session.state.stats.hp, 9);
        session.state =
            serde_json::from_str(&serde_json::to_string(&session.state).unwrap()).unwrap();
        while session.state.day == 1 {
            let _ = session.tick_day();
        }
        assert_eq!(session.state.stats.hp, 9);
        session.state.apply_pace_and_diet(&pacing);
        assert_eq!(session.state.stats.hp, 8);
    }

    #[test]
    fn session_construction_sets_policy_and_state() {
        let data = EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let seed = 4242;
        let mut session = JourneySession::new(
            GameMode::Classic,
            StrategyId::Balanced,
            seed,
            data,
            &endgame,
        );

        assert_eq!(session.strategy(), StrategyId::Balanced);
        assert_eq!(session.policy(), PolicyId::Classic);
        assert_eq!(session.state().seed, seed);
        assert_eq!(session.state().policy, Some(PolicyKind::Balanced));

        session.with_state_mut(|state| state.day_state.rest.rest_requested = true);
        assert!(session.state().day_state.rest.rest_requested);

        session.reseed(99);
        assert_eq!(session.state().seed, 99);
        assert_eq!(session.policy(), PolicyId::Classic);
    }

    #[test]
    fn session_from_state_resets_policy_and_ticks() {
        let data = EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let state = GameState::default().with_seed(7, GameMode::Deep, data);

        let mut session = JourneySession::from_state(state, StrategyId::Aggressive, &endgame);
        assert_eq!(session.policy(), PolicyId::Deep);
        assert_eq!(session.strategy(), StrategyId::Aggressive);
        assert_eq!(session.state().policy, Some(PolicyKind::Aggressive));

        // Ensure tick_day exercises daily application without panicking.
        let _ = session.tick_day();
    }

    #[test]
    fn route_scale_is_ready_before_departure_and_stable_after_reload() {
        let endgame = EndgameTravelCfg::default_config();
        let mut session = JourneySession::new(
            GameMode::Classic,
            StrategyId::Balanced,
            42,
            EncounterData::empty(),
            &endgame,
        );
        let expected = session.controller().config().victory_miles;
        assert!((session.state().trail_distance - expected).abs() < f32::EPSILON);
        session.with_state_mut(|gs| {
            gs.persona_id = Some("journalist".to_owned());
            gs.continuity.route_services.route_id = gs.persona_id.clone();
            gs.miles_traveled_actual = expected * 0.5;
            gs.sync_route_location();
        });
        let position = crate::route::physical_miles(session.state());
        let mut restored =
            JourneySession::from_state(session.into_state(), StrategyId::Balanced, &endgame);
        assert!((crate::route::physical_miles(restored.state()) - position).abs() < 0.01);
        let _ = restored.tick_day();
        assert!((restored.state().trail_distance - expected).abs() < f32::EPSILON);
        assert!(crate::route::physical_miles(restored.state()) >= position);
    }
}
