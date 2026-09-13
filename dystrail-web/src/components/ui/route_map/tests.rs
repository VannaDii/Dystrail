use super::*;
use futures::executor::block_on;
use yew::LocalServerRenderer;

#[test]
fn map_shows_the_same_receipt_count_as_the_journey() {
    crate::i18n::set_lang("en");
    let state = GameState {
        receipts: vec!["published-budget".into(), "grant-record".into()],
        ..GameState::default()
    };
    let props = yew::props! {Props {
        state: std::rc::Rc::new(state),
        on_continue: Callback::noop(),
        on_pause: Callback::noop(),
        automatic: false,
        running: false,
    }};
    let html = block_on(LocalServerRenderer::<RouteMap>::with_props(props).render());
    let receipt = html
        .split("data-stat=\"ux.receipt\"")
        .nth(1)
        .expect("The map must retain the shared Receipts stat");
    assert!(
        receipt
            .split("</div>")
            .next()
            .unwrap()
            .contains("<dd>2</dd>"),
        "The map must show the two collected receipts"
    );
}
