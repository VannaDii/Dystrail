use crate::endgame::EndgameTravelCfg;
use crate::journey::{JourneyController, PolicyId, StrategyId, apply_daily_effect};
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
        let controller = Self::build_controller(mode, strategy, seed, endgame_cfg);
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
        let controller = Self::build_controller(mode, strategy, seed, endgame_cfg);
        let mut session = Self { controller, state };
        session.reset_state_policy(strategy);
        session
    }

    fn build_controller(
        mode: GameMode,
        strategy: StrategyId,
        seed: u64,
        endgame_cfg: &EndgameTravelCfg,
    ) -> JourneyController {
        let mut controller = JourneyController::new(PolicyId::from(mode), strategy, seed);
        controller.set_endgame_config(endgame_cfg.clone());
        controller
    }

    fn reset_state_policy(&mut self, strategy: StrategyId) {
        self.state.policy = Some(strategy.into());
        self.state.attach_rng_bundle(self.controller.rng_bundle());
    }

    /// Advance the simulation by one day, returning the resulting outcome.
    pub fn tick_day(&mut self) -> DayOutcome {
        let cfg = self.controller.config();
        apply_daily_effect(&cfg.daily, &mut self.state);
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
