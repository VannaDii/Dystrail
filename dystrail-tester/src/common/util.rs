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
    use crate::browser::{BrowserConfig, BrowserKind, new_session};
    use hyper::body::to_bytes;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Method, Request, Response, Server, StatusCode};
    use serde_json::json;
    use tokio::sync::oneshot;

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

    fn response_with_value(value: &serde_json::Value) -> Response<Body> {
        let payload = serde_json::json!({ "value": value });
        let body = serde_json::to_vec(&payload).unwrap_or_default();
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap_or_else(|_| Response::new(Body::from("{}")))
    }

    async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        if method == Method::POST && path == "/session" {
            let payload = json!({
                "value": {
                    "sessionId": "mock-session",
                    "capabilities": { "browserName": "mock" }
                }
            });
            let body = serde_json::to_vec(&payload).unwrap_or_default();
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap_or_else(|_| Response::new(Body::from("{}"))));
        }

        if method == Method::POST && path.ends_with("/timeouts") {
            return Ok(response_with_value(&serde_json::Value::Null));
        }

        if method == Method::POST && path.ends_with("/execute/sync") {
            let body = to_bytes(req.into_body()).await?;
            let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
            let script = payload.get("script").and_then(|v| v.as_str()).unwrap_or("");
            let value = if script.contains("__dystrailTest.state") {
                json!({
                    "screen": "travel",
                    "hp": 9,
                    "day": 2,
                    "pos": { "x": 1 }
                })
            } else {
                serde_json::Value::Null
            };
            return Ok(response_with_value(&value));
        }

        if method == Method::GET && path.ends_with("/source") {
            return Ok(response_with_value(&json!("<html></html>")));
        }

        if method == Method::GET && path.ends_with("/screenshot") {
            return Ok(response_with_value(&json!("AA==")));
        }

        if method == Method::DELETE && path.starts_with("/session/") {
            return Ok(response_with_value(&serde_json::Value::Null));
        }

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .unwrap_or_else(|_| Response::new(Body::from("not found"))))
    }

    fn spawn_mock_webdriver() -> (String, oneshot::Sender<()>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let addr = listener.local_addr().expect("local addr");
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let service =
            make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });
        let server = Server::from_tcp(listener)
            .expect("server")
            .serve(service)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });
        tokio::spawn(server);

        (format!("http://{addr}"), shutdown_tx)
    }

    #[tokio::test]
    async fn capture_artifacts_writes_from_mock_driver() {
        let (hub, shutdown) = spawn_mock_webdriver();
        let cfg = BrowserConfig {
            headless: true,
            implicit_wait_secs: 0,
            remote_hub: Some(hub),
        };
        let driver = new_session(BrowserKind::Chrome, &cfg)
            .await
            .expect("driver");
        let dir = std::env::temp_dir().join(format!(
            "dystrail-capture-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        let err = anyhow::anyhow!("kaboom");
        capture_artifacts(&driver, dir.to_str().unwrap_or_default(), &err)
            .await
            .expect("capture artifacts");

        assert!(dir.join("screenshot.png").exists());
        assert!(dir.join("dom.html").exists());
        assert!(dir.join("state.json").exists());
        assert!(dir.join("error.txt").exists());

        driver.quit().await.expect("quit");
        let _ = shutdown.send(());
    }
}
