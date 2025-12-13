use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct DrawerProps {
    #[prop_or_default]
    pub open: bool,
    pub side: f::Html,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_close: f::Callback<()>,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Drawer)]
pub fn drawer(props: &DrawerProps) -> f::Html {
    let class = f::class_list(&["drawer"], &props.class);
    let close = {
        let cb = props.on_close.clone();
        f::Callback::from(move |_| cb.emit(()))
    };
    f::html! {
        <div class={class} data-open={props.open.to_string()}>
            <div class="drawer-content">{ for props.children.iter() }</div>
            { if props.open {
                f::html! {
                    <>
                        <div class="drawer-side" aria-label="Drawer" aria-expanded="true">
                            <div class="p-4 w-80 bg-base-200 h-full overflow-auto">
                                { props.side.clone() }
                            </div>
                        </div>
                        <div class="drawer-overlay fixed inset-0 bg-black/30" role="presentation" onclick={close.clone()}></div>
                    </>
                }
            } else {
                f::Html::default()
            }}
        </div>
    }
}
