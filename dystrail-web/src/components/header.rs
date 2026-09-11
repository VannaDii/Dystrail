use crate::i18n::t;
use yew::prelude::*;
#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub on_abandon: Callback<()>,
    #[prop_or_default]
    pub can_abandon: bool,
    pub on_open_save: Callback<()>,
    pub on_save: Callback<()>,
    pub status: String,
    pub on_lang_change: Callback<String>,
    pub current_lang: String,
    pub high_contrast: bool,
    pub on_toggle_hc: Callback<bool>,
}
#[function_component(Header)]
pub fn header(p: &Props) -> Html {
    let contrast = {
        let cb = p.on_toggle_hc.clone();
        let current = p.high_contrast;
        Callback::from(move |_| cb.emit(!current))
    };
    let save = {
        let cb = p.on_save.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let load = {
        let cb = p.on_open_save.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {<header class="game-header">
        <a href="#main" class="skip-link">{t("ui.skip_to_content")}</a>
        <span class="wordmark">{"DYSTOPIAN TRAIL"}<span class="wordmark-line"></span></span>
        <super::game_menu::GameMenu><nav class="header-controls" aria-label={t("play2.game_controls")}>
            <button data-menu-close="true" onclick={save}>{t("play2.save")}</button><button id="save-open-btn" data-menu-close="true" onclick={load}>{t("play2.load")}</button>
            <button class="contrast-switch" role="switch" aria-checked={p.high_contrast.to_string()} onclick={contrast}><span class="switch-track" aria-hidden="true"><span></span></span>{t("play2.high_contrast")}</button>
            <super::language_picker::LanguagePicker on_lang_change={p.on_lang_change.clone()} current_lang={p.current_lang.clone()} />
        if p.can_abandon {<button class="abandon-menu-item" data-menu-close="true" onclick={{let cb=p.on_abandon.clone();Callback::from(move |_|cb.emit(()))}}>{t("journey.abandon")}</button>}
        <super::offline_status::OfflineStatus /></nav></super::game_menu::GameMenu>
        if !p.status.is_empty(){<span class="save-feedback" role="status">{&p.status}</span>}
    </header>}
}
