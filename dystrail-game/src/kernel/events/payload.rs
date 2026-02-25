/// Stable payload container used by kernel events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelEventPayload(pub serde_json::Value);

impl KernelEventPayload {
    #[must_use]
    pub const fn as_value(&self) -> &serde_json::Value {
        &self.0
    }
}

impl From<serde_json::Value> for KernelEventPayload {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}
