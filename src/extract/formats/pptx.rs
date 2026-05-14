//! PowerPoint (.pptx) extraction using zip + XML parsing

use serde_json::{json, Value};
use std::io::Read;
use zip::ZipArchive;

/// Extract text from PowerPoint presentations
pub fn extract(data: &[u8]) -> Value {
    let cursor = std::io::Cursor::new(data);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => {
            return json!({
                "error": format!("failed to read zip archive: {}", e)
            });
        }
    };

    let mut slides: Vec<Value> = Vec::new();
    let slide_count;

    // Collect slide files
    let mut slide_files: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            // PPTX slides are in ppt/slides/slideN.xml
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_files.push(name);
            }
        }
    }

    // Sort by slide number
    slide_files.sort();
    slide_count = slide_files.len();

    for slide_path in slide_files {
        let mut slide_xml = String::new();
        if let Ok(mut file) = archive.by_name(&slide_path) {
            let _ = file.read_to_string(&mut slide_xml);
        }

        // Parse slide number from path
        let slide_num = slide_path
            .replace("ppt/slides/slide", "")
            .replace(".xml", "")
            .parse::<u32>()
            .unwrap_or(0);

        let (title, text) = extract_slide_text(&slide_xml);

        slides.push(json!({
            "slide_number": slide_num,
            "title": title,
            "text": text
        }));
    }

    json!({
        "slides": slides,
        "slide_count": slide_count
    })
}

fn extract_slide_text(xml: &str) -> (String, String) {
    // Use roxmltree to parse XML
    let doc = match roxmltree::Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return (String::new(), String::new()),
    };

    let mut title = String::new();
    let mut all_text = Vec::new();

    // Extract all text nodes
    for node in doc.descendants() {
        // roxmltree: Text nodes are Node::Text with the text directly
        if node.is_text() {
            let text = node.text().map(|t| t.trim()).unwrap_or("");
            if !text.is_empty() {
                if title.is_empty() {
                    title = text.to_string();
                }
                all_text.push(text.to_string());
            }
        }
    }

    (title, all_text.join(" "))
}