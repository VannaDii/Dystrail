//! A full travel scene, never an additional strip or panel in the normal play screen.
mod graphic;
mod labels;
pub mod location;
#[cfg(test)]
mod tests;
use crate::{
    game::{GameState, route},
    i18n,
};
use yew::prelude::*;
#[derive(Properties)]
pub struct Props {
    pub state: std::rc::Rc<GameState>,
    pub on_continue: Callback<MouseEvent>,
    pub on_pause: Callback<MouseEvent>,
    pub automatic: bool,
    pub running: bool,
    #[prop_or_default]
    pub children: Children,
}
impl PartialEq for Props {
    fn eq(&self, o: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.state, &o.state)
            && self.on_continue == o.on_continue
            && self.on_pause == o.on_pause
            && self.automatic == o.automatic
            && self.running == o.running
            && self.children == o.children
    }
}
#[function_component(RouteMap)]
pub fn route_map(p: &Props) -> Html {
    crate::i18n::use_language();
    let gs = &p.state;
    let route = route::for_state(gs);
    let miles = route::physical_miles(gs);
    let origin = route
        .and_then(|r| r.stops.first())
        .map_or("", |s| s.name.as_str());
    html! {<>
        <super::stats_bar::StatsBar part={super::stats_bar::HudPart::Resources} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region} />
        <section class="map-scene" aria-labelledby="map-title" data-automatic={p.automatic.to_string()} data-running={p.running.to_string()} data-region={format!("{:?}",gs.region)}>
        <div class="map-scene-heading"><div><p class="eyebrow">{format!("{} · {}",i18n::t(route::region_key(gs.region)),i18n::tr("route.day",Some(&std::collections::BTreeMap::from([("day",gs.day.to_string().as_str())]))))}</p><h1 id="map-title">{i18n::t("route.title")}</h1></div><div class="map-heading-baseline"><p class="map-current-location">{location::location(gs)}</p><p class="map-attribution"><a href="https://www.census.gov/geographies/mapping-files/time-series/geo/cartographic-boundary.html" target="_blank" rel="noopener noreferrer">{"U.S. Census"}</a>{" · "}<a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener noreferrer">{"© OpenStreetMap contributors"}</a></p></div></div>
        <figure class="map-scene-art">{graphic::render(gs)}<figcaption>{i18n::tr(&format!("route.origin_satire.{}",route.map_or("staffer",|r|r.id.as_str())),Some(&std::collections::BTreeMap::from([("origin",origin)])))}</figcaption></figure>
        <div class="map-scene-itinerary"><div class="map-distance"><span>{i18n::t("route.traveled")}</span><strong>{location::distance(miles)}</strong><span>{route.map_or_else(String::new,|r|location::distance((r.total_miles-miles).max(0.0)))}{" · "}{i18n::t("route.remaining")}</span></div><ol>{for route::upcoming(gs).take(2).map(|town|html!{<li><strong>{&town.name}</strong><span>{location::distance(f32::from(town.mile)-miles)}</span>if town.name=="D.C."{<small>{i18n::t("route.destination")}</small>}</li>})}</ol></div>
        <div class="map-scene-actions">
            if !p.automatic {<p>{i18n::t("route.help")}</p>}
            <div class="map-buttons">
                if p.running {<button onclick={p.on_pause.clone()}>{i18n::t("route.stay")}</button>}
                <button class="retro-btn-primary" onclick={p.on_continue.clone()}>{i18n::t(if p.automatic {"journey.resume"}else{"route.close"})}</button>
            </div>
            if p.automatic {<div class="map-auto-status">if p.running {{for p.children.iter()}}else{<p>{i18n::t("route.paused")}</p>}</div>}
        </div>
    </section><crate::app::journey_panel::JourneyPanel /></>}
}
