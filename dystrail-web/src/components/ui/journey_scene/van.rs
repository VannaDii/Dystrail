//! Vehicle and active crew share one coordinate system; no invented fallback occupants.
use crate::components::ui::cast_art::{self, Pose};
use crate::game::party::Party;
use yew::prelude::*;
#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    #[prop_or_default]
    pub party: Option<Party>,
    #[prop_or_default]
    pub empty: bool,
}
#[function_component(CrewVan)]
pub fn crew_van(p: &Props) -> Html {
    let occupants = if p.empty {
        Vec::new()
    } else {
        p.party.as_ref().map_or_else(Vec::new, Party::occupants)
    };
    html! {<div class="crew-van" data-occupants={occupants.len().to_string()}>
        <img class="van-body" src={crate::paths::asset_path("static/img/journey/van-empty.png")} alt="" decoding="sync" />
        <svg class="van-seating" viewBox="0 0 1536 1024" aria-hidden="true" focusable="false">
            <defs><clipPath id="crew-van-window-mask" clipPathUnits="userSpaceOnUse">
                <rect x="352" y="384" width="232" height="208" rx="12"/>
                <rect x="620" y="384" width="232" height="208" rx="12"/>
                <polygon points="904,384 1136,384 1268,568 1268,592 904,592"/>
            </clipPath></defs>
            <g clip-path="url(#crew-van-window-mask)">
            {for occupants.iter().map(|(seat, member)| {
                let (x, width, sprite_x, sprite_y, sprite_size, pose) = match seat {
                    0 => (352,116,-36,0,200,Pose::PassengerNear),
                    1 => (468,116,-64,-28,220,Pose::PassengerFar),
                    2 => (620,116,-36,0,200,Pose::PassengerNear),
                    3 => (736,116,-64,-28,220,Pose::PassengerFar),
                    4 => (1024,244,-40,-4,260,Pose::Driver),
                    5 => (904,120,-64,-28,220,Pose::PassengerFar),
                    _ => return Html::default(),
                };
                html! {<svg class={classes!("van-occupant",(*seat==4).then_some("driver-seat"))}
                    data-member={member.persona.clone()} data-seat={seat.to_string()} data-pose={pose.cell().to_string()}
                    x={x.to_string()} y="384" width={width.to_string()} height="208"
                    viewBox={format!("0 0 {width} 208")}>
                    <svg x={sprite_x.to_string()} y={sprite_y.to_string()} width={sprite_size.to_string()} height={sprite_size.to_string()} viewBox={cast_art::view_box(pose)}>
                        <image href={crate::paths::asset_path(&cast_art::path(&member.persona))} width="2048" height="1024" />
                    </svg>
                </svg>}
            })}
            </g>
        </svg>
    </div>}
}
