use crate::i18n::bundle::with_bundle;
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Intl, Object};

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
            arr.push(&wasm_bindgen::JsValue::from_str(lang));
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
            let ph1 = format!("{{{{{k}}}}}");
            let ph2 = format!("{{{k}}}");
            text = text.replace(&ph1, v);
            text = text.replace(&ph2, v);
        }
    }
    Some(text)
}

fn resolve(key: &str, args: Option<&BTreeMap<&str, &str>>) -> Option<String> {
    with_bundle(|bundle| {
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
