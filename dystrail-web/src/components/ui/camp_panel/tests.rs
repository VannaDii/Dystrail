use super::*;
use crate::game::vehicle::{Breakdown, Part};
use futures::executor::block_on;
use std::rc::Rc;
use yew::LocalServerRenderer;

fn base_props(state: GameState) -> Props {
    Props {
        gathering: Html::default(),
        game_state: Rc::new(state),
        camp_config: Rc::new(CampConfig::default_config()),
        on_state_change: Callback::from(|_: (GameState, String)| {}),
        on_close: Callback::noop(),
    }
}

#[test]
fn camp_panel_renders_each_shared_gathering_action_once() {
    crate::i18n::set_lang("en");
    let mut props = base_props(GameState::default());
    props.gathering = html! {
        <><button>{i18n::t("trail.forage")}</button><button>{i18n::t("trail.glean")}</button></>
    };

    let html = block_on(LocalServerRenderer::<CampPanel>::with_props(props).render());
    assert!(html.contains("Rest"));
    assert_eq!(html.matches(i18n::t("trail.forage").as_str()).count(), 1);
    assert_eq!(html.matches(i18n::t("trail.glean").as_str()).count(), 1);
}

#[test]
fn camp_panel_cannot_offer_camping_or_a_generic_repair_during_breakdown() {
    crate::i18n::set_lang("en");
    let mut game_state = GameState::default();
    game_state.day_state.travel.travel_blocked = true;
    game_state.breakdown = Some(Breakdown {
        part: Part::Battery,
        day_started: 3,
    });
    let props = base_props(game_state);

    let html = block_on(LocalServerRenderer::<CampPanel>::with_props(props).render());
    assert!(!html.contains("Handle breakdown"));
    assert!(!html.contains("camp-actions"));
}
