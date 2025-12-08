mod menu;
mod share;
#[cfg(test)]
mod tests;

use crate::game::{GameState, ResultConfig, ResultSummary};
use crate::i18n;
use menu::{handle_keyboard, render_menu_item};
use share::{resolved_epilogue_key, resolved_headline_key, summary};
use yew::prelude::*;

/// Properties for the result screen component
#[derive(Properties, Clone)]
pub struct Props {
    pub game_state: GameState,
    pub result_config: ResultConfig,
    pub boss_won: bool,
    pub on_replay_seed: Callback<()>,
    pub on_new_run: Callback<()>,
    pub on_title: Callback<()>,
    pub on_export: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.boss_won == other.boss_won && self.result_config == other.result_config
    }
}

/// Messages for the result screen component
pub enum Msg {
    MenuAction(u8),
    KeyDown(KeyboardEvent),
    AnnouncementChange(String),
}

/// The result screen component state
pub struct ResultScreen {
    current_focus: u8,
    announcement: String,
}

impl Component for ResultScreen {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            current_focus: 1,
            announcement: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MenuAction(action) => {
                Self::handle_menu_action(ctx, action);
                true
            }
            Msg::KeyDown(e) => {
                let on_menu_action = ctx.link().callback(Msg::MenuAction);
                self.current_focus = handle_keyboard(self.current_focus, &e, &on_menu_action);
                true
            }
            Msg::AnnouncementChange(text) => {
                self.announcement = text;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let summary = match summary(props) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to generate result summary: {e}");
                return html! {
                    <main role="main" class="error">
                        <h1>{ "Error generating result" }</h1>
                        <p>{ e }</p>
                    </main>
                };
            }
        };

        let headline_key = resolved_headline_key(&summary, props);
        let epilogue_key = resolved_epilogue_key(&summary, props);
        let headline_text = i18n::t(&headline_key);
        let epilogue_text = i18n::t(&epilogue_key);

        let on_keydown = ctx.link().callback(Msg::KeyDown);
        let on_menu_action = ctx.link().callback(Msg::MenuAction);

        html! {
            <main role="main" aria-labelledby="result-title" onkeydown={on_keydown} tabindex="0" class="result-screen">
                <h1 id="result-title" class="result-headline">{ &headline_text }</h1>

                <section class="result-info" aria-labelledby="result-info-heading">
                    <h2 id="result-info-heading" class="sr-only">{ i18n::t("result.labels.stats") }</h2>
                    <div class="result-metadata">
                        <span class="metadata-item">
                            <strong>{ i18n::t("result.labels.seed") }{": "}</strong>
                            { &summary.seed }
                        </span>
                        <span class="metadata-item">
                            <strong>{ i18n::t("result.labels.persona") }{": "}</strong>
                            { &summary.persona_name }{ " (" }{ &summary.mult_str }{ ")" }
                        </span>
                        <span class="metadata-item">
                            <strong>{ i18n::t("result.labels.mode") }{": "}</strong>
                            { &summary.mode }
                            { if summary.dp_badge {
                                html! { <span class="badge">{ i18n::t("result.badges.mode_deep") }</span> }
                            } else {
                                html! {}
                            }}
                        </span>
                    </div>

                    <div class="score-display">
                        <strong>{ i18n::t("result.labels.score") }{": "}</strong>
                        <span class="score-value">{ crate::i18n::fmt_number(f64::from(summary.score)) }</span>
                    </div>
                </section>

                <section class="stats-section" aria-labelledby="stats-heading">
                    <h2 id="stats-heading">{ i18n::t("result.labels.stats") }</h2>
                    <dl class="stats-grid">
                        <dt>{ i18n::t("result.labels.days") }</dt>
                        <dd>{ summary.days }</dd>
                        <dt>{ i18n::t("result.labels.encounters") }</dt>
                        <dd>{ summary.encounters }</dd>
                        <dt>{ i18n::t("result.labels.receipts") }</dt>
                        <dd>{ summary.receipts }</dd>
                        <dt>{ i18n::t("result.labels.allies") }</dt>
                        <dd>{ summary.allies }</dd>
                        <dt>{ i18n::t("result.labels.supplies") }</dt>
                        <dd>{ summary.supplies }</dd>
                        <dt>{ i18n::t("result.labels.credibility") }</dt>
                        <dd>{ summary.credibility }</dd>
                        <dt>{ i18n::t("result.labels.pants_pct") }</dt>
                        <dd>{ format!("{pants_pct}%", pants_pct = summary.pants_pct) }</dd>
                        <dt>{ i18n::t("result.labels.breakdowns") }</dt>
                        <dd>{ summary.vehicle_breakdowns }</dd>
                        <dt>{ i18n::t("result.labels.miles") }</dt>
                        <dd>{ crate::i18n::fmt_number(f64::from(summary.miles_traveled).round()) }</dd>
                        <dt>{ i18n::t("result.labels.score_threshold") }</dt>
                        <dd>{ crate::i18n::fmt_number(f64::from(summary.score_threshold)) }</dd>
                        <dt>{ i18n::t("result.labels.passed_threshold") }</dt>
                        <dd>{ if summary.passed_threshold { i18n::t("result.badges.success") } else { i18n::t("result.badges.fail") } }</dd>
                        <dt>{ i18n::t("result.labels.malnutrition") }</dt>
                        <dd>{ summary.malnutrition_days }</dd>
                    </dl>
                </section>

                <section class="epilogue-section">
                    <p class="epilogue">{ &epilogue_text }</p>
                </section>

                <nav class="result-menu" role="menu" aria-label={ i18n::t("result.title") }>
                    <ul role="none">
                        { render_menu_item(self.current_focus, 1, &i18n::t("result.menu.copy_share"), &on_menu_action) }
                        { render_menu_item(self.current_focus, 2, &i18n::t("result.menu.copy_seed"), &on_menu_action) }
                        { render_menu_item(self.current_focus, 3, &i18n::t("result.menu.replay_seed"), &on_menu_action) }
                        { render_menu_item(self.current_focus, 4, &i18n::t("result.menu.new_run"), &on_menu_action) }
                        { render_menu_item(self.current_focus, 5, &i18n::t("result.menu.export"), &on_menu_action) }
                        { render_menu_item(self.current_focus, 0, &i18n::t("result.menu.title"), &on_menu_action) }
                    </ul>
                </nav>

                <div aria-live="polite" aria-atomic="true" class="sr-only" id="announcements">
                    { &self.announcement }
                </div>
            </main>
        }
    }
}

impl ResultScreen {
    fn handle_menu_action(ctx: &Context<Self>, action: u8) {
        let props = ctx.props();
        let summary = match summary(props) {
            Ok(s) => s,
            Err(e) => {
                Self::announce(ctx, &format!("Error: {e}"));
                return;
            }
        };

        match action {
            1 => Self::copy_share_text(ctx, &summary),
            2 => Self::copy_seed(ctx, &summary.seed),
            3 => props.on_replay_seed.emit(()),
            4 => props.on_new_run.emit(()),
            5 => props.on_export.emit(()),
            0 => props.on_title.emit(()),
            _ => {}
        }
    }

    fn copy_share_text(ctx: &Context<Self>, summary: &ResultSummary) {
        let headline_key = resolved_headline_key(summary, ctx.props());
        let headline_text = i18n::t(&headline_key);
        let template = i18n::t("result.share.template");
        let share_text = share::interpolate_template(&template, summary, &headline_text);
        Self::copy_to_clipboard(ctx, &share_text);
    }

    fn copy_seed(ctx: &Context<Self>, seed: &str) {
        Self::copy_to_clipboard(ctx, seed);
    }

    fn copy_to_clipboard(ctx: &Context<Self>, text: &str) {
        match share::copy_payload(text) {
            Ok(()) => Self::announce(ctx, &i18n::t("result.announce.copied")),
            Err(_) => Self::announce(ctx, &i18n::t("result.announce.copy_failed")),
        }
    }

    fn announce(ctx: &Context<Self>, message: &str) {
        ctx.link()
            .send_message(Msg::AnnouncementChange(message.to_string()));
    }
}

/// Function component wrapper for the Result Screen
///
/// Provides a functional component interface for the `ResultScreen` struct component.
/// This wrapper handles the component props forwarding and rendering.
#[function_component(ResultScreenWrapper)]
pub fn result_screen_wrapper(props: &Props) -> Html {
    html! {
        <ResultScreen
            game_state={props.game_state.clone()}
            result_config={props.result_config.clone()}
            boss_won={props.boss_won}
            on_replay_seed={props.on_replay_seed.clone()}
            on_new_run={props.on_new_run.clone()}
            on_title={props.on_title.clone()}
            on_export={props.on_export.clone()}
        />
    }
}
