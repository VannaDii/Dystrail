use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement, HtmlSelectElement};
use yew::Renderer;

use dystrail_web::app::App;
use dystrail_web::dom;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn ensure_app_root() -> web_sys::Element {
    let doc = dom::document().expect("document");
    if let Some(root) = doc.get_element_by_id("app") {
        let _ = root.set_inner_html("");
        return root;
    }
    let root = doc.create_element("div").expect("create app root");
    root.set_id("app");
    doc.body()
        .expect("document body")
        .append_child(&root)
        .expect("append app root");
    root
}

fn render_app() {
    // Default language must be set to populate translated header strings.
    dystrail_web::i18n::set_lang("en");
    Renderer::<App>::with_root(ensure_app_root()).render();
}

#[wasm_bindgen_test]
fn skip_link_points_to_main_landmark() {
    render_app();
    let doc = dom::document().expect("document");
    let skip = doc
        .query_selector("a[href='#main']")
        .expect("query skip link")
        .expect("skip link exists");
    let main = doc
        .get_element_by_id("main")
        .expect("main landmark exists");
    assert_eq!(main.tag_name(), "MAIN");
    assert_eq!(
        main.get_attribute("role").unwrap_or_default(),
        "main",
        "main landmark must carry explicit role"
    );
    assert_eq!(
        skip.get_attribute("href").unwrap_or_default(),
        "#main",
        "skip link must target the main landmark"
    );
}

#[wasm_bindgen_test]
fn language_toggle_updates_lang_and_dir() {
    render_app();
    let doc = dom::document().expect("document");
    let select: HtmlSelectElement = doc
        .get_element_by_id("lang-select")
        .expect("lang select")
        .dyn_into()
        .expect("cast to select");
    select.set_value("ar");
    select
        .dispatch_event(&Event::new("change").expect("change event"))
        .expect("dispatch change");

    let html = doc.document_element().expect("document element");
    assert_eq!(html.get_attribute("lang"), Some("ar".into()));
    assert_eq!(html.get_attribute("dir"), Some("rtl".into()));

    select.set_value("en");
    select
        .dispatch_event(&Event::new("change").expect("change event"))
        .expect("dispatch change");
    assert_eq!(html.get_attribute("lang"), Some("en".into()));
    assert_eq!(html.get_attribute("dir"), Some("ltr".into()));
}

#[wasm_bindgen_test]
fn high_contrast_toggle_sets_html_class() {
    render_app();
    let doc = dom::document().expect("document");
    let html = doc.document_element().expect("document element");
    assert!(
        !html.class_list().contains("hc"),
        "high contrast should be off by default"
    );
    let btn: HtmlElement = doc
        .query_selector(".hc-toggle")
        .expect("query toggle")
        .expect("toggle exists")
        .dyn_into()
        .expect("cast to element");
    btn.click();
    assert!(
        html.class_list().contains("hc"),
        "high contrast toggle should add .hc"
    );
}
