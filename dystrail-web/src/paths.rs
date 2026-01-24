//! Helpers for constructing URLs to static assets that respect the deployment base path.
///
/// When `PUBLIC_URL` is set at compile time (e.g., `/play` for GitHub Pages),
/// generated URLs are prefixed accordingly. Local builds without `PUBLIC_URL`
/// fall back to root-anchored paths.
#[must_use]
pub fn asset_path(relative: &str) -> String {
    asset_path_with_base(relative, option_env!("PUBLIC_URL").unwrap_or(""))
}

/// Base path for the router (e.g., `/play` when hosted under a subdirectory).
///
/// Returns `None` when no base path is configured so the router falls back to root.
#[must_use]
pub fn router_base() -> Option<String> {
    router_base_with_base(option_env!("PUBLIC_URL").unwrap_or(""))
}

fn asset_path_with_base(relative: &str, base: &str) -> String {
    let base = base.trim_end_matches('/');
    let rel = relative.trim_start_matches('/');

    if base.is_empty() {
        format!("/{rel}")
    } else {
        format!("{base}/{rel}")
    }
}

fn router_base_with_base(base: &str) -> Option<String> {
    let base = base.trim_end_matches('/').trim();
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
    fn builds_paths_with_public_base() {
        assert_eq!(
            super::asset_path_with_base("static/img/logo.png", "/play"),
            "/play/static/img/logo.png"
        );
        assert_eq!(
            super::asset_path_with_base("/static/img/logo.png", "/play/"),
            "/play/static/img/logo.png"
        );
    }

    #[test]
    fn router_base_is_none_by_default() {
        assert_eq!(router_base(), None);
    }

    #[test]
    fn router_base_returns_trimmed_value() {
        assert_eq!(
            super::router_base_with_base("/play/"),
            Some(String::from("/play"))
        );
    }
}
