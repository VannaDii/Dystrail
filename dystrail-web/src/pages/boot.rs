use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BootPageProps {
    pub logo_src: AttrValue,
    pub ready: bool,
    pub preload_progress: u8,
    pub on_begin: Callback<()>,
}

#[function_component(BootPage)]
pub fn boot_page(props: &BootPageProps) -> Html {
    let on_click = {
        let on_begin = props.on_begin.clone();
        let ready = props.ready;
        Callback::from(move |_| {
            if ready {
                on_begin.emit(());
            }
        })
    };

    let on_keydown = {
        let on_begin = props.on_begin.clone();
        let ready = props.ready;
        Callback::from(move |e: web_sys::KeyboardEvent| {
            if ready {
                e.prevent_default();
                on_begin.emit(());
            }
        })
    };

    html! {
        <section
            class="panel boot-screen"
            aria-busy={(!props.ready).to_string()}
            aria-live="polite"
            onkeydown={on_keydown}
            onclick={on_click}
            tabindex="0"
        >
            <img src={props.logo_src.clone()} alt="Dystrail" loading="eager" style="width:min(520px,80vw)"/>
            <div class="bar-wrap" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={props.preload_progress.to_string()}>
                <div class="bar-fill" style={format!("width:{}%", props.preload_progress)}/>
            </div>
            <p class={classes!("muted", if props.ready { Some("cta-pulse") } else { None })}>
                { if props.ready { crate::i18n::t("ui.cta_start") } else { crate::i18n::t("ui.loading") } }
            </p>
        </section>
    }
}
