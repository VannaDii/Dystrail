#[cfg(target_arch = "wasm32")]
use dystrail_web::dom;
use dystrail_web::i18n;
use dystrail_web::router::Route;
use dystrail_web::{app::Phase, game::DataLoader, game::WebDataLoader};
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
#[test]
fn dom_helpers_handle_missing_window() {
    assert!(dom::window().is_some());
    assert!(dom::document().is_some());
}

#[test]
fn i18n_bundle_switches_and_formats() {
    i18n::set_lang("en");
    assert_eq!(i18n::current_lang(), "en");
    assert!(!i18n::is_rtl());

    let mut vars = BTreeMap::new();
    vars.insert("amount", "$10");
    let budget = i18n::tr("store.budget", Some(&vars));
    assert!(budget.contains("$10"));
    assert_eq!(i18n::t("missing.key"), "missing.key");

    assert_eq!(i18n::fmt_number(12.5), "12.5");
    assert_eq!(i18n::fmt_pct(40), "40");
    assert_eq!(i18n::fmt_date_iso("2025-01-01"), "2025-01-01");
    assert_eq!(i18n::fmt_currency(12345), "123.45");
    assert_eq!(i18n::fmt_currency(-250), "-2.50");
}

#[test]
fn i18n_locales_metadata_is_accessible() {
    let metas = i18n::locales();
    assert!(metas.iter().any(|m| m.code == "en"));
    assert!(metas.iter().any(|m| m.code == "ar" && m.rtl));
    i18n::set_lang("ar");
    assert!(i18n::is_rtl());
}

#[test]
fn route_menu_maps_to_menu_phase() {
    assert_eq!(Route::Menu.to_phase(), Some(Phase::Menu));
}

#[test]
fn web_data_loader_flags_unknown_config() {
    let loader = WebDataLoader;
    let err = loader
        .load_config::<Value>("missing-config")
        .expect_err("missing config should error");
    let msg = format!("{err}");
    assert!(msg.contains("Unknown config"));
}
