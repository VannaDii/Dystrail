//! Individually composed standing crew stay synchronized with the actual party.
use crate::game::party::{MemberStatus, Party};
use yew::prelude::*;

pub fn render(party: Option<&Party>) -> Html {
    let Some(party) = party else {
        return Html::default();
    };
    html! {<div class="parked-crew">
        <svg class="crew-color-key" aria-hidden="true" width="0" height="0"><defs>
            <filter id="standing-crew-key" color-interpolation-filters="sRGB">
                // Leave enough chroma-key margin for accelerated, fractional-size rendering.
                // The weaker key left translucent magenta rectangles in the in-app browser.
                <feColorMatrix type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  -4 8 -4 0 5" result="keyed"/>
                <feMorphology in="keyed" operator="erode" radius="2" result="silhouette"/>
                <feComposite in="keyed" in2="silhouette" operator="in"/>
            </filter>
        </defs></svg>
        <super::van::CrewVan party={Some(party.clone())} empty={true} />
        <div class="standing-crew">{for party.members.iter()
            .filter(|member| member.status == MemberStatus::Active)
            .map(|member| {
                let cell=match member.persona.as_str() {
                    "journalist"=>0,"organizer"=>1,"whistleblower"=>2,
                    "lobbyist"=>3,"satirist"=>5,_=>4,
                };
                let x=(cell%3)*512+96;
                let y=(cell/3)*512;
                html! {<svg class="standing-member" data-member={member.persona.clone()}
                    viewBox={format!("{x} {y} 320 512")} aria-hidden="true">
                    <image href={crate::paths::asset_path("static/img/journey/standing-crew-v1.png")} width="1536" height="1024" filter="url(#standing-crew-key)"/>
                </svg>}
            })}</div>
    </div>}
}
