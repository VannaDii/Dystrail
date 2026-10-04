//! Optional credits and external support links; payments stay on the recipient's site.
use crate::i18n;
use wasm_bindgen::JsCast;
use yew::prelude::*;

const DSA_URL: &str = "https://act.dsausa.org/donate/donation";
const COFFEE_URL: &str = "https://buymeacoffee.com/vannadi";

#[derive(Clone, PartialEq)]
pub struct CreditsAction(pub Callback<AttrValue>);

#[derive(Properties, PartialEq, Eq)]
pub struct ButtonProps {
    pub id: AttrValue,
    #[prop_or_default]
    pub menu: bool,
}

#[function_component(CreditsButton)]
pub fn credits_button(props: &ButtonProps) -> Html {
    i18n::use_language();
    let action = use_context::<CreditsAction>();
    let target = if props.menu {
        AttrValue::from("game-menu-button")
    } else {
        props.id.clone()
    };
    let onclick = Callback::from(move |_| {
        if let Some(action) = &action {
            action.0.emit(target.clone());
        }
    });
    html! {
        <button id={props.id.clone()} type="button" {onclick}
            data-menu-close={props.menu.then_some("true")}
            aria-haspopup="dialog" aria-controls="credits-support-dialog">
            {i18n::t("credits.title")}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub open: bool,
    pub on_close: Callback<()>,
    pub return_focus_id: AttrValue,
}

#[function_component(CreditsDialog)]
pub fn credits_dialog(props: &DialogProps) -> Html {
    i18n::use_language();
    let node = use_node_ref();
    let dialog = node.clone();
    use_effect_with(
        (props.open, props.return_focus_id.clone()),
        move |(open, target)| {
            if let Some(dialog) = dialog.cast::<web_sys::HtmlDialogElement>() {
                if *open && !dialog.open() {
                    let _ = dialog.show_modal();
                } else if !*open && dialog.open() {
                    dialog.close();
                    crate::a11y::restore_focus(target);
                }
            }
        },
    );
    let close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };
    let cancel = {
        let on_close = props.on_close.clone();
        Callback::from(move |event: Event| {
            event.prevent_default();
            on_close.emit(());
        })
    };
    let backdrop = {
        let on_close = props.on_close.clone();
        let node = node.clone();
        Callback::from(move |event: MouseEvent| {
            if let (Some(target), Some(dialog)) = (
                event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok()),
                node.cast::<web_sys::Element>(),
            ) && target == dialog
            {
                on_close.emit(());
            }
        })
    };
    html! {
        <dialog id="credits-support-dialog" class="credits-dialog" ref={node}
            aria-labelledby="credits-title" oncancel={cancel} onclick={backdrop}
            onkeydown={Callback::from(|event: KeyboardEvent| event.stop_propagation())}>
            <div class="credits-content">
                <header class="credits-header">
                    <h2 id="credits-title">{i18n::t("credits.title")}</h2>
                    <button type="button" autofocus={true} onclick={close}>{i18n::t("dialogs.close")}</button>
                </header>
                <section aria-labelledby="credits-people">
                    <h3 id="credits-people">{i18n::t("credits.people")}</h3>
                    <p>{i18n::t("credits.creator")}</p>
                    <p>{external("https://github.com/VannaDii/Dystrail/graphs/contributors", &i18n::t("credits.contributors"))}</p>
                    <p>{i18n::t("credits.art")}</p>
                </section>
                <section aria-labelledby="credits-licenses">
                    <h3 id="credits-licenses">{i18n::t("credits.licenses")}</h3>
                    <ul>
                        <li>{external("https://github.com/VannaDii/Dystrail", &i18n::t("credits.code"))}</li>
                        {for [
                            ("Atkinson Hyperlegible Next", "atkinson-hyperlegible-next-OFL.txt"),
                            ("Courier Prime", "courier-prime-OFL.txt"),
                            ("Noto Sans Arabic", "noto-sans-arabic-OFL.txt"),
                        ].into_iter().map(|(name, file)| html! {
                            <li>{external(&crate::paths::asset_path(&format!("static/fonts/{file}")), &format!("{name} · SIL OFL 1.1"))}</li>
                        })}
                        <li>{external("https://www.census.gov/geographies/mapping-files/time-series/geo/cartographic-boundary.html", &i18n::t("credits.boundaries"))}</li>
                        <li>{external("https://www.openstreetmap.org/copyright", "© OpenStreetMap contributors · ODbL")}</li>
                    </ul>
                </section>
                <section aria-labelledby="credits-coffee">
                    <h3 id="credits-coffee">{i18n::t("credits.game_support")}</h3>
                    <p>{i18n::t("credits.coffee_description")}</p>
                    <p>{external(COFFEE_URL, &i18n::t("credits.coffee_link"))}</p>
                </section>
                <section aria-labelledby="credits-democracy">
                    <h3 id="credits-democracy">{i18n::t("credits.democracy")}</h3>
                    <p>{i18n::t("credits.dsa_description")}</p>
                    <p>{external(DSA_URL, &i18n::t("credits.dsa_link"))}</p>
                    <p class="muted">{i18n::t("credits.dsa_direct")}</p>
                </section>
                <p class="muted">{i18n::t("credits.external_notice")}</p>
            </div>
        </dialog>
    }
}

fn external(url: &str, label: &str) -> Html {
    html! {<a href={url.to_owned()} target="_blank" rel="noopener noreferrer">{label}<span aria-hidden="true">{" ↗"}</span><span class="sr-only">{format!(" ({})", i18n::t("credits.new_tab"))}</span></a>}
}
