use crate::endgame::EndgameTravelCfg;
use crate::journey::{JourneyController, MechanicalPolicyId, PolicyId, StrategyId};
use crate::mechanics::OtDeluxeOccupation;
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
        Self::new_with_mechanics(
            MechanicalPolicyId::DystrailLegacy,
            mode,
            strategy,
            seed,
            data,
            endgame_cfg,
            None,
        )
    }

    /// Construct a fresh session from seed, mode, strategy, and encounter data.
    #[must_use]
    pub fn new_with_mechanics(
        mechanics: MechanicalPolicyId,
        mode: GameMode,
        strategy: StrategyId,
        seed: u64,
        data: EncounterData,
        endgame_cfg: &EndgameTravelCfg,
        otdeluxe_occupation: Option<OtDeluxeOccupation>,
    ) -> Self {
        let state = GameState::default().with_seed(seed, mode, data);
        let controller = Self::build_controller(mechanics, mode, strategy, seed, endgame_cfg);
        let mut session = Self { controller, state };
        session.reset_state_policy();
        if mechanics == MechanicalPolicyId::OtDeluxe90s {
            let occupation = otdeluxe_occupation.unwrap_or(OtDeluxeOccupation::Banker);
            session.state.apply_otdeluxe_start_config(occupation);
        }
        session.state.queue_otdeluxe_store_if_available();
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
        let controller =
            Self::build_controller(state.mechanical_policy, mode, strategy, seed, endgame_cfg);
        let mut session = Self { controller, state };
        session.reset_state_policy();
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

    fn reset_state_policy(&mut self) {
        let rng_bundle = self
            .state
            .rng_bundle
            .clone()
            .unwrap_or_else(|| self.controller.rng_bundle());
        self.controller.set_rng_bundle(rng_bundle);
        self.controller.configure_state(&mut self.state);
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

    /// Borrow the underlying mutable game state.
    pub const fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
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
    use crate::mechanics::OtDeluxeOccupation;
    use crate::state::PolicyKind;

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
    fn session_construction_supports_otdeluxe_mechanics() {
        let data = EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let session = JourneySession::new_with_mechanics(
            MechanicalPolicyId::OtDeluxe90s,
            GameMode::Classic,
            StrategyId::Balanced,
            7,
            data,
            &endgame,
            Some(OtDeluxeOccupation::Doctor),
        );
        assert_eq!(
            session.state().mechanical_policy,
            MechanicalPolicyId::OtDeluxe90s
        );
        assert_eq!(
            session.state().ot_deluxe.mods.occupation,
            Some(OtDeluxeOccupation::Doctor)
        );
        assert_eq!(session.state().ot_deluxe.inventory.cash_cents, 120_000);
    }

    #[test]
    fn session_accessors_expose_state_and_controller() {
        let data = EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let mut session =
            JourneySession::new(GameMode::Classic, StrategyId::Balanced, 11, data, &endgame);

        session.state_mut().day = 2;
        assert_eq!(session.state().day, 2);
        assert_eq!(session.controller().strategy(), StrategyId::Balanced);
    }
}
