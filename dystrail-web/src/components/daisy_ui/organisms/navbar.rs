use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct NavbarProps {
    #[prop_or_default]
    pub start: Option<f::Html>,
    #[prop_or_default]
    pub center: Option<f::Html>,
    #[prop_or_default]
    pub end: Option<f::Html>,
    #[prop_or_default]
    pub brand: Option<f::Html>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Navbar)]
pub fn navbar(props: &NavbarProps) -> f::Html {
    let class = f::class_list(&["navbar", "bg-base-200"], &props.class);
    f::html! {
        <nav class={class} role="navigation" aria-label="Main">
            <div class="navbar-start gap-2">
                { props.brand.clone().unwrap_or_default() }
                { props.start.clone().unwrap_or_default() }
            </div>
            <div class="navbar-center">
                { props.center.clone().unwrap_or_default() }
            </div>
            <div class="navbar-end gap-2">
                { props.end.clone().unwrap_or_default() }
            </div>
        </nav>
    }
}
