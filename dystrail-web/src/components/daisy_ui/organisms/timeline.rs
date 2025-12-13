use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq, Eq)]
pub struct TimelineItem {
    pub title: f::AttrValue,
    pub content: Option<f::AttrValue>,
}

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct TimelineProps {
    pub items: Vec<TimelineItem>,
    #[prop_or_default]
    pub horizontal: bool,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Timeline)]
pub fn timeline(props: &TimelineProps) -> f::Html {
    let mut class = f::class_list(&["timeline"], &props.class);
    if props.horizontal {
        class.push("timeline-horizontal");
    } else {
        class.push("timeline-vertical");
    }
    f::html! {
        <ul class={class}>
            { for props.items.iter().map(|item| f::html! {
                <li>
                    <div class="timeline-middle"></div>
                    <div class="timeline-end timeline-box">
                        <h4 class="font-semibold">{ item.title.clone() }</h4>
                        { item.content.as_ref().map(|c| f::html!{ <p class="text-sm text-base-content/70">{ c.clone() }</p> }).unwrap_or_default() }
                    </div>
                </li>
            }) }
        </ul>
    }
}
