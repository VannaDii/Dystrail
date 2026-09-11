use crate::app::{aftermath::Aftermath, state::AppState};

use crate::i18n;
use yew::prelude::*;

pub fn render_aftermath(state: &AppState, feedback: &Aftermath) -> Html {
    let Some(session) = state.session.as_ref() else {
        return Html::default();
    };
    let gs = session.state();
    let on_continue = {
        let state = state.clone();
        let next = feedback.next;
        Callback::from(move |_| {
            state.aftermath.set(None);
            *state.action_lock.borrow_mut() = false;
            state.phase.set(next);
        })
    };
    html! { <>
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={feedback.title.clone()} stage={Some(feedback.scene.clone())} />
        <section class="aftermath-panel" aria-labelledby="aftermath-title" aria-live="polite">
            <p class="eyebrow">{i18n::t(if feedback.changes().iter().any(|(_,_,bad)|*bad){"journey.consequence"}else{"journey.relief"})}</p>
            <h2 id="aftermath-title">{i18n::t("ux.outcome")}</h2>
            <p class="outcome-copy">{&feedback.message}</p>
            <ul class="resource-changes" aria-label={i18n::t("ux.outcome")}>
                {for feedback.changes().iter().map(|(key,delta,harmful)| html!{
                    <li class={if *harmful {"change harmful"} else {"change helpful"}}>
                        <span>{i18n::t(key)}</span><strong>{format!("{delta:+}")}</strong>
                    </li>
                })}
            </ul>
            <dl class="receipt-details">{for feedback.details.iter().filter(|(name,_)|name!=&i18n::t("journey.when")).map(|(name,value)|html!{<div><dt>{name}</dt><dd>{value}</dd></div>})}</dl>
            if feedback.changes().is_empty() && feedback.details.is_empty() {<p>{i18n::t("ux.no_change")}</p>}
            <button class="retro-btn-primary" onclick={on_continue}>{i18n::t(if feedback.next == crate::app::phase::Phase::Travel {"ux.back_road"} else {"ux.continue"})}</button>
        </section>
    </> }
}
