//! Reviewed town conversations with actual travelers seated behind the native table.
use super::{Props, SceneStage};
use crate::{components::ui::cast_art, game::party::MemberStatus};
use yew::prelude::*;

pub fn shared_setting(unit: &str) -> bool {
    matches!(unit, "TOWN-02-B" | "TOWN-17-C" | "TOWN-18-A" | "TOWN-19-C" |
        "TOWN-20-C" | "TOWN-23-B" | "TOWN-25-A" | "TOWN-26-C" | "TOWN-36-A")
}

pub fn aspect(stage: &SceneStage) -> Option<&'static str> {
    context(stage).map(|(unit, _)| if shared_setting(unit) { "1" } else { "1.5" })
}

pub fn context(stage: &SceneStage) -> Option<(&str, bool)> {
    let SceneStage::Encounter(unit) = stage else { return None; };
    match unit.as_str() {
        "TOWN-41-B" => Some((unit, true)),
        "TOWN-41-C" => Some((unit, false)),
        unit if shared_setting(unit) => Some((unit, true)),
        _ => None,
    }
}

pub fn render(p: &Props) -> Option<Html> {
    let (unit, indoors) = context(&p.stage)?;
    if shared_setting(unit) {
        return Some(html! {
            <svg class="scene-background town-setting" data-town-unit={unit.to_owned()} data-town-context="cafe"
                viewBox="512 0 512 512" preserveAspectRatio="xMidYMid meet">
                <image href={crate::paths::asset_path("static/img/journey/encounter-settings-v3.png")} width="1536" height="1024"/>
            </svg>
        });
    }
    let path = crate::paths::asset_path(&format!("static/img/scenes-v2/{}.png", unit.to_ascii_lowercase()));
    let (xs, top, width, table) = if indoors { ([325, 690], 410, 350, 728) } else { ([20, 500], 320, 440, 710) };
    let foreground = if indoors {
        format!("M0 {table} H1536 V1024 H0 Z")
    } else {
        // Follow the raised bread and fruit as well as the tabletop so the crew
        // cannot paint over objects that sit in front of them.
        "M0 710 H425 L444 699 L469 687 L506 682 L547 689 L580 699 L600 710 H673 L682 696 L699 683 L723 677 L729 658 L748 662 L758 678 L773 668 L792 664 L811 675 L830 697 L833 710 H1536 V1024 H0 Z".to_owned()
    };
    let clip = format!("town-table-{unit}");
    let members: Vec<_> = p.party.iter().flat_map(|party| &party.members)
        .filter(|member| member.status == MemberStatus::Active).take(2).collect();
    Some(html! {
        <svg class="scene-background town-setting" data-town-unit={unit.to_owned()} viewBox="0 0 1536 1024" preserveAspectRatio="xMidYMid meet">
            <image href={path.clone()} width="1536" height="1024"/>
            {for members.iter().zip(xs).map(|(member,x)| html! {
                <svg data-seated-traveler={member.persona.clone()} x={x.to_string()} y={top.to_string()}
                    width={width.to_string()} height={(width as f64 * 470.0 / 512.0).to_string()}
                    viewBox={cast_art::view_box(cast_art::Pose::Standard)} overflow="hidden">
                    <image href={crate::paths::asset_path(&cast_art::path(&member.persona))} width="2048" height="1024"/>
                </svg>
            })}
            <defs><clipPath id={clip.clone()}><path d={foreground}/></clipPath></defs>
            <image href={path} width="1536" height="1024" clip-path={format!("url(#{clip})")}/>
        </svg>
    })
}
