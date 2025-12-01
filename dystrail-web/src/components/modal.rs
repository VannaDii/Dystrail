use crate::a11y::{restore_focus, trap_focus_in};
use std::sync::atomic::{AtomicUsize, Ordering};
use yew::prelude::*;

static MODAL_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub open: bool,
    pub title: AttrValue,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub description: Option<AttrValue>,
    #[prop_or_default]
    pub return_focus_id: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(props: &Props) -> Html {
    if !props.open {
        return Html::default();
    }

    let modal_id = use_state(|| MODAL_IDS.fetch_add(1, Ordering::Relaxed));
    let container_id = format!("modal-{}", *modal_id);
    let title_id = format!("modal-title-{}", *modal_id);
    let desc_id: Option<String> = props
        .description
        .as_ref()
        .map(|_| format!("modal-desc-{}", *modal_id));

    let container_ref = use_node_ref();
    let prev_open = use_mut_ref(|| props.open);

    {
        let container_ref = container_ref.clone();
        let container_id = container_id.clone();
        let return_focus = props.return_focus_id.clone();
        let prev_open_handle = prev_open;
        use_effect_with(
            (props.open, return_focus),
            move |(is_open, return_focus_id)| {
                let was_open = *prev_open_handle.borrow();
                *prev_open_handle.borrow_mut() = *is_open;
                if *is_open {
                    if let Some(el) = container_ref.cast::<web_sys::HtmlElement>() {
                        let _ = el.set_attribute("tabindex", "-1");
                        let _ = el.focus();
                    }
                    trap_focus_in(&container_id);
                } else if was_open && let Some(id) = return_focus_id.as_ref() {
                    restore_focus(id);
                }
                || {}
            },
        );
    }

    let on_close = {
        let cb = props.on_close.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_keydown = {
        let cb = props.on_close.clone();
        let return_focus_id = props.return_focus_id.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                e.prevent_default();
                cb.emit(());
                if let Some(id) = return_focus_id.as_ref() {
                    restore_focus(id);
                }
            }
        })
    };

    html! {
        <div class="modal-backdrop" role="presentation" onclick={on_close.clone()}>
            <div
                id={container_id.clone()}
                class="modal"
                role="dialog"
                aria-modal="true"
                aria-labelledby={title_id.clone()}
                aria-describedby={desc_id.clone().unwrap_or_default()}
                onkeydown={on_keydown}
                ref={container_ref}
            >
                <div class="modal__header">
                    <h2 id={title_id}>{ props.title.clone() }</h2>
                    <button type="button" class="modal__close" aria-label="Close dialog" onclick={on_close.clone()}>
                        {"X"}
                    </button>
                </div>
                { props.description.as_ref().map(|desc| {
                    let id = desc_id.clone().unwrap_or_default();
                    html! {
                        <p id={id} class="modal__description">{ desc.clone() }</p>
                    }
                }).unwrap_or_default() }
                <div class="modal__body">
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    }
}
