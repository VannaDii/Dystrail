use super::CampView;
use crate::a11y::set_status;
use crate::game::{
    CampConfig, CampOutcome, EndgameTravelCfg, GameState, camp_forage_with_endgame,
    camp_repair_hack, camp_repair_spare, camp_rest_with_endgame, camp_therapy, can_repair,
};
use crate::i18n;
use std::rc::Rc;
use yew::prelude::*;

pub fn build_on_action(
    game_state: Rc<GameState>,
    camp_config: Rc<CampConfig>,
    endgame_config: Rc<EndgameTravelCfg>,
    on_state_change: Callback<GameState>,
    on_close: Callback<()>,
    current_view: &UseStateHandle<CampView>,
    status_msg: &UseStateHandle<String>,
) -> Callback<u8> {
    let view_state = current_view.clone();
    let view_setter = view_state.setter();
    let status_state = status_msg.clone();
    let status_setter = status_state.setter();

    Callback::from(move |action: u8| {
        let view_current = *view_state;
        let mut new_state = (*game_state).clone();
        let outcome = match (view_current, action) {
            (CampView::Main, 1) => {
                camp_rest_with_endgame(&mut new_state, &camp_config, &endgame_config)
            }
            (CampView::Main, 2) => {
                if can_repair(&new_state, &camp_config) {
                    view_setter.set(CampView::Repair);
                    return;
                }
                CampOutcome {
                    message: i18n::t("camp.announce.no_breakdown"),
                    rested: false,
                    supplies_delta: 0,
                }
            }
            (CampView::Main, 3) => {
                camp_forage_with_endgame(&mut new_state, &camp_config, &endgame_config)
            }
            (CampView::Main, 4) => camp_therapy(&mut new_state, &camp_config),
            (CampView::Main, 0) => {
                on_close.emit(());
                return;
            }
            (CampView::Repair, 1) => {
                if let Some(breakdown) = &new_state.breakdown {
                    let part = breakdown.part;
                    let result = camp_repair_spare(&mut new_state, &camp_config, part);
                    view_setter.set(CampView::Main);
                    result
                } else {
                    CampOutcome {
                        message: i18n::t("camp.announce.no_breakdown"),
                        rested: false,
                        supplies_delta: 0,
                    }
                }
            }
            (CampView::Repair, 2) => {
                let result = camp_repair_hack(&mut new_state, &camp_config);
                view_setter.set(CampView::Main);
                result
            }
            (CampView::Repair, 0) => {
                view_setter.set(CampView::Main);
                return;
            }
            _ => return,
        };

        status_setter.set(outcome.message.clone());
        set_status(&outcome.message);
        on_state_change.emit(new_state);

        if matches!(view_current, CampView::Repair)
            || (matches!(action, 1 | 3 | 4) && matches!(view_current, CampView::Main))
        {
            on_close.emit(());
        }
    })
}
