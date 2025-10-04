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

pub async fn new_session(kind: BrowserKind, cfg: &BrowserConfig) -> WebDriverResult<WebDriver> {
    let driver = match kind {
        BrowserKind::Chrome => {
            let mut caps = DesiredCapabilities::chrome();
            if cfg.headless {
                caps.set_headless()?;
            }

            let url = cfg.remote_hub.as_deref().unwrap_or("http://localhost:9515");
            WebDriver::new(url, caps).await?
        }
        BrowserKind::Edge => {
            let mut caps = DesiredCapabilities::edge();
            if cfg.headless {
                caps.set_headless()?;
            }

            let url = cfg
                .remote_hub
                .as_deref()
                .unwrap_or("http://localhost:17556");
            WebDriver::new(url, caps).await?
        }
        BrowserKind::Firefox => {
            let mut caps = DesiredCapabilities::firefox();
            if cfg.headless {
                caps.set_headless()?;
            }

            let url = cfg.remote_hub.as_deref().unwrap_or("http://localhost:4444");
            WebDriver::new(url, caps).await?
        }
        BrowserKind::Safari => {
            let caps = DesiredCapabilities::safari();
            let url = cfg.remote_hub.as_deref().unwrap_or("http://localhost:4445");
            WebDriver::new(url, caps).await?
        }
    };

    driver
        .set_implicit_wait_timeout(Duration::from_secs(cfg.implicit_wait_secs))
        .await?;
    Ok(driver)
}
