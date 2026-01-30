use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct AboutPageProps {
    pub on_back: Callback<()>,
}

#[function_component(AboutPage)]
pub fn about_page(props: &AboutPageProps) -> Html {
    let container_ref = use_node_ref();
    let on_back_key = props.on_back.clone();
    let on_back_click = props.on_back.clone();
    let on_keydown = {
        let on_back = on_back_key;
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Escape" {
                    on_back.emit(());
                    e.prevent_default();
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = on_back;
            Callback::from(|_e: web_sys::KeyboardEvent| {})
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        let container_ref = container_ref.clone();
        use_effect_with((), move |()| {
            if let Some(el) = container_ref.cast::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
            || {}
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = &container_ref;
    }

    html! {
        <div
            class="min-h-screen flex items-center justify-center bg-base-300 font-sans shell-screen"
            onkeydown={on_keydown}
            tabindex="0"
            ref={container_ref}
            data-testid="about-screen"
        >
            <div class="card border border-base-content bg-base-200 w-[420px] max-w-full rounded-none shadow-none shell-card">
                <div class="card-body items-center text-center gap-4">
                    <h1 class="text-2xl font-bold">{ crate::i18n::t("about.title") }</h1>
                    <p class="text-sm opacity-80">{ crate::i18n::t("about.subtitle") }</p>
                    <p class="text-xs opacity-70">{ crate::i18n::t("about.body") }</p>
                    <button class="btn btn-ghost w-full justify-start rounded-none text-left normal-case font-sans shell-btn" onclick={Callback::from(move |_| on_back_click.emit(()))} data-testid="about-back">
                        { crate::i18n::t("ui.back") }
                    </button>
                </div>
            </div>
        </div>
    }
}
