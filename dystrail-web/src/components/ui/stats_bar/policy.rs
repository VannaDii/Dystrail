//! Executive orders expose the same actual costs in the HUD and structured help.
use crate::{
    components::ui::stat_card::StatCard,
    game::{GameState, exec_orders::ExecOrder},
    i18n,
};
use std::collections::BTreeMap;
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Readout {
    pub impact: String,
    pub duration: String,
    pub costs: Vec<StatCard>,
    pub rates: Vec<StatCard>,
    pub protection: Option<StatCard>,
}

#[must_use]
pub fn readout(gs: &GameState, order: ExecOrder) -> Readout {
    let effect = order.daily_effect(gs.stats.morale, gs.inventory.has_tag("legal_fund"));
    let costs: Vec<_> = [
        ("ux.supplies", effect.supplies),
        ("ux.sanity", effect.sanity),
        ("play.morale", effect.morale),
    ]
    .into_iter()
    .filter(|(_, value)| *value != 0)
    .map(|(key, value)| StatCard {
        key: key.into(),
        value: format!("{value:+}"),
        subtitle: i18n::t("eo.panel.per_day"),
        tone: if value < 0 { "harmful" } else { "helpful" },
    })
    .collect();
    let rates: Vec<_> = [
        ("play.distance", effect.travel_multiplier - 1.0),
        ("eo.panel.breakdown", effect.breakdown_bonus),
    ]
    .into_iter()
    .filter(|(_, value)| value.abs() > f32::EPSILON)
    .map(|(key, value)| StatCard {
        key: key.into(),
        value: format!("{:+.0}%", value * 100.0),
        subtitle: i18n::t("eo.panel.while_active"),
        tone: "harmful",
    })
    .collect();
    let count = gs.exec_order_days_remaining.to_string();
    let days = i18n::fmt_number(f64::from(gs.exec_order_days_remaining));
    Readout {
        impact: compact(&costs, &rates),
        duration: i18n::tr(
            "eo.hud.remaining",
            Some(&BTreeMap::from([
                ("count", count.as_str()),
                ("days", days.as_str()),
            ])),
        ),
        costs,
        rates,
        protection: protection(gs, order),
    }
}

fn compact(costs: &[StatCard], rates: &[StatCard]) -> String {
    let mut effects = costs
        .iter()
        .map(|card| {
            i18n::tr(
                "eo.hud.daily",
                Some(&BTreeMap::from([
                    ("stat", i18n::t(&card.key).as_str()),
                    ("value", card.value.as_str()),
                ])),
            )
        })
        .collect::<Vec<_>>();
    effects.extend(
        rates
            .iter()
            .map(|card| format!("{} {}", i18n::t(&card.key), card.value.replace('-', "−"))),
    );
    if effects.is_empty() {
        i18n::t("eo.panel.protected")
    } else {
        effects.join(" · ")
    }
}

fn protection(gs: &GameState, order: ExecOrder) -> Option<StatCard> {
    match order {
        ExecOrder::BookPanic => Some(StatCard {
            key: "play.morale".into(),
            value: i18n::fmt_number(f64::from(gs.stats.morale)),
            subtitle: i18n::t("eo.panel.morale_protection"),
            tone: if gs.stats.morale >= 7 {
                "helpful"
            } else {
                "harmful"
            },
        }),
        ExecOrder::TariffTsunami => {
            let protected = gs.inventory.has_tag("legal_fund");
            Some(StatCard {
                key: "store.items.legal_fund.name".into(),
                value: i18n::t(if protected {
                    "weather.panel.carried"
                } else {
                    "weather.panel.missing"
                }),
                subtitle: i18n::t("eo.panel.fund_protection"),
                tone: if protected { "helpful" } else { "harmful" },
            })
        }
        _ => None,
    }
}

pub fn details(readout: &Readout) -> Html {
    html! {<div class="impact-details policy-details">
        if !readout.costs.is_empty() || readout.rates.is_empty() {
            <section><h3>{i18n::t("eo.panel.daily_cost")}</h3>
                if readout.costs.is_empty() {<p class="impact-protected">{i18n::t("eo.panel.protected")}</p>}
                else {<ul class="resource-changes">{for readout.costs.iter().map(StatCard::render)}</ul>}
            </section>
        }
        if !readout.rates.is_empty() {
            <section><h3>{i18n::t("eo.panel.while_active")}</h3><ul class="resource-changes">{for readout.rates.iter().map(StatCard::render)}</ul></section>
        }
        if let Some(protection) = &readout.protection {
            <section><h3>{i18n::t("weather.panel.protection")}</h3><ul class="resource-changes">{protection.render()}</ul></section>
        }
        <dl class="effect-schedule">
            <div><dt>{i18n::t("eo.panel.remaining")}</dt><dd>{&readout.duration}</dd></div>
            if !readout.costs.is_empty() || readout.protection.is_some() {
                <div><dt>{i18n::t("eo.panel.timing")}</dt><dd>{i18n::t("eo.panel.daily_timing")}</dd></div>
            }
        </dl>
    </div>}
}

#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub order: ExecOrder,
    pub readout: Option<Readout>,
}

#[function_component(PolicyIndicator)]
pub fn policy_indicator(p: &Props) -> Html {
    crate::i18n::use_language();
    let notifying = use_context::<crate::app::weather_status::PolicyNotification>()
        .unwrap_or_default()
        .0;
    let name = i18n::t(p.order.name_key());
    html! {<div key={p.order.key()} class={classes!("weather-indicator", "policy-indicator", notifying.then_some("weather-changed"))} data-policy={p.order.key()} role="status" aria-live="polite" aria-atomic="true">
        if notifying {<span class="sr-only">{i18n::t("journey.conditions_changed")}{": "}</span>}
        <span class={classes!("weather-name",super::helpers::exec_sprite_class(p.order))}><span class="weather-icon" aria-hidden="true">{"§"}</span><strong>{&name}</strong></span>
        if let Some(r)=&p.readout {
            <span class="sr-only">{&r.impact}{", "}{&r.duration}</span>
            <super::super::context_help::ContextHelp informational={true} icon={"i".to_owned()} title={name}>{details(r)}</super::super::context_help::ContextHelp>
        }
    </div>}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn book_panic_explains_actual_cost_protection_and_singular_duration() {
        i18n::set_lang("en");
        let mut gs = GameState::default();
        gs.stats.morale = 6;
        gs.exec_order_days_remaining = 1;
        let result = readout(&gs, ExecOrder::BookPanic);
        assert_eq!(result.costs[0].value, "-1");
        assert_eq!(result.protection.as_ref().unwrap().value, "6");
        assert_eq!(result.duration, "1 day left");
        gs.stats.morale = 7;
        let result = readout(&gs, ExecOrder::BookPanic);
        assert!(result.costs.is_empty());
        assert_eq!(result.impact, "Protected");
        assert_eq!(result.protection.as_ref().unwrap().tone, "helpful");
    }

    #[test]
    fn each_order_uses_engine_effects_without_applying_them() {
        i18n::set_lang("en");
        let mut gs = GameState {
            exec_order_days_remaining: 3,
            ..GameState::default()
        };
        let before = gs.clone();
        for &order in ExecOrder::ALL {
            let result = readout(&gs, order);
            assert_eq!(result.duration, "3 days left");
            assert!(
                !result.costs.is_empty() || !result.rates.is_empty() || result.protection.is_some()
            );
        }
        assert_eq!(gs.stats, before.stats);
        assert_eq!(gs.continuity.clock_minutes, before.continuity.clock_minutes);
        gs.inventory.tags.insert("legal_fund".into());
        assert!(readout(&gs, ExecOrder::TariffTsunami).costs.is_empty());
        assert_eq!(readout(&gs, ExecOrder::WarDeptReorg).rates[0].value, "+10%");
    }
}
