use dystrail_web::components::footer::Footer;
use dystrail_web::components::header::Header;
use dystrail_web::components::modal::Modal;
use dystrail_web::components::ui::seed_footer::SeedFooter;
use dystrail_web::game::seed::encode_friendly;
use futures::executor::block_on;
use yew::html::ChildrenRenderer;
use yew::{AttrValue, Callback, LocalServerRenderer};

#[test]
fn header_renders_language_and_actions() {
    dystrail_web::i18n::set_lang("en");
    let props = dystrail_web::components::header::Props {
        on_open_save: Callback::noop(),
        on_lang_change: Callback::noop(),
        current_lang: "en".to_string(),
        high_contrast: false,
        on_toggle_hc: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<Header>::with_props(props).render());
    assert!(html.contains("lang-select"));
    assert!(html.contains("save-open-btn"));
}

#[test]
fn footer_renders_copy() {
    dystrail_web::i18n::set_lang("en");
    let html = block_on(LocalServerRenderer::<Footer>::new().render());
    assert!(html.contains("<footer>"));
}

#[test]
fn modal_renders_when_open_and_skips_when_closed() {
    dystrail_web::i18n::set_lang("en");
    let open_props = dystrail_web::components::modal::Props {
        open: true,
        title: AttrValue::from("Title"),
        description: Some(AttrValue::from("Desc")),
        on_close: Callback::noop(),
        return_focus_id: None,
        children: ChildrenRenderer::default(),
    };
    let html = block_on(LocalServerRenderer::<Modal>::with_props(open_props).render());
    assert!(html.contains("modal__header"));
    assert!(html.contains("Desc"));

    let closed_props = dystrail_web::components::modal::Props {
        open: false,
        title: AttrValue::from("Title"),
        description: None,
        on_close: Callback::noop(),
        return_focus_id: None,
        children: ChildrenRenderer::default(),
    };
    let html = block_on(LocalServerRenderer::<Modal>::with_props(closed_props).render());
    assert!(!html.contains("modal-backdrop"));
}

#[test]
fn seed_footer_renders_share_code() {
    dystrail_web::i18n::set_lang("en");
    let seed = 42_u64;
    let share_code = encode_friendly(false, seed);
    let props = dystrail_web::components::ui::seed_footer::Props {
        seed,
        is_deep_mode: false,
        children: ChildrenRenderer::default(),
    };
    let html = block_on(LocalServerRenderer::<SeedFooter>::with_props(props).render());
    assert!(html.contains("seed-footer"));
    assert!(html.contains(&share_code));
}
