use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct PaginationProps {
    pub total_pages: u32,
    #[prop_or_default]
    pub current_page: u32,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<u32>,
}

#[f::function_component(Pagination)]
pub fn pagination(props: &PaginationProps) -> f::Html {
    let class = f::class_list(&["join"], &props.class);
    let total = props.total_pages.max(1);
    let current = props.current_page.min(total - 1);
    let go_to = |page: u32| {
        let cb = props.on_change.clone();
        f::Callback::from(move |_| cb.emit(page))
    };
    f::html! {
        <div class={class} role="group" aria-label="Pagination">
            <button class="join-item btn" disabled={current == 0} onclick={go_to(current.saturating_sub(1))}>{"«"}</button>
            { for (0..total).map(|page| {
                let mut btn_class = f::classes!("join-item", "btn");
                if page == current {
                    btn_class.push("btn-active");
                }
                let label = (page + 1).to_string();
                f::html! { <button class={btn_class} aria-current={if page==current { Some::<f::AttrValue>("page".into()) } else { None }} onclick={go_to(page)}>{ label }</button> }
            })}
            <button class="join-item btn" disabled={current + 1 >= total} onclick={go_to(current + 1)}>{"»"}</button>
        </div>
    }
}
