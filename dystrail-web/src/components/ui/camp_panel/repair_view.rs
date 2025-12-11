use crate::game::vehicle::Breakdown;
use crate::i18n;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn render_repair(
    focus_idx: u8,
    list_ref: &NodeRef,
    on_action: &Callback<u8>,
    on_keydown: &Callback<KeyboardEvent>,
    status_text: &str,
    breakdown: Option<&Breakdown>,
) -> Html {
    let part_name = breakdown.map_or_else(|| "Unknown".to_string(), |b| i18n::t(b.part.key()));

    let items = vec![
        (1_u8, i18n::t("camp.menu.use_spare")),
        (2, i18n::t("camp.menu.hack_fix")),
        (0, i18n::t("camp.menu.back")),
    ];

    html! {
        <section
            role="dialog"
            aria-modal="true"
            aria-labelledby="repair-title"
            onkeydown={on_keydown.clone()}
            class="ot-menu repair-modal"
            tabindex="0"
        >
            <h2 id="repair-title">{ i18n::t("camp.repair.title") }</h2>
            <p>{ format!("{breakdown_label}: {part_name}", breakdown_label = i18n::t("vehicle.breakdown")) }</p>

            <ul role="menu" aria-label={i18n::t("camp.repair.title")} ref={list_ref.clone()}>
                { for items.into_iter().enumerate().map(|(i, (idx, label))| {
                    let focused = focus_idx == idx;
                    let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);

                    html! {
                        <li
                            role="menuitem"
                            tabindex={ if focused { "0" } else { "-1" } }
                            data-key={idx.to_string()}
                            aria-posinset={posinset.to_string()}
                            aria-setsize="3"
                            onclick={{
                                let on_action = on_action.clone();
                                Callback::from(move |_| on_action.emit(idx))
                            }}
                            class="ot-menuitem"
                        >
                            <span class="num">{ format!("{idx})") }</span>
                            <span class="label">{ label }</span>
                        </li>
                    }
                }) }
            </ul>

            <p aria-live="polite" class="status">{ status_text }</p>
        </section>
    }
}
