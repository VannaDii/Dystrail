use crate::components::daisy_ui::foundation as f;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ChatPosition {
    #[default]
    Start,
    End,
}

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct ChatProps {
    pub author: f::AttrValue,
    pub message: f::AttrValue,
    #[prop_or_default]
    pub timestamp: Option<f::AttrValue>,
    #[prop_or_default]
    pub position: ChatPosition,
    #[prop_or_default]
    pub avatar_src: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Chat)]
pub fn chat(props: &ChatProps) -> f::Html {
    let mut class = f::class_list(&["chat"], &props.class);
    match props.position {
        ChatPosition::Start => class.push("chat-start"),
        ChatPosition::End => class.push("chat-end"),
    }
    f::html! {
        <div class={class} role="listitem">
            <div class="chat-image avatar">
                { props.avatar_src.as_ref().map(|src| f::html! { <img src={src.clone()} alt={format!("Avatar for {}", props.author)} /> }).unwrap_or_default() }
            </div>
            <div class="chat-header flex items-center gap-2">
                <span class="font-semibold">{ props.author.clone() }</span>
                { props.timestamp.as_ref().map(|time| f::html! { <time datetime={time.clone()}>{ time.clone() }</time> }).unwrap_or_default() }
            </div>
            <div class="chat-bubble">{ props.message.clone() }</div>
        </div>
    }
}
