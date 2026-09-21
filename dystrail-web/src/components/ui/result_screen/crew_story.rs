use super::super::character_portrait::{self, Expression};
use crate::{
    game::{GameState, party::MemberStatus},
    i18n,
};
use yew::prelude::*;

pub fn render(gs: &GameState) -> Html {
    html! {<section class="crew-epilogue" aria-labelledby="crew-epilogue-title">
        <p class="eyebrow">{i18n::t("journey.your_story")}</p><h2 id="crew-epilogue-title">{&gs.party.name}</h2>
        <p>{format!("{} · {} · {}",crate::game::route::origin(gs.persona_id.as_deref().unwrap_or("staffer")),super::super::route_map::location::location(gs),super::super::route_map::location::distance(crate::game::route::physical_miles(gs)))}</p>
        <ul class="ending-crew">{for gs.party.members.iter().map(|m|{
            let key=match m.status {MemberStatus::Dead=>"crew.dead",MemberStatus::Departed=>"crew.departed",MemberStatus::Active=>if gs.continuity.crew_care.strain.get(&m.persona).copied().unwrap_or(0)>0{"journey.struggling"}else{"journey.present"}};
            let expression=if m.status==MemberStatus::Active {Expression::ending(gs)} else {Expression::Standard};
            html!{<li data-fate={key} data-member={m.persona.clone()}><span class="ending-portrait">{character_portrait::art(&m.persona,expression)}</span><div><strong>{&m.name}</strong><span>{i18n::t(key)}</span></div></li>}
        })}</ul>
        <h3>{i18n::t("journey.turning_points")}</h3><ol class="ending-moments">{for gs.continuity.journal.iter().rev().filter(|e|e.title!=i18n::t("play.last_turn")).take(4).map(|entry|html!{<li><strong>{format!("{} · {}",entry.day,entry.title)}</strong><p>{moment_message(gs,entry)}</p></li>})}</ol>
    </section>}
}

fn moment_message(gs: &GameState, entry: &crate::game::journal::JournalEntry) -> String {
    let player = gs
        .party
        .members
        .iter()
        .find(|m| Some(m.persona.as_str()) == gs.persona_id.as_deref());
    // Old saves used the surviving-crew message for the player's own fatal care decision.
    if gs.ending
        == Some(crate::game::Ending::Collapse {
            cause: crate::game::CollapseCause::Disease,
        })
        && gs.continuity.journal.last() == Some(entry)
        && let Some(player) = player
        && player.status == MemberStatus::Dead
        && entry.title.ends_with(&player.name)
    {
        return i18n::tr(
            "journey.player_lost",
            Some(&std::collections::BTreeMap::from([(
                "name",
                player.name.as_str(),
            )])),
        );
    }
    entry.message.clone()
}
