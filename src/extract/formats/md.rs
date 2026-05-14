//! Markdown text extraction

use serde_json::json;

/// Extract raw markdown text
pub fn extract(data: &[u8]) -> serde_json::Value {
    let text = String::from_utf8_lossy(data);

    json!({
        "text": text
    })
}