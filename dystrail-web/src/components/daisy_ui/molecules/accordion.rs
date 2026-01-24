use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct AccordionProps {
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

#[f::function_component(Accordion)]
pub fn accordion(props: &AccordionProps) -> f::Html {
    let open_state = f::use_state(|| props.open);
    #[cfg(target_arch = "wasm32")]
    {
        let open_state = open_state.clone();
        let external_open = props.open;
        f::use_effect_with(external_open, move |is_open| {
            open_state.set(*is_open);
            || {}
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = props.open;
    }

    let toggle = {
        let open_state = open_state.clone();
        let on_toggle = props.on_toggle.clone();
        #[cfg(target_arch = "wasm32")]
        {
            f::Callback::from(move |_| {
                let next = !*open_state;
                open_state.set(next);
                on_toggle.emit(next);
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (open_state, on_toggle);
            f::Callback::from(|_| {})
        }
    };
    let class = f::class_list(&["collapse", "collapse-arrow"], &props.class);

    f::html! {
        <details class={class} open={*open_state}>
            <summary class="collapse-title" role="button" aria-expanded={(*open_state).to_string()} onclick={toggle}>
                { props.title.clone() }
            </summary>
            <div class="collapse-content">
                { for props.children.iter() }
            </div>
        </details>
    }
}
