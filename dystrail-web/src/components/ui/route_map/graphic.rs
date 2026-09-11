//! Census state geometry and OSM road polylines share one Albers projection.
use crate::{
    game::{
        GameState,
        route::{RoadPoint, RoadRoute},
    },
    i18n,
};
use std::fmt::Write;
use yew::prelude::*;
fn marker(route: &RoadRoute, miles: f32) -> (f32, f32) {
    for p in route.points.windows(2) {
        if miles <= p[1].mile {
            let ratio = ((miles - p[0].mile) / (p[1].mile - p[0].mile).max(0.001)).clamp(0.0, 1.0);
            return (
                (p[1].x - p[0].x).mul_add(ratio, p[0].x),
                (p[1].y - p[0].y).mul_add(ratio, p[0].y),
            );
        }
    }
    route.points.last().map_or((0.0, 0.0), |p| (p.x, p.y))
}
fn points(points: impl Iterator<Item = &'static RoadPoint>) -> String {
    points
        .map(|p| format!("{},{}", p.x, p.y))
        .collect::<Vec<_>>()
        .join(" ")
}
fn viewport(route: &RoadRoute) -> (f32, f32, f32, f32) {
    let left = route
        .points
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min)
        - 28.0;
    let right = route.points.iter().map(|p| p.x).fold(0.0, f32::max) + 55.0;
    let top = route
        .points
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        - 30.0;
    let bottom = route.points.iter().map(|p| p.y).fold(0.0, f32::max) + 30.0;
    (left, top, right - left, bottom - top)
}
pub fn render(gs: &GameState) -> Html {
    let Some(route) = crate::game::route::for_state(gs) else {
        return Html::default();
    };
    let miles = crate::game::route::physical_miles(gs);
    let (x, y) = marker(route, miles);
    let (left, top, width, height) = viewport(route);
    let mut trace = points(route.points.iter().filter(|p| p.mile <= miles));
    let _ = write!(trace, " {x},{y}");
    html! {<svg direction="ltr" class="us-route-map" data-route={route.id.clone()} style={format!("--map-ratio:{}",width/height)} viewBox={format!("{left} {top} {width} {height}")} xmlns="http://www.w3.org/2000/svg" role="img" aria-label={i18n::t("route.map")}>
        <image href={crate::paths::asset_path("static/img/map/us-states.svg")} x="0" y="0" width="1000" height="660"/>
        <polyline points={points(route.points.iter())} class="map-road-casing"/>
        <polyline points={points(route.points.iter())} class="map-road-future"/>
        <polyline points={trace} class="route-traced" pathLength="100"/>
        {for route.stops.iter().map(|town|html!{<g data-town={town.name.clone()} data-mile={town.mile.to_string()}><title>{if town.name=="D.C."{town.name.clone()}else{format!("{}, {}",town.name,town.state)}}</title><rect x={(town.x-2.5).to_string()} y={(town.y-2.5).to_string()} width="5" height="5" fill="#fff1b5" stroke="#263e3b" stroke-width="1"/></g>})}
        {super::labels::render(gs,route,true)}
        <g class="route-player-marker" data-x={x.to_string()} data-y={y.to_string()} transform={format!("translate({x} {y})")}><circle r="10" fill="#f4c861" stroke="#17353a" stroke-width="3"/><path d="M-6-3h9l4 4v4H-6z" fill="#138cd0" stroke="#122b35" stroke-width="1.5"/><path d="M-4-2h3v3h-3zM1-2h2l2 3H1z" fill="#cce7df"/></g>
    </svg>}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vehicle_marker_stays_on_real_road_and_reaches_dc() {
        for route in crate::game::route::routes() {
            let start = route.points.first().unwrap();
            let end = route.points.last().unwrap();
            assert_eq!(marker(route, 0.0), (start.x, start.y));
            assert_eq!(marker(route, route.total_miles + 1.0), (end.x, end.y));
            let (left, top, width, height) = viewport(route);
            for point in &route.points {
                assert!(point.x >= left && point.x <= left + width);
                assert!(point.y >= top && point.y <= top + height);
            }
        }
    }
}
