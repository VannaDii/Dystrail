use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct DropdownProps {
    pub label: f::AttrValue,
    #[prop_or_default]
    pub open: Option<bool>,
    #[prop_or_default]
    pub align_end: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_toggle: f::Callback<bool>,
    #[prop_or_default]
    pub menu: f::Children,
}

#[f::function_component(Dropdown)]
pub fn dropdown(props: &DropdownProps) -> f::Html {
    let open_state = f::use_state(|| props.open.unwrap_or(false));
    {
        let open_state = open_state.clone();
        let external = props.open;
        f::use_effect_with(external, move |ext| {
            if let Some(val) = ext {
                open_state.set(*val);
            }
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
    let mut class = f::class_list(&["dropdown"], &props.class);
    if props.align_end {
        class.push("dropdown-end");
    }
    f::html! {
        <div class={class}>
            <button class="btn" aria-haspopup="true" aria-expanded={(*open_state).to_string()} onclick={toggle.clone()}>
                { props.label.clone() }
            </button>
            <div class="dropdown-content menu p-2 shadow bg-base-100 rounded-box mt-2" role="menu" hidden={!*open_state}>
                { for props.menu.iter() }
            </div>
        </div>
    }
}
