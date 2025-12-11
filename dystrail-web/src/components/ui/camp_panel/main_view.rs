use crate::i18n;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn render_main(
    focus_idx: u8,
    list_ref: &NodeRef,
    on_action: &Callback<u8>,
    on_keydown: &Callback<KeyboardEvent>,
    status_text: &str,
    can_repair_now: bool,
    can_therapy_now: bool,
) -> Html {
    let items = vec![
        (1_u8, i18n::t("camp.menu.rest"), true),
        (2, i18n::t("camp.menu.repair"), true),
        (3, i18n::t("camp.menu.forage"), true),
        (4, i18n::t("camp.menu.therapy"), can_therapy_now),
        (0, i18n::t("camp.menu.close"), true),
    ];

    html! {
        <section
            role="dialog"
            aria-modal="true"
            aria-labelledby="camp-title"
            aria-describedby="camp-desc"
            onkeydown={on_keydown.clone()}
            class="ot-menu camp-modal"
            tabindex="0"
        >
            <h2 id="camp-title">{ i18n::t("camp.title") }</h2>
            <p id="camp-desc" class="sr-only">{ i18n::t("camp.desc") }</p>

            <ul role="menu" aria-label={i18n::t("camp.title")} ref={list_ref.clone()}>
                { for items.into_iter().enumerate().map(|(i, (idx, label, enabled))| {
                    let focused = focus_idx == idx;
                    let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);
                    let disabled_class = if enabled { "" } else { "disabled" };
                    let disabled_attr = if enabled { "false" } else { "true" };

                    html! {
                        <li
                            role="menuitem"
                            tabindex={ if focused { "0" } else { "-1" } }
                            data-key={idx.to_string()}
                            aria-posinset={posinset.to_string()}
                            aria-setsize="5"
                            aria-disabled={disabled_attr}
                            onclick={{
                                let on_action = on_action.clone();
                                Callback::from(move |_| on_action.emit(idx))
                            }}
                            class={format!("ot-menuitem {disabled_class}")}
                        >
                            <span class="num">{ format!("{idx})") }</span>
                            <span class="label">{ label }</span>
                            { if idx == 2 && !can_repair_now {
                                html!{ <span class="note">{ format!(" ({note})", note = i18n::t("camp.announce.no_breakdown")) }</span> }
                            } else { html!{} } }
                        </li>
                    }
                }) }
            </ul>

            <p aria-live="polite" class="status">{ status_text }</p>
        </section>
    }
}
