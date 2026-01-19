use crate::game::{DietId, PaceId};
use crate::i18n;

#[derive(Clone)]
pub struct MenuOption {
    pub idx: u8,
    pub text: String,
    pub selected: bool,
    pub tooltip: String,
}

pub fn menu_options(current_pace: PaceId, current_diet: DietId) -> Vec<MenuOption> {
    vec![
        MenuOption {
            idx: 1,
            text: i18n::t("pacediet.menu.pace_steady"),
            selected: current_pace == PaceId::Steady,
            tooltip: i18n::t("pacediet.tooltips.steady"),
        },
        MenuOption {
            idx: 2,
            text: i18n::t("pacediet.menu.pace_heated"),
            selected: current_pace == PaceId::Heated,
            tooltip: i18n::t("pacediet.tooltips.heated"),
        },
        MenuOption {
            idx: 3,
            text: i18n::t("pacediet.menu.pace_blitz"),
            selected: current_pace == PaceId::Blitz,
            tooltip: i18n::t("pacediet.tooltips.blitz"),
        },
        MenuOption {
            idx: 4,
            text: i18n::t("pacediet.menu.diet_quiet"),
            selected: current_diet == DietId::Quiet,
            tooltip: i18n::t("pacediet.tooltips.quiet"),
        },
        MenuOption {
            idx: 5,
            text: i18n::t("pacediet.menu.diet_mixed"),
            selected: current_diet == DietId::Mixed,
            tooltip: i18n::t("pacediet.tooltips.mixed"),
        },
        MenuOption {
            idx: 6,
            text: i18n::t("pacediet.menu.diet_doom"),
            selected: current_diet == DietId::Doom,
            tooltip: i18n::t("pacediet.tooltips.doom"),
        },
        MenuOption {
            idx: 0,
            text: i18n::t("pacediet.menu.back"),
            selected: false,
            tooltip: i18n::t("pacediet.menu.back"),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_options_mark_selected_entries() {
        crate::i18n::set_lang("en");
        let options = menu_options(PaceId::Heated, DietId::Mixed);
        assert!(options.iter().any(|opt| opt.idx == 2 && opt.selected));
        assert!(options.iter().any(|opt| opt.idx == 5 && opt.selected));
        assert!(options.iter().any(|opt| opt.idx == 0));
    }
}
