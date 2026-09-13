use crate::app::{aftermath::Aftermath, state::AppState};

use crate::i18n;
use yew::prelude::*;

pub fn render_aftermath(state: &AppState, feedback: &Aftermath) -> Html {
    let Some(session) = state.session.as_ref() else {
        return Html::default();
    };
    let gs = session.state();
    let ally_notice = crate::app::ally_loss::is_notice(state);
    let on_continue = {
        let state = state.clone();
        let next = feedback.next;
        Callback::from(move |_| {
            if ally_notice && let Some(mut session) = (*state.session).clone() {
                session.with_state_mut(|gs| gs.continuity.ally_notice = None);
                state.session.set(Some(session));
            }
            state.aftermath.set(None);
            *state.action_lock.borrow_mut() = false;
            state.phase.set(next);
        })
    };
    html! { <div class="outcome-screen">
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={feedback.title.clone()} stage={Some(feedback.scene.clone())} />
        <section class={classes!("aftermath-panel",ally_notice.then_some("ally-departure"))} aria-labelledby="aftermath-title" aria-live="polite">
            <div class="outcome-summary">
            <h2 id="aftermath-title" class="eyebrow">{i18n::t(if ally_notice {"ally_loss.notice"}else{"ux.outcome"})}</h2>
            <p class={classes!("outcome-copy",ally_notice.then_some("ally-message"))}>{&feedback.message}</p>
            <dl class="receipt-details">{for feedback.details.iter().filter(|(name,_)|crate::app::receipt::narrative_detail(name)).map(|(name,value)|html!{<div><dt>{name}</dt><dd>{value}</dd></div>})}</dl>
            if feedback.changes().is_empty() && feedback.resources.is_empty() && feedback.details.is_empty() {<p>{i18n::t("ux.no_change")}</p>}
            </div>
            {crate::components::ui::stat_card::render_changes(&feedback.before, &feedback.after, &feedback.resources)}
        </section>
        <div class="outcome-actions"><button id="outcome-continue" class="retro-btn-primary" onclick={on_continue}>
            {crate::components::ui::journey_icon::render("return")}
            <span>{i18n::t(match feedback.next {
                crate::app::phase::Phase::Town => "trail.listen",
                crate::app::phase::Phase::Travel => "ux.back_road",
                _ => "ux.continue",
            })}</span>
        </button></div>
    </div> }
}
