//! Iteration data for the collection runner, read from a CSV or JSON file.
use std::path::Path;

use serde_json::Value;

use crate::error::{AppError, AppResult};

const MAX_BYTES: u64 = 10 * 1024 * 1024;
const MAX_ROWS: usize = 10_000;

/// One row per iteration; each row holds its column values by name.
#[derive(Debug, Default, PartialEq)]
pub struct RunData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<(String, String)>>,
}

fn invalid(reason: impl std::fmt::Display) -> AppError {
    AppError::Validation(format!("File data tidak bisa dibaca: {reason}"))
}

fn text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// CSV with a header row, or a JSON array of objects.
pub fn parse(content: &str, json: bool) -> AppResult<RunData> {
    let mut data = RunData::default();
    if json {
        let rows: Vec<serde_json::Map<String, Value>> =
            serde_json::from_str(content).map_err(|_| {
                invalid("JSON harus berupa array objek, misalnya [{\"email\": \"...\"}]")
            })?;
        for row in rows {
            for key in row.keys() {
                if !data.columns.contains(key) {
                    data.columns.push(key.clone());
                }
            }
            data.rows
                .push(row.iter().map(|(k, v)| (k.clone(), text(v))).collect());
        }
    } else {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::Headers)
            .from_reader(content.as_bytes());
        data.columns = reader
            .headers()
            .map_err(invalid)?
            .iter()
            .map(str::to_string)
            .collect();
        for record in reader.records() {
            let record = record.map_err(invalid)?;
            let row = data
                .columns
                .iter()
                .zip(record.iter())
                .map(|(k, v)| (k.clone(), v.to_string()));
            data.rows.push(row.filter(|(k, _)| !k.is_empty()).collect());
        }
    }
    data.columns.retain(|c| !c.trim().is_empty());
    if data.rows.is_empty() {
        return Err(invalid("tidak ada baris data"));
    }
    if data.rows.len() > MAX_ROWS {
        return Err(invalid(format!("maksimal {MAX_ROWS} baris")));
    }
    Ok(data)
}

pub fn load(path: &str) -> AppResult<RunData> {
    if std::fs::metadata(path)?.len() > MAX_BYTES {
        return Err(invalid("ukuran file maksimal 10 MB"));
    }
    let content = std::fs::read_to_string(path)?;
    let json = Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));
    parse(content.trim_start_matches('\u{feff}'), json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_csv_and_json_rows() {
        let csv = parse(
            "email,outletId\nleader.a@contoh.id,242\n\"b, c\"@contoh.id,118\n",
            false,
        )
        .unwrap();
        assert_eq!(csv.columns, ["email", "outletId"]);
        assert_eq!(
            csv.rows[1],
            [
                ("email".to_string(), "b, c@contoh.id".to_string()),
                ("outletId".into(), "118".into())
            ]
        );
        let json = parse(
            r#"[{"email":"a@contoh.id","age":30},{"email":"b@contoh.id","admin":true}]"#,
            true,
        )
        .unwrap();
        assert_eq!(json.columns, ["email", "age", "admin"]);
        assert_eq!(json.rows[0][1], ("age".to_string(), "30".to_string()));
        assert!(parse("email\n", false).is_err());
        assert!(parse(r#"{"email":"a"}"#, true).is_err());
    }
}
