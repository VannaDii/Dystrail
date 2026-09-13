//! Keep mounted UI components synchronized with the selected language.
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Language(pub String);

/// Subscribe a component without resetting its local state when language changes.
#[hook]
pub fn use_language() {
    let _ = use_context::<Language>();
}
