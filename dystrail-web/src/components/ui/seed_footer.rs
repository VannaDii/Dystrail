use crate::game::seed::encode_friendly;
use crate::i18n;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub seed: u64,
    pub is_deep_mode: bool,
    #[prop_or_default]
    pub children: Children,
}

/// Footer strip that shows the current seed/share code and optional navigation controls.
#[function_component(SeedFooter)]
pub fn seed_footer(p: &Props) -> Html {
    let share_code = encode_friendly(p.is_deep_mode, p.seed);
    let share_code_copy = share_code.clone();
    let on_copy = Callback::from(move |_| {
        if let Some(win) = web_sys::window() {
            let nav = win.navigator();
            let clipboard = nav.clipboard();
            let _ = clipboard.write_text(&share_code_copy);
        }
    });

    html! {
        <div class="seed-footer" role="contentinfo" aria-live="polite">
            <div class="seed-nav">
                { for p.children.iter() }
            </div>
            <div class="seed-meta">
                <span class="seed-label">
                    { format!("{} {}", i18n::t("game.seed_label"), p.seed) }
                </span>
                <span class="seed-code" aria-label={i18n::t("share.code")}>{ share_code }</span>
                <button class="retro-btn-secondary" onclick={on_copy}>
                    { i18n::t("result.menu.copy_seed") }
                </button>
            </div>
        </div>
    }
}
