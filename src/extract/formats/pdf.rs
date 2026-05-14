//! PDF text extraction using lopdf

use serde_json::{json, Value};

/// Extract text from PDF documents
pub fn extract(data: &[u8]) -> Value {
    match lopdf::Document::load_mem(data) {
        Ok(doc) => {
            let page_count = doc.get_pages().len();
            let mut pages: Vec<Value> = Vec::new();

            for (page_num, _) in doc.get_pages().iter() {
                match doc.extract_text(&[*page_num]) {
                    Ok(text) => {
                        pages.push(json!({
                            "page": page_num,
                            "text": text.trim()
                        }));
                    }
                    Err(_) => {
                        pages.push(json!({
                            "page": page_num,
                            "text": ""
                        }));
                    }
                }
            }

            json!({
                "pages": pages,
                "page_count": page_count
            })
        }
        Err(e) => {
            json!({
                "error": format!("failed to parse PDF: {}", e)
            })
        }
    }
}