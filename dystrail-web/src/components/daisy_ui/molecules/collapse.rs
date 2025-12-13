use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct CollapseProps {
    pub title: f::AttrValue,
    #[prop_or_default]
    pub open: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_toggle: f::Callback<bool>,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Collapse)]
pub fn collapse(props: &CollapseProps) -> f::Html {
    let open_state = f::use_state(|| props.open);
    {
        let open_state = open_state.clone();
        let external_open = props.open;
        f::use_effect_with(external_open, move |open| {
            open_state.set(*open);
            || {}
        });
    }
    let toggle = {
        let open_state = open_state.clone();
        let on_toggle = props.on_toggle.clone();
        f::Callback::from(move |_| {
            let next = !*open_state;
            open_state.set(next);
            on_toggle.emit(next);
        })
    };
    let class = f::class_list(&["collapse", "collapse-plus"], &props.class);
    f::html! {
        <details class={class} open={*open_state}>
            <summary class="collapse-title" onclick={toggle} aria-expanded={(*open_state).to_string()}>
                { props.title.clone() }
            </summary>
            <div class="collapse-content">
                { for props.children.iter() }
            </div>
        </details>
    }
}
