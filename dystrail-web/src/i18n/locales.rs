use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LocaleMeta {
    pub code: &'static str,
    pub name: &'static str,
    pub rtl: bool,
}

pub const LOCALE_META: &[LocaleMeta] = &[
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
    ("en", include_str!("../../i18n/en.json")),
    ("it", include_str!("../../i18n/it.json")),
    ("es", include_str!("../../i18n/es.json")),
    ("ar", include_str!("../../i18n/ar.json")),
    ("zh", include_str!("../../i18n/zh.json")),
    ("hi", include_str!("../../i18n/hi.json")),
    ("fr", include_str!("../../i18n/fr.json")),
    ("bn", include_str!("../../i18n/bn.json")),
    ("pt", include_str!("../../i18n/pt.json")),
    ("ru", include_str!("../../i18n/ru.json")),
    ("ja", include_str!("../../i18n/ja.json")),
    ("de", include_str!("../../i18n/de.json")),
    ("id", include_str!("../../i18n/id.json")),
    ("jv", include_str!("../../i18n/jv.json")),
    ("ko", include_str!("../../i18n/ko.json")),
    ("mr", include_str!("../../i18n/mr.json")),
    ("pa", include_str!("../../i18n/pa.json")),
    ("ta", include_str!("../../i18n/ta.json")),
    ("te", include_str!("../../i18n/te.json")),
    ("tr", include_str!("../../i18n/tr.json")),
];

/// Supported locales with their native names and direction metadata.
#[must_use]
pub const fn locales() -> &'static [LocaleMeta] {
    LOCALE_META
}

#[must_use]
pub fn is_rtl_lang(lang: &str) -> bool {
    LOCALE_META.iter().any(|m| m.code == lang && m.rtl)
}

pub fn load_translations(lang: &str) -> Option<Value> {
    let bundle = LOCALE_TABLE
        .iter()
        .find_map(|(code, data)| (*code == lang).then_some(*data))
        .unwrap_or(LOCALE_TABLE[0].1);

    serde_json::from_str(bundle).ok()
}
