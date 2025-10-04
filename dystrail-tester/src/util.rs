use anyhow::{Context, Result};
use chrono::Utc;
use std::{fs, path::Path};
use thirtyfour::prelude::*;

pub fn artifacts_dir(base: &str, browser: &str, scenario: &str, seed: i64) -> String {
    let ts = Utc::now().format("%Y%m%dT%H%M%S");
    format!("{base}/{browser}/{scenario}/seed-{seed}/{ts}")
}

pub async fn capture_artifacts(driver: &WebDriver, dir: &str, err: &anyhow::Error) -> Result<()> {
    fs::create_dir_all(dir).context("creating artifacts dir")?;

    // Screenshot
    if let Ok(png) = driver.screenshot_as_png().await {
        let _ = fs::write(Path::new(dir).join("screenshot.png"), &png);
    }

    // Page source (even if canvas)
    if let Ok(src) = driver.source().await {
        let _ = fs::write(Path::new(dir).join("dom.html"), src);
    }

    // State dump via test bridge (best-effort)
    if let Ok(ret) = driver.execute("return window.__dystrailTest && window.__dystrailTest.state && window.__dystrailTest.state()", vec![]).await {
        let v = ret.json();
        let _ = fs::write(Path::new(dir).join("state.json"), serde_json::to_vec_pretty(v).unwrap_or_default());
    }

    // Error chain
    let chain = format!("{err:#}");
    let _ = fs::write(Path::new(dir).join("error.txt"), chain);

    Ok(())
}

pub fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
