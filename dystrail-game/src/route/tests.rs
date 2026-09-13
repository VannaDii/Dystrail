use super::*;
#[test]
fn real_routes_have_distinct_origins_ordered_mileage_and_a_dc_finish() {
    assert_eq!(routes().len(), 6);
    let origins: std::collections::BTreeSet<_> =
        routes().iter().map(|r| r.stops[0].name.as_str()).collect();
    assert_eq!(origins.len(), 6);
    for route in routes() {
        assert_eq!(route.stops.last().unwrap().name, "D.C.");
        assert!(["WA", "OR", "CA"].contains(&route.stops[0].state.as_str()));
        assert!(route.stops[0].lon < -117.0);
        assert!(route.total_miles > 2800.0);
        assert!(
            route
                .stops
                .iter()
                .map(|s| s.region.asset_key())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                >= 5
        );
        assert!(route.stops.windows(2).all(|p| p[0].mile < p[1].mile));
        assert!(route.points.windows(2).all(|p| p[0].mile <= p[1].mile));
        assert!(
            route
                .points
                .iter()
                .all(|p| (-125.0..=-76.0).contains(&p.lon) && (29.0..=49.0).contains(&p.lat))
        );
        assert!((route.points.last().unwrap().mile - route.total_miles).abs() < 0.01);
    }
}
#[test]
fn geography_and_scenes_follow_the_selected_route_and_survive_save() {
    let mut gs = GameState {
        persona_id: Some("journalist".into()),
        ..GameState::default()
    };
    gs.sync_route_location();
    assert_eq!(origin("journalist"), "Seattle");
    assert_eq!(gs.region, Region::PacificCoast);
    let route = for_state(&gs).unwrap();
    let chicago = route.stops.iter().find(|s| s.name == "Chicago").unwrap();
    gs.miles_traveled_actual =
        (f32::from(chicago.mile) + 1.0) / route.total_miles * gs.trail_distance;
    gs.sync_route_location();
    assert_eq!(gs.region, Region::RustBelt);
    assert_eq!(road_scene(&gs), "open-rustbelt-foundry");
    let frederick = route.stops.iter().find(|s| s.name == "Frederick").unwrap();
    gs.miles_traveled_actual =
        (f32::from(frederick.mile) + 1.0) / route.total_miles * gs.trail_distance;
    gs.sync_route_location();
    assert_eq!(gs.region, Region::Beltway);
    let loaded: GameState = serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
    assert_eq!(for_state(&loaded).unwrap().id, "journalist");
    let checkpoint = MapCheckpoint::current(&gs);
    gs.day += 5;
    assert_ne!(checkpoint, MapCheckpoint::current(&gs));
}
