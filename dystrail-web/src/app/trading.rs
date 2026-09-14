//! A town trading screen exposes all exchanges and records the selected offer.
use super::{Phase, aftermath::Aftermath, state::AppState};
use crate::{
    components::ui::{
        action_button::ActionButton, journey_scene::SceneStage, world_view::WorldView,
    },
    i18n,
};
use yew::prelude::*;
#[must_use]
pub fn toggle(app: &AppState, open: bool) -> Callback<MouseEvent> {
    let app = app.clone();
    Callback::from(move |_| {
        if let Some(mut session) = (*app.session).clone() {
            session.with_state_mut(|gs| gs.continuity.route_services.trading = open);
            app.session.set(Some(session));
        }
    })
}
pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let used = gs.continuity.route_services.traded_at == gs.continuity.route_services.stop;
    html! {<>
        <WorldView state={std::rc::Rc::new(gs.clone())} title={i18n::t("journey.trade_open")} stage={Some(SceneStage::Town)} />
        <section class="town-trading" aria-label={i18n::t("journey.trade_open")}><p>{i18n::t(if used {"play2.traded"}else{"journey.trade_help"})}</p>
            <div class="action-grid">{for (0..3_u8).map(|kind|html!{<div class="action-option"><p>{super::workshop::text(super::workshop::trade(kind),"setup")}</p><ActionButton label={i18n::t(&format!("journey.trade_label_{kind}"))} detail={i18n::t(&format!("journey.trade_{kind}"))} disabled={!gs.can_route_trade(kind)} onclick={choose(app,kind)} /></div>})}</div>
            <button onclick={toggle(app,false)}>{i18n::t("trail.listen")}</button>
        </section>
    </>}
}
fn choose(app: &AppState, kind: u8) -> Callback<MouseEvent> {
    let app = app.clone();
    Callback::from(move |_| {
        let Some(mut session) = (*app.session).clone() else {
            return;
        };
        let before = session.state().clone();
        if !session.with_state_mut(|gs| gs.trade_route_offer(kind)) {
            return;
        }
        let mut report = Aftermath {
            title: i18n::t("play2.exchange"),
            message: super::workshop::text(super::workshop::trade(kind), "outcome"),
            scene: SceneStage::Town,
            before: before.stats.clone(),
            after: session.state().stats.clone(),
            next: Phase::Town,
            resources: Vec::new(),
            details: super::receipt::resource_details(&before, session.state()),
        };
        session.with_state_mut(|gs| super::history::record(&before, gs, &mut report, 30));
        super::history::publish(&app, report, false);
        app.session.set(Some(session));
    })
}
