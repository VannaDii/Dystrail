use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::outfitting::OutfittingPage;
use yew::prelude::*;

pub fn render_outfitting(state: &AppState) -> Html {
    let mut current_state = (*state.pending_state).clone().unwrap_or_default();
    current_state.stats.supplies = 0;
    current_state.inventory = crate::game::state::Inventory::default();
    let on_continue = {
        let app = state.clone();
        let before = current_state.clone();
        let session = state.session.clone();
        let code = state.code.clone();
        let data = state.data.clone();
        let endgame = (*state.endgame_config).clone();
        let seed_handle = state.run_seed.clone();
        let pending_handle = state.pending_state.clone();
        let phase_handle = state.phase.clone();
        Callback::from(
            move |(new_state, _grants, _tags): (
                crate::game::GameState,
                crate::game::store::Grants,
                Vec<String>,
            )| {
                if let Some((deep, seed)) = crate::game::seed::decode_to_seed(&code) {
                    let mode = if deep {
                        crate::game::GameMode::Deep
                    } else {
                        crate::game::GameMode::Classic
                    };
                    let mut initialized = new_state.with_seed(seed, mode, (*data).clone());
                    initialized.continuity.visual_content.edition =
                        crate::app::visual_content::EDITION;
                    let mut report = crate::app::aftermath::Aftermath {
                        title: crate::i18n::t("play.loadout"),
                        message: crate::i18n::t("journey.mission"),
                        scene: crate::components::ui::journey_scene::SceneStage::Travel(
                            initialized.region,
                        ),
                        before: before.stats.clone(),
                        after: initialized.stats.clone(),
                        resources: Vec::new(),
                        details: crate::app::receipt::resource_details(&before, &initialized),
                        next: Phase::Travel,
                    };
                    crate::app::history::record(&before, &mut initialized, &mut report, 0);
                    crate::app::history::publish(&app, report, false);
                    app.travel_running.set(false);
                    let run = crate::app::phase::session_from_state(initialized, &endgame);
                    pending_handle.set(Some(run.state().clone()));
                    seed_handle.set(seed);
                    session.set(Some(run));
                    phase_handle.set(Phase::Travel);
                }
            },
        )
    };
    html! {
        <OutfittingPage game_state={current_state} {on_continue} />
    }
}
