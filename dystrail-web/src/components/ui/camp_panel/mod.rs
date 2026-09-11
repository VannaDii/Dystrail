#[cfg(test)]
mod tests;

use crate::game::{CampConfig, GameState, camp_forage, camp_rest};
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
    pub on_resolve_vehicle: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.game_state, &other.game_state)
            && Rc::ptr_eq(&self.camp_config, &other.camp_config)
            && self.on_state_change == other.on_state_change
            && self.on_close == other.on_close
            && self.on_resolve_vehicle == other.on_resolve_vehicle
    }
}

fn action_label(key: &str, values: &[(&str, i64)]) -> String {
    let values: Vec<_> = values
        .iter()
        .map(|(key, value)| (*key, value.to_string()))
        .collect();
    let vars: BTreeMap<_, _> = values
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect();
    i18n::tr(key, Some(&vars))
}

#[function_component(CampPanel)]
pub fn camp_panel(p: &Props) -> Html {
    let action = |rest: bool| {
        let p = p.clone();
        Callback::from(move |_| {
            let mut state = (*p.game_state).clone();
            let outcome = if rest {
                camp_rest(&mut state, &p.camp_config)
            } else {
                camp_forage(&mut state, &p.camp_config)
            };
            p.on_state_change.emit((
                state,
                i18n::t(if outcome.rested {
                    "ux.rested"
                } else if rest {
                    "ux.no_change"
                } else {
                    "ux.foraged"
                }),
            ));
            p.on_close.emit(());
        })
    };
    let close = {
        let cb = p.on_close.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let repair = {
        let cb = p.on_resolve_vehicle.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let cfg = &p.camp_config;
    let rest_label = action_label(
        "ux.rest",
        &[
            (
                "sanity",
                i64::from(
                    (p.game_state.stats.sanity + cfg.rest.sanity).clamp(0, 10)
                        - p.game_state.stats.sanity,
                ),
            ),
            (
                "hp",
                i64::from(
                    (p.game_state.stats.hp + cfg.rest.hp).clamp(0, 10) - p.game_state.stats.hp,
                ),
            ),
            (
                "supplies",
                i64::from(
                    (p.game_state.stats.supplies + cfg.rest.supplies).clamp(0, 20)
                        - p.game_state.stats.supplies,
                ),
            ),
            (
                "pants",
                i64::from(
                    (p.game_state.stats.pants + cfg.rest.pants).clamp(0, 100)
                        - p.game_state.stats.pants,
                ),
            ),
            ("days", i64::from(cfg.rest.day)),
        ],
    );
    let forage_label = action_label(
        "ux.forage",
        &[
            (
                "supplies",
                i64::from(
                    (p.game_state.stats.supplies
                        + crate::game::numbers::round_f64_to_i32(
                            f64::from(cfg.forage.supplies)
                                * f64::from(
                                    *cfg.forage
                                        .region_multipliers
                                        .get(p.game_state.region.asset_key())
                                        .unwrap_or(&1.0),
                                ),
                        ))
                    .clamp(0, 20)
                        - p.game_state.stats.supplies,
                ),
            ),
            ("days", i64::from(cfg.forage.day)),
        ],
    );
    html! { <section class="camp-modal" aria-labelledby="camp-title">
        <h2 id="camp-title" class="sr-only">{i18n::t("camp.title")}</h2>
        if p.game_state.breakdown.is_some() {
            <div class="alert"><p>{i18n::t("ux.repair_help")}</p>
                <button class="retro-btn-primary" onclick={repair}>{i18n::t("ux.resolve_vehicle")}</button>
            </div>
        }
        <p>{i18n::t("journey.camp_forecast")}</p>
        <div class="camp-actions">
            <button onclick={action(true)} disabled={p.game_state.camp.rest_cooldown > 0 || cfg.rest.day == 0}>{rest_label}</button>
            <button onclick={action(false)} disabled={p.game_state.camp.forage_cooldown > 0 || cfg.forage.day == 0}>{forage_label}</button>
        </div>
        <div class="cooldown-status">
            {for [("play2.rest_ready",p.game_state.camp.rest_cooldown,cfg.rest.cooldown_days),("play2.forage_ready",p.game_state.camp.forage_cooldown,cfg.forage.cooldown_days)].into_iter().map(|(key,days,total)|html!{<div><p>{if days==0{i18n::t("journey.ready")}else{i18n::tr(key,Some(&BTreeMap::from([("days",days.to_string().as_str())])))}}</p><progress max={total.to_string()} value={(total-days.min(total)).to_string()} aria-label={i18n::t("ux.camp_cooldown")} /></div>})}
        </div>
        <div class="controls"><button onclick={close}>{i18n::t("ux.back_road")}</button></div>
    </section> }
}
