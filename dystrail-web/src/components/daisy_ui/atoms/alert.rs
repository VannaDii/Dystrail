use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct AlertProps {
    #[prop_or_default]
    pub title: Option<f::AttrValue>,
    #[prop_or_default]
    pub message: Option<f::AttrValue>,
    #[prop_or_default]
    pub variant: Option<f::DaisyColor>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Alert)]
pub fn alert(props: &AlertProps) -> f::Html {
    let mut classes = f::class_list(&["alert", "alert-horizontal"], &props.class);
    if let Some(variant) = props.variant {
        classes.push(variant.class("alert"));
    }
    f::html! {
        <div class={classes} role="status">
            <div class="alert-content">
                { props.title.as_ref().map(|title| f::html! { <strong>{ title.clone() }</strong> }).unwrap_or_default() }
                { props.message.as_ref().map(|msg| f::html! { <p>{ msg.clone() }</p> }).unwrap_or_default() }
                { for props.children.iter() }
            </div>
        </div>
    }
}
