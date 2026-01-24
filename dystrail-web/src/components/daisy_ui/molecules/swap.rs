use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct SwapProps {
    pub on: f::Html,
    pub off: f::Html,
    #[prop_or_default]
    pub indeterminate: Option<f::Html>,
    #[prop_or_default]
    pub active: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_toggle: f::Callback<bool>,
}

#[f::function_component(Swap)]
pub fn swap(props: &SwapProps) -> f::Html {
    let active = f::use_state(|| props.active);
    #[cfg(target_arch = "wasm32")]
    {
        let active = active.clone();
        let external = props.active;
        f::use_effect_with(external, move |ext| {
            active.set(*ext);
            || {}
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = props.active;
    }
    let toggle = {
        let active = active.clone();
        let on_toggle = props.on_toggle.clone();
        #[cfg(target_arch = "wasm32")]
        {
            f::Callback::from(move |_| {
                let next = !*active;
                active.set(next);
                on_toggle.emit(next);
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (active, on_toggle);
            f::Callback::from(|_| {})
        }
    };
    let class = f::class_list(&["swap"], &props.class);
    f::html! {
        <div class={class} onclick={toggle} aria-pressed={(*active).to_string()} role="switch">
            <div class="swap-on" aria-hidden={(!*active).to_string()}>{ props.on.clone() }</div>
            <div class="swap-off" aria-hidden={(*active).to_string()}>{ props.off.clone() }</div>
            { props.indeterminate.clone().map(|node| f::html!{ <div class="swap-indeterminate">{ node }</div> }).unwrap_or_default() }
        </div>
    }
}
