use crate::components::ui::action_button::ActionButton;
#[cfg(test)]
mod tests;

use crate::game::{CampConfig, GameState, camp_rest};
use crate::i18n;
use std::{collections::BTreeMap, rc::Rc};
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub game_state: Rc<GameState>,
    pub camp_config: Rc<CampConfig>,
    pub on_state_change: Callback<(GameState, String)>,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub gathering: Html,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.game_state, &other.game_state)
            && Rc::ptr_eq(&self.camp_config, &other.camp_config)
            && self.on_state_change == other.on_state_change
            && self.on_close == other.on_close
            && self.gathering == other.gathering
    }
}

pub fn cooldown(days: u32, total: u32, key: &str) -> Html {
    html! {<div class="cooldown-status"><p>{if days==0{i18n::t("journey.ready")}else{i18n::tr(key,Some(&BTreeMap::from([("days",days.to_string().as_str())])))}}</p><progress max={total.max(1).to_string()} value={(total.saturating_sub(days)).to_string()} aria-label={i18n::t("ux.camp_cooldown")} /></div>}
}

#[function_component(CampPanel)]
pub fn camp_panel(p: &Props) -> Html {
    crate::i18n::use_language();
    if p.game_state.breakdown.is_some() {
        return Html::default();
    }
    let rest = {
        let p = p.clone();
        Callback::from(move |_| {
            let mut state = (*p.game_state).clone();
            let outcome = camp_rest(&mut state, &p.camp_config);
            let message = if outcome.rested {
                crate::app::visual_content::record_rest(&p.game_state, &mut state);
                i18n::encounter_text(
                    &crate::app::visual_content::rest_unit(&p.game_state),
                    "log_0",
                    &i18n::t("ux.rested"),
                )
            } else {
                i18n::t("ux.no_change")
            };
            p.on_state_change.emit((state, message));
        })
    };
    let close = {
        let cb = p.on_close.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let cfg = &p.camp_config;
    let effects = super::choice_effects::describe(
        &crate::game::data::Effects {
            sanity: cfg.rest.sanity,
            hp: cfg.rest.hp,
            supplies: cfg.rest.supplies,
            ..crate::game::data::Effects::default()
        },
        &p.game_state.stats,
    )
    .join(" · ");
    let days = i18n::fmt_number(f64::from(cfg.rest.day));
    let rest_label = i18n::tr(
        "ux.rest",
        Some(&BTreeMap::from([
            ("effects", effects.as_str()),
            ("days", days.as_str()),
        ])),
    );
    let rest_unit = crate::app::visual_content::rest_unit(&p.game_state);
    let rest_offer = i18n::encounter_text(&rest_unit, "desc", "");
    let choice = i18n::encounter_text(&rest_unit, "choice_0", &i18n::t("camp.title"));
    html! { <section class="camp-modal" aria-labelledby="camp-title">
        <h2 id="camp-title" class="sr-only">{i18n::t("camp.title")}</h2>
        <div class="camp-actions">
            <div class="camp-action"><p>{rest_offer}</p><ActionButton onclick={rest} disabled={p.game_state.camp.rest_cooldown > 0 || cfg.rest.day == 0} label={choice} detail={rest_label} />{cooldown(p.game_state.camp.rest_cooldown,cfg.rest.cooldown_days,"play2.rest_ready")}</div>
            {p.gathering.clone()}
        </div>
        <div class="controls"><button class="retro-btn-primary" onclick={close}>{i18n::t("ux.back_road")}</button></div>
    </section> }
}
