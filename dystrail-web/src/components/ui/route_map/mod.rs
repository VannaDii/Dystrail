//! A full travel scene, never an additional strip or panel in the normal play screen.
mod graphic;
mod labels;
pub mod location;
use crate::{
    game::{GameState, route},
    i18n,
};
use yew::prelude::*;
#[derive(Properties)]
pub struct Props {
    pub state: std::rc::Rc<GameState>,
    pub on_continue: Callback<MouseEvent>,
}
impl PartialEq for Props {
    fn eq(&self, o: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.state, &o.state) && self.on_continue == o.on_continue
    }
}
#[function_component(RouteMap)]
pub fn route_map(p: &Props) -> Html {
    let gs = &p.state;
    let route = route::for_state(gs);
    let miles = route::physical_miles(gs);
    html! {<section class="map-scene" aria-labelledby="map-title" data-region={format!("{:?}",gs.region)}>
        <div class="map-scene-heading"><div><p class="eyebrow">{format!("{} · {}",i18n::t(route::region_key(gs.region)),i18n::tr("route.day",Some(&std::collections::BTreeMap::from([("day",gs.day.to_string().as_str())]))))}</p><h1 id="map-title">{i18n::t("route.title")}</h1><p>{location::location(gs)}</p></div></div>
        <figure class="map-scene-art">{graphic::render(gs)}<figcaption>{i18n::t(&format!("route.origin_satire.{}",route.map_or("staffer",|r|r.id.as_str())))}</figcaption></figure>
        <div class="map-scene-itinerary"><div class="map-distance"><span>{i18n::t("route.traveled")}</span><strong>{location::distance(miles)}</strong><span>{route.map_or_else(String::new,|r|location::distance((r.total_miles-miles).max(0.0)))}{" · "}{i18n::t("route.remaining")}</span></div><ol>{for route::upcoming(gs).take(2).map(|town|html!{<li><strong>{&town.name}</strong><span>{location::distance(f32::from(town.mile)-miles)}</span>if town.name=="D.C."{<small>{i18n::t("route.destination")}</small>}else{{super::leg_summary::services()}}</li>})}</ol></div>
        <div class="map-scene-actions"><p>{i18n::t("route.help")}</p><button class="retro-btn-primary" onclick={p.on_continue.clone()}>{i18n::t("route.continue")}</button></div>
        <p class="map-attribution"><a href="https://www.census.gov/geographies/mapping-files/time-series/geo/cartographic-boundary.html" target="_blank" rel="noopener noreferrer">{"U.S. Census"}</a>{" · "}<a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener noreferrer">{"© OpenStreetMap contributors"}</a></p>
    </section>}
}
