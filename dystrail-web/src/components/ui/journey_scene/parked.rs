//! Standing poses use the same retained identities and the actual surviving party.
use crate::{
    components::ui::cast_art::{self, Pose},
    game::party::{MemberStatus, Party},
};
use yew::prelude::*;

pub fn render(party: Option<&Party>) -> Html {
    let Some(party) = party else {
        return Html::default();
    };
    html! {<div class="parked-crew">
        <super::van::CrewVan party={Some(party.clone())} empty={true} />
        <div class="standing-crew">{for party.members.iter()
            .filter(|member| member.status == MemberStatus::Active)
            .map(|member| html! {<div class="standing-member" data-member={member.persona.clone()}>
                {cast_art::art(&member.persona, Pose::Standing)}
            </div>})}</div>
    </div>}
}
