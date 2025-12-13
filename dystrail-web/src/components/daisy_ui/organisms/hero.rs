use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct HeroProps {
    pub title: f::AttrValue,
    #[prop_or_default]
    pub subtitle: Option<f::AttrValue>,
    #[prop_or_default]
    pub actions: Option<f::Html>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Hero)]
pub fn hero(props: &HeroProps) -> f::Html {
    let class = f::class_list(&["hero", "bg-base-200", "rounded-box", "p-6"], &props.class);
    f::html! {
        <section class={class} aria-label="Hero">
            <div class="hero-content flex-col lg:flex-row gap-6">
                <div class="hero-text">
                    <h1 class="text-3xl font-bold">{ props.title.clone() }</h1>
                    { props.subtitle.as_ref().map(|sub| f::html! { <p class="py-2 text-base-content/70">{ sub.clone() }</p> }).unwrap_or_default() }
                    { props.actions.clone().unwrap_or_default() }
                </div>
                <div class="hero-extra w-full">
                    { for props.children.iter() }
                </div>
            </div>
        </section>
    }
}
