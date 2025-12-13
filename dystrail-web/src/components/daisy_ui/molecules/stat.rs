use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq)]
pub struct StatItem {
    pub title: f::AttrValue,
    pub value: f::AttrValue,
    pub description: Option<f::AttrValue>,
    pub figure: Option<f::Html>,
    pub actions: Option<f::Html>,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct StatProps {
    pub items: Vec<StatItem>,
    #[prop_or_default]
    pub vertical: bool,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Stat)]
pub fn stat(props: &StatProps) -> f::Html {
    let mut class = f::class_list(&["stats", "shadow"], &props.class);
    if props.vertical {
        class.push("stats-vertical");
    } else {
        class.push("stats-horizontal");
    }
    f::html! {
        <div class={class}>
            { for props.items.iter().map(|item| f::html!{
                <div class="stat">
                    { item.figure.clone().map(|f| f::html!{ <div class="stat-figure">{ f }</div> }).unwrap_or_default() }
                    <div class="stat-title">{ item.title.clone() }</div>
                    <div class="stat-value">{ item.value.clone() }</div>
                    { item.description.as_ref().map(|d| f::html!{ <div class="stat-desc">{ d.clone() }</div> }).unwrap_or_default() }
                    { item.actions.clone().map(|a| f::html!{ <div class="stat-actions">{ a }</div> }).unwrap_or_default() }
                </div>
            }) }
        </div>
    }
}
