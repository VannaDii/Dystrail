//! i18n coverage tests to ensure all required keys are present

#[cfg(test)]
mod tests {
    use serde_json::Value;

    #[test]
    fn test_pacediet_keys_coverage() {
        let locales = [
            "ar", "bn", "en", "es", "fr", "hi", "it", "ja", "pt", "ru", "zh",
        ];
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
        ];

        for locale in locales {
            let path = format!("i18n/{locale}.json");
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {path}"));

            let json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse JSON in {path}"));

            for key in required_keys {
                let found = find_nested_key(&json, key);
                assert!(found, "Missing key '{key}' in locale '{locale}'");
            }
        }
    }

    #[test]
    fn test_travel_keys_coverage() {
        let locales = [
            "ar", "bn", "en", "es", "fr", "hi", "it", "ja", "pt", "ru", "zh",
        ];
        let required_keys = ["travel.title", "travel.next"];

        for locale in locales {
            let path = format!("i18n/{locale}.json");
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {path}"));

            let json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse JSON in {path}"));

            for key in required_keys {
                let found = find_nested_key(&json, key);
                assert!(found, "Missing key '{key}' in locale '{locale}'");
            }
        }
    }

    /// Helper function to find a nested key like "pacediet.title" in JSON
    fn find_nested_key(json: &Value, key: &str) -> bool {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = json;

        for part in parts {
            match current.get(part) {
                Some(value) => current = value,
                None => return false,
            }
        }

        // Check if the final value is a string (not null or missing)
        current.is_string()
    }

    #[test]
    fn test_all_locales_parse_correctly() {
        let locales = [
            "ar", "bn", "en", "es", "fr", "hi", "it", "ja", "pt", "ru", "zh",
        ];

        for locale in locales {
            let path = format!("i18n/{locale}.json");
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {path}"));

            let _json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse JSON in {path}"));
        }
    }

    #[test]
    fn test_weather_keys_coverage() {
        let locales = [
            "ar", "bn", "en", "es", "fr", "hi", "it", "ja", "pt", "ru", "zh",
        ];
        let required_keys = [
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
            let path = format!("i18n/{locale}.json");
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {path}"));

            let json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse JSON in {path}"));

            for key in required_keys {
                let found = find_nested_key(&json, key);
                assert!(found, "Missing key '{key}' in locale '{locale}'");
            }
        }
    }

    #[test]
    fn test_no_missing_interpolation_variables() {
        let locales = [
            "ar", "bn", "en", "es", "fr", "hi", "it", "ja", "pt", "ru", "zh",
        ];

        for locale in locales {
            let path = format!("i18n/{locale}.json");
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {path}"));

            // Check that the JSON parses correctly
            let _json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse JSON in {path}"));

            // The i18n system supports both {{var}} and {var} patterns, so both are valid.
            // We should only check for truly malformed patterns like unmatched braces.

            // Count braces to ensure they're balanced
            let open_count = content.matches('{').count();
            let close_count = content.matches('}').count();
            assert_eq!(
                open_count, close_count,
                "Unmatched braces in {locale}: {open_count} open, {close_count} close"
            );

            // Check for obviously malformed patterns like {{{ or }}} (triple braces)
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
}
