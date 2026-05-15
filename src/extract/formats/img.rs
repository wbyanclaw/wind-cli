//! Image metadata extraction using `image` crate

use base64::Engine;
use serde_json::{json, Value};

/// Extract image metadata and optionally base64 content
pub fn extract(data: &[u8], include_base64: bool) -> Value {
    // Decode image to get metadata
    let reader = image::ImageReader::new(std::io::Cursor::new(data));

    match reader.with_guessed_format() {
        Ok(reader) => {
            match reader.decode() {
                Ok(img) => {
                    let width = img.width();
                    let height = img.height();
                    let color_type = format!("{:?}", img.color());
                    let bit_depth = img.color().bits_per_pixel() / 3; // Approximate

                    let mut content = json!({
                        "width_px": width,
                        "height_px": height,
                        "color_type": color_type,
                        "bit_depth": bit_depth,
                        "format": format!("{:?}", img.color()),
                        "has_alpha": img.color().has_alpha(),
                        "base64": Value::Null
                    });

                    if include_base64 {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(data);
                        content["base64"] = json!(b64);
                    }

                    content
                }
                Err(_) => {
                    // Fall back to basic metadata from file
                    json!({
                        "width_px": 0,
                        "height_px": 0,
                        "error": "could not decode image"
                    })
                }
            }
        }
        Err(_) => {
            json!({
                "width_px": 0,
                "height_px": 0,
                "error": "unknown image format"
            })
        }
    }
}