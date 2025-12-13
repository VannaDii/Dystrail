use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq, Eq)]
pub struct FilterOption {
    pub label: f::AttrValue,
    pub value: f::AttrValue,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct FilterProps {
    pub options: Vec<FilterOption>,
    #[prop_or_default]
    pub selected: Vec<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<Vec<f::AttrValue>>,
}

#[f::function_component(Filter)]
pub fn filter(props: &FilterProps) -> f::Html {
    let state = f::use_state(|| props.selected.clone());
    {
        let state = state.clone();
        let selected = props.selected.clone();
        f::use_effect_with(selected, move |sel| {
            state.set(sel.clone());
            || {}
        });
    }
    let on_change = props.on_change.clone();
    let class = f::class_list(&["filter", "flex", "gap-2", "flex-wrap"], &props.class);
    f::html! {
        <div class={class} role="group" aria-label="Filters">
            { for props.options.iter().map(|opt| {
                let value = opt.value.clone();
                let active = state.contains(&value);
                let toggle = {
                    let state = state.clone();
                    let on_change = on_change.clone();
                    let value = value.clone();
                    f::Callback::from(move |_| {
                        let mut next = (*state).clone();
                        if let Some(idx) = next.iter().position(|v| v == &value) {
                            next.remove(idx);
                        } else {
                            next.push(value.clone());
                        }
                        state.set(next.clone());
                        on_change.emit(next);
                    })
                };
                let mut button_class = f::classes!("btn", "btn-sm", "btn-outline");
                if active {
                    button_class.push("btn-active");
                }
                f::html! {
                    <button class={button_class} aria-pressed={active.to_string()} onclick={toggle}>{ opt.label.clone() }</button>
                }
            }) }
        </div>
    }
}
