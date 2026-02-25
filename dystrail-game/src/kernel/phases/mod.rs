use crate::journey::JourneySession;

use super::{KernelTickInput, KernelTickOutput};

/// Execute one kernel day tick through the current phase pipeline.
#[must_use]
pub fn tick_day(session: &mut JourneySession, input: KernelTickInput) -> KernelTickOutput {
    session.state_mut().intent.pending = input.intent;
    session.tick_day().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endgame::EndgameTravelCfg;
    use crate::journey::{MechanicalPolicyId, StrategyId};
    use crate::state::{DayIntent, GameMode};

    #[test]
    fn tick_day_applies_intent_before_running_pipeline() {
        let data = crate::EncounterData::empty();
        let endgame = EndgameTravelCfg::default_config();
        let mut session = JourneySession::new_with_mechanics(
            MechanicalPolicyId::OtDeluxe90s,
            GameMode::Classic,
            StrategyId::Balanced,
            17,
            data,
            &endgame,
            None,
        );

        let _ = tick_day(
            &mut session,
            KernelTickInput {
                intent: DayIntent::Continue,
            },
        );

        assert_eq!(session.state().intent.pending, DayIntent::Continue);
    }
}
