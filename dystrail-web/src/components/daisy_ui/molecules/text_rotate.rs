use crate::components::daisy_ui::foundation as f;
use f::JsCast;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct TextRotateProps {
    pub items: Vec<f::AttrValue>,
    #[prop_or_default]
    pub active_index: Option<usize>,
    #[prop_or_default]
    pub interval_ms: Option<u32>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(TextRotate)]
pub fn text_rotate(props: &TextRotateProps) -> f::Html {
    let item_count = props.items.len();
    if item_count == 0 {
        return f::Html::default();
    }
    let active = f::use_state(|| props.active_index.unwrap_or(0).min(item_count - 1));
    {
        let active = active.clone();
        let external = props.active_index;
        f::use_effect_with(external, move |idx| {
            if let Some(idx) = idx {
                active.set((*idx).min(item_count - 1));
            }
            || {}
        });
    }
    {
        let active = active.clone();
        let interval = props.interval_ms;
        f::use_effect_with((item_count, interval), move |(len, interval)| {
            let mut interval_id: Option<i32> = None;
            let mut stored_closure: Option<f::Closure<dyn FnMut()>> = None;
            if let (Some(window), Some(ms)) = (web_sys::window(), *interval)
                && let Ok(timeout) = i32::try_from(ms)
            {
                let total = *len;
                let handle = active;
                let closure = f::Closure::wrap(Box::new(move || {
                    let next = (*handle + 1) % total;
                    handle.set(next);
                }) as Box<dyn FnMut()>);
                if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    timeout,
                ) {
                    interval_id = Some(id);
                    stored_closure = Some(closure);
                }
            }
            move || {
                if let Some(id) = interval_id
                    && let Some(win) = web_sys::window()
                {
                    win.clear_interval_with_handle(id);
                }
                if let Some(closure) = stored_closure {
                    drop(closure);
                }
            }
        });
    }
    let class = f::class_list(&["text-rotate", "font-mono"], &props.class);
    let next = {
        let active = active.clone();
        f::Callback::from(move |_| active.set((*active + 1) % item_count))
    };
    let prev = {
        let active = active.clone();
        f::Callback::from(move |_| active.set((*active + item_count - 1) % item_count))
    };
    f::html! {
        <div class={class} role="list" aria-live="polite">
            <div class="flex items-center gap-3">
                <button class="btn btn-xs" aria-label="Previous" onclick={prev}>{"‹"}</button>
                <span>{ props.items.get(*active).cloned().unwrap_or_default() }</span>
                <button class="btn btn-xs" aria-label="Next" onclick={next}>{"›"}</button>
            </div>
        </div>
    }
}
