//! Arrival is an explicit stop with local services, never a footer on the road.
use super::{Phase, aftermath::Aftermath, state::AppState};
use crate::components::ui::action_button::ActionButton;
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
        if before.continuity.route_services.stop.is_none() {
            return;
        }
        let Some(fact) = super::town_facts::fact(&before) else {
            return;
        };
        let message = fact.message();
        let npc = u8::try_from(js_sys::Math::random().to_bits() % 6).unwrap_or(0);
        session.with_state_mut(|gs| {
            super::town_content::seal(gs);
            gs.continuity.activities.local_word = Some(npc);
            if gs.claim_local_conversation().is_none() {
                return;
            }
            let mut report = Aftermath {
                title: i18n::t("journey.local_word"),
                message,
                scene: SceneStage::Town,
                before: before.stats.clone(),
                after: gs.stats.clone(),
                resources: Vec::new(),
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

#[must_use]
pub fn depart(app: &AppState) -> Callback<()> {
    let app = app.clone();
    Callback::from(move |()| {
        let Some(mut session) = (*app.session).clone() else {
            return;
        };
        session.with_state_mut(|gs| {
            gs.continuity.route_services.stop = None;
            gs.continuity.route_services.trading = false;
            gs.continuity.activities.local_word = None;
        });
        app.town_open.set(false);
        let next = super::map::after_transit(session.state(), Phase::Travel, *app.travel_speed);
        app.session.set(Some(session));
        app.phase.set(next);
        app.map_automatic.set(next == Phase::Map);
        app.travel_running
            .set(matches!(next, Phase::Travel | Phase::Map));
    })
}

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    if gs.continuity.route_services.trading {
        return super::trading::render(app);
    }
    if gs.continuity.activities.local_word.is_some() {
        return super::town_facts::render(app);
    }
    html! {<>
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={name(gs)} stage={Some(SceneStage::Town)} />
        <section class="town-arrival" aria-label={i18n::t("journey.arrival")}>
            <header class="town-heading"><div><p class="eyebrow">{i18n::t("journey.arrival")}</p><p>{i18n::t("journey.arrival_help")}</p></div>
            </header>
            {super::services::render_stop(app)}
            {super::activities::render(app)}
        </section>
    </>}
}

pub fn talk_button(app: &AppState, gs: &GameState) -> Html {
    let services = &gs.continuity.route_services;
    let claimed = services.stop.is_some() && services.talked_at == services.stop;
    let reward = if claimed {
        services.talk_reward
    } else {
        gs.local_conversation_reward()
    };
    let detail = reward.map_or_else(
        || i18n::t("journey.claimed"),
        |reward| {
            use crate::game::route_services::LocalReward;
            let key = match reward {
                LocalReward::Credibility => "play.credibility",
                LocalReward::Receipt => "ux.receipt",
                LocalReward::Ally => "play.allies",
            };
            if claimed {
                format!("{} +1", i18n::t(key))
            } else if matches!(reward, LocalReward::Receipt) {
                i18n::t("qualitative.evidence")
            } else {
                let stat = i18n::t(key);
                i18n::tr("qualitative.gain", Some(&std::collections::BTreeMap::from([("stat", stat.as_str())])))
            }
        },
    );
    html! {<ActionButton onclick={talk(app)} {claimed} label={i18n::t("journey.talk")} {detail} />}
}
