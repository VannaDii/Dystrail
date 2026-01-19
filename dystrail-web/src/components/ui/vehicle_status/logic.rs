use crate::game::Part;
use crate::i18n;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleAction {
    UseSpare(Part),
    HackFix,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SelectionResolution {
    Action(VehicleAction, String),
    Message(String),
    Back,
    None,
}

pub(super) fn evaluate_selection(
    idx: u8,
    breakdown_part: Option<Part>,
    spare_counts: Option<(i32, i32, i32, i32)>,
) -> SelectionResolution {
    let used_spare_message = |part: Part| {
        let part_name = i18n::t(part.key());
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("part", part_name.as_str());
        vars.insert("sup", "1");
        i18n::tr("vehicle.announce.used_spare", Some(&vars))
    };
    let missing_spare_message = |part: Part| {
        let part_name = i18n::t(part.key());
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("part", part_name.as_str());
        i18n::tr("vehicle.announce.no_spare", Some(&vars))
    };

    match idx {
        1 => match (breakdown_part, spare_counts) {
            (Some(Part::Tire), Some((tire, _, _, _))) if tire > 0 => SelectionResolution::Action(
                VehicleAction::UseSpare(Part::Tire),
                used_spare_message(Part::Tire),
            ),
            _ => SelectionResolution::Message(missing_spare_message(Part::Tire)),
        },
        2 => match (breakdown_part, spare_counts) {
            (Some(Part::Battery), Some((_, battery, _, _))) if battery > 0 => {
                SelectionResolution::Action(
                    VehicleAction::UseSpare(Part::Battery),
                    used_spare_message(Part::Battery),
                )
            }
            _ => SelectionResolution::Message(missing_spare_message(Part::Battery)),
        },
        3 => match (breakdown_part, spare_counts) {
            (Some(Part::Alternator), Some((_, _, alt, _))) if alt > 0 => {
                SelectionResolution::Action(
                    VehicleAction::UseSpare(Part::Alternator),
                    used_spare_message(Part::Alternator),
                )
            }
            _ => SelectionResolution::Message(missing_spare_message(Part::Alternator)),
        },
        4 => match (breakdown_part, spare_counts) {
            (Some(Part::FuelPump), Some((_, _, _, pump))) if pump > 0 => {
                SelectionResolution::Action(
                    VehicleAction::UseSpare(Part::FuelPump),
                    used_spare_message(Part::FuelPump),
                )
            }
            _ => SelectionResolution::Message(missing_spare_message(Part::FuelPump)),
        },
        5 => {
            if breakdown_part.is_some() {
                let mut vars = std::collections::BTreeMap::new();
                vars.insert("sup", "3");
                vars.insert("cred", "1");
                vars.insert("day", "1");
                SelectionResolution::Action(
                    VehicleAction::HackFix,
                    i18n::tr("vehicle.announce.hack_applied", Some(&vars)),
                )
            } else {
                SelectionResolution::Message(i18n::t("vehicle.no_active"))
            }
        }
        0 => SelectionResolution::Back,
        _ => SelectionResolution::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_selection_handles_spares_and_messages() {
        crate::i18n::set_lang("en");
        let resolution = evaluate_selection(1, Some(Part::Tire), Some((1, 0, 0, 0)));
        assert!(matches!(
            resolution,
            SelectionResolution::Action(VehicleAction::UseSpare(Part::Tire), _)
        ));

        let missing = evaluate_selection(2, Some(Part::Battery), Some((0, 0, 0, 0)));
        assert!(matches!(missing, SelectionResolution::Message(_)));
    }

    #[test]
    fn evaluate_selection_handles_hack_and_back() {
        crate::i18n::set_lang("en");
        let hack = evaluate_selection(5, Some(Part::Alternator), Some((0, 0, 0, 0)));
        assert!(matches!(
            hack,
            SelectionResolution::Action(VehicleAction::HackFix, _)
        ));

        let back = evaluate_selection(0, None, None);
        assert_eq!(back, SelectionResolution::Back);
    }

    #[test]
    fn evaluate_selection_ignores_unknown_indices() {
        let none = evaluate_selection(9, None, None);
        assert_eq!(none, SelectionResolution::None);
    }

    #[test]
    fn evaluate_selection_covers_remaining_spare_paths() {
        crate::i18n::set_lang("en");
        let alt = evaluate_selection(3, Some(Part::Alternator), Some((0, 0, 1, 0)));
        assert!(matches!(
            alt,
            SelectionResolution::Action(VehicleAction::UseSpare(Part::Alternator), _)
        ));

        let pump = evaluate_selection(4, Some(Part::FuelPump), Some((0, 0, 0, 2)));
        assert!(matches!(
            pump,
            SelectionResolution::Action(VehicleAction::UseSpare(Part::FuelPump), _)
        ));

        let no_active = evaluate_selection(5, None, Some((0, 0, 0, 0)));
        assert!(matches!(no_active, SelectionResolution::Message(_)));
    }

    #[test]
    fn evaluate_selection_reports_missing_spares() {
        crate::i18n::set_lang("en");
        let tire_missing = evaluate_selection(1, Some(Part::Tire), Some((0, 0, 0, 0)));
        assert!(matches!(tire_missing, SelectionResolution::Message(_)));

        let battery_missing = evaluate_selection(2, Some(Part::Battery), Some((0, 0, 0, 0)));
        assert!(matches!(battery_missing, SelectionResolution::Message(_)));

        let alt_missing = evaluate_selection(3, Some(Part::Alternator), Some((0, 0, 0, 0)));
        assert!(matches!(alt_missing, SelectionResolution::Message(_)));

        let pump_missing = evaluate_selection(4, Some(Part::FuelPump), Some((0, 0, 0, 0)));
        assert!(matches!(pump_missing, SelectionResolution::Message(_)));
    }

    #[test]
    fn evaluate_selection_uses_battery_spare_when_available() {
        crate::i18n::set_lang("en");
        let battery = evaluate_selection(2, Some(Part::Battery), Some((0, 2, 0, 0)));
        assert!(matches!(
            battery,
            SelectionResolution::Action(VehicleAction::UseSpare(Part::Battery), _)
        ));
    }
}
