use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thirtyfour::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    pub screen: Option<String>,
    pub hp: Option<i64>,
    pub day: Option<i64>,
    pub pos: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct TestBridge<'a> {
    driver: &'a WebDriver,
}

impl<'a> TestBridge<'a> {
    pub const fn new(driver: &'a WebDriver) -> Self {
        Self { driver }
    }

    pub async fn ensure_available(&self) -> Result<()> {
        #[rustfmt::skip]
        let result = self.driver.execute("return !!window.__dystrailTest", vec![]).await?;
        parse_bridge_available(result.json())
    }

    pub async fn seed(&self, n: i64) -> Result<()> {
        self.driver
            .execute("window.__dystrailTest.seed(arguments[0])", vec![n.into()])
            .await?;
        Ok(())
    }

    pub async fn speed(&self, mult: f64) -> Result<()> {
        self.driver
            .execute(
                "window.__dystrailTest.speed(arguments[0])",
                vec![mult.into()],
            )
            .await?;
        Ok(())
    }

    pub async fn click(&self, x: i64, y: i64) -> Result<()> {
        self.driver
            .execute(
                "window.__dystrailTest.click(arguments[0], arguments[1])",
                vec![x.into(), y.into()],
            )
            .await?;
        Ok(())
    }

    pub async fn key(&self, k: &str) -> Result<()> {
        self.driver
            .execute("window.__dystrailTest.key(arguments[0])", vec![k.into()])
            .await?;
        Ok(())
    }

    pub async fn state(&self) -> Result<GameState> {
        #[rustfmt::skip]
        let result = self.driver.execute("return window.__dystrailTest.state()", vec![]).await?;
        parse_game_state(result.json())
    }

    pub async fn screen(&self, name: &str) -> Result<()> {
        self.driver
            .execute(
                "window.__dystrailTest.screen(arguments[0])",
                vec![name.into()],
            )
            .await?;
        Ok(())
    }
}

fn parse_bridge_available(value: &Value) -> Result<()> {
    let ok = value.as_bool().unwrap_or(false);
    if !ok {
        bail!("__dystrailTest is not available. Did you pass ?test=1 and expose the bridge?");
    }
    Ok(())
}

fn parse_game_state(value: &Value) -> Result<GameState> {
    let parsed = serde_json::from_value(value.clone()).context("parsing GameState")?;
    Ok(parsed)
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
    fn bridge_game_state_defaults_empty() {
        let state = GameState::default();
        assert!(state.screen.is_none());
        assert!(state.hp.is_none());
        assert!(state.pos.is_none());
    }

    #[test]
    fn bridge_available_accepts_true() {
        parse_bridge_available(&json!(true)).expect("bridge should be available");
    }

    #[test]
    fn bridge_available_rejects_false() {
        let err = parse_bridge_available(&json!(false)).expect_err("bridge should be missing");
        assert!(err.to_string().contains("__dystrailTest"));
    }

    #[test]
    fn parse_game_state_from_json() {
        let value = json!({
            "screen": "travel",
            "hp": 9,
            "day": 2,
            "pos": { "x": 1 }
        });
        let parsed = parse_game_state(&value).expect("state should parse");
        assert_eq!(parsed.screen.as_deref(), Some("travel"));
        assert_eq!(parsed.hp, Some(9));
        assert_eq!(parsed.day, Some(2));
    }

    #[test]
    fn parse_game_state_rejects_invalid_value() {
        let err = parse_game_state(&json!("bad")).expect_err("invalid state should fail");
        assert!(err.to_string().contains("parsing GameState"));
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
            let value = if script.contains("return !!window.__dystrailTest") {
                json!(true)
            } else if script.contains("__dystrailTest.state") {
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
    async fn bridge_executes_commands_against_mock_driver() {
        let (hub, shutdown) = spawn_mock_webdriver();
        let cfg = BrowserConfig {
            headless: true,
            implicit_wait_secs: 0,
            remote_hub: Some(hub),
        };
        let driver = new_session(BrowserKind::Chrome, &cfg)
            .await
            .expect("driver");
        let bridge = TestBridge::new(&driver);

        bridge.ensure_available().await.expect("bridge available");
        bridge.seed(42).await.expect("seed ok");
        bridge.speed(2.0).await.expect("speed ok");
        bridge.click(10, 12).await.expect("click ok");
        bridge.key("w").await.expect("key ok");
        bridge.screen("travel").await.expect("screen ok");
        let state = bridge.state().await.expect("state ok");
        assert_eq!(state.screen.as_deref(), Some("travel"));
        assert_eq!(state.hp, Some(9));
        assert_eq!(state.day, Some(2));

        driver.quit().await.expect("quit");
        let _ = shutdown.send(());
    }
}
