use crate::components::ui::otdeluxe_store_panel::OtDeluxeStorePanel;
use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{GameState, OtDeluxeStoreLineItem};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct OtDeluxeStorePageProps {
    pub state: Rc<GameState>,
    pub weather: WeatherBadge,
    pub on_purchase: Callback<Vec<OtDeluxeStoreLineItem>>,
    pub on_leave: Callback<()>,
}

impl PartialEq for OtDeluxeStorePageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state) && self.weather == other.weather
    }
}

#[function_component(OtDeluxeStorePage)]
pub fn otdeluxe_store_page(props: &OtDeluxeStorePageProps) -> Html {
    let stats = props.state.stats.clone();
    let day = props.state.day;
    let region = props.state.region;
    let exec_order = props.state.current_order;
    let persona_id = props.state.persona_id.clone();

    html! {
        <section data-testid="store-screen">
            <crate::components::ui::stats_bar::StatsBar
                {stats}
                {day}
                {region}
                exec_order={exec_order}
                persona_id={persona_id}
                weather={Some(props.weather.clone())}
            />
            <OtDeluxeStorePanel
                state={props.state.clone()}
                on_purchase={props.on_purchase.clone()}
                on_leave={props.on_leave.clone()}
            />
        </section>
    }
}
