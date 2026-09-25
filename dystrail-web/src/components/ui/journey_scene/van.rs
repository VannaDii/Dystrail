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
        <svg class="van-body" width="100%" height="100%" viewBox="0 0 1536 1024" aria-hidden="true" focusable="false" style="mask-image:none">
            <defs><mask id="van-body-outline" maskUnits="userSpaceOnUse" x="0" y="0" width="1536" height="1024">
                // Follow this source's own silhouette; the old occupied-van alpha
                // is wider and reveals the empty source's baked checkerboard.
                <path fill="white" stroke="black" stroke-width="6" d="M109 744V727L137 710V696H128V630H139V590L241 382L240 365L253 349L288 331H337V316H327L316 303V266L325 249H369V209L377 191L399 176H487V160L504 147H553L570 161L579 147L593 131H687V116L703 99H749L770 120V129H832L849 147L858 160L871 145H892V136H944L962 152V175H1008L1025 192L1039 209V249H1067L1084 265V301L1070 316H1059V331H1119L1146 340L1165 361V382L1305 558L1377 600L1386 625V641H1392V714H1388V740L1407 755V792L1403 804L1384 821L1362 828L1341 844H1246C1238 900 1200 936 1140 936C1080 936 1040 900 1033 848H584C577 904 536 937 477 937C418 937 379 904 370 849H351L338 860H265L241 853L222 831H187L161 820L127 802L110 787Z"/>
            </mask></defs>
            <image href={crate::paths::asset_path("static/img/journey/van-empty.png")} width="1536" height="1024" mask="url(#van-body-outline)"/>
        </svg>
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
