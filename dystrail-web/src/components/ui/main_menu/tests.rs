use super::MainMenu;
use super::MainMenuProps;
use futures::executor::block_on;
use yew::LocalServerRenderer;

#[test]
fn main_menu_renders_entries_and_helper() {
    crate::i18n::set_lang("en");
    let html = block_on(
        LocalServerRenderer::<MainMenu>::with_props(MainMenuProps { on_select: None }).render(),
    );

    for key in [
        "data-key=\"1\"",
        "data-key=\"2\"",
        "data-key=\"3\"",
        "data-key=\"4\"",
    ] {
        assert!(
            html.contains(key),
            "Expected menu item with {key} in rendered HTML: {html}"
        );
    }
    assert!(
        html.contains("menu-helper"),
        "Rendered markup should expose helper text: {html}"
    );
}
