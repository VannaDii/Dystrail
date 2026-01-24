use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub id: f::AttrValue,
    pub label: f::AttrValue,
    pub disabled: bool,
    pub content: Option<f::Html>,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct TabProps {
    pub tabs: Vec<TabItem>,
    #[prop_or_default]
    pub active_id: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<f::AttrValue>,
}

#[f::function_component(Tab)]
pub fn tab(props: &TabProps) -> f::Html {
    let fallback = props
        .tabs
        .first()
        .map(|tab| tab.id.clone())
        .unwrap_or_default();
    let active = f::use_state(|| props.active_id.clone().unwrap_or(fallback));
    #[cfg(target_arch = "wasm32")]
    {
        let active = active.clone();
        let external = props.active_id.clone();
        f::use_effect_with(external, move |id| {
            if let Some(id) = id {
                active.set(id.clone());
            }
            || {}
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = props.active_id.clone();
    }
    let class = f::class_list(&["tabs", "tabs-box"], &props.class);
    let on_change = props.on_change.clone();
    let content = props
        .tabs
        .iter()
        .find(|tab| tab.id == *active)
        .and_then(|tab| tab.content.clone())
        .unwrap_or_default();
    f::html! {
        <div>
            <div class={class} role="tablist">
                { for props.tabs.iter().map(|tab| {
                    let disabled = tab.disabled;
                    let tab_id = tab.id.clone();
                    let tab_label = tab.label.clone();
                    let aria_id = tab_id.clone();
                    let mut tab_class = f::classes!("tab");
                    if tab_id == *active {
                        tab_class.push("tab-active");
                    }
                    if disabled {
                        tab_class.push("tab-disabled");
                    }
                    let change = {
                        let active = active.clone();
                        let on_change = on_change.clone();
                        let change_id = tab_id.clone();
                        f::Callback::from(move |_| {
                            if !disabled {
                                active.set(change_id.clone());
                                on_change.emit(change_id.clone());
                            }
                        })
                    };
                    f::html! {
                        <button class={tab_class} role="tab" aria-selected={(aria_id == *active).to_string()} onclick={change}>{ tab_label }</button>
                    }
                })}
            </div>
            <div class="tab-content mt-4">
                { content }
            </div>
        </div>
    }
}
