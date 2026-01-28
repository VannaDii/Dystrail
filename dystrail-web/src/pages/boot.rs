use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BootPageProps {
    pub logo_src: AttrValue,
    pub ready: bool,
    pub preload_progress: u8,
    pub on_begin: Callback<()>,
}

fn boot_begin_action(on_begin: Callback<()>, ready: bool) -> Callback<()> {
    Callback::from(move |()| {
        if ready {
            on_begin.emit(());
        }
    })
}

fn boot_keydown_action(on_begin: Callback<()>, ready: bool) -> Callback<()> {
    boot_begin_action(on_begin, ready)
}

#[function_component(BootPage)]
pub fn boot_page(props: &BootPageProps) -> Html {
    let container_ref = use_node_ref();

    let on_click =
        boot_begin_action(props.on_begin.clone(), props.ready).reform(|_e: MouseEvent| ());
    let on_keydown = {
        let on_keydown_action = boot_keydown_action(props.on_begin.clone(), props.ready);
        let ready = props.ready;
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: web_sys::KeyboardEvent| {
                if ready {
                    e.prevent_default();
                }
                on_keydown_action.emit(());
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (on_keydown_action, ready);
            Callback::from(|_e: web_sys::KeyboardEvent| {})
        }
    };

    {
        let container_ref = container_ref.clone();
        let ready = props.ready;
        #[cfg(target_arch = "wasm32")]
        {
            use_effect_with(ready, move |_| {
                if let Some(el) = container_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
                || {}
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (container_ref, ready);
        }
    }

    let status_text = if props.ready {
        crate::i18n::t("boot.ready")
    } else {
        crate::i18n::t("boot.loading")
    };

    html! {
        <div
            class="min-h-screen flex items-center justify-center bg-base-300 font-sans shell-screen"
            aria-busy={(!props.ready).to_string()}
            onkeydown={on_keydown}
            onclick={on_click}
            tabindex="0"
            ref={container_ref}
            data-testid="boot-screen"
        >
            <div class="card border border-base-content bg-base-200 w-[420px] max-w-full rounded-none shadow-none shell-card">
                <div class="card-body items-center text-center gap-6">
                    <div class="space-y-1">
                        <h1 class="text-3xl font-bold">{ crate::i18n::t("app.title") }</h1>
                        <p class="text-sm opacity-80">{ crate::i18n::t("boot.subtitle") }</p>
                    </div>

                    <div class="w-full space-y-2">
                        <progress
                            class="progress progress-primary h-3 w-full"
                            value={props.preload_progress.to_string()}
                            max="100"
                            role="progressbar"
                            aria-valuemin="0"
                            aria-valuemax="100"
                            aria-valuenow={props.preload_progress.to_string()}
                        />
                        <p class="text-xs opacity-70" role="status" aria-live="polite">
                            { crate::i18n::t("boot.loading_label") }
                        </p>
                    </div>

                    if props.ready {
                        <div class="text-sm animate-pulse-slow">
                            <kbd class="kbd kbd-sm">{ crate::i18n::t("boot.press_any_key") }</kbd>
                        </div>
                    }

                    <div class="text-[10px] opacity-50" aria-live="polite">
                        { status_text }
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn boot_begin_action_emits_when_ready() {
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let on_begin = Callback::from(move |()| called_ref.set(true));
        let on_click = boot_begin_action(on_begin, true);
        on_click.emit(());
        assert!(called.get());
    }

    #[test]
    fn boot_keydown_action_emits_when_ready() {
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let on_begin = Callback::from(move |()| called_ref.set(true));
        let on_keydown = boot_keydown_action(on_begin, true);
        on_keydown.emit(());
        assert!(called.get());
    }
}
