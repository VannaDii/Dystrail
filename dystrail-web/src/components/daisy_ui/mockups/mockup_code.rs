use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct MockupCodeProps {
    pub code: f::AttrValue,
    #[prop_or_default]
    pub language: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(MockupCode)]
pub fn mockup_code(props: &MockupCodeProps) -> f::Html {
    let class = f::class_list(&["mockup-code"], &props.class);
    f::html! {
        <pre class={class} role="presentation">
            { props.language.as_ref().map(|lang| f::html!{ <code class={format!("language-{lang}")}></code> }).unwrap_or_default() }
            { props.code.split('\n').map(|line| f::html!{ <code>{ line }</code> }).collect::<f::Html>() }
        </pre>
    }
}
