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

static CANONICAL: std::sync::LazyLock<Vec<crate::game::data::Encounter>> =
    std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../static/assets/data/game.json"))
            .expect("shipped encounters are valid")
    });

pub const ROAD_FAMILIES: &[(&str, &str)] = &[
    ("classic_bridge_crews", "ENC-C01"),
    ("classic_civic_potluck", "ENC-C02"),
    ("classic_crossing_block_party", "ENC-C03"),
    ("classic_freeway_mural", "ENC-C04"),
    ("classic_mail_drop", "ENC-C05"),
    ("classic_media_training", "ENC-C06"),
    ("classic_mutual_aid", "ENC-C07"),
    ("classic_mutual_aid_dispatch", "ENC-C08"),
    ("classic_neighborhood_watch", "ENC-C09"),
    ("classic_overpass_stage", "ENC-C10"),
    ("classic_press_briefing", "ENC-C11"),
    ("classic_press_pool_qna", "ENC-C12"),
    ("classic_radio_phonebank", "ENC-C13"),
    ("classic_service_station", "ENC-C14"),
    ("classic_union_blockade", "ENC-C15"),
    ("classic_water_drive", "ENC-C16"),
    ("fundraiser_detour", "ENC-C17"),
    ("town_hall_drift", "ENC-C18"),
    ("beltway_briefing", "ENC-D01"),
    ("deep_circuit_breaker", "ENC-D02"),
    ("deep_field_intel", "ENC-D03"),
    ("deep_grassroots_signal", "ENC-D04"),
    ("deep_media_ambush", "ENC-D05"),
    ("deep_memorandum_dump", "ENC-D06"),
    ("deep_secure_line", "ENC-D07"),
    ("deep_state_dirge", "ENC-D08"),
    ("deep_watch_party", "ENC-D09"),
    ("deep_watchdog_sync", "ENC-D10"),
    ("deep_waystation_boost", "ENC-D11"),
    ("deep_rustbelt_convoy", "ENC-D12"),
    ("deep_beltway_fastpass", "ENC-D13"),
    ("raw_milk", "ENC-S01"),
    ("tariff_whiplash", "ENC-S02"),
    ("clinic_triage", "ENC-S03"),
    ("overnight_briefing", "ENC-S04"),
    ("sat_straw_reserve", "ENC-S05"),
    ("sat_corn_bullets", "ENC-S06"),
    ("sat_grant_groceries", "ENC-S07"),
    ("sat_cow_citations", "ENC-S08"),
    ("sat_forecast_corrected", "ENC-S09"),
    ("sat_parking_gulf", "ENC-S10"),
    ("sat_library_minimum", "ENC-S11"),
    ("sat_tariff_forklift", "ENC-S12"),
    ("sat_alternator_tariff", "ENC-S13"),
    ("sat_billion_pothole", "ENC-S14"),
    ("sat_bridge_bullets", "ENC-S15"),
    ("sat_press_pool_radio", "ENC-S16"),
    ("sat_hydration_pressure", "ENC-S17"),
    ("sat_museum_grant", "ENC-S18"),
    ("sat_factcheck_shift", "ENC-S19"),
    ("sat_food_shelf", "ENC-S20"),
    ("sat_cabinet_guest", "ENC-S21"),
    ("sat_name_infrastructure", "ENC-S22"),
    ("sat_straw_inspection", "ENC-S23"),
    ("sat_shower_force", "ENC-S24"),
    ("sat_receipt_museum", "ENC-S25"),
    ("sat_transit_ribbon", "ENC-S26"),
    ("sat_weather_desk", "ENC-S27"),
    ("sat_bibliography_emergency", "ENC-S28"),
    ("west_grant_translation", "ENC-S29"),
    ("west_rail_replacement", "ENC-S30"),
    ("west_laboratory_overhead", "ENC-S31"),
    ("west_wind_loyalty", "ENC-S32"),
    ("west_desert_pressure", "ENC-S33"),
    ("west_beef_passports", "ENC-S34"),
];

/// Shared settings remain provisional until each variant's authored art is accepted.
pub fn runtime_for_unit(unit: &str) -> Option<&'static str> {
    ROAD_FAMILIES
        .iter()
        .find(|(_, family)| valid(unit, family))
        .map(|(runtime, _)| *runtime)
}

fn family(gs: &GameState) -> Option<&'static str> {
    let encounter = gs.current_encounter.as_ref()?;
    let family = ROAD_FAMILIES
        .iter()
        .find(|(runtime, _)| *runtime == encounter.id)?
        .1;
    is_shipped(encounter).then_some(family)
}

fn is_shipped(encounter: &crate::game::data::Encounter) -> bool {
    CANONICAL
        .iter()
        .find(|event| event.id == encounter.id)
        .is_some_and(|canonical| {
            encounter.name == canonical.name
                && encounter.desc == canonical.desc
                && encounter.choices.len() == canonical.choices.len()
                && encounter
                    .choices
                    .iter()
                    .zip(&canonical.choices)
                    .all(|(a, b)| a.label == b.label && a.effects == b.effects)
        })
}

/// Keep imported wording out of both variant and legacy localization namespaces.
/// This ID is only for rendering; it never replaces the simulation encounter ID.
pub fn copy_id(gs: &GameState) -> String {
    let Some(encounter) = gs.current_encounter.as_ref() else {
        return String::new();
    };
    if is_shipped(encounter) {
        encounter_unit(gs).unwrap_or(&encounter.id).to_owned()
    } else {
        format!("imported/{}", encounter.id)
    }
}

pub fn record_outcome(before: &GameState, after: &mut GameState, choice: usize) {
    let Some(family) = family(before) else {
        return;
    };
    if encounter_unit(before).is_none()
        || before
            .current_encounter
            .as_ref()
            .is_none_or(|event| choice >= event.choices.len())
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

/// Activity offers remain stable during a day or town visit without consuming RNG.
fn activity_key(
    gs: &GameState,
    action: crate::game::activities::Activity,
) -> (String, &'static str, u32) {
    use crate::game::activities::Activity;
    let family = match action {
        Activity::Forage => "ACT-FORAGE",
        Activity::Glean => "ACT-GLEAN",
        Activity::WorkSupplies => "ACT-FOODWORK",
        Activity::WorkCash => "ACT-CASHWORK",
    };
    let (scope, occurrence) = if Activity::TOWN.contains(&action) {
        (
            "town",
            gs.continuity.route_services.stop.unwrap_or_default(),
        )
    } else {
        ("day", gs.day)
    };
    (format!("{family}/{scope}/{occurrence}"), family, occurrence)
}

pub fn activity_unit(gs: &GameState, action: crate::game::activities::Activity) -> String {
    let (key, family, occurrence) = activity_key(gs, action);
    if let Some(unit) = gs
        .continuity
        .visual_content
        .selections
        .get(&key)
        .filter(|unit| valid(unit, family))
    {
        return unit.clone();
    }
    let suffix = ["A", "B", "C"][(hash(gs.seed, family, occurrence) % 3) as usize];
    format!("{family}-{suffix}")
}

/// Called after the existing activity handler succeeds, using its original offer identity.
pub fn record_activity(
    before: &GameState,
    after: &mut GameState,
    action: crate::game::activities::Activity,
) {
    let (key, _, _) = activity_key(before, action);
    let unit = activity_unit(before, action);
    after
        .continuity
        .visual_content
        .selections
        .entry(key.clone())
        .or_insert(unit);
    after
        .continuity
        .visual_content
        .outcomes
        .entry(key)
        .or_insert(0);
}

pub(crate) fn service_unit(gs: &GameState, family: &str, scope: &str, occurrence: u32) -> String {
    let key = format!("{family}/{scope}/{occurrence}");
    if let Some(unit) = gs
        .continuity
        .visual_content
        .selections
        .get(&key)
        .filter(|unit| valid(unit, family))
    {
        return unit.clone();
    }
    let suffix = ["A", "B", "C"][(hash(gs.seed, family, occurrence) % 3) as usize];
    format!("{family}-{suffix}")
}

pub fn rest_unit(gs: &GameState) -> String {
    service_unit(gs, "ACT-REST", "day", gs.day)
}

pub fn departure_unit(gs: &GameState) -> Option<String> {
    let persona = gs.persona_id.as_deref()?;
    if !matches!(persona, "journalist" | "lobbyist" | "organizer" | "satirist" | "staffer" | "whistleblower") {
        return None;
    }
    Some(service_unit(gs, &format!("OPEN-{}", persona.to_uppercase()), "departure", 0))
}

/// Describe recorded outcomes; never infer victory from the displayed score.
pub fn ending_unit(gs: &GameState) -> Option<String> {
    use crate::game::{Ending, boss::HearingOutcome};
    if gs.continuity.visual_content.edition != EDITION || gs.continuity.abandoned {
        return None;
    }
    let family = match gs.ending {
        Some(Ending::BossVictory) => "END-VICTORY".to_owned(),
        Some(Ending::BossVoteFailed) => "END-VOTEFAIL".to_owned(),
        Some(Ending::SanityLoss) => "END-SANITY".to_owned(),
        Some(Ending::VehicleFailure { .. }) => "END-DESTROYED".to_owned(),
        Some(Ending::Exposure { kind }) => format!("END-{}", kind.key().to_uppercase()),
        Some(Ending::Collapse { cause }) => format!("END-COLLAPSE-{}", cause.key().to_uppercase()),
        None => match gs.boss.hearing.as_ref().map(|report| report.outcome) {
            Some(HearingOutcome::Passed | HearingOutcome::Secured) => "END-VICTORY",
            Some(HearingOutcome::Failed) => "END-VOTEFAIL",
            Some(HearingOutcome::Exhausted) => "END-SANITY",
            None if gs.boss.outcome.attempted => if gs.boss.outcome.victory { "END-VICTORY" } else { "END-VOTEFAIL" },
            None if gs.boss.outcome.victory => "END-FALLBACK",
            None => "END-INCOMPLETE",
        }.to_owned(),
    };
    Some(service_unit(gs, &family, "ending", 0))
}

pub fn record_departure(gs: &mut GameState) -> Option<String> {
    let unit = departure_unit(gs)?;
    let family = unit.rsplit_once('-')?.0;
    let key = format!("{family}/departure/0");
    gs.continuity.visual_content.selections.entry(key.clone()).or_insert(unit.clone());
    gs.continuity.visual_content.outcomes.entry(key).or_insert(0);
    Some(unit)
}

fn trade_family(kind: u8) -> &'static str {
    match kind {
        1 => "ACT-BARTERBATTERY",
        2 => "ACT-BARTERSUPPLIES",
        _ => "ACT-BARTERTIRE",
    }
}

pub fn trade_unit(gs: &GameState, kind: u8) -> String {
    service_unit(
        gs,
        trade_family(kind),
        "town",
        gs.continuity.route_services.stop.unwrap_or_default(),
    )
}

pub fn record_rest(before: &GameState, after: &mut GameState) {
    let key = format!("ACT-REST/day/{}", before.day);
    after
        .continuity
        .visual_content
        .selections
        .entry(key.clone())
        .or_insert_with(|| rest_unit(before));
    after
        .continuity
        .visual_content
        .outcomes
        .entry(key)
        .or_insert(0);
}

pub fn record_trade(before: &GameState, after: &mut GameState, kind: u8) {
    let key = format!(
        "{}/town/{}",
        trade_family(kind),
        before.continuity.route_services.stop.unwrap_or_default()
    );
    after
        .continuity
        .visual_content
        .selections
        .entry(key.clone())
        .or_insert_with(|| trade_unit(before, kind));
    after
        .continuity
        .visual_content
        .outcomes
        .entry(key)
        .or_insert(0);
}

/// An unresolved illness retains its presentation across later care checks.
pub fn care_unit(gs: &GameState) -> Option<String> {
    let persona = gs.continuity.crew_care.pending.as_ref()?;
    if gs.continuity.crew_care.reason >= 8 {
        return None;
    }
    let family = format!("CARE-{:02}", gs.continuity.crew_care.reason + 1);
    let strain = gs
        .continuity
        .crew_care
        .strain
        .get(persona)
        .copied()
        .unwrap_or(1);
    if strain > 1 {
        if let Some(unit) = gs
            .continuity
            .visual_content
            .selections
            .get(&format!("care-active/{persona}"))
            .filter(|unit| valid(unit, &family))
        {
            return Some(unit.clone());
        }
    }
    Some(service_unit(
        gs,
        &family,
        &format!("care/{persona}"),
        gs.continuity.crew_care.last_check_day,
    ))
}

pub fn record_care(before: &GameState, after: &mut GameState, choice: usize) {
    let Some(unit) = care_unit(before) else {
        return;
    };
    let persona = before.continuity.crew_care.pending.as_ref().unwrap();
    let key = format!(
        "{}/care/{persona}/{}",
        &unit[..7],
        before.continuity.crew_care.last_check_day
    );
    after
        .continuity
        .visual_content
        .selections
        .insert(key.clone(), unit.clone());
    after
        .continuity
        .visual_content
        .outcomes
        .entry(key)
        .or_insert(choice);
    let active = format!("care-active/{persona}");
    if choice == 2
        && before
            .continuity
            .crew_care
            .strain
            .get(persona)
            .copied()
            .unwrap_or(1)
            < 3
    {
        after
            .continuity
            .visual_content
            .selections
            .insert(active, unit);
    } else {
        after.continuity.visual_content.selections.remove(&active);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ending_copy_uses_recorded_cause_and_never_the_score() {
        use crate::game::{Ending, CollapseCause};
        let mut gs = super::GameState::default();
        gs.continuity.visual_content.edition = super::EDITION;
        gs.stats.credibility = 100;
        assert!(super::ending_unit(&gs).unwrap().starts_with("END-INCOMPLETE-"));
        gs.boss.outcome.attempted = true;
        assert!(super::ending_unit(&gs).unwrap().starts_with("END-VOTEFAIL-"));
        gs.boss.outcome.victory = true;
        assert!(super::ending_unit(&gs).unwrap().starts_with("END-VICTORY-"));
        for cause in [CollapseCause::Hunger, CollapseCause::Vehicle, CollapseCause::Weather, CollapseCause::Breakdown, CollapseCause::Disease, CollapseCause::Crossing, CollapseCause::Panic] {
            gs.ending = Some(Ending::Collapse { cause });
            assert!(super::ending_unit(&gs).unwrap().starts_with(&format!("END-COLLAPSE-{}-", cause.key().to_uppercase())));
        }
        gs.ending = None;
        gs.boss.outcome.attempted = false;
        assert!(super::ending_unit(&gs).unwrap().starts_with("END-FALLBACK-"));
        gs.continuity.abandoned = true;
        assert_eq!(super::ending_unit(&gs), None);
    }
    #[test]
    fn departures_are_persona_bound_stable_and_presentation_only() {
        for persona in ["journalist", "lobbyist", "organizer", "satirist", "staffer", "whistleblower"] {
            let mut variants = std::collections::BTreeSet::new();
            for seed in 0..32 {
                let mut state = super::GameState::default();
                state.seed = seed;
                state.persona_id = Some(persona.into());
                let before = state.clone();
                let unit = super::departure_unit(&state).unwrap();
                assert!(unit.starts_with(&format!("OPEN-{}-", persona.to_uppercase())));
                variants.insert(unit.clone());
                assert_eq!(super::record_departure(&mut state), Some(unit.clone()));
                let restored: super::GameState = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
                assert_eq!(super::departure_unit(&restored), Some(unit));
                state.continuity.visual_content = before.continuity.visual_content.clone();
                assert_eq!(serde_json::to_value(state).unwrap(), serde_json::to_value(before).unwrap());
            }
            assert_eq!(variants.len(), 3);
        }
        assert_eq!(super::departure_unit(&super::GameState::default()), None);
    }
    use super::*;
    fn run() -> GameState {
        let mut gs = GameState {
            seed: u64::MAX - 7,
            ..GameState::default()
        };
        gs.continuity.visual_content.edition = EDITION;
        gs.continuity.last_encounter_driving_minutes = Some(300);
        gs.current_encounter = Some(
            CANONICAL
                .iter()
                .find(|event| event.id == "classic_bridge_crews")
                .unwrap()
                .clone(),
        );
        gs
    }
    #[test]
    fn care_keeps_its_story_through_deferral_and_records_only_presentation() {
        for reason in 0..8 {
            let mut before = GameState {
                seed: 42,
                day: 5,
                persona_id: Some("journalist".into()),
                ..GameState::default()
            };
            before.party.initialize("journalist", 7);
            before.stats.supplies = 10;
            before.continuity.crew_care.pending = Some("organizer".into());
            before.continuity.crew_care.reason = reason;
            before
                .continuity
                .crew_care
                .strain
                .insert("organizer".into(), 1);
            before.continuity.crew_care.last_check_day = 5;
            let unit = care_unit(&before).unwrap();
            for choice in 0..3 {
                let mut after = before.clone();
                assert!(after.resolve_crew_care(choice).is_some());
                let mechanics = serde_json::to_value(&after).unwrap();
                record_care(&before, &mut after, choice);
                let mut actual = serde_json::to_value(&after).unwrap();
                actual["visual_content"] = mechanics["visual_content"].clone();
                assert_eq!(actual, mechanics);
                if choice == 2 {
                    let mut restored: GameState =
                        serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
                    restored.day = 12; // Care checks need not be exactly five days apart.
                    restored.check_crew(11);
                    assert_eq!(care_unit(&restored), Some(unit.clone()));
                    let before_cure = restored.clone();
                    assert!(restored.resolve_crew_care(0).is_some());
                    record_care(&before_cure, &mut restored, 0);
                    assert!(
                        !restored
                            .continuity
                            .visual_content
                            .selections
                            .contains_key("care-active/organizer")
                    );
                }
            }
        }
    }
    #[test]
    fn all_195_road_variants_have_complete_copy_and_current_save_identity() {
        let copy: serde_json::Value =
            serde_json::from_str(include_str!("../../i18n/en.json")).unwrap();
        assert_eq!(ROAD_FAMILIES.len(), 65);
        for &(runtime, family) in ROAD_FAMILIES {
            let event = CANONICAL.iter().find(|event| event.id == runtime).unwrap();
            for suffix in ["A", "B", "C"] {
                let unit = format!("{family}-{suffix}");
                assert_eq!(runtime_for_unit(&unit), Some(runtime));
                let row = &copy["encounter_copy"][&unit];
                for field in ["name", "desc"].into_iter().map(str::to_owned).chain(
                    (0..event.choices.len())
                        .flat_map(|i| [format!("choice_{i}"), format!("log_{i}")]),
                ) {
                    let text = row[&field].as_str().unwrap();
                    assert!(!text.is_empty() && !text.ends_with('…'), "{unit}.{field}");
                }
                assert!(row[format!("choice_{}", event.choices.len())].is_null());
                let mut gs = run();
                gs.current_encounter = Some(event.clone());
                gs.continuity
                    .visual_content
                    .selections
                    .insert(format!("{family}/road/300"), unit.clone());
                let mut restored: GameState =
                    serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
                seal_encounter(&mut restored);
                assert_eq!(encounter_unit(&restored), Some(unit.as_str()));
            }
        }
        assert_eq!(runtime_for_unit("ENC-W01-A"), None);
        assert_eq!(runtime_for_unit("ENC-C01-D"), None);
    }
    #[test]
    fn rest_and_barter_record_original_offer_without_mutating_mechanics() {
        let mut before = GameState {
            seed: 42,
            day: 6,
            ..GameState::default()
        };
        before.stats.supplies = 10;
        before.inventory.spares.tire = 1;
        before.continuity.route_services.stop = Some(160);
        for kind in 0..3 {
            let mut after = before.clone();
            assert!(after.trade_route_offer(kind));
            let mechanics = serde_json::to_value(&after).unwrap();
            record_trade(&before, &mut after, kind);
            assert!(!after.can_route_trade(kind));
            let key = format!("{}/town/160", trade_family(kind));
            assert_eq!(
                after.continuity.visual_content.selections[&key],
                trade_unit(&before, kind)
            );
            let mut actual = serde_json::to_value(&after).unwrap();
            actual["visual_content"] = mechanics["visual_content"].clone();
            assert_eq!(actual, mechanics);
        }
        before.continuity.route_services.stop = None;
        let mut after = before.clone();
        assert!(
            crate::game::camp_rest(&mut after, &crate::game::CampConfig::default_config()).rested
        );
        assert!(after.day > before.day);
        let mechanics = serde_json::to_value(&after).unwrap();
        record_rest(&before, &mut after);
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        assert_eq!(
            restored.continuity.visual_content.selections["ACT-REST/day/6"],
            rest_unit(&before)
        );
        let mut actual = serde_json::to_value(&restored).unwrap();
        actual["visual_content"] = mechanics["visual_content"].clone();
        assert_eq!(actual, mechanics);
    }
    #[test]
    fn activity_identity_survives_overnight_completion_without_changing_simulation() {
        use crate::game::activities::Activity;
        for action in [
            Activity::Forage,
            Activity::Glean,
            Activity::WorkSupplies,
            Activity::WorkCash,
        ] {
            let mut seen = std::collections::BTreeSet::new();
            for seed in 0..12 {
                let mut before = GameState {
                    seed,
                    day: 6,
                    ..GameState::default()
                };
                before.stats.supplies = 5;
                before.continuity.clock_minutes = crate::game::travel_time::TRAVEL_DAY_END - 60;
                if Activity::TOWN.contains(&action) {
                    before.continuity.route_services.stop = Some(160);
                }
                let unit = activity_unit(&before, action);
                seen.insert(unit.clone());
                let restored: GameState =
                    serde_json::from_str(&serde_json::to_string(&before).unwrap()).unwrap();
                assert_eq!(activity_unit(&restored, action), unit);
                let mut after = restored.clone();
                assert!(after.perform_activity(action));
                let mechanics = serde_json::to_value(&after).unwrap();
                record_activity(&before, &mut after, action);
                let (key, _, _) = activity_key(&before, action);
                assert_eq!(after.continuity.visual_content.selections[&key], unit);
                assert_eq!(after.continuity.visual_content.outcomes[&key], 0);
                let mut actual = serde_json::to_value(&after).unwrap();
                actual["visual_content"] = mechanics["visual_content"].clone();
                assert_eq!(actual, mechanics);
            }
            assert_eq!(seen.len(), 3);
        }
    }
    #[test]
    fn every_integrated_family_preserves_imported_copy_and_outcome_identity() {
        let families = ROAD_FAMILIES;
        for &(runtime, family) in families {
            let mut original = run();
            original.current_encounter =
                Some(CANONICAL.iter().find(|e| e.id == runtime).unwrap().clone());
            seal_encounter(&mut original);
            let unit = encounter_unit(&original).unwrap().to_owned();
            assert!(unit.starts_with(family));
            let restored: GameState =
                serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
            assert_eq!(encounter_unit(&restored), Some(unit.as_str()));
            for choice in 0..original.current_encounter.as_ref().unwrap().choices.len() {
                let mut after = restored.clone();
                after.current_encounter = None;
                let before = serde_json::to_value(&after).unwrap();
                record_outcome(&restored, &mut after, choice);
                assert_eq!(
                    after
                        .continuity
                        .visual_content
                        .outcomes
                        .get(&format!("{family}/road/300")),
                    Some(&choice)
                );
                let mut actual = serde_json::to_value(&after).unwrap();
                actual["visual_content"] = before["visual_content"].clone();
                assert_eq!(
                    actual, before,
                    "presentation must not change simulation state: {runtime}"
                );
            }
            for field in ["name", "desc", "label"] {
                let mut custom = restored.clone();
                let event = custom.current_encounter.as_mut().unwrap();
                match field {
                    "name" => event.name = "Imported title".into(),
                    "desc" => event.desc = "Imported story".into(),
                    _ => event.choices[0].label = "Imported action".into(),
                }
                let before = serde_json::to_value(&custom).unwrap();
                seal_encounter(&mut custom);
                assert_eq!(encounter_unit(&custom), None, "custom {field}: {runtime}");
                assert_eq!(copy_id(&custom), format!("imported/{runtime}"));
                assert_eq!(serde_json::to_value(&custom).unwrap(), before);
                let mut after = custom.clone();
                after.current_encounter = None;
                record_outcome(&custom, &mut after, 0);
                assert!(after.continuity.visual_content.outcomes.is_empty());
            }
        }
    }
    #[test]
    fn three_choice_family_records_third_outcome_and_rejects_custom_effects() {
        let mut before = run();
        before.current_encounter = Some(
            CANONICAL
                .iter()
                .find(|e| e.id == "classic_crossing_block_party")
                .unwrap()
                .clone(),
        );
        seal_encounter(&mut before);
        assert!(encounter_unit(&before).unwrap().starts_with("ENC-C03-"));
        let mut after = before.clone();
        after.current_encounter = None;
        record_outcome(&before, &mut after, 2);
        assert_eq!(
            after
                .continuity
                .visual_content
                .outcomes
                .get("ENC-C03/road/300"),
            Some(&2)
        );
        before.current_encounter.as_mut().unwrap().choices[2]
            .effects
            .sanity = 99;
        assert_eq!(encounter_unit(&before), None);
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
