//! Shared journey reference panel; decisions never hide inventory or the journal.
use super::{Phase, state::AppState, view::handlers::AppHandlers};
use crate::{
    components::ui::travel_panel::{TravelPanel, tabs},
    game::{DietId, GameState, PaceId, PacingConfig},
    i18n,
};
use std::rc::Rc;
use yew::prelude::*;

pub struct PanelData {
    state: Rc<GameState>,
    logs: Vec<String>,
    receipt: Html,
    controls: Html,
    pacing: Rc<PacingConfig>,
    pace: Callback<PaceId>,
    diet: Callback<DietId>,
    moving: bool,
    pause: Callback<()>,
    detail: UseStateHandle<tabs::Selection>,
}

#[derive(Clone, Default)]
pub struct PanelContext(Option<Rc<PanelData>>);
impl PartialEq for PanelContext {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }
    }
}

#[must_use]
pub fn context(app: &AppState, handlers: &AppHandlers) -> PanelContext {
    if matches!(
        *app.phase,
        Phase::Boot | Phase::Persona | Phase::Crew | Phase::Outfitting | Phase::Result
    ) {
        return PanelContext::default();
    }
    let Some(session) = app.session.as_ref() else {
        return PanelContext::default();
    };
    let gs = session.state();
    let receipt = if app.aftermath.is_none() {
        super::turn::render_turn_entries(gs)
    } else {
        Html::default()
    };
    let running = app.travel_running.clone();
    PanelContext(Some(Rc::new(PanelData {
        state: Rc::new(gs.clone()),
        logs: (*app.logs).clone(),
        receipt,
        controls: if *app.phase == Phase::Map {
            Html::default()
        } else {
            super::flow::controls(app)
        },
        pacing: Rc::new((*app.pacing_config).clone()),
        pace: handlers.pace_change.clone(),
        diet: handlers.diet_change.clone(),
        moving: *app.phase != Phase::Map && (app.pending_turn.is_some() || *app.travel_running),
        pause: Callback::from(move |()| running.set(false)),
        detail: app.journey_detail.clone(),
    })))
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(JourneyPanel)]
pub fn journey_panel(p: &Props) -> Html {
    crate::i18n::use_language();
    let context = use_context::<PanelContext>().unwrap_or_default();
    let Some(data) = context.0 else {
        return Html::default();
    };
    let detail = &data.detail;
    html! {<section class="journey-reference" aria-label={i18n::t("journey.report")}>
        <div class="travel-toolbar"><div class="journey-controls">
            {tabs::render(detail)}
            if data.moving {<div class="transit-caption"><button onclick={{let cb=data.pause.clone();Callback::from(move |_|cb.emit(()))}}>{i18n::t("journey.pause")}</button></div>}
            else {{data.controls.clone()}}
        </div></div>
        {for p.children.iter()}
        if detail.expanded && (!data.moving || detail.tab!=3) {
            <TravelPanel detail={detail.tab} receipt={data.receipt.clone()} logs={data.logs.clone()} game_state={Some(data.state.clone())} pacing_config={data.pacing.clone()} on_pace_change={data.pace.clone()} on_diet_change={data.diet.clone()} />
        }
    </section>}
}
