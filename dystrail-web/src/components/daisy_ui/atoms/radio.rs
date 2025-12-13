use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct RadioProps {
    pub name: f::AttrValue,
    pub value: f::AttrValue,
    #[prop_or_default]
    pub checked: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub on_select: f::Callback<f::AttrValue>,
}

#[f::function_component(Radio)]
pub fn radio(props: &RadioProps) -> f::Html {
    let class = f::class_list(&["radio"], &props.class);
    let on_change = {
        let value = props.value.clone();
        let cb = props.on_select.clone();
        f::Callback::from(move |_| cb.emit(value.clone()))
    };
    f::html! {
        <label class="label cursor-pointer gap-2">
            <input
                class={class}
                type="radio"
                name={props.name.clone()}
                value={props.value.clone()}
                checked={props.checked}
                disabled={props.disabled}
                onchange={on_change}
            />
            { props.label.as_ref().map(|l| f::html! { <span>{ l.clone() }</span> }).unwrap_or_default() }
        </label>
    }
}
