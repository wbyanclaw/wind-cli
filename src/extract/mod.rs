//! `extract` command — multi-format document content extraction
//!
//! Phase 2 feature: extracts text/content from PDF, Excel, PowerPoint, Markdown,
//! HTML, and image files. All output is JSON for AI-agent consumption.

pub mod formats;
pub mod magic;

use serde::{Deserialize, Serialize};

/// Supported extract formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractFormat {
    Md,
    Html,
    Pdf,
    Xlsx,
    Pptx,
    Img,
}

/// Output envelope for all extract operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractOutput {
    pub ok: bool,
    #[serde(rename = "file")]
    pub file_name: String,
    #[serde(rename = "size_bytes")]
    pub size_bytes: u64,
    #[serde(rename = "format")]
    pub format: String,
    #[serde(rename = "format_confidence")]
    pub format_confidence: String,
    #[serde(rename = "content")]
    pub content: serde_json::Value,
    #[serde(rename = "warnings")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Core extract function — detects format and delegates to format-specific extractor
pub fn extract(
    path: &std::path::Path,
    force_format: Option<ExtractFormat>,
    include_base64: bool,
    tabular: bool,
) -> anyhow::Result<ExtractOutput> {
    use std::fs;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let data = fs::read(path)?;
    let size_bytes = data.len() as u64;

    // Detect format
    let (format, confidence, detected_data) =
        magic::detect_format(&data, path.extension().and_then(|e| e.to_str()), force_format)?;

    // Dispatch to format-specific extractor
    let content = match format {
        ExtractFormat::Md => formats::md::extract(&data),
        ExtractFormat::Html => formats::html::extract(&data),
        ExtractFormat::Pdf => formats::pdf::extract(&data),
        ExtractFormat::Xlsx => formats::xlsx::extract(&data, tabular),
        ExtractFormat::Pptx => formats::pptx::extract(&data),
        ExtractFormat::Img => formats::img::extract(&data, include_base64),
    };

    let format_str = match format {
        ExtractFormat::Md => "md",
        ExtractFormat::Html => "html",
        ExtractFormat::Pdf => "pdf",
        ExtractFormat::Xlsx => "xlsx",
        ExtractFormat::Pptx => "pptx",
        ExtractFormat::Img => "img",
    };

    Ok(ExtractOutput {
        ok: true,
        file_name,
        size_bytes,
        format: format_str.to_string(),
        format_confidence: confidence.to_string(),
        content,
        warnings: vec![],
    })
}