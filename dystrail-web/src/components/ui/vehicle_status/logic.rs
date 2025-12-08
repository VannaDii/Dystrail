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
