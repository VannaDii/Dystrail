use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct ModalProps {
    pub open: bool,
    pub title: f::AttrValue,
    #[prop_or_default]
    pub description: Option<f::AttrValue>,
    #[prop_or_default]
    pub actions: Option<f::Html>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_close: f::Callback<()>,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Modal)]
pub fn modal(props: &ModalProps) -> f::Html {
    if !props.open {
        return f::Html::default();
    }
    let class = f::class_list(&["modal", "modal-open"], &props.class);
    let close = {
        let cb = props.on_close.clone();
        f::Callback::from(move |_| cb.emit(()))
    };
    f::html! {
        <div class={class} role="dialog" aria-modal="true" aria-label={props.title.clone()}>
            <div class="modal-box">
                <div class="flex justify-between items-start">
                    <h3 class="font-bold text-lg">{ props.title.clone() }</h3>
                    <button class="btn btn-ghost btn-sm" aria-label="Close" onclick={close.clone()}>{"✕"}</button>
                </div>
                { props.description.as_ref().map(|d| f::html!{ <p class="py-2 text-base-content/70">{ d.clone() }</p> }).unwrap_or_default() }
                <div class="py-2">
                    { for props.children.iter() }
                </div>
                { props.actions.clone().map(|actions| f::html!{
                    <div class="modal-action">{ actions }</div>
                }).unwrap_or_default() }
            </div>
            <div class="modal-backdrop" onclick={close}></div>
        </div>
    }
}
