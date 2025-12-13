use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct CardProps {
    #[prop_or_default]
    pub title: Option<f::AttrValue>,
    #[prop_or_default]
    pub subtitle: Option<f::AttrValue>,
    #[prop_or_default]
    pub actions: Option<f::Html>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Card)]
pub fn card(props: &CardProps) -> f::Html {
    let class = f::class_list(&["card", "shadow"], &props.class);
    f::html! {
        <article class={class} role="article">
            { props.title.as_ref().map(|title| f::html! {
                <header class="card-title">
                    <h3>{ title.clone() }</h3>
                    { props.subtitle.as_ref().map(|sub| f::html!{ <p class="text-sm text-base-content/60">{ sub.clone() }</p> }).unwrap_or_default() }
                </header>
            }).unwrap_or_default() }
            <div class="card-body">
                { for props.children.iter() }
            </div>
            { props.actions.clone().map(|actions| f::html!{
                <footer class="card-actions">{ actions }</footer>
            }).unwrap_or_default() }
        </article>
    }
}
