//! Presentation copy only; never consumes simulation randomness or changes the hearing.
use crate::{
    app::visual_content,
    game::{
        GameState,
        boss::{HearingOutcome, HearingPhase},
    },
    i18n,
};

pub(super) fn unit(state: &GameState) -> Option<String> {
    (state.continuity.visual_content.edition == visual_content::EDITION).then(|| {
        visual_content::service_unit(
            state,
            if state.mode.is_deep() {
                "HEARING-D"
            } else {
                "HEARING-C"
            },
            "hearing",
            0,
        )
    })
}

pub(super) fn copy(state: &GameState, field: &str) -> Option<String> {
    unit(state).map(|unit| i18n::t(&format!("encounter_copy.{unit}.{field}")))
}

pub(super) fn narration(state: &GameState) -> Option<String> {
    let field = match state.boss.presentation {
        HearingPhase::Preparation => "desc",
        HearingPhase::RoundRolling(0) => "round_1",
        HearingPhase::RoundRolling(1) => "round_2",
        HearingPhase::RoundRolling(2) => "round_3",
        // Secured victories have no final vote; do not narrate a vote that never happens.
        HearingPhase::Closed
            if state
                .boss
                .hearing
                .as_ref()
                .is_some_and(|r| r.vote_roll.is_some()) =>
        {
            "before_vote"
        }
        HearingPhase::Verdict | HearingPhase::Complete
            if state
                .boss
                .hearing
                .as_ref()
                .is_some_and(|r| r.outcome == HearingOutcome::Exhausted) =>
        {
            "exhausted"
        }
        _ => return None,
    };
    copy(state, field)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mode_and_saved_variant_control_copy_without_mutation() {
        for mode in [crate::game::GameMode::Classic, crate::game::GameMode::Deep] {
            for suffix in ["A", "B", "C"] {
                let mut state = GameState::default();
                state.mode = mode;
                state.continuity.visual_content.edition = visual_content::EDITION;
                let family = if mode.is_deep() {
                    "HEARING-D"
                } else {
                    "HEARING-C"
                };
                let selected = format!("{family}-{suffix}");
                state
                    .continuity
                    .visual_content
                    .selections
                    .insert(format!("{family}/hearing/0"), selected.clone());
                for phase in [
                    HearingPhase::Preparation,
                    HearingPhase::RoundRolling(0),
                    HearingPhase::RoundRolling(1),
                    HearingPhase::RoundRolling(2),
                ] {
                    state.boss.presentation = phase;
                    let before = serde_json::to_value(&state).unwrap();
                    assert_eq!(unit(&state), Some(selected.clone()));
                    let text = narration(&state).unwrap();
                    assert!(!text.contains("encounter_copy."));
                    assert_ne!(text, "Deep End only");
                    assert_eq!(serde_json::to_value(&state).unwrap(), before);
                }
                state.boss.presentation = HearingPhase::RoundResult(0);
                assert_eq!(narration(&state), None);
                state.boss.presentation = HearingPhase::Closed;
                assert_eq!(narration(&state), None);
            }
        }
    }
}
