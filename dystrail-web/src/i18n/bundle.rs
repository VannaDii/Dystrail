use crate::i18n::locales::{is_rtl_lang, load_translations};
use serde_json::Value;
use std::cell::RefCell;

pub struct I18nBundle {
    pub lang: String,
    pub rtl: bool,
    pub translations: Value,
    pub fallback: Value,
}

fn build_bundle(lang: &str) -> Option<I18nBundle> {
    let rtl = is_rtl_lang(lang);

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
    #[cfg(all(not(test), target_arch = "wasm32"))]
    {
        web_sys::window()
            .and_then(|win| win.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("dystrail.locale").ok().flatten())
            .unwrap_or_else(|| "en".to_string())
    }

    #[cfg(any(test, not(target_arch = "wasm32")))]
    {
        "en".to_string()
    }
}

thread_local! {
    pub(super) static CURRENT: RefCell<I18nBundle> = RefCell::new({
        let initial = saved_lang();
        build_bundle(&initial).unwrap_or_else(|| build_bundle("en").unwrap_or_else(fallback_bundle))
    });
}

pub(super) fn with_bundle<R>(f: impl FnOnce(&I18nBundle) -> R) -> R {
    CURRENT.with(|cell| f(&cell.borrow()))
}

fn replace_bundle(bundle: I18nBundle) {
    CURRENT.with(|cell| cell.replace(bundle));
}

/// Set the current language for internationalization
///
/// Changes the active language bundle and updates the DOM lang/dir attributes.
/// Persists the language choice to localStorage for future sessions.
pub fn set_lang(lang: &str) {
    if let Some(bundle) = build_bundle(lang) {
        replace_bundle(bundle);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.document_element() {
                    with_bundle(|read| {
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
    with_bundle(|bundle| bundle.lang.clone())
}

/// Check if the current language uses right-to-left text direction
///
/// Returns true for RTL languages like Arabic, Hebrew, etc.
#[must_use]
pub fn is_rtl() -> bool {
    with_bundle(|bundle| bundle.rtl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_bundle_loads_requested_locale() {
        let bundle = build_bundle("ar").expect("bundle should load");
        assert_eq!(bundle.lang, "ar");
        assert!(bundle.rtl);
        assert!(bundle.translations.is_object());
        assert!(bundle.fallback.is_object());
    }

    #[test]
    fn fallback_bundle_defaults_to_en() {
        let bundle = fallback_bundle();
        assert_eq!(bundle.lang, "en");
        assert!(!bundle.rtl);
        assert!(bundle.translations.is_object());
    }
}
