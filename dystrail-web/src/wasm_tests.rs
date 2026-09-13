//! Registered browser tests for receipt presentation and real encounter effects.
use crate::components::ui::{encounter_card, stats_bar};
use futures::future::LocalBoxFuture;
use std::{cell::RefCell, rc::Rc, time::Duration};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn root() -> web_sys::Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    root
}

// Browser DOM handles stay on the browser's single thread.
fn wait_for<'a>(check: impl Fn() -> bool + 'a) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        for _ in 0..100 {
            if check() {
                return;
            }
            yew::platform::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            check(),
            "The browser component did not reach its expected state"
        );
    })
}

#[wasm_bindgen_test(async)]
fn receipt_hud_updates_without_replacing_the_stat_row() -> LocalBoxFuture<'static, ()> {
    Box::pin(async {
        crate::i18n::set_lang("en");
        let root = root();
        let mut props = yew::props! {stats_bar::Props {
            stats: crate::game::Stats::default(),
            receipts: 0,
            day: 1,
            region: crate::game::Region::PacificCoast,
            part: stats_bar::HudPart::Resources,
        }};
        let mut app =
            yew::Renderer::<stats_bar::StatsBar>::with_root_and_props(root.clone(), props.clone())
                .render();
        wait_for(|| root.query_selector(".critical-stats").unwrap().is_some()).await;
        let row = root.query_selector(".critical-stats").unwrap().unwrap();
        assert_eq!(row.child_element_count(), 7);
        props.receipts = 2;
        app.update(props);
        wait_for(|| {
            root.query_selector("[data-stat='ux.receipt'] dd")
                .unwrap()
                .unwrap()
                .text_content()
                .as_deref()
                == Some("2")
        })
        .await;
        assert!(
            row.is_same_node(
                root.query_selector(".critical-stats")
                    .unwrap()
                    .as_ref()
                    .map(AsRef::as_ref)
            )
        );
        app.destroy();
        root.remove();
    })
}

#[wasm_bindgen_test(async)]
fn evidence_choice_click_awards_what_the_button_offers() -> LocalBoxFuture<'static, ()> {
    Box::pin(async {
        crate::i18n::set_lang("en");
        let data =
            crate::game::EncounterData::from_json(include_str!("../static/assets/data/game.json"))
                .unwrap();
        let encounter = data
            .encounters
            .into_iter()
            .find(|e| e.id == "west_grant_translation")
            .unwrap();
        let state = Rc::new(RefCell::new(crate::game::GameState {
            current_encounter: Some(encounter.clone()),
            ..crate::game::GameState::default()
        }));
        let chosen = state.clone();
        let props = yew::props! {encounter_card::Props {
            encounter,
            stats: state.borrow().stats.clone(),
            on_choice: yew::Callback::from(move |index| chosen.borrow_mut().apply_choice(index)),
        }};
        let root = root();
        let app = yew::Renderer::<encounter_card::EncounterCard>::with_root_and_props(
            root.clone(),
            props,
        )
        .render();
        wait_for(|| {
            root.query_selector_all(".encounter-choice button")
                .unwrap()
                .length()
                == 3
        })
        .await;
        let button = root
            .query_selector_all(".encounter-choice button")
            .unwrap()
            .item(1)
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        assert!(button.text_content().unwrap().contains("Receipts +1"));
        button.click();
        assert_eq!(state.borrow().receipts, ["west_grant_translation"]);
        button.click();
        assert_eq!(state.borrow().receipts.len(), 1);
        app.destroy();
        root.remove();
    })
}
