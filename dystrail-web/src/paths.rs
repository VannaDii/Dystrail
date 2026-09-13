//! Helpers for constructing URLs to static assets that respect the deployment base path.
///
/// When `PUBLIC_URL` is set at compile time (e.g., `/play` for GitHub Pages),
/// generated URLs are prefixed accordingly. Local builds without `PUBLIC_URL`
/// fall back to root-anchored paths.
#[must_use]
pub fn asset_path(relative: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    if let Some(url) = prepared_asset(relative.trim_start_matches('/')) {
        return url;
    }
    let base = option_env!("PUBLIC_URL")
        .unwrap_or("")
        .trim_end_matches('/');
    let rel = relative.trim_start_matches('/');

    if base.is_empty() {
        format!("/{rel}")
    } else {
        format!("{base}/{rel}")
    }
}

#[cfg(target_arch = "wasm32")]
fn prepared_asset(relative: &str) -> Option<String> {
    let window = web_sys::window()?;
    let assets = js_sys::Reflect::get(&window, &"dystrailAssetUrls".into()).ok()?;
    js_sys::Reflect::get(&assets, &relative.into())
        .ok()?
        .as_string()
}

/// Base path for the router (e.g., `/play` when hosted under a subdirectory).
///
/// Returns `None` when no base path is configured so the router falls back to root.
#[must_use]
pub fn router_base() -> Option<String> {
    let base = option_env!("PUBLIC_URL")
        .unwrap_or("")
        .trim_end_matches('/')
        .trim();

    if base.is_empty() {
        None
    } else {
        Some(base.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_path, router_base};

    #[test]
    fn builds_root_prefixed_path_when_base_missing() {
        assert_eq!(asset_path("static/img/logo.png"), "/static/img/logo.png");
        assert_eq!(asset_path("/static/img/logo.png"), "/static/img/logo.png");
    }

    #[test]
    fn router_base_is_none_by_default() {
        assert_eq!(router_base(), None);
    }
}
