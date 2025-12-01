//! i18n coverage tests to ensure all required keys are present

use serde_json::Value;
use std::collections::BTreeSet;

fn locale_codes() -> Vec<String> {
    let mut locales = Vec::new();
    let entries = std::fs::read_dir("i18n").expect("i18n directory should exist");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            locales.push(stem.to_string());
        }
    }
    locales.sort();
    locales
}

fn load_locale(locale: &str) -> (String, Value) {
    let path = format!("i18n/{locale}.json");
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {path}"));
    let json: Value =
        serde_json::from_str(&content).unwrap_or_else(|_| panic!("Failed to parse JSON in {path}"));
    (content, json)
}

fn find_nested_key(json: &Value, key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = json;

    for part in parts {
        match current.get(part) {
            Some(value) => current = value,
            None => return false,
        }
    }

    current.is_string() || current.is_object()
}

fn collect_keys(prefix: &str, value: &Value, out: &mut BTreeSet<String>) {
    if let Value::Object(map) = value {
        for (k, v) in map {
            let next_prefix = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            if v.is_object() {
                collect_keys(&next_prefix, v, out);
            } else {
                out.insert(next_prefix);
            }
        }
    }
}

#[test]
fn locales_have_matching_keys() {
    let locales = locale_codes();
    let (_, base_json) = load_locale("en");
    let mut base_keys = BTreeSet::new();
    collect_keys("", &base_json, &mut base_keys);

    for locale in locales {
        let (_, json) = load_locale(&locale);
        let mut keys = BTreeSet::new();
        collect_keys("", &json, &mut keys);
        for key in &base_keys {
            assert!(
                keys.contains(key),
                "Missing key '{key}' in locale '{locale}'"
            );
        }
    }
}

#[test]
fn required_feature_keys_exist() {
    let locales = locale_codes();
    let required_keys = [
        "pacediet.title",
        "pacediet.menu.pace_steady",
        "pacediet.menu.pace_heated",
        "pacediet.menu.pace_blitz",
        "pacediet.menu.diet_quiet",
        "pacediet.menu.diet_mixed",
        "pacediet.menu.diet_doom",
        "pacediet.menu.back",
        "pacediet.announce.pace_set",
        "pacediet.announce.diet_set",
        "pacediet.tooltips.steady",
        "pacediet.tooltips.heated",
        "pacediet.tooltips.blitz",
        "pacediet.tooltips.quiet",
        "pacediet.tooltips.mixed",
        "pacediet.tooltips.doom",
        "travel.title",
        "travel.next",
        "weather.title",
        "weather.announce",
        "weather.states.Clear",
        "weather.states.Storm",
        "weather.states.HeatWave",
        "weather.states.ColdSnap",
        "weather.states.Smoke",
        "weather.effects.sup",
        "weather.effects.san",
        "weather.effects.pants",
        "weather.effects.enc",
        "weather.details.header",
        "weather.details.state",
        "weather.details.effects",
        "weather.details.gear",
        "weather.details.notes",
        "weather.details.back",
        "weather.gear.storm",
        "weather.gear.smoke",
        "weather.gear.cold",
        "weather.notes.storm_crossings",
    ];

    for locale in locales {
        let (_, json) = load_locale(&locale);
        for key in required_keys {
            assert!(
                find_nested_key(&json, key),
                "Missing key '{key}' in locale '{locale}'"
            );
        }
    }
}

#[test]
fn locales_have_balanced_templates() {
    for locale in locale_codes() {
        let (content, _json) = load_locale(&locale);
        let open_count = content.matches('{').count();
        let close_count = content.matches('}').count();
        assert_eq!(
            open_count, close_count,
            "Unmatched braces in {locale}: {open_count} open, {close_count} close"
        );
        assert!(
            !content.contains("{{{"),
            "Found triple opening brace in {locale}"
        );
        assert!(
            !content.contains("}}}"),
            "Found triple closing brace in {locale}"
        );
    }
}
