use super::*;
use futures::executor::block_on;
use std::rc::Rc;
use yew::LocalServerRenderer;

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

    let blitz = selection_outcome(&pacing, 3);
    assert!(matches!(
        blitz,
        Some(SelectionOutcome::Pace(PaceId::Blitz, _))
    ));

    let quiet = selection_outcome(&pacing, 4);
    assert!(matches!(
        quiet,
        Some(SelectionOutcome::Diet(DietId::Quiet, _))
    ));

    let mixed = selection_outcome(&pacing, 5);
    assert!(matches!(
        mixed,
        Some(SelectionOutcome::Diet(DietId::Mixed, _))
    ));

    assert!(selection_outcome(&pacing, 0).is_none());
    assert!(selection_outcome(&pacing, 42).is_none());
}

#[function_component(PaceDietHarness)]
fn pace_diet_harness() -> Html {
    crate::i18n::set_lang("en");
    let invoked = use_state(|| false);
    let pacing = Rc::new(PacingConfig::default_config());
    let game_state = Rc::new(GameState::default());
    let pace_state = use_state(|| game_state.pace);
    let diet_state = use_state(|| game_state.diet);
    let back_state = use_state(|| false);
    let status = use_state(String::new);
    let focus_state = use_state(|| 1_u8);

    let on_pace = Callback::from(move |pace| pace_state.set(pace));
    let on_diet = Callback::from(move |diet| diet_state.set(diet));
    let on_back = Callback::from(move |()| back_state.set(true));

    let on_activate = activate_handler(
        pacing.clone(),
        on_pace.clone(),
        on_diet.clone(),
        on_back.clone(),
        status,
    );
    let on_focus = focus_handler(focus_state.clone());
    let _ = keydown_handler(focus_state, on_activate.clone());

    if !*invoked {
        invoked.set(true);
        on_activate.emit(1);
        on_activate.emit(4);
        on_activate.emit(0);
        on_focus.emit(2);
    }

    html! {
        <PaceDietPanel
            game_state={game_state}
            pacing_config={pacing}
            on_pace_change={on_pace}
            on_diet_change={on_diet}
            on_back={on_back}
        />
    }
}

#[test]
fn pace_diet_panel_renders_and_updates_callbacks() {
    let html = block_on(LocalServerRenderer::<PaceDietHarness>::new().render());
    assert!(html.contains("pace-diet-panel"));
}

#[test]
fn pace_diet_props_equality_compares_pace_and_diet() {
    let gs = Rc::new(GameState::default());
    let pacing = Rc::new(PacingConfig::default_config());
    let props = PaceDietPanelProps {
        game_state: gs.clone(),
        pacing_config: pacing.clone(),
        on_pace_change: Callback::noop(),
        on_diet_change: Callback::noop(),
        on_back: Callback::noop(),
    };
    let mut changed = (*gs).clone();
    changed.pace = PaceId::Blitz;
    let props_changed = PaceDietPanelProps {
        game_state: Rc::new(changed),
        pacing_config: pacing,
        on_pace_change: Callback::noop(),
        on_diet_change: Callback::noop(),
        on_back: Callback::noop(),
    };
    assert!(props == props);
    assert!(!(props == props_changed));
}
