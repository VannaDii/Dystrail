use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct AvatarProps {
    #[prop_or_default]
    pub src: Option<f::AttrValue>,
    #[prop_or_default]
    pub alt: Option<f::AttrValue>,
    #[prop_or_default]
    pub initials: Option<f::AttrValue>,
    #[prop_or_default]
    pub size: Option<f::DaisySize>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Avatar)]
pub fn avatar(props: &AvatarProps) -> f::Html {
    let mut classes = f::class_list(&["avatar"], &props.class);
    if let Some(size) = props.size {
        classes.push(size.class("avatar"));
    }
    let alt = f::attr_value(&props.alt).unwrap_or_else(|| "avatar".into());
    f::html! {
        <div class={classes}>
            <div class="avatar-wrapper">
                { if let Some(src) = props.src.as_ref() {
                    f::html! { <img src={src.clone()} alt={alt} /> }
                } else {
                    let text = props.initials.clone().unwrap_or_else(|| "?".into());
                    f::html! { <span role="img" aria-label={alt}>{ text }</span> }
                }}
            </div>
        </div>
    }
}
