use yew::prelude::*;

/// Not-found page to show when routing fails to match a known view.
#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_go_home: Callback<()>,
}

#[function_component(NotFound)]
pub fn not_found(props: &Props) -> Html {
    let go_home = {
        let cb = props.on_go_home.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <section class="panel not-found" aria-live="assertive">
            <h1>{ crate::i18n::t("not_found.title") }</h1>
            <p>{ crate::i18n::t("not_found.message") }</p>
            <button type="button" onclick={go_home}>
                { crate::i18n::t("not_found.back") }
            </button>
        </section>
    }
}
