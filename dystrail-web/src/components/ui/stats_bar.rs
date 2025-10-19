use crate::game::exec_orders::ExecOrder;
use crate::game::state::{Region, Stats};
use crate::i18n;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub stats: Stats,
    pub day: u32,
    pub region: Region,
    #[prop_or_default]
    pub exec_order: Option<ExecOrder>,
}

#[function_component(StatsBar)]
pub fn stats_bar(p: &Props) -> Html {
    let eo = p.exec_order;
    let region_label = match p.region {
        crate::game::state::Region::Heartland => i18n::t("region.heartland"),
        crate::game::state::Region::RustBelt => i18n::t("region.rustbelt"),
        crate::game::state::Region::Beltway => i18n::t("region.beltway"),
    };
    let day_str = p.day.to_string();
    let pct_str = p.stats.pants.to_string();

    let day_region_text = {
        let mut m = std::collections::HashMap::new();
        m.insert("day", day_str.as_str());
        m.insert("region", region_label.as_str());
        i18n::tr("stats.day_region", Some(&m))
    };

    let pants_text = {
        let mut m = std::collections::HashMap::new();
        m.insert("pct", pct_str.as_str());
        i18n::tr("stats.pants", Some(&m))
    };

    html! {
        <section aria-label="Stats" class="panel stats-panel" role="region">
            <div class="stats-row">
                <span class="stat-label">{ i18n::t("stats.location") }</span>
                <span class="stat-value">{ day_region_text }</span>
            </div>
            <div class="stats-row">
                <div class="stats-list" aria-label="Party Stats">
                    <span class="stat-label">{format!("{hp_label}: {hp}", hp_label = i18n::t("stats.hp_short"), hp = p.stats.hp)}</span>
                    <span class="stat-label">{format!("{sanity_label}: {sanity}", sanity_label = i18n::t("stats.sanity_short"), sanity = p.stats.sanity)}</span>
                    <span class="stat-label">{format!("{cred_label}: {cred}", cred_label = i18n::t("stats.cred_short"), cred = p.stats.credibility)}</span>
                    <span class="stat-label">{format!("{sup_label}: {sup}", sup_label = i18n::t("stats.sup_short"), sup = p.stats.supplies)}</span>
                    <span class="stat-label">{format!("{mor_label}: {mor}", mor_label = i18n::t("stats.mor_short"), mor = p.stats.morale)}</span>
                    <span class="stat-label">{format!("{allies_label}: {allies}", allies_label = i18n::t("stats.allies_short"), allies = p.stats.allies)}</span>
                </div>
            </div>
            <div class="stats-row">
                <span class="stat-label" aria-label={i18n::t("stats.pants_label")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={p.stats.pants.to_string()} role="meter">
                    { pants_text }
                </span>
                <div class="bar-wrap">
                    <div class="bar-fill" style={format!("width: {pants}%", pants = p.stats.pants)}></div>
                </div>
            </div>
            { eo.map_or_else(|| html! {}, |order| {
                html! {
                    <div class="order" aria-live="polite">
                        { format!("{prefix} {order_name}", prefix = i18n::t("eo.prefix"), order_name = i18n::t(order.name_key())) }
                    </div>
                }
            }) }
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn stats_bar_renders_core_fields() {
        crate::i18n::set_lang("en");
        let stats = Stats {
            hp: 7,
            sanity: 5,
            credibility: 3,
            supplies: 12,
            morale: 6,
            allies: 2,
            pants: 42,
        };
        let props = Props {
            stats,
            day: 9,
            region: Region::RustBelt,
            exec_order: None,
        };

        let html = block_on(LocalServerRenderer::<StatsBar>::with_props(props).render());
        assert!(
            html.contains("Rust Belt"),
            "region label should appear: {html}"
        );
        assert!(
            html.contains("42%"),
            "pants percentage should render: {html}"
        );
        assert!(
            html.contains("HP"),
            "stat abbreviations should be present: {html}"
        );
    }

    #[test]
    fn stats_bar_announces_exec_order() {
        crate::i18n::set_lang("en");
        let props = Props {
            stats: Stats::default(),
            day: 1,
            region: Region::Heartland,
            exec_order: Some(ExecOrder::TariffTsunami),
        };

        let html = block_on(LocalServerRenderer::<StatsBar>::with_props(props).render());
        assert!(
            html.contains("Tariff Tsunami"),
            "exec order should render announcement block: {html}"
        );
    }
}
