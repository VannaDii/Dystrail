use super::*;

#[test]
fn selection_outcome_covers_all_menu_entries() {
    crate::i18n::set_lang("en");
    let pacing = PacingConfig::default_config();

    let steady = selection_outcome(&pacing, 1);
    assert!(matches!(
        steady,
        Some(SelectionOutcome::Pace(PaceId::Steady, _))
    ));

    let heated = selection_outcome(&pacing, 2);
    match heated {
        Some(SelectionOutcome::Pace(id, msg)) => {
            assert_eq!(id, PaceId::Heated);
            assert!(msg.contains('%'), "message should contain encounter delta");
        }
        other => panic!("expected heated pace outcome, got {other:?}"),
    }

    let doom = selection_outcome(&pacing, 6);
    match doom {
        Some(SelectionOutcome::Diet(id, msg)) => {
            assert_eq!(id, DietId::Doom);
            assert!(
                msg.contains("Doom"),
                "diet announcement should reference the Doom diet: {msg}"
            );
        }
        other => panic!("expected doom diet outcome, got {other:?}"),
    }

    assert!(selection_outcome(&pacing, 0).is_none());
    assert!(selection_outcome(&pacing, 42).is_none());
}
