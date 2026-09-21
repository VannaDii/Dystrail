//! Rust/Wasm composes a PNG using artwork already prepared by the offline gate.
use super::post::Post;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Blob, CanvasRenderingContext2d as Canvas, HtmlCanvasElement, HtmlImageElement};

struct ShareTheme {
    panel: String,
    inset: String,
    border: String,
    text: String,
    dim: String,
    accent: String,
    scrim: String,
    body_font: String,
    title_font: String,
}

impl ShareTheme {
    fn current(document: &web_sys::Document) -> Result<Self, JsValue> {
        let root = document
            .document_element()
            .ok_or_else(|| JsValue::from_str("Theme root unavailable"))?;
        let style = web_sys::window()
            .ok_or_else(|| JsValue::from_str("Window unavailable"))?
            .get_computed_style(&root)?
            .ok_or_else(|| JsValue::from_str("Theme unavailable"))?;
        let token = |name: &str| -> Result<String, JsValue> {
            let value = style.get_property_value(name)?.trim().to_owned();
            if value.is_empty() {
                Err(JsValue::from_str(&format!(
                    "Missing share theme token: {name}"
                )))
            } else {
                Ok(value)
            }
        };
        Ok(Self {
            panel: token("--panel")?,
            inset: token("--panel-inset")?,
            border: token("--panel-border")?,
            text: token("--text-bright")?,
            dim: token("--text-dim")?,
            accent: token("--accent")?,
            scrim: token("--scene-scrim")?,
            body_font: token("--body-font")?,
            title_font: token("--title-font")?,
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ShareImage {
    pub url: String,
    pub blob: Blob,
}

pub fn render(post: &Post) -> Result<RenderTask, JsValue> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| JsValue::from_str("Document unavailable"))?;
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(1200);
    canvas.set_height(1200);
    let ctx = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas unavailable"))?
        .dyn_into::<Canvas>()?;
    let scene = prepared_image("static/img/journey/journey-settings-v1.png")?;
    let avatar = prepared_image(&post.avatar.path)?;
    let theme = ShareTheme::current(&document)?;
    draw_header(&ctx, &scene, &avatar, post, &theme)?;
    draw_stats(&ctx, post, &theme)?;
    png(&canvas)
}

fn draw_header(
    ctx: &Canvas,
    scene: &HtmlImageElement,
    avatar: &HtmlImageElement,
    post: &Post,
    theme: &ShareTheme,
) -> Result<(), JsValue> {
    fill(ctx, &theme.panel, 0.0, 0.0, 1200.0, 1200.0);
    ctx.set_image_smoothing_enabled(false);
    let cell_w = f64::from(scene.natural_width()) / 2.0;
    let cell_h = f64::from(scene.natural_height()) / 3.0;
    ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        scene,
        if post.arrived { 0.0 } else { cell_w },
        cell_h * 2.0,
        cell_w,
        cell_h,
        0.0,
        112.0,
        1200.0,
        392.0,
    )?;
    ctx.set_global_alpha(0.6);
    fill(ctx, &theme.scrim, 0.0, 112.0, 1200.0, 392.0);
    ctx.set_global_alpha(1.0);
    line(
        ctx,
        "DYSTOPIAN TRAIL",
        56.0,
        74.0,
        &format!("bold 38px {}", theme.title_font),
        &theme.text,
    )?;
    fill(ctx, &theme.accent, 56.0, 99.0, 330.0, 4.0);
    let portrait_x = if post.rtl { 840.0 } else { 56.0 };
    fill(ctx, &theme.inset, portrait_x, 156.0, 304.0, 304.0);
    let avatar_width =
        f64::from(avatar.natural_width()) / if post.avatar.cell.is_some() { 2.0 } else { 1.0 };
    ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        avatar,
        f64::from(post.avatar.cell.unwrap_or(0)) * avatar_width,
        0.0,
        avatar_width,
        f64::from(avatar.natural_height()),
        portrait_x + 10.0,
        166.0,
        284.0,
        284.0,
    )?;
    ctx.set_stroke_style_str(&theme.accent);
    ctx.set_line_width(2.0);
    ctx.stroke_rect(portrait_x, 156.0, 304.0, 304.0);
    js_sys::Reflect::set(
        ctx,
        &"direction".into(),
        &if post.rtl { "rtl" } else { "ltr" }.into(),
    )?;
    let identity_x = if post.rtl { 792.0 } else { 408.0 };
    ctx.set_shadow_color("#000");
    ctx.set_shadow_blur(10.0);
    fit_name(ctx, &post.player, &theme.body_font)?;
    ctx.set_fill_style_str(&theme.text);
    let name_end = wrapped(ctx, &post.player, identity_x, 252.0, 736.0, 58.0, 2)?;
    ctx.set_font(&format!("30px {}", theme.body_font));
    ctx.set_fill_style_str(&theme.text);
    let role_end = wrapped(
        ctx,
        &post.persona,
        identity_x,
        name_end + 48.0,
        736.0,
        36.0,
        2,
    )?;
    line(
        ctx,
        &post.mode,
        identity_x,
        role_end + 42.0,
        &format!("bold 26px {}", theme.body_font),
        &theme.accent,
    )?;
    ctx.set_shadow_blur(0.0);
    let start = if post.rtl { 1144.0 } else { 56.0 };
    ctx.set_font(&format!("bold 46px {}", theme.title_font));
    ctx.set_fill_style_str(&theme.text);
    wrapped(ctx, &post.headline, start, 579.0, 1088.0, 52.0, 2)?;
    ctx.set_font(&format!("bold 28px {}", theme.body_font));
    ctx.set_fill_style_str(&theme.accent);
    wrapped(ctx, &post.location, start, 694.0, 1088.0, 32.0, 1)?;
    line(
        ctx,
        &post.progress,
        start,
        736.0,
        &format!("26px {}", theme.body_font),
        &theme.dim,
    )?;
    Ok(())
}

fn draw_stats(ctx: &Canvas, post: &Post, theme: &ShareTheme) -> Result<(), JsValue> {
    for (index, (label, value)) in post.stats.iter().enumerate() {
        let column = u32::try_from(index % 3).unwrap_or(0);
        let row = u32::try_from(index / 3).unwrap_or(0);
        let x = 56.0 + f64::from(if post.rtl { 2 - column } else { column }) * 376.0;
        let y = 794.0 + f64::from(row) * 150.0;
        fill(ctx, &theme.border, x, y, 336.0, 1.0);
        let start = x + if post.rtl { 336.0 } else { 0.0 };
        line(
            ctx,
            value,
            start,
            y + 62.0,
            &format!("bold 50px {}", theme.body_font),
            &theme.text,
        )?;
        ctx.set_font(&format!("24px {}", theme.body_font));
        ctx.set_fill_style_str(&theme.dim);
        wrapped(ctx, label, start, y + 101.0, 336.0, 27.0, 2)?;
    }
    fill(ctx, &theme.accent, 56.0, 1105.0, 1088.0, 2.0);
    let start = if post.rtl { 1144.0 } else { 56.0 };
    line(
        ctx,
        &format!("{}  {}", crate::i18n::t("result.labels.seed"), post.seed),
        start,
        1159.0,
        &format!("24px {}", theme.body_font),
        &theme.dim,
    )?;
    js_sys::Reflect::set(ctx, &"direction".into(), &"ltr".into())?;
    line(
        ctx,
        "dystrail.com",
        if post.rtl { 56.0 } else { 934.0 },
        1159.0,
        &format!("bold 28px {}", theme.title_font),
        &theme.accent,
    )?;
    Ok(())
}

pub fn finish(blob: JsValue) -> Result<ShareImage, JsValue> {
    let blob = blob.dyn_into::<Blob>()?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    Ok(ShareImage { url, blob })
}

fn prepared_image(path: &str) -> Result<HtmlImageElement, JsValue> {
    let source = crate::paths::asset_path(path);
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Window unavailable"))?;
    let images = js_sys::Reflect::get(&window, &"dystrailPreparedImages".into())?
        .dyn_into::<js_sys::Array>()?;
    images
        .iter()
        .filter_map(|value| value.dyn_into::<HtmlImageElement>().ok())
        .find(|image| image.src() == source && image.complete() && image.natural_width() > 0)
        .ok_or_else(|| JsValue::from_str("Artwork has not passed the loading gate"))
}

fn fill(ctx: &Canvas, color: &str, x: f64, y: f64, width: f64, height: f64) {
    ctx.set_fill_style_str(color);
    ctx.fill_rect(x, y, width, height);
}
fn line(ctx: &Canvas, text: &str, x: f64, y: f64, font: &str, color: &str) -> Result<(), JsValue> {
    ctx.set_font(font);
    ctx.set_fill_style_str(color);
    ctx.set_text_align("start");
    ctx.fill_text(text, x, y)
}
fn wrapped(
    ctx: &Canvas,
    text: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    limit: usize,
) -> Result<f64, JsValue> {
    ctx.set_text_align("start");
    let mut lines = text_lines(ctx, text, width)?;
    if lines.len() > limit {
        lines.truncate(limit);
        if let Some(last) = lines.last_mut() {
            while ctx.measure_text(&format!("{last}…"))?.width() > width && !last.is_empty() {
                last.pop();
            }
            last.push('…');
        }
    }
    let mut baseline = y;
    for (index, text) in lines.iter().enumerate() {
        baseline = y + f64::from(u32::try_from(index).unwrap_or(0)) * height;
        ctx.fill_text(text.trim(), x, baseline)?;
    }
    Ok(baseline)
}

fn fit_name(ctx: &Canvas, name: &str, family: &str) -> Result<(), JsValue> {
    for size in (32..=52).rev() {
        ctx.set_font(&format!("bold {size}px {family}"));
        if text_lines(ctx, name, 736.0)?.len() <= 2 {
            break;
        }
    }
    Ok(())
}

fn text_lines(ctx: &Canvas, text: &str, width: f64) -> Result<Vec<String>, JsValue> {
    let mut lines = vec![String::new()];
    for word in text.split_inclusive(char::is_whitespace) {
        let last = lines
            .last_mut()
            .ok_or_else(|| JsValue::from_str("Missing text line"))?;
        if !last.is_empty() && ctx.measure_text(&format!("{last}{word}"))?.width() > width {
            lines.push(String::new());
        }
        for character in word.chars() {
            let last = lines
                .last_mut()
                .ok_or_else(|| JsValue::from_str("Missing text line"))?;
            if ctx.measure_text(&format!("{last}{character}"))?.width() > width {
                lines.push(character.to_string());
            } else {
                last.push(character);
            }
        }
    }
    Ok(lines)
}

pub struct RenderTask {
    pub promise: js_sys::Promise,
    pub callback: Closure<dyn FnMut(Option<Blob>)>,
}

fn png(canvas: &HtmlCanvasElement) -> Result<RenderTask, JsValue> {
    let mut handler = None;
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let callback = Closure::once(move |blob: Option<Blob>| {
            if let Some(blob) = blob {
                let _ = resolve.call1(&JsValue::NULL, &blob);
            } else {
                let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("PNG encoding failed"));
            }
        });
        if let Err(error) = canvas.to_blob(callback.as_ref().unchecked_ref()) {
            handler = Some(Err(error));
        } else {
            handler = Some(Ok(callback));
        }
    });
    let callback = handler.ok_or_else(|| JsValue::from_str("PNG encoder unavailable"))??;
    Ok(RenderTask { promise, callback })
}
