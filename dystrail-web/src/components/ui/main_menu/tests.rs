use super::MainMenu;
use super::MainMenuProps;
use futures::executor::block_on;
use yew::LocalServerRenderer;

#[test]
fn main_menu_renders_all_entries_with_seed() {
    crate::i18n::set_lang("en");
    let html = block_on(
        LocalServerRenderer::<MainMenu>::with_props(MainMenuProps {
            seed_text: Some("CL-TEST42".to_string()),
            on_select: None,
        })
        .render(),
    );

    assert!(
        html.contains("Seed: CL-TEST42"),
        "Rendered menu should include helper text with seed, got: {html}"
    );
    for key in [
        "data-key=\"1\"",
        "data-key=\"2\"",
        "data-key=\"3\"",
        "data-key=\"4\"",
        "data-key=\"5\"",
        "data-key=\"6\"",
        "data-key=\"7\"",
        "data-key=\"8\"",
        "data-key=\"0\"",
    ] {
        assert!(
            html.contains(key),
            "Expected menu item with {key} in rendered HTML: {html}"
        );
    }
    assert!(
        html.contains("Main Menu"),
        "Rendered markup should expose localized menu title: {html}"
    );
}
