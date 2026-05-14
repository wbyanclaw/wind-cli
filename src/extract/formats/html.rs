//! HTML text extraction using scraper

use serde_json::json;

/// Extract text content from HTML
pub fn extract(data: &[u8]) -> serde_json::Value {
    let html_str = String::from_utf8_lossy(data);

    // Parse HTML and extract text
    let title = extract_title(&html_str);
    let text = extract_text(&html_str);

    json!({
        "title": title,
        "text": text
    })
}

fn extract_title(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Try <title> tag first
    if let Ok(selector) = Selector::parse("title") {
        if let Some(element) = document.select(&selector).next() {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return title;
            }
        }
    }

    // Fall back to first <h1>
    if let Ok(selector) = Selector::parse("h1") {
        if let Some(element) = document.select(&selector).next() {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return title;
            }
        }
    }

    String::new()
}

fn extract_text(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Remove script and style elements
    let mut text_parts = Vec::new();

    // Extract from paragraphs, headings, list items, etc.
    let selectors = ["p", "h1", "h2", "h3", "h4", "h5", "h6", "li", "td", "th", "div", "span", "a"];

    for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    text_parts.push(text);
                }
            }
        }
    }

    text_parts.join("\n")
}