use crate::components::ui::context_help::ContextHelp;
use crate::{
    game::{DietId, GameState, PaceId, PacingConfig},
    i18n,
};
use yew::prelude::*;

pub fn render_settings(
    gs: &GameState,
    cfg: &PacingConfig,
    on_pace: &Callback<PaceId>,
    on_diet: &Callback<DietId>,
) -> Html {
    let locked = gs.day_state.lifecycle.day_initialized;
    html! {<div class="daily-settings">
        <fieldset disabled={locked}><legend>{i18n::t("play.pace")}<ContextHelp title={i18n::t("play.pace")} text={i18n::t("play.pace_help")} /></legend><div class="setting-options">
        {for cfg.pace.iter().map(|p| {
            let id=match p.id.as_str(){"heated"=>PaceId::Heated,"blitz"=>PaceId::Blitz,_=>PaceId::Steady};
            let cb=on_pace.clone();
            html!{<button type="button" class={classes!("setting-option",(gs.pace==id).then_some("selected"))} aria-pressed={(gs.pace==id).to_string()} onclick={Callback::from(move |_|cb.emit(id))}>
                <strong>{i18n::t(&format!("play.{}",p.id))}</strong>
                <span>{format!("{} ×{:.2}",i18n::t("play.distance"),p.dist_mult)}</span>
                <small>{format!("{} {:+} · {} {:+}",i18n::t("ux.sanity"),p.sanity,i18n::t("ux.pants"),p.pants)}</small>
                <small>{format!("{} {:+.0}%",i18n::t("play.risk"),p.encounter_chance_delta*100.0)}</small>
            </button>}
        })}</div></fieldset>
        <fieldset disabled={locked}><legend>{i18n::t("play.diet")}<ContextHelp title={i18n::t("play.diet")} text={i18n::t("play.diet_help")} /></legend><div class="setting-options">
        {for cfg.diet.iter().map(|p| {
            let id=match p.id.as_str(){"quiet"=>DietId::Quiet,"doom"=>DietId::Doom,_=>DietId::Mixed};let cb=on_diet.clone();
            html!{<button type="button" class={classes!("setting-option",(gs.diet==id).then_some("selected"))} aria-pressed={(gs.diet==id).to_string()} onclick={Callback::from(move |_|cb.emit(id))}>
                <strong>{i18n::t(&format!("play.{}",p.id))}</strong>
                <span>{format!("{} {:+}",i18n::t("ux.sanity"),p.sanity)}</span>
                <small>{format!("{} {:+}",i18n::t("ux.pants"),p.pants)}</small>
                <small>{format!("{} {:+}%",i18n::t("play.receipts"),p.receipt_find_pct_delta)}</small>
            </button>}
        })}</div></fieldset>
        if locked {<p class="settings-note">{i18n::t("play.locked")}</p>}
    </div>}
}
