use super::{phase::session_from_state, state::AppState};
use crate::{
    game::{GameState, store::Grants},
    i18n,
};
use yew::prelude::*;
pub fn render_stop(state: &AppState) -> Html {
    let Some(gs) = state
        .session
        .as_ref()
        .map(crate::game::JourneySession::state)
    else {
        return Html::default();
    };
    let Some(mile) = gs.continuity.route_services.stop else {
        return Html::default();
    };
    let open = {
        let open = state.town_open.clone();
        Callback::from(move |_| open.set(true))
    };
    let trade = super::trading::toggle(state, true);
    html! {<section class="route-stop"><div><p class="eyebrow">{format!("{} · {}",i18n::t("play2.route_stop"),i18n::fmt_number(f64::from(mile)))}</p><h2>{crate::game::route::settlement(gs,mile).map_or_else(||i18n::t("play2.town"),|s|s.name.clone())}</h2>{super::town_profile::render(gs)}</div><div class="controls">{super::town::talk_button(state,gs)}<button onclick={trade}>{i18n::t("journey.trade_open")}</button><button onclick={open}>{i18n::t("play2.resupply")}</button></div></section>}
}
pub fn render_shop(state: &AppState) -> Html {
    let Some(gs) = state.session.as_ref().map(|s| s.state().clone()) else {
        return Html::default();
    };
    let close = {
        let open = state.town_open.clone();
        Callback::from(move |()| open.set(false))
    };
    let checkout = {
        let state = state.clone();
        Callback::from(move |(mut gs, _, _): (GameState, Grants, Vec<String>)| {
            if let Some(before) = state
                .session
                .as_ref()
                .map(crate::game::JourneySession::state)
            {
                let mut report = super::aftermath::Aftermath {
                    title: i18n::t("play2.purchase"),
                    message: i18n::t("play2.purchase_done"),
                    scene: crate::components::ui::journey_scene::SceneStage::Town,
                    before: before.stats.clone(),
                    after: gs.stats.clone(),
                    next: super::Phase::Town,
                    resources: Vec::new(),
                    details: super::receipt::resource_details(before, &gs),
                };
                super::history::record(before, &mut gs, &mut report, 30);
                super::history::publish(&state, report, false);
            }
            state
                .session
                .set(Some(session_from_state(gs, &state.endgame_config)));
            state.town_open.set(false);
        })
    };
    html! {<><crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={super::town::name(&gs)} stage={Some(crate::components::ui::journey_scene::SceneStage::Town)} /><crate::components::ui::outfitting_store::OutfittingStore game_state={gs} resupply={true} on_continue={checkout} on_close={close} /></>}
}
