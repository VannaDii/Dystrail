//! Ending a run is explicit and reversible only through the player's separate save slot.
use super::{Phase, state::AppState};
use crate::{components::ui::journey_scene::SceneStage, i18n};
use yew::prelude::*;

pub fn render(app: &AppState) -> Html {
    let cancel = {
        let show = app.show_abandon.clone();
        Callback::from(move |()| show.set(false))
    };
    let confirm = {
        let app = app.clone();
        Callback::from(move |()| {
            let session = app
                .pending_turn
                .as_ref()
                .map(|p| p.session.clone())
                .or_else(|| (*app.session).clone());
            let Some(mut session) = session else {
                return;
            };
            let before = session.state().clone();
            session.with_state_mut(|gs| {
                gs.continuity.abandoned = true;
                let mut report = super::aftermath::Aftermath {
                    title: i18n::t("journey.abandoned"),
                    message: i18n::tr(
                        "journey.abandoned_story",
                        Some(&std::collections::BTreeMap::from([
                            ("name", gs.party.player_name(gs.persona_id.as_deref())),
                            ("crew", gs.party.name.as_str()),
                        ])),
                    ),
                    scene: SceneStage::Ending(false),
                    before: before.stats.clone(),
                    after: gs.stats.clone(),
                    details: super::receipt::resource_details(&before, gs),
                    next: Phase::Result,
                };
                super::history::record(&before, gs, &mut report, 0);
                super::history::publish(&app, report, false);
            });
            app.travel_running.set(false);
            app.pending_turn.set(None);
            app.aftermath.set(None);
            app.weather_notice.set(false);
            app.town_open.set(false);
            app.show_abandon.set(false);
            *app.action_lock.borrow_mut() = false;
            app.session.set(Some(session));
            app.phase.set(Phase::Result);
        })
    };
    html! {<AbandonDialog {cancel} {confirm}/>}
}

#[derive(Properties, PartialEq)]
struct Props {
    cancel: Callback<()>,
    confirm: Callback<()>,
}
#[function_component(AbandonDialog)]
fn abandon_dialog(p: &Props) -> Html {
    let card = use_node_ref();
    crate::components::ui::dismiss::use_outside_dismiss(card.clone(), true, p.cancel.clone());
    crate::components::ui::save_drawer::focus::use_focus_trap(
        true,
        Some("game-menu-button".into()),
        card.clone(),
    );
    let keydown = crate::components::ui::save_drawer::focus::focus_keydown_handler(
        card.clone(),
        p.cancel.clone(),
    );
    html! {<div class="abandon-backdrop"><section class="abandon-dialog" ref={card} onkeydown={keydown} role="dialog" aria-modal="true" aria-labelledby="abandon-title" aria-describedby="abandon-explanation">
        <h1 id="abandon-title">{i18n::t("journey.abandon_question")}</h1><p id="abandon-explanation">{i18n::t("journey.abandon_explanation")}</p>
        <div class="controls"><button class="retro-btn-primary" onclick={{let cb=p.cancel.clone();Callback::from(move |_|cb.emit(()))}}>{i18n::t("journey.keep_traveling")}</button>
        <button onclick={{let cb=p.confirm.clone();Callback::from(move |_|cb.emit(()))}}>{i18n::t("journey.confirm_abandon")}</button></div>
    </section></div>}
}
