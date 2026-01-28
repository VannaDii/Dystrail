use crate::app::phase::Phase;
use crate::app::state::AppState;
#[cfg(any(test, target_arch = "wasm32"))]
use crate::game::data::EncounterData;
#[cfg(any(test, target_arch = "wasm32"))]
use crate::game::seed::{decode_to_seed, generate_code_from_entropy};
use crate::game::state::GameMode;
use crate::pages::mode_select::ModeSelectPage;
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
fn next_entropy() -> u64 {
    js_sys::Date::now().to_bits()
}

#[cfg(any(test, target_arch = "wasm32"))]
struct ModeSelectionOutcome {
    state: crate::game::GameState,
    seed: u64,
    code: AttrValue,
    phase: Phase,
}

#[cfg(any(test, target_arch = "wasm32"))]
fn build_mode_selection_outcome(
    pending_state: Option<crate::game::GameState>,
    data: &EncounterData,
    mode: GameMode,
    entropy: u64,
) -> Option<ModeSelectionOutcome> {
    let is_deep = matches!(mode, GameMode::Deep);
    let code = generate_code_from_entropy(is_deep, entropy);
    let (decoded_deep, seed) = decode_to_seed(&code)?;
    let base = pending_state.unwrap_or_default();
    let gs = base.with_seed(
        seed,
        if decoded_deep {
            GameMode::Deep
        } else {
            GameMode::Classic
        },
        data.clone(),
    );
    Some(ModeSelectionOutcome {
        state: gs,
        seed,
        code: AttrValue::from(code),
        phase: Phase::Outfitting,
    })
}

pub fn render_mode_select(state: &AppState) -> Html {
    let on_back = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::Persona))
    };

    let on_continue = {
        let pending_handle = state.pending_state.clone();
        let data_handle = state.data.clone();
        let code_handle = state.code.clone();
        let run_seed_handle = state.run_seed.clone();
        let phase_handle = state.phase.clone();
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |mode: GameMode| {
                let entropy = next_entropy();
                if let Some(outcome) = build_mode_selection_outcome(
                    (*pending_handle).clone(),
                    &data_handle,
                    mode,
                    entropy,
                ) {
                    pending_handle.set(Some(outcome.state));
                    run_seed_handle.set(outcome.seed);
                    code_handle.set(outcome.code);
                    phase_handle.set(outcome.phase);
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (
                pending_handle,
                data_handle,
                code_handle,
                run_seed_handle,
                phase_handle,
            );
            Callback::from(|_mode: GameMode| {})
        }
    };

    html! { <ModeSelectPage {on_continue} {on_back} /> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_mode_selection_outcome_handles_classic_and_deep() {
        let data = EncounterData::empty();
        let base = crate::game::GameState::default();
        let classic = build_mode_selection_outcome(Some(base.clone()), &data, GameMode::Classic, 7);
        let Some(classic) = classic else {
            panic!("classic selection should produce a seed");
        };
        assert_eq!(classic.state.mode, GameMode::Classic);
        assert_eq!(classic.state.seed, classic.seed);
        assert_eq!(classic.phase, Phase::Outfitting);
        assert!(!classic.code.is_empty());

        let deep = build_mode_selection_outcome(Some(base), &data, GameMode::Deep, 9);
        let Some(deep) = deep else {
            panic!("deep selection should produce a seed");
        };
        assert_eq!(deep.state.mode, GameMode::Deep);
        assert_eq!(deep.state.seed, deep.seed);
        assert_eq!(deep.phase, Phase::Outfitting);
        assert!(!deep.code.is_empty());
    }
}
