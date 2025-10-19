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
        let ok = result.json().as_bool().unwrap_or(false);
        if !ok {
            bail!("__dystrailTest is not available. Did you pass ?test=1 and expose the bridge?");
        }
        Ok(())
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
        let v = result.json().clone();
        let s: GameState = serde_json::from_value(v).context("parsing GameState")?;
        Ok(s)
    }
}
