use std::time::Duration;
use thirtyfour::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BrowserKind {
    Chrome,
    Edge,
    Firefox,
    Safari,
}

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub headless: bool,
    pub implicit_wait_secs: u64,
    pub remote_hub: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            implicit_wait_secs: 3,
            remote_hub: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionPlan {
    kind: BrowserKind,
    url: String,
    headless: bool,
    implicit_wait_secs: u64,
}

impl SessionPlan {
    fn from_config(kind: BrowserKind, cfg: &BrowserConfig) -> Self {
        let url = cfg.remote_hub.clone().unwrap_or_else(|| match kind {
            BrowserKind::Chrome => "http://localhost:9515".to_string(),
            BrowserKind::Edge => "http://localhost:17556".to_string(),
            BrowserKind::Firefox => "http://localhost:4444".to_string(),
            BrowserKind::Safari => "http://localhost:4445".to_string(),
        });
        Self {
            kind,
            url,
            headless: cfg.headless,
            implicit_wait_secs: cfg.implicit_wait_secs,
        }
    }
}

fn build_capabilities(kind: BrowserKind, headless: bool) -> WebDriverResult<Capabilities> {
    match kind {
        BrowserKind::Chrome => {
            let mut caps = DesiredCapabilities::chrome();
            if headless {
                caps.set_headless()?;
            }
            Ok(caps.into())
        }
        BrowserKind::Edge => {
            let mut caps = DesiredCapabilities::edge();
            if headless {
                caps.set_headless()?;
            }
            Ok(caps.into())
        }
        BrowserKind::Firefox => {
            let mut caps = DesiredCapabilities::firefox();
            if headless {
                caps.set_headless()?;
            }
            Ok(caps.into())
        }
        BrowserKind::Safari => Ok(DesiredCapabilities::safari().into()),
    }
}

pub async fn new_session(kind: BrowserKind, cfg: &BrowserConfig) -> WebDriverResult<WebDriver> {
    let plan = SessionPlan::from_config(kind, cfg);
    let caps = build_capabilities(plan.kind, plan.headless)?;
    let driver = WebDriver::new(plan.url.as_str(), caps).await?;

    driver
        .set_implicit_wait_timeout(Duration::from_secs(plan.implicit_wait_secs))
        .await?;
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_plan_uses_defaults_by_kind() {
        let cfg = BrowserConfig::default();
        let chrome = SessionPlan::from_config(BrowserKind::Chrome, &cfg);
        assert_eq!(chrome.url, "http://localhost:9515");
        let edge = SessionPlan::from_config(BrowserKind::Edge, &cfg);
        assert_eq!(edge.url, "http://localhost:17556");
        let firefox = SessionPlan::from_config(BrowserKind::Firefox, &cfg);
        assert_eq!(firefox.url, "http://localhost:4444");
        let safari = SessionPlan::from_config(BrowserKind::Safari, &cfg);
        assert_eq!(safari.url, "http://localhost:4445");
    }

    #[test]
    fn session_plan_prefers_remote_hub() {
        let cfg = BrowserConfig {
            headless: false,
            implicit_wait_secs: 1,
            remote_hub: Some("http://remote.example".to_string()),
        };
        let plan = SessionPlan::from_config(BrowserKind::Chrome, &cfg);
        assert_eq!(plan.url, "http://remote.example");
        assert!(!plan.headless);
        assert_eq!(plan.implicit_wait_secs, 1);
    }

    #[test]
    fn build_capabilities_handles_all_kinds() {
        for kind in [
            BrowserKind::Chrome,
            BrowserKind::Edge,
            BrowserKind::Firefox,
            BrowserKind::Safari,
        ] {
            let caps = build_capabilities(kind, true);
            assert!(caps.is_ok());
        }
    }
}
