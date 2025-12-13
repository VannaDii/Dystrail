use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct KbdProps {
    pub keys: Vec<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Kbd)]
pub fn kbd(props: &KbdProps) -> f::Html {
    let class = f::class_list(&["kbd", "kbd-sm"], &props.class);
    f::html! {
        <kbd class={class} aria-label={props.keys.join(" + ")}>
            { props.keys.iter().enumerate().map(|(idx, key)| {
                if idx + 1 == props.keys.len() {
                    f::html! { <span class="px-1">{ key.clone() }</span> }
                } else {
                    f::html! { <>
                        <span class="px-1">{ key.clone() }</span>
                        <span class="text-base-content/60">{ " +" }</span>
                    </> }
                }
            }).collect::<f::Html>() }
        </kbd>
    }
}
