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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Breakdown, EndgameTravelCfg, GameState, Part};
    use futures::executor::block_on;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[derive(Properties, PartialEq)]
    struct ActionHarnessProps {
        action: u8,
        start_view: CampView,
        with_breakdown: bool,
    }

    #[function_component(ActionHarness)]
    fn action_harness(props: &ActionHarnessProps) -> Html {
        crate::i18n::set_lang("en");
        let current_view = use_state(|| props.start_view);
        let status_msg = use_state(String::new);
        let mut base_state = GameState::default();
        if props.with_breakdown {
            base_state.breakdown = Some(Breakdown {
                part: Part::Tire,
                day_started: 0,
            });
        }
        let game_state = use_state(|| base_state);
        let closed = use_mut_ref(|| false);
        let changed = use_mut_ref(|| false);
        let invoked = use_mut_ref(|| false);
        let camp_config = Rc::new(CampConfig::default_config());
        let endgame_config = Rc::new(EndgameTravelCfg::default_config());
        let on_state_change = {
            let game_state = game_state.clone();
            let changed = changed.clone();
            Callback::from(move |state: GameState| {
                game_state.set(state);
                *changed.borrow_mut() = true;
            })
        };
        let on_close = {
            let closed = closed.clone();
            Callback::from(move |()| *closed.borrow_mut() = true)
        };
        let on_action = build_on_action(
            Rc::new((*game_state).clone()),
            camp_config,
            endgame_config,
            on_state_change,
            on_close,
            &current_view,
            &status_msg,
        );

        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            on_action.emit(props.action);
        }

        let view_label = match *current_view {
            CampView::Main => "main",
            CampView::Repair => "repair",
        };
        let status = (*status_msg).clone();
        let closed = (*closed.borrow()).to_string();
        let changed = (*changed.borrow()).to_string();
        html! {
            <div
                data-view={view_label}
                data-status={status}
                data-closed={closed}
                data-changed={changed}
            />
        }
    }

    #[test]
    fn rest_action_sets_status_and_closes() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 1,
                start_view: CampView::Main,
                with_breakdown: false,
            })
            .render(),
        );
        assert!(html.contains("data-view=\"main\""));
        assert!(html.contains("data-closed=\"true\""));
        assert!(html.contains("data-changed=\"true\""));
        assert!(html.contains("data-status=\""));
    }

    #[test]
    fn repair_action_switches_view_when_breakdown_present() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 2,
                start_view: CampView::Main,
                with_breakdown: true,
            })
            .render(),
        );
        assert!(html.contains("data-closed=\"false\""));
        assert!(html.contains("data-changed=\"false\""));
        assert!(html.contains("data-status=\"\""));
    }

    #[test]
    fn repair_spare_returns_to_main_and_closes() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 1,
                start_view: CampView::Repair,
                with_breakdown: true,
            })
            .render(),
        );
        assert!(html.contains("data-closed=\"true\""));
        assert!(html.contains("data-changed=\"true\""));
    }

    #[test]
    fn repair_back_returns_to_main_without_close() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 0,
                start_view: CampView::Repair,
                with_breakdown: true,
            })
            .render(),
        );
        assert!(html.contains("data-closed=\"false\""));
        assert!(html.contains("data-changed=\"false\""));
    }

    #[test]
    fn forage_action_closes_and_updates_state() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 3,
                start_view: CampView::Main,
                with_breakdown: false,
            })
            .render(),
        );
        assert!(html.contains("data-closed=\"true\""));
        assert!(html.contains("data-changed=\"true\""));
    }

    #[test]
    fn therapy_action_closes_and_updates_state() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 4,
                start_view: CampView::Main,
                with_breakdown: false,
            })
            .render(),
        );
        assert!(html.contains("data-closed=\"true\""));
        assert!(html.contains("data-changed=\"true\""));
    }

    #[test]
    fn repair_action_reports_missing_breakdown_without_close() {
        let html = block_on(
            LocalServerRenderer::<ActionHarness>::with_props(ActionHarnessProps {
                action: 2,
                start_view: CampView::Main,
                with_breakdown: false,
            })
            .render(),
        );
        assert!(html.contains("data-closed=\"false\""));
        assert!(html.contains("data-changed=\"true\""));
        assert!(html.contains("data-status=\""));
    }
}
