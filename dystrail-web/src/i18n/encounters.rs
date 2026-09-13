//! Localized presentation copy with a fallback for legacy and imported encounters.
#[must_use]
pub fn encounter_text(id: &str, field: &str, fallback: &str) -> String {
    let key = format!("encounter_copy.{id}.{field}");
    let text = super::t(&key);
    if text == key {
        fallback.to_owned()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn localized_copy_and_imported_fallback_stay_readable() {
        for (lang, expected) in [
            ("en", "Private Chat, Public Application"),
            ("it", "Chat privata, candidatura pubblica"),
            ("es", "Chat privado, solicitud pública"),
            ("ar", "دردشة خاصة، طلب عمل على الملأ"),
        ] {
            crate::i18n::set_lang(lang);
            assert_eq!(
                encounter_text("deep_secure_line", "name", "legacy"),
                expected
            );
            assert_eq!(
                encounter_text("imported_event", "desc", "Imported description"),
                "Imported description"
            );
            assert!(!encounter_text("deep_secure_line", "log_0", "legacy").contains("legacy"));
        }
        crate::i18n::set_lang("en");
    }
}
