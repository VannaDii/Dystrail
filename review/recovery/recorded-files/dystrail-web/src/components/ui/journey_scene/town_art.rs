//! Source-authored town settings with the actual surviving crew composed in-scene.
use super::{Props, SceneStage};
use crate::game::party::{MemberStatus, Party};
use yew::prelude::*;

pub fn unit(stage: &SceneStage) -> Option<&str> {
    match stage {
        SceneStage::TownConversation { unit }
            if matches!(unit.as_str(), "TOWN-41-A" | "TOWN-41-B" | "TOWN-41-C") => Some(unit),
        _ => None,
    }
}

pub fn indoors(stage: &SceneStage) -> bool {
    unit(stage) == Some("TOWN-41-B")
}

fn travelers<'a>(party: Option<&'a Party>, preferred: Option<&str>) -> Vec<&'a str> {
    let Some(party) = party else { return Vec::new(); };
    let mut members: Vec<_> = party.members.iter()
        .filter(|m| m.status == MemberStatus::Active).collect();
    members.sort_by_key(|m| preferred != Some(m.persona.as_str()));
    members.into_iter().take(2).map(|m| m.persona.as_str()).collect()
}

pub fn render(p: &Props) -> Option<Html> {
    let unit = unit(&p.stage)?;
    let variant = unit.chars().last()?.to_ascii_lowercase();
    let href = crate::paths::asset_path(&format!("static/img/scenes-v2/town-41-{variant}-20260915.png"));
    let standing = variant == 'a';
    let crew = travelers(p.party.as_ref(), p.subject.as_deref());
    let foreground = if variant == 'b' { 726 } else { 710 };
    // Only crop/layout existing artwork. The foreground is the unchanged table
    // from the same image, covering the seated travelers' lower torsos naturally.
    let foreground_id = format!("town-table-{unit}");
    Some(html! {<svg class="scene-background town-setting" data-town-setting={unit.to_owned()}
        viewBox="0 0 1536 1024" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
        <image href={href.clone()} width="1536" height="1024" preserveAspectRatio="none" />
        {for crew.iter().enumerate().map(|(index,role)| {
            let index = if crew.len() == 1 { 1 } else { index };
            let (x,y,width,height) = if standing {
                (if index == 0 {80} else {450},140,475,760)
            } else if variant == 'b' {
                (if index == 0 {270} else {655},420,350,327)
            } else {
                (if index == 0 {30} else {485},285,450,420)
            };
            let pose = if standing { crate::components::ui::cast_art::Pose::Standing }
                else { crate::components::ui::cast_art::Pose::Standard };
            html! {<svg class="town-traveler" data-member={(*role).to_owned()} data-posture={if standing {"standing"} else {"seated"}}
                x={x.to_string()} y={y.to_string()} width={width.to_string()} height={height.to_string()}
                viewBox={crate::components::ui::cast_art::view_box(pose)} overflow="hidden">
                <image href={crate::paths::asset_path(&crate::components::ui::cast_art::path(role))} width="2048" height="1024" preserveAspectRatio="none" />
            </svg>}
        })}
        if !standing {
            <defs><clipPath id={foreground_id.clone()}><rect x="0" y={foreground.to_string()} width="1536" height={(1024-foreground).to_string()} /></clipPath></defs>
            <image href={href} width="1536" height="1024" preserveAspectRatio="none" clip-path={format!("url(#{foreground_id})")} />
        }
    </svg>})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_active_travelers_are_cast_and_preferred_actor_is_not_resurrected() {
        let mut gs = crate::game::GameState::default();
        gs.party = crate::game::party::Party::default();
        // Real fixture includes all crew roles and the same serialized member states.
        let fixture: serde_json::Value = serde_json::from_str(include_str!("../../../../../review/art-satire/client-fixtures/road.json")).unwrap();
        let mut party: Party = serde_json::from_value(fixture["state"]["party"].clone()).unwrap();
        assert_eq!(travelers(Some(&party), Some("journalist"))[0], "journalist");
        for member in &mut party.members {
            if member.persona == "journalist" {member.status = MemberStatus::Dead;}
            else if member.persona != "organizer" {member.status = MemberStatus::Departed;}
        }
        assert_eq!(travelers(Some(&party), Some("journalist")), ["organizer"]);
        for member in &mut party.members {member.status = MemberStatus::Dead;}
        assert!(travelers(Some(&party), None).is_empty());
        assert!(travelers(None, None).is_empty());
    }
}
