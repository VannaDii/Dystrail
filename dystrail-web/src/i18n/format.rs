#[cfg(target_arch = "wasm32")]
use crate::i18n::bundle::with_bundle;
#[cfg(target_arch = "wasm32")]
use js_sys::{Date, Function, Intl, Object, Reflect};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Format a percentage value for display
///
/// Takes a percentage value and formats it using the current locale's percentage formatting.
/// Returns a localized string representation of the percentage.
#[must_use]
pub fn fmt_pct(pct: u8) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        with_bundle(|bundle| {
            let locales = js_sys::Array::new();
            locales.push(&JsValue::from_str(&bundle.lang));
            let options = Object::new();
            let _ = Reflect::set(
                &options,
                &JsValue::from_str("style"),
                &JsValue::from_str("percent"),
            );
            let nf = Intl::NumberFormat::new(&locales, &options);
            nf.format()
                .call1(&nf, &JsValue::from_f64(f64::from(pct) / 100.0))
                .ok()
                .and_then(|value| value.as_string())
                .unwrap_or_else(|| format!("{pct}%"))
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        format!("{pct}%")
    }
}

/// Format a number using the current locale via Intl
#[must_use]
pub fn fmt_number(num: f64) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        with_bundle(|bundle| {
            let locales = {
                let arr = js_sys::Array::new();
                arr.push(&JsValue::from_str(&bundle.lang));
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
        with_bundle(|bundle| {
            // Date-only sources describe a calendar day, not UTC midnight.
            // Local noon preserves that day when the browser formats it.
            let value = if date_iso.len() == 10 {
                format!("{date_iso}T12:00:00")
            } else {
                date_iso.to_owned()
            };
            let date = Date::new(&JsValue::from_str(&value));
            date.to_locale_date_string(&bundle.lang, &JsValue::UNDEFINED)
                .as_string()
                .unwrap_or_else(|| date_iso.to_string())
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        date_iso.to_string()
    }
}

/// Format whole-dollar currency (USD) using the current locale via Intl.
#[must_use]
pub fn fmt_currency(cents: i64) -> String {
    fn fallback_usd(cents: i64) -> String {
        let sign = if cents < 0 { "-" } else { "" };
        let abs = cents.unsigned_abs();
        let whole = abs / 100;
        format!("{sign}${whole}")
    }

    let cents = crate::game::numbers::whole_dollar_cents(cents);
    #[cfg(target_arch = "wasm32")]
    let amount = i32::try_from(cents).ok().map(|v| f64::from(v) / 100.0);
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(amount) = amount {
            return with_bundle(|bundle| {
                let locales = {
                    let arr = js_sys::Array::new();
                    arr.push(&JsValue::from_str(&bundle.lang));
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
                for key in ["minimumFractionDigits", "maximumFractionDigits"] {
                    let _ = Reflect::set(&opts, &JsValue::from_str(key), &JsValue::from_f64(0.0));
                }
                let nf = Intl::NumberFormat::new(&locales, &opts);
                nf.format()
                    .call1(&nf, &JsValue::from_f64(amount))
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_else(|| fallback_usd(cents))
            });
        }
        fallback_usd(cents)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        fallback_usd(cents)
    }
}
