use crate::components::ui::journey_scene::SceneStage;
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
    #[prop_or_default]
    pub on_camp: Callback<()>,
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

fn boss_rounds_text(cfg: &BossConfig) -> String {
    let rounds = cfg.rounds.to_string();
    let mut map = BTreeMap::new();
    map.insert("rounds", rounds.as_str());
    crate::i18n::tr("ux.boss_rounds", Some(&map))
}

#[function_component(BossPage)]
pub fn boss_page(props: &BossPageProps) -> Html {
    let gs = props.state.clone();
    let cfg = props.config.clone();
    let rounds_text = boss_rounds_text(&cfg);

    html! {
        <>
            <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={crate::i18n::t("boss.title")} stage={Some(SceneStage::Boss)} />
            <section class="panel boss-phase boss-panel">

                <div class="encounter-desc">
                    <p>{ crate::i18n::t("journey.mission") }</p>
                    <p class="vote-outlook">{crate::game::boss::vote_preview(&gs,&cfg).map_or_else(||crate::i18n::t("journey.vote_exhausted"),|chance|crate::i18n::tr("journey.vote_chance",Some(&BTreeMap::from([("chance",format!("{:.0}",chance*100.0).as_str())]))))}</p>
                    <p>{crate::i18n::t("journey.vote_basis")}</p>
                    <dl class="vote-strengths">{for [("play.credibility",gs.stats.credibility*15),("ux.receipt",i32::try_from(gs.receipts.len()).unwrap_or(0)*8),("play.allies",gs.stats.allies*5),("ux.health",gs.stats.hp*50),("play.morale",gs.stats.morale*25),("ux.supplies",gs.stats.supplies*10)].into_iter().map(|(key,value)|html!{<div><dt>{crate::i18n::t(key)}</dt><dd>{format!("+{value}")}</dd></div>})}</dl>
                    <ul class="boss-stats">
                        <li>{ rounds_text }</li>
                        if cfg.sanity_loss_per_round > 0 {
                            <li>{format!("{} −{}", crate::i18n::t("ux.boss_sanity"), cfg.sanity_loss_per_round)}</li>
                        }
                        if cfg.pants_gain_per_round > 0 {
                            <li>{format!("{} +{}", crate::i18n::t("ux.boss_pants"), cfg.pants_gain_per_round)}</li>
                        }
                    </ul>
                    <p class="muted">{ crate::i18n::t("ux.boss_resolution") }</p>
                </div>
                <div class="controls"><button onclick={{let on=props.on_camp.clone();Callback::from(move |_|on.emit(()))}}>{crate::i18n::t("journey.review_before_vote")}</button>
                    <button class="retro-btn-primary" onclick={{
                        let on_begin = props.on_begin.clone();
                        Callback::from(move |_| on_begin.emit(()))
                    }}>
                        { crate::i18n::t("ux.boss_begin") }
                    </button>
                </div>
            </section>
        </>
    }
}
