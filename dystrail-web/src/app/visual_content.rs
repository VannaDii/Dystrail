//! Stable presentation identity, independent of simulation RNG and UI navigation.
use crate::game::GameState;

pub const EDITION: u16 = 1;

fn hash(seed: u64, family: &str, occurrence: u32) -> u64 {
    family
        .bytes()
        .chain(occurrence.to_le_bytes())
        .fold(seed ^ 0xcbf2_9ce4_8422_2325, |value, byte| {
            (value ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

static C01: std::sync::LazyLock<crate::game::data::Encounter> = std::sync::LazyLock::new(|| {
    serde_json::from_str::<Vec<crate::game::data::Encounter>>(include_str!(
        "../../static/assets/data/game.json"
    ))
    .expect("shipped encounters are valid")
    .into_iter()
    .find(|event| event.id == "classic_bridge_crews")
    .expect("C01 runtime encounter exists")
});

fn family(gs: &GameState) -> Option<&'static str> {
    let encounter = gs.current_encounter.as_ref()?;
    // A reused runtime ID alone is not proof that an imported custom event has
    // the same choices. Authored labels must match the complete shipped effects.
    (encounter.id == C01.id
        && encounter.choices.len() == C01.choices.len()
        && encounter
            .choices
            .iter()
            .zip(&C01.choices)
            .all(|(a, b)| a.effects == b.effects))
    .then_some("ENC-C01")
}

pub fn record_outcome(before: &GameState, after: &mut GameState, choice: usize) {
    let Some(family) = family(before) else {
        return;
    };
    if encounter_unit(before).is_none()
        || choice >= C01.choices.len()
        || after.current_encounter.is_some()
    {
        return;
    }
    let (key, _) = selection_key(before, family);
    after
        .continuity
        .visual_content
        .outcomes
        .entry(key)
        .or_insert(choice);
}

fn selection_key(gs: &GameState, family: &str) -> (String, u32) {
    let occurrence = gs
        .continuity
        .last_encounter_driving_minutes
        .unwrap_or(gs.continuity.driving_minutes_total);
    (format!("{family}/road/{occurrence}"), occurrence)
}

fn valid(unit: &str, family: &str) -> bool {
    ["A", "B", "C"]
        .iter()
        .any(|suffix| unit == format!("{family}-{suffix}"))
}

/// Called only at encounter creation/restore boundaries, never while rendering.
pub fn seal_encounter(gs: &mut GameState) {
    let Some(family) = family(gs) else {
        return;
    };
    if gs.continuity.visual_content.edition != EDITION {
        return;
    }
    let (key, occurrence) = selection_key(gs, family);
    if gs
        .continuity
        .visual_content
        .selections
        .get(&key)
        .is_some_and(|unit| valid(unit, family))
    {
        return;
    }
    let suffix = ["A", "B", "C"][(hash(gs.seed, family, occurrence) % 3) as usize];
    gs.continuity
        .visual_content
        .selections
        .insert(key, format!("{family}-{suffix}"));
}

/// Missing legacy identity means retain legacy presentation, not silently adopt A.
pub fn encounter_unit(gs: &GameState) -> Option<&str> {
    let family = family(gs)?;
    let (key, _) = selection_key(gs, family);
    gs.continuity
        .visual_content
        .selections
        .get(&key)
        .filter(|unit| valid(unit, family))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run() -> GameState {
        let mut gs = GameState {
            seed: u64::MAX - 7,
            ..GameState::default()
        };
        gs.continuity.visual_content.edition = EDITION;
        gs.continuity.last_encounter_driving_minutes = Some(300);
        gs.current_encounter = Some(C01.clone());
        gs
    }
    #[test]
    fn committed_choice_survives_save_and_cannot_be_overwritten() {
        let mut before = run();
        seal_encounter(&mut before);
        let mut after = before.clone();
        after.current_encounter = None;
        record_outcome(&before, &mut after, 1);
        let mut restored: GameState =
            serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        record_outcome(&before, &mut restored, 0);
        assert_eq!(
            restored
                .continuity
                .visual_content
                .outcomes
                .get("ENC-C01/road/300"),
            Some(&1)
        );
        let mut pending = before.clone();
        record_outcome(&before, &mut pending, 0);
        assert!(pending.continuity.visual_content.outcomes.is_empty());
    }
    #[test]
    fn selection_survives_reload_without_changing_mechanics_or_other_state() {
        let mut gs = run();
        let before = serde_json::to_value(&gs).unwrap();
        seal_encounter(&mut gs);
        let selected = encounter_unit(&gs).unwrap().to_owned();
        let mut restored: GameState =
            serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        restored.continuity.clock_minutes += 60; // parked time is not a new occurrence
        seal_encounter(&mut restored);
        assert_eq!(encounter_unit(&restored), Some(selected.as_str()));
        restored.continuity.clock_minutes = gs.continuity.clock_minutes;
        let mut after = serde_json::to_value(&restored).unwrap();
        after["visual_content"] = before["visual_content"].clone();
        assert_eq!(after, before);
        let sealed = serde_json::to_value(&gs).unwrap();
        seal_encounter(&mut gs);
        assert_eq!(serde_json::to_value(&gs).unwrap(), sealed);
    }
    #[test]
    fn legacy_and_custom_encounters_are_not_reinterpreted() {
        let mut gs = run();
        let mut json = serde_json::to_value(&gs).unwrap();
        json.as_object_mut().unwrap().remove("visual_content");
        let mut legacy: GameState = serde_json::from_value(json).unwrap();
        seal_encounter(&mut legacy);
        assert_eq!(encounter_unit(&legacy), None);
        let mut custom = gs.clone();
        custom.current_encounter.as_mut().unwrap().choices[0]
            .effects
            .supplies = 99;
        seal_encounter(&mut custom);
        assert_eq!(encounter_unit(&custom), None);
        gs.current_encounter.as_mut().unwrap().choices.pop();
        seal_encounter(&mut gs);
        assert_eq!(encounter_unit(&gs), None);
        gs.current_encounter.as_mut().unwrap().id = "ENC-W01".into();
        seal_encounter(&mut gs);
        assert!(gs.continuity.visual_content.selections.is_empty());
    }
    #[test]
    fn replay_repeats_and_invalid_saved_selection_is_repaired_deterministically() {
        let original = run();
        let mut variants = std::collections::BTreeSet::new();
        for seed in 0..64 {
            let mut first = original.clone();
            first.seed = seed;
            let mut replay = first.clone();
            seal_encounter(&mut first);
            seal_encounter(&mut replay);
            assert_eq!(encounter_unit(&first), encounter_unit(&replay));
            variants.insert(encounter_unit(&first).unwrap().to_owned());
            replay
                .continuity
                .visual_content
                .selections
                .insert("ENC-C01/road/300".into(), "ENC-W01-A".into());
            seal_encounter(&mut replay);
            assert_eq!(encounter_unit(&first), encounter_unit(&replay));
        }
        assert_eq!(variants.len(), 3);
    }
}
