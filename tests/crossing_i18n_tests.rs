use serde_json::Value;

/// Test that all crossing i18n keys exist in all locales
#[test]
fn test_crossing_i18n_coverage() {
    let locales = ["ar", "en", "fr", "it", "pt", "zh", "bn", "es", "hi", "ja", "ru"];
    let expected_keys = get_expected_crossing_keys();

    for locale in &locales {
        let file_path = format!("i18n/{}.json", locale);
        let content = std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path));

        let json: Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON in {}", file_path));

        let mut missing_keys = Vec::new();
        for key in &expected_keys {
            if !key_exists_in_json(&json, key) {
                missing_keys.push(key.clone());
            }
        }

        if !missing_keys.is_empty() {
            panic!(
                "Missing i18n keys in {}: {}",
                file_path,
                missing_keys.join(", ")
            );
        }
    }
}

/// Test interpolation structure is present in English locale
#[test]
fn test_crossing_i18n_interpolation_structure() {
    let content = std::fs::read_to_string("i18n/en.json")
        .expect("Failed to read English locale file");

    let json: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse English JSON");

    let cross = json.get("cross").expect("Missing cross section");

    // Test that interpolatable strings contain placeholders
    let detour_option = cross.get("options").and_then(|o| o.get("detour"))
        .and_then(|v| v.as_str())
        .expect("Missing detour option");

    assert!(detour_option.contains("{days}"), "detour option should have days placeholder");
    assert!(detour_option.contains("{supplies}"), "detour option should have supplies placeholder");
    assert!(detour_option.contains("{pants}"), "detour option should have pants placeholder");

    let bribe_option = cross.get("options").and_then(|o| o.get("bribe"))
        .and_then(|v| v.as_str())
        .expect("Missing bribe option");

    assert!(bribe_option.contains("{cost}"), "bribe option should have cost placeholder");
}

/// Test that configuration JSON is valid
#[test]
fn test_crossing_config_structure() {
    let config_content = std::fs::read_to_string("static/assets/data/crossings.json")
        .expect("crossings.json should exist");

    let config: serde_json::Value = serde_json::from_str(&config_content)
        .expect("crossings.json should be valid JSON");

    // Should have types section
    assert!(config.get("types").is_some());

    // Should have checkpoint and bridge_out
    let types = config.get("types").unwrap();
    assert!(types.get("checkpoint").is_some());
    assert!(types.get("bridge_out").is_some());

    // Should have global_mods and money sections
    assert!(config.get("global_mods").is_some());
    assert!(config.get("money").is_some());

    // Validate checkpoint structure
    let checkpoint = types.get("checkpoint").unwrap();
    assert!(checkpoint.get("detour").is_some());
    assert!(checkpoint.get("bribe").is_some());
    assert!(checkpoint.get("permit").is_some());

    // Validate detour has required fields
    let detour = checkpoint.get("detour").unwrap();
    assert!(detour.get("days").is_some());
    assert!(detour.get("supplies").is_some());
    assert!(detour.get("pants").is_some());
}

fn get_expected_crossing_keys() -> Vec<String> {
    vec![
        "cross.title".to_string(),
        "cross.prompt".to_string(),
        "cross.options.detour".to_string(),
        "cross.options.bribe".to_string(),
        "cross.options.permit".to_string(),
        "cross.options.back".to_string(),
        "cross.desc.detour".to_string(),
        "cross.desc.bribe".to_string(),
        "cross.desc.permit".to_string(),
        "cross.announce.detour_applied".to_string(),
        "cross.announce.bribe_paid_passed".to_string(),
        "cross.announce.bribe_paid_failed".to_string(),
        "cross.announce.permit_used_receipt".to_string(),
        "cross.announce.permit_used_tag".to_string(),
        "cross.announce.no_receipt_or_tag".to_string(),
        "cross.announce.insufficient_funds".to_string(),
        "cross.policy.shutdown".to_string(),
        "cross.types.checkpoint".to_string(),
        "cross.types.bridge_out".to_string(),
    ]
}

fn key_exists_in_json(json: &Value, key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = json;

    for part in parts {
        match current.get(part) {
            Some(value) => current = value,
            None => return false,
        }
    }

    current.is_string()
}