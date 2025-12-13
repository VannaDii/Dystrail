use crate::components::daisy_ui::foundation as f;

#[derive(Clone, PartialEq)]
pub struct ToastItem {
    pub id: f::AttrValue,
    pub content: f::Html,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct ToastProps {
    pub toasts: Vec<ToastItem>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_dismiss: Option<f::Callback<f::AttrValue>>,
}

#[f::function_component(Toast)]
pub fn toast(props: &ToastProps) -> f::Html {
    let class = f::class_list(&["toast", "toast-end", "toast-top"], &props.class);
    f::html! {
        <div class={class} role="status" aria-live="polite">
            { for props.toasts.iter().map(|toast| {
                let dismiss_btn = props.on_dismiss.as_ref().map(|cb| {
                    let id = toast.id.clone();
                    let cb = cb.clone();
                    let on_click = f::Callback::from(move |_| cb.emit(id.clone()));
                    f::html! { <button class="btn btn-ghost btn-xs" aria-label="Dismiss" onclick={on_click}>{"✕"}</button> }
                }).unwrap_or_default();
                f::html! {
                    <div class="alert alert-info flex items-center gap-2">
                        { toast.content.clone() }
                        { dismiss_btn }
                    </div>
                }
            }) }
        </div>
    }
}
