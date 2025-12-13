use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq, Eq)]
pub struct Crumb {
    pub label: f::AttrValue,
    pub href: Option<f::AttrValue>,
}

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct BreadcrumbsProps {
    pub items: Vec<Crumb>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub aria_label: Option<f::AttrValue>,
}

#[f::function_component(Breadcrumbs)]
pub fn breadcrumbs(props: &BreadcrumbsProps) -> f::Html {
    let class = f::class_list(&["breadcrumbs"], &props.class);
    let aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| "Breadcrumbs".into());
    f::html! {
        <nav aria-label={aria_label} class={class}>
            <ol>
                { for props.items.iter().enumerate().map(|(idx, item)| {
                    let is_last = idx + 1 == props.items.len();
                    f::html! {
                        <li>
                            { item.href.as_ref().map_or_else(
                                || f::html! { <span aria-current={if is_last { Some::<f::AttrValue>("page".into()) } else { None }}>{ item.label.clone() }</span> },
                                |href| f::html! { <a href={href.clone()} aria-current={if is_last { Some::<f::AttrValue>("page".into()) } else { None }}>{ item.label.clone() }</a> },
                            )}
                        </li>
                    }
                }) }
            </ol>
        </nav>
    }
}
