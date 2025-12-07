#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Intl, Object, Reflect};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LocaleMeta {
    pub code: &'static str,
    pub name: &'static str,
    pub rtl: bool,
}

const LOCALE_META: &[LocaleMeta] = &[
    LocaleMeta {
        code: "en",
        name: "English",
        rtl: false,
    },
    LocaleMeta {
        code: "it",
        name: "Italiano",
        rtl: false,
    },
    LocaleMeta {
        code: "es",
        name: "Español",
        rtl: false,
    },
    LocaleMeta {
        code: "ar",
        name: "العربية",
        rtl: true,
    },
    LocaleMeta {
        code: "zh",
        name: "中文",
        rtl: false,
    },
    LocaleMeta {
        code: "hi",
        name: "हिन्दी",
        rtl: false,
    },
    LocaleMeta {
        code: "fr",
        name: "Français",
        rtl: false,
    },
    LocaleMeta {
        code: "bn",
        name: "বাংলা",
        rtl: false,
    },
    LocaleMeta {
        code: "pt",
        name: "Português",
        rtl: false,
    },
    LocaleMeta {
        code: "ru",
        name: "Русский",
        rtl: false,
    },
    LocaleMeta {
        code: "ja",
        name: "日本語",
        rtl: false,
    },
    LocaleMeta {
        code: "de",
        name: "Deutsch",
        rtl: false,
    },
    LocaleMeta {
        code: "id",
        name: "Bahasa Indonesia",
        rtl: false,
    },
    LocaleMeta {
        code: "jv",
        name: "Basa Jawa",
        rtl: false,
    },
    LocaleMeta {
        code: "ko",
        name: "한국어",
        rtl: false,
    },
    LocaleMeta {
        code: "mr",
        name: "मराठी",
        rtl: false,
    },
    LocaleMeta {
        code: "pa",
        name: "ਪੰਜਾਬੀ",
        rtl: false,
    },
    LocaleMeta {
        code: "ta",
        name: "தமிழ்",
        rtl: false,
    },
    LocaleMeta {
        code: "te",
        name: "తెలుగు",
        rtl: false,
    },
    LocaleMeta {
        code: "tr",
        name: "Türkçe",
        rtl: false,
    },
];

const LOCALE_TABLE: &[(&str, &str)] = &[
    ("en", include_str!("../i18n/en.json")),
    ("it", include_str!("../i18n/it.json")),
    ("es", include_str!("../i18n/es.json")),
    ("ar", include_str!("../i18n/ar.json")),
    ("zh", include_str!("../i18n/zh.json")),
    ("hi", include_str!("../i18n/hi.json")),
    ("fr", include_str!("../i18n/fr.json")),
    ("bn", include_str!("../i18n/bn.json")),
    ("pt", include_str!("../i18n/pt.json")),
    ("ru", include_str!("../i18n/ru.json")),
    ("ja", include_str!("../i18n/ja.json")),
    ("de", include_str!("../i18n/de.json")),
    ("id", include_str!("../i18n/id.json")),
    ("jv", include_str!("../i18n/jv.json")),
    ("ko", include_str!("../i18n/ko.json")),
    ("mr", include_str!("../i18n/mr.json")),
    ("pa", include_str!("../i18n/pa.json")),
    ("ta", include_str!("../i18n/ta.json")),
    ("te", include_str!("../i18n/te.json")),
    ("tr", include_str!("../i18n/tr.json")),
];

pub struct I18nBundle {
    pub lang: String,
    pub rtl: bool,
    translations: Value,
    fallback: Value,
}

fn load_translations(lang: &str) -> Option<Value> {
    let bundle = LOCALE_TABLE
        .iter()
        .find_map(|(code, data)| (*code == lang).then_some(*data))
        .unwrap_or(LOCALE_TABLE[0].1);

    serde_json::from_str(bundle).ok()
}

fn build_bundle(lang: &str) -> Option<I18nBundle> {
    let rtl = LOCALE_META.iter().any(|m| m.code == lang && m.rtl);

    let fallback = load_translations("en")?;
    let translations = load_translations(lang)?;

    Some(I18nBundle {
        lang: lang.to_string(),
        rtl,
        translations,
        fallback,
    })
}

/// Supported locales with their native names and direction metadata.
#[must_use]
pub const fn locales() -> &'static [LocaleMeta] {
    LOCALE_META
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
        web_sys::window()
            .and_then(|win| win.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("dystrail.locale").ok().flatten())
            .unwrap_or_else(|| "en".to_string())
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
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.document_element() {
                    CURRENT.with(|cell| {
                        let read = cell.borrow();
                        let _ = el.set_attribute("lang", &read.lang);
                        let _ = el.set_attribute("dir", if read.rtl { "rtl" } else { "ltr" });
                    });
                }
            }
            if let Some(storage) =
                web_sys::window().and_then(|win| win.local_storage().ok().flatten())
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

fn get_nested_value<'a>(obj: &'a Value, key: &str) -> Option<&'a Value> {
    let keys: Vec<&str> = key.split('.').collect();
    let mut current = obj;

    for k in keys {
        match current.get(k) {
            Some(value) => current = value,
            None => return None,
        }
    }
    Some(current)
}

fn plural_category(lang: &str, count: f64) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let locales = {
            let arr = Array::new();
            arr.push(&JsValue::from_str(lang));
            arr
        };
        let rules = Intl::PluralRules::new(&locales, &Object::new());
        match rules.select(count).as_string() {
            Some(selected) => selected,
            None => {
                if (count - 1.0).abs() < f64::EPSILON {
                    "one".to_string()
                } else if count.abs() < f64::EPSILON {
                    "zero".to_string()
                } else {
                    "other".to_string()
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = lang;
        if (count - 1.0).abs() < f64::EPSILON {
            "one".to_string()
        } else if count.abs() < f64::EPSILON {
            "zero".to_string()
        } else {
            "other".to_string()
        }
    }
}

fn render_value(value: &Value, lang: &str, args: Option<&BTreeMap<&str, &str>>) -> Option<String> {
    let mut text = match value {
        Value::String(s) => s.clone(),
        Value::Object(map) => {
            // Prefer plural categories if count provided
            if let Some(count_str) = args.and_then(|m| m.get("count")).copied() {
                if let Ok(count) = count_str.parse::<f64>() {
                    let category = plural_category(lang, count);
                    if let Some(s) = map.get(&category).and_then(Value::as_str) {
                        s.to_string()
                    } else if let Some(default) = map.get("_").and_then(Value::as_str) {
                        default.to_string()
                    } else {
                        return None;
                    }
                } else {
                    map.get("_")
                        .and_then(Value::as_str)
                        .map(std::string::ToString::to_string)?
                }
            } else if let Some(default) = map.get("_").and_then(Value::as_str) {
                default.to_string()
            } else {
                return None;
            }
        }
        _ => return None,
    };

    if let Some(args_map) = args {
        for (k, v) in args_map {
            let ph1 = format!("{{{{{k}}}}}"); // {{var}}
            let ph2 = format!("{{{k}}}"); // {var}
            text = text.replace(&ph1, v);
            text = text.replace(&ph2, v);
        }
    }
    Some(text)
}

fn resolve(key: &str, args: Option<&BTreeMap<&str, &str>>) -> Option<String> {
    CURRENT.with(|cell| {
        let bundle = cell.borrow();
        get_nested_value(&bundle.translations, key)
            .and_then(|v| render_value(v, &bundle.lang, args))
            .or_else(|| {
                get_nested_value(&bundle.fallback, key)
                    .and_then(|v| render_value(v, &bundle.lang, args))
            })
    })
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
/// Supports template variable replacement using ordered key-value pairs.
/// Variables in the translated string use the format {key} or {{key}}.
#[must_use]
pub fn tr(key: &str, args: Option<&BTreeMap<&str, &str>>) -> String {
    resolve(key, args).unwrap_or_else(|| key.to_string())
}

/// Format a percentage value for display
///
/// Takes a percentage value and formats it using the current locale's percentage formatting.
/// Returns a localized string representation of the percentage.
#[must_use]
pub fn fmt_pct(pct: u8) -> String {
    fmt_number(pct.into())
}

/// Format a number using the current locale via Intl
#[must_use]
pub fn fmt_number(num: f64) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        CURRENT.with(|c| {
            let lang = c.borrow().lang.clone();
            let locales = {
                let arr = Array::new();
                arr.push(&JsValue::from_str(&lang));
                arr
            };
            let nf = Intl::NumberFormat::new(&locales, &Object::new());
            let format_fn: Function = nf.format();
            format_fn
                .call1(&nf, &JsValue::from_f64(num))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| num.to_string())
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        num.to_string()
    }
}

/// Format an ISO 8601 date string using the current locale (browser-side)
#[must_use]
pub fn fmt_date_iso(date_iso: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        CURRENT.with(|c| {
            let lang = c.borrow().lang.clone();
            let date = js_sys::Date::new(&JsValue::from_str(date_iso));
            date.to_locale_date_string(&lang, &JsValue::UNDEFINED)
                .as_string()
                .unwrap_or_else(|| date_iso.to_string())
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        date_iso.to_string()
    }
}

/// Format currency (USD) using the current locale via Intl
#[must_use]
pub fn fmt_currency(cents: i64) -> String {
    fn fallback_usd(cents: i64) -> String {
        let sign = if cents < 0 { "-" } else { "" };
        let abs = cents.abs();
        let whole = abs / 100;
        let frac = abs % 100;
        format!("{sign}${whole}.{frac:02}")
    }

    let amount = i32::try_from(cents).ok().map(|v| f64::from(v) / 100.0);
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(amount) = amount {
            return CURRENT.with(|c| {
                let lang = c.borrow().lang.clone();
                let locales = {
                    let arr = Array::new();
                    arr.push(&JsValue::from_str(&lang));
                    arr
                };
                let opts = Object::new();
                let _ = Reflect::set(
                    &opts,
                    &JsValue::from_str("style"),
                    &JsValue::from_str("currency"),
                );
                let _ = Reflect::set(
                    &opts,
                    &JsValue::from_str("currency"),
                    &JsValue::from_str("USD"),
                );
                let nf = Intl::NumberFormat::new(&locales, &opts);
                nf.format()
                    .call1(&nf, &JsValue::from_f64(amount))
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_else(|| format!("{amount:.2}"))
            });
        }
        fallback_usd(cents)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        amount.map_or_else(|| fallback_usd(cents), |a| format!("{a:.2}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_selection_defaults() {
        let mut map = serde_json::Map::new();
        map.insert("one".into(), Value::String("one cat".into()));
        map.insert("other".into(), Value::String("{count} cats".into()));
        let value = Value::Object(map);
        let mut args = BTreeMap::new();
        args.insert("count", "1");
        let one = render_value(&value, "en", Some(&args)).unwrap();
        assert_eq!(one, "one cat");
        args.insert("count", "3");
        let many = render_value(&value, "en", Some(&args)).unwrap();
        assert_eq!(many, "3 cats");
    }

    #[test]
    fn interpolation_handles_braced_forms() {
        let value = Value::String("Hello, {name}! {{name}}!".into());
        let mut args = BTreeMap::new();
        args.insert("name", "Tester");
        let resolved = render_value(&value, "en", Some(&args)).unwrap();
        assert_eq!(resolved, "Hello, Tester! Tester!");
    }
}
