use super::logic::{evaluate_selection, SelectionResolution, VehicleAction};
use crate::game::Part;

#[test]
fn evaluate_selection_uses_spare_when_available() {
    crate::i18n::set_lang("en");
    let outcome = evaluate_selection(1, Some(Part::Tire), Some((1, 0, 0, 0)));
    assert!(matches!(
        outcome,
        SelectionResolution::Action(VehicleAction::UseSpare(Part::Tire), _)
    ));
}

#[test]
fn evaluate_selection_reports_missing_spare() {
    crate::i18n::set_lang("en");
    let outcome = evaluate_selection(2, Some(Part::Battery), Some((0, 0, 0, 0)));
    assert!(matches!(outcome, SelectionResolution::Message(_)));
}

#[test]
fn evaluate_selection_handles_hack_fix() {
    crate::i18n::set_lang("en");
    let with_breakdown = evaluate_selection(5, Some(Part::FuelPump), None);
    assert!(matches!(
        with_breakdown,
        SelectionResolution::Action(VehicleAction::HackFix, _)
    ));

    let without = evaluate_selection(5, None, None);
    assert!(matches!(without, SelectionResolution::Message(_)));
}

#[test]
fn evaluate_selection_back_option() {
    crate::i18n::set_lang("en");
    assert_eq!(evaluate_selection(0, None, None), SelectionResolution::Back);
}
