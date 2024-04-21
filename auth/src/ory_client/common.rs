use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UiContainer {
    action: String,
    #[serde(default)]
    messages: Vec<UiText>,
    method: String,
    nodes: Vec<UiNode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiText {
    context: Option<serde_json::Value>,
    id: i64,
    text: String,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiNode {
    attributes: serde_json::Value,
    group: String,
    #[serde(default)]
    messages: Vec<UiText>,
    meta: UiNodeMeta,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiNodeMeta {
    label: Option<UiText>,
}
