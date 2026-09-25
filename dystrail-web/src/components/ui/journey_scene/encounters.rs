//! Deliberate encounter-to-setting assignments; unknown events never inherit road art.
/// Reviewed current variants override the historical runtime event's location.
pub(super) fn shared_asset(id: &str) -> Option<&'static str> {
    Some(match id {
        "ENC-S27-B" | "ENC-S10-C" => "enc-motel",
        "ENC-S04-B" | "ENC-S07-A" => "enc-campground",
        "ENC-D02-A" | "ENC-D12-A" => "enc-civic-exterior",
        "ENC-C10-A" | "ENC-C12-A" | "ENC-C14-C" | "ENC-C16-A" |
        "ENC-D01-A" | "ENC-D01-B" | "ENC-D05-B" | "ENC-D05-C" |
        "ENC-D12-B" | "ENC-S08-A" | "ENC-S08-B" | "ENC-S16-A" |
        "ENC-C13-C" | "ENC-D03-C" | "ENC-S05-C" | "ENC-C12-B" | "ENC-D11-B" |
        "ENC-C11-C" | "ENC-D01-C" | "ENC-D04-C" | "ENC-C13-B" | "ENC-S31-C" => "enc-cafe",
        "ENC-C15-C" | "ENC-C17-C" | "ENC-D07-C" | "ENC-S02-B" |
        "ENC-S06-A" | "ENC-S29-A" | "ENC-S11-B" | "ENC-S18-A" | "ENC-S18-C" => "enc-library",
        "ENC-C11-B" | "ENC-D07-B" | "ENC-D10-A" | "ENC-S06-B" |
        "ENC-S11-C" | "ENC-S19-A" | "ENC-S25-A" | "ENC-S27-A" |
        "ENC-D06-A" | "ENC-D09-B" | "ENC-D11-C" | "ENC-S21-A" | "ENC-S21-C" | "ENC-S34-A" |
        "ENC-C09-B" | "ENC-C13-A" | "ENC-D03-B" | "ENC-D04-A" | "ENC-D04-B" | "ENC-D05-A" | "ENC-D08-C" | "ENC-D10-C" | "ENC-S21-B" | "ENC-S22-A" | "ENC-S22-C" | "ENC-S25-B" | "ENC-S25-C" | "ENC-S27-C" | "ENC-S28-B" | "ENC-S29-B" | "ENC-S29-C" | "ENC-S31-A" | "ENC-S31-B" | "ENC-S32-C" => "enc-service-counter",
        "ENC-S01-B" | "ENC-S07-B" | "ENC-S33-C" | "ENC-D13-C" => "enc-service",
        "ENC-D03-A" | "ENC-D06-C" | "ENC-D07-A" | "ENC-D09-C" | "ENC-D11-A" => "enc-rest-area",
        "ENC-C15-A" | "ENC-C15-B" | "ENC-C17-A" | "ENC-C17-B" |
        "ENC-S02-A" | "ENC-S02-C" | "ENC-S15-C" | "ENC-S20-A" | "ENC-S20-C" => "enc-community",
        "ENC-C18-A" | "ENC-C18-C" => "enc-civic",
        "ENC-S15-B" | "ENC-S23-C" => "enc-museum",
        _ => return None,
    })
}

pub(super) fn shared_aspect(id: &str) -> Option<&'static str> {
    shared_asset(id).or_else(|| {
        asset(id, false).filter(|name| matches!(*name,
            "enc-motel" | "enc-cafe" | "enc-library" | "enc-museum" |
            "enc-farm-office" | "enc-service-counter"))
    }).map(|name| match name {
        "enc-service" | "enc-community" | "enc-civic" => "2",
        "enc-rest-area" | "enc-campground" | "enc-civic-exterior" => "2.25",
        _ => "1.7777778",
    })
}

pub(super) fn asset(id: &str, _deep: bool) -> Option<&'static str> {
    if let Some(setting) = shared_asset(id) { return Some(setting); }
    Some(match id {
        // The rejected lander illustration stays excluded pending user review.
        "ENC-C04-A" => "enc-rest-area",
        "ENC-C02-A" | "ENC-C07-C" => "enc-community",
        "ENC-C08-A" => "enc-rest-area",
        "west_grant_translation"
        | "classic_media_training"
        | "classic_press_briefing"
        | "deep_media_ambush"
        | "deep_watch_party"
        | "sat_factcheck_shift" => "enc-media-workshop",
        "classic_civic_potluck"
        | "classic_mail_drop"
        | "classic_mutual_aid"
        | "classic_water_drive"
        | "fundraiser_detour"
        | "raw_milk"
        | "sat_cow_citations"
        | "sat_straw_reserve"
        | "sat_grant_groceries"
        | "sat_food_shelf" => "enc-community",
        "classic_bridge_crews" | "sat_bridge_bullets" => "enc-bridge",
        "west_desert_pressure"
        | "classic_service_station"
        | "deep_circuit_breaker"
        | "tariff_whiplash"
        | "sat_alternator_tariff"
        | "sat_hydration_pressure" => "enc-service",
        "deep_state_dirge" | "town_hall_drift" => "enc-civic",
        "classic_crossing_block_party" | "deep_beltway_fastpass" | "sat_straw_inspection" => {
            "enc-checkpoint"
        }
        "west_laboratory_overhead" | "clinic_triage" | "sat_bibliography_emergency" => "enc-clinic",
        "classic_mutual_aid_dispatch" | "classic_radio_phonebank" => "enc-radio",
        "classic_freeway_mural"
        | "classic_neighborhood_watch"
        | "classic_overpass_stage"
        | "classic_union_blockade"
        | "west_rail_replacement"
        | "west_wind_loyalty"
        | "sat_forecast_corrected"
        | "sat_weather_desk"
        | "sat_transit_ribbon" => "enc-street",
        "west_beef_passports"
        | "deep_waystation_boost"
        | "deep_rustbelt_convoy"
        | "sat_tariff_forklift" => "enc-convoy",
        "overnight_briefing" => "enc-night-briefing",
        "deep_grassroots_signal" | "sat_parking_gulf" | "sat_shower_force" => "enc-rest-area",
        // Authored locations take precedence over the encounter's historical ID.
        "deep_secure_line" => "enc-motel",
        "classic_press_pool_qna"
        | "sat_billion_pothole"
        | "sat_press_pool_radio"
        | "sat_cabinet_guest" => "enc-cafe",
        "deep_field_intel"
        | "deep_memorandum_dump"
        | "deep_watchdog_sync"
        | "sat_library_minimum" => "enc-library",
        "sat_museum_grant" | "sat_receipt_museum" => "enc-museum",
        "sat_corn_bullets" => "enc-farm-office",
        "beltway_briefing" | "sat_name_infrastructure" => "enc-service-counter",
        _ => {
            return crate::app::visual_content::runtime_for_unit(id)
                .and_then(|runtime| asset(runtime, _deep));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_shipped_encounter_has_an_existing_crew_free_atlas_cell() {
        let data: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../../static/assets/data/game.json")).unwrap();
        assert_eq!(data.len(), 65);
        for event in data {
            for deep in [false, true] {
                let name = asset(event["id"].as_str().unwrap(), deep).unwrap();
                assert!(!name.starts_with("road"));
                for hour in [8, 20] {
                    let (atlas, columns, rows, cell) =
                        super::super::composition::encounter_setting(name, hour, None).unwrap();
                    assert!(cell < columns * rows, "{name}: cell {cell}");
                    assert!(
                        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join(format!("static/img/journey/{atlas}.png"))
                            .exists(),
                        "{name}: {atlas}"
                    );
                }
            }
        }
        assert_eq!(
            asset("classic_media_training", false),
            Some("enc-media-workshop")
        );
        assert_eq!(asset("classic_mutual_aid", false), Some("enc-community"));
    }

    #[test]
    fn revised_encounter_locations_replace_historical_theme_assignments() {
        for deep in [false, true] {
            assert_eq!(asset("deep_secure_line", deep), Some("enc-motel"));
            assert_eq!(asset("sat_press_pool_radio", deep), Some("enc-cafe"));
            assert_eq!(asset("sat_corn_bullets", deep), Some("enc-farm-office"));
            assert_eq!(asset("deep_field_intel", deep), Some("enc-library"));
            assert_eq!(asset("sat_museum_grant", deep), Some("enc-museum"));
            assert_eq!(asset("beltway_briefing", deep), Some("enc-service-counter"));
            assert_eq!(asset("classic_radio_phonebank", deep), Some("enc-radio"));
            assert_eq!(asset("tariff_whiplash", deep), Some("enc-service"));
            assert_eq!(asset("sat_weather_desk", deep), Some("enc-street"));
            assert_eq!(asset("raw_milk", deep), Some("enc-community"));
        }
    }
}
