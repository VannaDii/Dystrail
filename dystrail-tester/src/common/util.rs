use anyhow::{Context, Result};
use chrono::Utc;
use std::{fs, path::Path};
use thirtyfour::prelude::*;

pub fn artifacts_dir(base: &str, browser: &str, scenario: &str, seed: u64) -> String {
    let ts = Utc::now().format("%Y%m%dT%H%M%S");
    format!("{base}/{browser}/{scenario}/seed-{seed}/{ts}")
}

pub async fn capture_artifacts(driver: &WebDriver, dir: &str, err: &anyhow::Error) -> Result<()> {
    let screenshot = driver.screenshot_as_png().await.ok();
    let source = driver.source().await.ok();
    let state = driver
        .execute(
            "return window.__dystrailTest && window.__dystrailTest.state && window.__dystrailTest.state()",
            vec![],
        )
        .await
        .ok()
        .map(|ret| ret.json().clone());
    let chain = format!("{err:#}");

    write_artifact_files(
        Path::new(dir),
        screenshot.as_deref(),
        source.as_deref(),
        state.as_ref(),
        &chain,
    )
}

fn write_artifact_files(
    dir: &Path,
    screenshot: Option<&[u8]>,
    source: Option<&str>,
    state: Option<&serde_json::Value>,
    error_chain: &str,
) -> Result<()> {
    fs::create_dir_all(dir).context("creating artifacts dir")?;

    if let Some(png) = screenshot {
        let _ = fs::write(dir.join("screenshot.png"), png);
    }

    if let Some(src) = source {
        let _ = fs::write(dir.join("dom.html"), src);
    }

    if let Some(state_json) = state {
        let payload = serde_json::to_vec_pretty(state_json).unwrap_or_default();
        let _ = fs::write(dir.join("state.json"), payload);
    }

    let _ = fs::write(dir.join("error.txt"), error_chain);

    Ok(())
}

pub fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn split_csv_trims_and_filters() {
        let parts = split_csv(" alpha, ,beta,  gamma ");
        assert_eq!(parts, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn artifacts_dir_includes_key_segments() {
        let dir = artifacts_dir("target/out", "chrome", "smoke", 42);
        assert!(dir.contains("target/out/chrome/smoke/seed-42/"));
    }

    #[test]
    fn write_artifact_files_writes_expected_payloads() {
        let base = std::env::temp_dir().join(format!(
            "dystrail-artifacts-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let state = json!({ "hp": 9, "day": 2 });
        write_artifact_files(
            &base,
            Some(&[1, 2, 3]),
            Some("<html />"),
            Some(&state),
            "boom",
        )
        .expect("write artifacts");

        assert!(base.join("screenshot.png").exists());
        assert!(base.join("dom.html").exists());
        assert!(base.join("state.json").exists());
        assert!(base.join("error.txt").exists());
    }
}
