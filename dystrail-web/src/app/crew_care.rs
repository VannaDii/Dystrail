//! A named crew interruption, with costs and future consequences visible before choosing.
use super::{aftermath::Aftermath, state::AppState};
use crate::components::ui::action_button::ActionButton;
use crate::{components::ui::journey_scene::SceneStage, i18n};
use yew::prelude::*;

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let Some(persona) = &gs.continuity.crew_care.pending else {
        return Html::default();
    };
    let Some(member) = gs.party.members.iter().find(|m| &m.persona == persona) else {
        return Html::default();
    };
    let strain = gs
        .continuity
        .crew_care
        .strain
        .get(persona)
        .copied()
        .unwrap_or(1);
    let choice = |index| {
        let app = app.clone();
        Callback::from(move |_| {
            if *app.action_lock.borrow() {
                return;
            }
            let Some(mut session) = (*app.session).clone() else {
                return;
            };
            let before = session.state().clone();
            let person = before
                .continuity
                .crew_care
                .pending
                .as_ref()
                .and_then(|id| before.party.members.iter().find(|m| &m.persona == id));
            let name = person.map_or_else(String::new, |m| m.name.clone());
            session.with_state_mut(|gs| {
                let Some(key) = gs.resolve_crew_care(index) else {
                    return;
                };
                let mut report = Aftermath {
                    title: format!("{} · {name}", i18n::t("journey.crew_stop")),
                    message: super::workshop::named(
                        &super::workshop::care(before.continuity.crew_care.reason),
                        super::workshop::care_outcome(key),
                        &name,
                    ),
                    scene: SceneStage::Care,
                    before: before.stats.clone(),
                    after: gs.stats.clone(),
                    resources: Vec::new(),
                    details: super::receipt::resource_details(&before, gs),
                    next: super::aftermath::next_phase(gs),
                };
                super::history::record(&before, gs, &mut report, 60);
                super::history::publish(&app, report, true);
            });
            app.session.set(Some(session));
        })
    };
    html! {<>
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={format!("{} · {}",member.name,i18n::t("journey.crew_stop"))} stage={Some(SceneStage::Care)} />
        <section class="crew-incident" aria-label={i18n::t("journey.crew_stop")}><p class="scene-narrative">{super::workshop::named(&super::workshop::care(gs.continuity.crew_care.reason),if strain>1 {"continuing"} else {"setup"},&member.name)}{" "}{if strain>=3 {super::workshop::named(&super::workshop::care(gs.continuity.crew_care.reason),"critical",&member.name)} else {String::new()}} {crate::components::ui::satire_context::hook(&super::workshop::care(gs.continuity.crew_care.reason))}</p>
            <div class="camp-actions"><ActionButton duration={Some(i18n::t("trail.one_hour"))} onclick={choice(0)} disabled={gs.stats.supplies<2} label={i18n::tr("journey.care_action",Some(&std::collections::BTreeMap::from([("morale",(10-gs.stats.morale).clamp(0,1).to_string().as_str())])))} />
            <ActionButton duration={Some(i18n::t("trail.one_hour"))} onclick={choice(1)} disabled={gs.persona_id.as_ref()==Some(persona)} label={i18n::tr("journey.shelter_action",Some(&std::collections::BTreeMap::from([("morale",gs.stats.morale.clamp(0,1).to_string().as_str())])))} />
            <ActionButton duration={Some(i18n::t("trail.one_hour"))} onclick={choice(2)} label={i18n::tr(if strain>=3{"journey.fatal_action"}else{"journey.defer_action"},Some(&std::collections::BTreeMap::from([("sanity",gs.stats.sanity.clamp(0,1).to_string().as_str()),("morale",gs.stats.morale.clamp(0,3).to_string().as_str())])))} /></div>
        </section>
    </>}
}
