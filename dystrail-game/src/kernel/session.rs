use crate::endgame::EndgameTravelCfg;
use crate::journey::{JourneySession, MechanicalPolicyId, PolicyId, StrategyId};
use crate::mechanics::OtDeluxeOccupation;
use crate::state::GameMode;
use thiserror::Error;

use super::{KernelTickInput, KernelTickOutput};

/// Errors constructing or running the `OTDeluxe` kernel facade.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum KernelSessionError {
    #[error("OTDeluxe kernel session requires OTDeluxe mechanical policy")]
    NonOtDeluxePolicy,
}

/// `OTDeluxe` parity kernel session facade.
#[derive(Debug, Clone)]
pub struct KernelSession {
    inner: JourneySession,
}

impl KernelSession {
    /// Creates a new `OTDeluxe` kernel session.
    #[must_use]
    pub fn new(
        mode: GameMode,
        strategy: StrategyId,
        seed: u64,
        data: crate::EncounterData,
        endgame_cfg: &EndgameTravelCfg,
        occupation: Option<OtDeluxeOccupation>,
    ) -> Self {
        let inner = JourneySession::new_with_mechanics(
            MechanicalPolicyId::OtDeluxe90s,
            mode,
            strategy,
            seed,
            data,
            endgame_cfg,
            occupation,
        );
        Self { inner }
    }

    /// Creates a kernel session from an existing state.
    ///
    /// # Errors
    ///
    /// Returns an error when the provided state is not using `OTDeluxe90s` mechanics.
    pub fn from_state(
        state: crate::GameState,
        strategy: StrategyId,
        endgame_cfg: &EndgameTravelCfg,
    ) -> Result<Self, KernelSessionError> {
        if state.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return Err(KernelSessionError::NonOtDeluxePolicy);
        }
        Ok(Self {
            inner: JourneySession::from_state(state, strategy, endgame_cfg),
        })
    }

    /// Advances the simulation one day under the provided intent.
    pub fn tick_day(&mut self, input: KernelTickInput) -> KernelTickOutput {
        self.inner.state_mut().intent.pending = input.intent;
        self.inner.tick_day().into()
    }

    /// Returns the immutable game state.
    #[must_use]
    pub const fn state(&self) -> &crate::GameState {
        self.inner.state()
    }

    /// Consumes the kernel session and returns the inner game state.
    #[must_use]
    pub fn into_state(self) -> crate::GameState {
        self.inner.into_state()
    }

    /// Mechanical policy used by this session.
    #[must_use]
    pub const fn mechanics(&self) -> MechanicalPolicyId {
        self.inner.state().mechanical_policy
    }

    /// Policy family currently configured by the underlying controller.
    #[must_use]
    pub const fn policy(&self) -> PolicyId {
        self.inner.policy()
    }
}
