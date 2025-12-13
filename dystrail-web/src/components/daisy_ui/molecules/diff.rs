use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct DiffProps {
    pub before: f::AttrValue,
    pub after: f::AttrValue,
    #[prop_or_default]
    pub caption: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Diff)]
pub fn diff(props: &DiffProps) -> f::Html {
    let class = f::class_list(&["diff", "grid", "grid-cols-2", "gap-4"], &props.class);
    f::html! {
        <div class={class}>
            <div class="diff-item diff-before" aria-label="Before">
                { props.before.clone() }
            </div>
            <div class="diff-item diff-after" aria-label="After">
                { props.after.clone() }
            </div>
            { props.caption.as_ref().map(|c| f::html! { <p class="col-span-2 text-sm text-base-content/70">{ c.clone() }</p> }).unwrap_or_default() }
        </div>
    }
}
