use crate::game::{DietId, GameState, PaceId, PacingConfig};
use crate::i18n;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub on_travel: Callback<()>,
    pub logs: Vec<String>,
    #[prop_or_default]
    pub receipt: Html,
    pub game_state: Option<Rc<GameState>>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<PaceId>,
    pub on_diet_change: Callback<DietId>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.logs == other.logs
            && match (&self.game_state, &other.game_state) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

/// Action controls keep the available choices and their effects visible together.
#[function_component(TravelPanel)]
pub fn travel_panel(p: &Props) -> Html {
    let tab = use_state(|| 3_u8);
    let Some(gs) = p.game_state.as_ref() else {
        return Html::default();
    };
    let travel = {
        let cb = p.on_travel.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {<section class="travel-controls">
        <div class="travel-command"><div><crate::components::ui::context_help::ContextHelp title={i18n::t("play.journey")} text={i18n::t("play.travel_help")} /></div>
        <button class="retro-btn-primary" onclick={travel}>{i18n::t(if gs.breakdown.is_some(){"ux.resolve_vehicle"}else{"journey.resume"})}</button></div>
        if let Some(b)=&gs.breakdown {<div class="repair-notice" role="alert"><strong>{i18n::t(b.part.key())}</strong><p>{i18n::t("vehicle.announce.blocked")}</p><p>{i18n::t("play.repair_help")}</p></div>}
        <div class="travel-tabs" aria-label={i18n::t("ux.details")}>
            {for [(2,"journey.assess"),(0,"play.inventory"),(1,"play.journal")].into_iter().map(|(n,key)|{let tab=tab.clone();html!{<button aria-pressed={(*tab==n).to_string()} onclick={Callback::from(move |_|tab.set(if *tab==n {3}else{n}))}>{i18n::t(key)}</button>}})}
        </div>
        if *tab==2 {{super::pace::render_settings(gs,&p.pacing_config,&p.on_pace_change,&p.on_diet_change)}{super::weather::render_weather_details(gs)}}
        if *tab==0 {{super::status::render_status(gs)}} else if *tab==1 {<section class="journal" aria-label={i18n::t("play.journal")}><ol class="trail-log" role="log">
            {for gs.continuity.journal.iter().rev().take(50).map(|entry|html!{<li><strong>{format!("{} · {} · {:02}:{:02} · {}",entry.title,entry.day,entry.minute/60,entry.minute%60,entry.place)}</strong><p>{&entry.message}</p><p>{crate::app::history::entry_deltas(entry)}</p><dl class="receipt-details">{for entry.details.iter().filter(|(name,_)|name!=&i18n::t("journey.when")).map(|(key,value)|html!{<div><dt>{key}</dt><dd>{value}</dd></div>})}</dl></li>})}
            if gs.continuity.journal.is_empty() {{for p.logs.iter().rev().take(30).map(|line|html!{<li>{crate::i18n::log_message(line)}</li>})}}
            if p.logs.is_empty(){<li>{i18n::t("play.journal_empty")}</li>}
        </ol></section>}
        if *tab!=1 {{p.receipt.clone()}}
    </section>}
}
