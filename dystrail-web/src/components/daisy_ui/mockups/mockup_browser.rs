use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct MockupBrowserProps {
    #[prop_or_default]
    pub url: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(MockupBrowser)]
pub fn mockup_browser(props: &MockupBrowserProps) -> f::Html {
    let class = f::class_list(&["mockup-browser", "border"], &props.class);
    f::html! {
        <div class={class}>
            <div class="mockup-browser-toolbar p-2">
                <div class="input input-bordered w-full">
                    { props.url.clone().unwrap_or_else(|| "https://example.com".into()) }
                </div>
            </div>
            <div class="bg-base-200 p-4">
                { for props.children.iter() }
            </div>
        </div>
    }
}
