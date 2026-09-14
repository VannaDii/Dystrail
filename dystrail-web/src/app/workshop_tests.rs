use super::{workshop, workshop_events};
use crate::game::{
    GameMode, GameState,
    state::{CrossingOutcomeTelemetry, CrossingTelemetry},
};

#[test]
fn revised_prose_never_claims_a_receipt_was_consumed_and_first_refusal_is_a_detour() {
    crate::i18n::set_lang("en");
    let before = GameState::default();
    let mut after = before.clone();
    let event: CrossingTelemetry = serde_json::from_value(serde_json::json!({
        "day":1,"region":"Heartland","season":"spring","kind":"checkpoint",
        "permit_used":true,"bribe_attempted":false,"bribe_success":null,"bribe_cost_cents":0,
        "bribe_chance":null,"bribe_roll":null,"detour_reason":null,"detour_taken":false,
        "detour_hours":null,"detour_base_supplies_delta":null,"detour_extra_supplies_loss":null,
        "terminal_threshold":0.0,"terminal_roll":null,"outcome":"passed"
    }))
    .unwrap();
    after.crossing_events.push(event);
    let copy = workshop_events::crossings(&before, &after).join(" ");
    assert!(copy.contains("alternator"));
    assert!(!copy.contains("Receipt"));
    assert_eq!(before.receipts, after.receipts);
    let event = after.crossing_events.last_mut().unwrap();
    event.permit_used = false;
    event.outcome = CrossingOutcomeTelemetry::Detoured;
    event.detour_reason = Some(crate::game::state::CrossingDetourReason::CheckpointDenied);
    let copy = workshop_events::crossings(&before, &after).join(" ");
    assert!(copy.contains("refuses passage"));
    assert!(copy.contains("detour"));
    assert!(!copy.contains("expedition ends"));
}

#[test]
fn scenario_messages_resolve_in_required_languages_without_unfilled_variables() {
    for language in ["en", "it", "es", "ar"] {
        crate::i18n::set_lang(language);
        for reason in 0..8 {
            for field in [
                "setup",
                "continuing",
                "critical",
                "outcomes.helped",
                "outcomes.sheltered",
                "outcomes.deferred",
                "outcomes.companion_lost",
                "outcomes.player_lost",
            ] {
                let text = workshop::named(&workshop::care(reason), field, "Reese");
                assert!(!text.is_empty());
                assert!(!text.contains("workshop."));
                assert!(!text.contains('{'));
            }
        }
        for mode in [GameMode::Classic, GameMode::Deep] {
            let gs = GameState {
                mode,
                ..GameState::default()
            };
            assert!(!workshop::hearing(&gs, "setup").starts_with("workshop."));
        }
    }
    crate::i18n::set_lang("en");
}

#[test]
fn cashless_and_roadside_repair_results_use_the_matching_branch() {
    use crate::game::repairs::RepairChoice;
    crate::i18n::set_lang("en");
    let family = "REPAIR-ALTERNATOR";
    assert!(
        workshop::text(family, workshop::repair_outcome(RepairChoice::Radio, true))
            .contains("arranged work")
    );
    assert!(
        workshop::text(
            family,
            workshop::repair_outcome(RepairChoice::Purchase, true)
        )
        .contains("arrives")
    );
    assert!(
        workshop::text(
            family,
            workshop::repair_outcome(RepairChoice::Purchase, false)
        )
        .contains("shop fits")
    );
    assert_eq!(
        workshop::care_outcome("journey.player_lost"),
        "outcomes.player_lost"
    );
}
