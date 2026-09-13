use super::{image, post::Post, share, share_browser};
use crate::i18n;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub post: Post,
    pub on_close: Callback<()>,
}

#[function_component(ShareComposer)]
pub fn share_composer(p: &Props) -> Html {
    i18n::use_language();
    let content = use_state(|| p.post.text.clone());
    let rendered = use_state(|| None::<Result<image::ShareImage, String>>);
    let retry = use_state(|| 0u32);
    let status = use_state(String::new);
    let busy = use_state(|| false);
    let completed_shares = use_state(|| 0_u32);
    let native_button = use_node_ref();
    let container = use_node_ref();
    crate::components::ui::save_drawer::focus::use_focus_trap(
        true,
        Some("result-share-open".into()),
        container.clone(),
    );
    {
        let native_button = native_button.clone();
        use_effect_with(*completed_shares, move |completed| {
            // Native sheets can return focus to the page. Wait for the button
            // to be enabled again before returning keyboard control to it.
            if *completed > 0
                && let Some(button) = native_button.cast::<web_sys::HtmlElement>()
            {
                let _ = button.focus();
            }
        });
    }
    {
        let rendered = rendered.clone();
        use_effect_with((p.post.clone(), *retry), move |(post, _)| {
            rendered.set(None);
            let active = Rc::new(Cell::new(true));
            let pending = active.clone();
            let current_url = Rc::new(std::cell::RefCell::new(None::<String>));
            let saved_url = current_url.clone();
            let post = post.clone();
            spawn_local(async move {
                let result = match image::render(&post) {
                    Ok(task) => {
                        let value = JsFuture::from(task.promise).await;
                        drop(task.callback);
                        value.and_then(image::finish)
                    }
                    Err(error) => Err(error),
                };
                match result {
                    Ok(image) => {
                        if pending.get() {
                            *saved_url.borrow_mut() = Some(image.url.clone());
                            rendered.set(Some(Ok(image)));
                        } else {
                            let _ = web_sys::Url::revoke_object_url(&image.url);
                        }
                    }
                    Err(_) if pending.get() => {
                        rendered.set(Some(Err(i18n::t("result.compose.image_failed"))));
                    }
                    Err(_) => (),
                }
            });
            move || {
                active.set(false);
                if let Some(url) = current_url.borrow_mut().take() {
                    let _ = web_sys::Url::revoke_object_url(&url);
                }
            }
        });
    }
    let on_input = {
        let content = content.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
            {
                content.set(input.value());
            }
        })
    };
    let on_copy = {
        let content = content.clone();
        let status = status.clone();
        Callback::from(move |_| {
            let text = (*content).clone();
            let status = status.clone();
            spawn_local(async move {
                let copied = match share::copy_payload(&text) {
                    Ok(promise) => JsFuture::from(promise).await.is_ok(),
                    Err(_) => false,
                };
                status.set(i18n::t(if copied {
                    "result.announce.copied"
                } else {
                    "result.announce.copy_failed"
                }));
            });
        })
    };
    let on_keydown = {
        let trap = crate::components::ui::save_drawer::focus::focus_keydown_handler(
            container.clone(),
            p.on_close.clone(),
        );
        Callback::from(move |event: KeyboardEvent| {
            event.stop_propagation();
            trap.emit(event);
        })
    };
    let close = {
        let close = p.on_close.clone();
        Callback::from(move |_| close.emit(()))
    };
    let on_retry = { Callback::from(move |_| retry.set(retry.wrapping_add(1))) };
    html! {<div class="share-backdrop"><section class="share-composer" ref={container} role="dialog" aria-modal="true" aria-labelledby="share-title" onkeydown={on_keydown}>
        <header><div><p class="eyebrow">{i18n::t("play.outcome")}</p><h2 id="share-title">{i18n::t("result.compose.title")}</h2></div><button class="share-close" onclick={close}>{i18n::t("result.compose.close")}</button></header>
        <div class="share-content"><div class="share-preview">
            {match rendered.as_ref() {
                Some(Ok(image)) => html!{<img src={image.url.clone()} width="1200" height="1200" alt={p.post.image_alt.clone()} />},
                Some(Err(error)) => html!{<div class="share-placeholder" role="alert"><p>{error}</p><button onclick={on_retry}>{i18n::t("result.compose.retry")}</button></div>},
                None => html!{<div class="share-placeholder" role="status">{i18n::t("result.compose.preparing")}</div>},
            }}
        </div><div class="share-editor"><label for="share-post">{i18n::t("result.compose.caption")}</label><textarea id="share-post" value={(*content).clone()} oninput={on_input} rows="10" />
            <div class="share-platforms"><h3>{i18n::t("result.compose.platforms")}</h3><div>
                <a href={share_browser::intent("https://bsky.app/intent/compose?text=", &content)} target="_blank" rel="noopener noreferrer">{"Bluesky"}</a>
                <a href={share_browser::intent("https://twitter.com/intent/tweet?text=", &content)} target="_blank" rel="noopener noreferrer">{"X"}</a>
            </div><p>{i18n::t("result.compose.platform_note")}</p></div>
        </div></div>
        <footer class="share-footer">
            <div class="share-actions"><button onclick={on_copy}>{i18n::t("result.compose.copy")}</button>
                if let Some(Ok(image)) = rendered.as_ref() {
                    <a class="share-download retro-btn" href={image.url.clone()} download="dystopian-trail.png">{i18n::t("result.compose.download")}</a>
                    if share_browser::supports_files(image) {<button ref={native_button} class="retro-btn-primary" disabled={*busy} onclick={native_share(image.clone(), content.clone(), status.clone(), busy.clone(), completed_shares.clone())}>{i18n::t("result.compose.native")}</button>}
                }
            </div>
            <p class="share-status" role="status" aria-live="polite">{&*status}</p>
        </footer>
    </section></div>}
}

fn native_share(
    image: image::ShareImage,
    content: UseStateHandle<String>,
    status: UseStateHandle<String>,
    busy: UseStateHandle<bool>,
    completed_shares: UseStateHandle<u32>,
) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let promise = share_browser::start(&image, &content);
        busy.set(true);
        let status = status.clone();
        let busy = busy.clone();
        let completed_shares = completed_shares.clone();
        spawn_local(async move {
            let result = match promise {
                Ok(promise) => JsFuture::from(promise).await,
                Err(error) => Err(error),
            };
            let key = match result {
                Ok(_) => "result.compose.handed_off",
                Err(error) if share_browser::cancelled(&error) => "result.compose.cancelled",
                Err(_) => "result.compose.share_failed",
            };
            status.set(i18n::t(key));
            busy.set(false);
            completed_shares.set(completed_shares.wrapping_add(1));
        });
    })
}
