use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct CarouselProps {
    #[prop_or_default]
    pub initial_index: usize,
    #[prop_or_default]
    pub show_indicators: bool,
    #[prop_or_default]
    pub show_controls: bool,
    #[prop_or_default]
    pub on_change: f::Callback<usize>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Carousel)]
pub fn carousel(props: &CarouselProps) -> f::Html {
    let slides: Vec<f::Html> = props.children.iter().collect();
    let len = slides.len();
    let active = f::use_state(|| props.initial_index.min(len.saturating_sub(1)));
    #[cfg(target_arch = "wasm32")]
    {
        let active = active.clone();
        let external = props.initial_index.min(len.saturating_sub(1));
        f::use_effect_with(external, move |idx| {
            active.set(*idx);
            || {}
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = props.initial_index;
    }
    let on_change = props.on_change.clone();
    let change_to = {
        #[cfg(target_arch = "wasm32")]
        {
            let active = active.clone();
            let on_change = on_change.clone();
            move |next: usize| {
                if len == 0 {
                    return f::Callback::noop();
                }
                let active = active.clone();
                let on_change = on_change.clone();
                f::Callback::from(move |_| {
                    active.set(next % len);
                    on_change.emit(next % len);
                })
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (&active, &on_change, len);
            move |_next: usize| f::Callback::noop()
        }
    };
    let class = f::class_list(&["carousel", "relative"], &props.class);
    f::html! {
        <div class={class} aria-roledescription="carousel">
            <div class="carousel-track">
                { for slides.iter().enumerate().map(|(idx, slide)| {
                    let is_active = idx == *active;
                    let mut slide_class = f::classes!("carousel-item");
                    if is_active {
                        slide_class.push("active");
                    }
                    f::html! {
                        <div class={slide_class} aria-hidden={(!is_active).to_string()}>
                            { slide.clone() }
                        </div>
                    }
                }) }
            </div>
            { if props.show_controls && len > 1 {
                let prev = (*active + len - 1) % len;
                let next = (*active + 1) % len;
                f::html! {
                    <div class="carousel-controls flex justify-between items-center mt-2">
                        <button class="btn btn-sm" aria-label="Previous slide" onclick={change_to(prev)}>{"‹"}</button>
                        <button class="btn btn-sm" aria-label="Next slide" onclick={change_to(next)}>{"›"}</button>
                    </div>
                }
            } else { f::Html::default() }}
            { if props.show_indicators && len > 1 {
                f::html! {
                    <div class="carousel-indicators flex justify-center gap-2 mt-2" role="tablist">
                        { for (0..len).map(|idx| {
                            let is_active = idx == *active;
                            let mut dot_class = f::classes!("btn", "btn-xs", "btn-circle");
                            if is_active {
                                dot_class.push("btn-active");
                            }
                            let handler = change_to(idx);
                            f::html! {
                                <button class={dot_class} aria-label={format!("Go to slide {}", idx + 1)} aria-pressed={is_active.to_string()} onclick={handler.clone()}></button>
                            }
                        }) }
                    </div>
                }
            } else { f::Html::default() }}
        </div>
    }
}
