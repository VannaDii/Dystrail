use yew::prelude::*;
#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub sanity: i32,
}
#[function_component(DangerNotice)]
pub fn danger_notice(p: &Props) -> Html {
    let visible = use_state(|| false);
    let previous = use_mut_ref(|| None::<i32>);
    {
        let visible = visible.clone();
        use_effect_with(p.sanity, move |value| {
            if *value <= 2 && previous.borrow().is_none_or(|old| old > 2) {
                visible.set(true);
            }
            if *value > 2 {
                visible.set(false);
            }
            *previous.borrow_mut() = Some(*value);
        });
    }
    let dismiss = {
        let visible = visible.clone();
        Callback::from(move |_| visible.set(false))
    };
    if !*visible {
        return Html::default();
    }
    html! {<aside class="danger-notice" role="alert" aria-live="assertive"><span aria-hidden="true">{"!"}</span><div><strong>{crate::i18n::t("play.danger")}</strong><p>{crate::i18n::t("play.danger_help")}</p><button onclick={dismiss}>{crate::i18n::t("play.dismiss")}</button></div></aside>}
}
