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
    if name == "enc-rest-area" || (name == "enc-night-briefing" && (6..17).contains(&hour)) {
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
        SceneStage::Camp | SceneStage::Care if p.region == Some(Region::Southwest) => {
            ("western-settings-v1", 2, 3, 4)
        }
        SceneStage::Camp | SceneStage::Care if p.region == Some(Region::MountainWest) => {
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
        SceneStage::Camp | SceneStage::Care => ("journey-settings-v1", 2, 3, 3),
        SceneStage::Ending(arrived) => ("journey-settings-v1", 2, 3, if *arrived { 4 } else { 5 }),
        _ => encounter_setting(name, p.hour, p.region)?,
    };
    let width = 1536.0 / f64::from(columns);
    let height = 1024.0 / f64::from(rows);
    let x = f64::from(cell % columns) * width;
    let y = f64::from(cell / columns) * height;
    Some(
        html! {<svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas} data-cell={cell.to_string()} viewBox={format!("{x} {y} {width} {height}")} preserveAspectRatio="xMidYMid slice"><image href={crate::paths::asset_path(&format!("static/img/journey/{atlas}.png"))} width="1536" height="1024"/></svg>},
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
            let expected = if (6..17).contains(&hour) {
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
