//! Editable naming step; every keystroke enters the existing recovery checkpoint.
use crate::{
    app::{phase::Phase, state::AppState},
    game::party::Party,
    i18n,
};
use yew::prelude::*;
pub fn render_crew(state: &AppState) -> Html {
    let Some(gs) = state.pending_state.as_ref() else {
        return Html::default();
    };
    let update = {
        let pending = state.pending_state.clone();
        Callback::from(move |party: Party| {
            if let Some(mut gs) = (*pending).clone() {
                gs.party = party;
                pending.set(Some(gs));
            }
        })
    };
    let done = {
        let phase = state.phase.clone();
        Callback::from(move |_| phase.set(Phase::Outfitting))
    };
    html! {<CrewNames party={gs.party.clone()} player={gs.persona_id.clone().unwrap_or_else(||"journalist".into())} on_change={update} on_continue={done}/>}
}
#[derive(Properties, PartialEq)]
struct Props {
    party: Party,
    player: String,
    on_change: Callback<Party>,
    on_continue: Callback<MouseEvent>,
}
#[function_component(CrewNames)]
fn crew_names(p: &Props) -> Html {
    let ready = !p.party.name.trim().is_empty()
        && p.party.members.iter().all(|m| !m.name.trim().is_empty());
    let crew_change = {
        let party = p.party.clone();
        let on = p.on_change.clone();
        Callback::from(move |event: InputEvent| {
            let mut party = party.clone();
            party.name = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            on.emit(party);
        })
    };
    html! {<section class="crew-onboarding" aria-labelledby="crew-title">
        <div class="crew-intro"><p class="eyebrow">{i18n::t("app.title")}</p><h1 id="crew-title">{i18n::t("crew.title")}</h1><p>{i18n::t("crew.hint")}</p></div>
        <div class="crew-name-form"><label for="crew-name">{i18n::t("crew.name")}</label><input id="crew-name" type="text" value={p.party.name.clone()} oninput={crew_change} autocomplete="off" />
        <div class="crew-names-grid">{for p.party.members.iter().filter(|m|m.persona==p.player).chain(p.party.members.iter().filter(|m|m.persona!=p.player)).map(|member|{
            let persona=member.persona.clone();let player=p.player.clone();let party=p.party.clone();let on=p.on_change.clone();
            let change=Callback::from(move |event:InputEvent|{let mut party=party.clone();if let Some(m)=party.members.iter_mut().find(|m|m.persona==persona){m.name=event.target_unchecked_into::<web_sys::HtmlInputElement>().value();}party.sync_names(&player);on.emit(party);});
            let id=format!("crew-name-{}",member.persona);
            html!{<div class="crew-name-card"><img class="crew-portrait" src={crate::paths::asset_path(&format!("static/img/journey/occupant-{}.png",member.persona))} alt="" /><label for={id.clone()}>{if member.persona==p.player{i18n::t("crew.player")}else{i18n::t(&format!("persona.{}.name",member.persona))}}</label><input {id} type="text" value={member.name.clone()} oninput={change} autocomplete="off" /></div>}
        })}</div><button disabled={!ready} onclick={p.on_continue.clone()}>{i18n::t("ui.continue")}</button></div>
    </section>}
}
