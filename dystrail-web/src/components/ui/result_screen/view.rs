use super::layout::render_body;
#[cfg(target_arch = "wasm32")]
use super::menu::handle_keyboard;
use super::share;
use crate::game::{GameState, ResultConfig, ResultSummary};
#[cfg(target_arch = "wasm32")]
use crate::i18n;
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
        self.boss_won == other.boss_won
            && self.result_config == other.result_config
            && self.game_state.ending == other.game_state.ending
    }
}

/// Messages for the result screen component
pub enum Msg {
    MenuAction(u8),
    KeyDown(KeyboardEvent),
    AnnouncementChange(String),
}

/// The result screen component state
#[cfg(target_arch = "wasm32")]
pub struct ResultScreen {
    current_focus: u8,
    announcement: String,
}

#[cfg(target_arch = "wasm32")]
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
        let on_keydown = ctx.link().callback(Msg::KeyDown);
        let on_menu_action = ctx.link().callback(Msg::MenuAction);

        render_result_body(
            props,
            share::summary(props),
            self.current_focus,
            self.announcement.clone(),
            on_keydown,
            &on_menu_action,
        )
    }
}

#[cfg(target_arch = "wasm32")]
impl ResultScreen {
    fn handle_menu_action(ctx: &Context<Self>, action: u8) {
        let props = ctx.props();
        let summary = match share::summary(props) {
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
        let headline_key = share::resolved_headline_key(summary, ctx.props());
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

fn render_error_view(error: &str) -> Html {
    html! {
        <main role="main" class="error">
            <h1>{ "Error generating result" }</h1>
            <p>{ error }</p>
        </main>
    }
}

pub(super) fn resolve_summary(
    _props: &Props,
    summary: Result<ResultSummary, String>,
) -> Result<ResultSummary, Html> {
    match summary {
        Ok(resolved) => Ok(resolved),
        Err(error) => {
            log::error!("Failed to generate result summary: {error}");
            Err(render_error_view(&error))
        }
    }
}

pub(super) fn render_result_body(
    props: &Props,
    summary: Result<ResultSummary, String>,
    current_focus: u8,
    announcement: String,
    on_keydown: Callback<KeyboardEvent>,
    on_menu_action: &Callback<u8>,
) -> Html {
    let summary = match resolve_summary(props, summary) {
        Ok(summary) => summary,
        Err(error_view) => return error_view,
    };

    render_body(
        props,
        &summary,
        current_focus,
        announcement,
        on_keydown,
        on_menu_action,
    )
}

#[cfg(not(target_arch = "wasm32"))]
#[function_component(ResultScreen)]
pub fn result_screen(props: &Props) -> Html {
    let on_keydown = Callback::from(|_e: KeyboardEvent| {});
    let on_menu_action = Callback::from(|_action: u8| {});

    render_result_body(
        props,
        share::summary(props),
        1,
        String::new(),
        on_keydown,
        &on_menu_action,
    )
}

/// Function component wrapper for the Result Screen
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
