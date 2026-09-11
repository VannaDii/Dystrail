//! Place only useful city labels, with collision avoidance around the dense eastern route.
use crate::game::{
    GameState,
    route::{self, RoadRoute, Settlement},
};
use yew::prelude::*;
struct Label<'a> {
    town: &'a Settlement,
    x: f32,
    y: f32,
    width: f32,
}
fn placements<'a>(gs: &GameState, road: &'a RoadRoute, detail: bool) -> Vec<Label<'a>> {
    let mut names = vec!["D.C."];
    if let Some(first) = road.stops.first() {
        names.push(&first.name);
    }
    names.extend(route::upcoming(gs).take(2).map(|s| s.name.as_str()));
    if let Some(town) = route::previous(gs) {
        names.push(&town.name);
    }
    names.extend(["Chicago", "Cleveland"]);
    let mut labels: Vec<Label<'a>> = Vec::new();
    for name in names {
        if labels.iter().any(|l| l.town.name == name) {
            continue;
        }
        let Some(town) = road.stops.iter().find(|s| s.name == name) else {
            continue;
        };
        let width = f32::from(u16::try_from(name.chars().count()).unwrap_or(30)) * 7.3 + 8.0;
        for (shift_x, shift_y) in [
            (7.0, -14.0),
            (7.0, 24.0),
            (-width - 7.0, -14.0),
            (-width - 7.0, 24.0),
            (7.0, -36.0),
            (-width - 7.0, 46.0),
        ] {
            let x = town.x + shift_x;
            let y = town.y + shift_y;
            if x < 10.0
                || x + width > 990.0
                || labels.iter().any(|l| {
                    x < l.x + l.width
                        && x + width > l.x
                        && y - 15.0 < l.y + 6.0
                        && y + 6.0 > l.y - 15.0
                })
            {
                continue;
            }
            labels.push(Label { town, x, y, width });
            break;
        }
        if !detail && labels.len() >= 6 {
            break;
        }
    }
    labels
}
pub fn render(gs: &GameState, road: &RoadRoute, detail: bool) -> Html {
    html! {<>{for placements(gs,road,detail).iter().map(|l|html!{<g class="map-city-label"><path d={format!("M{},{} L{},{}",l.town.x,l.town.y,l.x,l.y-4.0)} stroke="#193d49" stroke-width="1" fill="none"/><text class="map-city" x={l.x.to_string()} y={l.y.to_string()}>{&l.town.name}</text></g>})}</>}
}
