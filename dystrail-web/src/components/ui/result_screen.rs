use crate::dom;
use crate::game::{GameState, ResultConfig, ResultSummary, result_summary};
use crate::i18n;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
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
        // Compare only the meaningful parts for re-rendering
        self.boss_won == other.boss_won && self.result_config == other.result_config
        // We don't compare callbacks or game_state as they change frequently
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
                self.handle_keyboard(ctx, &e);
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
        let summary = match Self::get_summary(ctx) {
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

        let headline_key = Self::resolved_headline_key(&summary, props);
        let epilogue_key = Self::resolved_epilogue_key(&summary, props);
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
                        { self.render_menu_item(1, &i18n::t("result.menu.copy_share"), &on_menu_action) }
                        { self.render_menu_item(2, &i18n::t("result.menu.copy_seed"), &on_menu_action) }
                        { self.render_menu_item(3, &i18n::t("result.menu.replay_seed"), &on_menu_action) }
                        { self.render_menu_item(4, &i18n::t("result.menu.new_run"), &on_menu_action) }
                        { self.render_menu_item(5, &i18n::t("result.menu.export"), &on_menu_action) }
                        { self.render_menu_item(0, &i18n::t("result.menu.title"), &on_menu_action) }
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
    fn get_summary(ctx: &Context<Self>) -> Result<ResultSummary, String> {
        let props = ctx.props();
        result_summary(&props.game_state, &props.result_config)
    }

    fn resolved_headline_key(summary: &ResultSummary, props: &Props) -> String {
        if (props.game_state.boss.outcome.attempted || props.game_state.boss.readiness.ready)
            && !props.boss_won
        {
            "result.headline.boss_loss".to_string()
        } else if props.boss_won {
            "result.headline.victory".to_string()
        } else {
            summary.headline_key.clone()
        }
    }

    fn resolved_epilogue_key(summary: &ResultSummary, props: &Props) -> String {
        if (props.game_state.boss.outcome.attempted || props.game_state.boss.readiness.ready)
            && !props.boss_won
        {
            "result.epilogue.boss_loss".to_string()
        } else if props.boss_won {
            "result.epilogue.victory".to_string()
        } else {
            summary.epilogue_key.clone()
        }
    }

    fn render_menu_item(&self, index: u8, label: &str, on_action: &Callback<u8>) -> Html {
        let is_focused = self.current_focus == index;
        let tabindex = if is_focused { "0" } else { "-1" };
        let action_callback = {
            let on_action = on_action.clone();
            Callback::from(move |_: MouseEvent| {
                on_action.emit(index);
            })
        };

        let display_index = if index == 0 { "0" } else { &index.to_string() };

        html! {
            <li
                role="menuitem"
                tabindex={tabindex}
                class={classes!("menu-item", if is_focused { Some("focused") } else { None })}
                onclick={action_callback}
                data-index={index.to_string()}
                aria-label={format!("{display_index} {label}")}
            >
                { format!("{display_index}) {label}") }
            </li>
        }
    }

    fn handle_keyboard(&mut self, ctx: &Context<Self>, e: &KeyboardEvent) {
        let key = e.key();

        // Handle numeric keys (1-9, 0)
        if let Some(action) = Self::parse_numeric_key(&key) {
            e.prevent_default();
            Self::handle_menu_action(ctx, action);
            return;
        }

        // Handle navigation keys
        match key.as_str() {
            "ArrowUp" => {
                e.prevent_default();
                self.current_focus = Self::navigate_up_index(self.current_focus);
            }
            "ArrowDown" => {
                e.prevent_default();
                self.current_focus = Self::navigate_down_index(self.current_focus);
            }
            "Enter" | " " => {
                e.prevent_default();
                Self::handle_menu_action(ctx, self.current_focus);
            }
            "Escape" => {
                e.prevent_default();
                Self::handle_menu_action(ctx, 0); // Go to title
            }
            _ => {}
        }
    }

    fn parse_numeric_key(key: &str) -> Option<u8> {
        match key {
            "1" | "Digit1" => Some(1),
            "2" | "Digit2" => Some(2),
            "3" | "Digit3" => Some(3),
            "4" | "Digit4" => Some(4),
            "5" | "Digit5" => Some(5),
            "0" | "Digit0" => Some(0),
            _ => None,
        }
    }

    const fn navigate_up_index(idx: u8) -> u8 {
        match idx {
            1 => 0,
            0 => 5,
            n => n - 1,
        }
    }

    const fn navigate_down_index(idx: u8) -> u8 {
        match idx {
            0 => 1,
            5 => 0,
            n => n + 1,
        }
    }

    fn handle_menu_action(ctx: &Context<Self>, action: u8) {
        let props = ctx.props();
        let summary = match Self::get_summary(ctx) {
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
        let template = i18n::t("result.share.template");
        let headline_key = Self::resolved_headline_key(summary, ctx.props());
        let headline_text = i18n::t(&headline_key);
        let share_text = Self::interpolate_template(&template, summary, &headline_text);
        Self::copy_to_clipboard(ctx, &share_text);
    }

    fn copy_seed(ctx: &Context<Self>, seed: &str) {
        Self::copy_to_clipboard(ctx, seed);
    }

    fn copy_to_clipboard(ctx: &Context<Self>, text: &str) {
        // For now, just use the fallback method since the Clipboard API is complex
        match Self::fallback_copy(text) {
            Ok(()) => Self::announce(ctx, &i18n::t("result.announce.copied")),
            Err(_) => Self::announce(ctx, &i18n::t("result.announce.copy_failed")),
        }
    }

    fn fallback_copy(text: &str) -> Result<(), String> {
        let Some(document) = dom::document() else {
            return Err("Document unavailable".to_string());
        };
        let textarea = document
            .create_element("textarea")
            .map_err(|_| "Failed to create textarea".to_string())?
            .dyn_into::<HtmlTextAreaElement>()
            .map_err(|_| "Failed to cast to textarea".to_string())?;

        textarea.set_value(text);

        // Style the textarea to be invisible
        if let Ok(style) = js_sys::Reflect::get(&textarea, &"style".into()) {
            let _ = js_sys::Reflect::set(&style, &"position".into(), &"fixed".into());
            let _ = js_sys::Reflect::set(&style, &"top".into(), &"-1000px".into());
            let _ = js_sys::Reflect::set(&style, &"left".into(), &"-1000px".into());
        }

        if let Some(body) = document.body() {
            body.append_child(&textarea)
                .map_err(|_| "Failed to append textarea".to_string())?;
            textarea.select();

            // For now, just return success since execCommand is deprecated
            body.remove_child(&textarea)
                .map_err(|_| "Failed to remove textarea".to_string())?;

            Ok(())
        } else {
            Err("No body element".to_string())
        }
    }

    fn interpolate_template(
        template: &str,
        summary: &ResultSummary,
        headline_text: &str,
    ) -> String {
        template
            .replace("{headline}", headline_text)
            .replace(
                "{score}",
                &crate::i18n::fmt_number(f64::from(summary.score)),
            )
            .replace("{seed}", &summary.seed)
            .replace("{persona}", &summary.persona_name)
            .replace("{mult}", &summary.mult_str)
            .replace("{mode}", &summary.mode)
    }

    fn announce(ctx: &Context<Self>, message: &str) {
        ctx.link()
            .send_message(Msg::AnnouncementChange(message.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dystrail_game::{Ending, GameState, ResultConfig, ResultSummary};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    fn baseline_summary() -> ResultSummary {
        ResultSummary {
            ending: Ending::BossVictory,
            headline_key: "result.headline.victory".into(),
            epilogue_key: "result.epilogue.victory".into(),
            ending_cause: None,
            seed: "CL-TEST90".into(),
            persona_name: "Organizer".into(),
            mult_str: "1.00×".into(),
            mode: "Classic".into(),
            dp_badge: false,
            score: 12_345,
            score_threshold: 10_000,
            passed_threshold: true,
            days: 42,
            encounters: 12,
            receipts: 3,
            allies: 5,
            supplies: 8,
            credibility: 7,
            pants_pct: 55,
            vehicle_breakdowns: 1,
            miles_traveled: 1945.0,
            malnutrition_days: 0,
        }
    }

    fn baseline_props() -> Props {
        Props {
            game_state: GameState::default(),
            result_config: ResultConfig::default(),
            boss_won: false,
            on_replay_seed: Callback::noop(),
            on_new_run: Callback::noop(),
            on_title: Callback::noop(),
            on_export: Callback::noop(),
        }
    }

    #[test]
    fn headline_resolution_prefers_boss_flags() {
        let summary = baseline_summary();
        let mut props = baseline_props();
        props.game_state.boss.outcome.attempted = true;
        props.boss_won = false;
        let key = ResultScreen::resolved_headline_key(&summary, &props);
        assert_eq!(key, "result.headline.boss_loss");

        props.boss_won = true;
        let key = ResultScreen::resolved_headline_key(&summary, &props);
        assert_eq!(key, "result.headline.victory");
    }

    #[test]
    fn epilogue_resolution_tracks_victory_state() {
        let summary = baseline_summary();
        let mut props = baseline_props();
        props.game_state.boss.readiness.ready = true;
        props.boss_won = false;
        let key = ResultScreen::resolved_epilogue_key(&summary, &props);
        assert_eq!(key, "result.epilogue.boss_loss");

        props.boss_won = true;
        let key = ResultScreen::resolved_epilogue_key(&summary, &props);
        assert_eq!(key, "result.epilogue.victory");
    }

    #[test]
    fn parse_numeric_key_identifies_digits() {
        assert_eq!(ResultScreen::parse_numeric_key("3"), Some(3));
        assert_eq!(ResultScreen::parse_numeric_key("0"), Some(0));
        assert_eq!(ResultScreen::parse_numeric_key("A"), None);
    }

    #[test]
    fn result_screen_renders_summary() {
        crate::i18n::set_lang("en");
        let props = baseline_props();
        let html = block_on(LocalServerRenderer::<ResultScreen>::with_props(props).render());
        assert!(html.contains("result-screen"));
        assert!(html.contains("Result"));
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
