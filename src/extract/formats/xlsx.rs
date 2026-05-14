//! Excel (.xlsx/.xls) extraction using calamine

use calamine::{open_workbook, Data, Reader, Xlsx};
use serde_json::{json, Value};

/// Extract data from Excel files
pub fn extract(data: &[u8], tabular: bool) -> Value {
    // Write data to a temporary file for calamine to read
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("windcli_extract_temp.xlsx");

    if let Err(e) = std::fs::write(&temp_path, data) {
        return json!({
            "error": format!("failed to write temp file: {}", e)
        });
    }

    let path_str = temp_path.to_string_lossy().to_string();

    match open_workbook::<Xlsx<std::io::BufReader<std::fs::File>>, _>(&path_str) {
        Ok(mut workbook) => {
            // Clean up temp file
            let _ = std::fs::remove_file(&temp_path);

            let sheet_names = workbook.sheet_names().to_vec();
            let mut sheets: Vec<Value> = Vec::new();

            for name in &sheet_names {
                let range = workbook.worksheet_range(name);
                match range {
                    Ok(worksheet) => {
                        let content = extract_range_data(&worksheet, tabular);
                        sheets.push(json!({
                            "name": name,
                            "rows": content
                        }));
                    }
                    Err(e) => {
                        sheets.push(json!({
                            "name": name,
                            "error": format!("{}", e)
                        }));
                    }
                }
            }

            json!({
                "sheets": sheets
            })
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            json!({
                "error": format!("failed to open workbook: {}", e)
            })
        }
    }
}

fn extract_range_data(range: &calamine::Range<Data>, tabular: bool) -> Value {
    let mut sheet_data: Vec<Vec<Value>> = Vec::new();

    for row in range.rows() {
        let row_data: Vec<Value> = row.iter().map(cell_to_json).collect();
        sheet_data.push(row_data);
    }

    // Filter empty rows
    sheet_data.retain(|row| !row.iter().all(|v| matches!(v, json!(null))));

    if tabular && sheet_data.len() > 1 {
        // Convert to dict format: {header: value, ...}
        let headers = &sheet_data[0];
        let data_rows: Vec<serde_json::Map<String, Value>> = sheet_data[1..]
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, cell) in row.iter().enumerate() {
                    let key = headers
                        .get(i)
                        .and_then(|h| h.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("col_{}", i));
                    map.insert(key, cell.clone());
                }
                map
            })
            .collect();
        serde_json::to_value(data_rows).unwrap_or(json!([]))
    } else {
        serde_json::to_value(&sheet_data).unwrap_or(json!([]))
    }
}

fn cell_to_json(cell: &Data) -> Value {
    match cell {
        Data::Empty => json!(null),
        Data::String(s) => json!(s),
        Data::Float(f) => json!(f),
        Data::Int(i) => json!(i),
        Data::Bool(b) => json!(b),
        Data::DateTime(ref dt) => {
            // Use as_datetime() to get chrono::NaiveDateTime (requires "dates" feature)
            match dt.as_datetime() {
                Some(naive) => json!(naive.format("%Y-%m-%d %H:%M:%S").to_string()),
                None => json!(format!("{}", dt)),
            }
        }
        Data::DateTimeIso(s) => json!(s),
        Data::DurationIso(s) => json!(s),
        Data::Error(e) => json!({
            "error": format!("{:?}", e)
        }),
    }
}