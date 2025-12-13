use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub label: f::AttrValue,
    pub href: Option<f::AttrValue>,
    pub active: bool,
    pub disabled: bool,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct MenuProps {
    pub items: Vec<MenuItem>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_select: Option<f::Callback<usize>>,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Menu)]
pub fn menu(props: &MenuProps) -> f::Html {
    let class = f::class_list(&["menu", "p-2", "rounded-box", "bg-base-200"], &props.class);
    if !props.children.is_empty() {
        return f::html! { <ul class={class}>{ for props.children.iter() }</ul> };
    }
    f::html! {
        <ul class={class} role="list">
            { for props.items.iter().enumerate().map(|(idx, item)| {
                let mut li_class = f::classes!();
                if item.active {
                    li_class.push("active");
                }
                if item.disabled {
                    li_class.push("disabled");
                }
                let on_click = {
                    let on_select = props.on_select.clone();
                    f::Callback::from(move |e: f::MouseEvent| {
                        if let Some(cb) = on_select.as_ref() {
                            e.prevent_default();
                            cb.emit(idx);
                        }
                    })
                };
                f::html! {
                    <li class={li_class}>
                        { item.href.as_ref().map_or_else(
                            || f::html! { <button type="button" onclick={on_click.clone()} disabled={item.disabled}>{ item.label.clone() }</button> },
                            |href| f::html! { <a href={href.clone()} aria-current={if item.active { Some::<f::AttrValue>("page".into()) } else { None }} onclick={on_click.clone()}>{ item.label.clone() }</a> },
                        )}
                    </li>
                }
            })}
        </ul>
    }
}
