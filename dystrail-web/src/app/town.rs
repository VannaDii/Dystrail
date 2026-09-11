//! Arrival is an explicit stop with local services, never a footer on the road.
use super::{Phase, aftermath::Aftermath, state::AppState};
use crate::{
    components::ui::journey_scene::SceneStage,
    game::{GameState, route},
    i18n,
};
use yew::prelude::*;

fn talk(app: &AppState) -> Callback<MouseEvent> {
    let app = app.clone();
    Callback::from(move |_| {
        let Some(mut session) = (*app.session).clone() else {
            return;
        };
        let before = session.state().clone();
        let Some(stop) = before.continuity.route_services.stop else {
            return;
        };
        if before.continuity.route_services.talked_at == Some(stop) {
            return;
        }
        let Some(fact) = super::town_facts::fact(&before) else {
            return;
        };
        let message = fact.message();
        let npc = u8::try_from(js_sys::Math::random().to_bits() % 6).unwrap_or(0);
        session.with_state_mut(|gs| {
            gs.continuity.route_services.talked_at = Some(stop);
            gs.continuity.activities.local_word = Some(npc);
            gs.stats.credibility = (gs.stats.credibility + 1).min(20);
            let mut report = Aftermath {
                title: i18n::t("journey.local_word"),
                message,
                scene: SceneStage::Town,
                before: before.stats.clone(),
                after: gs.stats.clone(),
                details: super::receipt::resource_details(&before, gs),
                next: Phase::Town,
            };
            super::history::record(&before, gs, &mut report, 30);
            super::history::publish(&app, report, false);
        });
        app.session.set(Some(session));
    })
}

#[must_use]
pub fn name(gs: &GameState) -> String {
    gs.continuity
        .route_services
        .stop
        .and_then(|mile| route::settlement(gs, mile))
        .map_or_else(|| i18n::t("play2.town"), |s| s.name.clone())
}

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    if gs.continuity.activities.local_word.is_some() {
        return super::town_facts::render(app);
    }
    let depart = {
        let app = app.clone();
        Callback::from(move |_| {
            let Some(mut session) = (*app.session).clone() else {
                return;
            };
            session.with_state_mut(|gs| gs.continuity.route_services.stop = None);
            let next = super::map::after_transit(session.state(), Phase::Travel);
            app.session.set(Some(session));
            app.phase.set(next);
            app.travel_running
                .set(next == Phase::Travel && *app.travel_speed != super::flow::TravelSpeed::Step);
        })
    };
    html! {<>
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={name(gs)} stage={Some(SceneStage::Town)} />
        <section class="town-arrival" aria-label={i18n::t("journey.arrival")}><p class="eyebrow">{i18n::t("journey.arrival")}</p><p>{i18n::t("journey.arrival_help")}</p>
            {super::services::render_stop(app)}
            <div class="town-actions"><button onclick={talk(app)} disabled={gs.continuity.route_services.talked_at==gs.continuity.route_services.stop}>{format!("{} · {} {:+}",i18n::t("journey.talk"),i18n::t("play.credibility"),(gs.stats.credibility+1).min(20)-gs.stats.credibility)}</button>
            <button onclick={{let app=app.clone();Callback::from(move |_|app.phase.set(Phase::Camp))}}>{i18n::t("ux.camp")}</button>
            <button class="retro-btn-primary" onclick={depart}>{i18n::t("journey.depart")}</button></div>
            {super::activities::render(app)}
            {app.last_turn.as_ref().map_or_else(Html::default,super::turn::render_last_turn)}
        </section>
    </>}
}
