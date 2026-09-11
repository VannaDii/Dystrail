//! A vehicle body and independently rendered crew, composed in three two-seat rows.
use crate::game::party::{CrewMember, MemberStatus, PERSONAS, Party};
use yew::prelude::*;
#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    #[prop_or_default]
    pub party: Option<Party>,
}
#[function_component(CrewVan)]
pub fn crew_van(p: &Props) -> Html {
    let preview = Party {
        members: PERSONAS
            .iter()
            .map(|id| CrewMember {
                persona: (*id).into(),
                name: String::new(),
                status: MemberStatus::Active,
            })
            .collect(),
        ..Party::default()
    };
    let party = p.party.as_ref().unwrap_or(&preview);
    let occupants = party.occupants();
    html! {<div class="crew-van" data-occupants={occupants.len().to_string()}>
        <img class="van-body" src={crate::paths::asset_path("static/img/journey/van-empty.png")} alt="" />
        {for (0..3).map(|row|html!{<div class={classes!("van-window",format!("van-row-{row}"))}>
            {for occupants.iter().filter(|(seat,_)|*seat/2==row).rev().map(|(seat,member)|{
                let persona=if PERSONAS.contains(&member.persona.as_str()){member.persona.as_str()}else{"staffer"};
                html!{<img class={classes!("van-occupant",if seat%2==0{"near-seat"}else{"far-seat"})} data-member={member.persona.clone()} data-seat={seat.to_string()} src={crate::paths::asset_path(&format!("static/img/journey/occupant-{persona}.png"))} alt="" />}
            })}
        </div>})}
    </div>}
}
