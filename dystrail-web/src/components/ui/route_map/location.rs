//! Towns and route segments, rather than abstract mile markers.
use crate::{
    game::{GameState, route},
    i18n,
};
use std::collections::BTreeMap;
#[must_use]
pub fn location(gs: &GameState) -> String {
    if gs.miles_traveled_actual >= gs.trail_distance {
        return i18n::tr("route.at", Some(&BTreeMap::from([("town", "D.C.")])));
    }
    if let Some(town) = gs
        .continuity
        .route_services
        .stop
        .and_then(|mile| route::settlement(gs, mile))
    {
        return i18n::tr(
            "route.at",
            Some(&BTreeMap::from([("town", town.name.as_str())])),
        );
    }
    let from = route::previous(gs).map_or("", |s| s.name.as_str());
    route::upcoming(gs).next().map_or_else(
        || from.to_owned(),
        |to| {
            i18n::tr(
                "route.between",
                Some(&BTreeMap::from([("from", from), ("to", to.name.as_str())])),
            )
        },
    )
}
#[must_use]
pub fn distance(miles: f32) -> String {
    let miles = i18n::fmt_number(f64::from(miles.max(0.0).round()));
    i18n::tr(
        "route.miles",
        Some(&BTreeMap::from([("miles", miles.as_str())])),
    )
}
