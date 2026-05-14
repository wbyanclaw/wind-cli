//! Magic bytes detection for file format identification

use crate::extract::ExtractFormat;
use std::fmt;

/// Confidence level of format detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionConfidence {
    Extension,
    MagicBytes,
}

impl fmt::Display for DetectionConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionConfidence::Extension => write!(f, "extension"),
            DetectionConfidence::MagicBytes => write!(f, "magic_bytes"),
        }
    }
}

/// Detect file format from bytes and optional extension hint
pub fn detect_format(
    data: &[u8],
    extension: Option<&str>,
    force_format: Option<ExtractFormat>,
) -> anyhow::Result<(ExtractFormat, DetectionConfidence, Vec<u8>)> {
    // If format is forced, use it with extension confidence
    if let Some(fmt) = force_format {
        return Ok((fmt, DetectionConfidence::Extension, data.to_vec()));
    }

    // Try extension first
    if let Some(ext) = extension {
        if let Some(fmt) = format_from_extension(ext) {
            // Verify magic bytes match if possible
            if verify_format(data, fmt) {
                return Ok((fmt, DetectionConfidence::Extension, data.to_vec()));
            }
        }
    }

    // Fall back to magic bytes
    if let Some(fmt) = format_from_magic(data) {
        return Ok((fmt, DetectionConfidence::MagicBytes, data.to_vec()));
    }

    anyhow::bail!("unsupported file format: unable to detect from extension or content")
}

/// Map file extension to format
pub fn format_from_extension(ext: &str) -> Option<ExtractFormat> {
    match ext.to_lowercase().as_str() {
        "md" | "markdown" => Some(ExtractFormat::Md),
        "html" | "htm" => Some(ExtractFormat::Html),
        "pdf" => Some(ExtractFormat::Pdf),
        "xlsx" | "xls" => Some(ExtractFormat::Xlsx),
        "pptx" => Some(ExtractFormat::Pptx),
        "png" | "jpg" | "jpeg" => Some(ExtractFormat::Img),
        _ => None,
    }
}

/// Detect format from magic bytes
pub fn format_from_magic(data: &[u8]) -> Option<ExtractFormat> {
    if data.len() < 4 {
        return None;
    }

    // PDF
    if data.starts_with(b"%PDF") {
        return Some(ExtractFormat::Pdf);
    }

    // PNG
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return Some(ExtractFormat::Img);
    }

    // JPEG
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ExtractFormat::Img);
    }

    // ZIP-based formats (XLSX, PPTX)
    if data.starts_with(b"PK") {
        // Further distinguish by looking at zip entries
        // XLSX contains "xl/" prefix, PPTX contains "ppt/" prefix
        // For now, default to XLSX (more common)
        // A more robust implementation would inspect the zip central directory
        return Some(ExtractFormat::Xlsx);
    }

    // HTML (starts with <)
    if data.first() == Some(&b'<') {
        return Some(ExtractFormat::Html);
    }

    None
}

/// Verify that magic bytes match the expected format
fn verify_format(data: &[u8], format: ExtractFormat) -> bool {
    match format {
        ExtractFormat::Md => {
            // Markdown: plain text or starts with YAML frontmatter
            data.iter().all(|&b| b.is_ascii() || b == b'\n' || b == b'\r')
        }
        ExtractFormat::Html => {
            // HTML: starts with <!DOCTYPE, <html, <head, <body, etc.
            let start: Vec<u8> = data.iter().take(20).copied().collect();
            let text = String::from_utf8_lossy(&start).to_lowercase();
            text.starts_with("<!doctype") || text.starts_with("<html") || text.starts_with("<head")
                || text.starts_with("<body") || text.starts_with("<!")
        }
        ExtractFormat::Pdf => data.starts_with(b"%PDF"),
        ExtractFormat::Xlsx => data.starts_with(b"PK"),
        ExtractFormat::Pptx => data.starts_with(b"PK"),
        ExtractFormat::Img => {
            data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) || data.starts_with(&[0xFF, 0xD8, 0xFF])
        }
    }
}