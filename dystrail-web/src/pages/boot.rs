use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BootPageProps {
    pub logo_src: AttrValue,
    pub ready: bool,
    pub preload_progress: u8,
    pub on_begin: Callback<()>,
}

#[function_component(BootPage)]
pub fn boot_page(props: &BootPageProps) -> Html {
    let on_click = {
        let on_begin = props.on_begin.clone();
        let ready = props.ready;
        Callback::from(move |_| {
            if ready {
                on_begin.emit(());
            }
        })
    };

    let on_keydown = {
        let on_begin = props.on_begin.clone();
        let ready = props.ready;
        Callback::from(move |e: web_sys::KeyboardEvent| {
            if ready {
                e.prevent_default();
                on_begin.emit(());
            }
        })
    };

    html! {
        <div
            class="min-h-screen flex items-center justify-center bg-gradient-to-b from-[#4EC9E6] to-[#87CEEB] relative"
            aria-busy={(!props.ready).to_string()}
            aria-live="polite"
            onkeydown={on_keydown}
            onclick={on_click}
            tabindex="0"
        >
            // Ornate frame border
            <div class="absolute inset-0 border-[16px] border-[#8B6914] shadow-[inset_0_0_0_4px_#D4A76A,inset_0_0_0_8px_#654321]" style="pointer-events: none;"></div>

            <div class="card bg-gradient-to-b from-[#87CEEB] via-[#4EC9E6] to-[#E8D088] w-[600px] max-w-[90vw] rounded-none shadow-2xl border-4 border-[#D4A76A] relative overflow-hidden">
                // Prairie foreground
                <div class="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-[#C9A961] to-[#E8D088]"></div>

                <div class="card-body items-center text-center space-y-6 relative z-10 p-8">
                    // Title Banner
                    <div class="bg-white border-4 border-[#D4A76A] shadow-lg px-8 py-4 mb-4">
                        <h1 class="text-4xl text-black tracking-wide" style="font-family: Georgia, 'Times New Roman', serif;">{ "Dystrail" }</h1>
                    </div>

                    // Subtitle
                    <div class="bg-[#E8D088] border-2 border-[#8B6914] px-6 py-2 shadow-md">
                        <p class="text-sm text-[#654321] font-semibold">{ "What if Oregon Trail went wrong?" }</p>
                    </div>

                    // Loading Indicator
                    <div class="w-full max-w-md space-y-3 mt-8">
                        <div class="bg-white border-2 border-[#8B6914] p-4 shadow-md">
                            <progress
                                class="progress h-6 w-full"
                                value={props.preload_progress.to_string()}
                                max="100"
                                role="progressbar"
                                aria-valuemin="0"
                                aria-valuemax="100"
                                aria-valuenow={props.preload_progress.to_string()}
                                style="background-color: #C9A961;"
                            />
                        </div>
                        <p class="text-sm text-[#654321] font-semibold bg-[#E8D088] border-2 border-[#8B6914] px-4 py-2 inline-block shadow-sm">
                            { if props.ready { "Ready" } else { "Loading assets…" } }
                        </p>
                    </div>

                    // "Press Any Key" Prompt
                    if props.ready {
                        <div class={classes!("mt-6", if props.ready { Some("animate-pulse-slow") } else { None })}>
                            <div class="bg-[#E8D088] border-2 border-[#8B6914] px-6 py-3 shadow-lg inline-block">
                                <span class="text-base text-[#654321] font-bold">{ crate::i18n::t("ui.cta_start") }</span>
                            </div>
                        </div>
                    }

                    // Footer / Build Info
                    <div class="text-xs text-[#654321] mt-6 bg-[#E8D088] border border-[#8B6914] px-4 py-1 rounded-sm">
                        <p>{ "v0.1 • Seed system ready" }</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
