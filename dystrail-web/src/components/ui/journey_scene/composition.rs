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

/// Resolve real atlas cells rather than treating setting names as standalone files.
pub(super) fn encounter_setting(
    name: &str,
    hour: u8,
    region: Option<Region>,
) -> Option<(&'static str, u8, u8, u8)> {
    let interior = match name {
        "enc-motel" => Some(0),
        "enc-cafe" => Some(1),
        "enc-library" => Some(2),
        "enc-museum" => Some(3),
        "enc-farm-office" => Some(4),
        "enc-service-counter" => Some(5),
        _ => None,
    };
    if let Some(cell) = interior {
        return Some(("encounter-settings-v3", 3, 2, cell));
    }
    if name == "enc-rest-area" || (name == "enc-night-briefing" && super::lighting::profile(hour) != "night") {
        return Some(match region {
            Some(Region::Southwest) => ("western-settings-v1", 2, 3, 4),
            Some(Region::MountainWest) => ("western-settings-v1", 2, 3, 5),
            _ => ("journey-settings-v1", 2, 3, 5),
        });
    }
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
        "enc-night-briefing" => 11,
        _ => return None,
    };
    Some(("encounter-settings-v2", 3, 4, cell))
}

pub(super) fn is_indoors(name: &str) -> bool {
    matches!(
        name,
        "enc-media-workshop"
            | "enc-service"
            | "enc-civic"
            | "enc-clinic"
            | "enc-radio"
            | "enc-motel"
            | "enc-cafe"
            | "enc-library"
            | "enc-museum"
            | "enc-farm-office"
            | "enc-service-counter"
    )
}

pub fn setting(p: &Props, name: &str) -> Option<Html> {
    let (atlas, columns, rows, cell) = match &p.stage {
        SceneStage::Town
            if matches!(
                p.region,
                Some(Region::PacificCoast | Region::MountainWest | Region::Southwest)
            ) =>
        {
            (
                "western-settings-v1",
                2,
                3,
                match p.region {
                    Some(Region::PacificCoast)
                        if p.road_asset.as_deref() == Some("open-california-hills") =>
                    {
                        1
                    }
                    Some(Region::MountainWest) => 2,
                    Some(Region::Southwest) => 3,
                    _ => 0,
                },
            )
        }
        SceneStage::Camp | SceneStage::Care | SceneStage::CareIncident { .. } if p.region == Some(Region::Southwest) => {
            ("western-settings-v1", 2, 3, 4)
        }
        SceneStage::Camp | SceneStage::Care | SceneStage::CareIncident { .. } if p.region == Some(Region::MountainWest) => {
            ("western-settings-v1", 2, 3, 5)
        }
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
        SceneStage::Camp | SceneStage::Care | SceneStage::CareIncident { .. } => ("journey-settings-v1", 2, 3, 3),
        SceneStage::Ending(true) => ("journey-settings-v1", 2, 3, 4),
        SceneStage::Ending(false) => return None,
        _ => encounter_setting(name, p.hour, p.region)?,
    };
    let width = 1536.0 / f64::from(columns);
    let height = 1024.0 / f64::from(rows);
    let landscape_shared = match &p.stage { SceneStage::Encounter(unit) | SceneStage::EncounterOutcome { unit, .. } => super::encounters::shared_aspect(unit) == Some("1.7777778"), _ => false };
    let x = f64::from(cell % columns) * width;
    let y = f64::from(cell / columns) * height;
    // Crop surplus ceiling/floor in shared interiors; keep scale uniform.
    let (y, height) = if landscape_shared { (y + 64.0, width * 9.0 / 16.0) } else { (y, height) };
    // Blank the retained exhibit-paper marks in the atlas coordinate space.
    // These surfaces inherit the scene's one uniform transform and lighting.
    let blank_labels = (name == "enc-museum").then(|| html! {
        <g data-blank-exhibit-labels="true" shape-rendering="crispEdges">
            <path d="M44 781 L81 779 L94 813 L53 816 Z" fill="#e5dac3"/>
            <path d="M119 743 L137 740 L144 762 L126 766 Z" fill="#e4d7b7"/>
            <path d="M25 752 L48 750 L56 774 L32 780 Z" fill="#e4d7b7"/>
            <path d="M302 653 H322 V667 H302 Z" fill="#e4d6b4"/>
            <path d="M261 694 L297 693 L301 731 L264 733 Z" fill="#cdbb97"/>
            <path d="M278 729 H305 V737 H278 Z" fill="#e4d6b4"/>
            <path d="M355 723 H381 V736 H355 Z" fill="#e4d6b4"/>
            <path d="M452 700 L470 701 L462 727 L446 725 Z" fill="#e4d6b4"/>
            <path d="M184 767 L219 765 L230 795 L194 797 Z" fill="#dbc99f"/>
            <path d="M322 767 H352 V782 H322 Z" fill="#e4d6b4"/>
        </g>
    });
    let blank_farm_paper = (name == "enc-farm-office").then(|| html! {
        <g data-blank-farm-paper="true" shape-rendering="crispEdges">
            <path d="M672 632 H700 V672 H672 Z" fill="#cdb69d"/>
        </g>
    });
    let blank_counter_papers = (name == "enc-service-counter").then(|| html! {
        <g data-blank-counter-papers="true" shape-rendering="crispEdges">
            <path d="M1032 610 L1047 612 L1047 631 L1032 630 Z" fill="#d6c7ac"/>
            <path d="M1217 618 H1248 V632 H1217 Z" fill="#cfc0a6"/>
            <path d="M1217 637 H1248 V662 H1217 Z" fill="#c5bda9"/>
            <path d="M1278 747 H1310 L1312 758 H1276 Z" fill="#e4dfc9"/>
        </g>
    });
    Some(
        html! {<svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas} data-cell={cell.to_string()} viewBox={format!("{x} {y} {width} {height}")} preserveAspectRatio="xMidYMid slice"><image href={crate::paths::asset_path(&format!("static/img/journey/{atlas}.png"))} width="1536" height="1024"/>{blank_labels}{blank_farm_paper}{blank_counter_papers}</svg>},
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indoor_atlas_is_six_distinct_square_cells() {
        let scenes = [
            "enc-motel",
            "enc-cafe",
            "enc-library",
            "enc-museum",
            "enc-farm-office",
            "enc-service-counter",
        ];
        for (cell, name) in (0..6).zip(scenes) {
            assert_eq!(
                encounter_setting(name, 8, None),
                Some(("encounter-settings-v3", 3, 2, cell))
            );
            assert_eq!(
                encounter_setting(name, 8, None),
                encounter_setting(name, 20, None)
            );
            assert!(is_indoors(name));
        }
    }

    #[test]
    fn roadside_briefing_stays_outdoors_through_day_and_night() {
        for hour in 0..24 {
            let expected = if (6..21).contains(&hour) {
                ("journey-settings-v1", 2, 3, 5)
            } else {
                ("encounter-settings-v2", 3, 4, 11)
            };
            assert_eq!(
                encounter_setting("enc-night-briefing", hour, None),
                Some(expected)
            );
        }
        assert!(!is_indoors("enc-night-briefing"));
        assert!(!is_indoors("enc-rest-area"));
        assert!(is_indoors("enc-service"));
        assert!(is_indoors("enc-clinic"));
        assert_eq!(encounter_setting("unillustrated", 8, None), None);
    }

    #[test]
    fn outdoor_crew_discussions_keep_western_geography() {
        assert_eq!(
            encounter_setting("enc-rest-area", 12, Some(Region::Southwest)),
            Some(("western-settings-v1", 2, 3, 4))
        );
        assert_eq!(
            encounter_setting("enc-rest-area", 12, Some(Region::MountainWest)),
            Some(("western-settings-v1", 2, 3, 5))
        );
        assert_eq!(
            encounter_setting("enc-rest-area", 12, Some(Region::Heartland)),
            Some(("journey-settings-v1", 2, 3, 5))
        );
    }

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
