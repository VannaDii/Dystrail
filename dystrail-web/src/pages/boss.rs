use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{BossConfig, GameState};
use std::collections::BTreeMap;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct BossPageProps {
    pub state: GameState,
    pub config: BossConfig,
    pub weather: WeatherBadge,
    pub on_begin: Callback<()>,
}

impl PartialEq for BossPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.state.day == other.state.day
            && self.state.region == other.state.region
            && self.state.stats == other.state.stats
            && self.weather == other.weather
            && self.config.rounds == other.config.rounds
            && (self.config.max_chance - other.config.max_chance).abs() < f32::EPSILON
    }
}

fn boss_stats_text(
    cfg: &BossConfig,
    gs: &GameState,
) -> (String, String, Option<String>, Option<String>) {
    let mut chance = f64::from(cfg.base_victory_chance);
    chance += f64::from(gs.stats.credibility) * f64::from(cfg.credibility_weight);
    chance += f64::from(gs.stats.sanity) * f64::from(cfg.sanity_weight);
    chance += f64::from(gs.stats.supplies) * f64::from(cfg.supplies_weight);
    chance += f64::from(gs.stats.allies) * f64::from(cfg.allies_weight);
    chance -= f64::from(gs.stats.pants) * f64::from(cfg.pants_penalty_weight);
    chance = chance.clamp(f64::from(cfg.min_chance), f64::from(cfg.max_chance));
    let chance_pct = format!("{:.1}", chance * 100.0);

    let mut rounds_map: BTreeMap<&str, &str> = BTreeMap::new();
    let rounds_value = cfg.rounds.to_string();
    let passes_value = cfg.passes_required.to_string();
    rounds_map.insert("rounds", rounds_value.as_str());
    rounds_map.insert("passes", passes_value.as_str());
    let rounds_text = crate::i18n::tr("boss.stats.rounds", Some(&rounds_map));

    let mut chance_map: BTreeMap<&str, &str> = BTreeMap::new();
    chance_map.insert("chance", chance_pct.as_str());
    let chance_text = crate::i18n::tr("boss.stats.chance", Some(&chance_map));

    let sanity_text = if cfg.sanity_loss_per_round > 0 {
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        let delta = format!("{:+}", -cfg.sanity_loss_per_round);
        map.insert("sanity", delta.as_str());
        Some(crate::i18n::tr("boss.stats.sanity", Some(&map)))
    } else {
        None
    };

    let pants_text = if cfg.pants_gain_per_round > 0 {
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        let delta = format!("{:+}", cfg.pants_gain_per_round);
        map.insert("pants", delta.as_str());
        Some(crate::i18n::tr("boss.stats.pants", Some(&map)))
    } else {
        None
    };

    (rounds_text, chance_text, sanity_text, pants_text)
}

#[function_component(BossPage)]
pub fn boss_page(props: &BossPageProps) -> Html {
    let gs = props.state.clone();
    let cfg = props.config.clone();
    let persona_id = gs.persona_id.clone();
    let (rounds_text, chance_text, sanity_text, pants_text) = boss_stats_text(&cfg, &gs);

    html! {
        <>
            <crate::components::ui::stats_bar::StatsBar
                stats={gs.stats.clone()}
                day={gs.day}
                region={gs.region}
                exec_order={gs.current_order}
                persona_id={persona_id}
                weather={Some(props.weather.clone())}
            />
            <section class="panel boss-phase boss-panel">
                <h2>{ crate::i18n::t("boss.title") }</h2>
                <div class="encounter-desc">
                    <p>{ crate::i18n::t("boss.phases_hint") }</p>
                    <ul class="boss-stats">
                        <li>{ rounds_text }</li>
                        { sanity_text.map_or_else(
                            Html::default,
                            |text| html! { <li>{ text }</li> },
                        ) }
                        { pants_text.map_or_else(
                            Html::default,
                            |text| html! { <li>{ text }</li> },
                        ) }
                        <li>{ chance_text }</li>
                    </ul>
                    <p class="muted">{ crate::i18n::t("boss.reminder") }</p>
                </div>
                <div class="controls">
                    <button class="retro-btn-primary" onclick={{
                        let on_begin = props.on_begin.clone();
                        Callback::from(move |_| on_begin.emit(()))
                    }}>
                        { crate::i18n::t("boss.begin") }
                    </button>
                </div>
            </section>
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_stats_text_omits_optional_lines_when_zero() {
        crate::i18n::set_lang("en");
        let cfg = BossConfig {
            sanity_loss_per_round: 0,
            pants_gain_per_round: 0,
            ..BossConfig::default()
        };
        let gs = GameState::default();
        let (_rounds, _chance, sanity, pants) = boss_stats_text(&cfg, &gs);
        assert!(sanity.is_none());
        assert!(pants.is_none());
    }
}
