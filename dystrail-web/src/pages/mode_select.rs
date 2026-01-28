use crate::game::state::GameMode;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct ModeSelectPageProps {
    pub on_continue: Callback<GameMode>,
    pub on_back: Callback<()>,
}

fn emit_selected_mode(selected: Option<GameMode>, cb: &Callback<GameMode>) {
    if let Some(mode) = selected {
        cb.emit(mode);
    }
}

#[function_component(ModeSelectPage)]
pub fn mode_select_page(props: &ModeSelectPageProps) -> Html {
    let selected = use_state(|| None::<GameMode>);

    let on_select_classic = {
        let selected = selected.clone();
        Callback::from(move |_| selected.set(Some(GameMode::Classic)))
    };
    let on_select_deep = {
        let selected = selected.clone();
        Callback::from(move |_| selected.set(Some(GameMode::Deep)))
    };

    let on_continue = {
        let selected = selected.clone();
        let cb = props.on_continue.clone();
        Callback::from(move |_| emit_selected_mode(*selected, &cb))
    };

    let on_back = props.on_back.clone();

    html! {
        <section class="panel retro-menu" aria-labelledby="mode-title" data-testid="mode-select">
            <h2 id="mode-title">{ crate::i18n::t("mode.title") }</h2>
            <p class="muted">{ crate::i18n::t("mode.subtitle") }</p>
            <div class="mode-options" role="radiogroup" aria-label={crate::i18n::t("mode.title")}>
                <button
                    type="button"
                    role="radio"
                    aria-checked={matches!(*selected, Some(GameMode::Classic)).to_string()}
                    class={classes!("retro-btn-secondary", matches!(*selected, Some(GameMode::Classic)).then_some("selected"))}
                    onclick={on_select_classic}
                    data-testid="mode-classic"
                >
                    { crate::i18n::t("mode.classic") }
                </button>
                <button
                    type="button"
                    role="radio"
                    aria-checked={matches!(*selected, Some(GameMode::Deep)).to_string()}
                    class={classes!("retro-btn-secondary", matches!(*selected, Some(GameMode::Deep)).then_some("selected"))}
                    onclick={on_select_deep}
                    data-testid="mode-deep"
                >
                    { crate::i18n::t("mode.deep") }
                </button>
            </div>
            <div class="controls">
                <button class="retro-btn-secondary" onclick={Callback::from(move |_| on_back.emit(()))} data-testid="mode-back">
                    { crate::i18n::t("ui.back") }
                </button>
                <button
                    class="retro-btn-primary"
                    onclick={on_continue}
                    disabled={selected.is_none()}
                    data-testid="mode-continue"
                >
                    { crate::i18n::t("ui.continue") }
                </button>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn emit_selected_mode_emits_when_chosen() {
        let seen = Rc::new(Cell::new(None::<GameMode>));
        let seen_ref = seen.clone();
        let callback = Callback::from(move |mode| seen_ref.set(Some(mode)));
        emit_selected_mode(Some(GameMode::Deep), &callback);
        assert_eq!(seen.get(), Some(GameMode::Deep));
    }
}
