//! Crew-free settings and independently cast foreground portraits prevent ghost crew.
use super::{Props, SceneStage};
use crate::game::{
    Region,
    party::{CrewMember, MemberStatus, Party},
};
use yew::prelude::*;

#[must_use]
pub fn subject<'a>(
    party: &'a Party,
    preferred: Option<&str>,
    day: u32,
    scene: &str,
) -> Option<&'a CrewMember> {
    let active: Vec<_> = party
        .members
        .iter()
        .filter(|m| m.status == MemberStatus::Active)
        .collect();
    if let Some(member) = active
        .iter()
        .find(|m| Some(m.persona.as_str()) == preferred)
    {
        return Some(member);
    }
    if active.is_empty() {
        return None;
    }
    let salt = scene
        .bytes()
        .fold(day as usize, |n, b| n.wrapping_add(usize::from(b)));
    active.get(salt % active.len()).copied()
}

pub fn setting(p: &Props, name: &str) -> Option<Html> {
    let (atlas, columns, rows, cell) = match &p.stage {
        SceneStage::Town => (
            "journey-settings-v1",
            2,
            3,
            match p.region {
                Some(Region::RustBelt) => 1,
                Some(Region::Beltway) => 2,
                _ => 0,
            },
        ),
        SceneStage::Camp | SceneStage::Care => ("journey-settings-v1", 2, 3, 3),
        SceneStage::Ending(arrived) => ("journey-settings-v1", 2, 3, if *arrived { 4 } else { 5 }),
        _ => {
            let cell = match name {
                "enc-media-workshop" => 0,
                "enc-community" => 1,
                "enc-bridge" => 2,
                "enc-service" => 3,
                "enc-civic" => 4,
                "enc-checkpoint" => 5,
                "enc-clinic" => 6,
                "enc-radio" => 7,
                "enc-street" => 8,
                "enc-convoy" => 9,
                "milk-classic" | "milk-deep" => 10,
                "enc-night-briefing" => {
                    if p.hour >= 17 || p.hour < 6 {
                        11
                    } else {
                        0
                    }
                }
                _ => return None,
            };
            ("encounter-settings-v2", 3, 4, cell)
        }
    };
    let width = 1536.0 / f64::from(columns);
    let height = 1024.0 / f64::from(rows);
    let x = f64::from(cell % columns) * width;
    let y = f64::from(cell / columns) * height;
    Some(
        html! {<svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas} data-cell={cell.to_string()} viewBox={format!("{x} {y} {width} {height}")} preserveAspectRatio="xMidYMid slice"><image href={crate::paths::asset_path(&format!("static/img/journey/{atlas}.png"))} width="1536" height="1024"/></svg>},
    )
}

pub fn cast(p: &Props, name: &str) -> Html {
    let Some(party) = &p.party else {
        return Html::default();
    };
    let Some(member) = subject(party, p.subject.as_deref(), p.day, name) else {
        return Html::default();
    };
    html! {<><div class={classes!("scene-speaker",p.local_npc.is_some().then_some("conversation-player"))} data-subject={member.persona.clone()}><img src={crate::paths::asset_path(&format!("static/img/journey/occupant-{}.png",member.persona))} alt="" /><span>{&member.name}</span></div>
        if let Some(npc)=p.local_npc {<div class="scene-speaker scene-npc" data-npc={npc.to_string()}><svg viewBox={format!("{} {} 512 512",u32::from(npc%3)*512,u32::from(npc/3)*512)} aria-hidden="true"><image href={crate::paths::asset_path("static/img/journey/town-npcs-v1.png")} width="1536" height="1024"/></svg><span>{crate::i18n::t("trail.local")}</span></div>}
    </>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn casting_rotates_and_never_revives_absent_people() {
        let mut party = Party::default();
        party.initialize("journalist", 17);
        party.set_status("organizer", MemberStatus::Departed);
        party.set_status("satirist", MemberStatus::Dead);
        for day in 1..20 {
            assert!(
                subject(&party, Some("organizer"), day, "enc-community")
                    .is_some_and(|m| m.status == MemberStatus::Active)
            );
        }
        let cast: std::collections::BTreeSet<_> = (1..10)
            .filter_map(|day| {
                subject(&party, None, day, "enc-community").map(|m| m.persona.as_str())
            })
            .collect();
        assert_eq!(cast.len(), 4);
    }
}
