//! Deliberate encounter-to-setting assignments; unknown events never inherit road art.
pub(super) fn asset(id: &str, deep: bool) -> Option<&'static str> {
    Some(match id {
        "raw_milk" => {
            if deep {
                "milk-deep"
            } else {
                "milk-classic"
            }
        }
        "classic_media_training"
        | "deep_media_ambush"
        | "sat_factcheck_shift"
        | "sat_weather_desk" => "enc-media-workshop",
        "classic_civic_potluck"
        | "classic_mutual_aid"
        | "classic_water_drive"
        | "fundraiser_detour"
        | "sat_straw_reserve"
        | "sat_grant_groceries"
        | "sat_food_shelf" => "enc-community",
        "classic_bridge_crews" | "sat_billion_pothole" | "sat_bridge_bullets" => "enc-bridge",
        "classic_service_station"
        | "deep_circuit_breaker"
        | "sat_alternator_tariff"
        | "sat_hydration_pressure"
        | "sat_shower_force" => "enc-service",
        "beltway_briefing"
        | "classic_press_briefing"
        | "classic_press_pool_qna"
        | "deep_state_dirge"
        | "deep_watchdog_sync"
        | "town_hall_drift"
        | "sat_library_minimum"
        | "sat_museum_grant"
        | "sat_name_infrastructure"
        | "sat_receipt_museum" => "enc-civic",
        "tariff_whiplash"
        | "deep_beltway_fastpass"
        | "sat_parking_gulf"
        | "sat_straw_inspection" => "enc-checkpoint",
        "clinic_triage" | "sat_bibliography_emergency" => "enc-clinic",
        "classic_mail_drop"
        | "classic_mutual_aid_dispatch"
        | "classic_neighborhood_watch"
        | "classic_radio_phonebank"
        | "deep_field_intel"
        | "deep_grassroots_signal"
        | "deep_memorandum_dump"
        | "deep_secure_line"
        | "sat_corn_bullets"
        | "sat_forecast_corrected"
        | "sat_press_pool_radio"
        | "sat_cabinet_guest" => "enc-radio",
        "classic_crossing_block_party"
        | "classic_freeway_mural"
        | "classic_overpass_stage"
        | "classic_union_blockade"
        | "sat_transit_ribbon" => "enc-street",
        "deep_waystation_boost" | "deep_rustbelt_convoy" | "sat_tariff_forklift" => "enc-convoy",
        "deep_watch_party" | "overnight_briefing" => "enc-night-briefing",
        "sat_cow_citations" => "milk-classic",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_shipped_encounter_has_an_existing_non_road_illustration() {
        let data: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../../static/assets/data/game.json")).unwrap();
        assert_eq!(data.len(), 59);
        for event in data {
            for deep in [false, true] {
                let name = asset(event["id"].as_str().unwrap(), deep).unwrap();
                assert!(!name.starts_with("road"));
                assert!(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join(format!("static/img/journey/{name}.png"))
                        .exists(),
                    "{name}"
                );
            }
        }
        assert_eq!(
            asset("classic_media_training", false),
            Some("enc-media-workshop")
        );
        assert_eq!(asset("classic_mutual_aid", false), Some("enc-community"));
    }
}
