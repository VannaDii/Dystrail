use super::*;
use crate::game::vehicle::{Breakdown, Part};
use futures::executor::block_on;
use std::rc::Rc;
use yew::LocalServerRenderer;

fn base_props(state: GameState) -> Props {
    Props {
        game_state: Rc::new(state),
        camp_config: Rc::new(CampConfig::default_config()),
        on_state_change: Callback::from(|_: GameState| {}),
        on_close: Callback::noop(),
    }
}

#[test]
fn camp_panel_main_view_renders_actions() {
    crate::i18n::set_lang("en");
    let props = base_props(GameState::default());

    let html = block_on(LocalServerRenderer::<CampPanel>::with_props(props).render());
    assert!(
        html.contains("Rest") && html.contains("Forage"),
        "main view should list key camp actions: {html}"
    );
}

#[test]
fn camp_panel_with_breakdown_starts_in_repair_view() {
    crate::i18n::set_lang("en");
    let mut game_state = GameState::default();
    game_state.day_state.travel.travel_blocked = true;
    game_state.breakdown = Some(Breakdown {
        part: Part::Battery,
        day_started: 3,
    });
    let props = base_props(game_state);

    let html = block_on(LocalServerRenderer::<CampPanel>::with_props(props).render());
    assert!(
        html.contains("Repair Vehicle") || html.contains("Use Spare"),
        "repair menu should surface when breakdown present: {html}"
    );
}
