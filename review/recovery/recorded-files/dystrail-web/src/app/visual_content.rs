//! Presentation selection is independent of simulation RNG and browsing history.
use crate::game::GameState;

pub const EDITION: u16 = 1;

/// Enable families only after all required translations and staging are reviewed.
/// A family missing from this allowlist keeps the previously integrated A prose.
fn enabled(family: &str) -> bool {
    matches!(family, "ENC-C01")
}

fn hash(seed: u64, family: &str, occurrence: &str) -> u64 {
    family.bytes().chain(occurrence.bytes()).fold(
        seed ^ 0xcbf2_9ce4_8422_2325,
        |h, b| (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3),
    )
}

/// Cosmetic town identity repeats for the same route, stop and run code. Existing
/// saved identities are retained; neither lookup nor revisiting draws game RNG.
pub fn town_resident(gs: &GameState) -> u8 {
    gs.continuity.activities.local_word.unwrap_or_else(|| {
        let occurrence = format!(
            "{}/{}/{}",
            gs.continuity.route_services.route_id.as_deref().unwrap_or_default(),
            gs.continuity.route_services.stop.unwrap_or_default(),
            super::town::name(gs),
        );
        u8::try_from(hash(gs.seed, "town-resident", &occurrence) % 6).unwrap_or(0)
    })
}

pub fn select(gs: &mut GameState, family: &str, occurrence: &str) -> String {
    let key = format!("{family}/{occurrence}");
    if let Some(unit) = gs.continuity.visual_content.selections.get(&key) {
        return unit.clone();
    }
    let variant = if gs.continuity.visual_content.edition == EDITION && enabled(family) {
        ["A", "B", "C"][usize::try_from(hash(gs.seed, family, occurrence) % 3).unwrap_or(0)]
    } else {
        "A"
    };
    let unit = format!("{family}-{variant}");
    gs.continuity.visual_content.selections.insert(key, unit.clone());
    unit
}

pub fn encounter_family(runtime: &str) -> Option<&'static str> {
    match runtime {
        "classic_bridge_crews" => Some("ENC-C01"),
        _ => None,
    }
}

pub fn encounter_occurrence(gs: &GameState) -> String {
    // The existing engine timestamp is frozen while a road decision is pending.
    format!("road/{}", gs.continuity.last_encounter_driving_minutes.unwrap_or(gs.continuity.driving_minutes_total))
}

pub fn seal_encounter(gs: &mut GameState) {
    let family = gs.current_encounter.as_ref().and_then(|e| encounter_family(&e.id));
    if let Some(family) = family {
        let occurrence = encounter_occurrence(gs);
        select(gs, family, &occurrence);
    }
}

pub fn encounter_unit(gs: &GameState, runtime: &str) -> Option<String> {
    let family = encounter_family(runtime)?;
    let key = format!("{family}/{}", encounter_occurrence(gs));
    Some(gs.continuity.visual_content.selections.get(&key).cloned().unwrap_or_else(||format!("{family}-A")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn town_residents_repeat_without_changing_the_run_or_saved_identity() {
        let mut gs = GameState { seed: 42, ..GameState::default() };
        gs.continuity.route_services.route_id = Some("seattle".into());
        gs.continuity.route_services.stop = Some(280);
        let before = serde_json::to_value(&gs).unwrap();
        let resident = town_resident(&gs);
        let restored: GameState = serde_json::from_value(before.clone()).unwrap();
        assert_eq!(resident, town_resident(&restored));
        assert_eq!(serde_json::to_value(&gs).unwrap(), before);
        let residents: std::collections::BTreeSet<_> = (0..60).map(|seed| {
            let mut run = gs.clone(); run.seed = seed; town_resident(&run)
        }).collect();
        assert_eq!(residents.len(), 6);
        gs.continuity.activities.local_word = Some(4);
        assert_eq!(town_resident(&gs), 4);
    }
    #[test]
    fn variants_repeat_across_reload_without_touching_engine_randomness() {
        let mut gs = GameState {
            seed: 42,
            ..GameState::default()
        };
        gs.continuity.visual_content.edition = EDITION;
        let before = serde_json::to_value(&gs).unwrap();
        let unit = select(&mut gs,"ENC-C01","road/300");
        let mut restored: GameState = serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        assert_eq!(select(&mut restored,"ENC-C01","road/300"),unit);
        let mut after = serde_json::to_value(&restored).unwrap();
        after["visual_content"] = before["visual_content"].clone();
        assert_eq!(after,before);
        let variants: std::collections::BTreeSet<_> = (0..60).map(|n|select(&mut gs,"ENC-C01",&format!("road/{n}"))).collect();
        assert_eq!(variants.len(),3);
    }
    #[test]
    fn legacy_and_unreviewed_families_keep_a() {
        let mut old = GameState::default();
        assert_eq!(select(&mut old,"ENC-C01","road/300"),"ENC-C01-A");
        old.continuity.visual_content.edition=EDITION;
        assert_eq!(select(&mut old,"ENC-C01","road/300"),"ENC-C01-A");
        assert_eq!(select(&mut old,"ENC-W01","road/300"),"ENC-W01-A");
    }
}
