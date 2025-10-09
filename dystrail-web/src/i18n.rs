use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct I18nBundle {
    pub lang: String,
    pub rtl: bool,
    translations: Value,
    fallback: Value,
}

#[allow(clippy::match_same_arms)]
fn load_translations(lang: &str) -> Option<Value> {
    let bundle = match lang {
        "en" => include_str!("../i18n/en.json"),
        "it" => include_str!("../i18n/it.json"),
        "es" => include_str!("../i18n/es.json"),
        "ar" => include_str!("../i18n/ar.json"),
        "zh" => include_str!("../i18n/zh.json"),
        "hi" => include_str!("../i18n/hi.json"),
        "fr" => include_str!("../i18n/fr.json"),
        "bn" => include_str!("../i18n/bn.json"),
        "pt" => include_str!("../i18n/pt.json"),
        "ru" => include_str!("../i18n/ru.json"),
        "ja" => include_str!("../i18n/ja.json"),
        _ => include_str!("../i18n/en.json"), // Default to English
    };

    serde_json::from_str(bundle).ok()
}

fn build_bundle(lang: &str) -> Option<I18nBundle> {
    let rtl = lang == "ar";

    let fallback = load_translations("en")?;
    let translations = load_translations(lang)?;

    Some(I18nBundle {
        lang: lang.to_string(),
        rtl,
        translations,
        fallback,
    })
}

fn fallback_bundle() -> I18nBundle {
    let fallback = load_translations("en").unwrap_or(Value::Object(serde_json::Map::new()));

    I18nBundle {
        lang: "en".to_string(),
        rtl: false,
        translations: fallback.clone(),
        fallback,
    }
}

fn saved_lang() -> String {
    #[cfg(not(test))]
    {
        let win = web_sys::window();
        if let Some(win) = win
            && let Ok(Some(storage)) = win.local_storage()
            && let Ok(Some(lang)) = storage.get_item("dystrail.locale")
        {
            return lang;
        }
        "en".to_string()
    }
    #[cfg(test)]
    {
        "en".to_string()
    }
}

thread_local! {
    static CURRENT: RefCell<I18nBundle> = RefCell::new({
        let initial = saved_lang();
        build_bundle(&initial).unwrap_or_else(|| build_bundle("en").unwrap_or_else(fallback_bundle))
    });
}

/// Set the current language for internationalization
///
/// Changes the active language bundle and updates the DOM lang/dir attributes.
/// Persists the language choice to localStorage for future sessions.
pub fn set_lang(lang: &str) {
    if let Some(b) = build_bundle(lang) {
        CURRENT.with(|cell| cell.replace(b));
        #[cfg(not(test))]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                let _ = doc.document_element().map(|el| {
                    CURRENT.with(|cell| {
                        let read = cell.borrow();
                        let _ = el.set_attribute("lang", &read.lang);
                        let _ = el.set_attribute("dir", if read.rtl { "rtl" } else { "ltr" });
                    });
                });
            }
            if let Some(win) = web_sys::window()
                && let Ok(Some(storage)) = win.local_storage()
            {
                let _ = storage.set_item("dystrail.locale", lang);
            }
        }
    }
}

/// Get the current active language code
///
/// Returns the two-letter language code for the currently active locale.
#[must_use]
pub fn current_lang() -> String {
    CURRENT.with(|c| c.borrow().lang.clone())
}

/// Check if the current language uses right-to-left text direction
///
/// Returns true for RTL languages like Arabic, Hebrew, etc.
#[must_use]
pub fn is_rtl() -> bool {
    CURRENT.with(|c| c.borrow().rtl)
}

fn get_nested_value(obj: &Value, key: &str) -> Option<String> {
    let keys: Vec<&str> = key.split('.').collect();
    let mut current = obj;

    for k in keys {
        match current.get(k) {
            Some(value) => current = value,
            None => return None,
        }
    }

    current.as_str().map(std::string::ToString::to_string)
}

/// Translate a key to the current language
///
/// Simple translation without variable substitution.
/// Falls back to English if key is not found in current language.
#[must_use]
pub fn t(key: &str) -> String {
    tr(key, None)
}

/// Translate a key with variable substitution
///
/// Supports template variable replacement using `HashMap` of key-value pairs.
/// Variables in the translated string use the format {key} or {{key}}.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn tr(key: &str, args: Option<&HashMap<&str, &str>>) -> String {
    CURRENT.with(|cell| {
        let b = cell.borrow();

        // Try to get from main translations first, then fallback
        let result =
            get_nested_value(&b.translations, key).or_else(|| get_nested_value(&b.fallback, key));

        match result {
            Some(mut text) => {
                // Handle template variables like {{var}} and {var}
                if let Some(args_map) = args {
                    for (k, v) in args_map {
                        let ph1 = format!("{{{{{k}}}}}"); // {{var}}
                        let ph2 = format!("{{{k}}}"); // {var}
                        text = text.replace(&ph1, v);
                        text = text.replace(&ph2, v);
                    }
                }
                text
            }
            None => key.to_string(),
        }
    })
}

/// Format a percentage value for display
///
/// Takes a percentage value and formats it using the current locale's percentage formatting.
/// Returns a localized string representation of the percentage.
#[must_use]
pub fn fmt_pct(pct: u8) -> String {
    let pct_str = pct.to_string();
    let mut map = HashMap::new();
    map.insert("pct", pct_str.as_str());
    tr("ui.loading", Some(&map))
}

// wasm i18n tests provided in tests/wasm
