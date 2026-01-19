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
        let result = self
            .driver
            .execute("return !!window.__dystrailTest", vec![])
            .await?;
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
        let result = self
            .driver
            .execute("return window.__dystrailTest.state()", vec![])
            .await?;
        parse_game_state(result.json())
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
    use serde_json::json;

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
}
