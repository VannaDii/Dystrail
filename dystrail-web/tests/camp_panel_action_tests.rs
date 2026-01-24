use dystrail_web::components::ui::camp_panel::{CampView, build_on_action};
use dystrail_web::game::{Breakdown, CampConfig, EndgameTravelCfg, GameState, Part};
use futures::executor::block_on;
use std::rc::Rc;
use yew::LocalServerRenderer;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
struct ActionHarnessProps {
    action: u8,
    start_view: CampView,
    with_breakdown: bool,
}

#[function_component(ActionHarness)]
fn action_harness(props: &ActionHarnessProps) -> Html {
    dystrail_web::i18n::set_lang("en");
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

fn render_action(props: ActionHarnessProps) -> String {
    block_on(LocalServerRenderer::<ActionHarness>::with_props(props).render())
}

#[test]
fn main_view_close_emits_close_without_state_change() {
    let html = render_action(ActionHarnessProps {
        action: 0,
        start_view: CampView::Main,
        with_breakdown: false,
    });
    assert!(html.contains("data-view=\"main\""));
    assert!(html.contains("data-closed=\"true\""));
    assert!(html.contains("data-changed=\"false\""));
    assert!(html.contains("data-status=\"\""));
}

#[test]
fn repair_view_without_breakdown_reports_status_and_closes() {
    let html = render_action(ActionHarnessProps {
        action: 1,
        start_view: CampView::Repair,
        with_breakdown: false,
    });
    assert!(html.contains("data-view=\"repair\""));
    assert!(html.contains("data-closed=\"true\""));
    assert!(html.contains("data-changed=\"true\""));
    assert!(html.contains("data-status=\""));
}

#[test]
fn repair_hack_updates_state_and_closes() {
    let html = render_action(ActionHarnessProps {
        action: 2,
        start_view: CampView::Repair,
        with_breakdown: false,
    });
    assert!(html.contains("data-view=\"repair\""));
    assert!(html.contains("data-closed=\"true\""));
    assert!(html.contains("data-changed=\"true\""));
    assert!(html.contains("data-status=\""));
}

#[test]
fn main_view_close_with_breakdown_sets_state() {
    let html = render_action(ActionHarnessProps {
        action: 0,
        start_view: CampView::Main,
        with_breakdown: true,
    });
    assert!(html.contains("data-view=\"main\""));
    assert!(html.contains("data-closed=\"true\""));
}
