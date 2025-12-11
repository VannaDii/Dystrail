use super::builder::QuantityOption;
use crate::i18n;
use yew::prelude::*;

pub fn render_quantity_options(
    options: &[QuantityOption],
    focus_idx: u8,
    on_select: &Callback<u8>,
) -> Html {
    html! {
        <ul role="menu" aria-label={i18n::t("store.qty_prompt.title")} >
            { for options.iter().enumerate().map(|(i, option)| {
                let focused = focus_idx == option.idx;
                let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);
                html!{
                    <li role="menuitem"
                        tabindex={if focused { "0" } else { "-1" }}
                        data-key={option.idx.to_string()}
                        aria-posinset={posinset.to_string()}
                        aria-setsize={options.len().to_string()}
                        class="ot-menuitem"
                        onclick={{
                            let on_select = on_select.clone();
                            let idx = option.idx;
                            Callback::from(move |_| on_select.emit(idx))
                        }}>
                        <span class="num">{ format!("{})", option.idx) }</span>
                        <span class="label">
                            { option.label.clone() }
                            { if option.preview.is_empty() {
                                html! {}
                            } else {
                                html! { <span class="preview">{ format!(" {}", option.preview) }</span> }
                            }}
                        </span>
                    </li>
                }
            }) }
        </ul>
    }
}
